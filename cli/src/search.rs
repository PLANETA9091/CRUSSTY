use crate::lib;
use serde_json::Value;

pub fn search(query: &str) -> i32 {
    let token = std::env::var("GITHUB_TOKEN")
        .or_else(|_| std::env::var("GH_TOKEN"))
        .ok();
    let hits: Vec<Value> = if let Some(token) = &token {
        match code_search(query, token) {
            Some(v) => v,
            None => {
                eprintln!("crussty: code search failed, falling back to repo search");
                repo_search(query)
            }
        }
    } else {
        repo_search(query)
    };
    if hits.is_empty() {
        println!("crussty: no modules found for '{query}'");
        return 0;
    }
    let mut seen: Vec<String> = Vec::new();
    for item in &hits {
        let full = item.get("full_name").and_then(|x| x.as_str()).unwrap_or("?").to_string();
        if seen.contains(&full) {
            continue;
        }
        seen.push(full.clone());
        let stars = item.get("stargazers_count").and_then(|x| x.as_u64()).unwrap_or(0);
        let desc = item
            .get("description")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .chars()
            .take(80)
            .collect::<String>();
        let (owner, repo) = split_repo(&full);
        match fetch_manifest(owner, repo) {
            Some(m) => {
                let id = m.get("id").and_then(|x| x.as_str()).unwrap_or("");
                let ver = m.get("version").and_then(|x| x.as_str()).unwrap_or("?");
                println!(
                    "  {:<28} v{:<7} \u{2605} {:<4} {} ({})",
                    full, ver, stars, desc, id
                );
            }
            None => {
                println!(
                    "  {:<28} \u{2605} {:<4} {} (no cplugin.json)",
                    full, stars, desc
                );
            }
        }
    }
    println!();
    println!("install one: crussty install <owner/repo>");
    0
}

/// GitHub code search over cplugin.json contents — needs a token.
fn code_search(query: &str, token: &str) -> Option<Vec<Value>> {
    let q = format!("{} filename:cplugin.json", urlencode(query));
    let url = format!(
        "https://api.github.com/search/code?q={q}&per_page=10"
    );
    let resp = lib::http()
        .get(&url)
        .header("User-Agent", "crussty-cli")
        .header("Accept", "application/vnd.github+json")
        .header("Authorization", format!("Bearer {token}"))
        .call()
        .ok()?;
    let body = resp.into_body().read_to_string().ok()?;
    let v: Value = serde_json::from_str(&body).ok()?;
    let items = v.get("items").and_then(|i| i.as_array())?;
    let mut out = Vec::new();
    for item in items {
        let full = item
            .get("repository")
            .and_then(|r| r.get("full_name"))
            .and_then(|x| x.as_str())
            .unwrap_or("?");
        let mut repo = serde_json::Map::new();
        repo.insert("full_name".into(), Value::String(full.into()));
        repo.insert("description".into(), Value::String(String::new()));
        repo.insert("stargazers_count".into(), Value::from(0));
        out.push(Value::Object(repo));
    }
    Some(out)
}

fn repo_search(query: &str) -> Vec<Value> {
    let q = format!("crussty {query}");
    let url = format!(
        "https://api.github.com/search/repositories?q={}&sort=stars&per_page=10",
        urlencode(&q)
    );
    let resp = match lib::http()
        .get(&url)
        .header("User-Agent", "crussty-cli")
        .header("Accept", "application/vnd.github+json")
        .call()
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("crussty: GitHub search failed: {e}");
            eprintln!("crussty: hint: unauthenticated search is rate-limited (~10/min)");
            return Vec::new();
        }
    };
    let body = match resp.into_body().read_to_string() {
        Ok(b) => b,
        Err(e) => {
            eprintln!("crussty: cannot read search response: {e}");
            return Vec::new();
        }
    };
    let v: Value = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("crussty: search response parse error: {e}");
            return Vec::new();
        }
    };
    v.get("items")
        .and_then(|i| i.as_array())
        .cloned()
        .unwrap_or_default()
}

/// Install straight from a GitHub repo: reads cplugin.json (main/master),
/// then downloads a platform bundle from the repo's releases if present.
pub fn install_repo(repo: &str) -> i32 {
    let (owner, name) = split_repo(repo);
    if owner.is_empty() || name.is_empty() {
        eprintln!("crussty: expected <owner/repo>, got '{repo}'");
        return 1;
    }
    let manifest = match fetch_manifest(owner, name) {
        Some(m) => m,
        None => {
            eprintln!("crussty: no cplugin.json in {owner}/{name}");
            return 1;
        }
    };
    let id = manifest.get("id").and_then(|x| x.as_str()).unwrap_or(name);
    let version = manifest
        .get("version")
        .and_then(|x| x.as_str())
        .unwrap_or("?");
    println!("crussty: found module '{id}' v{version} in {owner}/{name}");

    let platform = platform_tag();
    let bundle = find_bundle(owner, name, id, &platform);
    let Some(bundle_url) = bundle else {
        eprintln!("crussty: no {id}-*.tar.gz bundle for {platform} in releases");
        eprintln!("crussty: hint: build from source (git clone {owner}/{name}) and use 'crussty module pack'");
        return 1;
    };

    let server = lib::server_dir();
    let dest_dir = lib::server_modules_dir(&server).join(id);
    if dest_dir.exists() {
        eprintln!("crussty: {} already installed", dest_dir.display());
        return 1;
    }
    let _ = std::fs::create_dir_all(&dest_dir);

    println!("crussty: downloading {bundle_url}");
    let resp = match lib::http().get(&bundle_url).call() {
        Ok(r) => r,
        Err(e) => {
            let _ = std::fs::remove_dir_all(&dest_dir);
            eprintln!("crussty: download failed: {e}");
            return 1;
        }
    };
    let body = match resp.into_body().read_to_vec() {
        Ok(b) => b,
        Err(e) => {
            let _ = std::fs::remove_dir_all(&dest_dir);
            eprintln!("crussty: download failed: {e}");
            return 1;
        }
    };
    match unpack(&body, &dest_dir) {
        Ok(()) => {
            println!("crussty: installed {} v{} -> {}", id, version, dest_dir.display());
            0
        }
        Err(e) => {
            let _ = std::fs::remove_dir_all(&dest_dir);
            eprintln!("crussty: unpack failed: {e}");
            1
        }
    }
}

