use clap::Args;
use std::fs;

#[derive(Args)]
pub struct InitArgs {
    /// Server directory (default: current directory).
    #[arg(long, default_value = ".")]
    pub dir: String,
    /// Kernel jar to download (Purpur version).
    #[arg(long, default_value = "1.21.10")]
    pub version: String,
    /// Runtime/launcher release tag on GitHub (default: latest release).
    #[arg(long)]
    pub release: Option<String>,
    /// Skip downloading the kernel jar.
    #[arg(long)]
    pub no_kernel: bool,
}

const RELEASE_BASE: &str = "https://github.com/PLANETA9091/CRUSSTY/releases/download";

/// Fallback when the GitHub API is unreachable; kept in sync with the latest
/// known tag so init never falls back to a manifest-incompatible runtime.
const FALLBACK_RELEASE: &str = "v2.2.5";

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

    let kernel = format!("purpur-{}.jar", args.version);
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

    if !args.no_kernel {
        let kernel_url = format!(
            "https://api.purpurmc.org/v2/purpur/{}/latest/download",
            args.version
        );
        let dest = server.join("versions").join(&kernel);
        if !dest.exists() {
            println!("crussty: downloading kernel {kernel_url}");
            match download(&kernel_url, &dest) {
                Ok(()) => println!("crussty:   {} -> {}", kernel, dest.display()),
                Err(e) => {
                    eprintln!("crussty:   kernel: {e}");
                    failed.push(kernel.clone());
                }
            }
        }
    } else if !server.join("versions").join(&kernel).exists() {
        println!("crussty: kernel skipped (--no-kernel); put purpur-{}.jar into versions/", args.version);
    }

    let toml = format!(
        "[server]\nkernel = \"{kernel}\"\nmemory = \"2G\"\n\n[catalog]\nrepo = \"PLANETA9091/crussty-catalog\"\n"
    );
    let _ = fs::write(server.join("crussty.toml"), toml);

    let readme = format!(
        "# Crussty server ({})\n\nRun:   crussty run\nStop:  crussty stop\nLogs:  crussty log --follow\n\nModules: put bundles into modules/ (crussty install <id>), park with `crussty disable <id>`.\n",
        args.version
    );
    let _ = fs::write(server.join("README.md"), readme);

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
}