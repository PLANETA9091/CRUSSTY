# Architecture

## Layers

```
┌────────────────────────────────────────────────────┐
│ launcher.jar (Java)                                │
│   - resolves the kernel jar from versions/         │
│   - spawns the child JVM with -agentpath           │
│   - tees logs, forwards stdin                      │
└───────────────────────┬────────────────────────────┘
                        │ -agentpath:libcrussty_runtime.so=...
┌───────────────────────▼────────────────────────────┐
│ Crussty Runtime (libcrussty_runtime.so, JVMTI)     │
│   - scans modules/, topologically loads plugins    │
│   - owns the ClassFileLoadHook pipeline            │
│   - owns the JVMTI byte allocator + retransform    │
└───────┬──────────────────────────┬─────────────────┘
        │ dlopen (RTLD_LOCAL)      │ CLASS_FILE_LOAD_HOOK
┌───────▼──────────┐      ┌────────▼─────────────────────────┐
│ modules/*.so     │      │ kernel JVM (Purpur/Paper)        │
│ (cplug-abi only) │◄────►│  every class load, pre-JIT        │
└──────────────────┘      └──────────────────────────────────┘
```

The **runtime** is a classic JVMTI native agent: the JVM calls its
`AgentMain`/`OnLoad` with a `jvmtiEnv*` and a `JavaVM*`. It is attached with
`-agentpath` before the kernel boots, so it observes every kernel class load
from the very first one.

The word "agent" is JVMTI's own term for this kind of library; in this
project the component is called the **runtime** to avoid confusion with
AI/LLM agents.

## Class file hook pipeline

The runtime enables `CAN_GENERATE_ALL_CLASS_HOOK_EVENTS` and
`CAN_RETRANSFORM_CLASSES`, then registers `CLASS_FILE_LOAD_HOOK`. On every
class load it walks its hook chain — one slot per plugin — in topological
(load) order. A plugin may:

- return `rc == 0` with replacement bytes → the runtime hands them to the
  JVM through its own `jvmti_allocate` buffer (freed by the JVM), or
- return `rc > 0` → skip, keep the original bytes.

Because hooks run on the class-load thread, plugins must not block there
(see [SDK: main thread](./sdk-main-thread.md) for deferred work).

## ABI contract

Plugins never link against the runtime. The only contract is `cplug-abi`, a
plain C struct of function pointers (`CPluginApi`) passed into
`cplugin_init(api, vm, options)`:

- `register_class_hook(module_ctx, ClassHookFn) -> i32`
- `alloc_class_bytes(size) -> *mut u8` (JVMTI allocator)
- `retransform_class(class_bytes, len) -> i32` (class name is embedded in the
  bytes; JVMTI reads it)

`CPAPI_VERSION` guards ABI drift: the runtime rejects plugins compiled
against a different ABI version.

## Module loading

`RTLD_LOCAL` everywhere: modules cannot see each other's symbols by default,
and every class produced by `register_class_hook` is named with the module
prefix (`dist/...`, `crussty/...`, `hello/...`) so modules never collide in
the kernel's loader namespace.

## Topological order

The scanner reads each manifest's `dependencies` and loads in topological
order (Kahn's algorithm; unknown ids fall back to sorted path order, cycles
keep their sorted position). Hooks then fire in load order, which is the
order plugins expect their patches applied.
