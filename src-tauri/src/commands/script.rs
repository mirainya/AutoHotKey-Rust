use rquickjs::{Context, Runtime};
use rquickjs::function::Func;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::Manager;
use tauri::Emitter;

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

static ABORT_FLAG: Mutex<Option<Arc<AtomicBool>>> = Mutex::new(None);

fn get_scripts_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let mut path = app.path().app_data_dir().map_err(|e| e.to_string())?;
    path.push("scripts");
    if !path.exists() {
        fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    }
    Ok(path)
}

#[tauri::command]
pub fn save_script(app: tauri::AppHandle, script: Script) -> Result<(), String> {
    let path = get_scripts_dir(&app)?.join(format!("{}.json", script.id));
    let json = serde_json::to_string_pretty(&script).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn load_scripts(app: tauri::AppHandle) -> Result<Vec<Script>, String> {
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
pub fn delete_script(app: tauri::AppHandle, id: String) -> Result<(), String> {
    let path = get_scripts_dir(&app)?.join(format!("{}.json", id));
    fs::remove_file(path).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn stop_script() -> Result<(), String> {
    if let Some(flag) = ABORT_FLAG.lock().unwrap().as_ref() {
        flag.store(true, Ordering::Relaxed);
    }
    Ok(())
}

#[tauri::command]
pub fn execute_script(app: tauri::AppHandle, code: String) -> Result<(), String> {
    let abort = Arc::new(AtomicBool::new(false));
    *ABORT_FLAG.lock().unwrap() = Some(abort.clone());

    std::thread::spawn(move || {
        let runtime = match Runtime::new() {
            Ok(r) => r,
            Err(e) => { let _ = app.emit("script-done", format!("运行时错误: {e}")); return; }
        };
        let context = match Context::full(&runtime) {
            Ok(c) => c,
            Err(e) => { let _ = app.emit("script-done", format!("上下文错误: {e}")); return; }
        };
        let output = Arc::new(Mutex::new(String::new()));
        let result = context.with(|ctx| {
            register_functions(&ctx, output.clone(), app.clone(), abort)?;
            ctx.eval::<(), _>(code).map_err(|e| e.to_string())
        });
        match result {
            Ok(_) => { let _ = app.emit("script-done", output.lock().unwrap().clone()); }
            Err(e) => { let _ = app.emit("script-done", format!("错误: {e}")); }
        }
    });

    Ok(())
}

fn register_functions(
    ctx: &rquickjs::Ctx,
    output: Arc<Mutex<String>>,
    app: tauri::AppHandle,
    abort: Arc<AtomicBool>,
) -> Result<(), String> {
    use crate::commands::{screenshot, automation, pixel, window};
    let globals = ctx.globals();

    let out = output.clone();
    let app_print = app.clone();
    globals.set("print", Func::from(move |msg: String| {
        let line = format!("{}\n", msg);
        out.lock().unwrap().push_str(&line);
        let _ = app_print.emit("script-output", msg);
    })).map_err(|e| e.to_string())?;

    let abort_clone = abort.clone();
    globals.set("sleep", Func::from(move |ms: i32| {
        let end = std::time::Instant::now() + std::time::Duration::from_millis(ms as u64);
        while std::time::Instant::now() < end {
            if abort_clone.load(Ordering::Relaxed) { break; }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    })).map_err(|e| e.to_string())?;

    let abort_clone = abort.clone();
    globals.set("isStopped", Func::from(move || -> bool {
        abort_clone.load(Ordering::Relaxed)
    })).map_err(|e| e.to_string())?;

    let app2 = app.clone();
    globals.set("screenshot", Func::from(move || {
        screenshot::capture_screen(app2.clone()).unwrap_or_default()
    })).map_err(|e| e.to_string())?;

    globals.set("moveMouse", Func::from(|x: i32, y: i32| { let _ = automation::move_mouse(x, y); })).map_err(|e| e.to_string())?;
    globals.set("clickMouse", Func::from(|x: i32, y: i32, button: String| { let _ = automation::click_mouse(x, y, button); })).map_err(|e| e.to_string())?;
    globals.set("scrollMouse", Func::from(|x: i32, y: i32, delta: i32| { let _ = automation::scroll_mouse(x, y, delta); })).map_err(|e| e.to_string())?;
    globals.set("typeText", Func::from(|text: String| { let _ = automation::type_text(text); })).map_err(|e| e.to_string())?;
    globals.set("keyPress", Func::from(|key: String| { let _ = automation::key_press(key); })).map_err(|e| e.to_string())?;
    globals.set("keyDown", Func::from(|key: String| { let _ = automation::key_down(key); })).map_err(|e| e.to_string())?;
    globals.set("keyUp", Func::from(|key: String| { let _ = automation::key_up(key); })).map_err(|e| e.to_string())?;

    globals.set("getPixelColor", Func::from(|x: i32, y: i32| -> String {
        pixel::get_pixel_color(x, y)
            .map(|c| serde_json::to_string(&c).unwrap_or_default())
            .unwrap_or_default()
    })).map_err(|e| e.to_string())?;

    let app3 = app.clone();
    globals.set("findPattern", Func::from(move |name: String, tolerance: u8| -> String {
        let patterns = pixel::load_pixel_patterns(app3.clone()).unwrap_or_default();
        if let Some(pattern) = patterns.into_iter().find(|p| p.name == name) {
            if let Ok(results) = pixel::find_pixel_pattern(pattern, tolerance) {
                let positions: Vec<(i32, i32)> = results.into_iter().map(|r| (r.x, r.y)).collect();
                return serde_json::to_string(&positions).unwrap_or_else(|_| "[]".to_string());
            }
        }
        "[]".to_string()
    })).map_err(|e| e.to_string())?;

    globals.set("findCustomPattern", Func::from(|pattern_json: String, tolerance: u8| -> String {
        if let Ok(pattern) = serde_json::from_str::<pixel::PixelPattern>(&pattern_json) {
            if let Ok(results) = pixel::find_pixel_pattern(pattern, tolerance) {
                let positions: Vec<(i32, i32)> = results.into_iter().map(|r| (r.x, r.y)).collect();
                return serde_json::to_string(&positions).unwrap_or_else(|_| "[]".to_string());
            }
        }
        "[]".to_string()
    })).map_err(|e| e.to_string())?;

    // Window API
    globals.set("findWindow", Func::from(|title: String| -> i64 {
        window::find_window(title).unwrap_or(0) as i64
    })).map_err(|e| e.to_string())?;

    globals.set("getForegroundWindow", Func::from(|| -> i64 {
        window::get_foreground_window().unwrap_or(0) as i64
    })).map_err(|e| e.to_string())?;

    globals.set("getWindowInfo", Func::from(|hwnd: i64| -> String {
        let info = window::get_window_info(hwnd as isize);
        serde_json::to_string(&info).unwrap_or_default()
    })).map_err(|e| e.to_string())?;

    globals.set("activateWindow", Func::from(|hwnd: i64| { let _ = window::activate_window(hwnd as isize); })).map_err(|e| e.to_string())?;
    globals.set("moveWindow", Func::from(|hwnd: i64, x: i32, y: i32, w: i32, h: i32| { let _ = window::move_window(hwnd as isize, x, y, w, h); })).map_err(|e| e.to_string())?;
    globals.set("showWindow", Func::from(|hwnd: i64, state: String| { let _ = window::show_window(hwnd as isize, state); })).map_err(|e| e.to_string())?;
    globals.set("postClick", Func::from(|hwnd: i64, x: i32, y: i32, button: String| { let _ = window::post_click(hwnd as isize, x, y, button); })).map_err(|e| e.to_string())?;
    globals.set("postKey", Func::from(|hwnd: i64, vk_code: u32| { let _ = window::post_key(hwnd as isize, vk_code); })).map_err(|e| e.to_string())?;
    globals.set("postChar", Func::from(|hwnd: i64, ch: String| { let _ = window::post_char(hwnd as isize, ch); })).map_err(|e| e.to_string())?;

    // waitForPattern
    let app4 = app.clone();
    let abort_clone = abort.clone();
    globals.set("waitForPattern", Func::from(move |name: String, timeout_ms: i32| -> String {
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms as u64);
        loop {
            if abort_clone.load(Ordering::Relaxed) { return "[]".to_string(); }
            let patterns = pixel::load_pixel_patterns(app4.clone()).unwrap_or_default();
            if let Some(pattern) = patterns.into_iter().find(|p| p.name == name) {
                if let Ok(results) = pixel::find_pixel_pattern(pattern, 10) {
                    if !results.is_empty() {
                        let positions: Vec<(i32, i32)> = results.into_iter().map(|r| (r.x, r.y)).collect();
                        return serde_json::to_string(&positions).unwrap_or_else(|_| "[]".to_string());
                    }
                }
            }
            if std::time::Instant::now() >= deadline { return "[]".to_string(); }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    })).map_err(|e| e.to_string())?;

    // 新增 API
    globals.set("getMousePos", Func::from(|| -> String {
        let (x, y) = automation::get_mouse_pos();
        serde_json::to_string(&[x, y]).unwrap_or_default()
    })).map_err(|e| e.to_string())?;

    globals.set("getScreenSize", Func::from(|| -> String {
        let (w, h) = automation::get_screen_size();
        serde_json::to_string(&[w, h]).unwrap_or_default()
    })).map_err(|e| e.to_string())?;

    globals.set("getClipboard", Func::from(|| -> String {
        window::get_clipboard()
    })).map_err(|e| e.to_string())?;

    globals.set("setClipboard", Func::from(|text: String| {
        let _ = window::set_clipboard(text);
    })).map_err(|e| e.to_string())?;

    globals.set("msgBox", Func::from(|text: String| {
        window::msg_box(text);
    })).map_err(|e| e.to_string())?;

    globals.set("enumWindows", Func::from(|| -> String {
        serde_json::to_string(&window::enum_windows()).unwrap_or_else(|_| "[]".to_string())
    })).map_err(|e| e.to_string())?;

    // 窗口截图与窗口内匹配
    globals.set("captureWindow", Func::from(|hwnd: i64| -> String {
        match window::capture_window_image(hwnd as isize) {
            Ok(r) => serde_json::to_string(&r).unwrap_or_default(),
            Err(_) => "{}".to_string(),
        }
    })).map_err(|e| e.to_string())?;

    let app5 = app.clone();
    globals.set("findPatternInWindow", Func::from(move |hwnd: i64, name: String, tolerance: u8| -> String {
        let cap = match window::capture_window_image(hwnd as isize) {
            Ok(c) => c, Err(_) => return "[]".to_string(),
        };
        // decode base64 -> raw RGBA bytes
        let bytes = match base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &cap.image_base64) {
            Ok(b) => b, Err(_) => return "[]".to_string(),
        };
        // PNG bytes -> RGBA raw
        let img = match image::load_from_memory(&bytes) {
            Ok(i) => i.to_rgba8(), Err(_) => return "[]".to_string(),
        };
        let patterns = pixel::load_pixel_patterns(app5.clone()).unwrap_or_default();
        if let Some(pattern) = patterns.into_iter().find(|p| p.name == name) {
            if let Ok(results) = pixel::find_pattern_in_image(img.into_raw(), cap.width, cap.height, pattern, tolerance) {
                let positions: Vec<(i32, i32)> = results.into_iter().map(|r| (r.x, r.y)).collect();
                return serde_json::to_string(&positions).unwrap_or_else(|_| "[]".to_string());
            }
        }
        "[]".to_string()
    })).map_err(|e| e.to_string())?;

    let app6 = app.clone();
    let abort_clone2 = abort.clone();
    globals.set("waitForPatternInWindow", Func::from(move |hwnd: i64, name: String, timeout_ms: i32| -> String {
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms as u64);
        loop {
            if abort_clone2.load(Ordering::Relaxed) { return "[]".to_string(); }
            let cap = match window::capture_window_image(hwnd as isize) {
                Ok(c) => c, Err(_) => { std::thread::sleep(std::time::Duration::from_millis(100)); continue; }
            };
            let bytes = match base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &cap.image_base64) {
                Ok(b) => b, Err(_) => return "[]".to_string(),
            };
            let img = match image::load_from_memory(&bytes) {
                Ok(i) => i.to_rgba8(), Err(_) => return "[]".to_string(),
            };
            let patterns = pixel::load_pixel_patterns(app6.clone()).unwrap_or_default();
            if let Some(pattern) = patterns.into_iter().find(|p| p.name == name) {
                if let Ok(results) = pixel::find_pattern_in_image(img.into_raw(), cap.width, cap.height, pattern, 10) {
                    if !results.is_empty() {
                        let positions: Vec<(i32, i32)> = results.into_iter().map(|r| (r.x, r.y)).collect();
                        return serde_json::to_string(&positions).unwrap_or_else(|_| "[]".to_string());
                    }
                }
            }
            if std::time::Instant::now() >= deadline { return "[]".to_string(); }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    })).map_err(|e| e.to_string())?;

    globals.set("readFile", Func::from(|path: String| -> String {
        std::fs::read_to_string(&path).unwrap_or_default()
    })).map_err(|e| e.to_string())?;

    globals.set("writeFile", Func::from(|path: String, text: String| -> bool {
        std::fs::write(&path, text).is_ok()
    })).map_err(|e| e.to_string())?;

    globals.set("getWindowTitle", Func::from(|hwnd: i64| -> String {
        window::get_window_info(hwnd as isize).title
    })).map_err(|e| e.to_string())?;

    globals.set("charToVkCode", Func::from(|key: String| -> u32 {        match key.to_lowercase().as_str() {
            "enter" => 0x0D, "space" => 0x20, "escape" | "esc" => 0x1B,
            "tab" => 0x09, "backspace" => 0x08,
            "up" => 0x26, "down" => 0x28, "left" => 0x25, "right" => 0x27,
            "f1" => 0x70, "f2" => 0x71, "f3" => 0x72, "f4" => 0x73,
            "f5" => 0x74, "f6" => 0x75, "f7" => 0x76, "f8" => 0x77,
            "f9" => 0x78, "f10" => 0x79, "f11" => 0x7A, "f12" => 0x7B,
            s if s.len() == 1 => {
                let c = s.chars().next().unwrap();
                if c.is_ascii_alphanumeric() { c.to_ascii_uppercase() as u32 } else { 0 }
            }
            _ => 0,
        }
    })).map_err(|e| e.to_string())?;

    Ok(())
}
