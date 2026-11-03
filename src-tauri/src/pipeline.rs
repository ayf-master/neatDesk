// 共享整理流水线 — 消除 commands/tray/scheduler/main 四处重复实现
// 整理逻辑唯一入口，四个调用点（命令/托盘/定时器/右键）统一走这里

use std::path::Path;

/// 执行一次完整的桌面整理流程
///
/// 流程：load_config → get_desktop_path → scan → exclusions::filter →
/// classifier::classify → organizer::organize → save_record（失败仅记日志）→
/// compact（按配置）→ refresh（cfg(windows)）
///
/// 返回 JSON 与前端 organize_desktop 契约一致：
/// `total_files / success_count / failed_count / categories / errors / compact_log`
pub fn perform_organize(config_dir: &Path, data_dir: &Path) -> Result<serde_json::Value, String> {
    let config = crate::config::load_config(config_dir);
    let desktop = crate::scanner::get_desktop_path()?;

    // 扫描桌面
    let files = crate::scanner::scan_desktop(&desktop)?;

    // 过滤排除项
    let files = crate::exclusions::filter(files, &config.exclusions);

    // 分类
    let classified = crate::classifier::classify(files, &config.categories);

    // 执行整理（各分类自行决定目标路径）
    let result = crate::organizer::organize(&desktop, classified, &config.categories)?;

    // 保存记录到 SQLite（用于撤销）；失败仅记录日志，不影响整理结果
    if let Err(e) = crate::undo::save_record(data_dir, &result) {
        eprintln!("保存整理记录失败: {}", e);
    }

    // 如果开启了自动补齐，整理后将桌面图标紧凑排列
    let compact_log = if config.settings.compact_after_organize && result.success_count > 0 {
        #[cfg(windows)]
        {
            crate::compact::compact_desktop_icons()
                .unwrap_or_else(|e| format!("补齐失败: {}", e))
        }
        #[cfg(not(windows))]
        {
            "仅支持 Windows".into()
        }
    } else {
        String::new()
    };

    // 通知资源管理器刷新桌面显示，让图标即时更新
    #[cfg(windows)]
    crate::compact::refresh_desktop_explorer();

    Ok(serde_json::json!({
        "total_files": result.total_files,
        "success_count": result.success_count,
        "failed_count": result.failed_count,
        "categories": result.categories,
        "errors": result.errors,
        "compact_log": compact_log,
    }))
}
