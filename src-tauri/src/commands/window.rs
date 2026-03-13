#![cfg(target_os = "windows")]

use serde::{Deserialize, Serialize};
use windows_sys::Win32::Foundation::{BOOL, HWND, LPARAM, RECT};
use windows_sys::Win32::UI::WindowsAndMessaging::*;
use windows_sys::Win32::System::DataExchange::{OpenClipboard, CloseClipboard, GetClipboardData, SetClipboardData, EmptyClipboard};
use windows_sys::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
use windows_sys::Win32::System::ProcessStatus::GetProcessImageFileNameW;
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub hwnd: isize,
    pub title: String,
    pub class_name: String,
    pub process_name: String,
    pub rect: WindowRect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowRect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

#[derive(Debug, Serialize)]
pub struct WindowCaptureResult {
    pub image_base64: String,
    pub width: u32,
    pub height: u32,
}

fn get_window_title(hwnd: HWND) -> String {
    let mut buf = [0u16; 512];
    let len = unsafe { GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32) };
    String::from_utf16_lossy(&buf[..len as usize])
}

fn get_window_class(hwnd: HWND) -> String {
    let mut buf = [0u16; 256];
    let len = unsafe { GetClassNameW(hwnd, buf.as_mut_ptr(), buf.len() as i32) };
    String::from_utf16_lossy(&buf[..len as usize])
}

fn get_process_name(hwnd: HWND) -> String {
    unsafe {
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, &mut pid);
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle == 0 { return String::new(); }
        let mut buf = [0u16; 512];
        let len = GetProcessImageFileNameW(handle, buf.as_mut_ptr(), buf.len() as u32);
        let full = String::from_utf16_lossy(&buf[..len as usize]);
        full.split(['/', '\\']).last().unwrap_or("").to_string()
    }
}

pub fn hwnd_to_info(hwnd: HWND) -> WindowInfo {
    let mut rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    unsafe { GetWindowRect(hwnd, &mut rect); }
    WindowInfo {
        hwnd: hwnd as isize,
        title: get_window_title(hwnd),
        class_name: get_window_class(hwnd),
        process_name: get_process_name(hwnd),
        rect: WindowRect {
            x: rect.left, y: rect.top,
            w: rect.right - rect.left,
            h: rect.bottom - rect.top,
        },
    }
}

#[tauri::command]
pub fn find_window(title: String) -> Option<isize> {
    let wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
    let hwnd = unsafe { FindWindowW(std::ptr::null(), wide.as_ptr()) };
    if hwnd != 0 { Some(hwnd as isize) } else { None }
}

#[tauri::command]
pub fn find_window_by_class(class_name: String) -> Option<isize> {
    let wide: Vec<u16> = class_name.encode_utf16().chain(std::iter::once(0)).collect();
    let hwnd = unsafe { FindWindowW(wide.as_ptr(), std::ptr::null()) };
    if hwnd != 0 { Some(hwnd as isize) } else { None }
}

#[tauri::command]
pub fn get_foreground_window() -> Option<isize> {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd != 0 { Some(hwnd as isize) } else { None }
}

#[tauri::command]
pub fn get_window_info(hwnd: isize) -> WindowInfo {
    hwnd_to_info(hwnd as HWND)
}

#[tauri::command]
pub fn enum_windows() -> Vec<WindowInfo> {
    let mut list: Vec<WindowInfo> = Vec::new();
    let ptr = &mut list as *mut Vec<WindowInfo> as LPARAM;
    unsafe {
        EnumWindows(Some(enum_callback), ptr);
    }
    list
}

unsafe extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    if IsWindowVisible(hwnd) == 0 { return 1; }
    let title = get_window_title(hwnd);
    if title.is_empty() { return 1; }
    let list = &mut *(lparam as *mut Vec<WindowInfo>);
    list.push(hwnd_to_info(hwnd));
    1
}

#[tauri::command]
pub fn activate_window(hwnd: isize) -> Result<(), AppError> {
    let h = hwnd as HWND;
    if h == 0 { return Err(AppError::Window("无效的窗口句柄".into())); }
    unsafe {
        ShowWindow(h, SW_RESTORE);
        SetForegroundWindow(h);
    }
    Ok(())
}

#[tauri::command]
pub fn move_window(hwnd: isize, x: i32, y: i32, width: i32, height: i32) -> Result<(), AppError> {
    let h = hwnd as HWND;
    if h == 0 { return Err(AppError::Window("无效的窗口句柄".into())); }
    let ok = unsafe { MoveWindow(h, x, y, width, height, 1) };
    if ok == 0 { Err(AppError::Window("移动窗口失败".into())) } else { Ok(()) }
}

