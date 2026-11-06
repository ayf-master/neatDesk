// 系统托盘模块 — 后台运行托盘图标 + 右键菜单（跟随界面语言）

use std::sync::Mutex;
use tauri::{
    AppHandle, Manager,
    image::Image,
    menu::{MenuBuilder, MenuItem, MenuItemBuilder},
    tray::TrayIconBuilder,
    WebviewUrl, WebviewWindowBuilder,
};

/// 托盘图标：32x32 蓝底白字 ND，由 scripts/gen-tray-icon.py 生成
///
/// 这里直接内嵌原始 RGBA 像素，而不是运行时解码 PNG：
/// Tauri 的 `Image::from_bytes` 需要开启 image-png / image-ico 特性（会引入 image 解码依赖），
/// 而 `Image::new` 只吃裸像素，零依赖、零解码开销。
fn make_tray_icon() -> Image<'static> {
    Image::new(include_bytes!("../icons/tray-32.rgba"), 32, 32)
}

/// 托盘菜单项文案（按语言）
struct TrayLabels {
    organize: &'static str,
    show: &'static str,
    quit: &'static str,
    tooltip: &'static str,
}

fn tray_labels(lang: &str) -> TrayLabels {
    if lang == "en" {
        TrayLabels {
            organize: "Organize Now",
            show: "Open NeatDesk",
            quit: "Quit",
            tooltip: "NeatDesk - Desktop Organizer",
        }
    } else {
        TrayLabels {
            organize: "立即整理",
            show: "打开主界面",
            quit: "退出",
            tooltip: "NeatDesk - 桌面整理工具",
        }
    }
}

/// 从配置文件读取界面语言（默认 zh-CN）
fn read_language(app: &AppHandle) -> String {
    let lang = app
        .try_state::<std::sync::Arc<Mutex<crate::commands::AppState>>>()
        .and_then(|state| {
            state
                .lock()
                .ok()
                .map(|s| crate::config::load_config(&s.config_dir).settings.language)
        })
        .unwrap_or_default();
    if lang.is_empty() { "zh-CN".to_string() } else { lang }
}

/// 托盘菜单项句柄，供语言切换时更新文字
pub struct TrayHandles {
    organize: MenuItem<tauri::Wry>,
    show: MenuItem<tauri::Wry>,
    quit: MenuItem<tauri::Wry>,
}

/// 创建系统托盘图标和右键菜单
pub fn create_tray(app: &AppHandle) -> Result<(), String> {
    let labels = tray_labels(&read_language(app));

    let organize_item = MenuItemBuilder::with_id("organize", labels.organize)
        .build(app)
        .map_err(|e| format!("创建菜单失败: {}", e))?;

    let show_item = MenuItemBuilder::with_id("show", labels.show)
        .build(app)
        .map_err(|e| format!("创建菜单失败: {}", e))?;

    let separator = tauri::menu::PredefinedMenuItem::separator(app)
        .map_err(|e| format!("创建分隔线失败: {}", e))?;

    let quit_item = MenuItemBuilder::with_id("quit", labels.quit)
        .build(app)
        .map_err(|e| format!("创建菜单失败: {}", e))?;

    let menu = MenuBuilder::new(app)
        .item(&organize_item)
        .item(&show_item)
        .item(&separator)
        .item(&quit_item)
        .build()
        .map_err(|e| format!("构建菜单失败: {}", e))?;

    let tray_icon = make_tray_icon();

    TrayIconBuilder::with_id("neatdesk-tray")
        .icon(tray_icon)
        .menu(&menu)
        .tooltip(labels.tooltip)
        .on_menu_event(move |app, event| {
            match event.id.as_ref() {
                "organize" => trigger_organize(app),
                "show" => show_or_create_window(app),
                "quit" => std::process::exit(0),
                _ => {}
            }
        })
        .build(app)
        .map_err(|e| format!("创建托盘失败: {}", e))?;

    // 托管菜单项句柄，供 set_tray_language 命令更新文字
    app.manage(Mutex::new(TrayHandles {
        organize: organize_item,
        show: show_item,
        quit: quit_item,
    }));

    Ok(())
}

/// 命令：切换托盘菜单语言（前端语言设置变更时调用）
#[tauri::command]
pub fn set_tray_language(app: AppHandle, language: String) -> Result<(), String> {
    let labels = tray_labels(&language);

    let handles = app
        .try_state::<Mutex<TrayHandles>>()
        .ok_or_else(|| "托盘句柄未初始化".to_string())?;
    let handles = handles
        .lock()
        .map_err(|_| "托盘句柄锁定失败".to_string())?;

    handles
        .organize
        .set_text(labels.organize)
        .map_err(|e| format!("更新菜单失败: {}", e))?;
    handles
        .show
        .set_text(labels.show)
        .map_err(|e| format!("更新菜单失败: {}", e))?;
    handles
        .quit
        .set_text(labels.quit)
        .map_err(|e| format!("更新菜单失败: {}", e))?;

    if let Some(tray) = app.tray_by_id("neatdesk-tray") {
        let _ = tray.set_tooltip(Some(labels.tooltip));
    }

    Ok(())
}

/// 显示已有窗口，或重新创建窗口
fn show_or_create_window(app: &AppHandle) {
    // 先尝试找已有窗口
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        return;
    }

    // 窗口不存在，重新创建（被用户关闭后）
    let builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
        .title("NeatDesk")
        .inner_size(800.0, 600.0)
        .min_inner_size(640.0, 480.0)
        .resizable(true)
        .center();

    if let Ok(window) = builder.build() {
        let _ = window.set_focus();
    }
}

pub fn trigger_organize(app: &AppHandle) {
    if let Some(state) = app.try_state::<std::sync::Arc<std::sync::Mutex<crate::commands::AppState>>>() {
        if let Ok(state) = state.lock() {
            let _ = crate::pipeline::perform_organize(&state.config_dir, &state.data_dir);
        }
    }
}
