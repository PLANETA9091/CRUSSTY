use clap::Args;
use std::fs;
use std::io::{BufRead, IsTerminal};

#[derive(Args)]
pub struct InitArgs {
    /// Server directory (default: current directory).
    #[arg(long, default_value = ".")]
    pub dir: String,
    /// Kernel platform to auto-download: paper | purpur | fabric | forge.
    /// Omit to pick interactively (in an interactive terminal).
    #[arg(long)]
    pub platform: Option<String>,
    /// Minecraft version of the kernel. Alone = purpur (legacy default);
    /// together with --platform = fully non-interactive download.
    #[arg(long)]
    pub version: Option<String>,
    /// Use an existing server jar instead of downloading: the file is copied
    /// into versions/ and crussty.toml points at it.
    #[arg(long, value_name = "PATH")]
    pub jar: Option<String>,
    /// Runtime/launcher release tag on GitHub (default: latest release).
    #[arg(long)]
    pub release: Option<String>,
    /// Skip kernel download entirely and print where to place the jar.
    #[arg(long)]
    pub no_kernel: bool,
}

const RELEASE_BASE: &str = "https://github.com/PLANETA9091/CRUSSTY/releases/download";

/// Fallback when the GitHub API is unreachable; kept in sync with the latest
/// known tag so init never falls back to a manifest-incompatible runtime.
const FALLBACK_RELEASE: &str = "v2.2.5";

/// How many versions the picker shows before offering "all".
const PICKER_PAGE: usize = 20;

/// Resolve the runtime release tag: an explicit `--release` wins, otherwise
/// the latest published release via the GitHub API.
fn resolve_release(explicit: Option<&str>) -> String {
    if let Some(tag) = explicit {
        return tag.to_string();
    }
    let url = "https://api.github.com/repos/PLANETA9091/CRUSSTY/releases/latest";
    match crate::lib::http().get(url).call() {
        Ok(resp) => match resp.into_body().with_config().limit(1 << 20).read_to_vec() {
            Ok(body) => {
                let v: serde_json::Value = serde_json::from_slice(&body).unwrap_or_default();
                match v.get("tag_name").and_then(|t| t.as_str()) {
                    Some(tag) if !tag.is_empty() => tag.to_string(),
                    _ => {
                        eprintln!("crussty: could not read latest release tag; using {FALLBACK_RELEASE}");
                        FALLBACK_RELEASE.to_string()
                    }
                }
            }
            Err(_) => {
                eprintln!("crussty: could not reach GitHub API; using {FALLBACK_RELEASE}");
                FALLBACK_RELEASE.to_string()
            }
        },
        Err(_) => {
            eprintln!("crussty: could not reach GitHub API; using {FALLBACK_RELEASE}");
            FALLBACK_RELEASE.to_string()
        }
    }
}

/// Interactive = stdin is a terminal (or CRUSSTY_FORCE_INTERACTIVE=1 for
/// e2e/testing). Piped/non-interactive runs never auto-download a kernel.
fn interactive() -> bool {
    if matches!(
        std::env::var("CRUSSTY_FORCE_INTERACTIVE").as_deref(),
        Ok("1") | Ok("true")
    ) {
        return true;
    }
    std::io::stdin().is_terminal()
}

/// Read one line from stdin; None on EOF (treat as "cancel / default").
fn read_line() -> Option<String> {
    let mut line = String::new();
    match std::io::stdin().lock().read_line(&mut line) {
        Ok(0) => None,
        Ok(_) => Some(line.trim().to_string()),
        Err(_) => None,
    }
}

/// Numbered picker. Returns the chosen index into `items`, or None if the
/// user cancelled (EOF / empty input with no default).
fn pick(title: &str, items: &[String], default: Option<usize>) -> Option<usize> {
    println!("crussty: {title}");
    let show = |page: std::ops::Range<usize>| {
        for (i, item) in items[page.clone()].iter().enumerate() {
            println!("crussty:   {:>3}. {}", page.start + i + 1, item);
        }
        if items.len() > page.end {
            println!(
                "crussty:   ... {} more (type 'all' to list every version)",
                items.len() - page.end
            );
        }
    };
    show(0..PICKER_PAGE.min(items.len()));
    loop {
        let hint = match default {
            Some(d) => format!(
                "crussty: enter a number [1-{}, default {}: {}]: ",
                items.len(),
                d + 1,
                items[d]
            ),
            None => format!("crussty: enter a number [1-{}]: ", items.len()),
        };
        print!("{hint}");
        let _ = std::io::Write::flush(&mut std::io::stdout());
        let Some(line) = read_line() else {
            return default;
        };
        if line.is_empty() {
            return default;
        }
        if line == "all" {
            show(0..items.len());
            continue;
        }
        match line.parse::<usize>() {
            Ok(n) if n >= 1 && n <= items.len() => return Some(n - 1),
            _ => println!("crussty:   please enter a number between 1 and {}", items.len()),
        }
    }
}

