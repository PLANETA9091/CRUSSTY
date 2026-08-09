---
title: Python
parent: Creating a module
nav_order: 3
has_children: true
---

# <img class="page-icon" src="/CRUSSTY/assets/images/icons/python.svg" alt=""> Python

Python modules embed **CPython** through a thin C shim: the runtime talks
C-ABI to the shim, the shim defers all hook logic to `.py`. The module body
is written entirely in Python.

| | |
|---|---|
| [Building an example module](./python/example.html) | shim + `hello_hello.py`, build and deploy |
| [Platform bricks in Python](./python/bricks.html) | which bricks the Python path can use |

Reference module: [`c-hello` (python
branch)](https://github.com/PLANETA9091/c-hello/tree/python).

> The JVM process owns our `.so` for its lifetime; we never finalize the
> interpreter, so objects are intentionally never released (test module).