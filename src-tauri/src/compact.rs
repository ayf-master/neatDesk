// 桌面图标紧凑排列模块 — 通过切换自动排列触发重排
#![cfg(windows)]
// 类型名与 Win32 API 官方命名保持一致，豁免大小写检查
#![allow(clippy::upper_case_acronyms)]

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ptr;

type BOOL = i32;
type HWND = isize;
type UINT = u32;
type LPCWSTR = *const u16;

const GWL_STYLE: i32 = -16;
const LVS_AUTOARRANGE: isize = 0x0100;

const SWP_NOMOVE: UINT = 0x0002;
const SWP_NOSIZE: UINT = 0x0001;
const SWP_NOZORDER: UINT = 0x0004;
const SWP_NOACTIVATE: UINT = 0x0010;
const SWP_FRAMECHANGED: UINT = 0x0020;

extern "system" {
    fn FindWindowW(lpClassName: LPCWSTR, lpWindowName: LPCWSTR) -> HWND;
    fn FindWindowExW(hWndParent: HWND, hWndChildAfter: HWND, lpszClass: LPCWSTR, lpszWindow: LPCWSTR) -> HWND;
    fn GetWindowLongW(hWnd: HWND, nIndex: i32) -> isize;
    fn SetWindowLongW(hWnd: HWND, nIndex: i32, dwNewLong: isize) -> isize;
    fn SetWindowPos(hWnd: HWND, hWndInsertAfter: HWND, X: i32, Y: i32, cx: i32, cy: i32, uFlags: UINT) -> BOOL;
    fn Sleep(dwMilliseconds: u32);
    fn InvalidateRect(hWnd: HWND, lpRect: *const u8, bErase: BOOL) -> BOOL;
}

fn to_wstring(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

fn get_desktop_listview() -> Result<HWND, String> {
    unsafe {
        let progman = to_wstring("Progman");
        let h = FindWindowW(progman.as_ptr(), ptr::null());
        if h == 0 {
            return Err("找不到 Progman 窗口".into());
        }
        let sdv = to_wstring("SHELLDLL_DefView");
        let sh = FindWindowExW(h, 0, sdv.as_ptr(), ptr::null());
        if sh != 0 {
            let lv = to_wstring("SysListView32");
            let lvh = FindWindowExW(sh, 0, lv.as_ptr(), ptr::null());
            if lvh != 0 { return Ok(lvh); }
        }
        // Windows 11 备用路径
        let ww = to_wstring("WorkerW");
        let mut w = FindWindowExW(0, 0, ww.as_ptr(), ptr::null());
        while w != 0 {
            let c = FindWindowExW(w, 0, sdv.as_ptr(), ptr::null());
            if c != 0 {
                let lv = to_wstring("SysListView32");
                let lvh = FindWindowExW(c, 0, lv.as_ptr(), ptr::null());
                if lvh != 0 { return Ok(lvh); }
            }
            w = FindWindowExW(0, w, ww.as_ptr(), ptr::null());
        }
        Err("找不到桌面 ListView".into())
    }
}

/// 切换自动排列来触发图标重排
fn toggle_auto_arrange(hwnd_lv: HWND) -> Result<(), String> {
    unsafe {
        let style = GetWindowLongW(hwnd_lv, GWL_STYLE);
        let had_auto = style & LVS_AUTOARRANGE != 0;

        if had_auto {
            // 先关掉自动排列（文件保持当前位置）
            SetWindowLongW(hwnd_lv, GWL_STYLE, style & !LVS_AUTOARRANGE);
            let flags = SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE;
            SetWindowPos(hwnd_lv, 0, 0, 0, 0, 0, flags);
            Sleep(100);
            // 再打开自动排列（触发重排）
            SetWindowLongW(hwnd_lv, GWL_STYLE, style | LVS_AUTOARRANGE);
            SetWindowPos(hwnd_lv, 0, 0, 0, 0, 0, flags);
        } else {
            // 初始就没开自动排列 → 打开再关掉触发重排
            SetWindowLongW(hwnd_lv, GWL_STYLE, style | LVS_AUTOARRANGE);
            let flags = SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE;
            SetWindowPos(hwnd_lv, 0, 0, 0, 0, 0, flags);
            Sleep(100);
            SetWindowLongW(hwnd_lv, GWL_STYLE, style);
            SetWindowPos(hwnd_lv, 0, 0, 0, 0, 0, flags);
        }
        // 刷新桌面显示
        InvalidateRect(hwnd_lv, ptr::null(), 1);
        Ok(())
    }
}

/// 整理后将桌面图标紧凑排列
pub fn compact_desktop_icons() -> Result<String, String> {
    unsafe { Sleep(300); }

    let hwnd_lv = get_desktop_listview()?;
    toggle_auto_arrange(hwnd_lv)?;

    Ok("桌面图标已重新排列".into())
}

/// 通知 Windows 资源管理器刷新桌面显示
/// 整理或撤销后调用，让桌面图标即时更新（无需手动 F5）
pub fn refresh_desktop_explorer() {
    // 获取桌面路径的宽字符形式
    let desktop = match crate::scanner::get_desktop_path() {
        Ok(p) => p,
        Err(_) => return,
    };

    let wide: Vec<u16> = OsStr::new(&desktop)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    // 调用 SHChangeNotify 通知资源管理器刷新
    extern "system" {
        fn SHChangeNotify(
            wEventId: u32,
            uFlags: u32,
            dwItem1: *const std::ffi::c_void,
            dwItem2: *const std::ffi::c_void,
        );
    }

    const SHCNE_UPDATEDIR: u32 = 0x00001000;
    const SHCNF_PATHW: u32 = 0x0005;

    unsafe {
        SHChangeNotify(
            SHCNE_UPDATEDIR,
            SHCNF_PATHW,
            wide.as_ptr() as *const std::ffi::c_void,
            std::ptr::null(),
        );
    }
}
