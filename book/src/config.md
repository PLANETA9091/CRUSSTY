# Configuration (crussty.toml)

Every server directory scaffolded by `crussty init` contains a
`crussty.toml`. The CLI reads it to know which kernel to boot, how much
memory to give the JVM, and which catalog to install modules from:

```toml
[server]
kernel = "purpur-1.21.10.jar"
memory = "2G"

[catalog]
repo = "PLANETA9091/crussty-catalog"
```

## Fields

| Section | Key | Default | Meaning |
|---------|-----|---------|---------|
| `server` | `kernel` | `purpur-1.21.10.jar` | Kernel jar file name inside `versions/` |
| `server` | `memory` | `2G` | Heap size, passed as `-Xms`/`-Xmx` to the JVM |
| `catalog` | `repo` | `PLANETA9091/crussty-catalog` | Catalog repo (`owner/name`) used by `crussty install <id>` |

## How it is read

The config is parsed by `crussty run` (kernel + memory) and by
`crussty install` (catalog repo). Lines starting with `#` and empty lines
are ignored; values may be quoted or bare. Sections not listed here are
preserved but currently unused by the CLI — the runtime itself reads its
own options from the launcher command line (see
[Env options](./quickstart.md#env-options)).

## Editing

- **Switch kernel version**: download another jar into `versions/` and
  change `kernel`.
- **Give the server more memory**: `memory = "4G"` (or `"2048M"`).
- **Use a different catalog**: point `repo` at any GitHub repo containing a
  `catalog.json` (see the
  [catalog format](https://github.com/PLANETA9091/crussty-catalog)).

The CLI never rewrites `crussty.toml` after `init` — the file is yours.
