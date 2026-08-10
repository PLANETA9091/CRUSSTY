---
title: Quick Start
nav_order: 2
---

# <img class="page-icon" src="/CRUSSTY/assets/images/icons/bolt.svg" alt=""> Quick Start

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
crussty search nbt                    # find modules on GitHub (repos with module.json)
crussty install hello                 # install from the catalog
crussty install PLANETA9091/c-hello   # or straight from a repo
crussty ls                            # active / parked / disabled
```

Write your own module while the server runs — `crussty tui` gives you a
full-screen menu: new module, build, auto-rebuild on file change, pack,
GitHub search. Try [Creating a module](./modules/creating.html) or just
`crussty module new hello`.

## Building the platform from source

You don't need to build anything — prebuilt `launcher.jar`,
`libcrussty_runtime.so` and run scripts are attached to every
[GitHub release](https://github.com/PLANETA9091/CRUSSTY/releases), and
`crussty init` downloads them for you. Building from source is for hacking
on the platform itself (runtime, launcher, SDK). Requirements: Rust
(stable) and a Java 21+ JDK.

```bash
# JVMTI runtime
cargo build --manifest-path runtime/Cargo.toml
cp runtime/target/debug/libcrussty_runtime.so libcrussty_runtime.so

# single-jar distribution (recommended) -> dist/crussty-<ver>.jar
./scripts/build-single-jar.sh
```

Run what you built (modules still go into `modules/` — `crussty install hello`
or drop a bundle in by hand):

```bash
echo "eula=true" > eula.txt
cp dist/crussty-1.21.10.jar server.jar
java -Xmx2G -jar server.jar --nogui
```

## Verify

Check the server log:

```
[crussty-runtime] module hello -> init rc=0
[hello-module] cplugin_init (native, before kernel boot)
[crussty-runtime] pipeline ready: 3 module hook(s)
[13:36:26 INFO]: hello from native c-plugin (v2 pipeline alive)
```

## Env options

Single-jar runs read `crussty/options.txt` (written by the bootstrapper) or
`CRUSSTY_RUNTIME_OPTIONS`. Modules read their own config from env variables
(e.g. `CRUSSTY_NATIVE_IMPROVED_NOISE=1`). `CRUSSTY_NO_SIGNALS=1` disables the
platform's crash handlers — use it for diagnostics.
