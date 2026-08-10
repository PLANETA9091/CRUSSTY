---
title: Example modules
parent: Modules
nav_order: 4
---

# <img class="page-icon" src="/CRUSSTY/assets/images/icons/rust.svg" alt=""> Example modules

Example modules live in their own repositories (`c-<name>`). The fastest way
to try any of them is the CLI — prebuilt bundles, no cloning or building:

```bash
npm i -g crussty
crussty install hello               # from the catalog
crussty install PLANETA9091/c-dist # or straight from a repo
crussty ls
```

Or clone the source and build it yourself:

| Module | Repo | What it demonstrates |
|---|---|---|
| `c-hello` | PLANETA9091/c-hello | Minimal module: class-load hook + GetLoadedClasses + JNI Bukkit logger |
| `c-dist` | PLANETA9091/c-dist | Dist engine (UDP leases, fencing, heartbeat/commit) as a module |
| `c-crussty` | PLANETA9091/c-crussty | Crussty CE native surface (283 natives) as a module |

## hello

Install: `crussty install hello`.

The minimal module proves the pipeline end to end:

- `module.json` manifest (`id: hello`), entry `libhello.so`
- `cplugin_init` registers a hook and logs once a kernel class is loaded
- Expected log: `hello from native c-plugin (v2 pipeline alive)`

End-to-end check: `crussty run` on a server with hello installed prints
`hello from native c-plugin`.

## examples-multilang — modules written in C, C++, Python, JS

The `modules/examples-multilang/` tree in this repo demonstrates the
**non-Rust** module path: `c`, `cpp`, `python` and `js` each contain a
`shim.c` exporting the C-ABI `cplugin_init`, a `build.sh`, a `module.json`
and a module body (Python/JS shims embed CPython/QuickJS). Go builds and
passes a harness but crashes the JVM in-process (Go runtime signal
handlers; keep Go modules as sidecar processes).

All four verified modules boot together on a live Purpur 1.21.10 server; each
fires its class hook on every class load.
