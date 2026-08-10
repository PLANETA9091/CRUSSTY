---
title: What a module is
parent: Creating a module
nav_order: 1
---

# <img class="page-icon" src="/CRUSSTY/assets/images/icons/src.svg" alt="Module"> What a module is

The easiest way to start a module is the CLI — it scaffolds the whole
layout from a language template (Rust, C, C++, Go, JS, Python), builds and
packs it, and can hot-reload it into a running server:

```bash
npm i -g crussty
crussty module new hello          # scaffold from a template
crussty module build              # build
crussty tui                       # full-screen menu (build, watch, pack, search)
```

A Crussty module is a **shared library** that exports exactly one C-ABI
function:

```c
int32_t cplugin_init(const CPluginApi* api, void* vm, const char* options);
```

The runtime scans `modules/` on every boot, loads each manifest, and for
each module calls `cplugin_init` once. From then on the module is on the
hot-patch pipeline: its class-file hook is asked for every kernel class load,
it can schedule work on the kernel main thread, and it can use the platform
bricks (events, storage, network, threads…) via the bridge.

That single function is the whole contract. Anything that can export it —
Rust, C, C++, Python, JavaScript, via shims — is a module.

## The lifecycle

1. **Scan** — the runtime finds `module.json` + the entry library
2. **Load** — `dlopen`, resolve `cplugin_init`
3. **Init** — called once, before the kernel boots; register hooks, stash
   state, **no JVM work here**
4. **Run** — hook callbacks on class-load threads; main-thread dispatches;
   bridge calls any time
5. **Reload** — a module can be swapped via the hot-reload bridge without a
   server restart

## Next

- [Building an example module (Rust)](rust/example.html)

Full contract details: [Modules](../../modules.html).