// 桌面文件扫描模块 — 扫描桌面文件并获取元信息

use serde::Serialize;
use std::path::Path;
use chrono::{DateTime, Local};

#[derive(Debug, Serialize, Clone)]
pub struct FileInfo {
    pub name: String,
    pub extension: String,
    pub path: String,
    pub size: u64,
    pub modified: String,
    pub is_hidden: bool,
}

/// 获取当前用户的桌面路径
pub fn get_desktop_path() -> Result<String, String> {
    // 优先使用 USERPROFILE 环境变量（Windows）
    if let Ok(profile) = std::env::var("USERPROFILE") {
        let desktop = format!("{}\\Desktop", profile);
        if Path::new(&desktop).exists() {
            return Ok(desktop);
        }
    }
    // 备用方案：使用 HOME 环境变量
    if let Ok(home) = std::env::var("HOME") {
        let desktop = format!("{}/Desktop", home);
        if Path::new(&desktop).exists() {
            return Ok(desktop);
        }
    }
    // 最后尝试已知常见路径
    for p in &["C:\\Users\\Public\\Desktop", "C:\\Windows\\Desktop"] {
        if Path::new(p).exists() {
            return Ok(p.to_string());
        }
    }
    Err("无法找到桌面路径。请确认当前用户有桌面文件夹。".into())
}

/// 检查文件是否具有 Windows 隐藏属性
#[cfg(windows)]
fn is_windows_hidden(metadata: &std::fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
    const FILE_ATTRIBUTE_SYSTEM: u32 = 0x4;
    let attrs = metadata.file_attributes();
    (attrs & (FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM)) != 0
}

#[cfg(not(windows))]
fn is_windows_hidden(_metadata: &std::fs::Metadata) -> bool {
    false
}

/// 将 SystemTime 格式化为可读时间字符串
fn format_time(system_time: std::time::SystemTime) -> String {
    let datetime: DateTime<Local> = system_time.into();
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// 扫描桌面文件夹，返回文件信息列表
/// 自动跳过：系统隐藏文件、desktop.ini、目录文件夹
pub fn scan_desktop(desktop_path: &str) -> Result<Vec<FileInfo>, String> {
    let dir = Path::new(desktop_path);
    if !dir.exists() {
        return Err(format!("桌面路径不存在: {}", desktop_path));
    }

    let entries = std::fs::read_dir(dir).map_err(|e| format!("读取桌面失败: {}", e))?;

    let mut files = Vec::new();

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue, // 跳过无法读取的条目
        };

        let path = entry.path();

        // 跳过文件夹（只处理文件）
        if path.is_dir() {
            continue;
        }

        let name = entry.file_name().to_string_lossy().to_string();

        // 跳过 desktop.ini
        if name.eq_ignore_ascii_case("desktop.ini") || name.eq_ignore_ascii_case("thumbs.db") {
            continue;
        }

        // 检查元数据
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        // 跳过 Windows 隐藏/系统文件
        if is_windows_hidden(&metadata) {
            continue;
        }

        // 跳过 Unix 点开头的隐藏文件
        if name.starts_with('.') {
            continue;
        }

        let ext = path.extension()
            .map(|e| format!(".{}", e.to_string_lossy().to_lowercase()))
            .unwrap_or_default();

        let modified = metadata.modified()
            .map(format_time)
            .unwrap_or_default();

        files.push(FileInfo {
            name,
            extension: ext,
            path: path.to_string_lossy().to_string(),
            size: metadata.len(),
            modified,
            is_hidden: is_windows_hidden(&metadata),
        });
    }

    // 按文件名排序，保证顺序一致
    files.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(files)
}
