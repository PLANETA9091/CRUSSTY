# CRUSSTY — native c-plugin platform (v2)

> **Docs:** [planeta9091.github.io/CRUSSTY](https://planeta9091.github.io/CRUSSTY/)
> (quickstart, module API, SDK reference, troubleshooting)
>
> **Download:** ready-to-run artifacts — `crussty-linux-x64` (single-jar),
> `launcher.jar`, `libcrussty_runtime.so`, run scripts — in
> [Releases](https://github.com/PLANETA9091/CRUSSTY/releases). Build from
> source only if you need to hack on the runtime itself.

Injects native Rust modules into any Paper-compatible kernel: a JVMTI runtime
with a ClassFileLoadHook hot-patch pipeline. A module is a plugin: a directory
(or `.zip`/`.jar` archive) with a `cplugin.json` manifest and an entry library.

Two distribution paths:

- **Single-jar (recommended)** — `dist/crussty-<ver>.jar` is a drop-in
  `server.jar`: extract `libcrussty_runtime.so` + all modules, `System.load`
  the runtime (JNI_OnLoad), then boot the kernel. Run with plain
  `java -jar server.jar --nogui` — no `-agentpath` needed.
- **Launcher** — `launcher.jar` spawns the kernel with `-agentpath` and tees
  logs.

## Layout

- `cplug-abi/` — the only contract between runtime and modules
- `cplug-sdk/` — SDK for module authors (hooks, JNI, main thread, ASM weaving)
- `runtime/` — JVMTI runtime: recursive scan, topological loading, hook pipeline
  - `runtime/src/platform/` — 12 native platform bricks modules build on
    (barriers, events, hot_reload, network, save_events, scheduler,
    side_table, signals, storage, telemetry, threads, transform)
- `launcher/` — launcher + single-jar bootstrapper (`Boot.java`)
- `scripts/` — `build-single-jar.sh`, `e2e.sh` (E2E smoke test),
  `gen_crussty_table.py` (JNI bridge table generator, see below)
- `modules/` — bundled modules: `cells`, `crussty`, `dist`, `hello`
  - `modules/crussty/native/` — published Crussty CE native libraries
    (283 JNI exports) with `JNI_EXPORTS.manifest` + `MANIFEST.md` (MIT)
- `docs/V2-DESIGN.md` — platform design
- `book/` — user documentation: https://planeta9091.github.io/CRUSSTY/

## Build

**Requires Java 21+** (for the kernel and `javac`/`jar` in the helper and
launcher builds). **Requires Rust** (stable toolchain). 

```bash
cargo build --manifest-path runtime/Cargo.toml
cp runtime/target/debug/libcrussty_runtime.so libcrussty_runtime.so
./scripts/build-single-jar.sh        # -> dist/crussty-<ver>.jar
```

Requires `versions/purpur-1.21.10.jar` (not committed) — the single-jar boot
loads the kernel from there, so it must be in place **before** running
`build-single-jar.sh` or booting `server.jar`.

## Run (single-jar)

```bash
# the kernel jar must already be in versions/ (see Build above):
#   mkdir -p versions && cp /path/to/purpur-1.21.10.jar versions/
cp dist/crussty-1.21.10.jar server.jar
echo "eula=true" > eula.txt
java -Xmx2G -jar server.jar --nogui
```

Expect in the log: `pipeline ready: 3 plugin hook(s)` and
`hello from native c-plugin`.

## E2E smoke test

```bash
./scripts/e2e.sh
```

Downloads `purpur-1.21.10` into `versions/` (cached), builds the runtime,
launcher and bundled modules, boots a real server via `run.sh` and greps the
log for `pipeline ready` and `hello from native c-plugin`. Exit code 0 only
if both markers appear within the timeout; every stage is logged. Env:
`PURPUR_VERSION`, `SERVER_PORT`, `TIMEOUT_SEC`, `MODULES`.

## Crussty CE native libraries (modules/crussty)

The `crussty` module injects the full Crussty CE native surface (283 JNI
exports) into any Paper-family kernel. The binaries are published in
`modules/crussty/native/` (MIT — see `MANIFEST.md` there):
`libpaper_native_jni.so` is required at runtime (the module logs
`missing …` and skips injection otherwise), `libpaper_native_chunk_encode_jni.so`
is optional. The bridge table `modules/crussty/src/jni_table.rs` is generated
from `native/JNI_EXPORTS.manifest` (single source of truth — never edit the
.rs by hand):

```bash
python3 scripts/gen_crussty_table.py render        # manifest -> jni_table.rs
python3 scripts/gen_crussty_table.py render --check  # CI: fail if out of sync
python3 scripts/gen_crussty_table.py verify        # cross-check against shipped .so
```

`scripts/build-single-jar.sh` embeds the `native/` dir into the jar
automatically.

## Platform bricks

The runtime ships twelve primitives under `runtime/src/platform/` — the
building blocks every module uses instead of reinventing them:

| Brick | File | What it gives |
|---|---|---|
| transform | `transform.rs` | ClassFileLoadHook patch pipeline (bytecode rewriting) |
| events | `events.rs` | async pub/sub event bus with backpressure |
| signals | `signals.rs` | crash isolation: native backtrace + fault stats, chains the JVM's own handlers |
| network | `network.rs` | connection registry, hooks, traffic counters |
| scheduler | `scheduler.rs` | tick routing on the kernel main thread |
| storage | `storage.rs` | key-value persistence for modules |
| side_table | `side_table.rs` | O(1) metadata beside kernel objects |
| hot_reload | `hot_reload.rs` | swap a module's library without a server restart |
| threads | `threads.rs` | managed worker threads |
| barriers | `barriers.rs` | multi-phase sync between module threads |
| telemetry | `telemetry.rs` | metrics + UDP export |
| save_events | `save_events.rs` | world-save lifecycle hooks |

Signal handlers chain to whatever the JVM had installed (`sigaction`,
`SA_SIGINFO`): the JVM's own SIGSEGV handling (hs_err, JIT null checks) is
never clobbered. Disable with `CRUSSTY_NO_SIGNALS=1` for diagnostics.

## Module API

A module is a cdylib exporting `cplugin_init(api, vm, options)`, where `api`
is the `cplug-abi` struct of function pointers. See
`book/src/manifest.md` and `book/src/sdk.md` for the full contract.
