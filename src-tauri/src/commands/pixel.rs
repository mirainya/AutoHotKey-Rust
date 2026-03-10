use screenshots::Screen;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;
use image::GenericImageView;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixelInfo {
    pub x: i32,
    pub y: i32,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixelPattern {
    pub name: String,
    pub pixels: Vec<PixelInfo>,
}

#[derive(Debug, Serialize)]
pub struct CaptureResult {
    pub pixels: Vec<PixelInfo>,
    pub image_base64: String,
    pub width: u32,
    pub height: u32,
}

#[tauri::command]
pub fn capture_pixels(x: i32, y: i32, width: u32, height: u32, layers: u32) -> Result<CaptureResult, String> {
    let screens = Screen::all().map_err(|e| e.to_string())?;
    let screen = screens.first().ok_or("没有找到屏幕")?;
    let image = screen.capture().map_err(|e| e.to_string())?;

    let center_x = x + (width / 2) as i32;
    let center_y = y + (height / 2) as i32;

    let mut positions = vec![(center_x, center_y)];

    for layer in 1..=layers {
        let offset = layer as i32;
        positions.push((center_x, center_y - offset));
        positions.push((center_x, center_y + offset));
        positions.push((center_x - offset, center_y));
        positions.push((center_x + offset, center_y));
        positions.push((center_x - offset, center_y - offset));
        positions.push((center_x + offset, center_y - offset));
        positions.push((center_x - offset, center_y + offset));
        positions.push((center_x + offset, center_y + offset));
    }

    let mut pixels = Vec::new();
    for (px, py) in positions {
        if px >= 0 && py >= 0 && px < image.width() as i32 && py < image.height() as i32 {
            let pixel = image.get_pixel(px as u32, py as u32);
            pixels.push(PixelInfo {
                x: px,
                y: py,
                r: pixel[0],
                g: pixel[1],
                b: pixel[2],
            });
        }
    }

    let crop_x = x.max(0) as u32;
    let crop_y = y.max(0) as u32;
    let crop_width = width.min(image.width() - crop_x);
    let crop_height = height.min(image.height() - crop_y);

    let cropped = image.view(crop_x, crop_y, crop_width, crop_height).to_image();
    let mut buffer = Vec::new();
    cropped.write_to(&mut std::io::Cursor::new(&mut buffer), image::ImageFormat::Png)
        .map_err(|e: image::ImageError| e.to_string())?;
    let image_base64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &buffer);

    Ok(CaptureResult {
        pixels,
        image_base64,
        width: crop_width,
        height: crop_height,
    })
}

fn color_matches(c1: (u8, u8, u8), c2: (u8, u8, u8), tolerance: u8) -> bool {
    let dr = (c1.0 as i32 - c2.0 as i32).abs();
    let dg = (c1.1 as i32 - c2.1 as i32).abs();
    let db = (c1.2 as i32 - c2.2 as i32).abs();
    dr <= tolerance as i32 && dg <= tolerance as i32 && db <= tolerance as i32
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MatchResult {
    pub x: i32,
    pub y: i32,
}

#[tauri::command]
pub fn find_pixel_pattern(pattern: PixelPattern, tolerance: u8) -> Result<Vec<MatchResult>, String> {
    let screens = Screen::all().map_err(|e| e.to_string())?;
    let screen = screens.first().ok_or("没有找到屏幕")?;
    let image = screen.capture().map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    let width = image.width() as i32;
    let height = image.height() as i32;

    for y in 0..height {
        for x in 0..width {
            let mut all_match = true;
            for pixel in &pattern.pixels {
                let px = x + pixel.x;
                let py = y + pixel.y;
                if px >= 0 && py >= 0 && px < width && py < height {
                    let img_pixel = image.get_pixel(px as u32, py as u32);
                    if !color_matches((img_pixel[0], img_pixel[1], img_pixel[2]), (pixel.r, pixel.g, pixel.b), tolerance) {
                        all_match = false;
                        break;
                    }
                } else {
                    all_match = false;
                    break;
                }
            }
            if all_match {
                results.push(MatchResult { x, y });
            }
        }
    }

    Ok(results)
}

fn get_patterns_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let mut dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    dir.push("resource");
    dir.push("patterns");
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    }
    Ok(dir)
}

#[tauri::command]
pub fn save_pixel_pattern(app: tauri::AppHandle, pattern: PixelPattern) -> Result<(), String> {
    let dir = get_patterns_dir(&app)?;
    let file_path = dir.join(format!("{}.json", pattern.name));
    let json = serde_json::to_string_pretty(&pattern).map_err(|e| e.to_string())?;
    fs::write(file_path, json).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn load_pixel_patterns(app: tauri::AppHandle) -> Result<Vec<PixelPattern>, String> {
    let dir = get_patterns_dir(&app)?;
    let mut patterns = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                if let Ok(pattern) = serde_json::from_str::<PixelPattern>(&content) {
                    patterns.push(pattern);
                }
            }
        }
    }

    Ok(patterns)
}

#[tauri::command]
pub fn delete_pixel_pattern(app: tauri::AppHandle, name: String) -> Result<(), String> {
    let dir = get_patterns_dir(&app)?;
    let file_path = dir.join(format!("{}.json", name));
    fs::remove_file(file_path).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_pixel_color(x: i32, y: i32) -> Result<PixelInfo, String> {
    let screens = Screen::all().map_err(|e| e.to_string())?;
    let screen = screens.first().ok_or("没有找到屏幕")?;
    let image = screen.capture().map_err(|e| e.to_string())?;

    if x >= 0 && y >= 0 && x < image.width() as i32 && y < image.height() as i32 {
        let pixel = image.get_pixel(x as u32, y as u32);
        Ok(PixelInfo { x, y, r: pixel[0], g: pixel[1], b: pixel[2] })
    } else {
        Err("坐标超出屏幕范围".to_string())
    }
}

/// 在给定的 RGBA 图像字节中匹配像素图案，返回窗口相对坐标
#[tauri::command]
pub fn find_pattern_in_image(
    image_bytes: Vec<u8>,
    img_width: u32,
    img_height: u32,
    pattern: PixelPattern,
    tolerance: u8,
) -> Result<Vec<MatchResult>, String> {
    let img = image::RgbaImage::from_raw(img_width, img_height, image_bytes)
        .ok_or("图像数据无效")?;
    let mut results = Vec::new();
    let w = img_width as i32;
    let h = img_height as i32;
    for y in 0..h {
        for x in 0..w {
            let mut all_match = true;
            for pixel in &pattern.pixels {
                let px = x + pixel.x;
                let py = y + pixel.y;
                if px >= 0 && py >= 0 && px < w && py < h {
                    let p = img.get_pixel(px as u32, py as u32);
                    if !color_matches((p[0], p[1], p[2]), (pixel.r, pixel.g, pixel.b), tolerance) {
                        all_match = false; break;
                    }
                } else { all_match = false; break; }
            }
            if all_match { results.push(MatchResult { x, y }); }
        }
    }
    Ok(results)
}
