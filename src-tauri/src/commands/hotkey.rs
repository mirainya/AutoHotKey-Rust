use rdev::{listen, Event, EventType};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::Emitter;
use crate::error::AppError;

static HOTKEY_ACTIVE: AtomicBool = AtomicBool::new(false);
static LISTENER_STARTED: AtomicBool = AtomicBool::new(false);
static HOTKEY_MAP: Mutex<Option<HashMap<String, String>>> = Mutex::new(None);

#[tauri::command]
pub fn register_hotkey(hotkey: String, script_id: String) -> Result<(), AppError> {
    tracing::info!("注册热键: {} -> {}", hotkey, script_id);
    let mut map = HOTKEY_MAP.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    if map.is_none() {
        *map = Some(HashMap::new());
    }
    let m = map.as_mut().unwrap();
    // 冲突检测：如果热键已被其他脚本占用，返回错误
    if let Some(existing_id) = m.get(&hotkey) {
        if existing_id != &script_id {
            return Err(AppError::Custom(format!(
                "热键 {} 已被脚本 {} 占用", hotkey, existing_id
            )));
        }
    }
    m.insert(hotkey, script_id);
    Ok(())
}

#[tauri::command]
pub fn unregister_hotkey(hotkey: String) -> Result<(), AppError> {
    tracing::info!("注销热键: {}", hotkey);
    let mut map = HOTKEY_MAP.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    if let Some(m) = map.as_mut() {
        m.remove(&hotkey);
    }
    Ok(())
}

#[tauri::command]
pub fn start_hotkey_listener(app: tauri::AppHandle) -> Result<(), AppError> {
    // 如果已经激活，直接返回
    if HOTKEY_ACTIVE.load(Ordering::SeqCst) {
        tracing::warn!("热键监听器已在运行中");
        return Ok(());
    }

    HOTKEY_ACTIVE.store(true, Ordering::SeqCst);

    // 只启动一次监听线程，后续 start/stop 只切换 flag
    if LISTENER_STARTED.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
        std::thread::spawn(move || {
            let mut ctrl_pressed = false;
            let mut shift_pressed = false;
            let mut alt_pressed = false;

            let _ = listen(move |event: Event| {
                // flag 未激活时跳过处理
                if !HOTKEY_ACTIVE.load(Ordering::Relaxed) {
                    return;
                }

                match event.event_type {
                    EventType::KeyPress(key) => {
                        let key_str = format!("{:?}", key);

                        if key_str == "ControlLeft" || key_str == "ControlRight" {
                            ctrl_pressed = true;
                        } else if key_str == "ShiftLeft" || key_str == "ShiftRight" {
                            shift_pressed = true;
                        } else if key_str == "Alt" || key_str == "AltGr" {
                            alt_pressed = true;
                        } else {
                            let mut hotkey_parts = Vec::new();
                            if ctrl_pressed { hotkey_parts.push("Ctrl"); }
                            if shift_pressed { hotkey_parts.push("Shift"); }
                            if alt_pressed { hotkey_parts.push("Alt"); }

                            let main_key = key_str.trim_start_matches("Key");
                            hotkey_parts.push(main_key);

                            let hotkey = hotkey_parts.join("+");
                            tracing::debug!("检测到按键: {} (原始: {})", hotkey, key_str);

                            if let Ok(map) = HOTKEY_MAP.try_lock() {
                                if let Some(m) = map.as_ref() {
                                    if let Some(script_id) = m.get(&hotkey) {
                                        tracing::info!("触发热键: {} -> {}", hotkey, script_id);
                                        let _ = app.emit("hotkey-triggered", (hotkey.clone(), script_id.clone()));
                                    }
                                }
                            }
                        }
                    }
                    EventType::KeyRelease(key) => {
                        let key_str = format!("{:?}", key);
                        if key_str == "ControlLeft" || key_str == "ControlRight" {
                            ctrl_pressed = false;
                        } else if key_str == "ShiftLeft" || key_str == "ShiftRight" {
                            shift_pressed = false;
                        } else if key_str == "Alt" || key_str == "AltGr" {
                            alt_pressed = false;
                        }
                    }
                    _ => {}
                }
            });
        });

        tracing::info!("热键监听线程已启动");
    }

    tracing::info!("热键监听器已激活");
    Ok(())
}

#[tauri::command]
pub fn stop_hotkey_listener() -> Result<(), AppError> {
    HOTKEY_ACTIVE.store(false, Ordering::SeqCst);
    tracing::info!("热键监听器已停止");
    Ok(())
}
