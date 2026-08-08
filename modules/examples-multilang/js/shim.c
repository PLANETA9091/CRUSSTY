// hello_js — a Crussty module whose hook logic is written in JavaScript,
// embedded via a QuickJS shim (quickjs-ng).
// Build:  cc -shared -fPIC -O2 -I/usr/include $(pkg-config --cflags --libs qjs) \
//             -o libhello_js.so shim.c
// Deploy: libhello_js.so + cplugin.json { "id": "hello_js" } in modules/
//
// The runtime dlopens this .so and calls cplugin_init. The shim embeds
// QuickJS, evaluates hello_hello.js, and calls its exported on_class_load
// from the runtime's class-file hook. Same "script-backed module needs a C
// shim" pattern as Python.
#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <dlfcn.h>
#include <pthread.h>
#include <quickjs.h>

#include "cplug-abi.h"

static JSRuntime* g_rt = NULL;
static JSContext* g_ctx = NULL;
static JSValue g_js_hook = JS_UNDEFINED;

/* QuickJS is single-threaded per context; class-load threads can race on the
 * context after boot, so serialize hook calls with a mutex. Must be
 * RECURSIVE: the JVM can re-enter the class-file hook on the same thread
 * (loading a nested class such as Thread$ThreadNumbering inside another
 * class), so a plain mutex would deadlock. */
static pthread_mutex_t g_js_lock = PTHREAD_MUTEX_INITIALIZER;

/* Dump the pending JS exception to stderr (quickjs-libc's js_std_dump_error
 * lives in a static lib without PIC; print the message ourselves). */
static void dump_error(JSContext* ctx) {
    JSValue exc = JS_GetException(ctx);
    const char* msg = JS_ToCString(ctx, exc);
    fprintf(stderr, "[hello-js] JS exception: %s\n", msg ? msg : "?");
    JS_FreeCString(ctx, msg);
    JS_FreeValue(ctx, exc);
}

/* Global JS function `logNative(msg)` -> stderr. Exposed so the JS module
 * body can log without quickjs-libc (bare engine has no console/print). */
static JSValue js_log(JSContext* ctx, JSValueConst this_val, int argc,
                      JSValueConst* argv) {
    (void)this_val;
    if (argc >= 1) {
        const char* msg = JS_ToCString(ctx, argv[0]);
        fprintf(stderr, "%s\n", msg ? msg : "");
        JS_FreeCString(ctx, msg);
    }
    return JS_UNDEFINED;
}

static int32_t on_class_hook(
    void* ctx, const char* name,
    const uint8_t* class_data, size_t class_data_len,
    uint8_t** out_data, size_t* out_len
) {
    (void)ctx; (void)class_data; (void)class_data_len;
    (void)out_data; (void)out_len;
    pthread_mutex_lock(&g_js_lock);
    if (g_ctx != NULL && JS_IsFunction(g_ctx, g_js_hook)) {
        /* The JVM class-name buffer is not NUL-terminated after the name;
         * bound it to printable ASCII before building the JS string. */
        char namebuf[512];
        size_t i = 0;
        while (i < sizeof(namebuf) - 1 && name != NULL && name[i] != '\0' &&
               (unsigned char)name[i] >= 0x20 && (unsigned char)name[i] <= 0x7e) {
            namebuf[i] = name[i];
            i++;
        }
        namebuf[i] = '\0';
        JSValue arg = JS_NewString(g_ctx, namebuf);
        JSValue r = JS_Call(g_ctx, g_js_hook, JS_UNDEFINED, 1, (JSValueConst[]){arg});
        if (JS_IsException(r)) {
            fprintf(stderr, "[hello-js] hook threw:\n");
            dump_error(g_ctx);
        }
        JS_FreeValue(g_ctx, r);
        JS_FreeValue(g_ctx, arg);
    }
    pthread_mutex_unlock(&g_js_lock);
    return 0; /* keep original bytes */
}

