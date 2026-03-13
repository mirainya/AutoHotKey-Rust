use screenshots::Screen;
use serde::Serialize;
use std::fs;
use tauri::Manager;
use crate::error::AppError;

#[derive(Debug, Serialize)]
pub struct ScreenInfo {
    pub id: u32,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub is_primary: bool,
}

#[tauri::command]
pub fn get_all_screens() -> Result<Vec<ScreenInfo>, AppError> {
    let screens = Screen::all().map_err(|e| AppError::Screenshot(e.to_string()))?;
    Ok(screens.iter().enumerate().map(|(i, s)| ScreenInfo {
        id: i as u32,
        x: s.display_info.x,
        y: s.display_info.y,
        width: s.display_info.width,
        height: s.display_info.height,
        is_primary: s.display_info.is_primary,
    }).collect())
}

#[tauri::command]
pub fn capture_screen(app: tauri::AppHandle) -> Result<String, AppError> {
    capture_screen_by_index(app, 0)
}

#[tauri::command]
pub fn capture_screen_by_index(app: tauri::AppHandle, index: usize) -> Result<String, AppError> {
    let screens = Screen::all().map_err(|e| AppError::Screenshot(e.to_string()))?;
    let screen = screens.get(index).ok_or(AppError::Screenshot(format!("显示器 {} 不存在", index)))?;

    let image = screen.capture().map_err(|e| AppError::Screenshot(e.to_string()))?;

    let mut screenshots_dir = app.path().app_data_dir().map_err(|e| AppError::Io(e.to_string()))?;
    screenshots_dir.push("resource");
    screenshots_dir.push("screenshot");
    if !screenshots_dir.exists() {
        fs::create_dir_all(&screenshots_dir)?;
    }

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let filename = format!("screenshot_{}.png", timestamp);
    let path = screenshots_dir.join(&filename);
    image.save(&path).map_err(|e| AppError::Screenshot(e.to_string()))?;

    tracing::info!("截图已保存: {}", path.display());
    Ok(path.to_string_lossy().to_string())
}
