use rquickjs::{Context, Runtime};
use rquickjs::function::Func;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::Manager;
use tauri::Emitter;
use crate::error::AppError;

#[cfg(target_os = "windows")]
use super::window;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    pub id: String,
    pub name: String,
    pub code: String,
    pub hotkey: Option<String>,
    pub enabled: bool,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScriptStatus {
    Idle,
    Running,
    Error(String),
}

static ABORT_FLAG: Mutex<Option<Arc<AtomicBool>>> = Mutex::new(None);
static SCRIPT_STATUS: Mutex<ScriptStatus> = Mutex::new(ScriptStatus::Idle);

fn get_scripts_dir(app: &tauri::AppHandle) -> Result<PathBuf, AppError> {
    let mut path = app.path().app_data_dir().map_err(|e| AppError::Io(e.to_string()))?;
    path.push("scripts");
    if !path.exists() {
        fs::create_dir_all(&path)?;
    }
    Ok(path)
}

#[tauri::command]
pub fn save_script(app: tauri::AppHandle, script: Script) -> Result<(), AppError> {
    let path = get_scripts_dir(&app)?.join(format!("{}.json", script.id));
    let json = serde_json::to_string_pretty(&script)?;
    fs::write(path, json)?;
    tracing::info!("脚本已保存: {} ({})", script.name, script.id);
    Ok(())
}

#[tauri::command]
pub fn load_scripts(app: tauri::AppHandle) -> Result<Vec<Script>, AppError> {
    let dir = get_scripts_dir(&app)?;
    let mut scripts = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                if let Ok(script) = serde_json::from_str::<Script>(&content) {
                    scripts.push(script);
                }
            }
        }
    }

    Ok(scripts)
}

#[tauri::command]
pub fn delete_script(app: tauri::AppHandle, id: String) -> Result<(), AppError> {
    let path = get_scripts_dir(&app)?.join(format!("{}.json", id));
    fs::remove_file(path)?;
    tracing::info!("脚本已删除: {}", id);
    Ok(())
}

#[tauri::command]
pub fn stop_script() -> Result<(), AppError> {
    if let Ok(flag_guard) = ABORT_FLAG.lock() {
        if let Some(flag) = flag_guard.as_ref() {
            flag.store(true, Ordering::Relaxed);
            tracing::info!("脚本停止信号已发送");
        }
    }
    Ok(())
}

#[tauri::command]
pub fn get_script_status() -> Result<ScriptStatus, AppError> {
    let status = SCRIPT_STATUS.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    Ok(status.clone())
}

#[tauri::command]
pub fn run_script_by_id(app: tauri::AppHandle, id: String) -> Result<(), AppError> {
    let dir = get_scripts_dir(&app)?;
    let path = dir.join(format!("{}.json", id));
    let content = fs::read_to_string(&path)?;
    let script: Script = serde_json::from_str(&content)?;
    tracing::info!("按ID运行脚本: {} ({})", script.name, id);
    execute_script(app, script.code, None)
}

#[tauri::command]
pub fn export_script(app: tauri::AppHandle, id: String, path: String) -> Result<(), AppError> {
    let dir = get_scripts_dir(&app)?;
    let src = dir.join(format!("{}.json", id));
    let content = fs::read_to_string(&src)?;
    let script: Script = serde_json::from_str(&content)?;
    // 导出为 .js 文件（纯代码）
    fs::write(&path, &script.code)?;
    tracing::info!("脚本已导出: {} -> {}", script.name, path);
    Ok(())
}

#[tauri::command]
pub fn import_script(app: tauri::AppHandle, path: String, name: String) -> Result<Script, AppError> {
    let code = fs::read_to_string(&path)?;
    let script = Script {
        id: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .to_string(),
        name,
        code,
        hotkey: None,
        enabled: false,
        tags: Vec::new(),
    };
    let dest = get_scripts_dir(&app)?.join(format!("{}.json", script.id));
    let json = serde_json::to_string_pretty(&script)?;
    fs::write(dest, json)?;
    tracing::info!("脚本已导入: {}", script.name);
    Ok(script)
}

