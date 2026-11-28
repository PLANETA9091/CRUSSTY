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
///
/// MULTI-MODULE ISOLATION: each user callback runs under catch_unwind — a
/// panicking hook body used to unwind through `sdk_dispatch_hook`'s
/// `extern "C"` boundary (= process abort, taking every OTHER loaded module
/// and the whole server down with it). Now the broken closure is removed
/// from the registry for the process lifetime (a dead hook that panics on
/// every load would otherwise slow every matching class load forever) and
/// the remaining hooks — other modules' included — keep running.
pub fn dispatch(name: &str) {
    let Some(_g) = DispatchGuard::try_new() else {
        return; // nested load from inside a callback — pass through
    };
    let hooks = registry().lock().unwrap();
    let mut dead: Vec<usize> = Vec::new();
    for (i, (pat, cb)) in hooks.iter().enumerate() {
        if sdk_glob::matches(pat, name) {
            let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| cb(name)));
            if r.is_err() {
                eprintln!(
                    "[cplug-sdk] class hook #{i} ({pat}) PANICKED on '{name}' — hook removed for the process lifetime; other hooks continue"
                );
                dead.push(i);
            }
        }
    }
    drop(hooks);
    if !dead.is_empty() {
        let mut hooks = registry().lock().unwrap();
        for i in dead.into_iter().rev() {
            if i < hooks.len() {
                let _ = hooks.remove(i);
            }
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
///
/// MULTI-MODULE ISOLATION: per-callback catch_unwind (see `dispatch`) — a
/// panicking byte hook is removed for the process lifetime and the chain
/// continues with the bytes the chain had BEFORE the broken hook ran.
pub fn dispatch_bytes(name: &str, data: &[u8]) -> Option<Vec<u8>> {
    let _g = DispatchGuard::try_new()?; // nested → None (unpatched)
    let hooks = byte_registry().lock().unwrap();
    let mut current: Option<Vec<u8>> = None;
    let mut dead: Vec<usize> = Vec::new();
    for (i, (pat, cb)) in hooks.iter().enumerate() {
        if sdk_glob::matches(pat, name) {
            let input = current.as_deref().unwrap_or(data);
            let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                cb(name, input)
            }));
            match r {
                Ok(Some(p)) => current = Some(p),
                Ok(None) => {}
                Err(_) => {
                    eprintln!(
                        "[cplug-sdk] byte hook #{i} ({pat}) PANICKED on '{name}' — hook removed for the process lifetime; other hooks continue"
                    );
                    dead.push(i);
                }
            }
        }
    }
    drop(hooks);
    if !dead.is_empty() {
        let mut hooks = byte_registry().lock().unwrap();
        for i in dead.into_iter().rev() {
            if i < hooks.len() {
                let _ = hooks.remove(i);
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn hits() -> &'static AtomicUsize {
        static H: OnceLock<AtomicUsize> = OnceLock::new();
        H.get_or_init(|| AtomicUsize::new(0))
    }

    #[test]
    fn panicking_name_hook_is_removed_others_keep_firing() {
        hits().store(0, Ordering::SeqCst);
        let pat = "test/panic/name/**";
        let healthy_pat = "test/panic/healthy/**";
        // The BROKEN hook is registered FIRST: it matches, panics, and must
        // not prevent the healthy hook (the "other module") from running.
        register(pat, |_name| panic!("module bug: hook exploded"));
        register(healthy_pat, |_name| {
            hits().fetch_add(1, Ordering::SeqCst);
        });

        dispatch("test/panic/name/One");
        // The broken hook's panic was caught (did not abort the test
        // process); the healthy hook — a different "module" — still fired.
        assert_eq!(hits().load(Ordering::SeqCst), 0, "healthy pattern differs — no hit yet");
        dispatch("test/panic/healthy/One");
        assert_eq!(hits().load(Ordering::SeqCst), 1, "healthy hook fired across dispatches");

        // The broken hook is GONE from the registry (removed, not skipped):
        // its pattern no longer has any matching entry.
        let n = registry().lock().unwrap().iter().filter(|(p, _)| p == pat).count();
        assert_eq!(n, 0, "panicking hook removed for the process lifetime");
        let m = registry()
            .lock()
            .unwrap()
            .iter()
            .filter(|(p, _)| p == healthy_pat)
            .count();
        assert_eq!(m, 1, "healthy hook stays registered");

        // Cleanup: leave the shared registries as we found them.
        let mut reg = registry().lock().unwrap();
        reg.retain(|(p, _)| p != pat && p != healthy_pat);
    }

    #[test]
    fn panicking_byte_hook_chain_continues_with_pre_panic_bytes() {
        let pat = "test/panic/bytes/**";
        // Chain: broken hook (panics) then healthy hook (appends 'H').
        // The healthy hook must see the ORIGINAL bytes (the chain state
        // before the broken hook ran), and the dispatch must return the
        // healthy hook's patch — the panic degrades to "hook absent".
        register_bytes(pat, |_name, _data| panic!("byte hook exploded"));
        register_bytes(pat, |_name, data| {
            let mut v = data.to_vec();
            v.push(b'H');
            Some(v)
        });

        let out = dispatch_bytes("test/panic/bytes/A", b"XY");
        assert_eq!(out.as_deref(), Some(&b"XYH"[..]), "chain continued past the panic with pre-panic bytes");

        // The broken byte hook is gone; the healthy one stays.
        let reg = byte_registry().lock().unwrap();
        assert_eq!(reg.iter().filter(|(p, _)| p == pat).count(), 1);
        drop(reg);
        let mut reg = byte_registry().lock().unwrap();
        reg.retain(|(p, _)| p != pat);
    }

    #[test]
    fn healthy_byte_hook_chain_still_patches() {
        let pat = "test/panic/healthybytes/**";
        register_bytes(pat, |_name, data| {
            let mut v = data.to_vec();
            v.insert(0, b'A');
            Some(v)
        });
        register_bytes(pat, |_name, data| {
            let mut v = data.to_vec();
            v.push(b'B');
            Some(v)
        });
        let out = dispatch_bytes("test/panic/healthybytes/A", b"Z");
        assert_eq!(out.as_deref(), Some(&b"AZB"[..]));
        let mut reg = byte_registry().lock().unwrap();
        reg.retain(|(p, _)| p != pat);
    }
}
