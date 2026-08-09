# Example modules

All examples are standalone repositories; each demonstrates one layer of the
platform.

## c-hello — the minimal module

```bash
git clone https://github.com/PLANETA9091/c-hello modules/hello
```

- `module.json` manifest (`id: hello`), entry `libhello.so`
- `hooks::register("org/bukkit/Bukkit")` — pipeline participation
- `on_kernel_ready` → `run_on_main_thread` → JNI `Bukkit.getLogger().info`
- Expected log: `hello from native c-plugin (v2 pipeline alive)`

The hello module is the e2e smoke test: if it prints, the whole pipeline
(scan → load → hook → main-thread bridge → JNI) is alive.

## c-dist — the distributed region engine as a module

```bash
git clone https://github.com/PLANETA9091/c-dist modules/dist
```

Ports the v1 dist engine (UDP region-mesh, leases/fencing, commits) from the
old standalone crate into a module:

- `DistKernel.java` bridged with `include_bytes!` — a helper class for
  `MinecraftServer.getTickTimesNanos()` load
- 100ms main-thread driver via `run_on_main_thread`
- full JNI surface via `RegisterNatives` — no reflection

Expected log: `[dist] lease granted region=0`, `[dist] commit region=...`.

## c-crussty — the Crussty CE native surface

```bash
git clone https://github.com/PLANETA9091/c-crussty modules/crussty
```

All bridge classes + `RegisterNatives` from the original CE
`libpaper_native_jni.so` / `libpaper_native_chunk_encode_jni.so`, plus the
improved worldgen noise pipeline (hot-path replacement via ASM weaving).

- runtime dependency: `native/libpaper_native_jni.so` comes from the
  original Crussty CE distribution
- demonstrates `replace_body` + `ArgSpec::ThisField` on real kernel classes

## modules/README

The `modules/` directory in the platform repo documents the convention:
clone one of these into `modules/<name>`, build, copy `lib<name>.so`
next to the manifest, and the runtime picks it up on the next boot.

## examples-multilang — module written in C, C++, Python, JS

The `examples-multilang/` tree (in the platform repo) demonstrates the
**non-Rust** module path: `c`, `cpp`, `python` and `js` each contain a
`shim.c` exporting the C-ABI `cplugin_init`, a `build.sh`, a `module.json`
and a module body (Python/JS shims embed CPython/QuickJS). Go builds and
passes a harness but crashes the JVM in-process — see
[other-languages.md](other-languages.md).

All four verified modules boot together on a live Purpur 1.21.10 server; each
fires its class hook on every class load. Checked-in build scripts and
module sources live at `examples-multilang/`.
