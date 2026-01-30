// 数据库操作

use super::{Database, DbError, KeyStat, MouseStat, DaySummary, TodayStats, DateStats};
use chrono::Local;
use rusqlite::params;

impl Database {
    /// 获取今日日期字符串
    fn today() -> String {
        Local::now().format("%Y-%m-%d").to_string()
    }

    /// 增加按键计数
    pub fn increment_key(&self, key_name: &str, key_code: Option<i32>) -> Result<(), DbError> {
        let date = Self::today();
        
        self.conn.execute(
            "INSERT INTO daily_key_stats (date, key_name, key_code, count)
             VALUES (?1, ?2, ?3, 1)
             ON CONFLICT(date, key_name) DO UPDATE SET count = count + 1",
            params![date, key_name, key_code],
        )?;

        // 更新汇总
        self.update_summary_keys(&date)?;
        
        Ok(())
    }

    /// 增加鼠标点击计数
    pub fn increment_mouse(&self, button: &str) -> Result<(), DbError> {
        let date = Self::today();
        
        self.conn.execute(
            "INSERT INTO daily_mouse_stats (date, button, count)
             VALUES (?1, ?2, 1)
             ON CONFLICT(date, button) DO UPDATE SET count = count + 1",
            params![date, button],
        )?;

        // 更新汇总
        self.update_summary_clicks(&date)?;
        
        Ok(())
    }

    /// 更新每日按键汇总
    fn update_summary_keys(&self, date: &str) -> Result<(), DbError> {
        let total: i64 = self.conn.query_row(
            "SELECT COALESCE(SUM(count), 0) FROM daily_key_stats WHERE date = ?1",
            params![date],
            |row| row.get(0),
        )?;

        self.conn.execute(
            "INSERT INTO daily_summary (date, total_keys, total_clicks)
             VALUES (?1, ?2, 0)
             ON CONFLICT(date) DO UPDATE SET total_keys = ?2",
            params![date, total],
        )?;

        Ok(())
    }

    /// 更新每日鼠标点击汇总
    fn update_summary_clicks(&self, date: &str) -> Result<(), DbError> {
        let total: i64 = self.conn.query_row(
            "SELECT COALESCE(SUM(count), 0) FROM daily_mouse_stats WHERE date = ?1",
            params![date],
            |row| row.get(0),
        )?;

        self.conn.execute(
            "INSERT INTO daily_summary (date, total_keys, total_clicks)
             VALUES (?1, 0, ?2)
             ON CONFLICT(date) DO UPDATE SET total_clicks = ?2",
            params![date, total],
        )?;

        Ok(())
    }

    /// 获取今日统计
    pub fn get_today_stats(&self) -> Result<TodayStats, DbError> {
        let date = Self::today();
        self.get_date_stats(&date)
            .map(|stats| TodayStats {
                date: stats.date,
                total_keys: stats.total_keys,
                total_clicks: stats.total_clicks,
                top_keys: stats.key_stats,
                mouse_stats: stats.mouse_stats,
            })
    }

    /// 获取指定日期统计
    pub fn get_date_stats(&self, date: &str) -> Result<DateStats, DbError> {
        // 获取汇总
        let (total_keys, total_clicks) = self.conn.query_row(
            "SELECT COALESCE(total_keys, 0), COALESCE(total_clicks, 0) 
             FROM daily_summary WHERE date = ?1",
            params![date],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
        ).unwrap_or((0, 0));

        // 获取按键统计
        let mut stmt = self.conn.prepare(
            "SELECT key_name, key_code, count FROM daily_key_stats 
             WHERE date = ?1 ORDER BY count DESC"
        )?;
        let key_stats: Vec<KeyStat> = stmt.query_map(params![date], |row| {
            Ok(KeyStat {
                key_name: row.get(0)?,
                key_code: row.get(1)?,
                count: row.get(2)?,
            })
        })?.filter_map(|r| r.ok()).collect();

        // 获取鼠标统计
        let mut stmt = self.conn.prepare(
            "SELECT button, count FROM daily_mouse_stats WHERE date = ?1"
        )?;
        let mouse_stats: Vec<MouseStat> = stmt.query_map(params![date], |row| {
            Ok(MouseStat {
                button: row.get(0)?,
                count: row.get(1)?,
            })
        })?.filter_map(|r| r.ok()).collect();

        Ok(DateStats {
            date: date.to_string(),
            total_keys,
            total_clicks,
            key_stats,
            mouse_stats,
        })
    }