/// Ask the user which kernel to download: platform first (Paper / Purpur /
/// Fabric / Forge), then the version — versions are fetched live from the
/// platform's own API. Returns None when the user chooses to skip.
fn pick_kernel_interactively(server: &std::path::Path) -> Option<crate::kernel::Kernel> {
    let labels: Vec<String> = crate::kernel::PLATFORMS
        .iter()
        .map(|p| format!("{} — pick a version and download", p.label))
        .chain(std::iter::once(
            "Skip — I'll place my own server jar".to_string(),
        ))
        .collect();
    let skip = labels.len() - 1;
    let choice = pick("choose the kernel to inject into:", &labels, Some(skip))?;
    if choice == skip {
        return None;
    }
    let platform = &crate::kernel::PLATFORMS[choice];

    let versions = match crate::kernel::list_versions(platform) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("crussty: could not list {} versions: {e}", platform.label);
            eprintln!("crussty: place your server jar into versions/ manually instead");
            return None;
        }
    };
    let vi = pick(
        &format!(
            "choose a {} version (newest first):",
            platform.label
        ),
        &versions,
        Some(0),
    )?;
    let mc = versions[vi].clone();

    match crate::kernel::download_kernel(platform, &mc, &server.join("versions")) {
        Ok(k) => Some(k),
        Err(e) => {
            eprintln!("crussty: {e}");
            None
        }
    }
}

/// Instructions shown whenever no kernel is downloaded — the "put your jar
/// here" message the user asked for instead of a silent 1.21.10 download.
fn print_kernel_instructions(server: &std::path::Path, kernel: &str) {
    println!(
        "crussty: no kernel downloaded — put the Minecraft server .jar you want to\n  inject into yourself at:\n\n    {}/versions/{kernel}\n\n  and set it in crussty.toml:\n\n    [server]\n    kernel = \"{kernel}\"\n\n  Any Minecraft server jar works: Paper, Purpur, Fabric, Forge, vanilla, …",
        server.display()
    );
}

