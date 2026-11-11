//! __NAME__ — Crussty c-plugin scaffold: registers a class-load hook,
//! resolves kernel classes via JVMTI GetLoadedClasses, and logs through
//! Bukkit's logger once the kernel is up.

use cplug_abi::{CPluginApi, JavaVmPtr};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cplugin_init(
    api: *const CPluginApi,
    vm: JavaVmPtr,
    _options: *const std::ffi::c_char,
) -> i32 {
    eprintln!("[__NAME__] cplugin_init (native, before kernel boot)");
    cplug_sdk::init(api, vm);

    cplug_sdk::hooks::register("org/bukkit/Bukkit", |_name| {
        eprintln!("[__NAME__] Bukkit class load observed (hook chain ok)");
    });

    cplug_sdk::on_kernel_ready("org.bukkit.Bukkit", || {
        let found = cplug_sdk::classes::find_class("org/bukkit/Bukkit").is_some();
        eprintln!(
            "[__NAME__] GetLoadedClasses resolved Bukkit: {}",
            found
        );
        cplug_sdk::run_on_main_thread(|_env| {
            cplug_sdk::log::info("hello from __NAME__ (c-plugin pipeline alive)");
        });
    });
    0
}