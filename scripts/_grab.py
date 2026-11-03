# 抓取 NeatDesk 窗口在【真实屏幕】上的样子（含 WebView2 内容）。
# PrintWindow 常常抓不到 WebView2（DirectComposition）内容，全屏截图可以。
import ctypes
import ctypes.wintypes as wt
import sys
import time

from PIL import ImageGrab

user32 = ctypes.windll.user32
user32.SetProcessDPIAware()


def find_window():
    best = [None, 0]

    @ctypes.WINFUNCTYPE(ctypes.c_bool, wt.HWND, wt.LPARAM)
    def cb(hwnd, lparam):
        if not user32.IsWindowVisible(hwnd):
            return True
        n = user32.GetWindowTextLengthW(hwnd)
        if n == 0:
            return True
        buf = ctypes.create_unicode_buffer(n + 1)
        user32.GetWindowTextW(hwnd, buf, n + 1)
        if "NeatDesk" in buf.value:
            r = wt.RECT()
            user32.GetWindowRect(hwnd, ctypes.byref(r))
            area = (r.right - r.left) * (r.bottom - r.top)
            if area > best[1]:
                best[0], best[1] = hwnd, area
        return True

    user32.EnumWindows(cb, 0)
    return best[0]


def main(out):
    hwnd = find_window()
    if not hwnd:
        print("NO WINDOW")
        sys.exit(1)
    user32.ShowWindow(hwnd, 9)          # SW_RESTORE
    user32.SetForegroundWindow(hwnd)
    time.sleep(1.5)
    r = wt.RECT()
    user32.GetWindowRect(hwnd, ctypes.byref(r))
    print("rect:", r.left, r.top, r.right, r.bottom)
    shot = ImageGrab.grab(all_screens=True)
    box = (r.left, r.top, r.right, r.bottom)
    shot.crop(box).save(out)
    print("saved", out, "screen", shot.size)


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else "_grab.png")
