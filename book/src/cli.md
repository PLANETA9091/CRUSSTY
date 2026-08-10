# CLI

The `crussty` CLI is published on [npm](https://www.npmjs.com/package/crussty)
(Linux, macOS, Windows — the right binary is picked automatically):

```bash
npm i -g crussty
```

Two audiences share one binary: **server builders** (the first half) and
**module creators** (the `module` subcommand plus the TUI).

## Server commands

```bash
crussty init --dir my-server   # scaffold a server dir (kernel, runtime, launcher)
crussty run                    # launch the server; console is forwarded to your terminal
crussty stop                   # stop the running server
crussty log [--follow]         # tail the console log
crussty ls                     # list modules: active / parked / disabled
crussty enable <name>          # activate a parked/disabled module
crussty disable <name>         # park (append .x) or --disabled (append .disabled)
crussty reload                 # hot-reload all modules (SIGUSR1 to the running JVM)
```

## Module commands

```bash
crussty search <query>             # find modules on GitHub
crussty install <name>             # install from the module catalog
crussty install <owner/repo>       # install straight from a GitHub repo
crussty module new <name>          # scaffold a new module (language template)
crussty module build               # build the module in the current directory
crussty module watch               # rebuild + hot-reload on every code change
crussty module pack                # package as a distributable tarball
```

`search` looks for GitHub repos containing a `module.json` manifest (fetched
in parallel, so results arrive in about a second). Repos without a manifest
are filtered out.

## The TUI

`crussty tui` is the full-screen menu for module creators:

- **New module** — scaffold from a language template
- **Build** — build the module in the current directory
- **Rebuild automatically** — rebuild + hot-reload on file changes
- **Pack** — package as a distributable tarball
- **Search modules on GitHub** — live search results inside the window;
  pressing Enter on a result installs it
- **Exit**

Every action runs in a captured subprocess and shows its output inside the
TUI (green `OK` / red `FAILED`), with `Esc` to return to the menu. Long-running
interactive tools (like the server console) stay outside the TUI.
