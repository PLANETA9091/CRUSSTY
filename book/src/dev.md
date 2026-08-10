# Building from source

**You don't need to build anything.** Prebuilt platform artifacts
(`launcher.jar`, `libcrussty_runtime.so`, `run.sh`, `run.bat`) are attached
to every [GitHub release](https://github.com/PLANETA9091/CRUSSTY/releases),
and `crussty init` downloads them automatically along with the kernel:

```bash
npm i -g crussty
crussty init --dir my-server
```

Building from source is only for hacking on the platform itself — the
runtime, the launcher, the SDK. You need Rust (stable) and a Java 21+ JDK.

## Requirements

- Rust (stable toolchain, tested on 1.8x+)
- Java 21+ (JDK for `javac`/`jar`) — the kernel needs 21+, but the platform's
  committed Java helper classes are compiled with `--release 8` (class-file
  target 52), so any modern JDK can rebuild them

## Build the platform

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

## Run what you built

For a launcher-style run, point `run.sh` at the built artifacts:

```bash
mkdir -p versions
cp /path/to/purpur-1.21.10.jar versions/
# accept the EULA on first boot
echo "eula=true" > eula.txt

./run.sh
```

For a single-jar run (no launcher process, no `-agentpath`):

```bash
cp dist/crussty-1.21.10.jar server.jar
java -Xmx2G -jar server.jar --nogui
```

The jar boots the kernel itself. Either way, modules go into `modules/` —
install them with `crussty install hello` (see
[Quick Start](./quickstart.md)) or drop a bundle in by hand
([Distribution](./zip.md)).

## E2E smoke test

```bash
./scripts/e2e.sh
```

Downloads the kernel into `versions/` (cached), builds runtime, launcher and
the bundled modules, boots a real server via `run.sh` and waits for the
markers instead of you watching the log:

```text
[crussty-runtime] pipeline ready: N hook(s)
[hello-module] ... hello from native c-plugin
```

Exit code 0 only if both appear before the timeout; stages are logged.
Environment knobs: `PURPUR_VERSION`, `SERVER_PORT`, `TIMEOUT_SEC`,
`MODULES` (default `hello dist crussty`).

## Verify

Check the server log:

```text
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