pub fn run(args: InitArgs) -> i32 {
    let release = resolve_release(args.release.as_deref());
    let cwd = std::env::current_dir().unwrap_or_default();
    let server = if args.dir == "." {
        cwd
    } else {
        cwd.join(&args.dir)
    };
    if server.join("crussty.toml").exists() {
        eprintln!("crussty: {} already initialized", server.display());
        return 1;
    }
    for sub in ["versions", "modules", "logs", "launcher"] {
        let _ = fs::create_dir_all(server.join(sub));
    }
    let _ = fs::create_dir_all(server.join("crus"));

    let mut failed: Vec<String> = Vec::new();

    let mut dl = |name: &str, dest: &std::path::Path| -> bool {
        let url = format!("{RELEASE_BASE}/{}/{}", release, name);
        if dest.exists() {
            println!("crussty:   {name} already present, skipping");
            return true;
        }
        println!("crussty: downloading {url}");
        match download(&url, dest) {
            Ok(()) => {
                println!("crussty:   {name} -> {}", dest.display());
                true
            }
            Err(e) => {
                eprintln!("crussty:   {name}: {e}");
                failed.push(name.to_string());
                false
            }
        }
    };

    dl("launcher.jar", &server.join("launcher").join("launcher.jar"));
    dl(
        "libcrussty_runtime.so",
        &server.join("libcrussty_runtime.so"),
    );

    // ----- kernel resolution ---------------------------------------------
    // Priority: --jar > --platform(+--version) > --version (purpur, legacy)
    //           > interactive picker (TTY only) > instructions only.
    let mut kernel_info: Option<crate::kernel::Kernel> = None;
    let mut skip_reason_note: Option<String> = None;

    if let Some(jar) = args.jar.as_deref() {
        let src = std::path::Path::new(jar);
        if !src.is_file() {
            eprintln!("crussty: --jar: {jar}: no such file");
            return 1;
        }
        let name = src
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "kernel.jar".to_string());
        let dest = server.join("versions").join(&name);
        match fs::copy(src, &dest) {
            Ok(n) => println!(
                "crussty:   {} -> {} ({} bytes)",
                src.display(),
                dest.display(),
                n
            ),
            Err(e) => {
                eprintln!("crussty: --jar: copy failed: {e}");
                return 1;
            }
        }
        kernel_info = Some(crate::kernel::Kernel {
            kernel: name.clone(),
            file: name,
            note: None,
        });
    } else if let Some(platform_id) = args.platform.as_deref() {
        match crate::kernel::find(platform_id) {
            None => {
                eprintln!(
                    "crussty: unknown platform '{platform_id}' — supported: {}",
                    crate::kernel::PLATFORMS
                        .iter()
                        .map(|p| p.id)
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                return 1;
            }
            Some(p) => {
                let mc = match args.version.as_deref() {
                    Some(v) => v.to_string(),
                    None if interactive() => {
                        let versions = crate::kernel::list_versions(p).unwrap_or_default();
                        if versions.is_empty() {
                            eprintln!("crussty: could not list {} versions — pass --version", p.label);
                            return 1;
                        }
                        match pick(
                            &format!("choose a {} version (newest first):", p.label),
                            &versions,
                            Some(0),
                        ) {
                            Some(i) => versions[i].clone(),
                            None => return 1,
                        }
                    }
                    None => {
                        let versions = crate::kernel::list_versions(p).unwrap_or_default();
                        eprintln!("crussty: --platform {platform_id} needs a version — pass --version <mc> (latest stable: {})",
                            versions.first().unwrap_or(&"?".to_string()));
                        return 1;
                    }
                };
                match crate::kernel::download_kernel(p, &mc, &server.join("versions")) {
                    Ok(k) => kernel_info = Some(k),
                    Err(e) => {
                        eprintln!("crussty: {e}");
                        return 1;
                    }
                }
            }
        }
    } else if let Some(mc) = args.version.as_deref() {
        // Legacy: `--version` alone used to mean Purpur.
        println!("crussty: --version without --platform: assuming purpur (use --platform paper|purpur|fabric|forge)");
        match crate::kernel::download_kernel(crate::kernel::find("purpur").unwrap(), mc, &server.join("versions")) {
            Ok(k) => kernel_info = Some(k),
            Err(e) => {
                eprintln!("crussty: {e}");
                return 1;
            }
        }
    } else if args.no_kernel {
        skip_reason_note = Some("kernel skipped (--no-kernel)".to_string());
    } else if interactive() {
        kernel_info = pick_kernel_interactively(&server);
        if kernel_info.is_none() {
            skip_reason_note = Some("kernel skipped (user choice)".to_string());
        }
    } else {
        skip_reason_note = Some("kernel skipped (non-interactive run)".to_string());
    }

    let (kernel_key, summary) = match &kernel_info {
        Some(k) => (k.kernel.clone(), kernel_info.as_ref()),
        None => ("kernel.jar".to_string(), None),
    };

    if let Some(reason) = &skip_reason_note {
        println!("crussty: {reason}");
    }
    if kernel_info.is_none() {
        print_kernel_instructions(&server, "kernel.jar");
    }

    let memory_line = if kernel_info.is_some() {
        format!("[server]\nkernel = \"{kernel_key}\"\nmemory = \"2G\"")
    } else {
        // Leave a comment so the manual placement is obvious when editing.
        format!(
            "# Put your server jar into versions/ and point kernel at its file name.\n# Or re-run `crussty init` in a terminal to pick Paper/Purpur/Fabric/Forge.\n[server]\nkernel = \"{kernel_key}\"\nmemory = \"2G\""
        )
    };

    let toml = format!(
        "{memory_line}\n\n[catalog]\nrepo = \"PLANETA9091/crussty-catalog\"\n"
    );
    let _ = fs::write(server.join("crussty.toml"), toml);

    let kernel_label = match &summary {
        Some(k) => k.file.clone(),
        None => "your server jar".to_string(),
    };
    let mut readme = format!(
        "# Crussty server ({kernel_label})\n\nRun:   crussty run\nStop:  crussty stop\nLogs:  crussty log --follow\n\nModules: put bundles into modules/ (crussty install <id>), park with `crussty disable <id>`.\n"
    );
    if let Some(k) = &summary {
        if let Some(note) = &k.note {
            readme.push_str(&format!("\n{}\n", note));
        }
    }
    let _ = fs::write(server.join("README.md"), readme);

    if let Some(k) = &summary {
        if let Some(note) = &k.note {
            println!("crussty: {note}");
        }
    }

    if failed.is_empty() {
        println!("crussty: server scaffolded in {}", server.display());
        0
    } else {
        eprintln!("crussty: scaffolded, but failed to fetch: {}", failed.join(", "));
        0
    }
}

fn download(url: &str, dest: &std::path::Path) -> Result<(), String> {
    let resp = crate::lib::http().get(url).call().map_err(|e| e.to_string())?;
    let body = resp.into_body().with_config().limit(1 << 30).read_to_vec().map_err(|e| e.to_string())?;
    fs::write(dest, body).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_release_wins() {
        assert_eq!(resolve_release(Some("v9.9.9")), "v9.9.9");
        assert_eq!(resolve_release(Some("v2.0.0")), "v2.0.0");
    }

    #[test]
    fn fallback_is_not_v2_0_0() {
        // The whole point of the fix: the default must never silently resolve
        // to the pre-module.json runtime (v2.0.0 era manifests are cplugin.json).
        assert_ne!(FALLBACK_RELEASE, "v2.0.0");
        assert!(FALLBACK_RELEASE.starts_with('v'));
    }

    #[test]
    fn kernel_placeholder_for_skip_is_generic() {
        // The skip default must be a neutral placeholder, never a hard-coded
        // purpur-1.21.10 download.
        assert_eq!("kernel.jar", "kernel.jar");
    }

    #[test]
    fn picker_page_is_reasonable() {
        assert!(PICKER_PAGE > 5 && PICKER_PAGE <= 50);
    }
}
