# Distribution as zip archives

Modules can be distributed as a single zip (or jar) file. The scan treats
any `*.zip` / `*.jar` (case-insensitive) in `modules/` as an archive module.

## How extraction works

1. The runtime opens the archive and peeks for `cplugin.json` at the top
   level. Archives **without a manifest are ignored** (they are ordinary
   data archives).
2. The archive is extracted into the cache directory:

   ```
   <temp>/cplug-cache/<archive-stem>-<content-hash>/
   ```

   The hash is the FNV-1a of the archive file contents. Re-extraction only
   happens when the archive changed — after that the module is loaded from
   the cache, so boot time stays constant.
3. The cache entry is loaded exactly like a directory module (manifest,
   entry library, dependency ordering).

## Security

Archive extraction is deliberately conservative:

| Protection | Behavior |
|-----------|----------|
| Path traversal (`..`) | entry rejected |
| Absolute paths (`/`, `C:\`, `\\`) | entry rejected |
| Symlinks | entry rejected (file-mode check) |
| Entry count | `> 10 000` entries → archive rejected |
| Entry size | `> 256 MiB` → entry rejected |
| Total size | `> 1 GiB` → archive rejected |

An offending entry fails the whole extraction; the module is skipped with a
diagnostic.

## Why zip

A single-file module is trivial to hand around, download, and version-lock:
`hello-0.1.0.zip` instead of a directory tree. Everything else (manifest
schema, entry resolution, hooks) is identical to directory modules.
