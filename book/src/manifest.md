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
  "dependencies": ["libb"]
}
```

## Fields

| Field | Type | Default | Meaning |
|-------|------|---------|---------|
| `id` | string | directory name | Module id; also used to derive the default entry library name (`lib<id>.so` etc.) |
| `version` | string | — | Version, informational |
| `main` | string | `lib<id>.so` / `lib<id>.dylib` / `<id>.dll` | Entry library path, relative to the manifest |
| `dependencies` | array of strings | `[]` | Module ids that must load first (topological order) |

`main` is validated: absolute paths (POSIX `/`, Windows `C:\`, UNC `\\`) and
any `..`-escaping component mark the manifest malformed, and the module is
skipped entirely.

## Id collisions

The module id comes from the manifest, or defaults to the directory name (for
archives: the archive file name). Two modules with the same id are both
loaded; only the dependency-ordering map dedupes (last wins).
