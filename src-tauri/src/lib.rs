// KeyLog - 键盘鼠标统计工具
// 主入口库文件

pub mod commands;
pub mod db;
pub mod input;


use std::sync::{Arc, Mutex};

use commands::AppState;
use db::Database;

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
        ])
        .setup(|app| {
            // 启动时自动开始监听
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                // 短暂延迟确保应用完全初始化
                std::thread::sleep(std::time::Duration::from_millis(500));
                if let Err(e) = commands::start_listening(handle) {
                    log::error!("Failed to auto start listener: {}", e);
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
