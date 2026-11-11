# CRUSSTY — inject native code into your server jar

CRUSSTY is a **jar injector**: it drops a native runtime into any
Minecraft server jar — Paper, Purpur, Fabric, Forge, vanilla, any `.jar` —
and runs modules written in Rust, C, C++, Python or JavaScript — hooking
the server at the bytecode level. No plugin API locks, no JNI boilerplate,
no `-agentpath`.

> **Documentation:** [planeta9091.github.io/CRUSSTY](https://planeta9091.github.io/CRUSSTY/)
> — quickstart, module API, SDK reference, other languages, troubleshooting.
>
> **Downloads:** the `crussty` CLI on
> [npm](https://www.npmjs.com/package/crussty) (Linux, macOS, Windows) —
> it republishes automatically on every push that touches the CLI, so
> `npm i -g crussty` / `npm update -g crussty` always gets the latest build —
> plus `launcher.jar`, `libcrussty_runtime.so` and run scripts in
> [Releases](https://github.com/PLANETA9091/CRUSSTY/releases).

![CRUSSTY demo](assets/demo.gif)

## Quick start (5 minutes)

You only need the `crussty` CLI from npm and a Java 21+ runtime:

```bash
npm i -g crussty

# 1. scaffold a server directory (runtime + launcher, plus a kernel of
#    your choice — Paper, Purpur, Fabric or Forge)
crussty init --dir my-server

# 2. start the server (console is forwarded to your terminal)
cd my-server && crussty run
```

In an interactive terminal `init` asks what to inject into:

```text
crussty: choose the kernel to inject into:
  1. Paper — pick a version and download
  2. Purpur — pick a version and download
  3. Fabric — pick a version and download
  4. Forge — pick a version and download
  5. Skip — I'll place my own server jar
```

Pick a platform and it fetches the live version list from the platform's
own API; pick a version and it downloads automatically. Forge additionally
runs its installer (`--installServer`) for you.

No kernel is downloaded without your choice — non-interactive runs just
print where to drop the jar. Alternative ways to provide the kernel:

```bash
crussty init --jar /path/to/server.jar              # use any existing .jar
crussty init --platform purpur --version 1.21.10    # fully non-interactive
crussty init --no-kernel                            # instructions only
```

That's it — the runtime boots with the kernel and loads whatever is in
`modules/`. Expect `pipeline ready: N module hook(s)` in the log.

What's in your server directory:

```
my-server/
├── crussty.toml          # kernel + memory + module catalog settings
├── versions/<kernel>.jar # any Minecraft server jar (your injection target)
├── launcher/launcher.jar # spawns the kernel with the JVMTI runtime
├── libcrussty_runtime.so # the runtime itself
├── modules/              # drop a module here to load it
├── logs/                 # server + runtime logs
└── crus/                 # data written by modules
```

### Managing modules

```bash
crussty ls        # list modules: active / parked / disabled
crussty search <query>   # find modules on GitHub (repos with module.json)
crussty install <name>   # install from the module catalog
crussty install <owner/repo>  # install straight from a GitHub repo
crussty enable <name>    # activate a parked/disabled module
crussty disable <name>   # park it (or --disabled to disable)
crussty reload    # hot-reload all modules (no server restart)
```

### One jar for everything: `crussty build`

Instead of carrying the launcher + the kernel + `modules/` around, pack
the whole server into a single bootable jar and drop it anywhere a JVM
exists:

```bash
cd my-server && crussty build
java -Xmx2G -jar dist/crussty-<your-kernel>.jar --nogui
```

The jar carries the kernel unmodified, the native runtime and every
installed module; on first boot it unpacks `./crussty/` and starts the
kernel. Works with Paper, Purpur, Fabric and vanilla kernels (Forge boots
from its libraries and cannot be packed).

### The TUI

`crussty tui` opens a full-screen menu: new module, build,
auto-rebuild on file change, pack, GitHub search with one-keystroke
install — and every action shows its output in the window.

## What is a module?

A module is a directory (or `.zip`/`.jar` archive) with a
`module.json` manifest and an entry library exporting
`cplugin_init(api, vm, options)`. It can hook class loading, patch
bytecode, run code on the server's main thread, and use the platform's
twelve native bricks (events, storage, hot reload, …) — see
[the docs](https://planeta9091.github.io/CRUSSTY/) for the full contract
and the [`hello` module](https://github.com/PLANETA9091/c-hello) for the
smallest working example.

Modules are not limited to Rust: the
[`cplug-sdk-c`](cplug-sdk-c/) binding exposes the same platform to C, C++,
Python and JavaScript — see
[Creating a module](https://planeta9091.github.io/CRUSSTY/modules/creating.html).

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
`build-single-jar.sh` or booting `server.jar`. The user-facing variant is
`crussty build` (any kernel from `versions/`, modules from the server's own
`modules/` — see `book/src/cli.md`).

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
- `modules/` — install location for modules (they live in their own repos:
  `c-hello`, `c-dist`, `c-crussty`, `c-cells`, `c-moduleslist`, plus the
  `multilang` branch of `c-hello` for c/cpp/go/js/python examples;
  see `modules/README.md`)
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

## Crussty CE native libraries (c-crussty)

The `crussty` module (repository
[`PLANETA9091/c-crussty`](https://github.com/PLANETA9091/c-crussty))
injects the full Crussty CE native surface (283 JNI exports) into any
Paper-family kernel. The binaries are published in its `native/` directory
(MIT — see `MANIFEST.md` there): `libpaper_native_jni.so` is required at
runtime (the module logs `missing …` and skips injection otherwise),
`libpaper_native_chunk_encode_jni.so` is optional. The bridge table
`modules/crussty/src/jni_table.rs` is generated from `native/JNI_EXPORTS.manifest`
(single source of truth — never edit the .rs by hand):

```bash
python3 scripts/gen_crussty_table.py render        # manifest -> jni_table.rs
python3 scripts/gen_crussty_table.py render --check  # CI: fail if out of sync
python3 scripts/gen_crussty_table.py verify        # cross-check against shipped .so
```

`scripts/build-single-jar.sh` embeds the `native/` dir into the jar
automatically.

## Star history

If CRUSSTY is useful to you, drop a star on GitHub — it's what keeps this
project alive.

[![GitHub stars](https://img.shields.io/github/stars/PLANETA9091/CRUSSTY?style=flat-square&label=stars)](https://github.com/PLANETA9091/CRUSSTY/stargazers)

<a href="https://www.star-history.com/?repos=PLANETA9091%2FCRUSSTY&type=date&legend=top-left">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/chart?repos=PLANETA9091/CRUSSTY&type=date&theme=dark&legend=top-left&sealed_token=4j7fLgibkeg94EJ6EC2dGfYc0Fl3zQioHSbgYmerex0GvFibN3mDp31CoKqxKt91sFGtoD5n99YGxtZ2nVRPTmmcguyICx5RvferDxO2Wckvoy-Dp8REOXnzwAnRrVFAYRX5SEYz3cfmBppauWEeo_dvLQQlWW006JKFFKUjzcOqEBmbdjlRKKdSCYhY" />
   <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/chart?repos=PLANETA9091/CRUSSTY&type=date&legend=top-left&sealed_token=4j7fLgibkeg94EJ6EC2dGfYc0Fl3zQioHSbgYmerex0GvFibN3mDp31CoKqxKt91sFGtoD5n99YGxtZ2nVRPTmmcguyICx5RvferDxO2Wckvoy-Dp8REOXnzwAnRrVFAYRX5SEYz3cfmBppauWEeo_dvLQQlWW006JKFFKUjzcOqEBmbdjlRKKdSCYhY" />
   <img alt="Star History Chart" src="https://api.star-history.com/chart?repos=PLANETA9091/CRUSSTY&type=date&legend=top-left&sealed_token=4j7fLgibkeg94EJ6EC2dGfYc0Fl3zQioHSbgYmerex0GvFibN3mDp31CoKqxKt91sFGtoD5n99YGxtZ2nVRPTmmcguyICx5RvferDxO2Wckvoy-Dp8REOXnzwAnRrVFAYRX5SEYz3cfmBppauWEeo_dvLQQlWW006JKFFKUjzcOqEBmbdjlRKKdSCYhY" />
 </picture>
</a>
