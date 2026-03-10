use screenshots::Screen;
use std::fs;
use tauri::Manager;

#[tauri::command]
pub fn capture_screen(app: tauri::AppHandle) -> Result<String, String> {
    let screens = Screen::all().map_err(|e| e.to_string())?;
    let screen = screens.first().ok_or("没有找到屏幕")?;

    let image = screen.capture().map_err(|e| e.to_string())?;

    let mut screenshots_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    screenshots_dir.push("resource");
    screenshots_dir.push("screenshot");
    if !screenshots_dir.exists() {
        fs::create_dir_all(&screenshots_dir).map_err(|e| e.to_string())?;
    }

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let filename = format!("screenshot_{}.png", timestamp);
    let path = screenshots_dir.join(&filename);
    image.save(&path).map_err(|e| e.to_string())?;

    Ok(path.to_string_lossy().to_string())
}
