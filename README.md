# CRUSSTY — native c-plugin platform (v2)

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
- `scripts/` — `build-single-jar.sh`, `gen_crussty_table.py`
- `modules/` — bundled modules: `cells`, `crussty`, `dist`, `hello`
- `docs/V2-DESIGN.md` — platform design
- `book/` — user documentation: https://planeta9091.github.io/CRUSSTY/

## Build

```bash
cargo build --manifest-path runtime/Cargo.toml
cp runtime/target/debug/libcrussty_runtime.so libcrussty_runtime.so
./scripts/build-single-jar.sh        # -> dist/crussty-<ver>.jar
```

Requires `versions/purpur-1.21.10.jar` (not committed).

## Run (single-jar)

```bash
cp dist/crussty-1.21.10.jar server.jar
echo "eula=true" > eula.txt
java -Xmx2G -jar server.jar --nogui
```

Expect in the log: `pipeline ready: 3 plugin hook(s)` and
`hello from native c-plugin`.

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
