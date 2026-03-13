use enigo::{Enigo, Mouse, Keyboard, Settings, Coordinate, Button, Direction};
use crate::error::AppError;

fn new_enigo() -> Result<Enigo, AppError> {
    Enigo::new(&Settings::default()).map_err(|e| AppError::Enigo(e.to_string()))
}

#[tauri::command]
pub fn get_mouse_pos() -> Result<(i32, i32), AppError> {
    let enigo = new_enigo()?;
    enigo.location().map_err(|e| AppError::Enigo(e.to_string()))
}

#[tauri::command]
pub fn get_screen_size() -> (i32, i32) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
    unsafe { (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN)) }
}

#[tauri::command]
pub fn move_mouse(x: i32, y: i32) -> Result<(), AppError> {
    let mut enigo = new_enigo()?;
    enigo.move_mouse(x, y, Coordinate::Abs).map_err(|e| AppError::Enigo(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub fn click_mouse(x: i32, y: i32, button: String) -> Result<(), AppError> {
    let mut enigo = new_enigo()?;
    enigo.move_mouse(x, y, Coordinate::Abs).map_err(|e| AppError::Enigo(e.to_string()))?;
    let btn = match button.as_str() {
        "right" => Button::Right,
        "middle" => Button::Middle,
        _ => Button::Left,
    };
    enigo.button(btn, Direction::Click).map_err(|e| AppError::Enigo(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub fn scroll_mouse(x: i32, y: i32, delta: i32) -> Result<(), AppError> {
    let mut enigo = new_enigo()?;
    enigo.move_mouse(x, y, Coordinate::Abs).map_err(|e| AppError::Enigo(e.to_string()))?;
    enigo.scroll(delta, enigo::Axis::Vertical).map_err(|e| AppError::Enigo(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub fn key_press(key: String) -> Result<(), AppError> {
    let mut enigo = new_enigo()?;
    let k = parse_key(&key)?;
    enigo.key(k, Direction::Click).map_err(|e| AppError::Enigo(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub fn key_down(key: String) -> Result<(), AppError> {
    let mut enigo = new_enigo()?;
    let k = parse_key(&key)?;
    enigo.key(k, Direction::Press).map_err(|e| AppError::Enigo(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub fn key_up(key: String) -> Result<(), AppError> {
    let mut enigo = new_enigo()?;
    let k = parse_key(&key)?;
    enigo.key(k, Direction::Release).map_err(|e| AppError::Enigo(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub fn type_text(text: String) -> Result<(), AppError> {
    let mut enigo = new_enigo()?;
    enigo.text(&text).map_err(|e| AppError::Enigo(e.to_string()))?;
    Ok(())
}

/// 供 script.rs 内部调用的公开版本
pub fn parse_key_public(key: &str) -> Result<enigo::Key, AppError> {
    parse_key(key)
}

fn parse_key(key: &str) -> Result<enigo::Key, AppError> {
    use enigo::Key;
    Ok(match key.to_lowercase().as_str() {
        "enter" | "return" => Key::Return,
        "tab" => Key::Tab,
        "escape" | "esc" => Key::Escape,
        "space" => Key::Space,
        "backspace" => Key::Backspace,
        "delete" | "del" => Key::Delete,
        "up" => Key::UpArrow,
        "down" => Key::DownArrow,
        "left" => Key::LeftArrow,
        "right" => Key::RightArrow,
        "home" => Key::Home,
        "end" => Key::End,
        "pageup" => Key::PageUp,
        "pagedown" => Key::PageDown,
        "f1" => Key::F1, "f2" => Key::F2, "f3" => Key::F3, "f4" => Key::F4,
        "f5" => Key::F5, "f6" => Key::F6, "f7" => Key::F7, "f8" => Key::F8,
        "f9" => Key::F9, "f10" => Key::F10, "f11" => Key::F11, "f12" => Key::F12,
        "ctrl" | "control" => Key::Control,
        "alt" => Key::Alt,
        "shift" => Key::Shift,
        "win" | "meta" => Key::Meta,
        "insert" | "ins" => Key::Other(0x2D),
        "capslock" => Key::CapsLock,
        "numlock" => Key::Other(0x90),
        "scrolllock" => Key::Other(0x91),
        "printscreen" => Key::Other(0x2C),
        "num0" => Key::Other(0x60), "num1" => Key::Other(0x61),
        "num2" => Key::Other(0x62), "num3" => Key::Other(0x63),
        "num4" => Key::Other(0x64), "num5" => Key::Other(0x65),
        "num6" => Key::Other(0x66), "num7" => Key::Other(0x67),
        "num8" => Key::Other(0x68), "num9" => Key::Other(0x69),
        "multiply" => Key::Other(0x6A),
        "add" => Key::Other(0x6B),
        "subtract" => Key::Other(0x6D),
        "decimal" => Key::Other(0x6E),
        "divide" => Key::Other(0x6F),
        s if s.len() == 1 => Key::Unicode(s.chars().next().unwrap()),
        _ => return Err(AppError::Custom(format!("未知按键: {}", key))),
    })
}
