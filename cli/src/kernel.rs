//! Kernel (server jar) catalog for `crussty init`: supported platforms,
//! version discovery and auto-download.
//!
//! Crussty injects into any Minecraft server .jar — Paper, Purpur, Fabric,
//! Forge or vanilla. The download step is entirely optional: `init` never
//! fetches a kernel unless the user picks one (interactively or via flags).

use std::path::Path;

const UA: &str = "crussty-cli (github.com/PLANETA9091/CRUSSTY)";

pub struct Platform {
    pub id: &'static str,
    pub label: &'static str,
}

pub const PLATFORMS: &[Platform] = &[
    Platform { id: "paper", label: "Paper" },
    Platform { id: "purpur", label: "Purpur" },
    Platform { id: "fabric", label: "Fabric" },
    Platform { id: "forge", label: "Forge" },
];

/// A downloaded kernel: the file placed inside `versions/` plus an optional
/// post-install note. For Forge the bootable artifact is the installer's
/// args file (`versions/libraries/.../unix_args.txt`), not a plain jar —
/// `kernel` in crussty.toml points there and the launcher boots it via
/// `java @<file>`.
pub struct Kernel {
    /// Path of the bootable artifact relative to `versions/`
    /// (e.g. "paper-1.21.10.jar" or "libraries/net/minecraftforge/forge/…/unix_args.txt").
    pub kernel: String,
    /// Downloaded/kept file name to mention in the summary.
    pub file: String,
    /// Extra instructions shown after init (e.g. Forge notes).
    pub note: Option<String>,
}

pub fn find(id: &str) -> Option<&'static Platform> {
    PLATFORMS.iter().find(|p| p.id == id)
}

// ---------------------------------------------------------------------------
// HTTP helpers
// ---------------------------------------------------------------------------

fn get_json(url: &str) -> Result<serde_json::Value, String> {
    let resp = crate::lib::http()
        .get(url)
        .header("User-Agent", UA)
        .call()
        .map_err(|e| format!("{url}: {e}"))?;
    let body = resp
        .into_body()
        .with_config()
        .limit(32 << 20)
        .read_to_vec()
        .map_err(|e| format!("{url}: {e}"))?;
    serde_json::from_slice(&body).map_err(|e| format!("{url}: bad json: {e}"))
}

fn download(url: &str, dest: &Path) -> Result<u64, String> {
    if let Some(parent) = dest.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let resp = crate::lib::http()
        .get(url)
        .header("User-Agent", UA)
        .call()
        .map_err(|e| format!("{url}: {e}"))?;
    let body = resp
        .into_body()
        .with_config()
        .limit(2 << 30)
        .read_to_vec()
        .map_err(|e| format!("{url}: {e}"))?;
    let len = body.len() as u64;
    std::fs::write(dest, &body).map_err(|e| format!("{}: {e}", dest.display()))?;
    Ok(len)
}

fn mb(bytes: u64) -> String {
    format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
}

// ---------------------------------------------------------------------------
// Version discovery
// ---------------------------------------------------------------------------

/// List available versions for a platform, newest first. Pre-releases
/// (-pre/-rc/-snapshot/-eab) are filtered out when at least one stable
/// version exists, so the interactive list stays clean.
pub fn list_versions(p: &Platform) -> Result<Vec<String>, String> {
    let all = match p.id {
        "paper" => paper_versions()?,
        "purpur" => purpur_versions()?,
        "fabric" => fabric_versions()?,
        "forge" => forge_versions()?,
        other => return Err(format!("unknown platform '{other}'")),
    };
    let stable: Vec<String> = all.iter().filter(|v| is_stable(v)).cloned().collect();
    if stable.is_empty() {
        Ok(all)
    } else {
        Ok(stable)
    }
}

fn paper_versions() -> Result<Vec<String>, String> {
    let v = get_json("https://fill.papermc.io/v3/projects/paper/versions")?;
    let mut out = Vec::new();
    if let Some(items) = v.get("versions").and_then(|x| x.as_array()) {
        for item in items {
            if let Some(id) = item
                .get("version")
                .and_then(|x| x.get("id"))
                .and_then(|x| x.as_str())
            {
                out.push(id.to_string());
            }
        }
    }
    if out.is_empty() {
        return Err("paper: empty version list (fill.papermc.io v3)".into());
    }
    sort_desc(&mut out);
    Ok(out)
}

fn purpur_versions() -> Result<Vec<String>, String> {
    let v = get_json("https://api.purpurmc.org/v2/purpur")?;
    let mut out: Vec<String> = v
        .get("versions")
        .and_then(|x| x.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str())
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default();
    if out.is_empty() {
        return Err("purpur: empty version list (api.purpurmc.org v2)".into());
    }
    sort_desc(&mut out);
    Ok(out)
}

