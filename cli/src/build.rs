//! `crussty build` — pack the whole server into ONE bootable jar.
//!
//! Until now running a Crussty server meant two artifacts: the crussty
//! launcher (`launcher/launcher.jar`, which spawns the kernel with the
//! native JVMTI runtime) and the kernel jar itself. `build` merges
//! everything into a single `java -jar` file:
//!
//! ```text
//!   kernel jar (unmodified, every entry copied)      ← the injection target
//! + dev/dist/launcher/Boot.class                     ← single-jar entry point
//! + libcrussty_runtime.so                            ← the native runtime
//! + modules/**                                       ← every installed module
//! ```
//!
//! Boot (launcher/src/main/java/dev/dist/launcher/Boot.java) extracts the
//! runtime + modules into ./crussty/ on first boot, System.load()s the
//! runtime (JNI_OnLoad brings up the JVMTI pipeline before any kernel class
//! loads) and then invokes the kernel's main class. The kernel's original
//! Main-Class is preserved in the merged manifest under `Crussty-Kernel-Main`,
//! so ANY bootable server jar works — Paper, Purpur, Fabric, vanilla — not
//! just paperclip kernels (the manifest attribute is Boot's only contract;
//! it falls back to io.papermc.paperclip.Main for hand-made jars).
//!
//! The Boot.class artifact is committed (see scripts/build-helpers.sh) and
//! embedded via include_bytes!, so building needs no Java toolchain on the
//! user's machine. Forge kernels are refused: they boot from
//! versions/libraries/ through an installer-generated args file and cannot
//! be merged into one jar.

use crate::lib;
use clap::Args;
use std::io::{Read as _, Write as _};
use std::path::{Path, PathBuf};

/// The committed, Java-8-targeted single-jar entry point (compiled with
/// `javac --release 8` by scripts/build-helpers.sh; keep Boot.java
/// source-compatible with Java 8).
const BOOT_CLASS: &[u8] = include_bytes!("../../launcher/build/dev/dist/launcher/Boot.class");

const BOOT_CLASS_ENTRY: &str = "dev/dist/launcher/Boot.class";
const BOOT_MAIN: &str = "dev.dist.launcher.Boot";
/// Boot's fallback for jars without the attribute (paperclip kernels).
const PAPERCLIP_MAIN: &str = "io.papermc.paperclip.Main";
/// Manifest attribute carrying the kernel's original Main-Class. Must match
/// Boot.KERNEL_MAIN_ATTR.
const KERNEL_MAIN_ATTR: &str = "Crussty-Kernel-Main";

/// Runtime library names the single jar can carry (Boot recognizes all of
/// them; the platform-native one wins at boot time).
const RUNTIME_NAMES: &[&str] = &[
    "libcrussty_runtime.so",
    "libcrussty_runtime.dylib",
    "crussty_runtime.dll",
];

/// Directories that must never leak into the jar from a module dir (build
/// artifacts; the runtime scan skips them too).
const BUILD_DIRS: &[&str] = &["target", "build", "out", "node_modules", ".git"];

#[derive(Args)]
pub struct BuildArgs {
    /// Output jar (default: dist/crussty-<kernel-stem>.jar inside the server dir).
    #[arg(long, value_name = "PATH")]
    pub out: Option<String>,
}

