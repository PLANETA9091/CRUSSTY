---
title: Hooks
parent: Module SDK
nav_order: 1
---

# <img class="page-icon" src="/CRUSSTY/assets/images/icons/function.svg" alt=""> Hooks

Hooks are the primary interface: a callback invoked by the runtime for
every class load.

```rust
hooks::register("org/bukkit/Bukkit", |name| {
    eprintln!("Bukkit loaded: {name}");
});
```

The callback receives the class name as `&str` and fires on every load
matching `pattern`. Glob syntax: `*` matches any run (including `/`),
`?` matches exactly one character — e.g. `"org/bukkit/**"`.

## Byte hooks

To patch class bytes, use `register_bytes` — the callback receives the name
and the current bytes and may return replacement bytes:

```rust
hooks::register_bytes("org/bukkit/Bukkit", |name, bytes| {
    if !looks_like_our_bytecode(bytes) {
        return None; // keep original
    }
    Some(patch(bytes))
});
```

- Return `None` → original bytes are kept.
- Return `Some(bytes)` → replacement bytes are handed to the JVM through
  the runtime's JVMTI allocator.
- Hooks are not unregistered by the SDK itself (outside hot reload, where
  the runtime purges the replaced module's hooks) — callbacks must stay
  valid for the module lifetime.

## Hook chain

Byte hooks chain in registration order: **each hook receives the previous
hook's output** (`None` from a hook passes the current bytes through). The
first hook in the chain sees the original bytes; a later hook sees whatever
the earlier one returned, and the final output is what the JVM gets.

## The SDK's single hook

The SDK itself registers exactly one internal dispatch hook per module
(`sdk_dispatch_hook`) and multiplexes module-level callbacks through it.
This keeps the pipeline free of per-module leaks: registration happens
only inside `init`, so a module that fails to init leaves no hooks behind
(while a module that did register before failing still needs a reload to
purge them — see hot reload).