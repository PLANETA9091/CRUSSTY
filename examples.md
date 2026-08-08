---
title: Example modules
nav_order: 8
---

# Example modules

Example modules live in their own repositories (`c-<name>`) and are cloned
into `modules/`:

| Module | Repo | What it demonstrates |
|---|---|---|
| `c-hello` | PLANETA9091/c-hello | Minimal module: class-load hook + GetLoadedClasses + JNI Bukkit logger |
| `c-dist` | PLANETA9091/c-dist | Dist engine (UDP leases, fencing, heartbeat/commit) as a module |
| `c-crussty` | PLANETA9091/c-crussty | Crussty CE native surface (283 natives) as a module |

## hello

The minimal module proves the pipeline end to end:

- `cplugin.json` manifest (`id: hello`), entry `libhello.so`
- `cplugin_init` registers a hook and logs once a kernel class is loaded
- Expected log: `hello from native c-plugin (v2 pipeline alive)`

Clone/build anyway like any module (see [Quick Start](./quickstart.html)).

## examples-multilang — modules written in C, C++, Python, JS

The `modules/examples-multilang/` tree in this repo demonstrates the
**non-Rust** module path: `c`, `cpp`, `python` and `js` each contain a
`shim.c` exporting the C-ABI `cplugin_init`, a `build.sh`, a `cplugin.json`
and a module body (Python/JS shims embed CPython/QuickJS). Go builds and
passes a harness but crashes the JVM in-process — see
[Other languages](./other-languages.html).

All four verified modules boot together on a live Purpur 1.21.10 server; each
fires its class hook on every class load.
