#![cfg(target_os = "windows")]

use serde::{Deserialize, Serialize};
use windows_sys::Win32::Foundation::{BOOL, HWND, LPARAM, RECT};
use windows_sys::Win32::UI::WindowsAndMessaging::*;
use windows_sys::Win32::System::DataExchange::{OpenClipboard, CloseClipboard, GetClipboardData, SetClipboardData, EmptyClipboard};
use windows_sys::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
use windows_sys::Win32::System::ProcessStatus::GetProcessImageFileNameW;
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

fn hwnd_to_info(hwnd: HWND) -> WindowInfo {
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
pub fn get_foreground_window() -> Option<isize> {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd != 0 { Some(hwnd as isize) } else { None }
}

#[tauri::command]
pub fn get_window_info(hwnd: isize) -> WindowInfo {
    hwnd_to_info(hwnd as HWND)
}

unsafe extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    if IsWindowVisible(hwnd) == 0 { return 1; }
    let title_len = GetWindowTextLengthW(hwnd);
    if title_len == 0 { return 1; }
    let list = &mut *(lparam as *mut Vec<WindowInfo>);
    list.push(hwnd_to_info(hwnd));
    1
}

#[tauri::command]
pub fn enum_windows() -> Vec<WindowInfo> {
    let mut list: Vec<WindowInfo> = Vec::new();
    unsafe { EnumWindows(Some(enum_callback), &mut list as *mut _ as LPARAM); }
    list
}

#[tauri::command]
pub fn activate_window(hwnd: isize) -> Result<(), String> {
    unsafe { SetForegroundWindow(hwnd as HWND); }
    Ok(())
}

#[tauri::command]
pub fn move_window(hwnd: isize, x: i32, y: i32, w: i32, h: i32) -> Result<(), String> {
    unsafe { MoveWindow(hwnd as HWND, x, y, w, h, 1); }
    Ok(())
}

#[tauri::command]
pub fn show_window(hwnd: isize, state: String) -> Result<(), String> {
    let cmd = match state.as_str() {
        "minimize" => SW_MINIMIZE,
        "maximize" => SW_MAXIMIZE,
        "hide" => SW_HIDE,
        "restore" | _ => SW_RESTORE,
    };
    unsafe { ShowWindow(hwnd as HWND, cmd); }
    Ok(())
}

#[tauri::command]
pub fn post_click(hwnd: isize, x: i32, y: i32, button: String) -> Result<(), String> {
    let lparam = ((y as u32 as usize) << 16 | (x as u32 as usize)) as LPARAM;
    let (down, up) = match button.as_str() {
        "right" => (WM_RBUTTONDOWN, WM_RBUTTONUP),
        _ => (WM_LBUTTONDOWN, WM_LBUTTONUP),
    };
    unsafe {
        PostMessageW(hwnd as HWND, down, 0, lparam);
        PostMessageW(hwnd as HWND, up, 0, lparam);
    }
    Ok(())
}

#[tauri::command]
pub fn post_key(hwnd: isize, vk_code: u32) -> Result<(), String> {
    let lparam = 1isize;
    unsafe {
        PostMessageW(hwnd as HWND, WM_KEYDOWN, vk_code as usize, lparam);
        PostMessageW(hwnd as HWND, WM_KEYUP, vk_code as usize, lparam | (1 << 31));
    }
    Ok(())
}

#[tauri::command]
pub fn post_char(hwnd: isize, ch: String) -> Result<(), String> {
    for c in ch.chars() {
        unsafe { PostMessageW(hwnd as HWND, WM_CHAR, c as usize, 1); }
    }
    Ok(())
}

#[tauri::command]
pub fn get_clipboard() -> String {
    unsafe {
        if OpenClipboard(0) == 0 { return String::new(); }
        let h = GetClipboardData(13);
        if h == 0 { CloseClipboard(); return String::new(); }
        let ptr = GlobalLock(h as *mut _) as *const u16;
        let mut len = 0;
        while *ptr.add(len) != 0 { len += 1; }
        let s = String::from_utf16_lossy(std::slice::from_raw_parts(ptr, len));
        GlobalUnlock(h as *mut _);
        CloseClipboard();
        s
    }
}

#[tauri::command]
pub fn set_clipboard(text: String) -> Result<(), String> {
    unsafe {
        if OpenClipboard(0) == 0 { return Err("无法打开剪贴板".into()); }
        EmptyClipboard();
        let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let hmem = GlobalAlloc(GMEM_MOVEABLE, wide.len() * 2);
        if hmem.is_null() { CloseClipboard(); return Err("内存分配失败".into()); }
        let ptr = GlobalLock(hmem) as *mut u16;
        std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr, wide.len());
        GlobalUnlock(hmem);
        SetClipboardData(13, hmem as HANDLE);
        CloseClipboard();
        Ok(())
    }
}

#[tauri::command]
pub fn msg_box(text: String) {
    let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe { MessageBoxW(0, wide.as_ptr(), std::ptr::null(), MB_OK); }
}

#[derive(Debug, Serialize)]
pub struct WindowCaptureResult {
    pub image_base64: String,
    pub width: u32,
    pub height: u32,
}

#[tauri::command]
pub fn capture_window_image(hwnd: isize) -> Result<WindowCaptureResult, String> {
    use windows_sys::Win32::Graphics::Gdi::*;
    unsafe {
        let hwnd = hwnd as windows_sys::Win32::Foundation::HWND;
        let mut rect = windows_sys::Win32::Foundation::RECT { left: 0, top: 0, right: 0, bottom: 0 };
        windows_sys::Win32::UI::WindowsAndMessaging::GetClientRect(hwnd, &mut rect);
        let width = (rect.right - rect.left) as u32;
        let height = (rect.bottom - rect.top) as u32;
        if width == 0 || height == 0 { return Err("窗口尺寸为零".into()); }

        let hdc_win = GetDC(hwnd);
        let hdc_mem = CreateCompatibleDC(hdc_win);
        let hbmp = CreateCompatibleBitmap(hdc_win, width as i32, height as i32);
        let old = SelectObject(hdc_mem, hbmp);
        windows_sys::Win32::Storage::Xps::PrintWindow(hwnd, hdc_mem, 2); // PW_CLIENTONLY = 2
        let mut bmi = BITMAPINFO {
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
        GetDIBits(hdc_mem, hbmp, 0, height, buf.as_mut_ptr() as *mut _, &mut bmi, DIB_RGB_COLORS);

        SelectObject(hdc_mem, old);
        DeleteObject(hbmp);
        DeleteDC(hdc_mem);
        ReleaseDC(hwnd, hdc_win);

        // BGRA -> RGBA
        for chunk in buf.chunks_mut(4) {
            chunk.swap(0, 2);
        }

        let img = image::RgbaImage::from_raw(width, height, buf)
            .ok_or("图像转换失败")?;
        let mut out = Vec::new();
        image::DynamicImage::ImageRgba8(img)
            .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
            .map_err(|e| e.to_string())?;
        let image_base64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &out);

        Ok(WindowCaptureResult { image_base64, width, height })
    }
}
