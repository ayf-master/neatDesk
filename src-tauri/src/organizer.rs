// 文件整理执行模块 — 移动文件到分类文件夹

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use chrono::Local;
use crate::scanner::FileInfo;

#[derive(Debug, Serialize, Clone)]
pub struct OrganizeResult {
    pub total_files: usize,
    pub success_count: usize,
    pub failed_count: usize,
    pub categories: Vec<CategorySummary>,
    pub items: Vec<OrganizeItem>,
    pub errors: Vec<OrganizeError>,
}

#[derive(Debug, Serialize, Clone)]
pub struct CategorySummary {
    pub name: String,
    pub count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrganizeItem {
    pub original_path: String,
    pub new_path: String,
    pub file_name: String,
    pub category_name: String,
    /// 文件大小（字节），用于撤销时校验文件身份；旧记录为 None
    pub size: Option<u64>,
    /// 文件修改时间（%Y-%m-%d %H:%M:%S），用于撤销时校验文件身份；旧记录为 None
    pub modified: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct OrganizeError {
    pub file_name: String,
    pub category: String,
    pub reason: String,
}

/// 执行整理：将分类好的文件移动到对应的分类文件夹
/// - 每个分类可指定 target_path，为空则整理到桌面
/// - 创建分类文件夹（中文名称，如"图片"、"文档"）
/// - 重名文件自动添加时间戳后缀
/// - 被占用/权限不足的文件跳过并记录
pub fn organize(
    desktop_path: &str,
    classified: HashMap<String, Vec<FileInfo>>,
    categories: &[crate::config::Category],
) -> Result<OrganizeResult, String> {
    let desktop = Path::new(desktop_path);

    let mut items = Vec::new();
    let mut errors = Vec::new();
    let mut cat_summaries = Vec::new();

    for (cat_id, files) in &classified {
        if files.is_empty() {
            continue;
        }

        // 获取分类的中文名称
        let cat_name = categories.iter()
            .find(|c| c.id == *cat_id)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| cat_id.clone());

        // 确定该分类的目标目录：分类指定路径 or 桌面
        let cat_target = categories.iter()
            .find(|c| c.id == *cat_id)
            .and_then(|c| if c.target_path.is_empty() { None } else { Some(c.target_path.as_str()) });

        let target_base = match cat_target {
            Some(p) => Path::new(p),
            None => desktop,
        };

        // 目标目录不存在时，尝试自动创建（例如用户手动删除了该文件夹）
        // 这样配置「D:\图片」之类自定义路径时，删了文件夹再整理也能自动恢复
        if !target_base.exists() {
            if let Err(e) = std::fs::create_dir_all(target_base) {
                for file in files {
                    errors.push(OrganizeError {
                        file_name: file.name.clone(),
                        category: cat_name.clone(),
                        reason: format!("目标目录创建失败: {}（请检查路径是否正确）", e),
                    });
                }
                continue;
            }
        }

        // 创建分类文件夹
        let target_dir = target_base.join(&cat_name);
        if let Err(e) = std::fs::create_dir_all(&target_dir) {
            // 如果文件夹创建失败，该分类下所有文件都报错
            for file in files {
                errors.push(OrganizeError {
                    file_name: file.name.clone(),
                    category: cat_name.clone(),
                    reason: format!("创建目录失败: {}", e),
                });
            }
            continue;
        }

        // 逐个移动文件
        for file in files {
            let source = Path::new(&file.path);

            // 安全检查：源文件必须在桌面路径下
            if !source.starts_with(desktop) {
                errors.push(OrganizeError {
                    file_name: file.name.clone(),
                    category: cat_name.clone(),
                    reason: "文件不在桌面目录中，跳过".to_string(),
                });
                continue;
            }

            // 检查源文件是否存在
            if !source.exists() {
                errors.push(OrganizeError {
                    file_name: file.name.clone(),
                    category: cat_name.clone(),
                    reason: "文件不存在，可能已被移动或删除".to_string(),
                });
                continue;
            }

            // 解决重名冲突
            let dest = resolve_conflict(&target_dir, &file.name);

            // 采集文件身份信息（移动前），供撤销时校验是否为同一文件
            let meta = std::fs::metadata(&file.path).ok();

            // 执行文件移动（move_file 内部已处理跨盘情况）
            match move_file(source, &dest) {
                Ok(_) => {
                    items.push(OrganizeItem {
                        original_path: file.path.clone(),
                        new_path: dest.to_string_lossy().to_string(),
                        file_name: file.name.clone(),
                        category_name: cat_name.clone(),
                        size: meta.as_ref().map(|m| m.len()),
                        modified: meta.as_ref().and_then(modified_str),
                    });
                }
                Err(e) => {
                    let reason = match e.raw_os_error() {
                        Some(32) => "文件被其他程序占用中".to_string(),
                        Some(5) => "权限不足，无法移动".to_string(),
                        Some(80) => "目标文件已存在（重名冲突）".to_string(),
                        Some(17) => "跨盘移动失败".to_string(),
                        _ => format!("移动失败: {}", e),
                    };
                    errors.push(OrganizeError {
                        file_name: file.name.clone(),
                        category: cat_name.clone(),
                        reason,
                    });
                }
            }
        }

        let count = items.iter().filter(|i| i.category_name == cat_name).count();
        if count > 0 {
            cat_summaries.push(CategorySummary {
                name: cat_name,
                count,
            });
        }
    }

    Ok(OrganizeResult {
        total_files: items.len() + errors.len(),
        success_count: items.len(),
        failed_count: errors.len(),
        categories: cat_summaries,
        items,
        errors,
    })
}

/// 将文件修改时间格式化为字符串（与数据库时间格式一致），
/// 供撤销时的文件身份校验使用；metadata 读取失败返回 None
fn modified_str(meta: &std::fs::Metadata) -> Option<String> {
    meta.modified().ok().map(|t| {
        let dt: chrono::DateTime<chrono::Local> = t.into();
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    })
}

/// 移动文件，自动处理跨盘（不同卷）情况
///
/// Windows 上 `std::fs::rename` 只能同卷移动，跨盘会返回
/// `ERROR_NOT_SAME_DEVICE`（原始错误码 17）。此时降级为「复制 + 删除源」。
/// 若复制成功但源删除失败，撤销已复制的目标，避免桌面留下重复副本。
pub fn move_file(source: &Path, dest: &Path) -> std::io::Result<()> {
    match std::fs::rename(source, dest) {
        Ok(()) => Ok(()),
        Err(e) if e.raw_os_error() == Some(17) => {
            std::fs::copy(source, dest)?;
            if let Err(del_err) = std::fs::remove_file(source) {
                // 源删不掉：撤销已复制的目标，保持一致性
                let _ = std::fs::remove_file(dest);
                return Err(del_err);
            }
            Ok(())
        }
        Err(e) => Err(e),
    }
}

/// 处理文件重名冲突：如果目标路径已存在文件，添加时间戳后缀
/// 格式：`文件名_20260617_183000.ext`
fn resolve_conflict(target_dir: &Path, file_name: &str) -> PathBuf {
    let target = target_dir.join(file_name);

    if !target.exists() {
        return target;
    }

    // 解析文件名和扩展名
    let stem = Path::new(file_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file");

    let ext = Path::new(file_name)
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();

    // 使用 chrono 格式化时间戳
    let ts = Local::now().format("_%Y%m%d_%H%M%S").to_string();
    let new_name = format!("{}{}{}", stem, ts, ext);

    target_dir.join(new_name)
}
