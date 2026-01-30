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

    /// 获取数据库文件路径
    fn get_db_path() -> Result<PathBuf, DbError> {
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
