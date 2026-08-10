# Other languages

A module is just a shared library exporting one C-ABI function. Nothing in
the loader knows or cares whether that library came from Rust, C, C++, Zig,
Go, or D — so in theory a module can be written in **any language that can
produce a `cdylib` / `.so` with a C-ABI export**.

## What the platform needs

Every module library must export:

```c
int32_t cplugin_init(const CPluginApi* api, JavaVM* vm, const char* options);
```

- `CPluginApi` is a plain C struct of function pointers (register a class-file
  hook, allocate replacement class bytes, retransform a loaded class).
- `JavaVM*` is the standard invocation-interface pointer — JNI `C` bindings.
- The class-file hook returns raw class bytes; patching them is pure byte
  manipulation in whatever language you like.

So the only real requirements are: an `extern "C"`-style export of that
signature, and JNI bindings for your language.

## Language status

| Language | Ease | Notes |
|---|---|---|
| **Rust** | Native | `cplug-sdk` — hooks, classes, main thread, ASM. Reference path. |
| **C** | ✅ Verified | `c-hello@multilang `examples/c`` — raw JNI headers; `CPluginApi` is already plain C. |
| **C++** | ✅ Verified | `c-hello@multilang `examples/c`pp` — same as C; RAII wrapper. |
| **Go (cgo)** | ⚠️ Harness-only | `c-hello@multilang `examples/go`` works in a test harness, but the Go runtime hijacks signal handlers and **crashes the JVM** (SIGABRT) — see [golang/go#13042]. Ship as a separate process, not a JVM in-process module. |
| **Python** | ✅ Verified | `c-hello@multilang `examples/python`` — C shim embeds CPython; module body is `.py`. |
| **JavaScript** | ✅ Verified | `c-hello@multilang `examples/js`` — C shim embeds QuickJS; module body is `.js`. |
| **Zig** | Straightforward | `@cImport` the JNI headers; export `cplugin_init`. |
| **Nim / D / Odin** | Possible | Compile a `dylib`/`cdylib` and export the symbol. |

[golang/go#13042]: https://github.com/golang/go/issues/13042

## Verified examples (c-hello `multilang` branch)

Each of `c`, `cpp`, `python`, `js` ships a `build.sh` that links against
the vendored `cplug-sdk-c/include`, a `module.json`, and a module body. All four
were exercised end-to-end on a live Purpur 1.21.10 server: the runtime loads
the four `.so`s, and every class load fires all four hooks (16,000+ hook
invocations per module per boot with no errors). The Go module additionally
built and fired its hook in a standalone harness.

Shared mechanics every shim needs:

- **JNI/JVMTI entry**: export `int32_t cplugin_init(const CPluginApi* api, JavaVM* vm, const char* options)` — `cplug-abi.h` provides the struct; the macro `CPLUG_ABI_NO_ENTRY` suppresses the prototype for bindgen/cgo.
- **Class-name bytes**: the JVM hands the hook a name buffer that is **not NUL-terminated** after the name — copy the printable-ASCII prefix into your own buffer before handing it to your runtime (C/C++ can print it raw; Python/JS must bound it).
- **Threads**: hooks run on arbitrary JVM class-load threads, possibly re-entrantly (a nested class can load inside another class's hook). Python survives this because the GIL is recursive; a JS shim needs a **recursive** mutex around the interpreter, and the Python shim must `PyEval_SaveThread()` after `Py_Initialize()` — otherwise the init thread keeps the GIL forever and the first hook on another thread deadlocks.
- **Interpreter startup**: `Py_Initialize()` and `JS_NewRuntime()` are called once in `cplugin_init`; QuickJS needs `JS_SetMaxStackSize(rt, 0)` because its default stack bookkeeping misfires on deep JVM stacks.
- **Entry resolution**: the runtime derives the module library name from the module id (`lib<id>.so`) unless `module.json` sets `"main"`.

## The shared-runtime gotcha

`cplug-sdk` (hooks, classes, main thread, ASM) is a **Rust** crate. The
platform API it wraps (`cplug-abi`) is pure C, but the SDK's convenience
helpers are Rust-only. Two ways to use them from another language:

1. **Raw ABI** — talk to `CPluginApi` + `JavaVM` directly, write your own
   bindings for the three `cplug-abi` entry points. No extra runtime.
2. **SDK re-export** — export the SDK's helper functions (hooks, main-thread
   dispatch) through your own `extern "C"` wrapper in a small Rust shim.

The platform's other bricks (events, telemetry, scheduler) are Rust-runtime
APIs today; they are reachable from other languages only through a C wrapper
if/when one is published.

## Example: minimal C hook

```c
#include <jni.h>
#include <cplug_abi.h>

static jvmtiEnv* jvmti;
static JavaVM*   jvm;

int32_t cplugin_init(const CPluginApi* api, JavaVM* vm, const char* options) {
    jvm = vm;
    return api->register_class_hook(0, on_class_load);  /* ... */
}

static int32_t on_class_load(const char* name, const unsigned char* bytes,
                             uint32_t len, unsigned char** out, uint32_t* outlen) {
    /* patch `bytes` in place, or return a replacement buffer */
    return 0;
}
```

`cplug-abi.h` is generated by the platform build and describes `CPluginApi`
and the hook callback shape. Signature details are versioned in the SDK.

## Caveats

- **Go is not viable in-process**: verified on a live JVM — loading a
  c-shared Go module crashes the host with SIGABRT because the Go runtime
  installs its own signal handlers. Keep Go modules as sidecar processes.
- **Threads**: hook callbacks run on class-load threads; do your JNI work
  there, or attach your own threads explicitly. Go (cgo) can't
  `DetachCurrentThread` from the goroutine-created thread you attach on —
  attach on a thread you own.
- **GC/native refs**: non-Rust runtimes with GC must handle JNI local refs
  carefully (push/pop local frames) and keep pointers the JVM handed you
  stable against moving GC.
- **Build-time**: the entry `.so` must match the kernel's JVM architecture
  (x86_64/aarch64), not the host compiler default.
- **Embedded interpreters**: script-backed modules (Python/JS) work, but only
  through a C shim, and each interpreter needs its own threading care
  (recursive GIL-safe locking, no interpreter finalization for the JVM's
  lifetime).

The Rust SDK remains the most ergonomic path with the deepest coverage of
Crussty's own bricks; C, C++, Python (via shim) and JS (via shim) are
verified working, and Zig is straightforward for direct ABI work.