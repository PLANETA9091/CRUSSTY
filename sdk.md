---
title: Module SDK
nav_order: 6
---

# <img class="page-icon" src="/CRUSSTY/assets/images/icons/src.svg" alt=""> Module SDK

`cplug-sdk` is the comfort layer on top of `cplug-abi`. Modules declare it
as a dependency and get: hook registration, class lookup across loaders,
main-thread dispatch, logging, and ASM method patching.

```rust
use cplug_sdk::{init, hooks, classes, log};

init(api, vm, options)?;

hooks::register("org/bukkit/Bukkit", |ctx, bytes, len| {
    // patch bytecode of every Bukkit class load
    Ok(Some(patched.to_vec()))
})?;
```

## Golden rules

1. **No JVM work from `cplugin_init`.** The init thread is the runtime's
   attach thread; `find_class`-style work can NPE or deadlock there. Only
   register hooks and stash state.
2. **No JVM work from a hook callback.** Class-load threads are cheap and
   must not block. Defer via `main_thread::run_on_main_thread`.
3. **Unique class names per module.** Prefix generated classes with your
   module id (`hello/Bridge`), or you collide with other modules' generated
   classes in the kernel's loader.
4. **Never call `jvmti_allocate` yourself** — use
   `cplug_sdk::alloc_replacement` / the ABI allocator, so the runtime can
   free everything after the JVM copies it.

## Modules

- **Hooks** — register_class_hook, byte hooks
- **Classes & JNI** — find_class, wait_class, retransform
- **Main thread** — run_on_main_thread
- **ASM weaving** — replace_body, ArgSpec

All SDK entry points are safe to call from any thread that has an attached
JNI env (`with_attached` handles attach/detach for the main thread; kernel
threads are already attached).

## Not writing Rust?

The same convenience layer is available to C, C++, Python, JavaScript and
Zig modules through **`cplug-sdk-c`** — a thin C binding of the SDK
(pattern hooks, byte hooks, class lookup, main-thread dispatch, kernel
logging) with one header and one library. Python modules drive it straight
from `hello_sdk.py` via `ctypes`; C, C++ and Zig link it directly. You
only write Rust if you need the bytecode weaving (`insert_call_at_start` /
`redirect_calls`) or the profiler — see [Other languages](other-languages.html)
for the full matrix.
