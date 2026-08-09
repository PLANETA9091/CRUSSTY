// __NAME__ — a Crussty module written in plain C.
//
// Build:  gcc -shared -fPIC -O2 -I../../cplug-sdk-c/include \
//             -o lib__NAME__.so hello.c
// Deploy: lib__NAME__.so  +  cplugin.json { "id": "__NAME__" }  in modules/
//
// Registers a class-file hook; on every kernel class load after boot it logs
// once via stderr (the JVM forwards stderr to the server console). Also
// exercises the CPlatformApi bridge: the platform bricks are the runtime's
// Rust internals (events, scheduler, telemetry, ...) exposed to every
// language through this trailing function table — see cplug-abi.h.
#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include "cplug-abi.h"

static const CPluginApi* g_api;
static int32_t metric_ok = -99;

/* Event-bus callback: fired on the brick's own thread, not the JVM main. */
static void on_event(const char* event, const char* payload, void* ctx) {
    (void)ctx;
    fprintf(stderr, "[__NAME__] event %s payload=%s\n", event, payload);
}

/* Scheduler-injected work: runs on the kernel main thread. */
static void on_task(void* ctx) {
    (void)ctx;
    fprintf(stderr, "[__NAME__] injected task ran on kernel thread\n");
}

/* Class-file hook: the hot-patch pipeline. Return 0 + out_data/out_len to
 * replace the class bytes; nonzero keeps the original. Here we only observe.
 */
static int32_t on_class_load(
    void* ctx, const char* name,
    const uint8_t* class_data, size_t class_data_len,
    uint8_t** out_data, size_t* out_len
) {
    (void)ctx; (void)class_data; (void)class_data_len;
    (void)out_data; (void)out_len;
    if (strstr(name, "net/minecraft/server") != NULL) {
        fprintf(stderr, "[__NAME__] class load hook fired for %s\n", name);
    }
    return 1; /* keep original bytes */
}

int32_t cplugin_init(const CPluginApi* api, void* vm, const char* options) {
    (void)vm; (void)options;
    fprintf(stderr, "[__NAME__] cplugin_init (C module, api v%u)\n", api->version);

    if (api->register_class_hook) {
        int rc = api->register_class_hook(NULL, on_class_load);
        fprintf(stderr, "[__NAME__] register_class_hook rc=%d\n", rc);
    } else {
        fprintf(stderr, "[__NAME__] register_class_hook unavailable\n");
        return 1;
    }

    if (api->platform == NULL) {
        fprintf(stderr, "[__NAME__] no platform table (old runtime?)\n");
        return 0;
    }
    fprintf(stderr, "[__NAME__] platform table v%u\n", (unsigned)api->platform->version);

    int rc = 0;
    if (api->platform->events_subscribe) {
        uint64_t tok = api->platform->events_subscribe("platform.tick_boundary", on_event, NULL);
        api->platform->events_unsubscribe(tok); /* demo: subscribe then release */
        fprintf(stderr, "[__NAME__] subscribed+unsubscribed token=%llu\n",
                (unsigned long long)tok);
        rc++;
    }
    if (api->platform->events_publish) {
        size_t n = api->platform->events_publish("__NAME__.hello", "{\"phase\":\"init\"}");
        fprintf(stderr, "[__NAME__] published, %zu sync handlers\n", n);
        rc++;
    }
    if (api->platform->scheduler_inject) {
        uint64_t tok = api->platform->scheduler_inject("__NAME__", on_task, NULL);
        fprintf(stderr, "[__NAME__] injected task token=%llu\n", (unsigned long long)tok);
        rc++;
    }
    if (api->platform->telemetry_publish_metric) {
        metric_ok = api->platform->telemetry_publish_metric("__NAME__.init", 42.0, "rc", NULL);
        fprintf(stderr, "[__NAME__] metric rc=%d\n", metric_ok);
        rc += (metric_ok == 0);
    }
    /* threads_spawn is not exercised here: spawning an attached thread from
     * cplugin_init (inside JNI_CreateJavaVM) faults the VM — the same reason
     * the runtime defers its hook-class installs by 2s. */
    if (api->platform->scheduler_current_tick) {
        fprintf(stderr, "[__NAME__] current tick=%llu\n",
                (unsigned long long)api->platform->scheduler_current_tick());
    }
    if (api->platform->telemetry_snapshot_json) {
        fprintf(stderr, "[__NAME__] snapshot=%s\n",
                api->platform->telemetry_snapshot_json());
    }
    fprintf(stderr, "[__NAME__] platform exercised %d calls, init done\n", rc);
    return 0;
}