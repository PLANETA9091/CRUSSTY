# CLI

The `crussty` CLI is published on [npm](https://www.npmjs.com/package/crussty)
(Linux, macOS, Windows — the right binary is picked automatically). It
republishes automatically on every push that touches the CLI, so
`npm i -g crussty` always installs the latest build; `npm update -g crussty`
picks it up on already-installed machines.

Two audiences share one binary: **server builders** (the first half) and
**module creators** (the `module` subcommand plus the TUI).

## Server commands

```bash
crussty init --dir my-server   # scaffold a server dir (kernel of your choice, runtime, launcher)
crussty run                    # launch the server; console is forwarded to your terminal
crussty stop                   # stop the running server
crussty log [--follow]         # tail the console log
crussty ls                     # list modules: active / parked / disabled
crussty enable <name>          # activate a parked/disabled module
crussty disable <name>         # park (append .x) or --disabled (append .disabled)
crussty reload                 # hot-reload all modules (SIGUSR1 to the running JVM)
```

## `crussty init` — the kernel

Crussty injects into **any Minecraft server `.jar`** — Paper, Purpur,
Fabric, Forge, vanilla. `init` never downloads a kernel on its own: in an
interactive terminal it asks, otherwise it just prints where to place the
jar.

```bash
crussty init                                # interactive: platform → version → download
crussty init --platform purpur --version 1.21.10   # fully non-interactive
crussty init --platform paper               # interactive version picker for Paper
crussty init --jar /path/to/server.jar      # use (copy) an existing jar
crussty init --no-kernel                    # skip; prints placement instructions
crussty init --release v2.2.5               # pin the runtime/launcher release
```

The interactive flow first asks for the platform (Paper, Purpur, Fabric,
Forge — or Skip), then fetches the live version list from that platform's
own API and asks for the version (newest stable first; type `all` to see
every version including pre-releases). Forge additionally runs its
installer (`java -jar …-installer.jar --installServer .`) so the server is
ready to boot; `crussty run` boots it via the installer-generated args
file automatically.

Nothing is downloaded without a choice — in scripts/CI the kernel step
always degrades to printed instructions:

```text
crussty: no kernel downloaded — put the Minecraft server .jar you want to
  inject into yourself at:

    my-server/versions/kernel.jar

  and set it in crussty.toml:

    [server]
    kernel = "kernel.jar"
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
