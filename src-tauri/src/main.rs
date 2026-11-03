// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // 检查是否为一键整理模式（从右键菜单启动）
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--quick-organize" {
        // 静默整理：不打开窗口，整理后直接退出（路径与主程序同源）
        desktop_clean_lib::quick_organize();
        return;
    }

    desktop_clean_lib::run()
}
