---
title: Plugins
nav_order: 3
---

---
title: Plugins
nav_order: 3
---

# Plugins

A plugin is either a **directory** or a **zip archive** containing a
`cplugin.json` manifest and an entry shared library. The scan is recursive:
any `cplugin.json` under `modules/` marks a plugin directory; any `.zip` /
`.jar` file (case-insensitive) is treated as an archive plugin.

## Directory layout

```
modules/
└── hello/
    ├── cplugin.json      # manifest
    ├── libhello.so       # entry library (Linux)
    ├── native/           # optional bundled libs (never dlopened as plugins)
    └── anything-else/    # resources
```

## Conventions

- Libraries **without** a manifest (e.g. `native/` dependencies) are never
  loaded as plugins.
- Anything named `*.disabled` (file or directory) is skipped — the Paper
  convention for disabling a plugin without deleting it.
- Build output directories (`target/`, `build/`, `out/`, `node_modules/`,
  `.git`) are skipped by the scan.

## Dependency ordering

Plugins can declare `dependencies` in the manifest. The runtime topologically
orders loading so a plugin's dependencies are loaded first. Unknown
dependencies fall back to the sorted path order (deterministic); cycles keep
their sorted position.

## Entry library resolution

The entry library is:

1. the manifest `main` field (relative path), if present, or
2. the platform cdylib name for the plugin id — `lib<id>.so` (Linux),
   `lib<id>.dylib` (macOS), `<id>.dll` (Windows, MSVC has no `lib` prefix).

A malformed `main` (absolute path, or escaping the plugin directory with
`..`) invalidates the whole plugin — it is skipped.

## The required export

Every plugin library must export the cplug-abi entry point:

```c
int32_t cplugin_init(const CPluginApi* api, JavaVM* vm, const char* options);
```

`api` provides three services: register a class-file hook, allocate
replacement class bytes via JVMTI, and retransform a loaded class. Everything
else is raw JNI/JVMTI through the `JavaVM*`.
