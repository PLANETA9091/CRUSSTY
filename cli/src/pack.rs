use clap::Args;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Args)]
pub struct PackArgs {
    /// Version to stamp in the bundle (from cplugin.json if omitted).
    #[arg(long)]
    pub version: Option<String>,
    /// Output file.
    #[arg(long, default_value = None)]
    pub out: Option<String>,
}

pub fn run(args: PackArgs) -> i32 {
    let dir = std::env::current_dir().unwrap_or_default();
    let manifest_path = dir.join("cplugin.json");
    let text = match fs::read_to_string(&manifest_path) {
        Ok(t) => t,
        Err(_) => {
            eprintln!("crussty: no cplugin.json here — run 'crussty module new' first");
            return 1;
        }
    };
    let v: serde_json::Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("crussty: cplugin.json parse error: {e}");
            return 1;
        }
    };
    let id = v
        .get("id")
        .and_then(|i| i.as_str())
        .unwrap_or_else(|| dir.file_name().unwrap_or_default().to_str().unwrap_or("module"));
    let version = args
        .version
        .or_else(|| v.get("version").and_then(|x| x.as_str()).map(String::from))
        .unwrap_or_else(|| "0.1.0".to_string());
    let lib_name = v
        .get("main")
        .and_then(|m| m.as_str())
        .map(String::from)
        .unwrap_or_else(|| format!("lib{id}.so"));

    let lib_path = dir.join(&lib_name);
    if !lib_path.exists() {
        eprintln!("crussty: {} not found — run 'crussty module build' first", lib_path.display());
        return 1;
    }

    let platform = platform_tag();
    let out = args.out.unwrap_or_else(|| format!("{id}-v{version}-{platform}.tar.gz"));
    let out_path = Path::new(&out);

    let file = match fs::File::create(out_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("crussty: cannot create {}: {e}", out_path.display());
            return 1;
        }
    };
    let enc = flate2::write::GzEncoder::new(file, flate2::Compression::default());
    let mut tar = tar::Builder::new(enc);

    let root = format!("{id}/");
    for (src, dst) in [
        (manifest_path.clone(), format!("{root}cplugin.json")),
        (
            lib_path.clone(),
            format!("{root}{}", lib_path.file_name().unwrap().to_string_lossy()),
        ),
    ] {
        let mut f = match fs::File::open(&src) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("crussty: cannot read {}: {e}", src.display());
                return 1;
            }
        };
        let mut header = tar::Header::new_gnu();
        let len = f.metadata().map(|m| m.len()).unwrap_or(0);
        header.set_size(len);
        header.set_mode(0o644);
        if let Err(e) = header.set_path(dst) {
            eprintln!("crussty: bad archive path: {e}");
            return 1;
        }
        header.set_cksum();
        if tar::Builder::append(&mut tar, &mut header, &mut f).is_err() {
            eprintln!("crussty: failed to pack {}", src.display());
            return 1;
        }
    }
    for (rel, full) in extra_files(&dir, &lib_name) {
        let mut f = match fs::File::open(&full) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("crussty: cannot read {}: {e}", full.display());
                return 1;
            }
        };
        let mut header = tar::Header::new_gnu();
        let len = f.metadata().map(|m| m.len()).unwrap_or(0);
        header.set_size(len);
        header.set_mode(0o644);
        if let Err(e) = header.set_path(format!("{root}{rel}")) {
            eprintln!("crussty: bad archive path: {e}");
            return 1;
        }
        header.set_cksum();
        if tar::Builder::append(&mut tar, &mut header, &mut f).is_err() {
            eprintln!("crussty: failed to pack {}", full.display());
            return 1;
        }
    }

    let enc = tar.into_inner().unwrap();
    let _ = enc.finish();
    println!("crussty: packed {}", out_path.display());
    println!("crussty: distribute this file (or upload it to a catalog release)");
    0
}

fn extra_files(dir: &Path, lib_name: &str) -> Vec<(String, PathBuf)> {
    let skipped_exts = ["c", "cpp", "h", "cc", "go", "py", "js", "md", "toml", "lock", "sh", "gz", "zip"];
    let mut out = Vec::new();
    for entry in walk(&dir) {
        let full = entry.0.clone();
        let name = entry.0.file_name().unwrap_or_default().to_string_lossy().into_owned();
        if name == "cplugin.json" || name == lib_name {
            continue;
        }
        if let Some(ext) = entry.0.extension().and_then(|e| e.to_str()) {
            if skipped_exts.contains(&ext) {
                continue;
            }
        }
        if entry.0.is_dir()
            && matches!(name.as_str(), "src" | "target" | ".git" | "build")
        {
            continue;
        }
        out.push((entry.1, full));
    }
    out
}

fn walk(dir: &Path) -> Vec<(PathBuf, String)> {
    let mut out = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                for (sub, rel) in walk(&p) {
                    out.push((
                        sub,
                        format!(
                            "{}/{}",
                            p.file_name().unwrap_or_default().to_string_lossy(),
                            rel
                        ),
                    ));
                }
            } else {
                out.push((
                    p.clone(),
                    p.file_name().unwrap_or_default().to_string_lossy().into_owned(),
                ));
            }
        }
    }
    out
}

fn platform_tag() -> String {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    match (os, arch) {
        ("linux", "x86_64") => "linux-x64".into(),
        ("linux", "aarch64") => "linux-arm64".into(),
        ("macos", "aarch64") => "macos-arm64".into(),
        ("macos", "x86_64") => "macos-x64".into(),
        ("windows", _) => "win-x64".into(),
        (o, a) => format!("{o}-{a}"),
    }
}