---
title: Architecture
nav_order: 3
---

# <img class="page-icon" src="/CRUSSTY/assets/images/icons/mcf_load.svg" alt=""> Architecture

## Layers

<div class="arch-diagram" markdown="0">

<div class="arch-layer arch-top">
<div class="arch-title">launcher.jar · server.jar</div>
<ul>
<li><b>single-jar</b> — extracts runtime + modules → <code>System.load</code> → <code>JNI_OnLoad</code></li>
<li><b>launcher</b> — spawns a child JVM with <code>-agentpath</code></li>
</ul>
</div>

<div class="arch-arrow">-agentpath / JNI_OnLoad</div>

<div class="arch-layer arch-core">
<div class="arch-title">Crussty Runtime <span class="arch-file">libcrussty_runtime.so · JVMTI</span></div>
<ul>
<li>scans <code>modules/</code> and loads modules in topological order</li>
<li>owns the <code>ClassFileLoadHook</code> pipeline</li>
<li>owns the JVMTI byte allocator + retransform</li>
<li>platform bricks in <code>src/platform/*</code></li>
</ul>
</div>

<div class="arch-split">
<div class="arch-layer arch-mod">
<div class="arch-title">modules/*.so</div>
<div class="arch-note">cplug-abi only · dlopen (RTLD_LOCAL) · patching</div>
</div>
<div class="arch-layer arch-jvm">
<div class="arch-title">kernel JVM</div>
<div class="arch-note">Purpur/Paper · every class load, pre-JIT</div>
</div>
</div>

<div class="arch-flow">
<div class="arch-step">class load</div>
<div class="arch-flow-arrow">→</div>
<div class="arch-step">hook chain<br><i>topological order</i></div>
<div class="arch-flow-arrow">→</div>
<div class="arch-step">rc &gt; 0 → skip</div>
<div class="arch-flow-arrow">→</div>
<div class="arch-step arch-step-ok">replacement bytes<br><i>jvmti_allocate</i></div>
</div>

</div>
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

| API | Purpose |
| --- | --- |
| `register_class_hook(module_ctx, ClassHookFn) -> i32` | attach a hook to the class-load pipeline |
| `jvmti_allocate(size) -> *mut u8` | allocate replacement bytes via the JVMTI allocator |
| `retransform_class(name) -> i32` | force retransformation of the named, already-loaded class (JVMTI `RetransformClasses`) |

`CPAPI_VERSION` guards ABI drift: the runtime rejects modules compiled
against a different ABI version.

## Module loading

`RTLD_LOCAL` everywhere. Every class produced by `register_class_hook` is
named with the module prefix (`dist/...`, `crussty/...`) so modules do not
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



