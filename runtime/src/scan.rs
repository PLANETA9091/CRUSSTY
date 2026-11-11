//! Convention (matches how the rest of the world ships plugins — LV2 bundles,
//! npm/Node packages, VST3 bundles, Mattermost plugins): a plugin is a
//! DIRECTORY containing a `module.json` manifest. The manifest names the
//! entry library in `main` (path relative to the manifest, like package.json
//! `main` / LV2 `lv2:binary`); when absent it defaults to the platform
//! cdylib name for the plugin id (`lib<id>.so` / `lib<id>.dylib` /
//! `<id>.dll`). Libraries without a manifest — e.g. bundled native
//! dependencies under `native/` — are never dlopened as plugins.
//!
//! Plugins can also be shipped as single-file archives (`.zip` or `.jar`,
//! like Java plugins / Chrome `.crx`): the archive is extracted to a
//! content-addressed cache dir under the system temp dir, then treated as a
//! regular plugin directory (manifest at the archive top level). Native
//! libraries cannot be dlopened from inside an archive, so extract-then-load
//! is the only viable path (same as this project's own `NativeLoader`:
//! lib from jar -> temp -> System.load). Extraction is guarded against
//! path-traversal ("zip-slip": no `..` / absolute / drive-letter components)
//! and zip bombs (entry count + per-entry and total uncompressed size caps).

use std::collections::HashMap;
use std::env::consts::{DLL_PREFIX, DLL_SUFFIX};
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct CPlugin {
    /// Path of the .so / .dylib / .dll to dlopen.
    pub lib_path: PathBuf,
    /// Manifest id (defaults to the plugin directory name).
    pub id: String,
    /// Declared dependencies (manifest `dependencies`), empty if none.
    pub deps: Vec<String>,
}

/// Scan `root` recursively for plugins. Plugins are directories containing a
/// `module.json` manifest; the manifest's `main` field names the entry
/// library relative to that directory (default: the platform cdylib name for
/// the plugin id, e.g. `lib<id>.so` on Linux, `<id>.dll` on Windows). Bundled
/// native libraries without a manifest are never plugins. `.zip` / `.jar`
/// archives anywhere under `root` are extracted to a content-addressed cache
/// dir under the system temp dir and scanned as plugins when they carry a
/// top-level `module.json`. Anything (file or dir) named `*.disabled` (Paper
/// convention) or `*.x` (ad-hoc park: `mv cells cells.x`) is skipped.
pub fn scan(root: &Path) -> Vec<CPlugin> {
    let mut found = Vec::new();
    let mut zips = Vec::new();
    walk_dir(root, &mut found, &mut zips);
    for zip_path in zips {
        match extract_zip_plugin(&zip_path) {
            Some(manifest) => {
                if let Some(p) = load_manifest(&manifest) {
                    found.push(p);
                }
            }
            None => eprintln!(
                "[crussty-runtime] archive {} is not a plugin (no top-level module.json or rejected)",
                zip_path.display()
            ),
        }
    }
    // Topological ordering by declared deps; unknown/missing deps fall back to
    // the sorted relative path (deterministic; warnings logged by caller).
    let ids: HashMap<String, usize> = found
        .iter()
        .enumerate()
        .map(|(i, p)| (p.id.clone(), i))
        .collect();
    let mut order = vec![];
    let mut state = vec![0u8; found.len()]; // 0=unvisited 1=visiting 2=done
    fn visit(
        i: usize,
        plugins: &[CPlugin],
        ids: &HashMap<String, usize>,
        state: &mut [u8],
        order: &mut Vec<usize>,
    ) {
        if state[i] == 2 {
            return;
        }
        if state[i] == 1 {
            return; // dependency cycle: keep sorted position, warn later
        }
        state[i] = 1;
        for dep in &plugins[i].deps {
            if let Some(&d) = ids.get(dep) {
                visit(d, plugins, ids, state, order);
            }
        }
        state[i] = 2;
        order.push(i);
    }
    for i in 0..found.len() {
        visit(i, &found, &ids, &mut state, &mut order);
    }
    order.into_iter().map(|i| found[i].clone()).collect()
}

