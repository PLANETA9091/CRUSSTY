---
title: Building a module in Python
parent: Python
nav_order: 1
---

# <img class="page-icon" src="/CRUSSTY/assets/images/icons/python.svg" alt=""> Building a module in Python

A Python module is a C shim (embeds CPython) that defers all hook logic to a
`.py` body. Everything here is from the [`c-hello` python
branch](https://github.com/PLANETA9091/c-hello/tree/python) and boots on a
live Purpur 1.21.10 server.

## 1. The module body — `hello_hello.py`

```python
def make_hook():
    """Return the callable the runtime invokes for every kernel class load."""
    def on_class_load(name, **_):
        import sys
        sys.stderr.write("[hello-py] hook fired for %s\n" % name)
        return None  # keep original bytes (a real patch would return bytes)

    on_class_load.__name__ = "on_class_load"
    return on_class_load
```

The shim calls `make_hook()` at init; the returned callable is invoked from
the class-file hook with `name` (and future kwargs) and may return `bytes` to
replace the class, or `None` to keep the original.

## 2. The shim — `shim.c`

`cplugin_init` embeds CPython, imports `hello_hello`, grabs `make_hook()`,
and forwards every class load into it:

```c
#include <Python.h>
#include "cplug-abi.h"

static PyObject* g_py_hook = NULL;

static int32_t on_class_hook(
    void* ctx, const char* name,
    const uint8_t* class_data, size_t class_data_len,
    uint8_t** out_data, size_t* out_len
) {
    (void)ctx; (void)class_data; (void)class_data_len;
    (void)out_data; (void)out_len;
    /* copy the name to a bounded NUL-terminated buffer first: the hook
     * passes the kernel class-name buffer, which can carry trailing
     * garbage after the name */
    char namebuf[512];
    /* ... bounded copy of name into namebuf ... */
    PyObject* ret = PyObject_CallFunction(g_py_hook, "s", namebuf);
    int keep = 1; /* None -> keep original bytes */
    Py_XDECREF(ret);
    return keep;
}

int32_t cplugin_init(const CPluginApi* api, JavaVM* vm, const char* options) {
    /* seed CPython once, import hello_hello, stash g_py_hook */
    ...
    return api->register_class_hook(NULL, on_class_hook);
}
```

## 3. Manifest — `module.json`

```json
{"id": "hello", "version": "0.1.0"}
```

## 4. Build — `build.sh`

```bash
#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
CFLAGS="$(python3-config --cflags 2>/dev/null || echo -I/usr/include/python3.14)"
LDFLAGS="$(python3-config --embed --ldflags 2>/dev/null || echo -lpython3.14)"
cc -shared -fPIC -O2 $CFLAGS \
    -I"$PWD/../../cplug-abi" \
    -o libhello.so shim.c $LDFLAGS
echo "built $(pwd)/libhello.so"
```

(`--embed` gives the full embedding flags — on split-python distros like Arch
plain `--ldflags` lacks `-lpythonX.Y`; the fallback is `-lpython3.14`.)

## 5. Deploy

Drop `libhello.so` and `module.json` into `modules/`. Expected log on boot:
`[hello-py] hook fired for ...` for every kernel class load.