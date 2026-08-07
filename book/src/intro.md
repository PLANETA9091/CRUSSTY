# Introduction

Crussty is a native plugin platform for Paper-family Minecraft kernels
(Purpur, Paper, etc.). Plugins — called **c-plugins** or **modules** — are
written in **Rust** and loaded into the server process as native shared
libraries.

## What it does

The Crussty Runtime (`libcrussty_runtime.so`) is attached to the kernel JVM with
`-agentpath`. It:

- scans a `modules/` directory for plugins,
- loads each plugin (`dlopen`, `RTLD_LOCAL`) in dependency order,
- forwards every class load through the plugin hook pipeline
  (`CLASS_FILE_LOAD_HOOK`), so plugins can patch kernel bytecode on the fly
  (hot-patching),
- lets plugins retransform already-loaded classes.

Plugins get a `JavaVM*` and can do anything JNI/JVMTI allows: resolve classes
across loaders, run code on the server main thread, call Bukkit APIs, rewrite
bytecode with ASM.

## Why native

Rust plugins run without the JVM in the hot path: no garbage collection, no
interpreter, no classloading — just compiled code. This is what makes
hot-path replacements (worldgen noise, chunk encoding, block collisions)
thousands of times faster than Java equivalents.

## Repository layout

| Path | Purpose |
|------|---------|
| `cplug-abi/` | The only contract between the runtime and modules |
| `cplug-sdk/` | SDK for module authors (hooks, JNI, main thread, ASM) |
| `runtime/` | The JVMTI runtime (scan, loading, hook pipeline) |
| `launcher/` | Java launcher that spawns the kernel with the runtime |
| `modules/` | Plugin drop-in directory (see the `c-<name>` repos) |
| `book/` | This documentation |

Example plugins live in separate repositories: `c-hello`, `c-dist`,
`c-crussty`.
