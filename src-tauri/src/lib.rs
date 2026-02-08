// KeyLog - 键盘鼠标统计工具
// 主入口库文件

pub mod commands;
pub mod db;
pub mod input;


use std::sync::{Arc, Mutex};

use commands::AppState;
use db::Database;

use tauri::{tray::TrayIconEvent, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // 初始化数据库
    let db = Database::new().expect("Failed to initialize database");
    let app_state = Arc::new(Mutex::new(AppState { db }));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, Some(vec!["--minimized"])))
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::get_today_stats,
            commands::get_date_stats,
            commands::get_top_keys,
            commands::get_date_range_summary,
            commands::start_listening,
            commands::stop_listening,
            commands::get_listening_status,
            commands::get_setting,
            commands::set_setting,
            commands::open_data_dir,
            commands::export_to_csv,
            commands::export_to_excel,
            commands::get_db_path,
            commands::set_db_path,
        ])
        .setup(|app| {
            // 设置托盘事件监听
            let handle = app.handle().clone();
            if let Some(tray) = app.tray_by_id("main") {
                tray.on_tray_icon_event(move |tray, event| {
                    if let TrayIconEvent::Click { .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.unminimize(); // 先取消最小化
                            let _ = window.show();       // 再显示窗口
                            let _ = window.set_focus();  // 最后设置焦点
                        }
                    }
                });
            }

            // 启动时自动开始监听
            std::thread::spawn(move || {
                // 短暂延迟确保应用完全初始化
                std::thread::sleep(std::time::Duration::from_millis(500));
                if let Err(e) = commands::start_listening(handle) {
                    log::error!("Failed to auto start listener: {}", e);
                }
            });
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Resized(_) = event {
                if window.is_minimized().unwrap_or(false) {
                    // 检查是否启用了"最小化到托盘"
                    // 这里我们尝试获取应用状态来读取配置
                    // 注意：这里可能会有一些性能开销，但在窗口最小化这种低频操作中是可以接受的
                    let app_handle = window.app_handle();
                    let state = app_handle.state::<Arc<Mutex<AppState>>>();
                    
                    let should_minimize_to_tray = if let Ok(state_guard) = state.lock() {
                        state_guard.db.get_setting("minimize_to_tray")
                            .unwrap_or(Some("true".to_string())) // 默认为 true
                            .map(|v| v == "true")
                            .unwrap_or(true)
                    } else {
                        true // 获取锁失败时默认为 true
                    };

                    if should_minimize_to_tray {
                        window.hide().unwrap();
                    }
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
