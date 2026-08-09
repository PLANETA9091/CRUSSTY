---
title: C & C++
parent: Creating a module
nav_order: 4
has_children: true
---

# <img class="page-icon" src="/CRUSSTY/assets/images/icons/c.svg" alt=""> C & C++

C is the ABI itself — a module in plain C needs no shim at all: `cplug-abi.h`
is the single header, your `.c` (or `.cpp`) exports `cplugin_init`, done.
C++ is the same path with the same header from `extern "C"` entry.

| | |
|---|---|
| [Building an example module](./c/example.html) | `hello.c` from scratch, build and deploy |
| [Platform bricks in C](./c/bricks.html) | the full bridge — 28 functions across 12 bricks |
| [SDK in C](../../sdk-c.html) | `cplug-sdk-c` — convenience layer on top of the ABI |

Reference module: [`c-hello` (c
branch)](https://github.com/PLANETA9091/c-hello/tree/c).