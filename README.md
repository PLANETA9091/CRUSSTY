# CRUSSTY — inject native code into your server jar

CRUSSTY is a **jar injector**: it drops a native runtime into any
Paper-compatible server jar and runs modules written in Rust, C, C++,
Python or JavaScript — hooking the server at the bytecode level. No plugin
API locks, no JNI boilerplate, no `-agentpath`.

> **Documentation:** [planeta9091.github.io/CRUSSTY](https://planeta9091.github.io/CRUSSTY/)
> — quickstart, module API, SDK reference, other languages, troubleshooting.
>
> **Downloads:** prebuilt `crussty-linux-x64` CLI, `launcher.jar`,
> `libcrussty_runtime.so` and run scripts in
> [Releases](https://github.com/PLANETA9091/CRUSSTY/releases).

## Quick start (5 minutes)

You only need the `crussty-linux-x64` binary from the
[latest release](https://github.com/PLANETA9091/CRUSSTY/releases/latest)
and a Java 21+ runtime.

```bash
# 1. scaffold a server directory (downloads the kernel, runtime and launcher)
crussty-linux-x64 init --dir my-server

# 2. start the server (console is forwarded to your terminal)
cd my-server && crussty-linux-x64 run
```

That's it — the runtime boots with the kernel and loads whatever is in
`modules/`. Expect `pipeline ready: N module hook(s)` in the log.

What's in your server directory:

```
my-server/
├── crussty.toml          # kernel + memory + module catalog settings
├── versions/purpur-1.21.10.jar
├── launcher/launcher.jar # spawns the kernel with the JVMTI runtime
├── libcrussty_runtime.so # the runtime itself
├── modules/              # drop a module here to load it
├── logs/                 # server + runtime logs
└── crus/                 # data written by modules
```

### Managing modules

```bash
crussty-linux-x64 ls        # list modules: active / parked / disabled
crussty-linux-x64 install <name>   # install from the module catalog
crussty-linux-x64 enable <name>    # activate a parked/disabled module
crussty-linux-x64 disable <name>   # park it (or --disabled to disable)
crussty-linux-x64 reload    # hot-reload all modules (no server restart)
```

## What is a module?

A module is a directory (or `.zip`/`.jar` archive) with a
`cplugin.json` manifest and an entry library exporting
`cplugin_init(api, vm, options)`. It can hook class loading, patch
bytecode, run code on the server's main thread, and use the platform's
twelve native bricks (events, storage, hot reload, …) — see
[the docs](https://planeta9091.github.io/CRUSSTY/) for the full contract
and `modules/hello` in this repo for the smallest working example.

Modules are not limited to Rust: the
[`cplug-sdk-c`](cplug-sdk-c/) binding exposes the same platform to C, C++,
Python and JavaScript — see
[other languages](https://planeta9091.github.io/CRUSSTY/modules/creating/other-languages.html).

## Building from source

For hacking on the runtime itself. **Requires Java 21+** and a stable Rust
toolchain.

```bash
cargo build --manifest-path runtime/Cargo.toml
cp runtime/target/debug/libcrussty_runtime.so libcrussty_runtime.so
./scripts/build-single-jar.sh        # -> dist/crussty-<ver>.jar
```

Requires `versions/purpur-1.21.10.jar` (not committed) — the single-jar boot
loads the kernel from there, so it must be in place **before** running
`build-single-jar.sh` or booting `server.jar`.

```bash
# run the built single-jar
cp dist/crussty-1.21.10.jar server.jar
echo "eula=true" > eula.txt
java -Xmx2G -jar server.jar --nogui
```

### E2E smoke test

```bash
./scripts/e2e.sh
```

Downloads `purpur-1.21.10` into `versions/` (cached), builds the runtime,
launcher and bundled modules, boots a real server via `run.sh` and greps the
log for `pipeline ready` and `hello from native c-plugin`. Exit code 0 only
if both markers appear within the timeout; every stage is logged. Env:
`PURPUR_VERSION`, `SERVER_PORT`, `TIMEOUT_SEC`, `MODULES`.

## Repository layout

- `cplug-abi/` — the only contract between runtime and modules
- `cplug-sdk/` — Rust SDK for module authors (hooks, JNI, main thread, ASM)
- `cplug-sdk-c/` — C binding: the same platform for C/C++/Python/JS modules
- `runtime/` — JVMTI runtime: recursive scan, topological loading, hook pipeline
  - `runtime/src/platform/` — the 12 native platform bricks (see table below)
- `launcher/` — launcher + single-jar bootstrapper (`Boot.java`)
- `modules/` — bundled modules: `cells`, `crussty`, `dist`, `hello`
- `scripts/` — `build-single-jar.sh`, `e2e.sh`, `gen_crussty_table.py`
- `docs/V2-DESIGN.md` — platform design
- `book/` — user documentation source (published to GitHub Pages)

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
