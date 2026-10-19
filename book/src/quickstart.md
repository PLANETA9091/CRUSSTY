# Quick Start

Everything you need is the `crussty` CLI from npm plus a Java 21+ runtime.
No jars to download, no scripts, no build toolchain.

## 1. Install the CLI

```bash
npm i -g crussty
```

The correct binary for your platform (Linux, macOS, Windows) is installed
automatically. The npm package is republished automatically on every CLI
change, so a plain `npm update -g crussty` always gets the latest build.

## 2. Scaffold a server

```bash
crussty init --dir my-server
cd my-server
```

`init` downloads the native runtime and the launcher, then asks what to
inject into:

```text
crussty: choose the kernel to inject into:
  1. Paper — pick a version and download
  2. Purpur — pick a version and download
  3. Fabric — pick a version and download
  4. Forge — pick a version and download
  5. Skip — I'll place my own server jar
```

Choose a platform and `init` fetches the live version list from the
platform's own API; choose a version and the download starts
automatically (Forge additionally runs its own `--installServer`).

Prefer to bring your own jar? Any Minecraft server `.jar` works —
Paper, Purpur, Fabric, Forge, vanilla:

```bash
crussty init --dir my-server --jar /path/to/server.jar   # copy an existing jar
crussty init --dir my-server --no-kernel                 # instructions only
```

Without a kernel you are simply told where to place it:

```text
crussty: no kernel downloaded — put the Minecraft server .jar you want to
  inject into yourself at:

    my-server/versions/kernel.jar

  and set it in crussty.toml:

    [server]
    kernel = "kernel.jar"
```

After scaffolding the directory looks like this:

```text
my-server/
├── crussty.toml          # kernel + memory + catalog settings
├── versions/<kernel>.jar # your server jar — the injection target
├── launcher/launcher.jar
├── libcrussty_runtime.so
├── modules/              # drop modules here
├── logs/
└── crus/                 # module data
```

## 3. Start the server

```bash
crussty run
```

The server boots with the runtime attached and the console is forwarded to
your terminal (`stop` shuts it down cleanly; `Ctrl+D` detaches without
stopping). A successful boot looks like this in the log:

```text
[crussty-runtime] pipeline ready: 1 module hook(s)
[13:36:26 INFO]: Done (1.23s)! For help, type "help"
```

## 4. Install modules

```bash
crussty search nbt                    # find modules on GitHub
crussty install hello                 # install from the catalog
crussty install PLANETA9091/c-hello   # or straight from a repo
crussty ls                            # active / parked / disabled
```

That's it — the server is running with Crussty modules live. See
[Example modules](./examples.md) for what `hello`, `dist` and `crussty`
demonstrate.

## Next steps

- **Write your own module** — `crussty tui` (or `crussty module new`)
  scaffolds, builds, hot-reloads and packs; see
  [Your first module](./first-module.md).
- **Configure the server** — kernel jar, memory and catalog live in
  [`crussty.toml`](./config.md).
- **Hack on the platform itself** — see
  [Building from source](./dev.md).
