// 配置模块 — 读取/写入 JSON 配置文件
// 将在 Step 2 实现

use serde::{Deserialize, Serialize};
use std::path::Path;
use chrono::Local;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub version: u32,
    pub categories: Vec<Category>,
    pub schedule: Schedule,
    pub exclusions: Exclusions,
    pub settings: Settings,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub extensions: Vec<String>,
    pub name_patterns: Vec<String>, // 文件名关键词：含这些字的文件归入此类
    pub target_path: String,        // 目标路径：空=桌面，可指定如 D:/图片
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Schedule {
    pub enabled: bool,
    #[serde(rename = "type")]
    pub schedule_type: String,
    pub interval_hours: u32,
    pub daily_time: String,
    pub weekly_day: String,
    pub weekly_time: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Exclusions {
    pub file_names: Vec<String>,
    pub folder_names: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub auto_start: bool,
    pub minimize_to_tray: bool,
    pub theme: String,
    pub shortcut: String,                 // 全局快捷键，如 "Ctrl+Shift+C"
    pub organize_target_path: String,     // 空=桌面，可指定 D:/整理 等
    pub onboarding_completed: bool,       // 新手引导是否完成
    pub language: String,                 // 界面语言 "zh-CN" / "en"
    pub compact_after_organize: bool,     // 整理后自动补齐空位
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: 1,
            categories: vec![
                Category { id: "images".into(), name: "图片".into(), icon: "Image".into(), extensions: vec![".jpg".into(), ".jpeg".into(), ".png".into(), ".gif".into(), ".bmp".into(), ".webp".into(), ".svg".into(), ".ico".into(), ".tiff".into()], name_patterns: vec![], target_path: "".into(), enabled: true },
                Category { id: "documents".into(), name: "文档".into(), icon: "FileText".into(), extensions: vec![".doc".into(), ".docx".into(), ".xls".into(), ".xlsx".into(), ".ppt".into(), ".pptx".into(), ".pdf".into(), ".txt".into(), ".md".into(), ".csv".into()], name_patterns: vec![], target_path: "".into(), enabled: true },
                Category { id: "videos".into(), name: "视频".into(), icon: "Video".into(), extensions: vec![".mp4".into(), ".avi".into(), ".mkv".into(), ".mov".into(), ".wmv".into(), ".flv".into(), ".webm".into()], name_patterns: vec![], target_path: "".into(), enabled: true },
                Category { id: "music".into(), name: "音乐".into(), icon: "Music".into(), extensions: vec![".mp3".into(), ".wav".into(), ".flac".into(), ".aac".into(), ".ogg".into(), ".wma".into()], name_patterns: vec![], target_path: "".into(), enabled: true },
                Category { id: "archives".into(), name: "压缩包".into(), icon: "Archive".into(), extensions: vec![".zip".into(), ".rar".into(), ".7z".into(), ".tar".into(), ".gz".into(), ".bz2".into()], name_patterns: vec![], target_path: "".into(), enabled: true },
                Category { id: "executables".into(), name: "可执行文件".into(), icon: "Terminal".into(), extensions: vec![".exe".into(), ".msi".into(), ".bat".into(), ".cmd".into(), ".ps1".into()], name_patterns: vec![], target_path: "".into(), enabled: true },
                Category { id: "others".into(), name: "其他".into(), icon: "FolderOpen".into(), extensions: vec![], name_patterns: vec![], target_path: "".into(), enabled: true },
            ],
            schedule: Schedule {
                enabled: false,
                schedule_type: "interval".into(),
                interval_hours: 6,
                daily_time: "09:00".into(),
                weekly_day: "Monday".into(),
                weekly_time: "09:00".into(),
            },
            exclusions: Exclusions {
                file_names: vec!["desktop.ini".into()],
                folder_names: vec![],
            },
            settings: Settings {
                auto_start: false,
                minimize_to_tray: true,
                theme: "system".into(),
                shortcut: "Ctrl+Shift+C".into(),
                organize_target_path: "".into(),
                onboarding_completed: false,
                language: "zh-CN".into(),
                compact_after_organize: false,
            },
        }
    }
}

/// 从 config.json 读取配置。
/// 解析失败时备份原文件（保留现场供用户恢复），再回退到默认配置；
/// 解析成功则做 deep-merge：以默认配置为基底叠加已保存字段，
/// 这样旧版本/部分字段缺失的配置文件也能正常加载，而不会整体丢失用户设置。
fn read_config(config_dir: &Path) -> Option<AppConfig> {
    let config_file = config_dir.join("config.json");
    if !config_file.exists() {
        return None;
    }
    let content = match std::fs::read_to_string(&config_file) {
        Ok(c) => c,
        Err(_) => return None,
    };
    let parsed: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("配置解析失败，已备份原文件: {}", e);
            backup_corrupt(config_dir, &content);
            return None;
        }
    };
    let default = serde_json::to_value(AppConfig::default()).unwrap_or(serde_json::Value::Null);
    let merged = deep_merge(default, parsed);
    match serde_json::from_value::<AppConfig>(merged) {
        Ok(cfg) => Some(cfg),
        Err(e) => {
            eprintln!("配置合并后解析失败，已备份原文件: {}", e);
            backup_corrupt(config_dir, &content);
            None
        }
    }
}

/// 把损坏的配置文件改名备份，避免直接覆盖导致用户设置永久丢失
fn backup_corrupt(config_dir: &Path, content: &str) {
    let ts = Local::now().format("%Y%m%d_%H%M%S");
    let bak = config_dir.join(format!("config.json.corrupt-{}", ts));
    let _ = std::fs::write(&bak, content);
}

/// 深合并两个 JSON：对象递归合并，数组/标量以 override 为准
fn deep_merge(base: serde_json::Value, override_val: serde_json::Value) -> serde_json::Value {
    match (base, override_val) {
        (serde_json::Value::Object(mut b), serde_json::Value::Object(o)) => {
            for (k, v) in o {
                let base_v = b.get(&k).cloned().unwrap_or(serde_json::Value::Null);
                b.insert(k, deep_merge(base_v, v));
            }
            serde_json::Value::Object(b)
        }
        (_, o) => o,
    }
}

pub fn ensure_config(config_dir: &Path) -> AppConfig {
    if let Some(cfg) = read_config(config_dir) {
        return cfg;
    }
    let config = AppConfig::default();
    let config_file = config_dir.join("config.json");
    if let Ok(json) = serde_json::to_string_pretty(&config) {
        let _ = std::fs::write(&config_file, json);
    }
    config
}

pub fn load_config(config_dir: &Path) -> AppConfig {
    read_config(config_dir).unwrap_or_default()
}

pub fn save_config(config_dir: &Path, config: &AppConfig) -> Result<(), String> {
    let config_file = config_dir.join("config.json");
    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    std::fs::write(&config_file, json).map_err(|e| e.to_string())?;
    Ok(())
}
