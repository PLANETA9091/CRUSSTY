---
title: Manifest (cplugin.json)
nav_order: 1
parent: Modules
---

# <img class="page-icon" src="/CRUSSTY/assets/images/icons/json.svg" alt=""> Manifest (cplugin.json)

A JSON file at the top level of the module (directory or archive). Only `id`
is required; the fields the runtime reads are:

| Field | Type | Default | Meaning |
|---|---|---|---|
| `id` | string | directory or archive name | Module id; also used to derive the default entry library name (`lib<id>.so` etc.) |
| `main` | string | derived | Path to the entry library, relative to the manifest |
| `dependencies` | array of strings | `[]` | Module ids that must load first (topological order) |

Other fields (for example `version`) are allowed but ignored by the loader —
anything the manifest needs beyond `id`/`main`/`dependencies` is read by
your own entry point.

A `main` that is an absolute path (**POSIX `/`**, Windows `C:\`, UNC `\\`) or
contains any `..`-escaping component marks the manifest malformed, and the
module is skipped.

The module id comes from the manifest, or defaults to the directory name
(for archives: the cache folder `cplug-cache/<stem>-<hash>` — the archive
file name plus a content hash, so an archive without `id` gets a new id
whenever its bytes change). The scanner does not deduplicate: two
modules with the same id are both loaded, and the dependency map ends up
with whichever manifest was scanned later — keep ids unique.
