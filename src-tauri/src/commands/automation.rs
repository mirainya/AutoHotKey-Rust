use enigo::{Enigo, Mouse, Keyboard, Settings, Coordinate, Button, Direction};

#[tauri::command]
pub fn get_mouse_pos() -> (i32, i32) {
    let enigo = Enigo::new(&Settings::default()).unwrap();
    enigo.location().map(|(x, y)| (x, y)).unwrap_or((0, 0))
}

#[tauri::command]
pub fn get_screen_size() -> (i32, i32) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
    unsafe { (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN)) }
}

#[tauri::command]
pub fn move_mouse(x: i32, y: i32) -> Result<(), String> {
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
    enigo.move_mouse(x, y, Coordinate::Abs).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn click_mouse(x: i32, y: i32, button: String) -> Result<(), String> {
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
    enigo.move_mouse(x, y, Coordinate::Abs).map_err(|e| e.to_string())?;
    let btn = match button.as_str() {
        "right" => Button::Right,
        "middle" => Button::Middle,
        _ => Button::Left,
    };
    enigo.button(btn, Direction::Click).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn scroll_mouse(x: i32, y: i32, delta: i32) -> Result<(), String> {
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
    enigo.move_mouse(x, y, Coordinate::Abs).map_err(|e| e.to_string())?;
    enigo.scroll(delta, enigo::Axis::Vertical).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn key_press(key: String) -> Result<(), String> {
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
    let k = parse_key(&key)?;
    enigo.key(k, Direction::Click).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn key_down(key: String) -> Result<(), String> {
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
    let k = parse_key(&key)?;
    enigo.key(k, Direction::Press).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn key_up(key: String) -> Result<(), String> {
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
    let k = parse_key(&key)?;
    enigo.key(k, Direction::Release).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn type_text(text: String) -> Result<(), String> {
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
    enigo.text(&text).map_err(|e| e.to_string())?;
    Ok(())
}

fn parse_key(key: &str) -> Result<enigo::Key, String> {
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
        s if s.len() == 1 => Key::Unicode(s.chars().next().unwrap()),
        _ => return Err(format!("未知按键: {}", key)),
    })
}
