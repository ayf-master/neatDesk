// 排除列表模块 — 根据用户配置过滤不应被整理的文件

use crate::config::Exclusions;
use crate::scanner::FileInfo;

/// 根据排除列表过滤文件
/// - 排除文件名匹配（如 desktop.ini）
/// - 排除用户指定的文件夹名（匹配路径中的任何段）
/// - 排除 . 开头的隐藏文件
pub fn filter(files: Vec<FileInfo>, exclusions: &Exclusions) -> Vec<FileInfo> {
    files.into_iter()
        .filter(|f| {
            // 排除 . 开头的隐藏文件（Unix 风格）
            if f.name.starts_with('.') {
                return false;
            }

            // 精确文件名匹配（不区分大小写）
            if exclusions.file_names.iter().any(|ex| f.name.eq_ignore_ascii_case(ex)) {
                return false;
            }

            // 路径中包含排除的文件夹名
            let path_lower = f.path.to_lowercase();
            if exclusions.folder_names.iter().any(|folder| {
                let folder_lower = folder.to_lowercase();
                // 检查路径中是否有 "/排除名/" 或 "\排除名\" 模式
                path_lower.contains(&format!("/{}/", folder_lower))
                    || path_lower.contains(&format!("\\{}\\", folder_lower))
            }) {
                return false;
            }

            true
        })
        .collect()
}
