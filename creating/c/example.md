---
title: Building a module in C
parent: C & C++
nav_order: 1
---

# <img class="page-icon" src="/CRUSSTY/assets/images/icons/c.svg" alt=""> Building a module in C

A C module is the smallest possible Crussty module: one `.c`, one header, one
build line. From the [`c-hello` c
branch](https://github.com/PLANETA9091/c-hello/tree/c):

## 1. `hello.c`

```c
// hello_c — a Crussty module written in plain C.
#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include "cplug-abi.h"

static void* vm;

static int32_t on_class_load(
    void* ctx, const char* name,
    const uint8_t* class_data, size_t class_data_len,
    uint8_t** out_data, size_t* out_len
) {
    (void)ctx; (void)class_data; (void)class_data_len;
    (void)out_data; (void)out_len;
    fprintf(stderr, "[hello-c] class load hook fired for %s\n", name);
    return 1; /* keep original bytes */
}

int32_t cplugin_init(const CPluginApi* api, void* vm_ptr, const char* options) {
    (void)options;
    fprintf(stderr, "[hello-c] cplugin_init (C module, api v%u)\n", api->version);
    if (api->register_class_hook) {
        int rc = api->register_class_hook(NULL, on_class_load);
        fprintf(stderr, "[hello-c] register_class_hook rc=%d\n", rc);
    } else {
        fprintf(stderr, "[hello-c] register_class_hook unavailable\n");
        return 1;
    }
    return 0;
}
```

## 2. `cplugin.json`

```json
{"id": "hello_c"}
```

## 3. Build

```bash
gcc -shared -fPIC -O2 -o libhello_c.so hello.c
```

(the runtime picks `lib<id>.so` = `libhello_c.so` next to the manifest.)

## 4. Deploy

Copy `libhello_c.so` + `cplugin.json` into `modules/` and start the server.
Expected log: `[hello-c] cplugin_init (C module, api v3)` then a hook log on
the first kernel class load after boot.

## C++

Identical: build the same sources with `g++ -shared -fPIC -O2`, and export the
entry with `extern "C"`. `cplug-abi.h` is plain C-compatible.