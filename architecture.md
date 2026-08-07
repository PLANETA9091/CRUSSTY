---
title: Architecture
nav_order: 4
---

# Architecture

## Layers

```
┌────────────────────────────────────────────────────┐
│ launcher.jar / server.jar (single-jar, Boot.class) │
│   - single-jar: extracts runtime + modules,        │
│     System.load -> JNI_OnLoad                      │
│   - launcher: spawns child JVM with -agentpath     │
└───────────────────────┬────────────────────────────┘
                        │ -agentpath / JNI_OnLoad
┌───────────────────────▼────────────────────────────┐
│ Crussty Runtime (libcrussty_runtime.so, JVMTI)     │
│   - scans modules/, topologically loads modules    │
│   - owns the ClassFileLoadHook pipeline            │
│   - owns the JVMTI byte allocator + retransform    │
│   - platform bricks (src/platform/*)               │
└───────┬──────────────────────────┬─────────────────┘
        │ dlopen (RTLD_LOCAL)      │ CLASS_FILE_LOAD_HOOK
┌───────▼──────────┐      ┌────────▼─────────────────────────┐
│ modules/*.so     │      │ kernel JVM (Purpur/Paper)        │
│ (cplug-abi only) │◄────►│  every class load, pre-JIT        │
└──────────────────┘      └──────────────────────────────────┘
```

Two entry points reach the same runtime: the launcher passes `-agentpath`
so the runtime is a classic JVMTI agent before the kernel boots; the
single-jar bootstrapper (in `boot`) extracts the runtime and modules, writes
MODE options, and `System.load`s the runtime, whose `JNI_OnLoad` brings up
the same pipeline. The launcher path requires a kernel module; the
single-jar path requires none.

## Class file hook pipeline

The runtime enables `CAN_GENERATE_ALL_CLASS_HOOK_EVENTS` and
`CAN_RETRANSFORM_CLASSES`, then registers `CLASS_FILE_LOAD_HOOK`. On every
class load it walks its hook chain — one slot per module — in topological
(load) order. A module may:

- return `rc == 0` with replacement bytes → the runtime hands them to the
  JVM through its own `jvmti_allocate` buffer (freed by the JVM), or
- return `rc > 0` → skip, keep the original bytes.

Because hooks run on the class-load thread, modules must not block there
(see [SDK: main thread](./sdk/main-thread.html) for deferred work).

## ABI contract

Modules never link against the runtime. The only contract is `cplug-abi`, a
plain C struct of function pointers (`CPluginApi`) passed into
`cplugin_init(api, vm, options)`:

- `register_class_hook(module_ctx, ClassHookFn) -> i32`
- `alloc_class_bytes(size) -> *mut u8` (JVMTI allocator)
- `retransform_class(class_bytes, len) -> i32` (class name is embedded in
  the bytes; JVMTI reads it)

`CPAPI_VERSION` guards ABI drift: the runtime rejects modules compiled
against a different ABI version.

## Module loading

`RTLD_LOCAL` everywhere. Every class produced by `register_class_hook` is
named with the module prefix (`dist/...`, `crussty/...`) so modules never
collide in the kernel's loader namespace.

## Topological order

The scanner reads each manifest's `dependencies` and loads in topological
order (Kahn's algorithm; unknown ids fall back to sorted path order, cycles
keep their sorted position). Hooks then fire in load order, which is the
order modules expect their patches applied.

## Platform bricks

Below the ABI layer, the runtime ships twelve reusable primitives in
`src/platform/` (see [Platform bricks](./platform.html) for the API
surface): an event bus, a class-patch pipeline (`transform`), crash
isolation that chains the JVM's own signal handlers (`signals`), tick
routing (`scheduler`), persistence (`storage`), an O(1) side table
(`side_table`), live module swapping (`hot_reload`), managed threads
(`threads`), multi-phase barriers (`barriers`), connection tracking
(`network`), telemetry export (`telemetry`) and save-lifecycle hooks
(`save_events`).

Bricks are compiled into the runtime, not into modules: a module links
`cplug-abi` only, and calls brick APIs through the runtime's exported
surface.

## Signal handling

The crash-isolation brick installs `sigaction` handlers with `SA_SIGINFO`
and *chains* to the previous disposition. The JVM installs its own handlers
for SIGSEGV and friends (hs_err reporting, JIT null-checks, stack banging);
a platform that overwrites them breaks the JVM. On a fault the platform
either records counters and forwards the fault to the previous handler, or —
when there was no previous handler — dumps a native backtrace itself and
re-raises the default disposition so the JVM's fatal-error machinery still
runs. `CRUSSTY_NO_SIGNALS=1` disables this.
