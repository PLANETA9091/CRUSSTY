---
title: SDK in C (cplug-sdk-c)
parent: Module SDK
nav_order: 5
---

# <img class="page-icon" src="/CRUSSTY/assets/images/icons/src.svg" alt=""> SDK in C (cplug-sdk-c)

`cplug-sdk-c` is a thin **C binding** of the Rust SDK: one header, one
library, plain `extern "C"` functions. It exposes the same convenience
layer — pattern hooks, byte hooks, cross-loader class lookup, kernel-ready
notification, main-thread dispatch and logging — without any JNI/JVMTI
boilerplate.

It lives in the [CRUSSTY](https://github.com/PLANETA9091/CRUSSTY) repository
under `cplug-sdk-c/` (build: `cargo build --release -p cplug-sdk-c` →
`target/release/libcplug_sdk_c.a`).

Who uses it:

- **C / C++** — link the static lib directly, call the functions.
- **Python** — `ctypes` only: no shim logic beyond the embedding trampoline,
  no JNI code at all. See `cplug-sdk-c/examples/python`.
- **JavaScript** — from a QuickJS C shim. See `modules/examples-multilang/js`.

## The header

`#include "cplug-sdk.h"` next to `cplug-abi.h`. It defines four callback
types and fourteen functions.

```c
typedef void  (*cplug_hook_fn)(void* ctx, const char* name);
typedef const uint8_t* (*cplug_byte_hook_fn)(
    void* ctx, const char* name,
    const uint8_t* data, size_t len, size_t* out_len);
typedef void  (*cplug_ready_fn)(void* ctx);
typedef void  (*cplug_main_fn)(void* ctx, void* env /* JNIEnv* */);
```

## Functions

| Function | Purpose | Returns |
|---|---|---|
| `cplug_sdk_init(api, vm)` | **Must be called first**, from `cplugin_init`: stores the vm, registers the SDK's single pipeline hook | — |
| `cplug_sdk_vm()` | raw JavaVM\* stored by init (for JNI bindings) | `JavaVmPtr` |
| `cplug_sdk_hook_register(pattern, ctx, cb)` | name-only hook; fires `cb(ctx, name)` on every class load matching `pattern` (`*` any run incl. `/`, `?` one char) | `0` ok / negative on bad args |
| `cplug_sdk_hook_register_bytes(pattern, ctx, cb)` | byte hook; return replacement bytes (malloc'd) or NULL to keep the original class | `0` / negative |
| `cplug_sdk_on_kernel_ready(class_name, ctx, cb)` | runs `cb(ctx)` **once** on a fresh thread when `class_name` has loaded; safe from `cplugin_init`; polls in background | `0` / negative |
| `cplug_sdk_run_on_main_thread(ctx, cb)` | queues `cb(ctx, env)` on the server's main thread (JNIEnv\* attached); jobs wait while the kernel boots | `0` / negative |
| `cplug_sdk_find_class(name)` | resolved class across **all** class loaders; dot or slash names | JNI global ref or `NULL` |
| `cplug_sdk_wait_class(name, timeout_ms)` | polls every 200 ms until loaded or timeout | JNI global ref or `NULL` |
| `cplug_sdk_retransform_class(name)` | re-runs the hook chain on an already-loaded class | `1` ok / `0` |
| `cplug_sdk_log_info(msg)` / `cplug_sdk_log_warn(msg)` | kernel logger (stderr until kernel is up) | — |
| `cplug_sdk_clear_exception(env)` | describes + clears a pending JNI exception on `env` | `1` if one was pending |
| `cplug_sdk_attach_current_thread()` | attaches if needed; thread stays attached | `JNIEnv*` or `NULL` |
| `cplug_sdk_detach_current_thread()` | detaches — only if **you** attached | — |

Callbacks must stay valid for the whole module lifetime — hooks are never
unregistered.

## Memory contract for byte hooks

Return a **heap-allocated** replacement buffer (e.g. `malloc`); the SDK
copies it into the JVM and then calls `free(3)` on your pointer. Return
`NULL` to keep the original class bytes. The returned length goes through
`out_len`:

```c
static const uint8_t* keep_bytes(void* ctx, const char* name,
                                 const uint8_t* data, size_t len, size_t* out_len) {
    (void)ctx; (void)data;
    *out_len = 0;
    return NULL;          /* keep original */
}
```

## Building a module against it

```bash
cargo build --release -p cplug-sdk-c            # libcplug_sdk_c.a
cc -shared -fPIC -O2 -I<repo>/cplug-sdk-c/include \
   -o libhello_sdk.so hello_sdk.c <repo>/target/release/libcplug_sdk_c.a
```

Deploy `libhello_sdk.so` + `cplugin.json` (`{"id": "hello_sdk"}`) into `modules/`.

## Using it from Python

`hello_sdk.py` in `cplug-sdk-c/examples/python` is the module body — plain
Python, no JNI, no Python C-API:

```python
import ctypes, os
SDK = ctypes.CDLL(os.path.join(os.path.dirname(os.path.abspath(__file__)),
                               "libcplug_sdk_c.so"))

def sdk(name, restype, argtypes):
    f = getattr(SDK, name); f.restype = restype; f.argtypes = argtypes
    return f

sdk_init = sdk("cplug_sdk_init", None, [ctypes.c_void_p, ctypes.c_void_p])
sdk_hook = sdk("cplug_sdk_hook_register", ctypes.c_int32,
               [ctypes.c_char_p, ctypes.c_void_p, ctypes.c_void_p])
HOOK = ctypes.CFUNCTYPE(None, ctypes.c_void_p, ctypes.c_char_p)

@HOOK
def on_class(ctx, name):
    sdk_log(b"[hello] python saw " + (name or b"?"))

def cplugin_init(api_addr, vm_addr, options):
    sdk_init(ctypes.c_void_p(api_addr), ctypes.c_void_p(vm_addr))
    sdk_hook(b"org/bukkit/**", None, on_class)
    return 0
```

Keep the callbacks alive for the whole module lifetime (CPython refcounts
free them otherwise) — the example keeps them in a list.

## Caveats

Same as elsewhere: hook callbacks run on arbitrary class-load threads
(re-entrant); script runtimes need their own threading care (Python — GIL
is recursive; QuickJS — a recursive mutex around the interpreter), and Go
is not viable in-process on a live JVM.

The Rust SDK remains the deepest path (ASM weaving, profiler); everything
below it is now available to C, C++, Python and JS through
`cplug-sdk-c`.