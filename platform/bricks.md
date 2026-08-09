---
title: Platform bricks
parent: Platform
nav_order: 1

---

# <img class="page-icon" src="/CRUSSTY/assets/images/icons/mcf.svg" alt="Platform bricks"> Platform bricks

Platform bricks are the runtime's native primitives exposed to modules: an
async event bus, tick routing, telemetry, crash isolation, storage, threads —
eleven of them reachable through the C bridge, each owned by exactly one
concern. A module uses them through the **platform bridge**: a
`CPlatformApi*` attached to the ABI's `CPluginApi` (version >= 3).

## The bridge

`CPluginApi` (v3+) carries a trailing platform pointer:

```c
typedef struct CPluginApi {
    uint32_t version;
    int32_t (*register_class_hook)(...);
    uint8_t* (*jvmti_allocate)(size_t);
    /* v3 trailing */
    const CPlatformApi* platform;   /* NULL on older runtimes */
} CPluginApi;
```

`CPlatformApi` is versioned itself (`CPB_VERSION=1`). Before touching any
brick function you must:

```c
if (api->version >= 3 && api->platform && api->platform->version == 1) {
    /* bridge is live: api->platform->events_subscribe(...) ... */
}
```

## Bricks

The C bridge exposes **28 functions in 11 brick groups**. `barriers`
(multi-phase sync between module threads) is a Rust-runtime-only brick and
is not part of `CPlatformApi`:

| Brick | In `CPlatformApi` | What it gives |
|---|---|---|
| events | `events_subscribe` / `events_publish` / `events_unsubscribe` | async pub/sub bus with backpressure, lifecycle events |
| scheduler | `scheduler_*` | tick routing on the kernel main thread, injected tasks |
| telemetry | `telemetry_*` | metrics registry + UDP export |
| signals | `signals_*` | crash isolation: native backtrace, watchdog, fault hooks |
| network | `network_*` | connection registry (LRU, UUID-keyed), traffic counters |
| storage | `storage_*` | key-value persistence |
| threads | `threads_*` | managed worker threads |
| transform | `transform_*` | the bytecode patch pipeline (ClassFileLoadHook) |
| save_events | `save_events_*` | world-save lifecycle hooks |
| hot_reload | `hot_reload_*` | swap a module's library without a server restart |
| side_table | `side_table_*` | O(1) metadata beside kernel objects |

## Rules

1. **Check `api->version >= 3`** and **NULL-check `api->platform`** — old
   runtimes carry no bridge.
2. **Version is trailing** — never touch fields beyond the version you
   checked.
3. **Callbacks you hand to the bridge must live forever** — the platform
   keeps them; module memory must stay valid for the whole runtime.
4. Bridge versions bump independently: `CPAPI_VERSION` and `CPB_VERSION`
   are separate.

## Per-language

- [Bricks in JavaScript](../modules/creating/javascript/bricks.html)
- [Bricks in Python](../modules/creating/python/bricks.html)
- [Bricks in C / C++](../modules/creating/c/bricks.html)
- [Bricks in Rust](../modules/creating/rust/bricks.html)

## Internals

How each brick is implemented (brick files, events/telemetry/signals usage
inside the runtime) is on [Platform internals](./internals.html).