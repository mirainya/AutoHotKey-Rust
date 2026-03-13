#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod error;

use commands::{hotkey, screenshot, automation, script, pixel};
#[cfg(target_os = "windows")]
use commands::window;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    Manager,
};

fn init_tracing(app: &tauri::App) {
    use tracing_subscriber::{fmt, EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

    let log_dir = app.path().app_data_dir().unwrap().join("logs");
    let _ = std::fs::create_dir_all(&log_dir);

    let file_appender = tracing_appender::rolling::daily(log_dir, "ahk-rust.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // 把 guard 泄漏到 'static，保证日志写入器不会被 drop
    std::mem::forget(_guard);

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(fmt::layer().with_writer(std::io::stdout).with_target(false))
        .with(fmt::layer().with_writer(non_blocking).with_ansi(false).with_target(false))
        .init();

    tracing::info!("AutoHotKey-Rust 启动");
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            init_tracing(app);

            let show = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, .. } = event {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                })
                .build(app)?;

            // 拦截关闭事件，改为隐藏窗口
            let win = app.get_webview_window("main").unwrap();
            win.clone().on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = win.hide();
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            hotkey::start_hotkey_listener,
            hotkey::stop_hotkey_listener,
            hotkey::register_hotkey,
            hotkey::unregister_hotkey,
            screenshot::capture_screen,
            screenshot::capture_screen_by_index,
            screenshot::get_all_screens,
            automation::move_mouse,
            automation::click_mouse,
            automation::scroll_mouse,
            automation::type_text,
            automation::key_press,
            automation::key_down,
            automation::key_up,
            automation::get_mouse_pos,
            automation::get_screen_size,
            script::execute_script,
            script::stop_script,
            script::save_script,
            script::load_scripts,
            script::delete_script,
            script::get_script_status,
            script::run_script_by_id,
            script::export_script,
            script::import_script,
            pixel::capture_pixels,
            pixel::find_pixel_pattern,
            pixel::find_pattern_in_image,
            pixel::save_pixel_pattern,
            pixel::load_pixel_patterns,
            pixel::delete_pixel_pattern,
            pixel::get_pixel_color,
            window::find_window,
            window::find_window_by_class,
            window::get_foreground_window,
            window::get_window_info,
            window::enum_windows,
            window::activate_window,
            window::move_window,
            window::show_window,
            window::post_click,
            window::post_key,
            window::post_char,
            window::send_keys,
            window::get_clipboard,
            window::set_clipboard,
            window::msg_box,
            window::capture_window_image,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
