# Main thread

Most kernel work must happen on the server main thread. The SDK delivers a
`Runnable` there:

```rust
main_thread::run_on_main_thread(|env| {
    let msg = cstr("hello from native c-plugin (v2 pipeline alive)");
    let _ = classes::static_method("org/bukkit/Bukkit", "getLogger", "()Lorg/bukkit/Logger;")
        .and_then(|_| { /* invoke */ Ok(()) });
    log::info(msg);
});
```

## Why the main thread is special

- `Bukkit.getLogger()` (and most Bukkit singletons) are `null` until
  `MinecraftServer.getServer()` is set — which happens on the main thread
  during boot. Calling them from `cplugin_init` or a hook thread either
  NPEs or races the boot.
- `run_on_main_thread` dispatches *after* the server is up, so Bukkit
  lookups are safe there.

## Delivery

The SDK bridges a small Java `Runnable` into the kernel (compiled with
`javac --release 8` and `include_bytes!`'d into the module), loads it on the
main thread, and invokes it — no reflection, no per-call classloading. The
callback receives a real `JNIEnv` of the main thread.

## Blocking

The main thread is the server. Never block it: keep closures short, or
spawn your own threads for heavy work and only marshal results back to the
main thread.
