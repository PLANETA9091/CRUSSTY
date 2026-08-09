---
title: Platform bricks in Rust
parent: Rust
nav_order: 2
---

# <img class="page-icon" src="/CRUSSTY/assets/images/icons/rust.svg" alt=""> Platform bricks in Rust

In Rust the bridge is already typed: `CPluginApi` carries a trailing
`platform: *const CPlatformApi` (v3+) and `cplug-sdk` wraps it from
`platform/mod.rs`. In Rust you mostly don't poke the raw struct directly —
the SDK exposes typed helpers for the bricks it touches:

| Brick | SDK face | What it gives |
|---|---|---|
| events | `events::*` | async pub/sub event bus with backpressure |
| scheduler | `main_thread::*` | tick routing on the kernel main thread |
| telemetry | `telemetry::*` | metrics registry + UDP export |
| signals | `signals::*` | crash isolation, fault hooks |
| network | `network::*` | connection registry (LRU, UUID-keyed), counters |
| storage | `storage::*` | key-value persistence |
| threads | `threads::*` | managed worker threads |
| transform | `hooks::*` | ClassFileLoadHook patch pipeline |
| save_events | `save_events::*` | world-save lifecycle hooks |
| hot_reload | runtime-managed | swap a module's library without a restart |
| side_table | `side_table::*` | O(1) metadata beside kernel objects |

The SDK itself ships hooks, classes/JNI, main-thread dispatch, ASM weaving
and logging — the other bricks live inside the runtime and are reached
through `CPlatformApi` directly (same contract as any language). On top of
that, Rust gets the `cplug-sdk` convenience layer (pattern/byte hooks,
class lookup, kernel-ready notification) that other languages reach through
`cplug-sdk-c` — see [SDK in C](../../../sdk-c.html).

> Note: the platform exposes **11 brick groups** through the C bridge
> (`barriers` is a Rust-runtime-only brick, not part of `CPlatformApi`).

## Version guard

```rust
if api.version >= 3 && !api.platform.is_null() {
    let p = api.platform.as_ref()?; // CPlatformApi, version 1
    p.events_subscribe(...)?;
}
```

1. **NULL-check `api.platform`** — it is a trailing field; older `CPluginApi`
   versions have no bridge.
2. **Version-guard everything** — read fields only past the version you
   checked (CPAPI_VERSION=3, CPB_VERSION=1).

Full contract: see [Platform bricks](../../../platform/bricks.html) — same bridge, any
language.
