# Modules

Crussty modules live in their own repositories (convention: `c-<id>`); this
directory is only the install location on a running server (the launcher
loads every subdirectory that carries a `module.json`).

Published modules (installable via `crussty install <id>` from the catalog
`PLANETA9091/crussty-catalog`):

| id              | repository                                       |
|-----------------|--------------------------------------------------|
| `hello`         | github.com/PLANETA9091/c-hello                    |
| `dist`          | github.com/PLANETA9091/c-dist                     |
| `crussty`       | github.com/PLANETA9091/c-crussty                  |
| `cells`         | github.com/PLANETA9091/c-cells (maintained separately) |
| `c-moduleslist` | github.com/PLANETA9091/c-moduleslist              |
| `menus`         | github.com/PLANETA9091/c-menus (animated HUD menus, Paper 26.3) |
| examples (c/cpp/go/js/python) | github.com/PLANETA9091/c-hello, branch `multilang` (in `examples/`) |
