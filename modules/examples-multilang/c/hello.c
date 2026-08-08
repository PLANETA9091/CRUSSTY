// hello_c — a Crussty module written in plain C.
//
// Build:  gcc -shared -fPIC -O2 -o libhello_c.so hello.c
// Deploy: libhello_c.so  +  cplugin.json { "id": "hello_c" }  in modules/
//
// Registers a class-file hook; on every kernel class load after boot it logs
// once via stderr (the JVM forwards stderr to the server console).
#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include "cplug-abi.h"

static void* vm;

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
    fprintf(stderr, "[hello-c] class load hook fired for %s\n", name);
    return 1; /* keep original bytes */
}

int32_t cplugin_init(const CPluginApi* api, void* vm_ptr, const char* options) {
    (void)options;
    vm = vm_ptr;
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