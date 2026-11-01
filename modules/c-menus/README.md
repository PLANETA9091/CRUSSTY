# c-menus

Fully customizable animated in-world menus for the
[CRUSSTY](https://github.com/PLANETA9091/CRUSSTY) platform.
Target kernel: **Paper 26.3** (Minecraft 26.3 "Wilderness Bound", protocol 777, Java 25).

The menu from the demo screenshot — an item view for «Кобальтовая пыль» with a
3D jar, characteristics panel, 3×3 craft grid, hover highlights and click
actions — ships as `menus/item_view.json` and opens automatically once the
resource pack is accepted.

```
┌──────────────────────────────┬──────────────────────┐
│ Кобальтовая пыль             │ ХАРАКТЕРИСТИКИ       │
│ Красная пыль, спечённая…     │ Количество: 1        │
│        [3D jar]              │ Прочность:  16/16    │
│                              │ В пачке:    1        │
│ ПКМ — высыпать               │ КРАФТ                │
│ ЛКМ — подмести               │ [░][░][░]            │
│                              │ [░][◈][░]  →  (jar)  │
│                              │ [░][░][░]            │
│                              │        [Вернуться F] │
└──────────────────────────────┴──────────────────────┘
```

## How it works

| Layer | Tech |
|---|---|
| Rendering | `TextDisplay` / `ItemDisplay` entities spawned through a tiny Java bridge (`dev.crussty.cmenus.Bridge`), client-side interpolation (`teleportDuration=2`, `interpolationDuration=2`) → 60 fps motion, zero ghosting |
| Animation | Per-tick easing on the server (out_back / out_elastic / out_quint / …), enter / idle / hover / exit states per element |
| Input | CRUSSTY network brick: serverbound play packets are decoded and consumed before vanilla (punch `0x2E`, attack `0x01`, use item `0x43`, use item on `0x42`, interact `0x1A`, F-swap `0x29:6`, hotbar `0x36`, chat command `0x07`); hover = ray/AABB test in menu space |
| Visual FX | Resource pack with **custom core shader** (`assets/minecraft/shaders/core/text.fsh`, GameTime-driven breathing glow), animated panel sheen (`.mcmeta`), custom item models via `minecraft:range_dispatch` on `custom_model_data` |
| Pack delivery | SHA-1-hashed zip pushed on first sighting of a player (`Player#setResourcePack`), default URL = this repo's committed `pack/cmenus-pack.zip` |
| Timing | CRUSSTY scheduler brick (main-thread tick pump) with an automatic `MinecraftServer.execute` fallback if the transform rules don't match the kernel |

## Install

```bash
# inside a CRUSSTY server directory
cargo build --release --manifest-path modules/c-menus/Cargo.toml
cp modules/c-menus/target/release/libcmenus.so modules/c-menus/
crussty reload   # or restart the server
```

The module dir must keep its layout:

```
modules/c-menus/
├── module.json          # {"id":"c-menus", ...}
├── libcmenus.so         # built artifact
├── config.json          # optional (defaults below)
├── menus/*.json         # menu definitions, hot-reloaded every 100 ticks
└── pack/cmenus-pack.zip # optional local override of the pack URL hash source
```

## Controls (demo menu)

| Input | Action |
|---|---|
| `/cmenus open item_view` | open the demo menu |
| `/cmenus close` / **F** | close (exit animation) |
| Mouse move | hover highlight + sound |
| **LMB / RMB** on the jar | pour / sweep (events `cmenus/pour`, `cmenus/sweep`) |
| RMB on the ring | ingredient inspect event |
| RMB on the result | craft event |
| Hotbar scroll | page switching (menus with `pages > 1`) |

While a menu is open the consumed packets never reach vanilla — no accidental
block breaking, no offhand swap, no attacks.

## Menu JSON reference

```jsonc
{
  "id": "item_view",
  "anchor": {                       // where the menu lives
    "mode": "hud",                  // "hud" follows the camera, "world" is fixed
    "distance": 3.8,                // meters in front of the eyes
    "height": -0.25,                // vertical offset
    "yaw_offset": 0                 // menu rotation relative to view yaw
  },
  "pages": 0,                       // hotbar-scroll pages (0/1 = single)
  "on_open": [ { "type": "sound", "sound": "...", "volume": 0.7, "pitch": 1.3 } ],
  "on_close": [ ],
  "elements": [
    {
      "id": "jar",                  // unique
      "page": 0,                    // optional page
      "type": "model",              // "text" | "model" (pack) | "item" (vanilla)
      "model": "jar",               // pack model name (see registry below)
      "item": "minecraft:iron_ingot",
      "pos": [-0.62, -0.42, 0.5],   // menu space: x right, y up, z toward viewer
      "scale": [1.0, 1.0, 1.0],
      "rot": [0, 0, 0],             // base pitch/yaw/roll degrees
      "billboard": 0,               // 0 fixed, 1 center, 2 vertical, 3 horizontal
      "hit": [0.8, 1.0, 0.5],       // hover/click hitbox (meters)
      "view_range": 1.0,
      "brightness": [-1, -1],       // or [block, sky] 0..15
      "anim": {
        "enter": { "delay": 0, "dur": 14, "ease": "out_back",
                   "from": { "offset": [0,-0.3,0], "scale": 0.3, "rot_y": -120, "bg_alpha": 0 } },
        "idle":  { "spin": {"period": 240}, "float": {"amp": 0.02, "period": 85},
                   "pulse": {"amp": 0.03, "period": 60} },
        "hover": { "dur": 5, "ease": "out_quint", "scale": 1.07, "lift": 0.02,
                   "bg_alpha_add": 0.25 },
        "exit":  { "delay": 0, "dur": 9, "ease": "in_cubic",
                   "to": { "scale": 0.25, "rot_y": 90, "bg_alpha": 0 } }
      },
      "on_hover": [ { "type": "sound", "sound": "..." } ],
      "on_click": {
        "left":  [ { "type": "event", "event": "cmenus/sweep", "payload": {} } ],
        "right": [ { "type": "close" } ]
      }
    }
  ]
}
```

### Text elements

`"type": "text"` renders a TextDisplay. `text` supports `&` legacy codes and
inline `#RRGGBB` hex colors; `bg` is `#AARRGGBB` (alpha animated by
enter/hover/exit); `align` = `left|center|right`; `line_width` in font pixels;
`see_through`, `shadow` booleans.

### Pack model registry (`model_cmd`)

| name | CMD |
|---|---|
| panel_main / panel_dark / panel_soft | 1001 / 1002 / 1003 |
| cell / cell_hover / button | 1004 / 1005 / 1006 |
| jar / ring / powder / cobalt_dust | 1101 / 1102 / 1103 / 1104 |

New models: add `assets/cmenus/models/item/<name>.json` + texture, add a
`threshold` entry to `pack/assets/minecraft/items/paper.json`, add the mapping
to `model_cmd()` in `src/menu.rs`, rebuild the pack (`scripts/build_pack.py`).

### Actions

| type | fields | effect |
|---|---|---|
| `sound` | `sound`, `volume`, `pitch` | custom/namespaced sound at the player |
| `msg` | `text` | legacy-colored chat message |
| `cmd` | `cmd` | console command |
| `event` | `event`, `payload` | publish on the CRUSSTY event bus (`cmenus/...`) |
| `close` | — | exit animation + remove |
| `open` | `menu` | open another menu |

## Building from source

```bash
cargo test                       # unit tests (easing, parser, protocol, geometry)
cargo build --release            # -> target/release/libcmenus.so
bash scripts/build_bridge.sh     # recompile the Java bridge (committed artifact)
python3 scripts/gen_textures.py  # regenerate pack textures
python3 scripts/build_pack.py    # rebuild pack/cmenus-pack.zip
node scripts/visual/visual_test.mjs   # visual frames vs golden (needs chromium)
```

`bridge-build/dev/crussty/cmenus/Bridge.class` and `pack/cmenus-pack.zip` are
committed artifacts — CI rebuilds both and fails on drift.

## Resource pack / core shaders

The pack overrides one core shader for Minecraft 26.3 (pack format 97):
`assets/minecraft/shaders/core/text.fsh` adds a subtle GameTime-driven
luminance breathing to all text (menu text included). Delete that one file for
pixel-vanilla text. Panel sheen, animated textures, the 3D jar, cells and
buttons are custom models/textures under the `cmenus` namespace.

## Module config (`config.json`)

```jsonc
{
  "pack": {
    "url": "https://.../cmenus-pack.zip",  // where clients download the pack
    "sha1": "",                             // "" = hash of the embedded zip
    "force": true,
    "send_on_join": true
  },
  "auto_open": { "enabled": true, "menu": "item_view", "on": "pack_accepted", "delay_ticks": 10 },
  "command": "cmenus",                      // /cmenus open|close|list
  "reload_every_ticks": 100
}
```

## E2E

`scripts/e2e.sh` builds the runtime + launcher from the repo, installs
c-menus, boots a real Paper 26.3 server (Java 25) and greps the log for
`pipeline ready`, `[c-menus] ready:` and `[c-menus] tick ... alive` markers.
