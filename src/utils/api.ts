// Tauri API 类型和调用封装

import { invoke } from "@tauri-apps/api/core";

// 类型定义
export interface KeyStat {
    key_name: string;
    key_code: number | null;
    count: number;
}

export interface MouseStat {
    button: string;
    count: number;
}

export interface TodayStats {
    date: string;
    total_keys: number;
    total_clicks: number;
    top_keys: KeyStat[];
    mouse_stats: MouseStat[];
}

export interface DateStats {
    date: string;
    total_keys: number;
    total_clicks: number;
    key_stats: KeyStat[];
    mouse_stats: MouseStat[];
}

export interface DaySummary {
    date: string;
    total_keys: number;
    total_clicks: number;
}

// API 调用
export async function getTodayStats(): Promise<TodayStats> {
    return await invoke("get_today_stats");
}

export async function getDateStats(date: string): Promise<DateStats> {
    return await invoke("get_date_stats", { date });
}

export async function getTopKeys(date: string, limit: number): Promise<KeyStat[]> {
    return await invoke("get_top_keys", { date, limit });
}

export async function getDateRangeSummary(start: string, end: string): Promise<DaySummary[]> {
    return await invoke("get_date_range_summary", { start, end });
}

export async function startListening(): Promise<void> {
    return await invoke("start_listening");
}

export async function stopListening(): Promise<void> {
    return await invoke("stop_listening");
}

export async function getListeningStatus(): Promise<boolean> {
    return await invoke("get_listening_status");
}

// Autostart 插件
export async function isAutostartEnabled(): Promise<boolean> {
    return await invoke("plugin:autostart|is_enabled");
}

export async function enableAutostart(): Promise<void> {
    return await invoke("plugin:autostart|enable");
}

export async function disableAutostart(): Promise<void> {
    return await invoke("plugin:autostart|disable");
}

// 设置 API
export async function getSetting(key: string): Promise<string | null> {
    return await invoke("get_setting", { key });
}

export async function setSetting(key: string, value: string): Promise<void> {
    return await invoke("set_setting", { key, value });
}

// 文件操作 API
export async function openDataDir(): Promise<void> {
    return await invoke("open_data_dir");
}

export async function exportToCsv(startDate: string, endDate: string, filePath: string): Promise<void> {
    return await invoke("export_to_csv", { startDate, endDate, filePath });
}

export async function exportToExcel(startDate: string, endDate: string, filePath: string): Promise<void> {
    return await invoke("export_to_excel", { startDate, endDate, filePath });
}
