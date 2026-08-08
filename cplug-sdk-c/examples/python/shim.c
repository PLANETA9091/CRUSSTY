// hello_py — Python Crussty module using the SDK C binding.
//
// The shim holds only the JVM-facing export; ALL logic lives in
// hello_sdk.py, which drives the SDK through ctypes against
// libcplug_sdk_c.so. No JNI code in C, no Python C-API save a two-call
// trampoline.
//
// Build:  cc -shared -fPIC -O2 $(python3-config --cflags) \
//             -o libhello_py_sdk.so shim.c $(python3-config --embed --ldflags)
// Deploy: libhello_py_sdk.so + libcplug_sdk_c.so + hello_sdk.py + cplugin.json
//
// GIL discipline (matches the plain-python example):
//   - Py_Initialize() runs on the OnLoad thread and keeps the GIL;
//   - import hello_sdk and call its cplugin_init while holding it;
//   - PyEval_SaveThread() afterwards, hooks from arbitrary class-load
//     threads then re-take the GIL via PyGILState_Ensure.

#define _GNU_SOURCE
#include <dlfcn.h>
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <Python.h>

#include "cplug-abi.h"

int32_t cplugin_init(const CPluginApi* api, void* vm, const char* options) {
    /* The runtime dlopens modules with RTLD_LOCAL, so symbols of the Python
     * host (e.g. PyUnicode_FromFormat needed by the _ctypes extension) are
     * invisible to nested dlopens. Re-register libpython in the global
     * scope first, like the runtime's own libcrussty_runtime.so does. */
    const char* py_so = "libpython3.14.so.1.0";
    void* h = dlopen(py_so, RTLD_NOW | RTLD_GLOBAL);
    if (h == NULL) {
        /* fall back to whatever the python3-config ldflags linked */
        h = dlopen("libpython3.so.1.0", RTLD_NOW | RTLD_GLOBAL);
    }
    if (h == NULL) {
        fprintf(stderr, "warning: libpython not found in global scope (%s)\n",
                dlerror());
    }

    Py_Initialize();

    /* Put the module dir (the .so's dir) on sys.path. */
    Dl_info info;
    const char* dir = ".";
    if (dladdr((void*)cplugin_init, &info) && info.dli_fname != NULL) {
        const char* slash = strrchr(info.dli_fname, '/');
        if (slash != NULL && slash != info.dli_fname) {
            size_t n = (size_t)(slash - info.dli_fname);
            char* buf = (char*)malloc(n + 1);
            memcpy(buf, info.dli_fname, n);
            buf[n] = '\0';
            dir = buf;
        }
    }
    {
        char setpath[1024];
        snprintf(setpath, sizeof(setpath),
                 "import sys\n"
                 "sys.path.insert(0, \"%s\")\n", dir);
        PyRun_SimpleString(setpath);
        if (dir[0] != '.') free((void*)dir);
    }

    PyObject* m = PyImport_ImportModule("hello_sdk");
    if (m == NULL) {
        PyErr_Print();
        return 2;
    }
    PyObject* fn = PyObject_GetAttrString(m, "cplugin_init");
    Py_DECREF(m);
    if (fn == NULL || !PyCallable_Check(fn)) {
        PyErr_Print();
        Py_XDECREF(fn);
        return 3;
    }
    PyObject* r = PyObject_CallFunction(
        fn, "(KKs)", (unsigned long)api, (unsigned long)vm,
        options != NULL ? options : "");
    Py_DECREF(fn);
    int rc = -1;
    if (r == NULL) {
        PyErr_Print();
    } else {
        rc = (int)PyLong_AsLong(r);
        Py_DECREF(r);
    }
    /* The interpreter holds the GIL from Py_Initialize on this thread.
     * Release it: SDK hooks fire from arbitrary JVM class-load threads and
     * take the GIL themselves via PyGILState_Ensure. */
    PyEval_SaveThread();
    return rc;
}