#[tauri::command]
pub fn execute_script(app: tauri::AppHandle, code: String, timeout: Option<u64>) -> Result<(), AppError> {
    // 设置状态为运行中
    if let Ok(mut status) = SCRIPT_STATUS.lock() {
        *status = ScriptStatus::Running;
    }

    let abort = Arc::new(AtomicBool::new(false));
    if let Ok(mut flag) = ABORT_FLAG.lock() {
        *flag = Some(abort.clone());
    }

    // 超时守护线程
    if let Some(ms) = timeout {
        if ms > 0 {
            let abort_clone = abort.clone();
            let app_clone = app.clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(ms));
                if !abort_clone.load(Ordering::Relaxed) {
                    abort_clone.store(true, Ordering::Relaxed);
                    tracing::warn!("脚本执行超时 ({}ms)，已自动终止", ms);
                    let _ = app_clone.emit("script-output", format!("[超时] 脚本执行超过 {}ms，已自动终止", ms));
                }
            });
        }
    }

    std::thread::spawn(move || {
        let runtime = match Runtime::new() {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("创建JS运行时失败: {}", e);
                let _ = app.emit("script-log", serde_json::json!({
                    "scriptName": "unknown",
                    "time": chrono_now(),
                    "result": format!("运行时创建失败: {e}"),
                    "success": false
                }));
                if let Ok(mut status) = SCRIPT_STATUS.lock() {
                    *status = ScriptStatus::Error(e.to_string());
                }
                return;
            }
        };

        // 设置内存限制 (64MB)
        runtime.set_memory_limit(64 * 1024 * 1024);

        let context = match Context::full(&runtime) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("创建JS上下文失败: {}", e);
                let _ = app.emit("script-log", serde_json::json!({
                    "scriptName": "unknown",
                    "time": chrono_now(),
                    "result": format!("上下文创建失败: {e}"),
                    "success": false
                }));
                if let Ok(mut status) = SCRIPT_STATUS.lock() {
                    *status = ScriptStatus::Error(e.to_string());
                }
                return;
            }
        };

        let result = context.with(|ctx| {
            let globals = ctx.globals();
            if let Err(e) = register_api(&globals, &app, &abort) {
                return Err(format!("API注册失败: {e}"));
            }
            match ctx.eval::<rquickjs::Value, _>(code.as_str()) {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("{e}")),
            }
        });

        match &result {
            Ok(_) => {
                tracing::info!("脚本执行完成");
                let _ = app.emit("script-log", serde_json::json!({
                    "scriptName": "script",
                    "time": chrono_now(),
                    "result": "执行完成",
                    "success": true
                }));
            }
            Err(e) => {
                tracing::error!("脚本执行失败: {}", e);
                let _ = app.emit("script-log", serde_json::json!({
                    "scriptName": "script",
                    "time": chrono_now(),
                    "result": e,
                    "success": false
                }));
            }
        }

        // 重置状态
        if let Ok(mut status) = SCRIPT_STATUS.lock() {
            *status = match &result {
                Ok(_) => ScriptStatus::Idle,
                Err(e) => ScriptStatus::Error(e.clone()),
            };
        }
    });

    Ok(())
}

fn chrono_now() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    format!("{}", now)
}

/// 沙箱化文件路径：只允许访问 app_data_dir/scripts/ 下的文件
fn sandbox_path(app: &tauri::AppHandle, path: &str) -> Result<PathBuf, String> {
    // 拒绝路径穿越
    if path.contains("..") {
        return Err("路径不允许包含 '..'".to_string());
    }

    let base = app.path().app_data_dir()
        .map_err(|e| e.to_string())?
        .join("scripts");

    let target = base.join(path);
    let canonical_base = base.canonicalize().unwrap_or_else(|_| base.clone());
    let canonical_target = if target.exists() {
        target.canonicalize().map_err(|e| e.to_string())?
    } else {
        // 文件不存在时，规范化父目录
        if let Some(parent) = target.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let canonical_parent = parent.canonicalize().map_err(|e| e.to_string())?;
            canonical_parent.join(target.file_name().ok_or("无效文件名")?)
        } else {
            return Err("无效路径".to_string());
        }
    };

    if !canonical_target.starts_with(&canonical_base) {
        return Err(format!("路径超出沙箱范围: {}", path));
    }

    Ok(canonical_target)
}

