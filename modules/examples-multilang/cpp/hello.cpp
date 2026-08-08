// hello_cpp — a Crussty module written in C++.
//
// Build:  g++ -shared -fPIC -O2 -std=c++17 -o libhello_cpp.so hello.cpp
// Deploy: libhello_cpp.so  +  cplugin.json { "id": "hello_cpp" }  in modules/
//
// Same ABI as the C module; C++ lets us keep state in a small struct. The
// extern "C" guard keeps the exported symbol unmangled.
#include <cstdint>
#include <cstdio>
#include "cplug-abi.h"

namespace {

class HookState {
public:
    explicit HookState(const CPluginApi* api) : api_(api) {}
    const CPluginApi* api() const { return api_; }

    static int32_t onClassLoad(
        void* ctx, const char* name,
        const uint8_t* classData, size_t classDataLen,
        uint8_t** outData, size_t* outLen
    ) {
        auto* self = static_cast<HookState*>(ctx);
        fprintf(stderr, "[hello-cpp] hook fired for %s (api ptr %p)\n",
                name, (void*)self->api());
        (void)classData; (void)classDataLen; (void)outData; (void)outLen;
        return 1; /* keep original bytes */
    }

private:
    const CPluginApi* api_;
};

} // namespace

extern "C" int32_t cplugin_init(const CPluginApi* api, void* vm, const char* options) {
    (void)vm; (void)options;
    static HookState state(api);
    fprintf(stderr, "[hello-cpp] cplugin_init (C++ module, api v%u)\n", api->version);
    if (!api->register_class_hook) {
        fprintf(stderr, "[hello-cpp] register_class_hook unavailable\n");
        return 1;
    }
    int rc = api->register_class_hook(&state, HookState::onClassLoad);
    fprintf(stderr, "[hello-cpp] register_class_hook rc=%d\n", rc);
    return 0;
}