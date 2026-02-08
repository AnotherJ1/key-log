// Tauri 命令模块

use crate::db::{Database, TodayStats, DateStats, KeyStat, DaySummary};
use crate::input::{start_global_listener, stop_global_listener, is_listener_running, InputEvent};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, State};

use tauri_plugin_opener::OpenerExt;
use std::fs::File;
use std::io::Write;
use rust_xlsxwriter::*;

/// 应用状态
pub struct AppState {
    pub db: Database,
}

type SharedState = Arc<Mutex<AppState>>;

/// 获取当前数据库路径
#[tauri::command]
pub fn get_db_path() -> Result<String, String> {
    Ok(Database::get_current_db_path())
}

/// 设置数据库路径
#[tauri::command]
pub fn set_db_path(state: State<'_, SharedState>, new_path: String) -> Result<(), String> {
    // 1. 更新配置文件
    Database::set_db_path(&new_path).map_err(|e| e.to_string())?;
    
    // 2. 重新初始化数据库连接
    let mut state_guard = state.lock().map_err(|e| e.to_string())?;
    let new_db = Database::open_at_path(&new_path).map_err(|e| e.to_string())?;
    state_guard.db = new_db;
    
    log::info!("Database re-initialized at new path: {}", new_path);
    Ok(())
}

/// 打开数据目录
#[tauri::command]
pub fn open_data_dir(handle: AppHandle) -> Result<(), String> {
    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("KeyLog");
    
    if !data_dir.exists() {
        std::fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
    }

    handle.opener().open_path(data_dir.to_string_lossy().to_string(), None::<String>)
        .map_err(|e| e.to_string())
}

/// 导出数据到 Excel (每个日期一个 Sheet)
#[tauri::command]
pub fn export_to_excel(
    state: State<'_, SharedState>,
    start_date: String,
    end_date: String,
    file_path: String,
) -> Result<(), String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    
    let mut workbook = Workbook::new();
    
    // 获取日期范围内的所有日期
    let start = chrono::NaiveDate::parse_from_str(&start_date, "%Y-%m-%d").map_err(|e| e.to_string())?;
    let end = chrono::NaiveDate::parse_from_str(&end_date, "%Y-%m-%d").map_err(|e| e.to_string())?;
    
    let mut current = start;
    let mut sheet_count = 0;

    while current <= end {
        let date_str = current.format("%Y-%m-%d").to_string();
        let stats = state.db.get_date_stats(&date_str).map_err(|e| e.to_string())?;
        
        // 只有有数据的日期才导出
        if stats.total_keys > 0 || stats.total_clicks > 0 {
            let worksheet = workbook.add_worksheet();
            worksheet.set_name(&date_str).map_err(|e| e.to_string())?;
            
            // 设置标题样式
            let header_format = Format::new()
                .set_bold()
                .set_background_color(Color::Gray)
                .set_font_color(Color::White);
            
            // 写入汇总
            worksheet.write_string_with_format(0, 0, "项目", &header_format).map_err(|e| e.to_string())?;
            worksheet.write_string_with_format(0, 1, "数值", &header_format).map_err(|e| e.to_string())?;
            
            worksheet.write_string(1, 0, "日期").map_err(|e| e.to_string())?;
            worksheet.write_string(1, 1, &stats.date).map_err(|e| e.to_string())?;
            
            worksheet.write_string(2, 0, "总按键次数").map_err(|e| e.to_string())?;
            worksheet.write_number(2, 1, stats.total_keys as f64).map_err(|e| e.to_string())?;
            
            worksheet.write_string(3, 0, "总鼠标点击").map_err(|e| e.to_string())?;
            worksheet.write_number(3, 1, stats.total_clicks as f64).map_err(|e| e.to_string())?;
            
            // 写入按键统计
            let mut row = 5;
            worksheet.write_string_with_format(row, 0, "按键名称", &header_format).map_err(|e| e.to_string())?;
            worksheet.write_string_with_format(row, 1, "次数", &header_format).map_err(|e| e.to_string())?;
            row += 1;
            
            for key in stats.key_stats {
                worksheet.write_string(row, 0, &key.key_name).map_err(|e| e.to_string())?;
                worksheet.write_number(row, 1, key.count as f64).map_err(|e| e.to_string())?;
                row += 1;
            }
            
            // 写入鼠标统计
            row += 1;
            worksheet.write_string_with_format(row, 0, "鼠标按键", &header_format).map_err(|e| e.to_string())?;
            worksheet.write_string_with_format(row, 1, "次数", &header_format).map_err(|e| e.to_string())?;
            row += 1;
            
            for mouse in stats.mouse_stats {
                worksheet.write_string(row, 0, &mouse.button).map_err(|e| e.to_string())?;
                worksheet.write_number(row, 1, mouse.count as f64).map_err(|e| e.to_string())?;
                row += 1;
            }
            
            sheet_count += 1;
        }
        
        current = current.succ_opt().ok_or("Date calculation error")?;
    }
    
    if sheet_count == 0 {
        return Err("选定日期范围内没有任何记录".to_string());
    }
    
    workbook.save(&file_path).map_err(|e| e.to_string())?;
    
    Ok(())
}

