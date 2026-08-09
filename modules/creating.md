---
title: Creating a module
parent: Modules
nav_order: 3
has_children: true
---

# <img class="page-icon" src="/CRUSSTY/assets/images/icons/src.svg" alt="Creating"> Creating a module

A Crussty module is a native shared library that exports one C-ABI function —
`cplugin_init` — and everything else is up to you: hooks, event buses,
storage, threads, your own language. This section starts from the empty
folder and ends with a module running on a live server.

## Where to start

- [What a module is](./creating/intro.html) — the module contract
- [JavaScript](./creating/javascript.html) — QuickJS shim
- [Python](./creating/python.html) — CPython shim
- [C & C++](./creating/c.html) — the native path, no shim
- [Rust](./creating/rust.html) — the reference path

Each language page links the **building** guide (step by step) and the
**platform bricks** page (which runtime primitives it can use).