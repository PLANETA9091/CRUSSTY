//! Menu definition model + JSON parsing.
//!
//! A menu is defined by a JSON file (see menus/*.json): an anchor, a list of
//! elements (text panels / custom-model items) with per-element animation
//! sets (enter / idle / hover / exit) and click/hover actions.

use serde::Deserialize;
use std::collections::HashMap;

use crate::easing::Ease;

// ---------------------------------------------------------------------------
// Model name -> custom model data registry (must match pack paper.json)
// ---------------------------------------------------------------------------

pub fn model_cmd(name: &str) -> Option<i32> {
    let cmd = match name {
        "panel" | "panel_main" => 1001,
        "panel_dark" => 1002,
        "panel_soft" => 1003,
        "cell" => 1004,
        "cell_hover" => 1005,
        "button" => 1006,
        "jar" => 1101,
        "ring" => 1102,
        "powder" => 1103,
        "cobalt_dust" => 1104,
        _ => return None,
    };
    Some(cmd)
}

// ---------------------------------------------------------------------------
// Actions
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    Sound {
        sound: String,
        #[serde(default = "default_volume")]
        volume: f32,
        #[serde(default = "default_pitch")]
        pitch: f32,
    },
    Msg {
        text: String,
    },
    Cmd {
        cmd: String,
    },
    Event {
        event: String,
        #[serde(default)]
        payload: serde_json::Value,
    },
    Close,
    Open {
        menu: String,
    },
}

fn default_volume() -> f32 {
    1.0
}
fn default_pitch() -> f32 {
    1.0
}

// ---------------------------------------------------------------------------
// Animations
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Deserialize, Default)]
pub struct FromTo {
    #[serde(default)]
    pub offset: [f64; 3],
    #[serde(default = "one_f")]
    pub scale: f64,
    #[serde(default)]
    pub rot_x: f64,
    #[serde(default)]
    pub rot_y: f64,
    #[serde(default)]
    pub rot_z: f64,
    /// Background alpha multiplier 0..1 (text panels).
    #[serde(default = "one_f")]
    pub bg_alpha: f64,
}

fn one_f() -> f64 {
    1.0
}

#[derive(Clone, Copy, Debug, Deserialize, Default)]
pub struct EnterExit {
    #[serde(default)]
    pub delay: f64,
    #[serde(default = "default_dur")]
    pub dur: f64,
    #[serde(default)]
    pub ease: Ease,
    #[serde(default)]
    pub from: FromTo,
}

fn default_dur() -> f64 {
    10.0
}

