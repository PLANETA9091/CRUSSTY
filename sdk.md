---
title: Module SDK
nav_order: 6
has_children: true
---

# <img class="page-icon" src="/CRUSSTY/assets/images/icons/src.svg" alt=""> Module SDK

In plain words: **the comfortable way to write modules.** The bare
contract (`cplugin_init` + the ABI) works, but the SDK gives you ready
helpers for the common stuff — hooks, finding classes, running on the main
thread, logging, patching methods — so you write less low-level code.

`cplug-sdk` is the comfort layer on top of `cplug-abi`. Modules declare it
as a dependency and get: hook registration, class lookup across loaders,
main-thread dispatch, logging, and ASM method patching.

```rust
use cplug_sdk::{init, hooks, classes, log};

init(api, vm); // stores the JavaVM, registers the SDK dispatch hook

hooks::register("org/bukkit/Bukkit", |name| {
    // fires on every Bukkit class load (name-only hook)
    eprintln!("Bukkit loaded: {name}");
});
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
4. **Never call `jvmti_allocate` yourself for replacement bytes** — byte
   hooks return `Vec<u8>` and the SDK routes them through the ABI allocator,
   so the runtime can free everything after the JVM copies it.

## Modules

- **Hooks** — register_class_hook, byte hooks
- **Classes & JNI** — find_class, wait_class, retransform
- **Main thread** — run_on_main_thread
- **ASM weaving** — replace_body, ArgSpec

See also [SDK in C](sdk-c.html) for the same modules through the
`cplug-sdk-c` C binding.

All SDK entry points are safe to call from any thread that has an attached
JNI env (`with_attached` handles attach/detach for the main thread; kernel
threads are already attached).

## Not writing Rust?

The same convenience layer is available to C, C++, Python and JavaScript
modules (or any C-ABI language) through **`cplug-sdk-c`** — a thin C
binding of the SDK
(pattern hooks, byte hooks, class lookup, main-thread dispatch, kernel
logging) with one header and one library. Python modules drive it straight
from `hello_sdk.py` via `ctypes`; C and C++ link it directly. You
only write Rust if you need the bytecode weaving (`insert_call_at_start` /
`redirect_calls`) or the profiler — see the [language pages](./modules/creating.html)
for the full matrix.
