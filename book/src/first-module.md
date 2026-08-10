# Your first module

This page walks through creating, building and installing a module using
the `crussty` CLI. Everything runs from the terminal — no IDE, no Makefile,
no copying files by hand.

## 1. Install the CLI

```bash
npm i -g crussty
```

## 2. Scaffold the module

```bash
crussty module new hello
```

This creates a `hello/` directory from a language template (Rust by default;
C, C++, Go, JS and Python templates are available — pass `--template`). The
template ships with a `module.json` manifest and a working `cplugin_init`
entry point, so the module compiles before you write a single line.

```text
hello/
├── module.json      # manifest: id, version, main library
├── Cargo.toml       # (rust template)
└── src/
    └── lib.rs       # cplugin_init(api, vm, options)
```

## 3. Build

```bash
crussty module build
```

Compiles the module via its `build.sh` and reports the entry library (e.g.
`libhello.so`). Zero configuration — the template's `build.sh` knows what
to do.

## 4. Test it on a live server

Install the module into the server directory, then start watching for
changes:

```bash
crussty module pack             # produce the bundle
cd ../my-server && crussty install hello   # drop it into modules/

cd ../hello && crussty module watch
```

`watch` rebuilds on every source change and hot-reloads the module into the
running server (SIGUSR1) — no restart, no manual copying. See
[`module.json`](./manifest.md) for the manifest reference.

## 5. Share the module

```bash
crussty module pack     # -> hello-v0.1.0-linux-x64.tar.gz
```

Attach the bundle to a GitHub release of your module repo — then anyone can
install it straight from there:

```bash
crussty install <owner/repo>
```

(`install` finds the `<id>-v<version>-<platform>.tar.gz` bundle in the
repo's releases automatically.) Or drop the archive into a server's
`modules/` — the runtime extracts and loads it without any tooling.

`pack` produces a `.tar.gz` bundle ready to attach to a GitHub release —
then anyone can `crussty install <owner/repo>` it. Drop the archive into
`modules/` and the runtime extracts and loads it automatically.

## Or use the TUI

`crussty tui` wraps all of the above in a full-screen menu — New module,
Build, Rebuild automatically, Pack, Search modules on GitHub (results shown
in the window, Enter installs). Every action's output appears inside the
window with a green OK / red FAILED verdict.
