---
title: Manifest (cplugin.json)
parent: Plugins
nav_order: 1
---

---
title: Manifest (cplugin.json)
parent: Plugins
nav_order: 1
---

# Manifest (cplugin.json)

A JSON file at the top level of the plugin (directory or archive). Only `id`
matters to the platform; the rest is metadata for tooling.

```json
{
  "id": "hello",
  "version": "0.1.0",
  "main": "libhello.so",
  "dependencies": ["libb"]
}
```

## Fields

| Field | Type | Default | Meaning |
|-------|------|---------|---------|
| `id` | string | directory name | Plugin id; also used to derive the default entry library name (`lib<id>.so` etc.) |
| `version` | string | — | Version, informational |
| `main` | string | `lib<id>.so` / `lib<id>.dylib` / `<id>.dll` | Entry library path, relative to the manifest |
| `dependencies` | array of strings | `[]` | Plugin ids that must load first (topological order) |

`main` is validated: absolute paths (POSIX `/`, Windows `C:\`, UNC `\\`) and
any `..`-escaping component mark the manifest malformed, and the plugin is
skipped entirely.

## Id collisions

The plugin id comes from the manifest, or defaults to the directory name (for
archives: the archive file name). Two plugins with the same id are both
loaded; only the dependency-ordering map dedupes (last wins).
