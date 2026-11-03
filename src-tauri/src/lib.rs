// NeatDesk — Tauri 应用入口

pub mod commands;
pub mod config;
pub mod scanner;
pub mod classifier;
pub mod organizer;
pub mod undo;
mod scheduler;
pub mod exclusions;
mod compact;
mod tray;
mod pipeline;

use std::sync::{Arc, Mutex};
use tauri::Manager;

/// 单例检测 + 唤起到前台
/// 使用命名事件通信：第二实例 → 通知事件 → 第一实例唤起到前台
#[cfg(windows)]
fn handle_single_instance() -> bool {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;

    let mutex_name = OsStr::new("NeatDesk_SingleInstance_Mutex");
    let event_name = OsStr::new("NeatDesk_ShowWindow_Event");
    let mutex_wide: Vec<u16> = mutex_name.encode_wide().chain(std::iter::once(0)).collect();
    let event_wide: Vec<u16> = event_name.encode_wide().chain(std::iter::once(0)).collect();

    extern "system" {
        fn CreateMutexW(
            lpMutexAttributes: *const std::ffi::c_void,
            bInitialOwner: i32,
            lpName: *const u16,
        ) -> isize;
        fn CreateEventW(
            lpEventAttributes: *const std::ffi::c_void,
            bManualReset: i32,
            bInitialState: i32,
            lpName: *const u16,
        ) -> isize;
        fn SetEvent(hEvent: isize) -> i32;
        fn GetLastError() -> u32;
    }

    unsafe {
        let _mutex = CreateMutexW(ptr::null(), 0, mutex_wide.as_ptr());
        let event = CreateEventW(ptr::null(), 0, 0, event_wide.as_ptr());

        if GetLastError() == 183 {
            // 已有实例在运行 → 通知它唤起到前台
            if event != 0 {
                let _ = SetEvent(event);
            }
            return false;
        }
    }
    true
}

#[cfg(not(windows))]
fn handle_single_instance() -> bool {
    true
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 单例检测 + 唤起到前台
    if !handle_single_instance() {
        return;
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, _shortcut, event| {
                    if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        crate::tray::trigger_organize(app);
                    }
                })
                .build(),
        )
        .setup(|app| {
            // 初始化配置
            let config_path = app.path().app_config_dir()?;
            std::fs::create_dir_all(&config_path)?;
            let _ = config::ensure_config(&config_path);

            // 初始化数据库
            let data_path = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_path)?;
            let _ = undo::init_db(&data_path);

            // 注册应用状态（Arc 托管，供命令与后台调度器共享）
            let app_state = commands::AppState {
                config_dir: config_path,
                data_dir: data_path,
            };
            app.manage(Arc::new(Mutex::new(app_state)));

            // 创建系统托盘
            let _ = tray::create_tray(app.handle());

            // 从配置读取并注册全局快捷键
            let state = app.state::<Arc<Mutex<crate::commands::AppState>>>();
            if let Ok(s) = state.lock() {
                let cfg = crate::config::load_config(&s.config_dir);
                let shortcut_key = cfg.settings.shortcut;
                if !shortcut_key.is_empty() {
                    use tauri_plugin_global_shortcut::GlobalShortcutExt;
                    let _ = app.global_shortcut().register(shortcut_key.as_str());
                }
            }

            // 启动定时调度（界面配置的定时规则到点自动整理）
            let state = app.state::<Arc<Mutex<crate::commands::AppState>>>();
            crate::scheduler::Scheduler::new().start(app.handle().clone(), state.inner().clone());

            // 后台线程等待唤醒事件
            #[cfg(windows)]
            {
                let handle = app.handle().clone();
                std::thread::spawn(move || {
                    use std::ffi::OsStr;
                    use std::os::windows::ffi::OsStrExt;
                    use std::ptr;

                    let event_name = OsStr::new("NeatDesk_ShowWindow_Event");
                    let wide: Vec<u16> = event_name.encode_wide().chain(std::iter::once(0)).collect();

                    extern "system" {
                        fn CreateEventW(
                            lpEventAttributes: *const std::ffi::c_void,
                            bManualReset: i32,
                            bInitialState: i32,
                            lpName: *const u16,
                        ) -> isize;
                        fn WaitForSingleObject(hHandle: isize, dwMilliseconds: u32) -> u32;
                    }

                    unsafe {
                        let evt = CreateEventW(ptr::null(), 0, 0, wide.as_ptr());
                        if evt == 0 {
                            return;
                        }
                        loop {
                            // 无限等待事件
                            WaitForSingleObject(evt, 0xFFFFFFFF);
                            // 收到信号 → 唤起到前台
                            if let Some(window) = handle.get_webview_window("main") {
                                let _ = window.unminimize();
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                });
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::update_config,
            commands::scan_desktop,
            commands::categorize_desktop,
            commands::organize_desktop,
            commands::undo_organize,
            commands::get_history,
            commands::set_auto_start,
            commands::delete_history_record,
            commands::clear_history,
            commands::update_shortcut,
            commands::install_context_menu,
            commands::uninstall_context_menu,
            commands::compact_icons,
            commands::exit_app,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 右键"立即整理"模式：静默整理，不打开窗口，整理后直接退出
/// 路径与主程序同源（%APPDATA%\<identifier>），读取用户自定义配置
pub fn quick_organize() {
    use std::path::Path;

    // 与 Tauri 路径解析同源：data_dir.join(identifier)
    // （Tauri 2 Windows 上 app_config_dir == app_data_dir，config.json 直接放该目录）
    let context: tauri::Context<tauri::Wry> = tauri::generate_context!();
    let identifier = context.config().identifier.clone();
    let app_dir = directories::BaseDirs::new()
        .map(|d| d.data_dir().join(&identifier))
        .unwrap_or_else(|| Path::new(".").join(&identifier));

    let _ = crate::pipeline::perform_organize(&app_dir, &app_dir);
}