fn register_api(
    globals: &rquickjs::Object,
    app: &tauri::AppHandle,
    abort: &Arc<AtomicBool>,
) -> Result<(), String> {
    use enigo::{Enigo, Mouse, Keyboard, Settings, Coordinate, Button, Direction};
    use super::pixel;

    // print
    let app_clone = app.clone();
    globals.set("print", Func::from(move |msg: String| {
        tracing::info!("[脚本输出] {}", msg);
        let _ = app_clone.emit("script-output", msg);
    })).map_err(|e| e.to_string())?;

    // sleep
    let abort_clone = abort.clone();
    globals.set("sleep", Func::from(move |ms: u64| {
        let step = 50u64;
        let mut remaining = ms;
        while remaining > 0 && !abort_clone.load(Ordering::Relaxed) {
            let wait = remaining.min(step);
            std::thread::sleep(std::time::Duration::from_millis(wait));
            remaining = remaining.saturating_sub(wait);
        }
    })).map_err(|e| e.to_string())?;

    // isStopped
    let abort_clone = abort.clone();
    globals.set("isStopped", Func::from(move || -> bool {
        abort_clone.load(Ordering::Relaxed)
    })).map_err(|e| e.to_string())?;

    // screenshot
    let app_clone = app.clone();
    globals.set("screenshot", Func::from(move || -> String {
        match super::screenshot::capture_screen(app_clone.clone()) {
            Ok(path) => path,
            Err(e) => { tracing::error!("截图失败: {}", e); String::new() }
        }
    })).map_err(|e| e.to_string())?;

    // moveMouse
    let app_print = app.clone();
    globals.set("moveMouse", Func::from(move |x: i32, y: i32| {
        match Enigo::new(&Settings::default()) {
            Ok(mut enigo) => {
                if let Err(e) = enigo.move_mouse(x, y, Coordinate::Abs) {
                    let _ = app_print.emit("script-output", format!("[moveMouse] 失败: {e}"));
                }
            }
            Err(e) => { let _ = app_print.emit("script-output", format!("[moveMouse] 初始化失败: {e}")); }
        }
    })).map_err(|e| e.to_string())?;

    // clickMouse
    let app_print = app.clone();
    globals.set("clickMouse", Func::from(move |x: i32, y: i32, button: String| {
        match Enigo::new(&Settings::default()) {
            Ok(mut enigo) => {
                if let Err(e) = enigo.move_mouse(x, y, Coordinate::Abs) {
                    let _ = app_print.emit("script-output", format!("[clickMouse] 移动失败: {e}"));
                    return;
                }
                let btn = match button.as_str() {
                    "right" => Button::Right,
                    "middle" => Button::Middle,
                    _ => Button::Left,
                };
                if let Err(e) = enigo.button(btn, Direction::Click) {
                    let _ = app_print.emit("script-output", format!("[clickMouse] 点击失败: {e}"));
                }
            }
            Err(e) => { let _ = app_print.emit("script-output", format!("[clickMouse] 初始化失败: {e}")); }
        }
    })).map_err(|e| e.to_string())?;

    // scrollMouse
    let app_print = app.clone();
    globals.set("scrollMouse", Func::from(move |x: i32, y: i32, delta: i32| {
        match Enigo::new(&Settings::default()) {
            Ok(mut enigo) => {
                if let Err(e) = enigo.move_mouse(x, y, Coordinate::Abs) {
                    let _ = app_print.emit("script-output", format!("[scrollMouse] 移动失败: {e}"));
                    return;
                }
                if let Err(e) = enigo.scroll(delta, enigo::Axis::Vertical) {
                    let _ = app_print.emit("script-output", format!("[scrollMouse] 滚动失败: {e}"));
                }
            }
            Err(e) => { let _ = app_print.emit("script-output", format!("[scrollMouse] 初始化失败: {e}")); }
        }
    })).map_err(|e| e.to_string())?;

    // typeText
    let app_print = app.clone();
    globals.set("typeText", Func::from(move |text: String| {
        match Enigo::new(&Settings::default()) {
            Ok(mut enigo) => {
                if let Err(e) = enigo.text(&text) {
                    let _ = app_print.emit("script-output", format!("[typeText] 失败: {e}"));
                }
            }
            Err(e) => { let _ = app_print.emit("script-output", format!("[typeText] 初始化失败: {e}")); }
        }
    })).map_err(|e| e.to_string())?;

    // keyPress
    let app_print = app.clone();
    globals.set("keyPress", Func::from(move |key: String| {
        match Enigo::new(&Settings::default()) {
            Ok(mut enigo) => {
                match super::automation::parse_key_public(&key) {
                    Ok(k) => { if let Err(e) = enigo.key(k, Direction::Click) { let _ = app_print.emit("script-output", format!("[keyPress] 失败: {e}")); } }
                    Err(e) => { let _ = app_print.emit("script-output", format!("[keyPress] 未知按键 '{key}': {e}")); }
                }
            }
            Err(e) => { let _ = app_print.emit("script-output", format!("[keyPress] 初始化失败: {e}")); }
        }
    })).map_err(|e| e.to_string())?;

    // keyDown
    let app_print = app.clone();
    globals.set("keyDown", Func::from(move |key: String| {
        match Enigo::new(&Settings::default()) {
            Ok(mut enigo) => {
                match super::automation::parse_key_public(&key) {
                    Ok(k) => { if let Err(e) = enigo.key(k, Direction::Press) { let _ = app_print.emit("script-output", format!("[keyDown] 失败: {e}")); } }
                    Err(e) => { let _ = app_print.emit("script-output", format!("[keyDown] 未知按键 '{key}': {e}")); }
                }
            }
            Err(e) => { let _ = app_print.emit("script-output", format!("[keyDown] 初始化失败: {e}")); }
        }
    })).map_err(|e| e.to_string())?;

    // keyUp
    let app_print = app.clone();
    globals.set("keyUp", Func::from(move |key: String| {
        match Enigo::new(&Settings::default()) {
            Ok(mut enigo) => {
                match super::automation::parse_key_public(&key) {
                    Ok(k) => { if let Err(e) = enigo.key(k, Direction::Release) { let _ = app_print.emit("script-output", format!("[keyUp] 失败: {e}")); } }
                    Err(e) => { let _ = app_print.emit("script-output", format!("[keyUp] 未知按键 '{key}': {e}")); }
                }
            }
            Err(e) => { let _ = app_print.emit("script-output", format!("[keyUp] 初始化失败: {e}")); }
        }
    })).map_err(|e| e.to_string())?;

    // getMousePos
    let app_print = app.clone();
    globals.set("getMousePos", Func::from(move || -> String {
        match Enigo::new(&Settings::default()) {
            Ok(enigo) => {
                match enigo.location() {
                    Ok((x, y)) => format!("{{\"x\":{},\"y\":{}}}", x, y),
                    Err(e) => {
                        let _ = app_print.emit("script-output", format!("[getMousePos] 失败: {e}"));
                        "{}".to_string()
                    }
                }
            }
            Err(e) => {
                let _ = app_print.emit("script-output", format!("[getMousePos] 初始化失败: {e}"));
                "{}".to_string()
            }
        }
    })).map_err(|e| e.to_string())?;

    // getScreenSize
    globals.set("getScreenSize", Func::from(|| -> String {
        let (w, h) = super::automation::get_screen_size();
        format!("{{\"width\":{},\"height\":{}}}", w, h)
    })).map_err(|e| e.to_string())?;

    // getPixelColor
    globals.set("getPixelColor", Func::from(|x: i32, y: i32| -> String {
        match pixel::get_pixel_color(x, y) {
            Ok(info) => format!("#{:02X}{:02X}{:02X}", info.r, info.g, info.b),
            Err(_) => String::new(),
        }
    })).map_err(|e| e.to_string())?;

    // findPattern
    let app_clone = app.clone();
    globals.set("findPattern", Func::from(move |name: String, tolerance: u8| -> String {
        if let Ok(patterns) = pixel::load_pixel_patterns(app_clone.clone()) {
            if let Some(pattern) = patterns.into_iter().find(|p| p.name == name) {
                if let Ok(results) = pixel::find_pixel_pattern(pattern, tolerance) {
                    let positions: Vec<(i32, i32)> = results.into_iter().map(|r| (r.x, r.y)).collect();
                    return serde_json::to_string(&positions).unwrap_or_else(|_| "[]".to_string());
                }
            }
        }
        "[]".to_string()
    })).map_err(|e| e.to_string())?;

    // findCustomPattern
    globals.set("findCustomPattern", Func::from(|pattern_json: String, tolerance: u8| -> String {
        if let Ok(pattern) = serde_json::from_str::<pixel::PixelPattern>(&pattern_json) {
            if let Ok(results) = pixel::find_pixel_pattern(pattern, tolerance) {
                let positions: Vec<(i32, i32)> = results.into_iter().map(|r| (r.x, r.y)).collect();
                return serde_json::to_string(&positions).unwrap_or_else(|_| "[]".to_string());
            }
        }
        "[]".to_string()
    })).map_err(|e| e.to_string())?;

    // waitForPattern
    let app_clone = app.clone();
    let abort_clone = abort.clone();
    globals.set("waitForPattern", Func::from(move |name: String, timeout_ms: u64| -> String {
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
        loop {
            if abort_clone.load(Ordering::Relaxed) { return "[]".to_string(); }
            if let Ok(patterns) = pixel::load_pixel_patterns(app_clone.clone()) {
                if let Some(pattern) = patterns.into_iter().find(|p| p.name == name) {
                    if let Ok(results) = pixel::find_pixel_pattern(pattern, 30) {
                        if !results.is_empty() {
                            let positions: Vec<(i32, i32)> = results.into_iter().map(|r| (r.x, r.y)).collect();
                            return serde_json::to_string(&positions).unwrap_or_else(|_| "[]".to_string());
                        }
                    }
                }
            }
            if std::time::Instant::now() >= deadline { return "[]".to_string(); }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    })).map_err(|e| e.to_string())?;

    // readFile (sandboxed to scripts/ directory)
    let app_clone = app.clone();
    globals.set("readFile", Func::from(move |path: String| -> String {
        match sandbox_path(&app_clone, &path) {
            Ok(safe_path) => std::fs::read_to_string(&safe_path).unwrap_or_else(|e| {
                tracing::error!("[readFile] 读取失败: {} - {}", path, e);
                String::new()
            }),
            Err(msg) => {
                tracing::error!("[readFile] 路径被拒绝: {} - {}", path, msg);
                String::new()
            }
        }
    })).map_err(|e| e.to_string())?;

    // writeFile (sandboxed to scripts/ directory)
    let app_clone = app.clone();
    globals.set("writeFile", Func::from(move |path: String, text: String| -> bool {
        match sandbox_path(&app_clone, &path) {
            Ok(safe_path) => std::fs::write(&safe_path, text).map_err(|e| {
                tracing::error!("[writeFile] 写入失败: {} - {}", path, e);
                e
            }).is_ok(),
            Err(msg) => {
                tracing::error!("[writeFile] 路径被拒绝: {} - {}", path, msg);
                false
            }
        }
    })).map_err(|e| e.to_string())?;

    // === Windows-specific APIs ===
    #[cfg(target_os = "windows")]
    {
        // findWindow
        globals.set("findWindow", Func::from(|title: String| -> i64 {
            window::find_window(title).unwrap_or(0) as i64
        })).map_err(|e| e.to_string())?;

        // findWindowByClass
        globals.set("findWindowByClass", Func::from(|class_name: String| -> i64 {
            window::find_window_by_class(class_name).unwrap_or(0) as i64
        })).map_err(|e| e.to_string())?;

        // getForegroundWindow
        globals.set("getForegroundWindow", Func::from(|| -> i64 {
            window::get_foreground_window().unwrap_or(0) as i64
        })).map_err(|e| e.to_string())?;

        // getWindowInfo -> returns JSON string
        globals.set("getWindowInfo", Func::from(|hwnd: i64| -> String {
            let info = window::get_window_info(hwnd as isize);
            serde_json::to_string(&info).unwrap_or_default()
        })).map_err(|e| e.to_string())?;

        // getWindowTitle
        globals.set("getWindowTitle", Func::from(|hwnd: i64| -> String {
            window::get_window_info(hwnd as isize).title
        })).map_err(|e| e.to_string())?;

        // activateWindow
        globals.set("activateWindow", Func::from(|hwnd: i64| {
            let _ = window::activate_window(hwnd as isize);
        })).map_err(|e| e.to_string())?;

        // moveWindow
        globals.set("moveWindow", Func::from(|hwnd: i64, x: i32, y: i32, w: i32, h: i32| {
            let _ = window::move_window(hwnd as isize, x, y, w, h);
        })).map_err(|e| e.to_string())?;

        // showWindow
        globals.set("showWindow", Func::from(|hwnd: i64, cmd: i32| {
            let _ = window::show_window(hwnd as isize, cmd);
        })).map_err(|e| e.to_string())?;

        // postClick
        globals.set("postClick", Func::from(|hwnd: i64, x: i32, y: i32, button: String| {
            let _ = window::post_click(hwnd as isize, x, y, button);
        })).map_err(|e| e.to_string())?;

        // postKey
        globals.set("postKey", Func::from(|hwnd: i64, vk_code: u32| {
            let _ = window::post_key(hwnd as isize, vk_code);
        })).map_err(|e| e.to_string())?;

        // postChar
        globals.set("postChar", Func::from(|hwnd: i64, ch: String| {
            let _ = window::post_char(hwnd as isize, ch);
        })).map_err(|e| e.to_string())?;

        // getClipboard
        globals.set("getClipboard", Func::from(|| -> String {
            window::get_clipboard().unwrap_or_default()
        })).map_err(|e| e.to_string())?;

        // setClipboard
        globals.set("setClipboard", Func::from(|text: String| {
            let _ = window::set_clipboard(text);
        })).map_err(|e| e.to_string())?;

        // sendKeys - 组合键支持，如 "Ctrl+A", "Alt+F4"
        globals.set("sendKeys", Func::from(|keys: String| {
            if let Ok(mut enigo) = Enigo::new(&Settings::default()) {
                let parts: Vec<&str> = keys.split('+').collect();
                let mut modifiers = Vec::new();
                for part in &parts[..parts.len().saturating_sub(1)] {
                    let modifier = match part.trim().to_lowercase().as_str() {
                        "ctrl" | "control" => Some(enigo::Key::Control),
                        "alt" => Some(enigo::Key::Alt),
                        "shift" => Some(enigo::Key::Shift),
                        "win" | "meta" => Some(enigo::Key::Meta),
                        _ => None,
                    };
                    if let Some(m) = modifier {
                        let _ = enigo.key(m, Direction::Press);
                        modifiers.push(m);
                    }
                }
                if let Some(main_key) = parts.last() {
                    if let Ok(k) = super::automation::parse_key_public(main_key.trim()) {
                        let _ = enigo.key(k, Direction::Click);
                    }
                }
                for m in modifiers.into_iter().rev() {
                    let _ = enigo.key(m, Direction::Release);
                }
            }
        })).map_err(|e| e.to_string())?;

        // charToVkCode - 完善版
        globals.set("charToVkCode", Func::from(|key: String| -> u32 {
            char_to_vk_code(&key)
        })).map_err(|e| e.to_string())?;

        // captureWindowImage
        globals.set("captureWindowImage", Func::from(|hwnd: i64| -> String {
            match window::capture_window_image(hwnd as isize) {
                Ok(result) => serde_json::to_string(&result).unwrap_or_default(),
                Err(_) => String::new(),
            }
        })).map_err(|e| e.to_string())?;

        // beep (Windows Beep API)
        globals.set("beep", Func::from(|freq: u32, duration: u32| {
            #[link(name = "kernel32")]
            extern "system" {
                fn Beep(dwFreq: u32, dwDuration: u32) -> i32;
            }
            unsafe { Beep(freq, duration); }
        })).map_err(|e| e.to_string())?;
    }

    // doubleClick
    let app_print = app.clone();
    globals.set("doubleClick", Func::from(move |x: i32, y: i32, button: String| {
        match Enigo::new(&Settings::default()) {
            Ok(mut enigo) => {
                if let Err(e) = enigo.move_mouse(x, y, Coordinate::Abs) {
                    let _ = app_print.emit("script-output", format!("[doubleClick] 移动失败: {e}"));
                    return;
                }
                let btn = match button.as_str() {
                    "right" => Button::Right,
                    "middle" => Button::Middle,
                    _ => Button::Left,
                };
                let _ = enigo.button(btn, Direction::Click);
                std::thread::sleep(std::time::Duration::from_millis(50));
                let _ = enigo.button(btn, Direction::Click);
            }
            Err(e) => { let _ = app_print.emit("script-output", format!("[doubleClick] 初始化失败: {e}")); }
        }
    })).map_err(|e| e.to_string())?;

    // dragMouse
    let app_print = app.clone();
    globals.set("dragMouse", Func::from(move |x1: i32, y1: i32, x2: i32, y2: i32, button: String| {
        match Enigo::new(&Settings::default()) {
            Ok(mut enigo) => {
                let btn = match button.as_str() {
                    "right" => Button::Right,
                    "middle" => Button::Middle,
                    _ => Button::Left,
                };
                if let Err(e) = enigo.move_mouse(x1, y1, Coordinate::Abs) {
                    let _ = app_print.emit("script-output", format!("[dragMouse] 移动失败: {e}"));
                    return;
                }
                let _ = enigo.button(btn, Direction::Press);
                std::thread::sleep(std::time::Duration::from_millis(50));
                if let Err(e) = enigo.move_mouse(x2, y2, Coordinate::Abs) {
                    let _ = app_print.emit("script-output", format!("[dragMouse] 拖拽失败: {e}"));
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
                let _ = enigo.button(btn, Direction::Release);
            }
            Err(e) => { let _ = app_print.emit("script-output", format!("[dragMouse] 初始化失败: {e}")); }
        }
    })).map_err(|e| e.to_string())?;

    // run(command) — 执行外部程序，返回 pid
    globals.set("run", Func::from(|command: String| -> String {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() { return "0".to_string(); }
        match std::process::Command::new(parts[0])
            .args(&parts[1..])
            .spawn()
        {
            Ok(child) => child.id().to_string(),
            Err(e) => {
                tracing::error!("[run] 执行失败: {e}");
                format!("error:{e}")
            }
        }
    })).map_err(|e| e.to_string())?;

    // runWait(command) — 执行并等待，返回 JSON {exitCode, stdout}
    globals.set("runWait", Func::from(|command: String| -> String {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() { return "{}".to_string(); }
        match std::process::Command::new(parts[0])
            .args(&parts[1..])
            .output()
        {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let code = output.status.code().unwrap_or(-1);
                serde_json::json!({"exitCode": code, "stdout": stdout}).to_string()
            }
            Err(e) => {
                tracing::error!("[runWait] 执行失败: {e}");
                serde_json::json!({"exitCode": -1, "stdout": format!("error:{e}")}).to_string()
            }
        }
    })).map_err(|e| e.to_string())?;

    // msgBox(title, text) — 通过 emit 让前端弹窗
    let app_clone = app.clone();
    globals.set("msgBox", Func::from(move |title: String, text: String| {
        let _ = app_clone.emit("script-msgbox", serde_json::json!({"title": title, "text": text}));
    })).map_err(|e| e.to_string())?;

    // notify(title, text) — 系统通知
    let app_clone = app.clone();
    globals.set("notify", Func::from(move |title: String, text: String| {
        let _ = app_clone.emit("script-notify", serde_json::json!({"title": title, "text": text}));
    })).map_err(|e| e.to_string())?;

    // random(min, max) — 随机整数
    globals.set("random", Func::from(|min: i64, max: i64| -> i64 {
        use std::time::SystemTime;
        if min >= max { return min; }
        let seed = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos() as i64;
        min + (seed.abs() % (max - min + 1))
    })).map_err(|e| e.to_string())?;

    // fileExists(path) — 沙箱内检查文件存在
    let app_clone = app.clone();
    globals.set("fileExists", Func::from(move |path: String| -> bool {
        match sandbox_path(&app_clone, &path) {
            Ok(safe_path) => safe_path.exists(),
            Err(_) => false,
        }
    })).map_err(|e| e.to_string())?;

    // waitForPixelColor(x, y, color, timeout, tolerance)
    let abort_clone = abort.clone();
    globals.set("waitForPixelColor", Func::from(move |x: i32, y: i32, color: String, timeout_ms: u64, tolerance: u8| -> bool {
        let target = parse_hex_color(&color);
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
        loop {
            if abort_clone.load(Ordering::Relaxed) { return false; }
            if let Ok(info) = pixel::get_pixel_color(x, y) {
                if color_matches(info.r, info.g, info.b, target.0, target.1, target.2, tolerance) {
                    return true;
                }
            }
            if std::time::Instant::now() >= deadline { return false; }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    })).map_err(|e| e.to_string())?;

    // httpGet(url) — GET 返回 body string
    globals.set("httpGet", Func::from(|url: String| -> String {
        match reqwest::blocking::get(&url) {
            Ok(resp) => resp.text().unwrap_or_default(),
            Err(e) => format!("error:{e}"),
        }
    })).map_err(|e| e.to_string())?;

    // httpPost(url, body, contentType) — POST 返回 body string
    globals.set("httpPost", Func::from(|url: String, body: String, content_type: String| -> String {
        let client = reqwest::blocking::Client::new();
        match client.post(&url)
            .header("Content-Type", &content_type)
            .body(body)
            .send()
        {
            Ok(resp) => resp.text().unwrap_or_default(),
            Err(e) => format!("error:{e}"),
        }
    })).map_err(|e| e.to_string())?;

    Ok(())
}

fn char_to_vk_code(key: &str) -> u32 {
    match key.to_lowercase().as_str() {
        "enter" | "return" => 0x0D,
        "space" => 0x20,
        "escape" | "esc" => 0x1B,
        "tab" => 0x09,
        "backspace" => 0x08,
        "delete" | "del" => 0x2E,
        "insert" | "ins" => 0x2D,
        "home" => 0x24,
        "end" => 0x23,
        "pageup" => 0x21,
        "pagedown" => 0x22,
        "up" => 0x26,
        "down" => 0x28,
        "left" => 0x25,
        "right" => 0x27,
        "ctrl" | "control" => 0xA2,
        "alt" => 0xA4,
        "shift" => 0xA0,
        "win" | "meta" => 0x5B,
        "capslock" => 0x14,
        "numlock" => 0x90,
        "scrolllock" => 0x91,
        "printscreen" => 0x2C,
        "f1" => 0x70, "f2" => 0x71, "f3" => 0x72, "f4" => 0x73,
        "f5" => 0x74, "f6" => 0x75, "f7" => 0x76, "f8" => 0x77,
        "f9" => 0x78, "f10" => 0x79, "f11" => 0x7A, "f12" => 0x7B,
        "num0" => 0x60, "num1" => 0x61, "num2" => 0x62, "num3" => 0x63,
        "num4" => 0x64, "num5" => 0x65, "num6" => 0x66, "num7" => 0x67,
        "num8" => 0x68, "num9" => 0x69,
        "multiply" | "num*" => 0x6A,
        "add" | "num+" => 0x6B,
        "subtract" | "num-" => 0x6D,
        "decimal" | "num." => 0x6E,
        "divide" | "num/" => 0x6F,
        ";" | "semicolon" => 0xBA,
        "=" | "equal" => 0xBB,
        "," | "comma" => 0xBC,
        "-" | "minus" => 0xBD,
        "." | "period" => 0xBE,
        "/" | "slash" => 0xBF,
        "`" | "backquote" => 0xC0,
        "[" | "bracketleft" => 0xDB,
        "\\" | "backslash" => 0xDC,
        "]" | "bracketright" => 0xDD,
        "'" | "quote" => 0xDE,
        s if s.len() == 1 => {
            let c = s.chars().next().unwrap();
            if c.is_ascii_alphanumeric() { c.to_ascii_uppercase() as u32 } else { 0 }
        }
        _ => 0,
    }
}

fn parse_hex_color(color: &str) -> (u8, u8, u8) {
    let hex = color.trim_start_matches('#');
    if hex.len() >= 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        (r, g, b)
    } else {
        (0, 0, 0)
    }
}

fn color_matches(r1: u8, g1: u8, b1: u8, r2: u8, g2: u8, b2: u8, tolerance: u8) -> bool {
    let t = tolerance as i16;
    (r1 as i16 - r2 as i16).abs() <= t
        && (g1 as i16 - g2 as i16).abs() <= t
        && (b1 as i16 - b2 as i16).abs() <= t
}
