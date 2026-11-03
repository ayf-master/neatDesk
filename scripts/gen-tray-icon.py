# 生成系统托盘图标（32x32 RGBA 原始字节）
#
# 为什么单独做一套托盘图标：
#   通知区域（任务栏右下角）只显示 16x16 左右，若直接缩小 512 的主图标，
#   "ND" 会糊成一小团。所以这里重新排布：方块铺满、字母放大加粗，
#   在 512 画布上渲染后整体降采样到 32x32，得到干净的抗锯齿小图。
#
# 产物:
#   src-tauri/icons/tray-32.rgba  -> 供 Rust include_bytes! 内嵌（无需图片解码依赖）
#   src-tauri/icons/tray-32.png   -> 视觉参考
#   src-tauri/icons/tray-preview.png -> 多尺寸效果预览

import os
from PIL import Image, ImageDraw, ImageFont

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ICON_DIR = os.path.join(ROOT, "src-tauri", "icons")

S = 512                      # 设计画布（也是超采样母版）
TARGET = 32                  # 最终托盘图标尺寸
INSET = 16                   # 方块内边距（托盘要尽量铺满）
TILE = S - INSET * 2         # 方块边长
RADIUS = int(TILE * 0.24)    # 圆角比例与主图标一致
TEXT_WIDTH_RATIO = 0.76      # 字母总宽度占方块的比例（比主图标更大更醒目）
LETTER_SPACING = -0.035      # 字距（em），负值收紧

STOPS = [(0.0, (0x1A, 0x73, 0xE8)),
         (0.5, (0x00, 0x66, 0xFF)),
         (1.0, (0x00, 0x55, 0xCC))]


def lerp(a, b, t):
    return tuple(round(a[i] + (b[i] - a[i]) * t) for i in range(3))


def gradient(size):
    """135 度对角线性渐变（左上 -> 右下）"""
    small = 96
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


def build_master():
    # 1) 方块（渐变 + 圆角遮罩）
    tile = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    grad = gradient(TILE).convert("RGBA")
    mask = Image.new("L", (TILE, TILE), 0)
    ImageDraw.Draw(mask).rounded_rectangle([0, 0, TILE - 1, TILE - 1], radius=RADIUS, fill=255)
    tile.paste(grad, (INSET, INSET), mask)

    # 2) 自动算出让 "ND" 宽度刚好占 TEXT_WIDTH_RATIO 的字号
    probe = ImageDraw.Draw(tile)
    target_w = TILE * TEXT_WIDTH_RATIO
    size = 100
    while size < 600:
        f = load_font(size)
        gap = size * LETTER_SPACING
        w = probe.textlength("N", font=f) + gap + probe.textlength("D", font=f)
        if w >= target_w:
            break
        size += 1
    font = load_font(size)
    gap = size * LETTER_SPACING

    # 3) 逐字绘制（Pillow 不支持字距，手动排布），按实际墨迹外框居中
    wn = probe.textlength("N", font=font)
    wd = probe.textlength("D", font=font)
    total = wn + gap + wd
    cy = S / 2
    pos = (S - total) / 2
    for ch, adv in (("N", wn), ("D", wd)):
        box = probe.textbbox((0, 0), ch, font=font)          # 以绘制原点为基准的墨迹外框
        ink_cx = (box[0] + box[2]) / 2
        ink_cy = (box[1] + box[3]) / 2
        # 让该字的墨迹中心落在 (pos + adv/2, cy)
        probe.text((pos + adv / 2 - ink_cx, cy - ink_cy), ch, font=font, fill=(255, 255, 255, 255))
        pos += adv + gap
    return tile


def main():
    os.makedirs(ICON_DIR, exist_ok=True)
    master = build_master()

    # 超采样母版整体降采样（BOX = 面积平均，缩小时最干净）
    icon = master.resize((TARGET, TARGET), Image.BOX)

    # 透明度保护：方块外的像素必须完全透明
    icon.putalpha(icon.getchannel("A"))

    icon.save(os.path.join(ICON_DIR, "tray-32.png"))
    with open(os.path.join(ICON_DIR, "tray-32.rgba"), "wb") as f:
        f.write(icon.tobytes())   # RGBA 行优先，top->bottom，正好是 Tauri Image::new 需要的格式

    # 预览条：模拟任务栏上的实际观感
    sizes = [16, 20, 24, 32, 48]
    pad = 16
    strip_w = sum(sizes) + pad * (len(sizes) + 1)
    strip_h = max(sizes) + pad * 2
    strip = Image.new("RGB", (strip_w, strip_h), (243, 243, 243))
    x = pad
    for s in sizes:
        strip.paste(master.resize((s, s), Image.LANCZOS), (x, (strip_h - s) // 2), master.resize((s, s), Image.LANCZOS))
        x += s + pad
    strip.resize((strip_w * 2, strip_h * 2), Image.NEAREST).save(os.path.join(ICON_DIR, "tray-preview.png"))

    print("OK  %s  (%d bytes rgba, %dx%d)" % (
        os.path.join(ICON_DIR, "tray-32.rgba"), os.path.getsize(os.path.join(ICON_DIR, "tray-32.rgba")), TARGET, TARGET))
    print("OK  %s" % os.path.join(ICON_DIR, "tray-32.png"))
    print("OK  %s  (放大 2x 预览，左->右 16/20/24/32/48)" % os.path.join(ICON_DIR, "tray-preview.png"))


if __name__ == "__main__":
    main()
