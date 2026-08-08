// cplug-abi.h — the ONLY contract between crussty-runtime and modules.
//
// A module is a shared library exporting:
//
//     int32_t cplugin_init(const CPluginApi* api, [JavaVM]* vm, const char* options);
//
// Everything else is raw JVM access (JNI/JVMTI through the JavaVM*). The api
// gives three tiny services: register a class-file hook, allocate replacement
// class bytes through the JVMTI allocator, and retransform a loaded class.
//
// This header is the hand-written equivalent of the cplug-abi Rust crate
// (cplug-abi/src/lib.rs). Keep the two in lockstep; CPAPI_VERSION is the
// wire version the runtime checks.

#ifndef CPLUG_ABI_H
#define CPLUG_ABI_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

#define CPAPI_VERSION 2u

/* Opaque JavaVM* — cast to your JNI bindings' JavaVM type. */
typedef void* JavaVmPtr;

/*
 * Optional class-file hook, invoked for every class load before the class is
 * defined (JVMTI CLASS_FILE_LOAD_HOOK). The hot-patch pipeline: hooks get the
 * raw class bytes and may return patched bytes.
 *
 * Contract:
 * - `out_data`/`out_len` must be set only when replacing the class; the
 *   replacement buffer must come from `api->jvmti_allocate`.
 * - return 0  -> patched: use *out_data/*out_len as the new class bytes;
 *   return nonzero -> keep original bytes (plugin error or no-op).
 */
typedef int32_t (*cpapi_class_hook_fn)(
    void*         ctx,
    const char*   name,
    const uint8_t* class_data,
    size_t        class_data_len,
    uint8_t**     out_data,
    size_t*       out_len);

/* Agent -> plugin services. */
typedef struct CPluginApi {
    uint32_t version;
    /* Register `hook` (with plugin-owned `ctx`) in the runtime's pipeline. */
    int32_t (*register_class_hook)(void* ctx, cpapi_class_hook_fn hook);
    /* Allocate a buffer via JVMTI (the VM frees it with Deallocate). */
    uint8_t* (*jvmti_allocate)(size_t size);
    /* Retransform a loaded class by internal name; re-enters the hook
     * pipeline with its current bytes. 0 on success, negative on failure. */
    int32_t (*retransform_class)(const char* name);
} CPluginApi;

/*
 * Some toolchains (cgo, ctypes, bindgen) declare cplugin_init themselves and
 * get "conflicting types" when this header re-declares it. Define
 * CPLUG_ABI_NO_ENTRY before including to skip the prototype while keeping
 * CPluginApi / JavaVmPtr.
 */
#ifndef CPLUG_ABI_NO_ENTRY
/* The single required module export. options carries runtime info, e.g.
 * "modules=<dir>;versions=<dir>;kernel=<jar>". `vm` is a raw JavaVM* — cast
 * to your JNI bindings' type. Return 0 on success. */
int32_t cplugin_init(const CPluginApi* api, void* vm, const char* options);
#endif /* CPLUG_ABI_NO_ENTRY */

#ifdef __cplusplus
}
#endif

#endif /* CPLUG_ABI_H */