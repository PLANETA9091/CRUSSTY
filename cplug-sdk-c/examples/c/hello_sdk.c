// hello_sdk — Crussty module skeleton on the SDK C binding.
//
// The same shape as hello.c from c-hello@multilang examples, but using the SDK's
// convenience layer instead of raw JNI/JVMTI: pattern hooks (no manual
// name matching), kernel-ready notification, main-thread dispatch, logging
// and cross-loader class resolution.
//
// Build:  cargo build --release -p cplug-sdk-c    (produces libcplug_sdk_c.a)
//         cc -shared -fPIC -O2 -I../../cplug-sdk-c/include \
//             -o libhello_sdk.so hello_sdk.c ../../cplug-sdk-c/target/release/libcplug_sdk_c.a
// Deploy: libhello_sdk.so + cplugin.json { "id": "hello_sdk" } in modules/

#include <stdio.h>
#include <stdint.h>
#include <string.h>

#include "cplug-abi.h"
#include "cplug-sdk.h"

/* name-only hook: fires for every class matching "org/bukkit/**" */
static void on_bukkit_class(void* ctx, const char* name) {
    (void)ctx;
    static int count = 0;
    if (++count <= 5) {
        cplug_sdk_log_info("[hello_sdk] saw bukkit class (name hook)");
    }
}

/* byte hook: log the class, keep the original bytes (return NULL) */
static const uint8_t* keep_bytes(
    void* ctx, const char* name,
    const uint8_t* data, size_t len, size_t* out_len
) {
    (void)ctx; (void)data;
    static int count = 0;
    if (++count <= 3) {
        char buf[512];
        snprintf(buf, sizeof(buf), "[hello_sdk] class %s (%zu bytes)", name, len);
        cplug_sdk_log_info(buf);
    }
    *out_len = 0;
    return NULL;
}

/* main-thread callback (JNIEnv* attached) */
static void on_main(void* ctx, void* env) {
    (void)ctx;
    (void)env;
    cplug_sdk_log_info("[hello_sdk] inside a main-thread task");
}

static void on_kernel_ready(void* ctx) {
    (void)ctx;
    cplug_sdk_log_info("[hello_sdk] kernel is up!");
    cplug_sdk_run_on_main_thread(NULL, on_main);
}

int32_t cplugin_init(const CPluginApi* api, void* vm, const char* options) {
    (void)options;
    cplug_sdk_init(api, vm);

    cplug_sdk_hook_register("org/bukkit/**", NULL, on_bukkit_class);
    cplug_sdk_hook_register_bytes("net/minecraft/**", NULL, keep_bytes);
    cplug_sdk_on_kernel_ready("org/bukkit/Bukkit", NULL, on_kernel_ready);

    return 0;
}