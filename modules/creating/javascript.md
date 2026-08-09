---
title: JavaScript
parent: Creating a module
nav_order: 2
has_children: true
---

# <img class="page-icon" src="/CRUSSTY/assets/images/icons/js.svg" alt=""> JavaScript

JavaScript modules run on an embedded **QuickJS** interpreter inside a thin C
shim. The module body is plain `.js`; JVM-facing glue (the ABI export) lives
in `shim.c`.

| | |
|---|---|
| [Building an example module](./javascript/example.html) | create the folder, shim, manifest and build script |
| [Platform bricks in JavaScript](./javascript/bricks.html) | which bricks the JS path can use today |

Reference module: [`c-hello` (js branch)](https://github.com/PLANETA9091/c-hello/tree/js).

> QuickJS embedded bare has no `console` — that lives in `quickjs-libc`, which
> the shim does not link. The shim exposes `logNative(msg)` instead.