fn fabric_versions() -> Result<Vec<String>, String> {
    let v = get_json("https://meta.fabricmc.net/v2/versions/game")?;
    let mut out: Vec<String> = v
        .as_array()
        .map(|a| {
            a.iter()
                .filter(|x| x.get("stable").and_then(|s| s.as_bool()) == Some(true))
                .filter_map(|x| x.get("version").and_then(|s| s.as_str()))
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default();
    if out.is_empty() {
        return Err("fabric: empty version list (meta.fabricmc.net v2)".into());
    }
    sort_desc(&mut out);
    Ok(out)
}

fn forge_versions() -> Result<Vec<String>, String> {
    let v = get_json("https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json")?;
    let mut out: Vec<String> = Vec::new();
    if let Some(promos) = v.get("promos").and_then(|x| x.as_object()) {
        for key in promos.keys() {
            if let Some(mc) = key
                .strip_suffix("-recommended")
                .or_else(|| key.strip_suffix("-latest"))
            {
                if !out.iter().any(|x| x == mc) {
                    out.push(mc.to_string());
                }
            }
        }
    }
    if out.is_empty() {
        return Err("forge: empty version list (promotions_slim.json)".into());
    }
    sort_desc(&mut out);
    Ok(out)
}

/// The Forge build number for a Minecraft version: `-latest` promo first,
/// `-recommended` as fallback.
fn forge_build(mc: &str) -> Result<String, String> {
    let v = get_json("https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json")?;
    let promos = v.get("promos").and_then(|x| x.as_object()).ok_or("forge: bad promos")?;
    for suffix in ["-latest", "-recommended"] {
        if let Some(b) = promos.get(format!("{mc}{suffix}").as_str()).and_then(|x| x.as_str()) {
            return Ok(b.to_string());
        }
    }
    Err(format!("forge: no build found for {mc}"))
}

// ---------------------------------------------------------------------------
// Version sorting helpers
// ---------------------------------------------------------------------------

fn is_stable(v: &str) -> bool {
    // pre/release candidates and early-access builds carry a dash suffix
    // ("1.21.11-pre5", "26.3-rc-3"); stable versions ("1.21.10", "26.3") do not.
    if v.contains('-') {
        return false;
    }
    // weekly snapshots have no dash: "25w14a", "24w40a", "25w14craftmine" …
    let b = v.as_bytes();
    if b.len() >= 4
        && b[0].is_ascii_digit()
        && b[1].is_ascii_digit()
        && b[2] == b'w'
        && b[3].is_ascii_digit()
    {
        return false;
    }
    true
}

/// Natural, numeric-aware sort key: "1.21.10" < "1.21.11" < "26.3".
fn nat_key(v: &str) -> Vec<(u64, String)> {
    let mut key = Vec::new();
    let mut num = String::new();
    let mut txt = String::new();
    let flush = |num: &mut String, txt: &mut String, key: &mut Vec<(u64, String)>| {
        if !num.is_empty() {
            key.push((num.parse().unwrap_or(0), String::new()));
            num.clear();
        }
        if !txt.is_empty() {
            key.push((0, std::mem::take(txt)));
        }
    };
    for c in v.chars() {
        if c.is_ascii_digit() {
            if !txt.is_empty() {
                flush(&mut num, &mut txt, &mut key);
            }
            num.push(c);
        } else {
            txt.push(c.to_ascii_lowercase());
        }
    }
    flush(&mut num, &mut txt, &mut key);
    key
}

fn sort_desc(items: &mut [String]) {
    items.sort_by(|a, b| nat_key(b).cmp(&nat_key(a)));
}

// ---------------------------------------------------------------------------
// Download
// ---------------------------------------------------------------------------

/// Download (and for Forge, install) the chosen kernel into `versions_dir`.
/// Returns the artifact to write into crussty.toml.
pub fn download_kernel(p: &Platform, mc: &str, versions_dir: &Path) -> Result<Kernel, String> {
    match p.id {
        "paper" => download_paper(mc, versions_dir),
        "purpur" => download_purpur(mc, versions_dir),
        "fabric" => download_fabric(mc, versions_dir),
        "forge" => download_forge(mc, versions_dir),
        other => Err(format!("unknown platform '{other}'")),
    }
}

fn download_paper(mc: &str, versions_dir: &Path) -> Result<Kernel, String> {
    let build = get_json(&format!(
        "https://fill.papermc.io/v3/projects/paper/versions/{mc}/builds/latest"
    ))?;
    let url = build
        .get("downloads")
        .and_then(|d| d.get("server:default"))
        .and_then(|d| d.get("url"))
        .and_then(|u| u.as_str())
        .ok_or("paper: no download url in latest build")?;
    let name = format!("paper-{mc}.jar");
    let dest = versions_dir.join(&name);
    println!("crussty: downloading Paper {mc} ...");
    let len = download(url, &dest)?;
    println!("crussty:   {} ({})", dest.display(), mb(len));
    Ok(Kernel { kernel: name.clone(), file: name, note: None })
}

fn download_purpur(mc: &str, versions_dir: &Path) -> Result<Kernel, String> {
    let url = format!("https://api.purpurmc.org/v2/purpur/{mc}/latest/download");
    let name = format!("purpur-{mc}.jar");
    let dest = versions_dir.join(&name);
    println!("crussty: downloading Purpur {mc} ...");
    let len = download(&url, &dest)?;
    println!("crussty:   {} ({})", dest.display(), mb(len));
    Ok(Kernel { kernel: name.clone(), file: name, note: None })
}

fn download_fabric(mc: &str, versions_dir: &Path) -> Result<Kernel, String> {
    let loader = get_json(&format!(
        "https://meta.fabricmc.net/v2/versions/loader/{mc}"
    ))?;
    let loader = loader
        .as_array()
        .and_then(|a| a.first())
        .and_then(|x| x.get("loader"))
        .and_then(|l| l.get("version"))
        .and_then(|v| v.as_str())
        .ok_or("fabric: no loader version")?
        .to_string();
    let installer = get_json("https://meta.fabricmc.net/v2/versions/installer")?;
    let installer = installer
        .as_array()
        .and_then(|a| a.first())
        .and_then(|x| x.get("version"))
        .and_then(|v| v.as_str())
        .ok_or("fabric: no installer version")?
        .to_string();
    let url = format!(
        "https://meta.fabricmc.net/v2/versions/loader/{mc}/{loader}/{installer}/server/jar"
    );
    let name = format!("fabric-{mc}.jar");
    let dest = versions_dir.join(&name);
    println!("crussty: downloading Fabric {mc} (loader {loader}) ...");
    let len = download(&url, &dest)?;
    println!("crussty:   {} ({})", dest.display(), mb(len));
    Ok(Kernel { kernel: name.clone(), file: name, note: None })
}

fn download_forge(mc: &str, versions_dir: &Path) -> Result<Kernel, String> {
    let build = forge_build(mc)?;
    let installer_name = format!("forge-{mc}-{build}-installer.jar");
    let installer_path = versions_dir.join(&installer_name);
    println!("crussty: downloading Forge {mc}-{build} ...");
    let len = download(
        &format!(
            "https://maven.minecraftforge.net/net/minecraftforge/forge/{mc}-{build}/forge-{mc}-{build}-installer.jar"
        ),
        &installer_path,
    )?;
    println!("crussty:   {} ({})", installer_path.display(), mb(len));

    // Forge ships as an installer: run it once (--installServer) so the
    // bootable libraries/ + args file land next to the other kernels.
    println!("crussty: installing Forge server (java -jar {installer_name} --installServer .) ...");
    let status = std::process::Command::new("java")
        .arg("-jar")
        .arg(&installer_path)
        .arg("--installServer")
        .arg(".")
        .current_dir(versions_dir)
        .status();
    let args_file = if cfg!(windows) { "win_args.txt" } else { "unix_args.txt" };
    let rel = format!("libraries/net/minecraftforge/forge/{mc}-{build}/{args_file}");
    match status {
        Ok(s) if s.success() && versions_dir.join(&rel).exists() => Ok(Kernel {
            kernel: rel.clone(),
            file: installer_name.clone(),
            note: Some(format!(
                "Forge installed. It boots from versions/ — run the server as usual (`crussty run`),\n  and put eula.txt into versions/ (echo eula=true > versions/eula.txt)."
            )),
        }),
        _ => Err(format!(
            "forge: auto-install failed — run it manually:\n  cd {} && java -jar {installer_name} --installServer .",
            versions_dir.display()
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn natural_sort_descends() {
        let mut v = vec![
            "1.14.1".into(),
            "1.21.10".into(),
            "26.3".into(),
            "1.21.11".into(),
            "1.21.9".into(),
            "26.1".into(),
        ];
        sort_desc(&mut v);
        assert_eq!(v, vec!["26.3", "26.1", "1.21.11", "1.21.10", "1.21.9", "1.14.1"]);
    }

    #[test]
    fn natural_sort_handles_multi_digit_chunks() {
        // Lexicographic would put 1.21.10 before 1.21.9; numeric-aware must not.
        let mut v = vec!["1.21.9".into(), "1.21.10".into(), "1.21.100".into()];
        sort_desc(&mut v);
        assert_eq!(v, vec!["1.21.100", "1.21.10", "1.21.9"]);
    }

    #[test]
    fn stability_by_dash() {
        assert!(is_stable("1.21.10"));
        assert!(is_stable("26.3"));
        assert!(!is_stable("1.21.11-pre5"));
        assert!(!is_stable("26.3-rc-3"));
        assert!(!is_stable("25w14craftmine"));
    }

    #[test]
    fn platforms_catalog() {
        let ids: Vec<&str> = PLATFORMS.iter().map(|p| p.id).collect();
        assert_eq!(ids, vec!["paper", "purpur", "fabric", "forge"]);
        assert!(find("paper").is_some());
        assert!(find("Paper").is_none()); // exact match only, CLI validates
        assert!(find("vanilla").is_none());
    }

    #[test]
    fn unknown_platform_download_fails_fast() {
        let tmp = std::env::temp_dir().join(format!("crussty-test-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&tmp);
        let p = Platform { id: "vanilla", label: "Vanilla" };
        assert!(download_kernel(&p, "1.0", &tmp).is_err());
        assert!(list_versions(&p).is_err());
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
