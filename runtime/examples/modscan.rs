//! modscan — the multi-module discovery proof: scan a REAL modules/ tree
//! the same way the runtime does at boot and print the load order.
//!
//! Usage: modscan <modules-dir>
//!
//! Verifies, per directory: the manifest parses, the entry library exists,
//! per-module `options` reach the CPlugin, the topological order puts
//! dependencies first. This is the scan half of the boot; the dlopen+init
//! half needs a live JVM (see scripts/e2e.sh).

use crussty_runtime::scan::scan;

fn main() {
    let dir = std::env::args().nth(1).expect("usage: modscan <modules-dir>");
    let plugins = scan(std::path::Path::new(&dir));
    println!("modscan: {} module(s) in load order:", plugins.len());
    for p in &plugins {
        println!(
            "  {:<10} lib={} deps={:?} options={}",
            p.id,
            p.lib_path.display(),
            p.deps,
            p.options.as_deref().unwrap_or("<global>")
        );
    }
    if plugins.is_empty() {
        eprintln!("modscan: no modules found under {dir}");
        std::process::exit(1);
    }
    println!("modscan: OK");
}
