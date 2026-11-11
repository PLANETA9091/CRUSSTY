use crate::lib;


pub fn list() -> i32 {
    let server = lib::server_dir();
    let dir = lib::server_modules_dir(&server);
    let Ok(entries) = std::fs::read_dir(&dir) else {
        eprintln!("crussty: no modules dir — run 'crussty init' first");
        return 1;
    };
    let mut names: Vec<String> = entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n != ".")
        .collect();
    names.sort();

    let active = names.iter().filter(|n| !n.ends_with(".disabled") && !n.ends_with(".x")).count();
    let parked = names.iter().filter(|n| n.ends_with(".x")).count();
    let disabled = names.iter().filter(|n| n.ends_with(".disabled")).count();
    println!("modules in {}", dir.display());
    println!("{} active · {} parked · {} disabled\n", active, parked, disabled);

    for name in names {
        let status = if name.ends_with(".disabled") {
            "disabled"
        } else if name.ends_with(".x") {
            "parked"
        } else {
            "active "
        };
        let entry = lib::entry_for(&server, &name);
        println!("  {:<24} {}  {}", name, status, entry.display());
    }
    0
}

pub fn set_enabled(module: &str, enable: bool) -> i32 {
    set_status(module, if enable { Status::Active } else { Status::Disabled })
}

pub fn park(module: &str) -> i32 {
    set_status(module, Status::Parked)
}

enum Status {
    Active,
    Parked,
    Disabled,
}

fn set_status(module: &str, want: Status) -> i32 {
    let server = lib::server_dir();
    let dir = lib::server_modules_dir(&server);
    let candidates: [std::path::PathBuf; 3] = [
        dir.join(module),
        dir.join(format!("{module}.x")),
        dir.join(format!("{module}.disabled")),
    ];
    let src = candidates.iter().find(|c| c.exists());
    let Some(src) = src else {
        eprintln!("crussty: no module '{}'", module);
        return 1;
    };
    let src = src.clone();
    if src.is_file() {
        eprintln!("crussty: '{}' is not a module directory", src.display());
        return 1;
    }
    let dst: std::path::PathBuf = match want {
        Status::Active => dir.join(module),
        Status::Parked => dir.join(format!("{module}.x")),
        Status::Disabled => dir.join(format!("{module}.disabled")),
    };
    if dst == src {
        println!("crussty: {} is already in that desired state", dst.display());
        return 0;
    }
    if dst.exists() {
        eprintln!("crussty: target {} already exists", dst.display());
        return 1;
    }
    match std::fs::rename(&src, &dst) {
        Ok(_) => {
            println!("crussty: {} -> {}", src.display(), dst.display());
            0
        }
        Err(e) => {
            eprintln!("crussty: cannot rename: {e}");
            1
        }
    }
}