/// 导出数据到 CSV
#[tauri::command]
pub fn export_to_csv(
    state: State<'_, SharedState>,
    start_date: String,
    end_date: String,
    file_path: String,
) -> Result<(), String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    
    // 获取统计数据
    let stats = if start_date == end_date {
        state.db.get_date_stats(&start_date).map_err(|e| e.to_string())?
    } else {
        state.db.get_range_stats(&start_date, &end_date).map_err(|e| e.to_string())?
    };
    
    let mut file = File::create(file_path).map_err(|e| e.to_string())?;
    
    // 写入 BOM 以支持 Excel 中文显示
    file.write_all(&[0xEF, 0xBB, 0xBF]).map_err(|e| e.to_string())?;
    
    // 写入汇总信息
    writeln!(file, "日期,{}", stats.date).map_err(|e| e.to_string())?;
    writeln!(file, "总按键次数,{}", stats.total_keys).map_err(|e| e.to_string())?;
    writeln!(file, "总鼠标点击,{}", stats.total_clicks).map_err(|e| e.to_string())?;
    writeln!(file, "").map_err(|e| e.to_string())?;
    
    // 写入按键详细统计
    writeln!(file, "按键统计").map_err(|e| e.to_string())?;
    writeln!(file, "按键名称,次数").map_err(|e| e.to_string())?;
    for key in stats.key_stats {
        writeln!(file, "{},{}", key.key_name, key.count).map_err(|e| e.to_string())?;
    }
    writeln!(file, "").map_err(|e| e.to_string())?;
    
    // 写入鼠标详细统计
    writeln!(file, "鼠标统计").map_err(|e| e.to_string())?;
    writeln!(file, "按键,次数").map_err(|e| e.to_string())?;
    for mouse in stats.mouse_stats {
        writeln!(file, "{},{}", mouse.button, mouse.count).map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

/// 获取今日统计
#[tauri::command]
pub fn get_today_stats(state: State<'_, SharedState>) -> Result<TodayStats, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    state.db.get_today_stats().map_err(|e| e.to_string())
}

/// 获取指定日期统计
#[tauri::command]
pub fn get_date_stats(
    state: State<'_, SharedState>,
    date: String,
) -> Result<DateStats, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    state.db.get_date_stats(&date).map_err(|e| e.to_string())
}

/// 获取按键排行榜
#[tauri::command]
pub fn get_top_keys(
    state: State<'_, SharedState>,
    date: String,
    limit: i32,
) -> Result<Vec<KeyStat>, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    state.db.get_top_keys(&date, limit).map_err(|e| e.to_string())
}

/// 获取日期范围汇总
#[tauri::command]
pub fn get_date_range_summary(
    state: State<'_, SharedState>,
    start: String,
    end: String,
) -> Result<Vec<DaySummary>, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    state.db.get_date_range_summary(&start, &end).map_err(|e| e.to_string())
}

/// 启动监听
#[tauri::command]
pub fn start_listening(handle: AppHandle) -> Result<(), String> {
    if is_listener_running() {
        return Ok(());
    }

    let (tx, rx) = mpsc::channel::<InputEvent>();
    
    // 启动监听线程
    start_global_listener(tx)?;
    
    // 启动事件处理线程
    let state: State<'_, SharedState> = handle.state();
    let state_clone = state.inner().clone();
    
    std::thread::spawn(move || {
        log::info!("Event processing thread started");
        
        let mut buffer = Vec::new();
        let buffer_limit = 50; // 缓冲区大小增加到 50
        let flush_timeout = std::time::Duration::from_secs(3); // 刷新超时增加到 3 秒
        let mut last_flush = std::time::Instant::now();

        loop {
            // 使用 recv_timeout 实现定期刷新
            let event_result = rx.recv_timeout(std::time::Duration::from_millis(100));
            
            match event_result {
                Ok(event) => {
                    buffer.push(event);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // 超时，继续检查是否需要 flush
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    log::info!("Event channel closed, stopping processing");
                    break;
                }
            }

            // 检查是否需要写入数据库
            let should_flush = !buffer.is_empty() && 
                (buffer.len() >= buffer_limit || last_flush.elapsed() >= flush_timeout);

            if should_flush {
                if let Ok(state) = state_clone.lock() {
                    let count = buffer.len();
                    for event in buffer.drain(..) {
                        match event {
                            InputEvent::KeyPress { key_name, key_code } => {
                                if let Err(e) = state.db.increment_key(&key_name, key_code) {
                                    log::error!("Failed to increment key: {}", e);
                                }
                            }
                            InputEvent::MouseClick { button } => {
                                if let Err(e) = state.db.increment_mouse(&button) {
                                    log::error!("Failed to increment mouse click: {}", e);
                                }
                            }
                        }
                    }
                    if count > 0 {
                        // 仅在大量写入时打印日志，避免刷屏
                        log::debug!("Flushed {} events to database", count);
                    }
                    last_flush = std::time::Instant::now();
                } else {
                    log::warn!("Failed to request lock for database write");
                }
            }
        }
    });

    log::info!("Input listening started");
    Ok(())
}

/// 停止监听
#[tauri::command]
pub fn stop_listening() -> Result<(), String> {
    stop_global_listener();
    log::info!("Input listening stopped");
    Ok(())
}

/// 获取监听状态
#[tauri::command]
pub fn get_listening_status() -> Result<bool, String> {
    Ok(is_listener_running())
}

/// 获取配置
#[tauri::command]
pub fn get_setting(state: State<'_, SharedState>, key: String) -> Result<Option<String>, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    state.db.get_setting(&key).map_err(|e| e.to_string())
}

/// 设置配置
#[tauri::command]
pub fn set_setting(state: State<'_, SharedState>, key: String, value: String) -> Result<(), String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    state.db.set_setting(&key, &value).map_err(|e| e.to_string())
}
