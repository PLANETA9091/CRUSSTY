<div id="boot" aria-hidden="true">
  <div class="boot-logo"><img src="./assets/crussty-logo.png" alt="Crussty logo" /></div>
  <pre></pre>
</div>

<div class="glide-banner">
  <img class="glide-banner-logo" src="./assets/crussty-logo.png" alt="Crussty logo" />
  <div class="glide-banner-text">
    <h1 class="glide-banner-title">Crussty Platform</h1>
    <p class="glide-banner-subtitle">Native Rust plugin platform for Paper-family Minecraft kernels</p>
    <div class="glide-banner-actions">
      <a class="glide-btn glide-btn-primary" href="./quickstart.html">Get started</a>
      <a class="glide-btn" href="./architecture.html">Architecture</a>
    </div>
  </div>
</div>

```text
  ____ ____  _   _ ____ ____ _______   __
 / ___|  _ \| | | / ___/ ___|_   _\ \ / /
| |   | |_) | | | \___ \___ \ | |  \ V /
| |___|  _ <| |_| |___) |__) || |   | |
 \____|_| \_\\___/|____/____/ |_|   |_|
```

# Introduction

Crussty is a native plugin platform for Paper-family Minecraft kernels
(Purpur, Paper, etc.). Plugins — called **c-plugins** or **modules** — are
written in **Rust** and loaded into the server process as native shared
libraries.

<img class="logo" src="./assets/crussty-logo.png" alt="Crussty logo" />

## What it does

A JVMTI runtime (`libcrussty_runtime.so`) is attached to the kernel JVM with
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
