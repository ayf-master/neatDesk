# 生成应用图标全套（icon.ico + 各尺寸 PNG）
#
# 为什么重做：
#   1) Tauri 只把 icon.ico 目录里的【第 0 项】当作“窗口/任务栏运行时图标”
#      （见 tauri-codegen 的 CachedIcon::new_ico -> entries()[0]）。
#      旧 icon.ico 第 0 项是 16x16，高分屏放大后任务栏图标很糊。
#   2) Pillow 保存 ICO 时会 sorted(sizes) 升序排列，天然把 16x16 放首位。
#      所以这里【自己写 ICO 目录】，强制“大尺寸在前”，让 entries()[0] = 256。
#   3) 复用托盘图标同款清晰管线：1024 超采样母版 -> BOX 面积平均降采样。
#
# 产物（src-tauri/icons/）:
#   icon.ico            256 在前的完整尺寸梯（供 exe 资源 + 运行时窗口图标）
#   32x32.png / 128x128.png / 128x128@2x.png / 512x512.png / icon-1024.png
#   icon-512-maskable.png   铺满画布、无圆角的可遮罩版
#   app-icon.png            Tauri CLI 通用源图（1024）

import io
import os
import struct
from PIL import Image, ImageDraw, ImageFont

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ICON_DIR = os.path.join(ROOT, "src-tauri", "icons")

S = 1024                     # 超采样母版画布
INSET_RATIO = 0.10           # 方块内边距占画布比例
RADIUS_RATIO = 0.225         # 圆角占方块边长比例
TEXT_WIDTH_RATIO = 0.66      # “ND”总宽度占方块比例
LETTER_SPACING = -0.03       # 字距（em）

STOPS = [(0.0, (0x1A, 0x73, 0xE8)),
         (0.5, (0x00, 0x66, 0xFF)),
         (1.0, (0x00, 0x55, 0xCC))]

# ICO 目录顺序：最大在前（关键！）
ICO_SIZES = [256, 128, 96, 64, 48, 32, 24, 16]


def lerp(a, b, t):
    return tuple(round(a[i] + (b[i] - a[i]) * t) for i in range(3))


def gradient(size):
    """135 度对角线性渐变（左上 -> 右下）"""
    small = 128
    img = Image.new("RGB", (small, small))
    px = img.load()
    for y in range(small):
        for x in range(small):
            t = (x + y) / (2 * (small - 1))
            if t <= STOPS[1][0]:
                c = lerp(STOPS[0][1], STOPS[1][1], t / STOPS[1][0])
            else:
                c = lerp(STOPS[1][1], STOPS[2][1], (t - STOPS[1][0]) / (1 - STOPS[1][0]))
            px[x, y] = c
    return img.resize((size, size), Image.BILINEAR)


def load_font(px):
    for name in ("segoeuib.ttf", "arialbd.ttf"):
        path = os.path.join(os.environ.get("WINDIR", r"C:\Windows"), "Fonts", name)
        if os.path.exists(path):
            return ImageFont.truetype(path, px)
    raise SystemExit("未找到可用粗体字体")


def draw_nd(canvas, tile_box, text_ratio):
    """在 canvas 的 tile_box 区域内居中绘制加粗 “ND”（按墨迹外框居中）"""
    x0, y0, x1, y1 = tile_box
    tile = x1 - x0
    probe = ImageDraw.Draw(canvas)
    target_w = tile * text_ratio
    size = 100
    while size < tile * 2:
        f = load_font(size)
        gap = size * LETTER_SPACING
        w = probe.textlength("N", font=f) + gap + probe.textlength("D", font=f)
        if w >= target_w:
            break
        size += 1
    font = load_font(size)
    gap = size * LETTER_SPACING
    wn = probe.textlength("N", font=font)
    wd = probe.textlength("D", font=font)
    total = wn + gap + wd
    cy = (y0 + y1) / 2
    pos = (x0 + x1 - total) / 2
    for ch, adv in (("N", wn), ("D", wd)):
        box = probe.textbbox((0, 0), ch, font=font)
        ink_cx = (box[0] + box[2]) / 2
        ink_cy = (box[1] + box[3]) / 2
        probe.text((pos + adv / 2 - ink_cx, cy - ink_cy), ch, font=font,
                   fill=(255, 255, 255, 255))
        pos += adv + gap


