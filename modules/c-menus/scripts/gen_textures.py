#!/usr/bin/env python3
"""Generate c-menus resource pack textures (pure stdlib: PNG via zlib).

Deterministic output (fixed rgba, no timestamps) — CI rebuilds and verifies
the committed pack zip stays in sync.
"""
import struct
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TEX = ROOT / "pack" / "assets" / "cmenus" / "textures"


def write_png(path: Path, w: int, h: int, px: list[list[tuple[int, int, int, int]]]) -> None:
    raw = b""
    for row in px:
        raw += b"\x00" + b"".join(struct.pack("4B", *p) for p in row)

    def chunk(tag: bytes, data: bytes) -> bytes:
        c = struct.pack(">I", len(data)) + tag + data
        return c + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)

    ihdr = struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0)
    png = (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", ihdr)
           + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b""))
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(png)


def blank(w: int, h: int) -> list[list[tuple[int, int, int, int]]]:
    return [[(0, 0, 0, 0) for _ in range(w)] for _ in range(h)]


def rect(px, x0, y0, x1, y1, color, alpha=None):
    for y in range(y0, y1):
        for x in range(x0, x1):
            if 0 <= y < len(px) and 0 <= x < len(px[0]):
                c = color if alpha is None else (color[0], color[1], color[2], alpha)
                px[y][x] = c


def outline(px, x0, y0, x1, y1, color):
    rect(px, x0, y0, x1, y0 + 1, color)
    rect(px, x0, y1 - 1, x1, y1, color)
    rect(px, x0, y0, x0 + 1, y1, color)
    rect(px, x1 - 1, y0, x1, y1, color)


# ---------------------------------------------------------------------------
# UI panels
# ---------------------------------------------------------------------------

W, H = 96, 60  # matches the 3.7 x 2.3 m main panel aspect (1.61 vs 1.6)