/* Directory of the .so via dladdr, for finding hello_hello.js next to it. */
static void module_dir(char* out, size_t out_sz) {
    Dl_info info;
    if (dladdr((void*)cplugin_init, &info) && info.dli_fname) {
        const char* slash = strrchr(info.dli_fname, '/');
        if (slash) {
            size_t n = (size_t)(slash - info.dli_fname);
            if (n >= out_sz) n = out_sz - 1;
            memcpy(out, info.dli_fname, n);
            out[n] = '\0';
            return;
        }
    }
    snprintf(out, out_sz, ".");
}

int32_t cplugin_init(const CPluginApi* api, void* vm, const char* options) {
    (void)vm; (void)options;

    g_rt = JS_NewRuntime();
    g_ctx = JS_NewContext(g_rt);
    if (g_rt == NULL || g_ctx == NULL) {
        fprintf(stderr, "[hello-js] cannot create QuickJS runtime\n");
        return 2;
    }
    /* Default QuickJS stack limit is small; class loads nest (JVM hooks fire
     * while inside another hook), so give the JS stack headroom. */
    JS_SetMaxStackSize(g_rt, 0);
    /* Expose logNative for the JS module body (bare engine: no console). */
    JS_SetPropertyStr(
        g_ctx, JS_GetGlobalObject(g_ctx), "logNative",
        JS_NewCFunction(g_ctx, js_log, "logNative", 1));

    char dir[1024];
    char path[2048];
    module_dir(dir, sizeof(dir));
    snprintf(path, sizeof(path), "%s/hello_hello.js", dir);

    FILE* f = fopen(path, "rb");
    if (f == NULL) {
        fprintf(stderr, "[hello-js] cannot open %s\n", path);
        return 3;
    }
    fseek(f, 0, SEEK_END);
    long sz = ftell(f);
    fseek(f, 0, SEEK_SET);
    char* buf = (char*)malloc((size_t)sz + 1);
    size_t rd = fread(buf, 1, (size_t)sz, f);
    buf[rd] = '\0';
    fclose(f);

    JSValue eval_ret = JS_Eval(g_ctx, buf, strlen(buf), "hello_hello.js", 0);
    free(buf);
    if (JS_IsException(eval_ret)) {
        fprintf(stderr, "[hello-js] eval error:\n");
        dump_error(g_ctx);
        return 4;
    }
    JS_FreeValue(g_ctx, eval_ret);

    JSValue make = JS_GetPropertyStr(g_ctx, JS_GetGlobalObject(g_ctx), "make_hook");
    JSValue hook = JS_Call(g_ctx, make, JS_UNDEFINED, 0, NULL);
    JS_FreeValue(g_ctx, make);
    if (JS_IsException(hook)) {
        fprintf(stderr, "[hello-js] make_hook error:\n");
        dump_error(g_ctx);
        return 5;
    }
    g_js_hook = JS_DupValue(g_ctx, hook);
    JS_FreeValue(g_ctx, hook);

    fprintf(stderr, "[hello-js] cplugin_init (JS module, QuickJS embedded)\n");
    if (api->register_class_hook) {
        int rc = api->register_class_hook(NULL, on_class_hook);
        fprintf(stderr, "[hello-js] register_class_hook rc=%d\n", rc);
        if (rc != 0) return rc;
    } else {
        return 1;
    }

    /* CPlatformApi bridge demo (the same surface a pure-C module uses). */
    if (api->platform) {
        fprintf(stderr, "[hello-js] platform table v%u\n", (unsigned)api->platform->version);
        int n = 0;
        if (api->platform->telemetry_publish_metric)
            n += (api->platform->telemetry_publish_metric("hello_js.init", 42.0, "rc", NULL) == 0);
        if (api->platform->events_publish)
            n += (api->platform->events_publish("hello-js.hello", "{\"phase\":\"init\"}") > 0);
        if (api->platform->scheduler_current_tick)
            (void)api->platform->scheduler_current_tick();
        fprintf(stderr, "[hello-js] platform exercised %d calls, snapshot=%s\n", n,
                api->platform->telemetry_snapshot_json
                    ? api->platform->telemetry_snapshot_json() : "n/a");
    } else {
        fprintf(stderr, "[hello-js] no platform table (old runtime?)\n");
    }
    return 0;
}