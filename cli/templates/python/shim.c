// __NAME__ — a Crussty module whose hook logic is written in Python,
// embedded via a thin C shim.
//
// Build:  cc -shared -fPIC -O2 $(python3-config --cflags)
//         -o lib__NAME__.so shim.c $(python3-config --embed --ldflags)
// Deploy: lib__NAME__.so  +  module.json { "id": "__NAME__" }  in modules/
//
// The runtime dlopens this .so and calls cplugin_init. The shim embeds a
// CPython interpreter and defers hook logic to __NAME__.py — so the
// module body is written in Python while the JVM-facing ABI stays C. This is
// the "script-backed module needs a C shim" pattern.
#define _GNU_SOURCE
#include <Python.h> /* must come before any glibc header (pyconfig defines
                     * _POSIX_C_SOURCE itself; a prior <stdio.h> triggers
                     * the "redefined" warning) */
#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include <pthread.h>
#include <dlfcn.h>
#include <Python.h>

#include "cplug-abi.h"

/* Python callable invoked from the class-file hook. The JVM process owns our
 * .so for its lifetime; we never finalize the interpreter, so the callable
 * is intentionally never released (test module). */
static PyObject* g_py_hook = NULL;

static int32_t on_class_hook(
    void* ctx, const char* name,
    const uint8_t* class_data, size_t class_data_len,
    uint8_t** out_data, size_t* out_len
) {
    (void)ctx; (void)class_data; (void)class_data_len;
    (void)out_data; (void)out_len;
    /* The JVM's class-name buffer is not NUL-terminated after the name —
     * the runtime passes it through, so it can contain trailing garbage.
     * Copy only the valid prefix (bounded) before handing it to Python,
     * and never pass a NULL arg into the interpreter. */
    char namebuf[512];
    size_t i = 0;
    while (i < sizeof(namebuf) - 1 && name != NULL && name[i] != '\0' &&
           (unsigned char)name[i] >= 0x20 && (unsigned char)name[i] <= 0x7e) {
        namebuf[i] = name[i];
        i++;
    }
    namebuf[i] = '\0';

    PyGILState_STATE gil = PyGILState_Ensure();
    PyObject* r = NULL;
    if (g_py_hook != NULL && PyCallable_Check(g_py_hook)) {
        PyObject* arg = PyUnicode_FromString(namebuf);
        if (arg != NULL) {
            r = PyObject_CallOneArg(g_py_hook, arg);
            Py_DECREF(arg);
        } else {
            PyErr_Clear();
        }
        if (r == NULL) { PyErr_Print(); }
    }
    Py_XDECREF(r);
    PyGILState_Release(gil);
    return 0; /* keep original bytes */
}

int32_t cplugin_init(const CPluginApi* api, void* vm, const char* options) {
    (void)vm; (void)options;

    /* Hot-reload re-runs cplugin_init on a fresh dlopen. Py_Initialize() a
     * second time in the same process is a hard SIGSEGV inside CPython, so
     * boot the interpreter exactly once and reuse it on reloads. */
    static int py_booted = 0;
    if (!py_booted) {
        Py_Initialize();
        py_booted = 1;
    }
    PyGILState_STATE gil = PyGILState_Ensure();

    /* Put the module dir (the .so's dir) on sys.path so the Python half
     * imports cleanly when deployed next to the library. */
    {
        Dl_info info;
        const char* dir = ".";
        if (dladdr((void*)cplugin_init, &info) && info.dli_fname) {
            const char* slash = strrchr(info.dli_fname, '/');
            if (slash != NULL && slash != info.dli_fname) {
                size_t n = (size_t)(slash - info.dli_fname);
                char* buf = (char*)malloc(n + 1);
                memcpy(buf, info.dli_fname, n);
                buf[n] = '\0';
                dir = buf;
            }
        }
        char setpath[1024];
        snprintf(setpath, sizeof(setpath),
                 "import sys\n"
                 "sys.path.insert(0, \"%s\")\n",
                 dir);
        PyRun_SimpleString(setpath);
        if (dir[0] != '.') free((void*)dir);
    }

    PyObject* m = PyImport_ImportModule("__NAME__");
    if (m == NULL) {
        PyErr_Print();
        PyGILState_Release(gil);
        return 2;
    }
    PyObject* fn = PyObject_GetAttrString(m, "make_hook");
    Py_DECREF(m);
    if (fn == NULL || !PyCallable_Check(fn)) {
        PyErr_Print();
        Py_XDECREF(fn);
        PyGILState_Release(gil);
        return 3;
    }
    PyObject* hook = PyObject_CallNoArgs(fn);
    Py_DECREF(fn);
    if (hook == NULL || !PyCallable_Check(hook)) {
        PyErr_Print();
        Py_XDECREF(hook);
        PyGILState_Release(gil);
        return 4;
    }
    static PyObject* owner = NULL;
    Py_XDECREF(owner);
    owner = hook;
    g_py_hook = owner;

    if (api->register_class_hook) {
        int rc = api->register_class_hook(NULL, on_class_hook);
        fprintf(stderr, "[__NAME__] register_class_hook rc=%d\n", rc);
        PyGILState_Release(gil);
        if (rc != 0) return rc;
    } else {
        PyGILState_Release(gil);
        return 1;
    }

    /* CPlatformApi bridge demo (the same surface a pure-C module uses). */
    if (api->platform) {
        fprintf(stderr, "[__NAME__] platform table v%u\n", (unsigned)api->platform->version);
        int n = 0;
        if (api->platform->telemetry_publish_metric)
            n += (api->platform->telemetry_publish_metric("__NAME__.init", 42.0, "rc", NULL) == 0);
        if (api->platform->events_publish)
            n += (api->platform->events_publish("__NAME__.hello", "{\"phase\":\"init\"}") > 0);
        if (api->platform->scheduler_current_tick)
            (void)api->platform->scheduler_current_tick();
        fprintf(stderr, "[__NAME__] platform exercised %d calls, snapshot=%s\n", n,
                api->platform->telemetry_snapshot_json
                    ? api->platform->telemetry_snapshot_json() : "n/a");
    } else {
        fprintf(stderr, "[__NAME__] no platform table (old runtime?)\n");
    }
    return 0;
}