#[tauri::command]
pub fn show_window(hwnd: isize, cmd: i32) -> Result<(), AppError> {
    let h = hwnd as HWND;
    if h == 0 { return Err(AppError::Window("无效的窗口句柄".into())); }
    unsafe { ShowWindow(h, cmd); }
    Ok(())
}

#[tauri::command]
pub fn post_click(hwnd: isize, x: i32, y: i32, button: String) -> Result<(), AppError> {
    let h = hwnd as HWND;
    if h == 0 { return Err(AppError::Window("无效的窗口句柄".into())); }
    let lparam = ((y & 0xFFFF) << 16 | (x & 0xFFFF)) as LPARAM;
    let (down_msg, up_msg) = match button.as_str() {
        "right" => (WM_RBUTTONDOWN, WM_RBUTTONUP),
        "middle" => (WM_MBUTTONDOWN, WM_MBUTTONUP),
        _ => (WM_LBUTTONDOWN, WM_LBUTTONUP),
    };
    unsafe {
        PostMessageW(h, down_msg, 0, lparam);
        PostMessageW(h, up_msg, 0, lparam);
    }
    Ok(())
}

#[tauri::command]
pub fn post_key(hwnd: isize, vk_code: u32) -> Result<(), AppError> {
    let h = hwnd as HWND;
    if h == 0 { return Err(AppError::Window("无效的窗口句柄".into())); }
    unsafe {
        PostMessageW(h, WM_KEYDOWN, vk_code as usize, 0);
        PostMessageW(h, WM_KEYUP, vk_code as usize, 0);
    }
    Ok(())
}

#[tauri::command]
pub fn post_char(hwnd: isize, ch: String) -> Result<(), AppError> {
    let h = hwnd as HWND;
    if h == 0 { return Err(AppError::Window("无效的窗口句柄".into())); }
    for c in ch.encode_utf16() {
        unsafe { PostMessageW(h, WM_CHAR, c as usize, 0); }
    }
    Ok(())
}

#[tauri::command]
pub fn send_keys(hwnd: isize, keys: String) -> Result<(), AppError> {
    let h = hwnd as HWND;
    if h == 0 { return Err(AppError::Window("无效的窗口句柄".into())); }

    let parts: Vec<&str> = keys.split('+').map(|s| s.trim()).collect();
    let mut modifiers = Vec::new();
    let mut main_key: Option<u32> = None;

    for part in &parts {
        match part.to_lowercase().as_str() {
            "ctrl" | "control" => modifiers.push(0xA2u32), // VK_LCONTROL
            "shift" => modifiers.push(0xA0u32),              // VK_LSHIFT
            "alt" => modifiers.push(0xA4u32),                // VK_LMENU
            "win" => modifiers.push(0x5Bu32),                // VK_LWIN
            other => main_key = Some(str_to_vk(other)),
        }
    }

    unsafe {
        for &m in &modifiers {
            PostMessageW(h, WM_KEYDOWN, m as usize, 0);
        }
        if let Some(vk) = main_key {
            if vk != 0 {
                PostMessageW(h, WM_KEYDOWN, vk as usize, 0);
                PostMessageW(h, WM_KEYUP, vk as usize, 0);
            }
        }
        for &m in modifiers.iter().rev() {
            PostMessageW(h, WM_KEYUP, m as usize, 0);
        }
    }
    Ok(())
}

pub fn str_to_vk(key: &str) -> u32 {
    match key.to_lowercase().as_str() {
        "enter" => 0x0D, "space" => 0x20, "escape" | "esc" => 0x1B,
        "tab" => 0x09, "backspace" => 0x08,
        "up" => 0x26, "down" => 0x28, "left" => 0x25, "right" => 0x27,
        "home" => 0x24, "end" => 0x23, "pageup" => 0x21, "pagedown" => 0x22,
        "insert" | "ins" => 0x2D, "delete" | "del" => 0x2E,
        "f1" => 0x70, "f2" => 0x71, "f3" => 0x72, "f4" => 0x73,
        "f5" => 0x74, "f6" => 0x75, "f7" => 0x76, "f8" => 0x77,
        "f9" => 0x78, "f10" => 0x79, "f11" => 0x7A, "f12" => 0x7B,
        "num0" => 0x60, "num1" => 0x61, "num2" => 0x62, "num3" => 0x63,
        "num4" => 0x64, "num5" => 0x65, "num6" => 0x66, "num7" => 0x67,
        "num8" => 0x68, "num9" => 0x69,
        "multiply" => 0x6A, "add" => 0x6B, "subtract" => 0x6D,
        "decimal" => 0x6E, "divide" => 0x6F,
        "capslock" => 0x14, "numlock" => 0x90, "scrolllock" => 0x91,
        "printscreen" => 0x2C,
        s if s.len() == 1 => {
            let c = s.chars().next().unwrap();
            if c.is_ascii_alphanumeric() { c.to_ascii_uppercase() as u32 } else { 0 }
        }
        _ => 0,
    }
}

