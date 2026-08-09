// __NAME__ — a Crussty module written in Go (cgo, c-shared).
//
// Build:  go build -buildmode=c-shared -o lib__NAME__.so .
// Deploy: lib__NAME__.so  +  module.json { "id": "__NAME__" }  in modules/
//
// Go exports cplugin_init via //export, so cgo emits an unmangled C symbol.
// Cgo bakes its own runtime into the .so, which the *host* (the JVM
// process) then runs. The class-hook callback is a C trampoline in the cgo
// preamble that forwards to the Go function hook_forward. The CPlatformApi
// bridge (platform bricks) is exercised through the same preamble.

package main

/*
#define CPLUG_ABI_NO_ENTRY
#include <stdint.h>
#include <stdlib.h>
#include "cplug-abi.h"

extern int32_t hook_forward(void* ctx, char* name, uint8_t* class_data,
                            size_t class_data_len, uint8_t** out_data,
                            size_t* out_len);
extern void event_forward(char* event, char* payload, void* ctx);
extern void task_forward(void* ctx);

static int32_t hook_trampoline(void* ctx, const char* name,
                               const uint8_t* class_data, size_t class_data_len,
                               uint8_t** out_data, size_t* out_len) {
    return hook_forward(ctx, (char*)name, (unsigned char*)class_data,
                        class_data_len, (unsigned char**)out_data, (size_t*)out_len);
}

static void event_trampoline(const char* event, const char* payload, void* ctx) {
    event_forward((char*)event, (char*)payload, ctx);
}

static void task_trampoline(void* ctx) {
    task_forward(ctx);
}

static int32_t cgo_register(const CPluginApi* api, void* ctx) {
    if (!api || !api->register_class_hook) return 1;
    return api->register_class_hook(ctx, hook_trampoline);
}

// Platform-bricks demo; returns number of successful calls.
static int32_t cgo_platform(const CPluginApi* api, uint64_t* tick_out) {
    if (!api || !api->platform) return -1;
    int32_t rc = 0;
    if (api->platform->events_subscribe) {
        uint64_t tok = api->platform->events_subscribe("platform.tick_boundary", event_trampoline, NULL);
        api->platform->events_unsubscribe(tok); // demo: subscribe then release
        rc++;
    }
    if (api->platform->events_publish) {
        (void)api->platform->events_publish("__NAME__.hello", "{\"phase\":\"init\"}");
        rc++;
    }
    if (api->platform->scheduler_inject) {
        (void)api->platform->scheduler_inject("__NAME__", task_trampoline, NULL);
        rc++;
    }
    if (api->platform->telemetry_publish_metric) {
        int32_t mrc = api->platform->telemetry_publish_metric("__NAME__.init", 42.0, "rc", NULL);
        if (mrc == 0) rc++;
    }
    if (api->platform->scheduler_current_tick && tick_out) {
        *tick_out = api->platform->scheduler_current_tick();
    }
    return rc;
}

static const char* cgo_snapshot(const CPluginApi* api) {
    if (!api || !api->platform || !api->platform->telemetry_snapshot_json) return NULL;
    return api->platform->telemetry_snapshot_json();
}
*/
import "C"

import (
	"fmt"
	"unsafe"
)

//export hook_forward
func hook_forward(ctx unsafe.Pointer, name *C.char, classData *C.uint8_t,
	classDataLen C.size_t, outData **C.uint8_t, outLen *C.size_t) C.int32_t {
	_ = ctx
	_ = classData
	_ = classDataLen
	_ = outData
	_ = outLen
	fmt.Printf("[__NAME__] hook fired for %s\n", C.GoString(name))
	return 0 // keep original bytes
}

//export event_forward
func event_forward(event *C.char, payload *C.char, ctx unsafe.Pointer) {
	_ = ctx
	fmt.Printf("[__NAME__] event %s payload=%s\n", C.GoString(event), C.GoString(payload))
}

//export task_forward
func task_forward(ctx unsafe.Pointer) {
	_ = ctx
	fmt.Printf("[__NAME__] injected task ran on kernel thread\n")
}

//export cplugin_init
func cplugin_init(api *C.CPluginApi, vm unsafe.Pointer, options *C.char) C.int32_t {
	_ = vm
	_ = options
	fmt.Printf("[__NAME__] cplugin_init (Go module, api v%d)\n", int(api.version))
	rc := C.cgo_register(api, nil)
	fmt.Printf("[__NAME__] register_class_hook rc=%d\n", int32(rc))

	if api.platform == nil {
		fmt.Printf("[__NAME__] no platform table (old runtime?)\n")
		return 0
	}
	fmt.Printf("[__NAME__] platform table v%d\n", uint32(api.platform.version))

	var tick C.uint64_t
	n := C.cgo_platform(api, &tick)
	if n < 0 {
		fmt.Printf("[__NAME__] platform table broken\n")
		return 0
	}
	fmt.Printf("[__NAME__] platform exercised %d calls, current tick=%d\n",
		int32(n), uint64(tick))
	if snap := C.cgo_snapshot(api); snap != nil {
		fmt.Printf("[__NAME__] snapshot=%s\n", C.GoString(snap))
	}
	return 0
}

func main() {}