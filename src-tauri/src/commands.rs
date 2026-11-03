// Tauri 命令接口 — 前端调用的后端函数
// 将在 Step 2-5 逐步实现

use tauri::State;
use crate::config;
use std::sync::{Arc, Mutex};

pub struct AppState {
    pub config_dir: std::path::PathBuf,
    pub data_dir: std::path::PathBuf,
}

#[tauri::command]
pub fn get_config(state: State<'_, Arc<Mutex<AppState>>>) -> Result<config::AppConfig, String> {
    let app_state = state.lock().map_err(|e| e.to_string())?;
    Ok(config::load_config(&app_state.config_dir))
}

#[tauri::command]
pub fn update_config(
    state: State<'_, Arc<Mutex<AppState>>>,
    new_config: config::AppConfig,
) -> Result<(), String> {
    let app_state = state.lock().map_err(|e| e.to_string())?;
    config::save_config(&app_state.config_dir, &new_config)
}

#[tauri::command]
pub fn scan_desktop(state: State<'_, Arc<Mutex<AppState>>>) -> Result<Vec<crate::scanner::FileInfo>, String> {
    let _app_state = state.lock().map_err(|e| e.to_string())?;
    let desktop = crate::scanner::get_desktop_path()?;
    crate::scanner::scan_desktop(&desktop)
}

/// 扫描桌面并分类（不移动文件），用于首页展示概况
#[tauri::command]
pub fn categorize_desktop(state: State<'_, Arc<Mutex<AppState>>>) -> Result<serde_json::Value, String> {
    let app_state = state.lock().map_err(|e| e.to_string())?;
    let config = config::load_config(&app_state.config_dir);
    let desktop = crate::scanner::get_desktop_path()?;

    let files = crate::scanner::scan_desktop(&desktop)?;
    let files = crate::exclusions::filter(files, &config.exclusions);
    let classified = crate::classifier::classify(files, &config.categories);

    let mut categories = Vec::new();
    let mut total = 0;
    for cat in &config.categories {
        if let Some(cat_files) = classified.get(&cat.id) {
            if !cat_files.is_empty() {
                categories.push(serde_json::json!({
                    "id": cat.id,
                    "name": cat.name,
                    "icon": cat.icon,
                    "count": cat_files.len(),
                }));
                total += cat_files.len();
            }
        }
    }

    Ok(serde_json::json!({
        "total_files": total,
        "categories": categories,
    }))
}

#[tauri::command]
pub fn organize_desktop(state: State<'_, Arc<Mutex<AppState>>>) -> Result<serde_json::Value, String> {
    let app_state = state.lock().map_err(|e| e.to_string())?;
    crate::pipeline::perform_organize(&app_state.config_dir, &app_state.data_dir)
}

#[tauri::command]
pub fn undo_organize(state: State<'_, Arc<Mutex<AppState>>>, record_id: i64) -> Result<String, String> {
    let app_state = state.lock().map_err(|e| e.to_string())?;
    let result = crate::undo::undo_record(&app_state.data_dir, record_id);

    // 撤销成功后通知资源管理器刷新桌面显示
    #[cfg(windows)]
    if result.is_ok() {
        crate::compact::refresh_desktop_explorer();
    }

    result
}

#[tauri::command]
pub fn get_history(state: State<'_, Arc<Mutex<AppState>>>) -> Result<Vec<serde_json::Value>, String> {
    let app_state = state.lock().map_err(|e| e.to_string())?;
    crate::undo::get_history(&app_state.data_dir)
}

/// 删除指定整理记录
#[tauri::command]
pub fn delete_history_record(state: State<'_, Arc<Mutex<AppState>>>, record_id: i64) -> Result<String, String> {
    let app_state = state.lock().map_err(|e| e.to_string())?;
    crate::undo::delete_record(&app_state.data_dir, record_id)
}

/// 清空全部整理历史
#[tauri::command]
pub fn clear_history(state: State<'_, Arc<Mutex<AppState>>>) -> Result<String, String> {
    let app_state = state.lock().map_err(|e| e.to_string())?;
    crate::undo::clear_history(&app_state.data_dir)
}

