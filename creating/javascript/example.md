---
title: Building a module in JavaScript
parent: JavaScript
nav_order: 1
---

# <img class="page-icon" src="/CRUSSTY/assets/images/icons/js.svg" alt=""> Building a module in JavaScript

A JS module is a C shim (embeds QuickJS) plus a `.js` body. Everything below
is from the [`c-hello` js
branch](https://github.com/PLANETA9091/c-hello/tree/js) and works on a live
Purpur 1.21.10 server.

## 1. The module body — `hello_hello.js`

```js
// hello_js — module body written in JavaScript (loaded by the QuickJS shim).
function make_hook() {
    // Called by the C shim; returns the per-class-load callback.
    return function on_class_load(name) {
        logNative("[hello-js] hook fired for " + name);
        return null; // keep original bytes
    };
}
```

QuickJS embedded bare has no `console` — the shim exposes `logNative(msg)`.

## 2. The shim — `shim.c`

The shim embeds QuickJS, evaluates `hello_hello.js`, grabs `make_hook()`, and
exposes the hook callback to the runtime as a C class-file hook:

```c
#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include "quickjs.h"
#include "cplug-abi.h"

/* ... quickjs_std.h eval of hello_hello.js, pick make_hook, wrap it ... */

static int32_t on_class_load(
    void* ctx, const char* name,
    const uint8_t* class_data, size_t class_data_len,
    uint8_t** out_data, size_t* out_len
) {
    JSContext* cx = (JSContext*)ctx;
    JSValue fn = JS_GetPropertyStr(cx, g_js_hook, "on_class_load");
    JSValue argv[1] = { JS_NewString(cx, name) };
    JSValue ret = JS_Call(cx, fn, JS_UNDEFINED, 1, argv);
    /* ret == null -> keep original bytes (return 1 from the hook) */
    return 1;
}

int32_t cplugin_init(const CPluginApi* api, void* vm, const char* options) {
    /* init the interpreter, eval hello_hello.js once */
    ...
    return api->register_class_hook(NULL, on_class_hook);
}
```

## 3. Manifest — `cplugin.json`

```json
{"id": "hello", "main": "libhello.so"}
```

## 4. Build — `build.sh`

```bash
#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
QJS_DIR="${QJS_DIR:-$PWD/qjs}"
cc -shared -fPIC -O2 \
    -I"$PWD/../../cplug-abi" \
    -I"$QJS_DIR/../.." \
    -o libhello.so shim.c \
    -L"$QJS_DIR" -Wl,-rpath,'$ORIGIN/qjs' -lqjs
```

(the distro's prebuilt `libqjs` targets a newer glibc than the JVM's — a
working copy is committed in `./qjs` next to the script.)

## 5. Deploy

Drop `libhello.so` and `cplugin.json` into `modules/` and start the server.
Expected log: `[hello-js] hook fired for ...` on every kernel class load.