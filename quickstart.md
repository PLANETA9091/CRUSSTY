---
title: Quick Start
nav_order: 2
---

# <img class="page-icon" src="/CRUSSTY/assets/images/icons/bolt.svg" alt=""> Quick Start

## Requirements

- Rust (stable toolchain, tested on 1.8x+)
- Java 21+ (JDK for `javac`/`jar`)
- A Paper-family kernel jar, e.g. Purpur 1.21.10

## Build the platform

```bash
# JVMTI runtime
cargo build --manifest-path runtime/Cargo.toml
cp runtime/target/debug/libcrussty_runtime.so libcrussty_runtime.so

# single-jar distribution (recommended) -> dist/crussty-<ver>.jar
./scripts/build-single-jar.sh
```

## Install modules

Clone example modules into `modules/`:

```bash
git clone https://github.com/PLANETA9091/c-hello modules/hello
cd modules/hello && cargo build && cp target/debug/libhello.so libhello.so
```

Or drop a module zip (see [Distribution](./modules/zip.html)) into `modules/`
— the runtime extracts and loads it.

## Run — single-jar (recommended)

```bash
echo "eula=true" > eula.txt
cp dist/crussty-1.21.10.jar server.jar
java -Xmx2G -jar server.jar --nogui
```

The jar boots the kernel itself: no `-agentpath`, no launcher process.

## Verify

Check the server log:

```
[crussty-runtime] plugin hello -> init rc=0
[hello-plugin] cplugin_init (native, before kernel boot)
[crussty-runtime] pipeline ready: 3 plugin hook(s)
[13:36:26 INFO]: hello from native c-plugin (v2 pipeline alive)
```

## Env options

Single-jar runs read `crussty/options.txt` (written by the bootstrapper) or
`CRUSSTY_RUNTIME_OPTIONS`. Modules read their own config from env variables
(e.g. `CRUSSTY_NATIVE_IMPROVED_NOISE=1`). `CRUSSTY_NO_SIGNALS=1` disables the
platform's crash handlers — use it for diagnostics.
