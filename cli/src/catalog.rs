use crate::lib;
use std::fs;
use std::io::Read;

/// Catalog entry as published in catalog.json of the catalog repo.
#[derive(serde::Deserialize)]
struct CatalogEntry {
    id: String,
    version: String,
    /// URL of the .tar.gz bundle (release asset).
    url: String,
    /// Optional platform filter ("linux-x64" etc.).
    #[serde(default)]
    platform: Option<String>,
}

pub fn install(module: &str, catalog_repo: Option<&str>) -> i32 {
    let server = lib::server_dir();
    let repo = catalog_repo.unwrap_or("PLANETA9091/crussty-catalog");
    let url = format!("https://raw.githubusercontent.com/{repo}/main/catalog.json");
    let text = match crate::lib::http().get(&url).call() {
        Ok(resp) => match resp.into_body().read_to_string() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("crussty: cannot read catalog response: {e}");
                return 1;
            }
        },
        Err(e) => {
            eprintln!("crussty: catalog not reachable ({url}): {e}");
            eprintln!("crussty: hint: create the catalog repo, or pass --catalog owner/repo");
            return 1;
        }
    };
    let entries: Vec<CatalogEntry> = match serde_json::from_str(&text) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("crussty: catalog.json parse error: {e}");
            return 1;
        }
    };
    let entry = entries.iter().find(|e| e.id == module);
    let Some(entry) = entry else {
        eprintln!("crussty: module '{module}' not found in catalog ({repo})");
        eprintln!("crussty: available:");
        for e in &entries {
            eprintln!("  {}", e.id);
        }
        return 1;
    };
    let platform = platform_tag();
    if let Some(p) = &entry.platform {
        if p != &platform {
            eprintln!(
                "crussty: {module} is built for {p}, this host is {platform}"
            );
            return 1;
        }
    }

    let dest_dir = lib::server_modules_dir(&server).join(module);
    if dest_dir.exists() {
        eprintln!("crussty: {} already installed (disable/remove it first)", dest_dir.display());
        return 1;
    }
    let _ = fs::create_dir_all(&dest_dir);

    println!("crussty: fetching {} v{}", entry.id, entry.version);
    println!("crussty:   {entry_url}", entry_url = entry.url);
    let resp = match crate::lib::http().get(&entry.url)

        .call()
    {
        Ok(r) => r,
        Err(e) => {
            let _ = fs::remove_dir_all(&dest_dir);
            eprintln!("crussty: download failed: {e}");
            return 1;
        }
    };
    let body = match resp.into_body().with_config().limit(1 << 30).read_to_vec() {
        Ok(b) => b,
        Err(e) => {
            let _ = fs::remove_dir_all(&dest_dir);
            eprintln!("crussty: download failed: {e}");
            return 1;
        }
    };

    match unpack_tar_gz(&body, &dest_dir) {
        Ok(()) => {
            println!("crussty: installed {} v{} -> {}", entry.id, entry.version, dest_dir.display());
            0
        }
        Err(e) => {
            let _ = fs::remove_dir_all(&dest_dir);
            eprintln!("crussty: unpack failed: {e}");
            1
        }
    }
}

fn unpack_tar_gz(body: &[u8], dest: &std::path::Path) -> Result<(), String> {
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
            let _ = fs::create_dir_all(parent);
        }
        let mut data = Vec::new();
        entry.read_to_end(&mut data).map_err(|e| e.to_string())?;
        fs::write(&full, data).map_err(|e| e.to_string())?;
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