#[derive(Clone, Copy, Debug, Deserialize, Default)]
pub struct Idle {
    #[serde(default)]
    pub float: Option<Osc>,
    #[serde(default)]
    pub spin: Option<Spin>,
    #[serde(default)]
    pub pulse: Option<Pulse>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Osc {
    pub amp: f64,
    #[serde(default = "default_period")]
    pub period: f64,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Spin {
    /// Ticks per full rotation. 0 = no spin.
    pub period: f64,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Pulse {
    pub amp: f64,
    #[serde(default = "default_period")]
    pub period: f64,
}

fn default_period() -> f64 {
    80.0
}

#[derive(Clone, Copy, Debug, Deserialize, Default)]
pub struct Hover {
    #[serde(default = "default_hover_dur")]
    pub dur: f64,
    #[serde(default)]
    pub ease: Ease,
    #[serde(default = "one_f")]
    pub scale: f64,
    #[serde(default)]
    pub lift: f64,
    /// Added to background alpha 0..1.
    #[serde(default)]
    pub bg_alpha_add: f64,
}

fn default_hover_dur() -> f64 {
    5.0
}

#[derive(Clone, Copy, Debug, Deserialize, Default)]
pub struct AnimSet {
    #[serde(default)]
    pub enter: Option<EnterExit>,
    #[serde(default)]
    pub idle: Option<Idle>,
    #[serde(default)]
    pub hover: Option<Hover>,
    #[serde(default)]
    pub exit: Option<EnterExit>,
}

// ---------------------------------------------------------------------------
// Elements
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AnchorMode {
    #[default]
    Hud,
    World,
}

/// Flattened element body: `type` selects the variant ("text" | "model" |
/// "item"); the remaining fields depend on that type.
#[derive(Clone, Debug, Deserialize, Default)]
pub struct Kind {
    #[serde(rename = "type", default)]
    pub kind: String,
    /// text element body
    #[serde(default)]
    pub text: String,
    /// ARGB hex "#AARRGGBB" or "" for none.
    #[serde(default)]
    pub bg: String,
    #[serde(default)]
    pub align: String,
    #[serde(default = "default_line_width")]
    pub line_width: i32,
    #[serde(default)]
    pub see_through: bool,
    #[serde(default)]
    pub shadow: bool,
    /// model element body: named model from the pack registry
    #[serde(default)]
    pub model: String,
    /// item element body: vanilla material
    #[serde(default)]
    pub item: String,
}

fn default_line_width() -> i32 {
    220
}

#[derive(Clone, Debug, Deserialize)]
pub struct Element {
    pub id: String,
    #[serde(default)]
    pub page: i32,
    /// Menu-space position: x right, y up, z toward the viewer.
    pub pos: [f64; 3],
    /// Hitbox size in meters (used for hover/click ray tests).
    #[serde(default)]
    pub hit: Option<[f64; 3]>,
    #[serde(default = "default_scale")]
    pub scale: [f64; 3],
    #[serde(default)]
    pub rot: [f64; 3],
    /// 0 fixed, 1 center, 2 vertical, 3 horizontal.
    #[serde(default)]
    pub billboard: u8,
    #[serde(default = "one_f")]
    pub view_range: f64,
    #[serde(default)]
    pub shadow_radius: f64,
    #[serde(default)]
    pub shadow_strength: f64,
    /// [-1,-1] = default; otherwise [block_light, sky_light] 0..15.
    #[serde(default = "default_brightness")]
    pub brightness: [i32; 2],
    #[serde(default = "default_hoverable")]
    pub hoverable: bool,
    #[serde(flatten)]
    pub kind: Kind,
    #[serde(default)]
    pub anim: AnimSet,
    #[serde(default)]
    pub on_click: HashMap<String, Vec<Action>>,
    #[serde(default)]
    pub on_hover: Vec<Action>,
}

fn default_scale() -> [f64; 3] {
    [1.0, 1.0, 1.0]
}
fn default_brightness() -> [i32; 2] {
    [-1, -1]
}
fn default_hoverable() -> bool {
    true
}

impl Element {
    /// Default hitbox when "hit" is not set.
    pub fn hitbox(&self) -> [f64; 3] {
        self.hit.unwrap_or(match self.kind.kind.as_str() {
            "text" => [1.2, 0.4, 0.3],
            _ => [0.6, 0.6, 0.3],
        })
    }
}

// ---------------------------------------------------------------------------
// Menu
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Deserialize)]
pub struct Anchor {
    #[serde(default)]
    pub mode: AnchorMode,
    #[serde(default = "default_distance")]
    pub distance: f64,
    #[serde(default)]
    pub height: f64,
    #[serde(default)]
    pub yaw_offset: f64,
}

fn default_distance() -> f64 {
    3.6
}

impl Default for Anchor {
    fn default() -> Self {
        Anchor {
            mode: AnchorMode::Hud,
            distance: default_distance(),
            height: 0.0,
            yaw_offset: 0.0,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct MenuDef {
    pub id: String,
    #[serde(default)]
    pub anchor: Anchor,
    /// Number of scroll pages (elements carry "page").
    #[serde(default)]
    pub pages: i32,
    #[serde(default)]
    pub elements: Vec<Element>,
    #[serde(default)]
    pub on_open: Vec<Action>,
    #[serde(default)]
    pub on_close: Vec<Action>,
}

// ---------------------------------------------------------------------------
// Parsing helpers
// ---------------------------------------------------------------------------

/// Convert "#RRGGBB" spans into legacy §x sequences and "&x" codes into §x.
pub fn legacy(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 16);
    let bytes: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == '#' && i + 6 < bytes.len() {
            let hex: String = bytes[i + 1..i + 7].iter().collect();
            if hex.chars().all(|c| c.is_ascii_hexdigit()) {
                out.push('§');
                out.push('x');
                for c in hex.chars() {
                    out.push('§');
                    out.push(c.to_ascii_uppercase());
                }
                i += 7;
                continue;
            }
        }
        if bytes[i] == '&' && i + 1 < bytes.len() {
            let n = bytes[i + 1];
            if "0123456789abcdefklmnorABCDEFKLMNOR".contains(n) {
                out.push('§');
                out.push(n.to_ascii_lowercase());
                i += 2;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    out
}

/// "#AARRGGBB" or "#RRGGBB" -> ARGB int; 0 when empty/invalid.
pub fn parse_argb(s: &str) -> u32 {
    let h = s.trim().trim_start_matches('#');
    if h.is_empty() {
        return 0;
    }
    u32::from_str_radix(h, 16).unwrap_or(0)
}

pub fn parse_menu(json: &str) -> Result<MenuDef, String> {
    let def: MenuDef = serde_json::from_str(json).map_err(|e| format!("json: {e}"))?;
    if def.id.is_empty() {
        return Err("menu id is empty".into());
    }
    for el in &def.elements {
        match el.kind.kind.as_str() {
            "text" => {
                if el.kind.text.is_empty() {
                    return Err(format!("element {}: text element with empty text", el.id));
                }
            }
            "model" => {
                if model_cmd(&el.kind.model).is_none() {
                    return Err(format!("element {}: unknown model {}", el.id, el.kind.model));
                }
            }
            "item" => {
                if el.kind.item.trim().is_empty() {
                    return Err(format!("element {}: empty item", el.id));
                }
            }
            other => return Err(format!("element {}: unknown type {other}", el.id)),
        }
    }
    Ok(def)
}

/// Validate a whole menus directory: parse every *.json and report errors.
#[allow(dead_code)]
pub fn validate_dir(entries: &HashMap<String, String>) -> Result<usize, String> {
    let mut n = 0;
    for (name, body) in entries {
        if let Err(e) = parse_menu(body) {
            return Err(format!("{name}: {e}"));
        }
        n += 1;
    }
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r##"{
        "id": "test_menu",
        "anchor": {"mode": "hud", "distance": 3.0, "height": -0.2},
        "pages": 1,
        "on_open": [{"type": "sound", "sound": "cmenus:ui.open"}],
        "elements": [
            {
                "id": "title",
                "type": "text",
                "pos": [-0.5, 0.4, 0.2],
                "text": "&l&#33CCFFCobalt",
                "bg": "#E6101418",
                "align": "left",
                "anim": {
                    "enter": {"delay": 4, "dur": 12, "ease": "out_back",
                              "from": {"offset": [0, -0.3, 0], "scale": 0.5, "bg_alpha": 0.0}},
                    "idle": {"float": {"amp": 0.02, "period": 60}},
                    "hover": {"dur": 5, "ease": "out_quint", "scale": 1.08, "lift": 0.02},
                    "exit": {"dur": 8, "ease": "in_cubic", "to": {"scale": 0.4, "bg_alpha": 0.0}}
                },
                "on_click": {"left": [{"type": "close"}]}
            },
            {
                "id": "jar",
                "type": "model",
                "pos": [0.4, 0.0, 0.5],
                "model": "jar",
                "scale": [1.3, 1.3, 1.3],
                "anim": {"idle": {"spin": {"period": 200}}}
            },
            {
                "id": "raw",
                "type": "item",
                "pos": [0.4, -0.5, 0.5],
                "item": "minecraft:iron_ingot"
            }
        ]
    }"##;

    #[test]
    fn parse_sample() {
        let m = parse_menu(SAMPLE).expect("parses");
        assert_eq!(m.id, "test_menu");
        assert_eq!(m.elements.len(), 3);
        assert_eq!(m.anchor.distance, 3.0);
        let title = &m.elements[0];
        assert_eq!(title.anim.enter.as_ref().unwrap().delay, 4.0);
        assert_eq!(title.on_click.get("left").map(|v| v.len()), Some(1));
        let jar = &m.elements[1];
        assert_eq!(jar.kind.kind, "model");
        assert_eq!(model_cmd(&jar.kind.model), Some(1101));
    }

    #[test]
    fn legacy_codes() {
        assert_eq!(legacy("&l&fHi"), "\u{a7}l\u{a7}fHi");
        assert_eq!(legacy("#33CCFFx"), "\u{a7}x\u{a7}3\u{a7}3\u{a7}C\u{a7}C\u{a7}F\u{a7}Fx");
        assert_eq!(legacy("plain"), "plain");
        assert_eq!(legacy("&z"), "&z");
    }

    #[test]
    fn argb() {
        assert_eq!(parse_argb("#E6101418"), 0xE6101418);
        assert_eq!(parse_argb("101418"), 0x00101418);
        assert_eq!(parse_argb(""), 0);
    }

    #[test]
    fn rejects_unknown_model() {
        let bad = r#"{"id":"m","elements":[{"id":"x","type":"model","pos":[0,0,0],"model":"nope"}]}"#;
        assert!(parse_menu(bad).is_err());
    }
}
