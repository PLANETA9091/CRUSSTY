---
title: Troubleshooting
nav_order: 8
---

# Troubleshooting

## Boot fails / kernel won't start

- Check the launcher log: the runtime's own messages are printed to stderr
  with the `[crussty-runtime]` prefix. `JNI_ERR` from `cplugin_init` means a
  module rejected init (ABI version mismatch, missing dependency).
- `no jvmti env` — the kernel is running without the runtime attached
  (`-agentpath` missing from the launcher invocation).
- After a crash, `hs_err_*.log` files appear next to the kernel; the
  runtime's `[crussty-runtime]` lines are at the top of the stack section.

## Module not loaded

- Does `cplugin.json` exist? Does `modules/` contain a `cplugin.json` in the
  module directory or a `*.zip` with a manifest inside?
- Is it named `*.disabled`? That is honored.
- Entry library missing → `lib<id>.so` not found next to the manifest, or
  `main` points at a missing file.
- A malformed `main` (absolute path, `..` escape) invalidates the whole
  module silently — check the runtime's diagnostic line.

## Zip module never appears

- The zip must contain `cplugin.json` at the top level — not inside a
  folder. Peek check: archives without a manifest are ignored entirely.
- Extract to `<temp>/cplug-cache/<stem>-<hash>/` — verify the cache was
  created (`ls /tmp/cplug-cache/`).
- Check the security limits: >10 000 entries, >256 MiB per entry, >1 GiB
  total, `..` or absolute paths, symlinks — any of these rejects the whole
  archive.

## JVMTI errors from retransform

The runtime logs the raw JVMTI error code and, when possible,
`GetErrorName` of it. Raw codes are the ground truth — `GetErrorName` can
itself fail during early boot. Retransform before the clip phase fails:
wait for kernel classes to load first (`wait_class`).

## `hello from native c-plugin` missing

The pipeline is not delivering to the main thread. Check in order:

1. `[crussty-runtime] module hello -> init rc=0` — module loaded?
2. `[crussty-runtime] pipeline ready: N module hook(s)` — hook registered?
3. Server log for `hello from native c-plugin` — main-thread bridge alive?

## JVM flags

- `-agentpath` is the JVM's own flag for native libraries (the JVMTI spec
  calls them "agents"); this project's component is the *runtime*. The flag
  name itself cannot change.
- Java 21+: dynamic agent loading is deprecated (JEP 451) — this platform
  attaches at boot via `-agentpath`, which remains fully supported.
