---
title: Example modules
nav_order: 7
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
