// 文件分类引擎 — 按文件名关键词 + 扩展名匹配分类

use std::collections::HashMap;
use crate::config::Category;
use crate::scanner::FileInfo;

/// 对文件列表进行分类，返回 分类名 → [文件列表] 的映射
/// 匹配规则（按优先级）：
/// 1. 文件名包含关键词（name_patterns）
/// 2. 文件扩展名匹配（extensions）
pub fn classify(files: Vec<FileInfo>, categories: &[Category]) -> HashMap<String, Vec<FileInfo>> {
    let mut result: HashMap<String, Vec<FileInfo>> = HashMap::new();

    let others_id = categories.iter()
        .find(|c| c.id == "others")
        .map(|c| c.id.clone())
        .unwrap_or_else(|| "others".into());

    // 初始化启用的分类
    for cat in categories {
        if cat.enabled {
            result.entry(cat.id.clone()).or_default();
        }
    }

    // 检查"其他"分类是否启用
    let others_enabled = categories.iter()
        .any(|c| c.id == "others" && c.enabled);

    if others_enabled {
        result.entry(others_id.clone()).or_default();
    }

    // 对每个文件分类（两步匹配）
    for file in files {
        let mut classified = false;

        // 第一步：按文件名关键词匹配
        for cat in categories {
            if !cat.enabled || cat.id == "others" || cat.name_patterns.is_empty() {
                continue;
            }
            let name_lower = file.name.to_lowercase();
            if cat.name_patterns.iter().any(|p| name_lower.contains(&p.to_lowercase())) {
                result.get_mut(&cat.id).unwrap().push(file.clone());
                classified = true;
                break;
            }
        }

        // 第二步：按扩展名匹配（仅当关键词没匹配到）
        if !classified {
            for cat in categories {
                if !cat.enabled || cat.id == "others" {
                    continue;
                }
                if cat.extensions.is_empty() {
                    continue;
                }
                if cat.extensions.iter().any(|ext| ext.eq_ignore_ascii_case(&file.extension)) {
                    result.get_mut(&cat.id).unwrap().push(file.clone());
                    classified = true;
                    break;
                }
            }
        }

        // 都没匹配到 → 归入"其他"
        if !classified && others_enabled {
            result.get_mut(&others_id).unwrap().push(file);
        }
    }

    result
}
