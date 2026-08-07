# Quick Start

## Requirements

- Rust (stable toolchain, tested on 1.8x+)
- Java 21+ (JDK for `javac`/`jar`)
- A Paper-family kernel jar, e.g. Purpur 1.21.10

## Build the platform

```bash
# JVMTI runtime
cargo build --manifest-path runtime/Cargo.toml
cp runtime/target/debug/libcrussty_runtime.so libcrussty_runtime.so

# launcher
javac -d launcher/out launcher/src/main/java/dev/dist/launcher/Main.java
jar cfe launcher/launcher.jar dev.dist.launcher.Main -C launcher/out .
```

## Install plugins

Clone example plugins into `modules/`:

```bash
git clone https://github.com/PLANETA9091/c-hello modules/hello
cd modules/hello && cargo build && cp target/debug/libhello.so libhello.so
```

Or drop a plugin zip (see [Distribution](./zip.md)) into `modules/` — the
runtime extracts and loads it.

## Run

```bash
mkdir -p versions
cp /path/to/purpur-1.21.10.jar versions/
# accept the EULA on first boot
echo "eula=true" > eula.txt

./run.sh
```

Verify in the server log:

```
[crussty-runtime] plugin hello -> init rc=0
[hello-plugin] cplugin_init (native, before kernel boot)
[13:36:26 INFO]: hello from native c-plugin (v2 pipeline alive)
```

## Env options

The runtime reads its options from the launcher: `modules=<dir>`,
`versions=<dir>`, `kernel=<jar name>`. Plugins read their own config from env
variables (e.g. `CRUSSTY_NATIVE_IMPROVED_NOISE=1`).
