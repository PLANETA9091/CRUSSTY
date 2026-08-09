---
title: Platform bricks in C
parent: C & C++
nav_order: 2
---

# <img class="page-icon" src="/CRUSSTY/assets/images/icons/mcf.svg" alt="MCF"> Platform bricks in C

A C module receives the entire platform API: `CPluginApi` (class hooks,
JVMTI, claims) **plus** its trailing `platform` pointer — `CPlatformApi`,
version `CPB_VERSION=1`. In C you call the bridge directly, no shim.

## The API surface

The bridge groups 28 functions into 12 bricks under `runtime/src/platform/`:

| Brick | Group in `CPlatformApi` | Key verbs |
|---|---|---|
| events | `events_subscribe`, `events_publish`, `events_unsubscribe` | async pub/sub with backpressure |
| scheduler | tick routing on the kernel main thread | `scheduler_*` |
| telemetry | metrics registry + UDP export | `telemetry_*` |
| signals | crash isolation, backtraces, watchdog, fault hooks | `signals_*` |
| network | connection registry (LRU, UUID-keyed), traffic counters | `network_*` |
| storage | key-value persistence | `storage_*` |
| threads | managed worker threads | `threads_*` |
| transform | bytecode pipeline (the SDK's `transform` hook) | `transform_*` |
| save_events | world-save lifecycle hooks | `save_events_*` |
| hot_reload | swap a module's library without a restart | `hot_reload_*` |
| side_table | O(1) metadata beside kernel objects | `side_table_*` |
| barriers | multi-phase sync between module threads | `barriers_*` |

## Version and contract

```c
if (api->version >= 3 && api->platform && api->platform->version == 1) {
    /* bridge is live — use api->platform->events_subscribe(...) etc. */
}
```

1. **NULL-check `api->platform`** — older runtimes have no bridge.
2. **Version is trailing** — don't touch fields past the version you checked.
3. **Callbacks your module passes must live for the whole runtime** — the
   platform does not own module memory.

Full source: `cplug-abi/src/lib.rs` (Rust) / `cplug-sdk-c/include/cplug-abi.h`
(C headers) in the [CRUSSTY repo](https://github.com/PLANETA9091/CRUSSTY).