def build_master(rounded=True, inset_ratio=INSET_RATIO, text_ratio=TEXT_WIDTH_RATIO, full_bleed=False):
    """构建 1024 超采样母版。full_bleed=True 时铺满画布（可遮罩版）。"""
    canvas = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    if full_bleed:
        box = (0, 0, S, S)
        grad = gradient(S).convert("RGBA")
        canvas.paste(grad, (0, 0))
        # 留出安全区：内容缩小到中间 78%
        safe = int(S * 0.78)
        off = (S - safe) // 2
        draw_nd(canvas, (off, off, off + safe, off + safe), 0.62)
        return canvas

    inset = int(S * inset_ratio)
    tile = S - inset * 2
    radius = int(tile * RADIUS_RATIO)
    grad = gradient(tile).convert("RGBA")
    mask = Image.new("L", (tile, tile), 0)
    if rounded:
        ImageDraw.Draw(mask).rounded_rectangle([0, 0, tile - 1, tile - 1], radius=radius, fill=255)
    else:
        ImageDraw.Draw(mask).rectangle([0, 0, tile - 1, tile - 1], fill=255)
    canvas.paste(grad, (inset, inset), mask)
    draw_nd(canvas, (inset, inset, inset + tile, inset + tile), text_ratio)
    return canvas


def downsample(master, size):
    return master.resize((size, size), Image.BOX)


def write_ico(path, items):
    """自定义 ICO：目录顺序原样保留（与 Pillow 的 sorted 行为相反）。"""
    png_blobs = []
    for size, data in items:
        b = io.BytesIO()
        data.save(b, format="PNG", optimize=True)
        png_blobs.append((size, b.getvalue()))
    n = len(png_blobs)
    header = struct.pack("<HHH", 0, 1, n)
    offset = 6 + 16 * n
    entries = b""
    blobs = b""
    for size, blob in png_blobs:
        dim = 0 if size >= 256 else size        # 0 表示 256
        entries += struct.pack("<BBBBHHII", dim, dim, 0, 0, 1, 32, len(blob), offset)
        offset += len(blob)
        blobs += blob
    with open(path, "wb") as f:
        f.write(header + entries + blobs)


def main():
    os.makedirs(ICON_DIR, exist_ok=True)
    master = build_master()

    # 多尺寸 PNG 导出
    exports = {
        "32x32.png": 32,
        "128x128.png": 128,
        "128x128@2x.png": 256,
        "512x512.png": 512,
        "icon-1024.png": 1024,
        "app-icon.png": 1024,
    }
    for name, size in exports.items():
        downsample(master, size).save(os.path.join(ICON_DIR, name))

    # 可遮罩版（铺满画布、无圆角）
    maskable = build_master(full_bleed=True)
    downsample(maskable, 512).save(os.path.join(ICON_DIR, "icon-512-maskable.png"))

    # ICO：大尺寸在前
    ico_items = [(s, downsample(master, s)) for s in ICO_SIZES]
    ico_path = os.path.join(ICON_DIR, "icon.ico")
    write_ico(ico_path, ico_items)

    # 校验：重新解析目录，确认第 0 项是 256
    data = open(ico_path, "rb").read()
    cnt = struct.unpack("<H", data[4:6])[0]
    order = []
    for i in range(cnt):
        e = data[6 + i * 16: 6 + i * 16 + 16]
        bw, bh = e[0], e[1]
        order.append(0 if bw == 0 else bw)

    # 预览条
    sizes = [16, 24, 32, 48, 64, 128, 256]
    pad = 20
    w = sum(sizes) + pad * (len(sizes) + 1)
    h = max(sizes) + pad * 2
    strip = Image.new("RGB", (w, h), (243, 243, 243))
    x = pad
    for s in sizes:
        im = downsample(master, s)
        strip.paste(im, (x, (h - s) // 2), im)
        x += s + pad
    strip.save(os.path.join(ICON_DIR, "icon-preview.png"))

    print("icon.ico 目录顺序:", order, "(应为 256 在前)")
    print("OK  icon.ico %d bytes" % len(data))
    print("OK  预览:", os.path.join(ICON_DIR, "icon-preview.png"))


if __name__ == "__main__":
    main()
