// 撤销模块 — SQLite 记录整理历史，支持一键回退

use std::path::Path;
use rusqlite::Connection;
/// 初始化数据库，创建表结构
pub fn init_db(data_dir: &Path) -> Result<(), String> {
    let db_path = data_dir.join("history.db");
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库打开失败: {}", e))?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS organize_records (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            organized_at TEXT NOT NULL,
            total_files INTEGER NOT NULL,
            categories TEXT NOT NULL,
            items TEXT NOT NULL,
            undone INTEGER DEFAULT 0
        );"
    ).map_err(|e| format!("数据库初始化失败: {}", e))?;

    Ok(())
}

/// 保存整理记录到数据库
pub fn save_record(data_dir: &Path, result: &crate::organizer::OrganizeResult) -> Result<i64, String> {
    let db_path = data_dir.join("history.db");
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库打开失败: {}", e))?;

    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let categories_json = serde_json::to_string(&result.categories)
        .map_err(|e| format!("序列化分类失败: {}", e))?;
    let items_json = serde_json::to_string(&result.items)
        .map_err(|e| format!("序列化文件列表失败: {}", e))?;

    conn.execute(
        "INSERT INTO organize_records (organized_at, total_files, categories, items) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![now, result.success_count, categories_json, items_json],
    ).map_err(|e| format!("保存记录失败: {}", e))?;

    Ok(conn.last_insert_rowid())
}

/// 撤销指定记录：将文件移回原始位置
/// 自动清理空文件夹
pub fn undo_record(data_dir: &Path, record_id: i64) -> Result<String, String> {
    let db_path = data_dir.join("history.db");
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库打开失败: {}", e))?;

    // 查询记录
    let mut stmt = conn.prepare(
        "SELECT id, items, undone, categories FROM organize_records WHERE id = ?1"
    ).map_err(|e| format!("查询失败: {}", e))?;

    let row = stmt.query_row(rusqlite::params![record_id], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i32>(2)?,
            row.get::<_, String>(3)?,
        ))
    }).map_err(|e| format!("记录不存在 (ID: {}): {}", record_id, e))?;

    if row.2 != 0 {
        return Err("该记录已被撤销，不能重复撤销".into());
    }

    let items: Vec<crate::organizer::OrganizeItem> = serde_json::from_str(&row.1)
        .map_err(|e| format!("解析记录失败: {}", e))?;

    let mut restored = 0;
    let mut skipped = 0;
    let mut mismatch = 0;

    // 收集涉及的文件夹路径（用于后续清理）
    let mut category_folders: Vec<std::path::PathBuf> = Vec::new();

    for item in &items {
        let source = Path::new(&item.new_path);
        let target = Path::new(&item.original_path);

        // 记录文件夹路径
        if let Some(parent) = source.parent() {
            if !category_folders.contains(&parent.to_path_buf()) {
                category_folders.push(parent.to_path_buf());
            }
        }

        // 检查源文件是否存在
        if !source.exists() {
            skipped += 1;
            continue;
        }

        // 身份校验：防止把用户新放入分类文件夹的同名文件误移回
        if !identity_matches(source, item) {
            skipped += 1;
            mismatch += 1;
            continue;
        }

        // 如果原位置已有文件（用户手工放入了新文件），跳过
        if target.exists() {
            skipped += 1;
            continue;
        }

        // 确保目标父目录存在（桌面路径等）
        if let Some(parent) = target.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        match crate::organizer::move_file(source, target) {
            Ok(_) => {
                restored += 1;
            }
            Err(e) => {
                skipped += 1;
                eprintln!("撤销失败: {} -> {}: {}", item.file_name, item.original_path, e);
            }
        }
    }

    // 全部恢复成功才标记为已撤销；有跳过（文件被占用/身份不匹配）时不标记，
    // 用户可重复点击撤销重试（跳过逻辑幂等，已恢复的文件不会重复移动）
    if skipped == 0 {
        conn.execute(
            "UPDATE organize_records SET undone = 1 WHERE id = ?1",
            rusqlite::params![record_id],
        ).map_err(|e| format!("更新记录失败: {}", e))?;
    }

    // 清理空的分类文件夹
    let mut cleaned = 0;
    for folder in &category_folders {
        if folder.exists() && folder.is_dir()
            && std::fs::read_dir(folder).map(|mut d| d.next().is_none()).unwrap_or(false)
        {
            let _ = std::fs::remove_dir(folder);
            cleaned += 1;
        }
    }

    let mut msg = format!("已撤销：成功恢复 {} 个文件", restored);
    if skipped > 0 {
        msg.push_str(&format!("，{} 个跳过", skipped));
    }
    if mismatch > 0 {
        msg.push_str(&format!("，{} 个跳过（文件身份不匹配，可能已被修改）", mismatch));
    }
    if cleaned > 0 {
        msg.push_str(&format!("，清理了 {} 个空文件夹", cleaned));
    }

    Ok(msg)
}

