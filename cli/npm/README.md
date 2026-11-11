# Crussty CLI

CLI for Crussty servers and c-plugins: scaffold, run and manage your Crussty server, build and package c-plugins. Works with **any Minecraft server `.jar`** — Paper, Purpur, Fabric, Forge, vanilla.

## Install

```bash
npm i -g crussty
```

No install scripts, no runtime dependencies — the binary for your platform is pulled in automatically as an optional dependency. The package republishes automatically on every CLI change, so `npm update -g crussty` always gets the latest build.

## Quick start

```bash
# scaffold a new server (asks which kernel to download:
# Paper / Purpur / Fabric / Forge — or bring your own .jar)
crussty init --dir my-server

# launch it
cd my-server && crussty run

# create a new c-plugin (interactive template selection)
crussty module new my-module
```

## Commands

| Command | Description |
| --- | --- |
| `init` | Scaffold a Crussty server directory; interactively pick a kernel (Paper / Purpur / Fabric / Forge → version → auto-download), or use `--jar <path>`, `--platform <p> --version <mc>`, `--no-kernel` |
| `run` | Launch the server; stdin is forwarded to the server console |
| `build` | Pack launcher + runtime + installed modules + kernel into ONE bootable jar (`java -jar`, no launcher needed; any jar kernel except Forge) |
| `stop` | Stop the running server |
| `log --follow` | Tail the server log |
| `reload` | Hot-reload c-plugins (SIGUSR1) |
| `ls` / `enable` / `disable` / `park` | Manage c-plugins |
| `search` | Search for modules on GitHub |
| `install` | Install a module from the catalog or `owner/repo` |
| `module new/build/watch/pack` | c-plugin development workflow |

## Source

https://github.com/PLANETA9091/CRUSSTY
