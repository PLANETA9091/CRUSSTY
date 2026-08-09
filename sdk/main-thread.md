---
title: Main thread
parent: Module SDK
nav_order: 3
---

# <img class="page-icon" src="/CRUSSTY/assets/images/icons/mcf_tick.svg" alt=""> Main thread

Most kernel work must happen on the server main thread. The SDK delivers a
`Runnable` there:

```rust
main_thread::run_on_main_thread(|_env| {
    log::info("dispatched on the server main thread");
});
```

The callback receives a real `JNIEnv` of the main thread — use it for
method lookups (see [Classes & JNI](classes.html)).

## Why the main thread is special

- `Bukkit.getLogger()` (and most Bukkit singletons) are `null` until
  `MinecraftServer.getServer()` is set — which happens on the main thread
  during boot. Calling them from `cplugin_init` or a hook thread either
  NPEs or races the boot.
- `run_on_main_thread` dispatches *after* the server is up, so Bukkit
  lookups are safe there.

## Delivery

The SDK hand-assembles a small Java-compatible `Runnable` (Java 8 class
format) and defines it into the kernel on demand, then invokes it on the
main thread — no reflection, no per-call classloading. The callback
receives a real `JNIEnv` of the main thread.

## Blocking

The main thread is the server. Never block it: keep closures short, or
spawn your own threads for heavy work and only marshal results back to the
main thread.