/// 校验分类文件夹中的文件与撤销记录中的原始文件是否为同一文件
/// - 记录字段为 None（旧版本库的项）时跳过校验，返回 true 保持旧行为
/// - metadata 可读且 size 与 modified 均相等才视为匹配
fn identity_matches(path: &Path, item: &crate::organizer::OrganizeItem) -> bool {
    // 旧记录无身份字段，无法校验
    let (Some(size), Some(modified)) = (item.size, item.modified.as_deref()) else {
        return true;
    };

    // metadata 不可读 → 文件状态异常，视为不匹配
    let Ok(meta) = std::fs::metadata(path) else {
        return false;
    };

    // 大小不同 → 不是同一文件
    if meta.len() != size {
        return false;
    }

    // 修改时间不同 → 不是同一文件（或内容已变）
    let Some(actual) = meta.modified().ok().map(|t| {
        let dt: chrono::DateTime<chrono::Local> = t.into();
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    }) else {
        return false;
    };

    actual == modified
}

/// 获取整理历史
pub fn get_history(data_dir: &Path) -> Result<Vec<serde_json::Value>, String> {
    let db_path = data_dir.join("history.db");
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库打开失败: {}", e))?;

    let mut stmt = conn.prepare(
        "SELECT id, organized_at, total_files, categories, undone
         FROM organize_records
         ORDER BY id DESC
         LIMIT 50"
    ).map_err(|e| format!("查询失败: {}", e))?;

    let records = stmt.query_map([], |row| {
        let id: i64 = row.get(0)?;
        let organized_at: String = row.get(1)?;
        let total_files: i32 = row.get(2)?;
        let categories_str: String = row.get(3)?;
        let undone: i32 = row.get(4)?;

        let categories: serde_json::Value =
            serde_json::from_str(&categories_str).unwrap_or(serde_json::Value::Null);

        Ok(serde_json::json!({
            "id": id,
            "organized_at": organized_at,
            "total_files": total_files,
            "categories": categories,
            "undone": undone != 0,
        }))
    }).map_err(|e| format!("查询失败: {}", e))?;

    Ok(records.flatten().collect())
}

/// 删除指定整理记录
pub fn delete_record(data_dir: &Path, record_id: i64) -> Result<String, String> {
    let db_path = data_dir.join("history.db");
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库打开失败: {}", e))?;

    let affected = conn.execute(
        "DELETE FROM organize_records WHERE id = ?1",
        rusqlite::params![record_id],
    ).map_err(|e| format!("删除失败: {}", e))?;

    if affected == 0 {
        return Err("记录不存在".into());
    }
    Ok("已删除".into())
}

/// 清空全部整理记录
pub fn clear_history(data_dir: &Path) -> Result<String, String> {
    let db_path = data_dir.join("history.db");
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库打开失败: {}", e))?;

    conn.execute("DELETE FROM organize_records", [])
        .map_err(|e| format!("清空失败: {}", e))?;

    Ok("已清空全部历史记录".into())
}
