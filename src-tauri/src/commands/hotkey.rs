use rdev::{listen, Event, EventType};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::Emitter;

static HOTKEY_ENABLED: Mutex<bool> = Mutex::new(false);
static HOTKEY_MAP: Mutex<Option<HashMap<String, String>>> = Mutex::new(None);

#[tauri::command]
pub fn register_hotkey(hotkey: String, script_id: String) -> Result<(), String> {
    println!("注册热键: {} -> {}", hotkey, script_id);
    let mut map = HOTKEY_MAP.lock().unwrap();
    if map.is_none() {
        *map = Some(HashMap::new());
    }
    map.as_mut().unwrap().insert(hotkey, script_id);
    Ok(())
}

#[tauri::command]
pub fn unregister_hotkey(hotkey: String) -> Result<(), String> {
    let mut map = HOTKEY_MAP.lock().unwrap();
    if let Some(m) = map.as_mut() {
        m.remove(&hotkey);
    }
    Ok(())
}

#[tauri::command]
pub fn start_hotkey_listener(app: tauri::AppHandle) -> Result<(), String> {
    let mut enabled = HOTKEY_ENABLED.lock().unwrap();
    if *enabled {
        return Ok(());
    }
    *enabled = true;
    drop(enabled);

    std::thread::spawn(move || {
        let mut ctrl_pressed = false;
        let mut shift_pressed = false;
        let mut alt_pressed = false;

        let _ = listen(move |event: Event| {
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
                        println!("检测到按键: {} (原始: {})", hotkey, key_str);

                        let map = HOTKEY_MAP.lock().unwrap();
                        if let Some(m) = map.as_ref() {
                            println!("当前注册的热键: {:?}", m.keys().collect::<Vec<_>>());
                            if let Some(script_id) = m.get(&hotkey) {
                                println!("触发热键: {} -> {}", hotkey, script_id);
                                let _ = app.emit("hotkey-triggered", (hotkey.clone(), script_id.clone()));
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

    Ok(())
}

#[tauri::command]
pub fn stop_hotkey_listener() -> Result<(), String> {
    let mut enabled = HOTKEY_ENABLED.lock().unwrap();
    *enabled = false;
    Ok(())
}