fn find_bundle(owner: &str, repo: &str, id: &str, platform: &str) -> Option<String> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}/releases?per_page=5");
    let resp = lib::http()
        .get(&url)
        .header("User-Agent", "crussty-cli")
        .header("Accept", "application/vnd.github+json")
        .call()
        .ok()?;
    let body = resp.into_body().read_to_string().ok()?;
    let releases: Value = serde_json::from_str(&body).ok()?;
    let assets = releases
        .as_array()?
        .iter()
        .flat_map(|r| r.get("assets").and_then(|a| a.as_array()).into_iter().flatten());
    for asset in assets {
        let name = asset.get("name").and_then(|n| n.as_str()).unwrap_or("");
        if name.starts_with(&format!("{id}-")) && name.ends_with(".tar.gz") && name.contains(platform) {
            return asset
                .get("browser_download_url")
                .and_then(|u| u.as_str())
                .map(String::from);
        }
    }
    None
}

fn fetch_manifest(owner: &str, repo: &str) -> Option<Value> {
    for branch in ["main", "master"] {
        let url = format!("https://raw.githubusercontent.com/{owner}/{repo}/{branch}/cplugin.json");
        if let Ok(resp) = lib::http().get(&url).call() {
            if let Ok(text) = resp.into_body().read_to_string() {
                if let Ok(v) = serde_json::from_str(&text) {
                    return Some(v);
                }
            }
        }
    }
    fetch_monorepo_manifest(owner, repo)
}

/// Monorepo layout: cplugin.json under modules/<name>/.
fn fetch_monorepo_manifest(owner: &str, repo: &str) -> Option<Value> {
    for branch in ["main", "master"] {
        let url = format!("https://api.github.com/repos/{owner}/{repo}/contents/modules?ref={branch}");
        let resp = match lib::http()
            .get(&url)
            .header("User-Agent", "crussty-cli")
            .header("Accept", "application/vnd.github+json")
            .call()
        {
            Ok(r) => r,
            Err(_) => continue,
        };
        let Ok(body) = resp.into_body().read_to_string() else {
            continue;
        };
        let Ok(v) = serde_json::from_str::<Value>(&body) else {
            continue;
        };
        let Some(arr) = v.as_array() else {
            continue;
        };
        let dirs = arr
            .iter()
            .filter(|e| e.get("type").and_then(|t| t.as_str()) == Some("dir"))
            .filter_map(|e| e.get("name").and_then(|n| n.as_str()).map(String::from));
        for dir in dirs {
            let url = format!(
                "https://raw.githubusercontent.com/{owner}/{repo}/{branch}/modules/{dir}/cplugin.json"
            );
            if let Ok(resp) = lib::http().get(&url).call() {
                if let Ok(text) = resp.into_body().read_to_string() {
                    if let Ok(m) = serde_json::from_str(&text) {
                        return Some(m);
                    }
                }
            }
        }
    }
    None
}

fn split_repo(repo: &str) -> (&str, &str) {
    let trimmed = repo.trim().trim_start_matches("https://github.com/").trim_end_matches('/');
    match trimmed.split_once('/') {
        Some((o, r)) => {
            let r = r.split('/').next().unwrap_or(r);
            (o, r)
        }
        None => (trimmed, ""),
    }
}

fn unpack(body: &[u8], dest: &std::path::Path) -> Result<(), String> {
    use std::io::Read;
    let gz = flate2::read::GzDecoder::new(body);
    let mut archive = tar::Archive::new(gz);
    for entry in archive.entries().map_err(|e| e.to_string())? {
        let mut entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path().map_err(|e| e.to_string())?.into_owned();
        let name = path
            .components()
            .skip(1)
            .fold(std::path::PathBuf::new(), |a, c| a.join(c.as_os_str()));
        if name.as_os_str().is_empty() {
            continue;
        }
        let full = dest.join(&name);
        if let Some(parent) = full.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let mut data = Vec::new();
        entry.read_to_end(&mut data).map_err(|e| e.to_string())?;
        std::fs::write(&full, data).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn platform_tag() -> String {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => "linux-x64".into(),
        ("linux", "aarch64") => "linux-arm64".into(),
        ("macos", "aarch64") => "macos-arm64".into(),
        ("macos", "x86_64") => "macos-x64".into(),
        ("windows", _) => "win-x64".into(),
        (o, a) => format!("{o}-{a}"),
    }
}

fn urlencode(s: &str) -> String {
    s.chars()
        .flat_map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                vec![c]
            }
            ' ' => vec!['+'],
            _ => {
                let mut b = [0u8; 4];
                let bytes = c.encode_utf8(&mut b).as_bytes();
                bytes
                    .iter()
                    .map(|&x| format!("%{:02X}", x))
                    .collect::<String>()
                    .chars()
                    .collect()
            }
        })
        .collect()
}