use std::path::{Path, PathBuf};

pub const RUNTIME_CRATE: &str = "libcrussty_runtime.so";
pub const CONFIG_NAME: &str = "crussty.toml";

pub fn http() -> ureq::Agent {
    ureq::Agent::config_builder().build().into()
}

/// The server directory: cwd by default, or the nearest ancestor containing
/// crussty.toml (so `crussty run` works from modules/ too).
pub fn server_dir() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_default();
    for dir in cwd.ancestors() {
        if dir.join(CONFIG_NAME).is_file() {
            return dir.to_path_buf();
        }
    }
    cwd
}

pub fn server_modules_dir(server: &Path) -> PathBuf {
    server.join("modules")
}

pub fn is_plugin_dir(server: &Path, name: &str) -> bool {
    if name.ends_with(".disabled") || name.ends_with(".x") {
        return false;
    }
    let manifest = server_modules_dir(server).join(name).join("cplugin.json");
    let lib = entry_for(server, name);
    manifest.is_file() && lib.exists()
}

/// Library entry for a module dir: manifest `main` if present, else
/// lib<id>.so (id with any status suffix stripped).
pub fn entry_for(server: &Path, name: &str) -> PathBuf {
    let dir = server_modules_dir(server).join(name);
    let manifest_path = dir.join("cplugin.json");
    if let Ok(text) = std::fs::read_to_string(&manifest_path) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(main) = v.get("main").and_then(|m| m.as_str()) {
                return dir.join(main);
            }
        }
    }
    let id = name
        .strip_suffix(".disabled")
        .or_else(|| name.strip_suffix(".x"))
        .unwrap_or(name);
    dir.join(format!("lib{id}.so"))
}

pub fn jvm_pids(server: &Path) -> Vec<u32> {
    let marker = server.join(RUNTIME_CRATE).to_string_lossy().into_owned();
    let out = std::process::Command::new("pgrep")
        .arg("-f")
        .arg(&marker)
        .output();
    let Ok(out) = out else { return vec![] };
    let text = String::from_utf8_lossy(&out.stdout);
    text.split_whitespace()
        .filter_map(|p| p.parse::<u32>().ok())
        .collect()
}