    /// 获取按键排行榜
    pub fn get_top_keys(&self, date: &str, limit: i32) -> Result<Vec<KeyStat>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT key_name, key_code, count FROM daily_key_stats 
             WHERE date = ?1 ORDER BY count DESC LIMIT ?2"
        )?;
        
        let keys: Vec<KeyStat> = stmt.query_map(params![date, limit], |row| {
            Ok(KeyStat {
                key_name: row.get(0)?,
                key_code: row.get(1)?,
                count: row.get(2)?,
            })
        })?.filter_map(|r| r.ok()).collect();

        Ok(keys)
    }

    /// 获取日期范围的汇总统计
    pub fn get_date_range_summary(&self, start: &str, end: &str) -> Result<Vec<DaySummary>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT date, total_keys, total_clicks FROM daily_summary 
             WHERE date BETWEEN ?1 AND ?2 ORDER BY date DESC"
        )?;

        let summaries: Vec<DaySummary> = stmt.query_map(params![start, end], |row| {
            Ok(DaySummary {
                date: row.get(0)?,
                total_keys: row.get(1)?,
                total_clicks: row.get(2)?,
            })
        })?.filter_map(|r| r.ok()).collect();

        Ok(summaries)
    }

    /// 获取指定日期范围的聚合统计
    pub fn get_range_stats(&self, start: &str, end: &str) -> Result<DateStats, DbError> {
        // 获取范围内的总计
        let (total_keys, total_clicks) = self.conn.query_row(
            "SELECT COALESCE(SUM(total_keys), 0), COALESCE(SUM(total_clicks), 0) 
             FROM daily_summary WHERE date BETWEEN ?1 AND ?2",
            params![start, end],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
        ).unwrap_or((0, 0));

        // 获取范围内的按键聚合统计
        let mut stmt = self.conn.prepare(
            "SELECT key_name, MAX(key_code), SUM(count) as total_count FROM daily_key_stats 
             WHERE date BETWEEN ?1 AND ?2 GROUP BY key_name ORDER BY total_count DESC"
        )?;
        let key_stats: Vec<KeyStat> = stmt.query_map(params![start, end], |row| {
            Ok(KeyStat {
                key_name: row.get(0)?,
                key_code: row.get(1)?,
                count: row.get(2)?,
            })
        })?.filter_map(|r| r.ok()).collect();

        // 获取范围内的鼠标聚合统计
        let mut stmt = self.conn.prepare(
            "SELECT button, SUM(count) as total_count FROM daily_mouse_stats 
             WHERE date BETWEEN ?1 AND ?2 GROUP BY button ORDER BY total_count DESC"
        )?;
        let mouse_stats: Vec<MouseStat> = stmt.query_map(params![start, end], |row| {
            Ok(MouseStat {
                button: row.get(0)?,
                count: row.get(1)?,
            })
        })?.filter_map(|r| r.ok()).collect();

        Ok(DateStats {
            date: format!("{} 至 {}", start, end),
            total_keys,
            total_clicks,
            key_stats,
            mouse_stats,
        })
    }

    /// 获取配置信息
    pub fn get_setting(&self, key: &str) -> Result<Option<String>, DbError> {
        let result = self.conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        );

        match result {
            Ok(val) => Ok(Some(val)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DbError::Sqlite(e)),
        }
    }

    /// 设置配置信息
    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT INTO settings (key, value)
             VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = ?2",
            params![key, value],
        )?;
        Ok(())
    }
}