pub fn run(args: BuildArgs) -> i32 {
    let server = lib::server_dir();
    let cfg_text = match std::fs::read_to_string(server.join(lib::CONFIG_NAME)) {
        Ok(t) => t,
        Err(_) => {
            eprintln!("crussty: no crussty.toml here — run 'crussty init' first");
            return 1;
        }
    };
    let cfg = crate::server::parse_tomlish(&cfg_text);
    let kernel = crate::server::cfg_str(&cfg, "server", "kernel", "kernel.jar");

    let kernel_path = server.join("versions").join(&kernel);
    if !kernel_path.is_file() {
        eprintln!(
            "crussty: kernel jar not found at {}\n  put your Minecraft server .jar into versions/ and set kernel = \"<file>\" in crussty.toml\n  or run `crussty init` to pick and download one",
            kernel_path.display()
        );
        return 1;
    }
    if let Err(reason) = validate_kernel_name(&kernel) {
        eprintln!("crussty: {reason}");
        return 1;
    }

    let runtime_path = match find_runtime(&server) {
        Some(p) => p,
        None => {
            eprintln!(
                "crussty: no crussty runtime in {}\n  run `crussty init` (downloads libcrussty_runtime.so) or copy the runtime library here",
                server.display()
            );
            return 1;
        }
    };

    let (module_files, modules, skipped) = collect_modules(&server);
    if module_files.is_empty() {
        println!("crussty: no modules installed — building a runtime-only jar");
    }

    let out_path = match args.out.as_deref() {
        Some(o) => PathBuf::from(o),
        None => server
            .join("dist")
            .join(format!("crussty-{}.jar", file_stem(&kernel_path))),
    };
    if let Some(parent) = out_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if out_path.exists() {
        println!("crussty: overwriting {}", out_path.display());
    }

    println!("crussty: kernel:  {} ({})", kernel_path.display(), mb(&kernel_path));
    println!("crussty: runtime: {} ({})", runtime_path.display(), mb(&runtime_path));
    if modules.is_empty() {
        println!("crussty: modules: none");
    } else {
        println!("crussty: modules: {}", modules.join(", "));
    }
    if !skipped.is_empty() {
        println!("crussty: skipped (parked/disabled, not loaded at runtime anyway): {}", skipped.join(", "));
    }

    if let Err(e) = build_jar(&kernel_path, &runtime_path, &module_files, &out_path) {
        eprintln!("crussty: build failed: {e}");
        let _ = std::fs::remove_file(&out_path);
        return 1;
    }

    println!("crussty: built {} ({})", out_path.display(), mb(&out_path));
    println!("crussty: run it anywhere with a matching JVM — no launcher, no crussty dir needed:");
    println!("crussty:   java -Xmx2G -jar {} --nogui", out_path.file_name().unwrap_or_default().to_string_lossy());
    println!("crussty: on first boot it unpacks ./crussty/ (runtime + modules) into the working");
    println!("crussty: directory and boots the kernel; eula.txt etc. live next to the jar as usual.");
    0
}

/// Only plain bootable jars can be merged. Forge (and anything else that
/// boots through an args file into versions/libraries/) cannot.
fn validate_kernel_name(kernel: &str) -> Result<(), String> {
    if kernel.to_ascii_lowercase().ends_with(".jar") {
        return Ok(());
    }
    Err(format!(
        "kernel '{kernel}' is not a plain jar — it boots from versions/libraries/ via its args file and cannot be packed into one jar\n  single-jar packaging needs a jar kernel: Paper, Purpur, Fabric or vanilla"
    ))
}

/// The runtime library in the server root; platform-native name first.
fn find_runtime(server: &Path) -> Option<PathBuf> {
    let preferred = if cfg!(target_os = "windows") {
        "crussty_runtime.dll"
    } else if cfg!(target_os = "macos") {
        "libcrussty_runtime.dylib"
    } else {
        "libcrussty_runtime.so"
    };
    let mut names = vec![preferred];
    for n in RUNTIME_NAMES {
        if !names.contains(n) {
            names.push(n);
        }
    }
    names
        .into_iter()
        .map(|n| server.join(n))
        .find(|p| p.is_file())
}

