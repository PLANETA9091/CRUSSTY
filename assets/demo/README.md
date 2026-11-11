# Demo gif generator

Regenerates `assets/demo.gif` (the desktop-scene recording shown at the top
of the README).

## Prerequisites

- [`asciinema`](https://asciinema.org/docs/installation) (pip)
- [`agg`](https://github.com/asciinema/agg) — asciicast → gif
- Python 3 with Pillow

## Steps

1. Record the terminal session from an empty workdir:

   ```bash
   asciinema rec --cols 90 --rows 28 -t "crussty demo" demo.cast \
     --command "bash <repo>/assets/demo/demo.sh"
   ```

   `demo.sh` needs `crussty` on PATH (use a freshly built `cli/target/release`
   or `npm i -g crussty`). The first `crussty run` boots a real Purpur server
   and generates the world, so the recording takes a few minutes.

2. Render the terminal gif with the flat `#0c0c0e` theme (must match
   `TERM_BG` in `render.py`):

   ```bash
   agg --theme 0c0c0e,d5d8e0,0c0c0e,777777,ff5555,50fa7b,f1fa8c,bd93f9,ff79c6,8be9fd,f8f8f2,343746,ff6e67,5af78e,f4f99d,caa9fa,ff92d0,9aedfe \
       --speed 2.0 --idle-time-limit 1.2 --font-size 13 \
       --cols 90 --rows 28 demo.cast demo_terminal.gif
   ```

3. Build the desktop-scene gif (wallpaper background, camera zoom-in/out,
   `$` prompt on the idle terminal, `^C` + `clear` ending):

   ```bash
   python3 <repo>/assets/demo/render.py demo_terminal.gif demo_final.gif
   ```

4. Replace `assets/demo.gif` with `demo_final.gif`.

Set `CRUSSTY_DEMO_WALLPAPER` to use a different wallpaper than the default
`~/.local/share/wallpaper.png`.
