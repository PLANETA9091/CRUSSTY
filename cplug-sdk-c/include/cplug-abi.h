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
// Since CPAPI 2.1 the struct carries one more trailing field: `platform` —
// the C-visible surface of the runtime's platform bricks (events, scheduler,
// telemetry, storage, ...). Old modules compiled against the 3-field struct
// keep working unchanged: the field is appended at the end, so offsets of
// the original fields never move.
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

#define CPAPI_VERSION 3u        /* minimum version every module must accept */
#define CPAPI_PLATFORM_VERSION 1u

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
 * - return 0  -> patched: use [out_data]/[out_len] as the new class bytes;
 *   return nonzero -> keep original bytes (plugin error or no-op).
 */
typedef int32_t (*cpapi_class_hook_fn)(
    void*         ctx,
    const char*   name,
    const uint8_t* class_data,
    size_t        class_data_len,
    uint8_t**     out_data,
    size_t*       out_len);

/* ---------------------------------------------------------------------------
 * Platform bricks C bridge (CPAPI 2.1+).
 *
 * Modules are dlopened with RTLD_LOCAL and can never link the runtime
 * directly; instead the agent hands every module this function table
 * (`CPlatformApi`, appended to `CPluginApi` as a trailing pointer). Strings
 * are NUL-terminated UTF-8; NULL means empty. Callbacks fire on the brick's
 * own thread (module context must not assume the JVM main thread).
 * ------------------------------------------------------------------------- */

/* Event-bus callback; `event` and `payload_json` valid during the call. */
typedef void (*cpapi_event_cb)(const char* event, const char* payload_json, void* ctx);
/* Scheduler-injected work; runs on the kernel main thread. */
typedef void (*cpapi_task_cb)(void* ctx);
/* Crash-isolation callback. */
typedef void (*cpapi_fault_cb)(
    int32_t signal, uint64_t timestamp_unix, uint64_t count, void* ctx);
/* Packet hook: 0=pass, 1=drop, 2=disconnect. `pkt` valid during the call. */
typedef int32_t (*cpapi_packet_cb)(const void* pkt, void* ctx);
/* Save-lifecycle callback; kind 0=autosave/1=manual; status 0=ok/1=failed. */
typedef void (*cpapi_save_cb)(
    int32_t kind, int32_t status, uint64_t chunks_written, uint64_t duration_ms, void* ctx);

/* Storage provider callbacks (all optional). */
typedef const char* (*cpapi_storage_name_cb)(void* ctx);
typedef int32_t (*cpapi_storage_read_cb)(
    void* ctx, int32_t rx, int32_t rz, int32_t cx, int32_t cz,
    const uint8_t** out, size_t* out_len); /* 1=found 0=not_found -1=corrupt */
typedef int32_t (*cpapi_storage_write_cb)(
    void* ctx, int32_t rx, int32_t rz, int32_t cx, int32_t cz,
    const uint8_t* payload, size_t payload_len); /* 0=ok, nonzero=error */
typedef int32_t (*cpapi_storage_begin_save_cb)(void* ctx);
typedef int32_t (*cpapi_storage_end_save_cb)(void* ctx);

/* Complete brick-API table. Append new entries at the END only. */
typedef struct CPlatformApi {
    uint32_t version; /* = CPAPI_PLATFORM_VERSION */

    /* events (brick 6) */
    uint64_t (*events_subscribe)(const char* event, cpapi_event_cb cb, void* ctx);
    int32_t  (*events_unsubscribe)(uint64_t token);
    size_t   (*events_publish)(const char* event, const char* payload_json);

    /* scheduler (brick 3) */
    uint64_t (*scheduler_inject)(const char* tag, cpapi_task_cb cb, void* ctx);
    uint64_t (*scheduler_current_tick)(void);
    size_t   (*scheduler_injected_pending)(void);

    /* telemetry (brick 7); `unit` and `labels_json` may be NULL */
    int32_t  (*telemetry_publish_metric)(
        const char* name, double value, const char* unit, const char* labels_json);
    const char* (*telemetry_snapshot_json)(void); /* valid until next call */

    /* signals (brick 9) */
    int32_t  (*signals_on_fault)(cpapi_fault_cb cb, void* ctx);
    uint64_t (*signals_fault_count)(void);
    int32_t  (*signals_crash_log)(const char* path);

    /* network (brick 6b) */
    int32_t  (*network_add_hook)(cpapi_packet_cb cb, void* ctx);
    int32_t  (*network_attach_conn)(uint64_t conn_id, uint64_t uuid_hi, uint64_t uuid_lo);
    int32_t  (*network_detach_conn)(uint64_t conn_id);
    int32_t  (*network_conn_state)(uint64_t conn_id, uint8_t state_code);
    size_t   (*network_conn_count)(void);

    /* storage (brick 5); one-shot vtable capture */
    int32_t  (*storage_install)(
        void* ctx, cpapi_storage_name_cb name, cpapi_storage_read_cb read_chunk,
        cpapi_storage_write_cb write_chunk, cpapi_storage_begin_save_cb begin_save,
        cpapi_storage_end_save_cb end_save);
    int32_t  (*storage_active)(void);

    /* threads (brick threads) */
    int32_t  (*threads_spawn)(const char* name, cpapi_task_cb f, void* ctx);
    int32_t  (*threads_spawn_daemon)(const char* name, cpapi_task_cb f, void* ctx);
    int32_t  (*threads_current_name)(char* out, size_t out_len);

    /* transform (brick 1); injection: 0=MethodEntry, 1=BeforeCall(helper) */
    int32_t  (*transform_register_rule)(
        const char* class_pattern, const char* method, const char* descriptor,
        int32_t injection, const char* helper);

    /* save_events (brick 8) */
    int32_t  (*save_events_on_save)(cpapi_save_cb cb, void* ctx);

    /* hot_reload (brick 10) */
    int32_t  (*hot_reload_module)(const char* id);
    int32_t  (*hot_reload_enter)(const char* id);
    int32_t  (*hot_reload_leave)(const char* id);

    /* side_table (brick 2); needs an attached thread */
    int32_t  (*side_table_key)(void* obj, uint64_t* out);
    int32_t  (*side_table_named)(const char* name, uint64_t* out);
} CPlatformApi;

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
    /* Claim a global resource key before registering it with the JVM
     * ("class:a/b/C", "native:a/b/C#name:sig"). 0 free or already owned by
     * the same `owner`; -1 owned by another module (skip registration). */
    int32_t (*claim)(unsigned long owner, const char* key);
    /* (CPAPI 3.1+) Platform-bricks table; NULL on runtimes without it. */
    const CPlatformApi* platform;
} CPluginApi;

/*
 * Some toolchains (cgo, ctypes, bindgen) declare cplugin_init themselves and
 * get "conflicting types" when this header re-declares it. Define
 * CPLUG_ABI_NO_ENTRY before including to suppress the prototype while keeping
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