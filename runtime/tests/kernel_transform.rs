//! Regression fixtures against REAL kernel classes (BUG-4 in BUG_REPORTS.md).
//!
//! Both crash classes found by e2e — the StackMapTable mis-parse on
//! MinecraftServer/ServerLevel/RegionFileStorage and the missing hook
//! classes — were invisible to all unit tests. This test re-runs the exact
//! platform rule set over the actual purpur-1.21.10 jar (the one downloaded
//! by scripts/e2e.sh into versions/) and asserts the engine never fails on a
//! kernel class. Deterministic, cheap, and it catches both past bug classes.
//!
//! Skips (prints a note, test passes) when the jar is absent, so `cargo
//! test` still works on a fresh clone without network.

use std::io::Read;

const KERNEL: &str = "purpur-1.21.10";

/// Kernel classes the platform transform rules have ever matched, in
/// package-local names as they appear in the jar.
const TARGETS: &[&str] = &[
    "net/minecraft/server/MinecraftServer",
    "net/minecraft/server/level/ServerLevel",
    "net/minecraft/world/level/chunk/storage/RegionFileStorage",
    "ca/spottedleaf/moonrise/patches/chunk_system/io/MoonriseRegionFileIO",
    "net/minecraft/network/PacketDecoder",
    "net/minecraft/network/PacketEncoder",
    "net/minecraft/network/Connection",
    "net/minecraft/server/network/ServerHandshakePacketListenerImpl",
    "org/bukkit/craftbukkit/scheduler/CraftScheduler",
];

fn kernel_jar() -> Option<std::path::PathBuf> {
    // e2e.sh downloads into <repo>/versions/. Probe the repo root first
    // (integration tests run with CWD = runtime/), then CARGO_MANIFEST_DIR.
    // e2e.sh stores the fetched kernel at <repo>/versions/<ver>/<kernel>.jar;
    // some local setups also keep a copy flat at <repo>/versions/.
    let mut p1 = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p1.pop();
    p1.push("versions");
    p1.push("1.21.10");
    p1.push(format!("{KERNEL}.jar"));
    let mut p2 = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p2.pop();
    p2.push("versions");
    p2.push(format!("{KERNEL}.jar"));
    let _ = p2;
    for probe in [p1, p2] {
        let _ = probe;
        if probe.exists() {
            return Some(probe);
        }
    }
    None
}

#[test]
fn kernel_classes_transform_cleanly() {
    let Some(jar) = kernel_jar() else {
        eprintln!(
            "[kernel-transform] jar absent — skipping (run scripts/e2e.sh first); probed versions/ and {}",
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("versions").display()
        );
        return;
    };
    crussty_runtime::platform::network::install_default_rules();
    crussty_runtime::platform::scheduler::install_default_rules();
    let _ = crussty_runtime::platform::storage::install_default_rules();

    let file = std::fs::File::open(&jar).expect("open kernel jar");
    let mut archive = zip::ZipArchive::new(file).expect("parse kernel jar");
    let engine = crussty_runtime::platform::transform::global_engine();

    for target in TARGETS {
        let name = format!("{target}.class");
        let mut entry = match archive.by_name(&name) {
            Ok(e) => e,
            Err(_) => {
                eprintln!("[kernel-transform] {name} not in jar — skipped");
                continue;
            }
        };
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).expect("read class bytes");
        // The engine must never Err on a real kernel class; Ok(None) is
        // fine for targets no current rule matches.
        match engine.apply(target, &bytes) {
            Ok(t) => eprintln!("[kernel-transform] {target}: transformed={}", t.is_some()),
            Err(e) => panic!("[kernel-transform] {target}: transform failed: {e}"),
        }
    }
}