/// Every file that goes into the jar under `modules/…`, plus the module
/// names for the summary and the parked/disabled dirs that were skipped.
/// Semantics mirror the runtime scan (runtime/src/scan.rs): every active
/// directory is embedded recursively; `*.x` / `*.disabled` are skipped;
/// top-level `.zip`/`.jar` archive modules are embedded as-is.
fn collect_modules(server: &Path) -> (Vec<(String, PathBuf)>, Vec<String>, Vec<String>) {
    let mut files = Vec::new();
    let mut modules = Vec::new();
    let mut skipped = Vec::new();
    let root = lib::server_modules_dir(server);
    let Ok(entries) = std::fs::read_dir(&root) else {
        return (files, modules, skipped);
    };
    let mut items: Vec<(String, PathBuf)> = entries
        .flatten()
        .map(|e| (e.file_name().to_string_lossy().into_owned(), e.path()))
        .collect();
    items.sort_by(|a, b| a.0.cmp(&b.0));
    for (name, path) in items {
        if path.is_dir() {
            if name.ends_with(".x") || name.ends_with(".disabled") {
                skipped.push(name);
                continue;
            }
            modules.push(name.clone());
            walk_module_dir(&path, &name, &mut files);
        } else if path.is_file() && is_archive(&name) {
            // archive module (runtime extracts + scans it at boot)
            modules.push(name.clone());
            files.push((format!("modules/{name}"), path));
        }
    }
    (files, modules, skipped)
}

fn is_archive(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.ends_with(".zip") || lower.ends_with(".jar")
}

fn walk_module_dir(dir: &Path, rel: &str, out: &mut Vec<(String, PathBuf)>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut items: Vec<(String, PathBuf)> = entries
        .flatten()
        .map(|e| (e.file_name().to_string_lossy().into_owned(), e.path()))
        .collect();
    items.sort_by(|a, b| a.0.cmp(&b.0));
    for (name, path) in items {
        let child = format!("{rel}/{name}");
        if path.is_dir() {
            if BUILD_DIRS.contains(&name.as_str()) {
                continue;
            }
            walk_module_dir(&path, &child, out);
        } else if path.is_file() {
            out.push((format!("modules/{child}"), path));
        }
    }
}

// ---------------------------------------------------------------------------
// Jar assembly
// ---------------------------------------------------------------------------

fn build_jar(
    kernel_path: &Path,
    runtime_path: &Path,
    module_files: &[(String, PathBuf)],
    out_path: &Path,
) -> Result<(), String> {
    let kernel_file = std::fs::File::open(kernel_path)
        .map_err(|e| format!("open {}: {e}", kernel_path.display()))?;
    let mut kernel = zip::ZipArchive::new(kernel_file)
        .map_err(|e| format!("{}: not a readable jar: {e}", kernel_path.display()))?;

    // Kernel manifest: keep every attribute, override Main-Class to Boot and
    // remember the original under Crussty-Kernel-Main.
    let manifest_bytes = read_entry(&mut kernel, "META-INF/MANIFEST.MF")?;
    let attrs = parse_manifest(&manifest_bytes);
    let merged = build_manifest(&attrs);

    let out_file = std::fs::File::create(out_path)
        .map_err(ierr(format!("create {}", out_path.display())))?;
    let mut w = zip::ZipWriter::new(out_file);
    let opts = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    w.start_file("META-INF/MANIFEST.MF", opts.clone())
        .map_err(zerr("write manifest"))?;
    w.write_all(&merged).map_err(ierr("write manifest"))?;

    // Kernel entries: everything except the manifest (replaced above) and the
    // signing files (signatures cannot survive a re-zip; drop instead of
    // shipping broken ones). Duplicate names keep the first occurrence.
    let mut seen: std::collections::HashSet<String> =
        std::iter::once("META-INF/MANIFEST.MF".to_string()).collect();
    for i in 0..kernel.len() {
        let mut entry = kernel
            .by_index(i)
            .map_err(|e| format!("kernel entry {i}: {e}"))?;
        let name = entry.name().to_string();
        if entry.is_dir() || !seen.insert(name.clone()) {
            continue;
        }
        if name.starts_with("META-INF/")
            && (name.ends_with(".SF") || name.ends_with(".RSA") || name.ends_with(".DSA"))
        {
            continue;
        }
        let mut buf = Vec::new();
        entry
            .read_to_end(&mut buf)
            .map_err(|e| format!("read {name}: {e}"))?;
        w.start_file(name.clone(), opts.clone())
            .map_err(zerr(format!("write {name}")))?;
        w.write_all(&buf).map_err(ierr(format!("write {name}")))?;
    }

    // The single-jar entry point (committed, Java-8 bytecode).
    w.start_file(BOOT_CLASS_ENTRY, opts.clone())
        .map_err(zerr("write Boot.class"))?;
    w.write_all(BOOT_CLASS).map_err(ierr("write Boot.class"))?;

    // The native runtime at the jar root (Boot scans for the known names).
    let runtime_name = runtime_path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .ok_or("runtime path has no file name")?;
    w.start_file(runtime_name.clone(), opts.clone())
        .map_err(zerr("write runtime"))?;
    let mut rt = std::fs::File::open(runtime_path)
        .map_err(ierr(format!("open {}", runtime_path.display())))?;
    std::io::copy(&mut rt, &mut w).map_err(ierr("write runtime"))?;

    // Modules: module.json + entry lib + bundled native deps, paths preserved.
    for (rel, path) in module_files {
        w.start_file(rel.clone(), opts.clone())
            .map_err(zerr(format!("write {rel}")))?;
        let mut f = std::fs::File::open(path)
            .map_err(ierr(format!("open {}", path.display())))?;
        std::io::copy(&mut f, &mut w).map_err(ierr(format!("write {rel}")))?;
    }

    w.finish().map_err(zerr("finish jar"))?;
    Ok(())
}

