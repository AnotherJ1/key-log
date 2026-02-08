// 数据库模块

mod schema;
mod operations;

pub use schema::*;


use rusqlite::{Connection, Result};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Database not initialized")]
    NotInitialized,
}

/// 数据库管理器
pub struct Database {
    conn: Connection,
}

impl Database {
    /// 创建新的数据库连接
    pub fn new() -> Result<Self, DbError> {
        let db_path = Self::get_db_path()?;
        
        // 确保目录存在
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)?;
        let db = Self { conn };
        db.init_tables()?;
        
        log::info!("Database initialized at: {:?}", db_path);
        Ok(db)
    }

    /// 获取配置文件的路径
    fn get_config_path() -> PathBuf {
        let config_dir = dirs::config_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("KeyLog");
        config_dir.join("config.json")
    }

    /// 获取当前数据库路径（供外部查询）
    pub fn get_current_db_path() -> String {
        Self::get_db_path().map(|p| p.to_string_lossy().to_string()).unwrap_or_default()
    }

    /// 更新数据库路径并重新连接
    pub fn set_db_path(new_path: &str) -> Result<(), DbError> {
        let path = PathBuf::from(new_path);
        
        // 1. 验证路径有效性
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        // 2. 更新配置文件
        let config_path = Self::get_config_path();
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let mut config = serde_json::Map::new();
        if config_path.exists() {
             if let Ok(content) = std::fs::read_to_string(&config_path) {
                 if let Ok(serde_json::Value::Object(map)) = serde_json::from_str(&content) {
                     config = map;
                 }
             }
        }
        
        config.insert("db_path".to_string(), serde_json::Value::String(new_path.to_string()));
        let content = serde_json::to_string_pretty(&config).unwrap();
        std::fs::write(config_path, content)?;
        
        Ok(())
    }

    /// 创建新的数据库连接（支持指定路径）
    pub fn open_at_path(path_str: &str) -> Result<Self, DbError> {
        let path = PathBuf::from(path_str);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(&path)?;
        let db = Self { conn };
        db.init_tables()?;
        Ok(db)
    }

    /// 获取数据库文件路径
    fn get_db_path() -> Result<PathBuf, DbError> {
        let config_file = Self::get_config_path();
        
        // 尝试读取配置文件中的数据库路径
        if config_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_file) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(db_path_str) = json.get("db_path").and_then(|v| v.as_str()) {
                        let path = PathBuf::from(db_path_str);
                        // 确保路径有效且父目录存在
                        if let Some(parent) = path.parent() {
                            if std::fs::create_dir_all(parent).is_ok() {
                                return Ok(path);
                            }
                        }
                    }
                }
            }
        }
        
        // 默认路径
        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("KeyLog");
        Ok(data_dir.join("keylog.db"))
    }

    /// 初始化数据表
    fn init_tables(&self) -> Result<(), DbError> {
        // 每日按键统计表
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS daily_key_stats (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                date TEXT NOT NULL,
                key_name TEXT NOT NULL,
                key_code INTEGER,
                count INTEGER DEFAULT 0,
                UNIQUE(date, key_name)
            )",
            [],
        )?;

        // 每日鼠标统计表
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS daily_mouse_stats (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                date TEXT NOT NULL,
                button TEXT NOT NULL,
                count INTEGER DEFAULT 0,
                UNIQUE(date, button)
            )",
            [],
        )?;

        // 每日汇总表
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS daily_summary (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                date TEXT NOT NULL UNIQUE,
                total_keys INTEGER DEFAULT 0,
                total_clicks INTEGER DEFAULT 0
            )",
            [],
        )?;

        // 配置信息表
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;

        // 初始化默认设置
        self.conn.execute(
            "INSERT OR IGNORE INTO settings (key, value) VALUES ('minimize_to_tray', 'true')",
            [],
        )?;

        // 创建索引以提高查询性能
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_key_stats_date ON daily_key_stats(date)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_mouse_stats_date ON daily_mouse_stats(date)",
            [],
        )?;

        Ok(())
    }

    /// 获取数据库连接引用
    pub fn conn(&self) -> &Connection {
        &self.conn
    }
}