def panel_base(alpha_fill: int, border: tuple) -> list:
    px = blank(W, H)
    rect(px, 0, 0, W, H, (13, 16, 22, alpha_fill))
    # inner sheen (very subtle vertical gradient)
    for y in range(H):
        a = max(0, alpha_fill - y * 2 // 3)
        for x in range(1, W - 1):
            r, g, b, _ = px[y][x]
            px[y][x] = (r, g, min(28, b + 2), a)
    outline(px, 0, 0, W, H, border)
    # corner accents
    for (cx, cy, dx, dy) in [(1, 1, 3, 3), (W - 4, 1, W - 1, 3), (1, H - 4, 3, H - 1), (W - 4, H - 4, W - 1, H - 1)]:
        rect(px, cx, cy, dx + 1, dy + 1, border)
    return px


# main panel: dark translucent + light steel border
write_png(TEX / "ui" / "panel_main.png", W, H, panel_base(232, (94, 106, 128, 255)))
# darker variant
write_png(TEX / "ui" / "panel_dark.png", W, H, panel_base(244, (70, 76, 92, 255)))
# soft variant
write_png(TEX / "ui" / "panel_soft.png", W, H, panel_base(214, (110, 124, 150, 255)))

# animated sheen overlay (8 frames, vertical strip 96 x 60*8) — core-pack
# "living panel" layer, interpolated by the client
FR = 8
frames = []
for f in range(FR):
    px = blank(W, H)
    t = f / FR
    # diagonal light band sweeping through
    for y in range(H):
        for x in range(W):
            band = (x / W + y / H) / 2
            d = (band - t) % 1.0
            glow = max(0.0, 1.0 - abs(d - 0.5) * 6.0)
            a = int(26 * glow)
            if a:
                px[y][x] = (120, 170, 255, a)
    frames.append(px)
strip = blank(W, H * FR)
for f, fr in enumerate(frames):
    for y in range(H):
        strip[f * H + y] = fr[y]
write_png(TEX / "ui" / "panel_sheen.png", W, H * FR, strip)
(TEX / "ui" / "panel_sheen.png.mcmeta").write_text(
    '{"animation":{"frametime":3,"interpolate":true,"frames":[0,1,2,3,4,5,6,7]}}\n'
)

# craft cells 32x32
def cell(hover: bool) -> list:
    px = blank(32, 32)
    rect(px, 1, 1, 31, 31, (16, 19, 26, 235))
    outline(px, 0, 0, 32, 32, (86, 96, 116, 255) if not hover else (133, 200, 255, 255))
    if hover:
        rect(px, 2, 2, 30, 3, (133, 200, 255, 160))
    return px


write_png(TEX / "ui" / "cell.png", 32, 32, cell(False))
write_png(TEX / "ui" / "cell_hover.png", 32, 32, cell(True))

# button 64x16
btn = blank(64, 16)
rect(btn, 1, 1, 63, 15, (24, 34, 30, 240))
outline(btn, 0, 0, 64, 16, (66, 148, 96, 255))
rect(btn, 2, 2, 62, 3, (88, 190, 126, 120))
write_png(TEX / "ui" / "button.png", 64, 16, btn)

# ---------------------------------------------------------------------------
# Item textures
# ---------------------------------------------------------------------------

# jar glass 16x16 (body texture, transparent glass with rim)
g = blank(16, 16)
rect(g, 1, 1, 15, 15, (170, 205, 235, 38))
outline(g, 0, 0, 16, 16, (200, 228, 250, 190))
rect(g, 2, 12, 14, 15, (200, 228, 250, 120))
rect(g, 2, 1, 14, 2, (225, 242, 255, 150))
write_png(TEX / "item" / "jar_glass.png", 16, 16, g)

# powder fill 16x16 (cobalt blue)
p = blank(16, 16)
rect(p, 0, 0, 16, 16, (28, 84, 210, 255))
for y in range(16):
    for x in range(16):
        n = (x * 7 + y * 13) % 5
        if n == 0:
            p[y][x] = (44, 110, 235, 255)
        elif n == 1:
            p[y][x] = (18, 60, 170, 255)
write_png(TEX / "item" / "jar_powder.png", 16, 16, p)

# lid 16x16 (orange metal)
l = blank(16, 16)
rect(l, 0, 0, 16, 16, (208, 108, 34, 255))
outline(l, 0, 0, 16, 16, (246, 150, 62, 255))
rect(l, 1, 1, 15, 3, (246, 150, 62, 255))
rect(l, 3, 6, 13, 8, (178, 88, 22, 255))
write_png(TEX / "item" / "jar_lid.png", 16, 16, l)

# ring 16x16 (gold torus silhouette)
r = blank(16, 16)
cx, cy, ro, ri = 7.5, 7.5, 6.2, 3.1
for y in range(16):
    for x in range(16):
        d = ((x - cx) ** 2 + (y - cy) ** 2) ** 0.5
        if ri <= d <= ro:
            shade = 1.0 - abs(d - (ri + ro) / 2) / ((ro - ri) / 2) * 0.35
            r[y][x] = (int(232 * shade), int(190 * shade), int(64 * shade), 255)
# small gem on top
rect(r, 6, 0, 10, 3, (120, 220, 255, 255))
write_png(TEX / "item" / "ring.png", 16, 16, r)

# powder pile 16x16 (heap of cobalt dust)
h = blank(16, 16)
for y in range(16):
    for x in range(16):
        top = 14 - int(9 * (1 - abs(x - 7.5) / 7.5) ** 0.5)
        if y >= top and 1 <= x <= 14:
            n = (x * 5 + y * 11) % 7
            c = (36, 96, 224, 255)
            if n == 0:
                c = (58, 124, 250, 255)
            elif n == 1:
                c = (24, 70, 180, 255)
            h[y][x] = c
write_png(TEX / "item" / "powder.png", 16, 16, h)

# cobalt dust icon (flat pile, brighter)
d = blank(16, 16)
for y in range(16):
    for x in range(16):
        if h[y][x][3]:
            d[y][x] = (min(255, h[y][x][0] + 30), min(255, h[y][x][1] + 40), min(255, h[y][x][2] + 20), 255)
write_png(TEX / "item" / "cobalt_dust.png", 16, 16, d)

print("textures generated under", TEX)
