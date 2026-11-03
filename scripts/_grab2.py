# 抓取指定标题窗口的真实屏幕内容（强制置顶，避免被其它窗口遮挡）。
import ctypes
import ctypes.wintypes as wt
import sys
import time

from PIL import ImageGrab

user32 = ctypes.windll.user32
user32.SetProcessDPIAware()

SWP_NOSIZE = 0x0001
SWP_NOMOVE = 0x0002
SWP_SHOWWINDOW = 0x0040
HWND_TOPMOST = -1
HWND_NOTOPMOST = -2


def find_window(substr):
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
        if substr in buf.value:
            r = wt.RECT()
            user32.GetWindowRect(hwnd, ctypes.byref(r))
            area = (r.right - r.left) * (r.bottom - r.top)
            if area > best[1]:
                best[0], best[1] = hwnd, area
        return True

    user32.EnumWindows(cb, 0)
    return best[0]


def main(substr, out):
    hwnd = find_window(substr)
    if not hwnd:
        print("NO WINDOW for", substr)
        sys.exit(1)
    user32.ShowWindow(hwnd, 9)  # SW_RESTORE
    user32.SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW)
    user32.SetForegroundWindow(hwnd)
    time.sleep(2.0)
    r = wt.RECT()
    user32.GetWindowRect(hwnd, ctypes.byref(r))
    print("rect:", r.left, r.top, r.right, r.bottom)
    shot = ImageGrab.grab(all_screens=True)
    shot.crop((r.left, r.top, r.right, r.bottom)).save(out)
    user32.SetWindowPos(hwnd, HWND_NOTOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE)
    print("saved", out)


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
