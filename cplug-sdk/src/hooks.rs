//! Pattern hook registry + kernel-ready notifications.
//!
//! The SDK registers a single hook in the runtime pipeline; every class load is
//! dispatched here and passed to registered pattern closures. Callbacks run on
//! the class-loading thread — keep them cheap (spawn if heavy).
//!
//! IMPORTANT: nothing here does JNI/JVMTI work synchronously from
//! `cplugin_init` (that runs on the JVMTI OnLoad thread before the VM is
//! ready — GetLoadedClasses there can crash the boot). All class lookups run
//! on background threads.

use crate::sdk_glob;
use std::cell::Cell;
use std::sync::{Arc, Mutex, OnceLock};

type Cb = Box<dyn Fn(&str) + Send + Sync>;
type ByteCb = Box<dyn Fn(&str, &[u8]) -> Option<Vec<u8>> + Send + Sync>;
type OnceJob = Option<Box<dyn FnOnce() + Send>>;

static HOOKS: OnceLock<Mutex<Vec<(String, Cb)>>> = OnceLock::new();
static BYTE_HOOKS: OnceLock<Mutex<Vec<(String, ByteCb)>>> = OnceLock::new();

fn registry() -> &'static Mutex<Vec<(String, Cb)>> {
    HOOKS.get_or_init(|| Mutex::new(Vec::new()))
}

fn byte_registry() -> &'static Mutex<Vec<(String, ByteCb)>> {
    BYTE_HOOKS.get_or_init(|| Mutex::new(Vec::new()))
}

/// Register a pattern hook: `cb(name)` fires on every class load matching
/// `pattern`. Glob syntax: `*` any run (incl. '/'), `?` one char,
/// e.g. "org/bukkit/**".
pub fn register(pattern: &str, cb: impl Fn(&str) + Send + Sync + 'static) {
    registry()
        .lock()
        .unwrap()
        .push((pattern.to_string(), Box::new(cb)));
}

/// Reentrancy guard (DEADLOCK LAW, observed on Paper 1.21.8): a patch
/// callback that triggers a class load of its own (ASM ClassWriter resolving
/// bootstrap-method constants, helper classes, …) re-enters the JVMTI
/// ClassFileLoadHook on the SAME thread. dispatch/dispatch_bytes hold the
/// registry mutex across callbacks — a nested dispatch then self-deadlocks
/// on a non-reentrant std::sync::Mutex and, transitively, every other
/// thread's hook invocation blocks behind the frozen registry. Nested loads
/// are helper/asm-internal classes — they must pass through UNPATCHED.
thread_local! {
    static IN_DISPATCH: Cell<bool> = const { Cell::new(false) };
}

/// RAII: sets the flag on create, ALWAYS clears on drop (panic-safe —
/// a panicking hook body must not permanently disable the thread's hooks).
struct DispatchGuard;
impl DispatchGuard {
    fn try_new() -> Option<Self> {
        if IN_DISPATCH.with(Cell::get) {
            return None; // nested load from inside a callback — pass through
        }
        IN_DISPATCH.with(|c| c.set(true));
        Some(Self)
    }
}
impl Drop for DispatchGuard {
    fn drop(&mut self) {
        IN_DISPATCH.with(|c| c.set(false));
    }
}

/// Dispatch a class name to all matching hooks.
pub fn dispatch(name: &str) {
    let Some(_g) = DispatchGuard::try_new() else {
        return; // nested load from inside a callback — pass through
    };
    let hooks = registry().lock().unwrap();
    for (pat, cb) in hooks.iter() {
        if sdk_glob::matches(pat, name) {
            cb(name);
        }
    }
}

/// Register a byte-level hook: `cb(name, bytes)` fires on every class load
/// matching `pattern` and may return patched bytes (the SDK allocates the
/// replacement via the runtime's JVMTI allocator and hands it to the JVM).
/// Hooks chain in registration order: each gets the previous output.
pub fn register_bytes(
    pattern: &str,
    cb: impl Fn(&str, &[u8]) -> Option<Vec<u8>> + Send + Sync + 'static,
) {
    byte_registry()
        .lock()
        .unwrap()
        .push((pattern.to_string(), Box::new(cb)));
}

/// Apply all byte hooks matching `name` to `data`; returns the final patched
/// bytes, or None if no hook modified them.
pub fn dispatch_bytes(name: &str, data: &[u8]) -> Option<Vec<u8>> {
    let _g = DispatchGuard::try_new()?; // nested → None (unpatched)
    let hooks = byte_registry().lock().unwrap();
    let mut current: Option<Vec<u8>> = None;
    for (pat, cb) in hooks.iter() {
        if sdk_glob::matches(pat, name) {
            if let Some(p) = cb(name, current.as_deref().unwrap_or(data)) {
                current = Some(p);
            }
        }
    }
    current
}

/// Run `cb` (on a fresh thread) when `class_name` has loaded. Fires exactly
/// once, by background polling — no JVM work in the caller (safe to call from
/// `cplugin_init`).
pub fn on_kernel_ready(class_name: &str, cb: impl FnOnce() + Send + 'static) {
    let slot: Arc<Mutex<OnceJob>> =
        Arc::new(Mutex::new(Some(Box::new(cb) as Box<dyn FnOnce() + Send>)));
    let class_name = class_name.to_string();
    std::thread::spawn(move || {
        // Give the VM time to finish initializing before the first JNI attach;
        // AttachCurrentThread from a fresh thread during VM init can SIGSEGV.
        std::thread::sleep(std::time::Duration::from_millis(3000));
        const POLL_MS: u64 = 200;
        const MAX_MS: u64 = 120_000;
        let mut waited = 0u64;
        let found = loop {
            if let Some(c) = crate::classes::find_class(&class_name) {
                let _ = c;
                break true;
            }
            if waited >= MAX_MS {
                break false;
            }
            std::thread::sleep(std::time::Duration::from_millis(POLL_MS));
            waited += POLL_MS;
        };
        if !found {
            eprintln!("[cplug-sdk] kernel-ready wait timed out for {class_name}");
        }
        let f = slot.lock().unwrap().take();
        if let Some(f) = f {
            f();
        }
    });
}
