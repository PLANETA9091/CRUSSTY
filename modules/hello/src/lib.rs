//! hello — proof c-plugin (V2-DESIGN): registers a class-load hook in the
//! agent's ClassFileLoadHook pipeline, resolves kernel classes via JVMTI
//! GetLoadedClasses (cross-loader safe), and logs through Bukkit's logger via
//! JNI once the kernel is up.

use cplug_abi::{CPluginApi, JavaVmPtr};

/// The single required export (cplug-abi contract).
///
/// # Safety
/// `api` must be a valid CPluginApi owned by the agent, `vm` the live
/// JavaVM pointer, `options` a NUL-terminated string for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cplugin_init(
    api: *const CPluginApi,
    vm: JavaVmPtr,
    _options: *const std::ffi::c_char,
) -> i32 {
    eprintln!("[hello-plugin] cplugin_init (native, before kernel boot)");
    cplug_sdk::init(api, vm);

    // Pattern hook: fires through the agent's class-file hook pipeline when
    // org/bukkit/Bukkit is defined — proves the hook chain reaches modules.
    // The callback runs on the class-loading thread, so it stays cheap.
    cplug_sdk::hooks::register("org/bukkit/Bukkit", |_name| {
        eprintln!("[hello-plugin] Bukkit class load observed (hook chain ok)");
    });

    // GetLoadedClasses + JNI: once the kernel is up, log through
    // Bukkit.getLogger() — from the main thread, via the SDK's server queue
    // (Bukkit.getLogger() NPEs until MinecraftServer.getServer() is set; the
    // main-thread flush delivers only once the server object exists).
    cplug_sdk::on_kernel_ready("org.bukkit.Bukkit", || {
        let found = cplug_sdk::classes::find_class("org/bukkit/Bukkit").is_some();
        eprintln!(
            "[hello-plugin] GetLoadedClasses resolved Bukkit: {}",
            found
        );
        cplug_sdk::run_on_main_thread(|_env| {
            cplug_sdk::log::info("hello from native c-plugin (v2 pipeline alive)");
        });
    });
    0
}
