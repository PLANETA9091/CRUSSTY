#!/usr/bin/env python3
"""Render crussty demo as a "desktop scene" gif: the user's wallpaper, a plain
dark rounded terminal window, camera zooms in, terminal plays (frames from
agg), then ^C + clear, camera zooms back out.

Usage: render.py [input_terminal.gif] [output.gif]
Env:   CRUSSTY_DEMO_WALLPAPER — path to the wallpaper used as the scene
       background (defaults to the end4 location ~/.local/share/wallpaper.png).
"""
import os
import sys
from PIL import Image, ImageDraw, ImageFont

TERM_W, TERM_H = 718, 528          # agg render size (90x28 @ font 13)
SCENE_W, SCENE_H = 1600, 1000      # desktop canvas
FRAME_W, FRAME_H = 900, 620        # output frame size
WALL = os.environ.get("CRUSSTY_DEMO_WALLPAPER", "/home/btw/.local/share/wallpaper.png")
TERM_BG = (12, 12, 14)             # single flat terminal background
PROMPT = "$ "
FONT = "/home/btw/.fonts/windows/JetBrainsMono-Regular.ttf"

if not os.path.exists(FONT):
    FONT = "/usr/share/fonts/TTF/DejaVuSansMono.ttf"

# ---------------------------------------------------------------- easing
def ease_in_out(t):
    return t * t * (3 - 2 * t)

def camera_box(t):
    """Camera crop box on the scene for zoom factor t in [0,1].
    t=0 -> whole desktop; t=1 -> terminal with small margins."""
    w = SCENE_W + (FRAME_W * 0.96 - SCENE_W) * t
    h = SCENE_H + (FRAME_H * 0.96 - SCENE_H) * t
    x = (SCENE_W - w) / 2
    y = (SCENE_H - h) / 2
    return (x, y, x + w, y + h)

# ---------------------------------------------------------------- scene
def draw_desktop(canvas):
    wall = Image.open(WALL).convert("RGB")
    scale = max(SCENE_W / wall.width, SCENE_H / wall.height)
    wall = wall.resize((int(wall.width * scale), int(wall.height * scale)), Image.LANCZOS)
    x = (wall.width - SCENE_W) // 2
    y = (wall.height - SCENE_H) // 2
    canvas.paste(wall.crop((x, y, x + SCENE_W, y + SCENE_H)), (0, 0))

def rounded_mask(w, h, radius):
    mask = Image.new("L", (w, h), 0)
    ImageDraw.Draw(mask).rounded_rectangle([(0, 0), (w - 1, h - 1)], radius, fill=255)
    return mask

def draw_terminal(canvas, box, content=None, prompt=True):
    """Terminal window: soft shadow, flat dark body with rounded corners,
    then optional content (RGB image) and/or a $ prompt line, both clipped
    to the same rounded shape."""
    x0, y0, x1, y1 = box
    w, h = x1 - x0, y1 - y0
    # soft shadow so the window reads against any wallpaper
    sh = Image.new("RGBA", canvas.size, (0, 0, 0, 0))
    ImageDraw.Draw(sh).rounded_rectangle(
        [(x0 + 5, y0 + 12), (x1 + 5, y1 + 12)], radius=18, fill=(0, 0, 0, 90))
    canvas.alpha_composite(sh)
    # flat dark body, rounded
    body = Image.new("RGBA", (w, h), TERM_BG + (255,))
    canvas.paste(body, (x0, y0), rounded_mask(w, h, 14))
    # content clipped to rounded shape
    if content is not None:
        layer = Image.new("RGBA", canvas.size, (0, 0, 0, 0))
        layer.paste(content.resize((w, h), Image.LANCZOS), (x0, y0), rounded_mask(w, h, 14))
        canvas.alpha_composite(layer)
    if prompt:
        f = ImageFont.truetype(FONT, 13)
        d = ImageDraw.Draw(canvas)
        d.text((x0 + 13, y0 + 15), PROMPT, font=f, fill=(213, 216, 224))

# ---------------------------------------------------------------- main
def build():
    src = Image.open("demo_terminal.gif")
    n = src.n_frames
    durs = []
    frames = []
    for i in range(n):
        src.seek(i)
        frames.append(src.convert("RGB").copy())
        durs.append(src.info.get("duration", 100))

    # terminal window rect on the scene (fixed, camera moves)
    tw = int(TERM_W * 1.05)
    th = int(TERM_H * 1.05)
    bx = (SCENE_W - tw) // 2
    by = (SCENE_H - th) // 2 - 10
    win = (bx, by, bx + tw, by + th)

    out_frames, out_durs = [], []

    def scene():
        canvas = Image.new("RGBA", (SCENE_W, SCENE_H))
        draw_desktop(canvas)
        return canvas

    def emit(canvas, dur):
        out_frames.append(canvas.crop(camera_box(1.0)).resize(
            (FRAME_W, FRAME_H), Image.LANCZOS).convert("P",
            palette=Image.ADAPTIVE, colors=256))
        out_durs.append(dur)

    # ---- phase 1: zoom in over empty terminal (2.5s, 25 frames) ----
    zoom_frames = 25
    for k in range(zoom_frames):
        canvas = scene()
        draw_terminal(canvas, win, content=None, prompt=True)
        box = camera_box(ease_in_out(k / (zoom_frames - 1)))
        out_frames.append(canvas.crop(box).resize((FRAME_W, FRAME_H), Image.LANCZOS).convert("P",
            palette=Image.ADAPTIVE, colors=256))
        out_durs.append(100)

    # ---- phase 2: terminal plays (agg frames), camera fixed ---------
    for i, f in enumerate(frames):
        canvas = scene()
        draw_terminal(canvas, win, content=f, prompt=False)
        emit(canvas, durs[i])

    # ---- phase 3: zoom back out over empty terminal (2.5s) ---------
    for k in range(zoom_frames):
        canvas = scene()
        draw_terminal(canvas, win, content=None, prompt=True)
        box = camera_box(ease_in_out(1 - k / (zoom_frames - 1)))
        out_frames.append(canvas.crop(box).resize((FRAME_W, FRAME_H), Image.LANCZOS).convert("P",
            palette=Image.ADAPTIVE, colors=256))
        out_durs.append(100)

    out_frames[0].save(sys.argv[2] if len(sys.argv) > 2 else "demo_final.gif", save_all=True, append_images=out_frames[1:],
                       duration=out_durs, loop=0, optimize=True)
    print(f"saved demo_final.gif: {len(out_frames)} frames, "
          f"{sum(out_durs) / 1000:.1f}s, {FRAME_W}x{FRAME_H}")

if __name__ == "__main__":
    build()
