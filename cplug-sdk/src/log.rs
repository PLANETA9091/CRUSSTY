//! Kernel logging via `java.util.logging.Logger` (bootstrap class, resolvable
//! with plain JNI — no GetLoadedClasses needed). Falls back to stderr until
//! the kernel is up.

use crate::classes::{find_class, method, static_method, ClassRef};
use crate::jni_util::with_attached;
use jvmti_bindings::prelude::*;
use std::sync::{Mutex, OnceLock};

enum Level {
    Info,
    Warning,
}

struct LoggerIds {
    bukkit: ClassRef,
    get_logger: usize,
    info: usize,
    warning: usize,
}

static IDS: OnceLock<Mutex<Option<LoggerIds>>> = OnceLock::new();

fn ids() -> &'static Mutex<Option<LoggerIds>> {
    IDS.get_or_init(|| Mutex::new(None))
}

fn resolve() -> Option<LoggerIds> {
    let bukkit = find_class("org/bukkit/Bukkit")?;
    let logger = find_class("java/util/logging/Logger")?;
    with_attached(|env| {
        let get_logger = static_method(
            env,
            bukkit.as_jclass(),
            "getLogger",
            "()Ljava/util/logging/Logger;",
        )?;
        let info = method(env, logger.as_jclass(), "info", "(Ljava/lang/String;)V")?;
        let warning = method(env, logger.as_jclass(), "warning", "(Ljava/lang/String;)V")?;
        Some(LoggerIds {
            bukkit,
            get_logger,
            info,
            warning,
        })
    })
    .flatten()
}

fn emit(level: Level, msg: &str) {
    let mut cache = ids().lock().unwrap();
    if cache.is_none() {
        *cache = resolve();
    }
    let Some(ids) = cache.as_ref() else {
        eprintln!("[cplug-sdk] log dropped (kernel not ready): {msg}");
        return;
    };
    let mid = match level {
        Level::Info => ids.info,
        Level::Warning => ids.warning,
    } as jni::jmethodID;
    let get_logger = ids.get_logger as jni::jmethodID;
    let ok = with_attached(|env| {
        let s = env.new_string_utf(msg)?;
        let logger = env.call_static_object_method(ids.bukkit.as_jclass(), get_logger, &[]);
        if logger.is_null() {
            env.delete_local_ref(s);
            return None;
        }
        env.call_void_method(logger, mid, &[jni::jvalue { l: s }]);
        env.delete_local_ref(s);
        env.delete_local_ref(logger);
        Some(())
    })
    .is_some();
    if !ok {
        eprintln!("[cplug-sdk] log emit failed: {msg}");
    }
}

/// Log an info message through the kernel logger (stderr fallback before the
/// kernel is up).
pub fn info(msg: &str) {
    emit(Level::Info, msg);
}

/// Log a warning through the kernel logger.
pub fn warn(msg: &str) {
    emit(Level::Warning, msg);
}

pub fn log_info(msg: &str) {
    info(msg);
}
