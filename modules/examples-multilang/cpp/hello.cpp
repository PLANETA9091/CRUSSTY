// hello_cpp — a Crussty module written in C++.
//
// Build:  g++ -shared -fPIC -O2 -std=c++17 -I../../cplug-sdk-c/include \
//             -o libhello_cpp.so hello.cpp
// Deploy: libhello_cpp.so  +  cplugin.json { "id": "hello_cpp" }  in modules/
//
// Same ABI as the C module; C++ lets us keep state in a small struct. The
// extern "C" guard keeps the exported symbol unmangled. Also exercises the
// CPlatformApi bridge (platform bricks), see cplug-abi.h.
#include <cstdint>
#include <cstdio>
#include <cstring>
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
        if (strstr(name, "net/minecraft/server") != nullptr) {
            fprintf(stderr, "[hello-cpp] hook fired for %s (api ptr %p)\n",
                    name, (void*)self->api());
        }
        (void)classData; (void)classDataLen; (void)outData; (void)outLen;
        return 1; /* keep original bytes */
    }

private:
    const CPluginApi* api_;
};

extern "C" void cppOnEvent(const char* ev, const char* payload, void* ctx) {
    (void)ctx;
    fprintf(stderr, "[hello-cpp] event %s payload=%s\n", ev, payload);
}

extern "C" void cppOnTask(void* ctx) {
    (void)ctx;
    fprintf(stderr, "[hello-cpp] injected task ran on kernel thread\n");
}

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

    if (api->platform == nullptr) {
        fprintf(stderr, "[hello-cpp] no platform table (old runtime?)\n");
        return 0;
    }
    fprintf(stderr, "[hello-cpp] platform table v%u\n", (unsigned)api->platform->version);

    int rc2 = 0;
    if (api->platform->events_subscribe) {
        uint64_t tok = api->platform->events_subscribe("platform.tick_boundary", cppOnEvent, nullptr);
        api->platform->events_unsubscribe(tok);
        fprintf(stderr, "[hello-cpp] subscribed+unsubscribed token=%llu\n",
                (unsigned long long)tok);
        rc2++;
    }
    if (api->platform->events_publish) {
        size_t n = api->platform->events_publish("hello_cpp.hello", "{\"phase\":\"init\"}");
        fprintf(stderr, "[hello-cpp] published, %zu sync handlers\n", n);
        rc2++;
    }
    if (api->platform->scheduler_inject) {
        uint64_t tok = api->platform->scheduler_inject("hello-cpp", cppOnTask, nullptr);
        fprintf(stderr, "[hello-cpp] injected task token=%llu\n", (unsigned long long)tok);
        rc2++;
    }
    if (api->platform->telemetry_publish_metric) {
        int32_t mrc = api->platform->telemetry_publish_metric("hello_cpp.init", 42.0, "rc", nullptr);
        fprintf(stderr, "[hello-cpp] metric rc=%d\n", mrc);
        rc2 += (mrc == 0);
    }
    /* threads_spawn is not exercised from cplugin_init: an attached thread
     * spawned inside JNI_CreateJavaVM faults the VM. */
    if (api->platform->scheduler_current_tick) {
        fprintf(stderr, "[hello-cpp] current tick=%llu\n",
                (unsigned long long)api->platform->scheduler_current_tick());
    }
    if (api->platform->telemetry_snapshot_json) {
        fprintf(stderr, "[hello-cpp] snapshot=%s\n",
                api->platform->telemetry_snapshot_json());
    }
    fprintf(stderr, "[hello-cpp] platform exercised %d calls, init done\n", rc2);
    return 0;
}