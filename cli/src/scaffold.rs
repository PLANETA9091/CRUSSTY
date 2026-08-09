use clap::ValueEnum;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[derive(clap::Args)]
pub struct NewArgs {
    /// Module id/directory name (lowercase, [a-z0-9_-]).
    pub name: String,
    /// Language template.
    #[arg(long, value_enum, default_value_t = LangTemplate::Rust)]
    pub template: LangTemplate,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum LangTemplate {
    C,
    Cpp,
    Go,
    Js,
    Python,
    Rust,
}

const MANIFEST: &str = include_str!("../templates/cplugin.json.tpl");

struct Tpl<'a> {
    name: LangTemplate,
    files: &'a [(&'a str, &'a str)],
}

const C: &[(&str, &str)] = &[
    ("module.c", include_str!("../templates/c/module.c")),
    ("build.sh", include_str!("../templates/c/build.sh")),
];
const CPP: &[(&str, &str)] = &[
    ("module.cpp", include_str!("../templates/cpp/module.cpp")),
    ("build.sh", include_str!("../templates/cpp/build.sh")),
];
const GO: &[(&str, &str)] = &[
    ("main.go", include_str!("../templates/go/main.go")),
    ("go.mod", include_str!("../templates/go/go.mod")),
    ("build.sh", include_str!("../templates/go/build.sh")),
];
const JS: &[(&str, &str)] = &[
    ("shim.c", include_str!("../templates/js/shim.c")),
    ("plugin.js", include_str!("../templates/js/plugin.js")),
    ("build.sh", include_str!("../templates/js/build.sh")),
];
const PY: &[(&str, &str)] = &[
    ("shim.c", include_str!("../templates/python/shim.c")),
    ("plugin.py", include_str!("../templates/python/plugin.py")),
    ("build.sh", include_str!("../templates/python/build.sh")),
];
const RUST: &[(&str, &str)] = &[
    ("Cargo.toml", include_str!("../templates/rust/Cargo.toml")),
    ("src/lib.rs", include_str!("../templates/rust/src/lib.rs")),
];

const ALL: &[Tpl<'static>] = &[
    Tpl { name: LangTemplate::C, files: C },
    Tpl { name: LangTemplate::Cpp, files: CPP },
    Tpl { name: LangTemplate::Go, files: GO },
    Tpl { name: LangTemplate::Js, files: JS },
    Tpl { name: LangTemplate::Python, files: PY },
    Tpl { name: LangTemplate::Rust, files: RUST },
];

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

pub fn run_new(args: NewArgs) -> i32 {
    if !valid_name(&args.name) {
        eprintln!("crussty: module name must match [a-z0-9_-]+");
        return 2;
    }
    let dir = std::env::current_dir().unwrap_or_default().join(&args.name);
    if dir.exists() {
        eprintln!("crussty: {} already exists", dir.display());
        return 1;
    }
    let Some(tpl) = ALL
        .iter()
        .find(|t| std::mem::discriminant(&t.name) == std::mem::discriminant(&args.template))
    else {
        unreachable!();
    };
    let manifest = MANIFEST.replace("__NAME__", &args.name);
    let _ = fs::create_dir(dir.join("src"));
    let mut files: Vec<(String, String)> = vec![("cplugin.json".into(), manifest)];
    for (path, content) in tpl.files {
        files.push((path.to_string(), content.replace("__NAME__", &args.name)));
    }
    for (path, content) in files {
        let full = dir.join(&path);
        if let Some(parent) = full.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Err(e) = fs::write(&full, content) {
            eprintln!("crussty: cannot write {}: {e}", full.display());
            return 1;
        }
    }
    for (path, _) in tpl.files {
        if path.ends_with(".sh") {
            #[cfg(unix)]
            let _ = fs::set_permissions(
                dir.join(path),
                fs::Permissions::from_mode(0o755),
            );
        }
    }
    println!("created module {} in {}", args.name, dir.display());
    println!("  cd {} && crussty module build", args.name);
    0
}

pub fn run_build() -> i32 {
    let script = std::path::Path::new("build.sh");
    if !script.exists() {
        eprintln!("crussty: no build.sh in this directory (run 'crussty module new')");
        return 1;
    }
    let status = std::process::Command::new("bash")
        .arg(script)
        .status();
    match status {
        Ok(s) if s.success() => 0,
        Ok(_) => 1,
        Err(e) => {
            eprintln!("crussty: failed to run build.sh: {e}");
            1
        }
    }
}

pub fn run_watch() -> i32 {
    let dir = std::env::current_dir().unwrap_or_default();
    println!("crussty: watching {} for changes (Ctrl-C to stop)", dir.display());
    let exts = ["rs", "c", "cpp", "h", "cc", "js", "py", "go"];
    let mut last = snapshot(&dir, &exts);
    loop {
        std::thread::sleep(std::time::Duration::from_millis(500));
        let now = snapshot(&dir, &exts);
        if now != last {
            last = now;
            println!("crussty: change detected, rebuilding");
            let rc = run_build();
            if rc == 0 {
                println!("crussty: reloading modules on live server");
                crate::server::reload();
            }
        }
    }
}

fn snapshot(root: &std::path::Path, exts: &[&str]) -> Vec<(String, u64)> {
    let mut out = Vec::new();
    for entry in walkdir(root, exts) {
        let meta = match fs::metadata(&entry) {
            Ok(m) => m,
            Err(_) => continue,
        };
        out.push((entry.display().to_string(), meta.modified().ok().map(|t| {
            t.duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0)
        }).unwrap_or(0)));
    }
    out
}

fn walkdir(dir: &std::path::Path, exts: &[&str]) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                out.extend(walkdir(&p, exts));
            } else if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                if exts.contains(&ext) {
                    out.push(p);
                }
            }
        }
    }
    out
}