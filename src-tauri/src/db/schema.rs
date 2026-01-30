// 数据库表结构定义

use serde::{Deserialize, Serialize};

/// 按键统计记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyStat {
    pub key_name: String,
    pub key_code: Option<i32>,
    pub count: i64,
}

/// 鼠标统计记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseStat {
    pub button: String,
    pub count: i64,
}

/// 每日统计汇总
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaySummary {
    pub date: String,
    pub total_keys: i64,
    pub total_clicks: i64,
}

/// 今日完整统计数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodayStats {
    pub date: String,
    pub total_keys: i64,
    pub total_clicks: i64,
    pub top_keys: Vec<KeyStat>,
    pub mouse_stats: Vec<MouseStat>,
}

/// 日期统计数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateStats {
    pub date: String,
    pub total_keys: i64,
    pub total_clicks: i64,
    pub key_stats: Vec<KeyStat>,
    pub mouse_stats: Vec<MouseStat>,
}
