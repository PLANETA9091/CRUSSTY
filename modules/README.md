# Plugins

Plugins (modules) live in their own `c-<name>` repositories:

- [c-hello](https://github.com/PLANETA9091/c-hello) — minimal proof module (hook + GetLoadedClasses + JNI)
- [c-dist](https://github.com/PLANETA9091/c-dist) — the dist engine (UDP leases, fencing) as a module
- [c-crussty](https://github.com/PLANETA9091/c-crussty) — Crussty CE native surface as a module

Install: clone into `modules/<name>` and build (`cargo build && cp target/debug/lib<name>.so .`),
or pack the plugin directory into a `.zip` and drop it next to the others — the
runtime extracts and loads it automatically.