fn zerr(ctx: impl Into<String>) -> impl Fn(zip::result::ZipError) -> String {
    let ctx = ctx.into();
    move |e| format!("{ctx}: {e}")
}

fn ierr(ctx: impl Into<String>) -> impl Fn(std::io::Error) -> String {
    let ctx = ctx.into();
    move |e| format!("{ctx}: {e}")
}

fn read_entry(archive: &mut zip::ZipArchive<std::fs::File>, name: &str) -> Result<Vec<u8>, String> {
    let mut entry = archive
        .by_name(name)
        .map_err(|e| format!("kernel jar has no {name}: {e}"))?;
    let mut buf = Vec::new();
    entry
        .read_to_end(&mut buf)
        .map_err(|e| format!("read {name}: {e}"))?;
    Ok(buf)
}

// ---------------------------------------------------------------------------
// Manifest parse / merge (main attributes only)
// ---------------------------------------------------------------------------

/// Parse the main section of a JAR manifest into `key: value` pairs.
/// Handles \r\n and \n line endings and the 72-byte folding convention
/// (continuation lines start with a single space that is not part of the
/// value). Per-entry sections (after the first blank line) are ignored.
fn parse_manifest(bytes: &[u8]) -> Vec<(String, String)> {
    let text = String::from_utf8_lossy(bytes).replace("\r\n", "\n");
    let mut out: Vec<(String, String)> = Vec::new();
    let mut cur: Option<(String, String)> = None;
    for line in text.split('\n') {
        if line.is_empty() {
            break; // blank line ends the main section
        }
        if let Some(rest) = line.strip_prefix(' ') {
            if let Some((_, v)) = cur.as_mut() {
                v.push_str(rest.trim_end_matches('\r'));
            }
            continue;
        }
        if let Some((k, v)) = line.split_once(':') {
            if let Some(pair) = cur.take() {
                out.push(pair);
            }
            let v = v.strip_prefix(' ').unwrap_or(v);
            cur = Some((k.trim().to_string(), v.trim_end_matches('\r').to_string()));
        }
    }
    if let Some(pair) = cur.take() {
        out.push(pair);
    }
    out
}

/// The merged manifest: the kernel's own attributes, then the overrides.
fn build_manifest(kernel_attrs: &[(String, String)]) -> Vec<u8> {
    let mut kernel_main = PAPERCLIP_MAIN.to_string();
    let mut version = "1.0".to_string();
    for (k, v) in kernel_attrs {
        let t = v.trim();
        if t.is_empty() {
            continue;
        }
        match k.as_str() {
            "Main-Class" => kernel_main = t.to_string(),
            "Manifest-Version" => version = t.to_string(),
            _ => {}
        }
    }
    let mut out = Vec::new();
    push_attr(&mut out, "Manifest-Version", &version);
    for (k, v) in kernel_attrs {
        if matches!(
            k.as_str(),
            "Main-Class" | "Manifest-Version" | "Enable-Native-Access" | KERNEL_MAIN_ATTR
        ) {
            continue;
        }
        push_attr(&mut out, k, v);
    }
    push_attr(&mut out, "Main-Class", BOOT_MAIN);
    push_attr(&mut out, KERNEL_MAIN_ATTR, &kernel_main);
    push_attr(&mut out, "Enable-Native-Access", "ALL-UNNAMED");
    out
}

