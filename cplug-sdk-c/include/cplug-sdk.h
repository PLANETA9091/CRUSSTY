// cplug-sdk.h — C binding of cplug-sdk (hooks, classes, main-thread
// dispatch, kernel-ready notifications, logging).
//
// The Rust SDK implements the heavy JNI/JVMTI patterns (cross-loader class
// resolution, main-thread execution, kernel logging, pattern hooks) once;
// this header exposes them to C/C++/Python(ctypes)/JS(QuickJS shim)/Zig/Go
// modules. Link modules against libcplug_sdk_c (.a static or .so shared)
// from the cplug-sdk-c crate.
//
// This header is the hand-written equivalent of
// cplug-sdk-c/src/lib.rs — keep the two in lockstep.

#ifndef CPLUG_SDK_C_H
#define CPLUG_SDK_C_H

#include <stddef.h>
#include <stdint.h>

#include "cplug-abi.h" /* CPluginApi, JavaVmPtr */

#ifdef __cplusplus
extern "C" {
#endif

/*
 * Must be called first, from cplugin_init: stores the vm, registers the
 * SDK's single pipeline hook.
 */
void cplug_sdk_init(const CPluginApi* api, JavaVmPtr vm);

/* Raw JavaVM* stored by cplug_sdk_init (for JNI bindings). */
JavaVmPtr cplug_sdk_vm(void);

/*
 * Register a name-only pattern hook: cb(ctx, name) fires on every class load
 * matching `pattern`. Glob syntax: `*` any run (incl. '/'), `?` one char,
 * e.g. "org/bukkit/**".
 *
 * cb must stay valid for the whole module lifetime (never unregistered).
 * Returns 0 on success, negative on bad args.
 */
typedef void (*cplug_hook_fn)(void* ctx, const char* name);
int32_t cplug_sdk_hook_register(const char* pattern, void* ctx, cplug_hook_fn cb);

/*
 * Register a byte-level hook: cb(ctx, name, data, len, *out_len) fires on
 * every class load matching `pattern` and MAY return replacement class
 * bytes (allocate with malloc; the SDK copies them and calls free(3)).
 * Return NULL to keep the original class. cb must stay valid for the whole
 * module lifetime.
 */
typedef const uint8_t* (*cplug_byte_hook_fn)(
    void* ctx, const char* name,
    const uint8_t* data, size_t len, size_t* out_len);
int32_t cplug_sdk_hook_register_bytes(const char* pattern, void* ctx, cplug_byte_hook_fn cb);

/*
 * Run cb(ctx) exactly once, on a fresh thread, when `class_name` has loaded
 * (kernel-ready notification, background polling). Safe from cplugin_init.
 */
typedef void (*cplug_ready_fn)(void* ctx);
int32_t cplug_sdk_on_kernel_ready(const char* class_name, void* ctx, cplug_ready_fn cb);

/*
 * Queue cb(ctx, env) to run on the server's main thread (with an attached
 * JNIEnv*). Safe from any thread; jobs wait while the kernel boots. cb
 * stays valid for the module lifetime.
 */
typedef void (*cplug_main_fn)(void* ctx, void* env /* JNIEnv* */);
int32_t cplug_sdk_run_on_main_thread(void* ctx, cplug_main_fn cb);

/*
 * Resolve a loaded class across all class loaders by name ("org/bukkit/Bukkit"
 * or dotted). Returns a JNI global ref (process-lifetime, cached) or NULL.
 * Call from an attached thread (e.g. a main-thread callback).
 */
void* cplug_sdk_find_class(const char* name);

/* Like find_class, but polls every 200ms up to timeout_ms. */
void* cplug_sdk_wait_class(const char* name, uint64_t timeout_ms);

/* Re-run the class-file hook chain for an already loaded class
 * (JVMTI retransform). 1 on success, 0 on failure. */
int32_t cplug_sdk_retransform_class(const char* name);

/* Kernel logger (java.util.logging; stderr until the kernel is up). */
void cplug_sdk_log_info(const char* msg);
void cplug_sdk_log_warn(const char* msg);

/* Clear a pending JNI exception (describe + clear). 1 if one was pending. */
int32_t cplug_sdk_clear_exception(void* env /* JNIEnv* */);

/*
 * Attach the current thread if needed; returns a JNIEnv* or NULL. The thread
 * stays attached; call cplug_sdk_detach_current_thread only if YOU attached
 * it.
 */
void* cplug_sdk_attach_current_thread(void);
void  cplug_sdk_detach_current_thread(void);

#ifdef __cplusplus
}
#endif

#endif /* CPLUG_SDK_C_H */