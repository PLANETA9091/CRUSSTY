// __NAME__ — a Crussty module written in C++.
//
// Build:  g++ -shared -fPIC -O2 -std=c++17 -I../../cplug-sdk-c/include \
//             -o lib__NAME__.so hello.cpp
// Deploy: lib__NAME__.so  +  module.json { "id": "__NAME__" }  in modules/
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
            fprintf(stderr, "[__NAME__] hook fired for %s (api ptr %p)\n",
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
    fprintf(stderr, "[__NAME__] event %s payload=%s\n", ev, payload);
}

extern "C" void cppOnTask(void* ctx) {
    (void)ctx;
    fprintf(stderr, "[__NAME__] injected task ran on kernel thread\n");
}

} // namespace

extern "C" int32_t cplugin_init(const CPluginApi* api, void* vm, const char* options) {
    (void)vm; (void)options;
    static HookState state(api);
    fprintf(stderr, "[__NAME__] cplugin_init (C++ module, api v%u)\n", api->version);
    if (!api->register_class_hook) {
        fprintf(stderr, "[__NAME__] register_class_hook unavailable\n");
        return 1;
    }
    int rc = api->register_class_hook(&state, HookState::onClassLoad);
    fprintf(stderr, "[__NAME__] register_class_hook rc=%d\n", rc);

    if (api->platform == nullptr) {
        fprintf(stderr, "[__NAME__] no platform table (old runtime?)\n");
        return 0;
    }
    fprintf(stderr, "[__NAME__] platform table v%u\n", (unsigned)api->platform->version);

    int rc2 = 0;
    if (api->platform->events_subscribe) {
        uint64_t tok = api->platform->events_subscribe("platform.tick_boundary", cppOnEvent, nullptr);
        api->platform->events_unsubscribe(tok);
        fprintf(stderr, "[__NAME__] subscribed+unsubscribed token=%llu\n",
                (unsigned long long)tok);
        rc2++;
    }
    if (api->platform->events_publish) {
        size_t n = api->platform->events_publish("__NAME__.hello", "{\"phase\":\"init\"}");
        fprintf(stderr, "[__NAME__] published, %zu sync handlers\n", n);
        rc2++;
    }
    if (api->platform->scheduler_inject) {
        uint64_t tok = api->platform->scheduler_inject("__NAME__", cppOnTask, nullptr);
        fprintf(stderr, "[__NAME__] injected task token=%llu\n", (unsigned long long)tok);
        rc2++;
    }
    if (api->platform->telemetry_publish_metric) {
        int32_t mrc = api->platform->telemetry_publish_metric("__NAME__.init", 42.0, "rc", nullptr);
        fprintf(stderr, "[__NAME__] metric rc=%d\n", mrc);
        rc2 += (mrc == 0);
    }
    /* threads_spawn is not exercised from cplugin_init: an attached thread
     * spawned inside JNI_CreateJavaVM faults the VM. */
    if (api->platform->scheduler_current_tick) {
        fprintf(stderr, "[__NAME__] current tick=%llu\n",
                (unsigned long long)api->platform->scheduler_current_tick());
    }
    if (api->platform->telemetry_snapshot_json) {
        fprintf(stderr, "[__NAME__] snapshot=%s\n",
                api->platform->telemetry_snapshot_json());
    }
    fprintf(stderr, "[__NAME__] platform exercised %d calls, init done\n", rc2);
    return 0;
}