/// 设置开机自启（通过 Windows 注册表）
#[tauri::command]
pub fn set_auto_start(enabled: bool) -> Result<String, String> {
    let app_exe = std::env::current_exe()
        .map_err(|e| format!("获取程序路径失败: {}", e))?;
    let app_path = app_exe.to_string_lossy().to_string();

    let key_path = r"Software\Microsoft\Windows\CurrentVersion\Run";
    let key = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);

    let run_key = key.open_subkey_with_flags(
        key_path,
        winreg::enums::KEY_READ | winreg::enums::KEY_WRITE,
    ).map_err(|e| format!("打开注册表失败: {}", e))?;

    if enabled {
        run_key.set_value("NeatDesk", &app_path)
            .map_err(|e| format!("设置注册表失败: {}", e))?;
        Ok("已开启开机自启".into())
    } else {
        // 删除当前名称的键；值不存在也 OK（未开启过自启）
        let _ = run_key.delete_value("NeatDesk");
        // 顺带清理旧项目名 "DesktopClean" 的历史残留
        let _ = run_key.delete_value("DesktopClean");
        Ok("已关闭开机自启".into())
    }
}

/// 更新全局快捷键
#[tauri::command]
pub fn update_shortcut(app: tauri::AppHandle, shortcut: String) -> Result<String, String> {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    // 注销旧的快捷键
    let _ = app.global_shortcut().unregister_all();

    // 注册新的
    if !shortcut.is_empty() {
        app.global_shortcut()
            .register(shortcut.as_str())
            .map_err(|e| format!("注册快捷键失败: {}", e))?;
    }

    Ok(format!("快捷键已设置为 {}", shortcut))
}

/// 安装桌面右键菜单
#[tauri::command]
pub fn install_context_menu() -> Result<String, String> {
    use winreg::RegKey;
    use winreg::enums::*;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let base_path = r"Software\Classes\DesktopBackground\Shell\NeatDesk";

    // 创建主键
    let (main_key, _) = hkcu.create_subkey(base_path)
        .map_err(|e| format!("创建注册表键失败: {}", e))?;
    main_key.set_value("", &"一键整理桌面")
        .map_err(|e| format!("设置显示名称失败: {}", e))?;
    main_key.set_value("Icon", &std::env::current_exe().unwrap_or_default().to_string_lossy().to_string())
        .map_err(|e| format!("设置图标失败: {}", e))?;
    main_key.set_value("Position", &"Bottom")
        .map_err(|e| format!("设置位置失败: {}", e))?;

    // 创建 command 子键
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("获取程序路径失败: {}", e))?
        .to_string_lossy()
        .to_string();
    let (cmd_key, _) = hkcu.create_subkey(format!(r"{}\command", base_path))
        .map_err(|e| format!("创建命令键失败: {}", e))?;
    cmd_key.set_value("", &format!("\"{}\" --quick-organize", exe_path))
        .map_err(|e| format!("设置命令失败: {}", e))?;

    Ok("右键菜单已安装".into())
}

/// 手动触发桌面图标补齐
#[tauri::command]
pub fn compact_icons() -> Result<String, String> {
    #[cfg(windows)]
    {
        crate::compact::compact_desktop_icons()
    }
    #[cfg(not(windows))]
    {
        Err("仅支持 Windows".into())
    }
}

/// 退出应用
#[tauri::command]
pub fn exit_app(_app: tauri::AppHandle) -> Result<String, String> {
    std::process::exit(0);
}

/// 卸载桌面右键菜单
#[tauri::command]
pub fn uninstall_context_menu() -> Result<String, String> {
    use winreg::RegKey;
    use winreg::enums::*;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let base_path = r"Software\Classes\DesktopBackground\Shell\NeatDesk";

    let _ = hkcu.delete_subkey_all(base_path);

    Ok("右键菜单已卸载".into())
}
