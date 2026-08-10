---
title: Quick Start
nav_order: 2
---

# <img class="page-icon" src="/CRUSSTY/assets/images/icons/bolt.svg" alt=""> Quick Start

Five minutes from nothing to a running Crussty server. All you need is
the `crussty` CLI from npm and Java 21+.

## 1. Install the CLI

```bash
npm i -g crussty
```

That's the whole setup — the right binary for your platform (Linux, macOS,
Windows) comes with it.

## 2. Create a server

```bash
crussty init --dir my-server
cd my-server
```

`crussty init` sets up the folder and downloads everything the server needs
(the kernel, the runtime, the launcher). You get:

```
my-server/
├── crussty.toml          # kernel version, memory, catalog — easy to edit
├── versions/             # the kernel jar
├── modules/              # your modules live here
├── logs/                 # server logs
└── crus/                 # data written by modules
```

## 3. Start the server

```bash
crussty run
```

The console is right in your terminal — type `stop` to shut the server
down. A successful boot ends with `pipeline ready` in the log.

## 4. Install modules

```bash
crussty search <query>                  # find modules on GitHub
crussty install hello                   # install from the catalog
crussty install PLANETA9091/c-hello     # or straight from a repo
crussty ls                              # see what's active
```

Done — modules load at the next start. See
[Example modules](./modules/examples.html) for what's available.

## What now?

- **Write your own module** — `crussty module new hello` scaffolds one from
  a template, `crussty tui` gives you a menu (build, watch, pack, search).
  Step-by-step: [Creating a module](./modules/creating.html).
- **Tune the server** — kernel version, memory and the module catalog are
  simple fields in `crussty.toml`.
- **Something off?** — check [Troubleshooting](./troubleshooting.html).