/// Append one manifest line, folding values longer than the 72-byte spec
/// limit (first line ≤ 70 bytes, continuations = single space + ≤ 69 bytes,
/// split only on UTF-8 char boundaries).
fn push_attr(out: &mut Vec<u8>, key: &str, value: &str) {
    let line = format!("{key}: {value}");
    let bytes = line.as_bytes();
    if bytes.len() <= 70 {
        out.extend_from_slice(bytes);
        out.extend_from_slice(b"\r\n");
        return;
    }
    let mut pos = 0usize;
    let mut first = true;
    while pos < bytes.len() {
        let max = if first { 70 } else { 69 };
        let mut end = (pos + max).min(bytes.len());
        while end > pos && !line.is_char_boundary(end) {
            end -= 1;
        }
        if !first {
            out.push(b' ');
        }
        out.extend_from_slice(&bytes[pos..end]);
        out.extend_from_slice(b"\r\n");
        pos = end;
        first = false;
    }
}

// ---------------------------------------------------------------------------
// Small helpers
// ---------------------------------------------------------------------------

fn file_stem(path: &Path) -> String {
    path.file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "kernel".to_string())
}

fn mb(path: &Path) -> String {
    let len = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    format!("{:.1} MB", len as f64 / (1024.0 * 1024.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_test_zip(path: &Path, entries: &[(&str, &[u8])]) {
        let file = fs::File::create(path).unwrap();
        let mut w = zip::ZipWriter::new(file);
        let opts = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        for (name, data) in entries {
            w.start_file(*name, opts.clone()).unwrap();
            w.write_all(data).unwrap();
        }
        w.finish().unwrap();
    }

    fn temp_dir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("crussty-build-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn manifest_parse_handles_folding_and_crlf() {
        let raw = b"Manifest-Version: 1.0\r\nImplementation-Title: Paper\r\n  Server\r\nMain-Class: io.papermc.paperclip.Main\r\nMulti-Release: true\r\n\r\nName: x\r\nContent-Type: y\r\n\r\n";
        let attrs = parse_manifest(raw);
        assert_eq!(
            attrs,
            vec![
                ("Manifest-Version".to_string(), "1.0".to_string()),
                (
                    "Implementation-Title".to_string(),
                    // the continuation's leading space is a marker, not value:
                    // "Paper" + " Server"
                    "Paper Server".to_string()
                ),
                ("Main-Class".to_string(), "io.papermc.paperclip.Main".to_string()),
                ("Multi-Release".to_string(), "true".to_string()),
            ]
        );
    }

    #[test]
    fn manifest_lines_stay_within_spec_limit() {
        let attrs = vec![
            ("Manifest-Version".to_string(), "1.0".to_string()),
            (
                "Implementation-Version".to_string(),
                "1.21.10-R0.1-SNAPSHOT-very-long-build-metadata-string-that-forces-folding".repeat(2),
            ),
        ];
        let bytes = build_manifest(&attrs);
        for line in String::from_utf8_lossy(&bytes).split("\r\n") {
            assert!(
                line.len() <= 72,
                "manifest line exceeds 72 bytes: {line}"
            );
        }
        // roundtrip: the folded value must parse back intact
        let parsed = parse_manifest(&bytes);
        let impl_version = parsed
            .iter()
            .find(|(k, _)| k == "Implementation-Version")
            .map(|(_, v)| v.clone())
            .unwrap();
        assert_eq!(impl_version, "1.21.10-R0.1-SNAPSHOT-very-long-build-metadata-string-that-forces-folding".repeat(2));
    }

    #[test]
    fn manifest_overrides_main_class_and_records_kernel() {
        let attrs = vec![
            ("Manifest-Version".to_string(), "1.0".to_string()),
            ("Main-Class".to_string(), "io.papermc.paperclip.Main".to_string()),
            ("Multi-Release".to_string(), "true".to_string()),
        ];
        let text = String::from_utf8(build_manifest(&attrs)).unwrap();
        assert!(text.contains("Main-Class: dev.dist.launcher.Boot\r\n"));
        assert!(text.contains("Crussty-Kernel-Main: io.papermc.paperclip.Main\r\n"));
        assert!(text.contains("Enable-Native-Access: ALL-UNNAMED\r\n"));
        assert!(text.contains("Multi-Release: true\r\n"));
        // kernel main comes first among the overrides, manifest version first overall
        let mv = text.find("Manifest-Version: 1.0").unwrap();
        assert_eq!(mv, 0);
    }

    #[test]
    fn manifest_defaults_to_paperclip_when_kernel_has_no_main() {
        let text = String::from_utf8(build_manifest(&[])).unwrap();
        assert!(text.contains("Crussty-Kernel-Main: io.papermc.paperclip.Main\r\n"));
        assert!(text.contains("Manifest-Version: 1.0\r\n"));
    }

    #[test]
    fn forge_style_kernel_names_are_rejected() {
        assert!(validate_kernel_name("purpur-1.21.10.jar").is_ok());
        assert!(validate_kernel_name("KERNEL.JAR").is_ok());
        let err = validate_kernel_name(
            "libraries/net/minecraftforge/forge/1.21.10-52.0.24/unix_args.txt",
        )
        .unwrap_err();
        assert!(err.contains("not a plain jar"));
    }

    #[test]
    fn collect_modules_skips_parked_disabled_and_build_dirs() {
        let dir = temp_dir("collect");
        // active module with manifest + lib + bundled native dep
        fs::create_dir_all(dir.join("modules/hello/native")).unwrap();
        fs::write(dir.join("modules/hello/module.json"), b"{\"id\":\"hello\"}").unwrap();
        fs::write(dir.join("modules/hello/libhello.so"), b"x").unwrap();
        fs::write(dir.join("modules/hello/native/libdep.so"), b"x").unwrap();
        // build output inside a module crate must not leak into the jar
        fs::create_dir_all(dir.join("modules/hello/target/debug")).unwrap();
        fs::write(dir.join("modules/hello/target/debug/junk.so"), b"x").unwrap();
        // parked / disabled
        fs::create_dir_all(dir.join("modules/old.x")).unwrap();
        fs::write(dir.join("modules/old.x/module.json"), b"{}").unwrap();
        fs::create_dir_all(dir.join("modules/gone.disabled")).unwrap();
        fs::write(dir.join("modules/gone.disabled/module.json"), b"{}").unwrap();
        // archive module at the top level
        fs::write(dir.join("modules/bundled.zip"), b"PK").unwrap();
        // unrelated loose file: not a module, not embedded
        fs::write(dir.join("modules/notes.txt"), b"hi").unwrap();

        let (files, modules, skipped) = collect_modules(&dir);
        assert_eq!(modules, vec!["bundled.zip", "hello"]);
        assert_eq!(skipped, vec!["gone.disabled", "old.x"]);
        let rels: Vec<&str> = files.iter().map(|(r, _)| r.as_str()).collect();
        assert!(rels.contains(&"modules/hello/module.json"));
        assert!(rels.contains(&"modules/hello/libhello.so"));
        assert!(rels.contains(&"modules/hello/native/libdep.so"));
        assert!(rels.contains(&"modules/bundled.zip"));
        assert!(!rels.iter().any(|r| r.contains("target")));
        assert!(!rels.iter().any(|r| r.contains("notes.txt")));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn built_jar_has_boot_runtime_modules_and_kernel_entries() {
        let dir = temp_dir("jar");
        // --- fake server dir ---
        let kernel_manifest = "Manifest-Version: 1.0\r\nMain-Class: io.papermc.paperclip.Main\r\nMulti-Release: true\r\n\r\n";
        let kernel_entries: Vec<(&str, &[u8])> = vec![
            ("META-INF/MANIFEST.MF", kernel_manifest.as_bytes()),
            (
                "io/papermc/paperclip/Main.class",
                b"\xca\xfe\xba\xbe fake paperclip main class",
            ),
            (
                "META-INF/services/java.net.spi.URLClassLoaderProvider",
                b"example.Provider\n",
            ),
            ("META-INF/PAPER.SF", b"broken signature bytes"),
            ("net/minecraft/server/MinecraftServer.class", b"fake vanilla class"),
        ];
        let kernel_path = dir.join("versions/purpur-9.9.9.jar");
        fs::create_dir_all(kernel_path.parent().unwrap()).unwrap();
        write_test_zip(&kernel_path, &kernel_entries);

        fs::create_dir_all(dir.join("modules/hello")).unwrap();
        fs::write(dir.join("modules/hello/module.json"), b"{\"id\":\"hello\"}").unwrap();
        fs::write(dir.join("modules/hello/libhello.so"), b"module bytes").unwrap();
        let runtime_path = dir.join("libcrussty_runtime.so");
        fs::write(&runtime_path, b"fake runtime lib").unwrap();

        let out_path = dir.join("dist/crussty-purpur-9.9.9.jar");
        fs::create_dir_all(out_path.parent().unwrap()).unwrap();
        let (files, _, _) = collect_modules(&dir);
        build_jar(&kernel_path, &runtime_path, &files, &out_path).unwrap();
        assert!(out_path.is_file());

        let f = fs::File::open(&out_path).unwrap();
        let mut jar = zip::ZipArchive::new(f).unwrap();
        let names: Vec<String> = (0..jar.len())
            .map(|i| jar.by_index(i).unwrap().name().to_string())
            .collect();

        // kernel entries survive (services kept, signature dropped)
        assert!(names.contains(&"io/papermc/paperclip/Main.class".to_string()));
        assert!(names.contains(&"net/minecraft/server/MinecraftServer.class".to_string()));
        assert!(names.contains(&"META-INF/services/java.net.spi.URLClassLoaderProvider".to_string()));
        assert!(!names.iter().any(|n| n.ends_with(".SF")));

        // boot + runtime + modules
        assert!(names.contains(&BOOT_CLASS_ENTRY.to_string()));
        assert!(names.contains(&"libcrussty_runtime.so".to_string()));
        assert!(names.contains(&"modules/hello/module.json".to_string()));
        assert!(names.contains(&"modules/hello/libhello.so".to_string()));

        // manifest: Boot boots, paperclip is remembered
        let mf_bytes = {
            let mut mf = jar.by_name("META-INF/MANIFEST.MF").unwrap();
            let mut buf = Vec::new();
            std::io::Read::read_to_end(&mut mf, &mut buf).unwrap();
            buf
        };
        let text = String::from_utf8_lossy(&mf_bytes);
        assert!(text.contains("Main-Class: dev.dist.launcher.Boot\r\n"));
        assert!(text.contains("Crussty-Kernel-Main: io.papermc.paperclip.Main\r\n"));
        assert!(text.contains("Multi-Release: true\r\n"));
        assert!(text.contains("Enable-Native-Access: ALL-UNNAMED\r\n"));

        // Boot.class bytes are embedded verbatim
        let boot_bytes = {
            let mut boot = jar.by_name(BOOT_CLASS_ENTRY).unwrap();
            let mut buf = Vec::new();
            std::io::Read::read_to_end(&mut boot, &mut buf).unwrap();
            buf
        };
        assert_eq!(boot_bytes, BOOT_CLASS);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn runtime_names_cover_all_platforms() {
        // Boot recognizes exactly the names build may embed.
        assert_eq!(RUNTIME_NAMES.len(), 3);
        assert!(RUNTIME_NAMES.contains(&"libcrussty_runtime.so"));
        assert!(RUNTIME_NAMES.contains(&"libcrussty_runtime.dylib"));
        assert!(RUNTIME_NAMES.contains(&"crussty_runtime.dll"));
    }
}
