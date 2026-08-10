# Quick Start

## Fastest path — the CLI (5 minutes)

You only need the `crussty` CLI from npm and a Java 21+ runtime:

```bash
npm i -g crussty
```

Scaffold a server directory (this downloads the Purpur kernel, the native
runtime and the launcher for you):

```bash
crussty init --dir my-server
cd my-server
```

Start the server — the console is forwarded to your terminal, `stop` shuts
it down cleanly:

```bash
crussty run
```

Install modules without leaving the terminal:

```bash
crussty search nbt       # find modules on GitHub (repos with module.json)
crussty install hello    # install from the catalog
crussty install PLANETA9091/c-hello   # or straight from a repo
crussty ls               # see active / parked / disabled modules
```

Write your own module while the server runs — `crussty tui` gives you a
menu: scaffold, build, auto-rebuild on file change, pack. See
[Your first module](./first-module.md).

## Building the platform from source

If you want to hack on the runtime itself, clone the repository. You need
Rust (stable) and a Java 21+ JDK.

The Rust crates embed pre-built Java helper classes (committed); a fresh
clone builds with zero Java toolchain. After editing a helper source, or for
CI parity, regenerate and commit the artifacts:

```bash
# cplug-sdk SdkAsmHelper, modules/crussty area-map + improved-noise bridges,
# modules/dist DistKernel -> .class into build dirs (ASM jar auto-downloaded)
bash scripts/build-helpers.sh
```

```bash
# JVMTI runtime
cargo build --manifest-path runtime/Cargo.toml
cp runtime/target/debug/libcrussty_runtime.so libcrussty_runtime.so

# single-jar distribution (recommended) -> dist/crussty-<ver>.jar
./scripts/build-single-jar.sh

# launcher (alternative to single-jar; spawns the kernel with -agentpath)
javac -encoding UTF-8 -d launcher/out launcher/src/main/java/dev/dist/launcher/Main.java
jar cfe launcher/launcher.jar dev.dist.launcher.Main -C launcher/out .
```

## Install modules

Clone example modules into `modules/`:

```bash
git clone https://github.com/PLANETA9091/c-hello modules/hello
cd modules/hello && cargo build && cp target/debug/libhello.so libhello.so
```

Or drop a module zip (see [Distribution](./zip.md)) into `modules/` — the
runtime extracts and loads it.

## Run — single-jar (recommended)

```bash
mkdir -p versions
cp /path/to/purpur-1.21.10.jar versions/
# accept the EULA on first boot
echo "eula=true" > eula.txt

cp dist/crussty-1.21.10.jar server.jar
java -Xmx2G -jar server.jar --nogui
```

The jar boots the kernel itself: no `-agentpath`, no launcher process.

## Run — launcher

```bash
./run.sh
```

## E2E smoke test

```bash
./scripts/e2e.sh
```

Downloads the kernel into `versions/` (cached), builds runtime, launcher and
the bundled modules, boots a real server via `run.sh` and waits for the
markers instead of you watching the log:

```
[crussty-runtime] pipeline ready: N hook(s)
[hello-module] ... hello from native c-plugin
```

Exit code 0 only if both appear before the timeout; stages are logged.
Environment knobs: `PURPUR_VERSION`, `SERVER_PORT`, `TIMEOUT_SEC`,
`MODULES` (default `hello dist crussty`).

## Verify

Check the server log:

```
[crussty-runtime] module hello -> init rc=0
[hello-module] cplugin_init (native, before kernel boot)
[crussty-runtime] pipeline ready: 3 module hook(s)
[13:36:26 INFO]: hello from native c-plugin (v2 pipeline alive)
```

## Env options

The runtime reads its options from the launcher: `modules=<dir>`,
`versions=<dir>`, `kernel=<jar name>`. Single-jar runs read
`crussty/options.txt` (written by the bootstrapper) or
`CRUSSTY_RUNTIME_OPTIONS`. Modules read their own config from env variables
(e.g. `CRUSSTY_NATIVE_IMPROVED_NOISE=1`).

`CRUSSTY_NO_SIGNALS=1` disables the platform's crash handlers — use it for
diagnostics when a native crash report is suspected.
