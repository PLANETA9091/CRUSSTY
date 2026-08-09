---
title: Creating a module
nav_order: 5
has_children: true
---

# <img class="page-icon" src="./assets/images/icons/src.svg" alt="Creating"> Creating a module

A Crussty module is a native shared library that exports one C-ABI function —
`cplugin_init` — and everything else is up to you: hooks, event buses,
storage, threads, your own language. This section starts from the empty
folder and ends with a module running on a live server.

## Where to start

- [What a module is](./creating/intro.html) — the module contract
- [Rust](./creating/rust.html) — the reference path
- [C & C++](./creating/c.html) — the native path, no shim
- [Python](./creating/python.html) — CPython shim
- [JavaScript](./creating/javascript.html) — QuickJS shim

Each language page links the **building** guide (step by step) and the
**platform bricks** page (which runtime primitives it can use).