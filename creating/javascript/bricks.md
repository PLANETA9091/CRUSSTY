---
title: Platform bricks in JavaScript
parent: JavaScript
nav_order: 2
---

# <img class="page-icon" src="/CRUSSTY/assets/images/icons/mcf.svg" alt=""> Platform bricks in JavaScript

A JS module reaches the platform through its C shim: `cplugin_init` receives
the full `CPluginApi`, so **all** bricks the C path exposes are available —
the shim is the boundary, not the JS runtime.

What this means in practice:

| Brick | From JS | Notes |
|---|---|---|
| events | via shim export | expose `api->platform->events_subscribe` as a JS function |
| scheduler | via shim export | `run_on_main_thread`-style dispatch, JS callback |
| network | via shim export | registry + traffic counters, string keys |
| storage | via shim export | key-value, JSON-serializable in JS |
| threads | via shim export | worker thread with JS event loop |
| hot_reload | automatic | swap `libhello.so` — QuickJS state is re-seeded |

Everything else (transform, signals, telemetry, side_table, barriers,
save_events) is reachable the same way: the shim forwards fields of
`api->platform` into functions callable from JS.

The full API and the version-guard contract (CPAPI_VERSION=3,
CPB_VERSION=1) is on [Platform
bricks](../../platform.html).