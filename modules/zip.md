---
title: Distribution as zip archives
nav_order: 2
---

# <img class="page-icon" src="/CRUSSTY/assets/images/icons/jar.svg" alt=""> Distribution as zip archives

Modules can be distributed as a single zip (or jar) file. The scan treats
any `*.zip` / `*.jar` (case-insensitive) in `modules/` as an archive module.

1. The runtime opens the archive and peeks for `cplugin.json` at the top
   level of the zip.
2. The archive is extracted into a cache directory under `modules/` the
   first time; the cache is keyed by the archive's mtime — re-extraction
   happens only when the archive changed — after that the module is loaded
   from cache.
3. The cache entry is loaded exactly like a directory module (manifest,
   entry resolution, hooks) — the two distribution forms are equivalent.

## Security

Entries whose paths escape the extraction directory via `..` are rejected.
An offending entry fails the whole extraction; the module is skipped with a
warning.

## When to use zip

A single-file module is trivial to hand around and version-lock: download
one file, drop it into `modules/`, restart the server. The module schema,
entry resolution, and hooks are identical to directory modules.
