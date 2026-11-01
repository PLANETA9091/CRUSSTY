//! Module configuration: file layout, options parsing, config.json.

use serde::Deserialize;
use std::path::{Path, PathBuf};

pub const MODULE_ID: &str = "c-menus";
pub const DEFAULT_PACK_URL: &str =
    "https://raw.githubusercontent.com/PLANETA9091/CRUSSTY/master/modules/c-menus/pack/cmenus-pack.zip";

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct PackCfg {
    /// URL the client downloads the pack from.
    pub url: String,
    /// SHA-1 hex (40 chars) or "" to compute from the embedded zip.
    pub sha1: String,
    pub force: bool,
    pub send_on_join: bool,
}

impl Default for PackCfg {
    fn default() -> Self {
        PackCfg {
            url: DEFAULT_PACK_URL.to_string(),
            sha1: String::new(),
            force: true,
            send_on_join: true,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct AutoOpenCfg {
    pub enabled: bool,
    pub menu: String,
    /// "pack_accepted" (default) or "join".
    pub on: String,
    pub delay_ticks: u64,
}

impl Default for AutoOpenCfg {
    fn default() -> Self {
        AutoOpenCfg {
            enabled: true,
            menu: "item_view".to_string(),
            on: "pack_accepted".to_string(),
            delay_ticks: 10,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct Config {
    pub pack: PackCfg,
    pub auto_open: AutoOpenCfg,
    /// Chat command root ("cmenus"): /cmenus open <id> | /cmenus list
    pub command: String,
    /// Reload menu files every N ticks.
    pub reload_every_ticks: u64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            pack: PackCfg::default(),
            auto_open: AutoOpenCfg::default(),
            command: "cmenus".to_string(),
            reload_every_ticks: 100,
        }
    }
}

impl Config {
    pub fn effective_sha1(&self) -> String {
        if self.pack.sha1.len() == 40 {
            self.pack.sha1.clone()
        } else {
            crate::pack::pack_sha1_hex()
        }
    }
}

/// Parse the runtime options string: "modules=<dir>;versions=<dir>;kernel=<jar>".
pub fn modules_dir_from_options(options: &str) -> Option<PathBuf> {
    for part in options.split(';') {
        let part = part.trim();
        if let Some(rest) = part.strip_prefix("modules=") {
            let p = PathBuf::from(rest.trim());
            if p.is_dir() {
                return Some(p);
            }
        }
    }
    None
}

/// Locate this module's directory: <modules>/c-menus if it carries our
/// module.json, otherwise a scan for a module.json with our id.
pub fn module_dir(modules: &Path) -> Option<PathBuf> {
    let direct = modules.join(MODULE_ID);
    if direct.join("module.json").is_file() {
        return Some(direct);
    }
    let rd = std::fs::read_dir(modules).ok()?;
    for e in rd.flatten() {
        let cand = e.path();
        if cand.is_dir() {
            let mf = cand.join("module.json");
            if let Ok(body) = std::fs::read_to_string(&mf) {
                if body.contains("\"id\"") && body.contains(MODULE_ID) {
                    return Some(cand);
                }
            }
        }
    }
    None
}

/// Load config.json from the module dir (defaults when absent/broken).
pub fn load(dir: &Path) -> (Config, PathBuf) {
    let cfg_path = dir.join("config.json");
    let cfg: Config = std::fs::read_to_string(&cfg_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    (cfg, dir.to_path_buf())
}

/// Load all menu JSON files from <dir>/menus.
pub fn load_menus(dir: &Path) -> Result<Vec<(String, String)>, String> {
    let menus = dir.join("menus");
    let mut out = Vec::new();
    let rd = std::fs::read_dir(&menus)
        .map_err(|e| format!("cannot read {}: {e}", menus.display()))?;
    for e in rd.flatten() {
        let p = e.path();
        if p.extension().and_then(|s| s.to_str()) == Some("json") {
            let body = std::fs::read_to_string(&p)
                .map_err(|e| format!("{}: {e}", p.display()))?;
            out.push((p.file_name().unwrap().to_string_lossy().to_string(), body));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn options_parse() {
        let d = modules_dir_from_options("modules=/srv/m;versions=/srv/v;kernel=kernel.jar");
        assert_eq!(d, None); // dir does not exist in CI
        let d = modules_dir_from_options("versions=x;modules=/tmp");
        assert_eq!(d, Some(PathBuf::from("/tmp"))); // /tmp always exists
        assert_eq!(modules_dir_from_options("kernel=only"), None);
    }

    #[test]
    fn config_defaults() {
        let cfg: Config = serde_json::from_str("{}").unwrap();
        assert!(cfg.auto_open.enabled);
        assert_eq!(cfg.command, "cmenus");
        assert_eq!(cfg.pack.url, DEFAULT_PACK_URL);
    }

    #[test]
    fn module_dir_matches_id() {
        let tmp = std::env::temp_dir().join(format!("cmenus-test-{}", std::process::id()));
        std::fs::create_dir_all(tmp.join("some-module")).unwrap();
        std::fs::write(
            tmp.join("some-module/module.json"),
            r#"{"id":"c-menus","version":"0.1.0"}"#,
        )
        .unwrap();
        std::fs::create_dir_all(tmp.join(MODULE_ID)).unwrap(); // empty, no manifest
        let d = module_dir(&tmp).expect("found via scan");
        assert!(d.ends_with("some-module"));
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
