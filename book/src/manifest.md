# Manifest (module.json)

In plain words: **the module's ID card.** A tiny JSON file telling the
runtime who the module is, where its library lives, and what must load
before it. Only `id` is required — the rest is optional.

A JSON file at the top level of the module (directory or archive). Only `id`
matters to the platform; the rest is metadata for tooling.

```json
{
  "id": "hello",
  "version": "0.1.0",
  "main": "libhello.so",
  "dependencies": ["libb"],
  "options": "seed=42;regen=off"
}
```

## Fields

| Field | Type | Default | Meaning |
|-------|------|---------|---------|
| `id` | string | directory name | Module id; also used to derive the default entry library name (`lib<id>.so` etc.) |
| `version` | string | — | Version, informational |
| `main` | string | `lib<id>.so` / `lib<id>.dylib` / `<id>.dll` | Entry library path, relative to the manifest |
| `dependencies` | array of strings | `[]` | Module ids that must load first (topological order) |
| `options` | string | — | Per-module options (`k=v;k=v`, same format as the global agent options). When present, THIS module's `cplugin_init` receives exactly this string — modules stop parsing each other's settings. Without it the global options are passed (historical behavior) |

`main` is validated: absolute paths (POSIX `/`, Windows `C:\`, UNC `\\`) and
any `..`-escaping component mark the manifest malformed, and the module is
skipped entirely.

## Id collisions

The module id comes from the manifest, or defaults to the directory name (for
archives: the archive file name). **Two modules with the same id cannot both
load**: the first in scan order (sorted path) wins, every later duplicate is
refused LOUDLY in the log. A silent second load would double-register hooks
under one owner id (a hot-reload purge of that id would kill both modules'
hooks) and the hot-reload registry would refuse it anyway — leaving a
half-initialized zombie.

## Isolation between modules

Multi-module servers are first-class: a broken module must fail alone.

- **Boot**: every `cplugin_init` runs under `catch_unwind` — a panicking
  module fails by itself, the remaining modules still come up.
- **Class hooks**: each module hook is panic-isolated in the dispatch loop —
  the hook that panicked is disabled for the process lifetime, the class
  keeps its current bytes, and the OTHER modules' hooks still run.
- **SDK modules** (Rust via `cplug-sdk`): user hook closures are isolated per
  callback inside the SDK dispatch; a panicking closure is removed from the
  registry and the hook chain continues with the bytes it had before.
- **Reload**: `SIGUSR1` / `crussty reload` swaps only the modules whose
  library file actually changed on disk (dev+inode+mtime identity captured
  at admit); unchanged modules keep their state.