/// Build artifacts would be picked up by a recursive scan if a module crate
/// lives inside modules/ — skip known build/output dirs.
fn is_build_dir(name: &str) -> bool {
    matches!(name, "target" | "build" | "out" | "node_modules" | ".git")
}

fn walk_dir(dir: &Path, out: &mut Vec<CPlugin>, zips: &mut Vec<PathBuf>) {
    let mut entries = match fs::read_dir(dir) {
        Ok(e) => e.flatten().collect::<Vec<_>>(),
        Err(_) => return,
    };
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let path = e.path();
        let name = e.file_name().to_string_lossy().into_owned();
        if name.ends_with(".disabled") || name.ends_with(".x") {
            continue;
        }
        if path.is_dir() {
            if is_build_dir(&name) {
                continue;
            }
            walk_dir(&path, out, zips);
            continue;
        }
        let lower = name.to_ascii_lowercase();
        if lower.ends_with(".zip") || lower.ends_with(".jar") {
            zips.push(path);
            continue;
        }
        if name == "module.json" {
            if let Some(p) = load_manifest(&path) {
                out.push(p);
            }
        }
    }
}

fn load_manifest(manifest_path: &Path) -> Option<CPlugin> {
    let dir = manifest_path.parent()?;
    let text = fs::read_to_string(manifest_path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&text).unwrap_or_default();
    let id = v
        .get("id")
        .and_then(|s| s.as_str())
        .map(str::to_string)
        .unwrap_or_else(|| dir_name(dir));
    let deps = v
        .get("dependencies")
        .and_then(|d| d.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|d| d.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    // Entry library: manifest `main` relative to the manifest, otherwise the
    // platform cdylib name for the id next to it. It must resolve to an
    // existing file inside the plugin directory — an absolute path (POSIX
    // `/`, Windows `C:\`, UNC `\\`) or any `..`-escaping component in `main`
    // marks the manifest as malformed and the whole plugin is skipped.
    let entry = match v.get("main").and_then(|s| s.as_str()) {
        Some(m) if !is_absolute_path(m) && !m.split(['/', '\\']).any(|c| c == "..") => {
            dir.join(m)
        }
        None => dir.join(default_entry(&id)),
        Some(_) => return None,
    };
    if !entry.starts_with(dir) || !entry.is_file() {
        return None;
    }
    Some(CPlugin {
        lib_path: entry,
        id,
        deps,
    })
}

fn dir_name(dir: &Path) -> String {
    dir.file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Default entry library for a plugin id: the cdylib name Rust produces on
/// this platform — `lib<id>.so` (Linux), `lib<id>.dylib` (macOS), `<id>.dll`
/// (Windows, no `lib` prefix on MSVC).
fn default_entry(id: &str) -> String {
    format!("{DLL_PREFIX}{id}{DLL_SUFFIX}")
}

/// Absolute paths in a manifest `main` must be rejected on every platform:
/// POSIX `/foo`, Windows drive `C:\foo`, UNC `\\host\share\foo`. This runs on
/// the host the runtime is built for; `\`-led paths are absolute on Windows and
/// rejected there, and a `\` is not a path separator on POSIX (a lone `\`
/// component is harmless), but rejecting both separators is the safe call.
fn is_absolute_path(m: &str) -> bool {
    let drive = m.len() >= 2 && m.as_bytes()[0].is_ascii_alphabetic() && m.as_bytes()[1] == b':';
    m.starts_with('/') || m.starts_with('\\') || drive
}

// ---------------------------------------------------------------------------
// Archive plugins (.zip / .jar): extract to a content-addressed cache dir in
// the system temp dir, then scan the extracted dir like a regular plugin.
// ---------------------------------------------------------------------------

/// Zip-bomb / zip-slip limits (OWASP: validate entry names, cap counts and
/// sizes, never write symlinks).
const MAX_ARCHIVE_ENTRIES: usize = 10_000;
const MAX_ENTRY_SIZE: u64 = 256 * 1024 * 1024;
const MAX_TOTAL_SIZE: u64 = 1024 * 1024 * 1024;

/// Extract `zip_path` to `temp_dir()/cplug-cache/<stem>-<content-hash>/` when
/// not already cached, then return the path of its top-level `module.json`
/// if the archive actually is a plugin. Re-extraction only happens when the
/// content changes (the hash is part of the dir name). Archives without a
/// top-level `module.json` are not plugins and are never extracted/cached.
fn extract_zip_plugin(zip_path: &Path) -> Option<PathBuf> {
    let bytes = fs::read(zip_path).ok()?;
    // Peek first: a manifest-less archive (docs, build output) must not leave
    // a cache dir behind.
    let manifest_ok = {
        let mut peek = zip::ZipArchive::new(Cursor::new(&bytes)).ok()?;
        peek.by_name("module.json")
            .is_ok_and(|e| e.is_file())
    };
    if !manifest_ok {
        return None;
    }
    let stem = zip_path.file_stem()?.to_string_lossy().into_owned();
    let dir = archive_cache_root().join(format!("{stem}-{}", fnv1a_hex(&bytes)));
    if !dir.exists() {
        let tmp = archive_cache_root().join(format!(".{stem}-{}.tmp", fnv1a_hex(&bytes)));
        let _ = fs::remove_dir_all(&tmp);
        match extract_zip(&bytes, &tmp) {
            Ok(()) => {
                if let Err(e) = fs::rename(&tmp, &dir) {
                    let _ = fs::remove_dir_all(&tmp);
                    eprintln!("[crussty-runtime] cache {}: {e}", dir.display());
                    return None;
                }
            }
            Err(reason) => {
                let _ = fs::remove_dir_all(&tmp);
                eprintln!("[crussty-runtime] reject {}: {reason}", zip_path.display());
                return None;
            }
        }
    }
    let manifest = dir.join("module.json");
    manifest.is_file().then_some(manifest)
}

fn archive_cache_root() -> PathBuf {
    std::env::temp_dir().join("cplug-cache")
}

/// Safe extraction: every entry path must stay inside `dest` (no `..`, no
/// absolute or drive-letter components on either separator), no symlinks or
/// other special files, and size/count caps enforced. Returns Err on the
/// first violation; nothing is partially kept (caller discards `dest`).
fn extract_zip(bytes: &[u8], dest: &Path) -> Result<(), String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|e| format!("invalid zip: {e}"))?;
    if archive.len() > MAX_ARCHIVE_ENTRIES {
        return Err(format!("too many entries: {}", archive.len()));
    }
    let mut total: u64 = 0;
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("entry {i}: {e}"))?;
        let raw = entry.name().to_string();
        // A leading `/` is how some tools mark absolute paths; `C:` / `C:\`
        // components are drive-absolute on Windows. Anything else escaping
        // (or a `\` used instead of `/`) is rejected up front.
        let name = raw.trim_start_matches('/');
        if name.is_empty() {
            continue;
        }
        let mut components = name.split(['/', '\\']).peekable();
        if components.any(|c| c == ".." || c.is_empty() || c.ends_with(':')) {
            return Err(format!("unsafe path in archive: {raw}"));
        }
        // Never materialize symlinks (or devices): a malicious archive could
        // point a symlink outside `dest` and have the next write follow it.
        if entry
            .unix_mode()
            .is_some_and(|m| (m & 0o170000) == 0o120000)
        {
            return Err(format!("symlink in archive: {raw}"));
        }
        let size = entry.size();
        if size > MAX_ENTRY_SIZE {
            return Err(format!("entry too large: {raw}"));
        }
        total += size;
        if total > MAX_TOTAL_SIZE {
            return Err("archive exceeds total size limit".to_string());
        }
        let out = dest.join(name);
        if entry.is_dir() {
            fs::create_dir_all(&out).map_err(|e| format!("mkdir {raw}: {e}"))?;
            continue;
        }
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
        }
        let mut file = fs::File::create(&out).map_err(|e| format!("create {raw}: {e}"))?;
        std::io::copy(&mut entry, &mut file).map_err(|e| format!("write {raw}: {e}"))?;
    }
    Ok(())
}

