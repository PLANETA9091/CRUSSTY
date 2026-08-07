# Hooks

Hooks are the primary interface: a callback invoked by the runtime for
every class load.

```rust
hooks::register("org/bukkit/Bukkit", |ctx, bytes, len| {
    if !looks_like_our_bytecode(bytes) {
        return Ok(None); // keep original
    }
    let patched = patch(bytes);
    Ok(Some(patched))
})?;
```

- Return `Ok(None)` → original bytes are kept.
- Return `Ok(Some(bytes))` → replacement bytes are handed to the JVM
  through the runtime's JVMTI allocator.
- Return `Err` → the module is unregistered from the pipeline
  (diagnostic + no more callbacks).

## Byte hooks

`hooks::register_bytes` registers a hook that receives a raw
`&[u8]` slice (`*const u8` + length) instead of a `Vec` — zero-copy when you
only need to inspect. Use it for cheap prefix/signature checks before
deciding to allocate and patch.

## Hook chain

Hooks fire in plugin load order (topological). All modules share one
pipeline: each class load visits every hook until one returns replacement
bytes. The runtime keeps the first plugin's replacement — later plugins see
the *original* bytes, not the patched ones.

## The SDK's single hook

The SDK itself registers exactly one internal dispatch hook per module
(`sdk_dispatch_hook`) and multiplexes module-level callbacks through it.
This keeps the pipeline free of per-module leaks: a module that fails to
`init` correctly never leaves a half-registered hook behind.
