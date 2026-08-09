# Crussty CLI

CLI for Crussty servers and c-plugins: scaffold, run and manage your Crussty server, build and package c-plugins.

## Install

```bash
npm i -g crussty
```

No install scripts, no runtime dependencies — the binary for your platform is pulled in automatically as an optional dependency.

## Quick start

```bash
# scaffold a new server
crussty init my-server

# launch it
cd my-server && crussty run

# create a new c-plugin (interactive template selection)
crussty module new my-module
```

## Commands

| Command | Description |
| --- | --- |
| `init` | Scaffold a Crussty server directory (downloads kernel) |
| `run` | Launch the server; stdin is forwarded to the server console |
| `stop` | Stop the running server |
| `log --follow` | Tail the server log |
| `reload` | Hot-reload c-plugins (SIGUSR1) |
| `ls` / `enable` / `disable` / `park` | Manage c-plugins |
| `search` | Search for modules on GitHub |
| `install` | Install a module from the catalog or `owner/repo` |
| `module new/build/watch/pack` | c-plugin development workflow |

## Source

https://github.com/PLANETA9091/CRUSSTY
