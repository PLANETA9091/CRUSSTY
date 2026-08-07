# CRUSSTY — native c-plugin platform (v2)

Injects native Rust modules into any Paper-compatible kernel: a JVMTI runtime
with a ClassFileLoadHook hot-patch pipeline. A module is a plugin: a directory
(or `.zip`/`.jar` archive) with a `cplugin.json` manifest and an entry library.

## Layout

- `cplug-abi/` — the only contract between runtime and modules
- `cplug-sdk/` — SDK for module authors (hooks, JNI, main thread, ASM weaving)
- `runtime/` — JVMTI runtime: recursive scan, topological loading, hook pipeline
- `launcher/` — launcher (spawns the kernel with `-agentpath`)
- `modules/` — plugins live in their own `c-<name>` repos; clone them here
- `docs/V2-DESIGN.md` — platform design
- `book/` — user documentation: https://planeta9091.github.io/CRUSSTY/

## Build

```bash
cargo build --manifest-path runtime/Cargo.toml
cp runtime/target/debug/libcrussty_runtime.so libcrussty_runtime.so
javac -d launcher/out launcher/src/main/java/dev/dist/launcher/Main.java && \
  jar cfe launcher/launcher.jar dev.dist.launcher.Main -C launcher/out .
./run.sh
```

Requires `versions/purpur-1.21.10.jar` (not committed).