#[tauri::command]
pub fn get_clipboard() -> Result<String, AppError> {
    unsafe {
        if OpenClipboard(0) == 0 {
            return Err(AppError::Window("无法打开剪贴板".into()));
        }
        let handle = GetClipboardData(13); // CF_UNICODETEXT
        if handle == 0 {
            CloseClipboard();
            return Ok(String::new());
        }
        let ptr = GlobalLock(handle as *mut _) as *const u16;
        if ptr.is_null() {
            CloseClipboard();
            return Err(AppError::Window("无法锁定剪贴板内存".into()));
        }
        let mut len = 0;
        while *ptr.add(len) != 0 { len += 1; }
        let text = String::from_utf16_lossy(std::slice::from_raw_parts(ptr, len));
        GlobalUnlock(handle as *mut _);
        CloseClipboard();
        Ok(text)
    }
}

#[tauri::command]
pub fn set_clipboard(text: String) -> Result<(), AppError> {
    unsafe {
        if OpenClipboard(0) == 0 {
            return Err(AppError::Window("无法打开剪贴板".into()));
        }
        EmptyClipboard();
        let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let size = wide.len() * 2;
        let hmem = GlobalAlloc(GMEM_MOVEABLE, size);
        if hmem.is_null() {
            CloseClipboard();
            return Err(AppError::Window("内存分配失败".into()));
        }
        let ptr = GlobalLock(hmem) as *mut u16;
        std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr, wide.len());
        GlobalUnlock(hmem);
        SetClipboardData(13, hmem as HANDLE);
        CloseClipboard();
        Ok(())
    }
}

#[tauri::command]
pub fn msg_box(title: String, text: String) -> Result<i32, AppError> {
    let wide_title: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
    let wide_text: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    let result = unsafe { MessageBoxW(0, wide_text.as_ptr(), wide_title.as_ptr(), MB_OK) };
    Ok(result)
}

#[tauri::command]
pub fn capture_window_image(hwnd: isize) -> Result<WindowCaptureResult, AppError> {
    use windows_sys::Win32::Graphics::Gdi::*;

    let hwnd = hwnd as HWND;
    if hwnd == 0 { return Err(AppError::Window("无效的窗口句柄".into())); }

    unsafe {
        let mut rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
        GetClientRect(hwnd, &mut rect);
        let width = (rect.right - rect.left) as u32;
        let height = (rect.bottom - rect.top) as u32;

        if width == 0 || height == 0 {
            return Err(AppError::Window("窗口尺寸为零".into()));
        }

        let hdc_win = GetDC(hwnd);
        let hdc_mem = CreateCompatibleDC(hdc_win);
        let hbmp = CreateCompatibleBitmap(hdc_win, width as i32, height as i32);
        let old = SelectObject(hdc_mem, hbmp);

        // 使用 BitBlt 替代 PrintWindow
        BitBlt(hdc_mem, 0, 0, width as i32, height as i32, hdc_win, 0, 0, SRCCOPY);

        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width as i32,
                biHeight: -(height as i32),
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [RGBQUAD { rgbBlue: 0, rgbGreen: 0, rgbRed: 0, rgbReserved: 0 }],
        };

        let mut buf = vec![0u8; (width * height * 4) as usize];
        GetDIBits(hdc_mem, hbmp, 0, height, buf.as_mut_ptr() as *mut _, &bmi as *const _ as *mut _, DIB_RGB_COLORS);

        SelectObject(hdc_mem, old);
        DeleteObject(hbmp);
        DeleteDC(hdc_mem);
        ReleaseDC(hwnd, hdc_win);

        // BGRA -> RGBA
        for chunk in buf.chunks_mut(4) {
            chunk.swap(0, 2);
        }

        let img = image::RgbaImage::from_raw(width, height, buf)
            .ok_or(AppError::Image("图像转换失败".into()))?;
        let mut out = Vec::new();
        image::DynamicImage::ImageRgba8(img)
            .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
            .map_err(|e| AppError::Image(e.to_string()))?;
        let image_base64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &out);

        Ok(WindowCaptureResult { image_base64, width, height })
    }
}
