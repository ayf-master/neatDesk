// 系统托盘模块 — 后台运行托盘图标 + 右键菜单

use tauri::{
    AppHandle, Manager,
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder},
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

/// 创建系统托盘图标和右键菜单
pub fn create_tray(app: &AppHandle) -> Result<(), String> {
    let organize_item = MenuItemBuilder::with_id("organize", "立即整理")
        .build(app)
        .map_err(|e| format!("创建菜单失败: {}", e))?;

    let show_item = MenuItemBuilder::with_id("show", "打开主界面")
        .build(app)
        .map_err(|e| format!("创建菜单失败: {}", e))?;

    let separator = tauri::menu::PredefinedMenuItem::separator(app)
        .map_err(|e| format!("创建分隔线失败: {}", e))?;

    let quit_item = MenuItemBuilder::with_id("quit", "退出")
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

    TrayIconBuilder::new()
        .icon(tray_icon)
        .menu(&menu)
        .tooltip("NeatDesk - 桌面整理工具")
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
