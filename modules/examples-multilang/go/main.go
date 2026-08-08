// hello_go — a Crussty module written in Go (cgo, c-shared).
//
// Build:  go build -buildmode=c-shared -o libhello_go.so .
// Deploy: libhello_go.so  +  cplugin.json { "id": "hello_go" }  in modules/
//
// Go exports cplugin_init via //export, so cgo emits an unmangled C symbol.
// Cgo bakes its own runtime into the .so, which the *host* (the JVM
// process) then runs. The class-hook callback is a C trampoline in the cgo
// preamble that forwards to the Go function hook_forward.

package main

/*
#define CPLUG_ABI_NO_ENTRY
#include <stdint.h>
#include <stdlib.h>
#include "../../cplug-abi/cplug-abi.h"

extern int32_t hook_forward(void* ctx, char* name, uint8_t* class_data,
                            size_t class_data_len, uint8_t** out_data,
                            size_t* out_len);

static int32_t hook_trampoline(void* ctx, const char* name,
                               const uint8_t* class_data, size_t class_data_len,
                               uint8_t** out_data, size_t* out_len) {
    return hook_forward(ctx, (char*)name, (unsigned char*)class_data,
                        class_data_len, (unsigned char**)out_data, (size_t*)out_len);
}

static int32_t cgo_register(const CPluginApi* api, void* ctx) {
    if (!api || !api->register_class_hook) return 1;
    return api->register_class_hook(ctx, hook_trampoline);
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
	fmt.Printf("[hello-go] hook fired for %s\n", C.GoString(name))
	return 0 // keep original bytes
}

//export cplugin_init
func cplugin_init(api *C.CPluginApi, vm unsafe.Pointer, options *C.char) C.int32_t {
	_ = vm
	_ = options
	fmt.Printf("[hello-go] cplugin_init (Go module, api v%d)\n", int(api.version))
	rc := C.cgo_register(api, nil)
	fmt.Printf("[hello-go] register_class_hook rc=%d\n", int32(rc))
	return 0
}

func main() {}