/// FNV-1a 64 over the archive bytes — cheap change detection for the cache
/// dir name (not a security hash; the archive is validated on the way out).
fn fnv1a_hex(bytes: &[u8]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{h:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn recursive_scan_with_groups_and_disabled() {
        let dir = std::env::temp_dir().join(format!("cplug-test-{}", std::process::id()));
        fs::create_dir_all(dir.join("dist")).unwrap();
        fs::create_dir_all(dir.join("group")).unwrap();
        fs::create_dir_all(dir.join("liba")).unwrap();
        fs::write(dir.join("cplug.disabled"), b"").unwrap();
        fs::write(
            dir.join("dist/module.json"),
            r#"{"id":"dist","main":"libdist_core.so","dependencies":["libb"]}"#,
        )
        .unwrap();
        fs::write(dir.join("dist/libdist_core.so"), b"x").unwrap();
        fs::write(dir.join("liba.so"), b"x").unwrap();
        fs::write(dir.join("liba.disabled"), b"x").unwrap();
        fs::write(dir.join("liba/module.json"), r#"{"id":"liba"}"#).unwrap();
        fs::write(dir.join("liba/libliba.so"), b"x").unwrap();
        fs::write(dir.join("group/module.json"), r#"{"id":"libb"}"#).unwrap();
        fs::write(dir.join("group/liblibb.so"), b"x").unwrap();
        fs::write(dir.join("group/readme.txt"), b"ignore").unwrap();
        // bundled native libs without a manifest must NOT be scanned
        fs::create_dir_all(dir.join("group/vendor")).unwrap();
        fs::write(dir.join("group/vendor/libpaper_native_jni.so"), b"x").unwrap();
        // default entry lib<id>.so missing -> whole plugin skipped
        fs::create_dir_all(dir.join("empty")).unwrap();
        fs::write(dir.join("empty/module.json"), r#"{"id":"empty"}"#).unwrap();
        // *.x parked plugins are skipped too (ad-hoc disable)
        fs::create_dir_all(dir.join("parked.x")).unwrap();
        fs::write(
            dir.join("parked.x/module.json"),
            r#"{"id":"parked","main":"libparked.so"}"#,
        )
        .unwrap();
        fs::write(dir.join("parked.x/libparked.so"), b"x").unwrap();

        let plugins = scan(&dir);
        let ids: Vec<String> = plugins.iter().map(|p| p.id.clone()).collect();
        // depth-first sorted scan: dist(grouped), libb, liba — but dist declares
        // dep on libb, so topological order puts libb before dist.
        assert_eq!(ids, vec!["libb", "dist", "liba"]);
        // dist names its entry via `main`, decoupled from `lib<id>.so`.
        let dist = plugins.iter().find(|p| p.id == "dist").unwrap();
        assert_eq!(
            dist.lib_path.file_name().unwrap().to_string_lossy(),
            "libdist_core.so"
        );
        // `main` must not escape the plugin directory.
        fs::write(
            dir.join("group/module.json"),
            r#"{"id":"libb","main":"../evil.so"}"#,
        )
        .unwrap();
        fs::write(dir.join("evil.so"), b"x").unwrap();
        let plugins = scan(&dir);
        assert!(plugins.iter().all(|p| p.id != "libb"));
        let ids: Vec<String> = plugins.iter().map(|p| p.id.clone()).collect();
        assert_eq!(ids, vec!["dist", "liba"]);
        // Windows-style escaping must also be rejected, not just POSIX `../`.
        fs::write(
            dir.join("group/module.json"),
            r#"{"id":"libb","main":"C:\evil.dll"}"#,
        )
        .unwrap();
        fs::write(dir.join("evil.dll"), b"x").unwrap();
        let plugins = scan(&dir);
        assert!(plugins.iter().all(|p| p.id != "libb"));
        fs::write(
            dir.join("group/module.json"),
            r#"{"id":"libb","main":"..\evil.dll"}"#,
        )
        .unwrap();
        let plugins = scan(&dir);
        assert!(plugins.iter().all(|p| p.id != "libb"));
        fs::remove_dir_all(&dir).unwrap();
    }

    fn write_test_zip(path: &Path, entries: &[(&str, &[u8])]) {
        let file = fs::File::create(path).unwrap();
        let mut w = zip::ZipWriter::new(file);
        let opts = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        for (name, data) in entries {
            w.start_file(*name, opts).unwrap();
            std::io::Write::write_all(&mut w, data).unwrap();
        }
        w.finish().unwrap();
    }

    fn temp_root(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!("cplug-zip-{tag}-{}", std::process::id()))
    }

    #[test]
    fn zip_plugin_scanned() {
        let root = temp_root("zip");
        fs::create_dir_all(&root).unwrap();
        write_test_zip(
            &root.join("foo.zip"),
            &[
                ("module.json", br#"{"id":"foo","main":"libfoo.so"}"#),
                ("libfoo.so", b"x"),
            ],
        );
        let plugins = scan(&root);
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].id, "foo");
        // entry lib must resolve inside the extraction cache dir
        let lib = &plugins[0].lib_path;
        assert_eq!(lib.file_name().unwrap().to_string_lossy(), "libfoo.so");
        assert!(lib.to_string_lossy().contains("cplug-cache"));
        assert!(lib.is_file());
        // cached: second scan must not re-extract (same dir)
        let plugins2 = scan(&root);
        assert_eq!(plugins2[0].lib_path, *lib);
        let cache = lib.parent().unwrap();
        fs::remove_dir_all(&root).unwrap();
        fs::remove_dir_all(cache).unwrap();
    }

    #[test]
    fn jar_plugin_scanned() {
        let root = temp_root("jar");
        fs::create_dir_all(&root).unwrap();
        write_test_zip(
            &root.join("bar.jar"),
            &[("module.json", br#"{"id":"bar"}"#), ("libbar.so", b"x")],
        );
        let plugins = scan(&root);
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].id, "bar");
        fs::remove_dir_all(&root).unwrap();
        fs::remove_dir_all(plugins[0].lib_path.parent().unwrap()).unwrap();
    }

    #[test]
    fn zip_without_manifest_ignored() {
        let root = temp_root("nomanifest");
        fs::create_dir_all(&root).unwrap();
        write_test_zip(&root.join("data.zip"), &[("readme.txt", b"hi")]);
        let plugins = scan(&root);
        assert!(plugins.is_empty());
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn zip_slip_rejected() {
        let root = temp_root("slip");
        fs::create_dir_all(&root).unwrap();
        // `../`, backslash `..\`, and drive-letter escapes must all be rejected
        for (name, entries) in [
            (
                "evil.zip",
                vec![
                    ("module.json", br#"{"id":"evil","main":"libevil.so"}"#.as_slice()),
                    ("../evil.so", b"x".as_slice()),
                ],
            ),
            (
                "evil2.zip",
                vec![
                    ("module.json", br#"{"id":"evil2","main":"libevil2.so"}"#.as_slice()),
                    (r"..\evil2.so", b"x".as_slice()),
                ],
            ),
            (
                "evil3.zip",
                vec![
                    ("module.json", br#"{"id":"evil3","main":"libevil3.so"}"#.as_slice()),
                    (r"C:\evil3.dll", b"x".as_slice()),
                ],
            ),
        ] {
            write_test_zip(&root.join(name), &entries);
            assert!(scan(&root).is_empty(), "archive {name} must be rejected");
        }
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn zip_plugin_disabled() {
        let root = temp_root("disabled");
        fs::create_dir_all(&root).unwrap();
        write_test_zip(
            &root.join("old.zip.disabled"),
            &[
                ("module.json", br#"{"id":"old","main":"libold.so"}"#),
                ("libold.so", b"x"),
            ],
        );
        assert!(scan(&root).is_empty());
        fs::remove_dir_all(&root).unwrap();
    }
}
