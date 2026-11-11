//! Brick 3: platform threads — JVM-attached native threads for modules.
//!
//! Modules must not create raw OS threads for Java-side work: the JVM needs
//! every such thread attached (AttachCurrentThread) before JNI/JVMTI calls.
//! `PlatformThread` wraps a std::thread whose entry point runs with the
//! calling thread attached to the VM, and detaches on exit.
//!
//! # Attachment strategy
//!
//! When a VM is present (`crate::VM`), the spawned thread:
//!
//! 1. probes `GetEnv` (JNI_OK: already attached — never possible for a fresh
//!    std::thread, kept as a defensive branch — and NOT detached afterwards,
//!    since we did not attach it; JNI_EDETACHED: we attach);
//! 2. attaches with `AttachCurrentThread` ([`PlatformThread::spawn`]) or
//!    `AttachCurrentThreadAsDaemon` ([`PlatformThread::spawn_daemon`]),
//!    passing the thread name via `JavaVMAttachArgs` so the JVM-side
//!    `java.lang.Thread` is named from the first instruction and shows up in
//!    `jstack` immediately;
//! 3. renames the JVM thread to `crussty-<name>` via
//!    `java.lang.Thread.currentThread().setName(...)` through a jni-rs
//!    `EnvUnowned` — the 0.22 way of wrapping a raw `JNIEnv*`: the thread
//!    attached itself, so the env is "unowned" from the crate's perspective
//!    and `with_env` gives us the safe `Env` to call through;
//! 4. sets the OS-level thread name (`pthread_setname_np` on Linux, truncated
//!    to the kernel's 15-byte `TASK_COMM_LEN` limit; no-op elsewhere);
//! 5. runs the user closure while attached;
//! 6. detaches (`DetachCurrentThread`) via a RAII guard, so the detach also
//!    happens if the closure panics.
//!
//! `AttachCurrentThread` vs `AttachCurrentThreadAsDaemon` (Oracle JNI spec,
//! JDK-4496330): identical mechanics; the daemon variant only marks the new
//! `java.lang.Thread` as daemon. A non-daemon attached thread keeps
//! `DestroyJavaVM` blocked until it detaches; a daemon one does not. Hence
//! `spawn` (finite, joinable work) uses the regular attach and `spawn_daemon`
//! (fire-and-forget workers) the daemon variant. jni-rs 0.22 removed its
//! daemon attach helper (jni-rs#593, "JavaVMAttachArgs name is dropped too
//! early") because daemon semantics are considered poorly defined; we still
//! need daemon threads for kernel-owned worker pools, so the raw
//! `AttachCurrentThreadAsDaemon` call goes through the JVM function table
//! directly (the same table `lib.rs`'s `with_attached` uses), and the
//! resulting env is wrapped with `EnvUnowned::from_raw` + `with_env` — the
//! exact pattern the jni-rs 0.22 migration guide prescribes for manual
//! attachments.
//!
//! # Panic policy
//!
//! Panics in user closures are caught (`catch_unwind`), logged to stderr, and
//! the thread exits normally — `join()` returns `Ok` and the thread is
//! detached either way. This is deliberate: a panicking module thread must
//! never unwind into the JVM (UB, per the JNI spec) and must never leave the
//! VM with a zombie attachment.
//!
//! # Thread-local context tag
//!
//! `set_thread_tag`/`thread_tag` is a per-thread marker (e.g. "scheduler",
//! "region-7") other bricks (scheduler, side_table cleanup) use to know which
//! module/region a thread belongs to. Tags are `&'static str` — module or
//! region names are fixed at startup; use `Box::leak` for dynamic strings.

use std::cell::Cell;
use std::panic::{catch_unwind, AssertUnwindSafe};

use jni::objects::{JObject, JString, JValue};
use jni::{EnvUnowned, Outcome};

/// Raw JNI bindings (jvmti-bindings' own). Note: in `lib.rs` the prelude glob
/// shadows the `jni` crate with these bindings; this file uses them
/// explicitly so raw vtable calls match `lib.rs`'s `with_attached`.
use jvmti_bindings::sys::jni as vmjni;

thread_local! {
    /// OS-visible name of the current crussty thread ("crussty-<name>").
    static CURRENT_OS_NAME: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
    /// Module/region tag of the current thread, set by the owning brick.
    static THREAD_TAG: Cell<Option<&'static str>> = const { Cell::new(None) };
}

pub struct PlatformThread {
    handle: Option<std::thread::JoinHandle<()>>,
    name: String,
}

impl PlatformThread {
    /// Spawn a JVM-attached thread (`AttachCurrentThread` — a *non-daemon*
    /// `java.lang.Thread`: the JVM will not exit until this thread detaches).
    /// `f` runs with the thread attached; the `JNIEnv` pointer is only valid
    /// inside `f`. See module docs for the full attach sequence and the panic
    /// policy.
    pub fn spawn<F>(name: &str, f: F) -> std::io::Result<Self>
    where
        F: FnOnce() + Send + 'static,
    {
        Self::spawn_impl(name, false, f)
    }

    /// Spawn a JVM-attached daemon thread (`AttachCurrentThreadAsDaemon`).
    /// Identical to [`spawn`](Self::spawn) except the JVM-side thread is a
    /// daemon, so it does not keep the JVM alive on shutdown — use for
    /// fire-and-forget workers.
    pub fn spawn_daemon<F>(name: &str, f: F) -> std::io::Result<Self>
    where
        F: FnOnce() + Send + 'static,
    {
        Self::spawn_impl(name, true, f)
    }

    fn spawn_impl<F>(name: &str, daemon: bool, f: F) -> std::io::Result<Self>
    where
        F: FnOnce() + Send + 'static,
    {
        let name = name.to_string();
        let thread_name = name.clone();
        let entry_name = name.clone();
        let handle = std::thread::Builder::new()
            .name(thread_name)
            .spawn(move || thread_entry(&entry_name, daemon, f))?;
        Ok(Self {
            handle: Some(handle),
            name,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Join the thread. Returns immediately if already joined. Detachment is
    /// handled by the thread itself (also on panic).
    pub fn join(&mut self) {
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

impl Drop for PlatformThread {
    fn drop(&mut self) {
        // JoinHandle left running is fine: with spawn() the attached thread
        // blocks JVM exit until it finishes; with spawn_daemon() it is a
        // daemon and the JVM tears it down on exit.
        let _ = self.handle.take();
    }
}

/// Convenience: spawn a detached VM-attached thread (fire-and-forget).
pub fn spawn_attached<F>(name: &str, f: F)
where
    F: FnOnce() + Send + 'static,
{
    let _ = PlatformThread::spawn(name, f);
}

/// Report the current thread as "os-thread-name(java-visible-name)" when the
/// calling thread is attached to the VM; `None` otherwise (host threads, or
/// no VM). The Java name is fetched live from the JVM (`Thread.getName`), so
/// it reflects renames done by the kernel too.
pub fn current_thread_info() -> Option<String> {
    let java = current_java_thread_name()?;
    let os = CURRENT_OS_NAME
        .with(|n| n.borrow().clone())
        .or_else(|| std::thread::current().name().map(str::to_string))
        .unwrap_or_else(|| "<unnamed>".to_string());
    Some(format!("{os}({java})"))
}

/// Mark the current thread with a context tag (module/region name). Tags are
/// `&'static str` by design — other bricks (scheduler, side_table cleanup)
/// read the tag back without allocation or lifetime bookkeeping; for dynamic
/// strings use `Box::leak`.
pub fn set_thread_tag(tag: &'static str) {
    THREAD_TAG.with(|t| t.set(Some(tag)));
}

/// The context tag previously set with [`set_thread_tag`], if any.
pub fn thread_tag() -> Option<&'static str> {
    THREAD_TAG.with(|t| t.get())
}

/// Whether the current thread is attached to the VM and has a JVM name.
fn current_java_thread_name() -> Option<String> {
    let raw_vm = crate::VM.get().copied().map(|v| v as *mut vmjni::JavaVM)?;
    if raw_vm.is_null() || unsafe { (*raw_vm).is_null() } {
        return None;
    }
    let env = match env_for(raw_vm) {
        EnvFor::Attached(env) => env,
        _ => return None,
    };
    let mut unowned = unsafe { EnvUnowned::from_raw(env as *mut jni::sys::JNIEnv) };
    let outcome = unowned.with_env(|env| -> jni::errors::Result<Option<String>> {
        let thread = env.call_static_method(
            jni::jni_str!("java/lang/Thread"),
            jni::jni_str!("currentThread"),
            jni::jni_sig!("()Ljava/lang/Thread;"),
            &[],
        )?;
        let thread = thread.l()?;
        let name = env.call_method(
            &thread,
            jni::jni_str!("getName"),
            jni::jni_sig!("()Ljava/lang/String;"),
            &[],
        )?;
        let name = name.l()?;
        let js = unsafe { JString::from_raw(env, name.into_raw()) };
        Ok(Some(js.try_to_string(env)?))
    });
    match outcome.into_outcome() {
        Outcome::Ok(name) => name,
        Outcome::Err(_) | Outcome::Panic(_) => {
            clear_pending_exception(&mut unowned);
            None
        }
    }
}

/// Entry point of every PlatformThread: attach (if a VM is present), name the
/// thread on both sides, run the closure, detach on every exit path.
fn thread_entry(name: &str, daemon: bool, f: impl FnOnce() + Send) {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let Some(raw_vm) = crate::VM.get().copied().map(|v| v as *mut vmjni::JavaVM) else {
            run_host(name, f);
            return;
        };
        if raw_vm.is_null() || unsafe { (*raw_vm).is_null() } {
            run_host(name, f);
            return;
        }
        match env_for(raw_vm) {
            EnvFor::Attached(env) => {
                // Already attached (should never happen for a fresh std::thread;
                // defensive). Never detach a thread we did not attach ourselves.
                run_attached(env, name, f);
            }
            EnvFor::Detached => match unsafe { attach_current_thread(raw_vm, name, daemon) } {
                Some(env) => {
                    let _guard = DetachGuard(raw_vm);
                    run_attached(env, name, f);
                }
                None => {
                    eprintln!("[crussty] thread {name}: JVM attach failed; running unattached");
                    run_host(name, f);
                }
            },
            EnvFor::Error(rc) => {
                eprintln!("[crussty] thread {name}: GetEnv failed ({rc}); running unattached");
                run_host(name, f);
            }
        }
    }));
    if let Err(payload) = result {
        log_panic(name, payload);
    }
}

/// Runs with the thread attached: names the thread on both sides, then runs
/// the user closure.
fn run_attached(env: *mut vmjni::JNIEnv, name: &str, f: impl FnOnce() + Send) {
    set_os_thread_name(name);
    set_java_thread_name(env, name);
    CURRENT_OS_NAME.with(|n| *n.borrow_mut() = Some(format!("crussty-{name}")));
    f();
}

/// Host-side execution without a VM: name the OS thread (if we can) and run.
fn run_host(name: &str, f: impl FnOnce() + Send) {
    set_os_thread_name(name);
    CURRENT_OS_NAME.with(|n| *n.borrow_mut() = Some(format!("crussty-{name}")));
    f();
}

/// Attach the calling thread via `AttachCurrentThread[AsDaemon]`, naming the
/// JVM thread up front through `JavaVMAttachArgs` (visible in `jstack` before
/// `Thread#setName` runs). Returns the raw `JNIEnv*` or `None` on failure.
///
/// # Safety
///
/// `raw_vm` must be a valid, non-null `JavaVM*` of the running VM; the caller
/// must be a thread that is not yet attached (checked beforehand via
/// `env_for`).
unsafe fn attach_current_thread(
    raw_vm: *mut vmjni::JavaVM,
    name: &str,
    daemon: bool,
) -> Option<*mut vmjni::JNIEnv> {
    // The name must live for the duration of the call; the VM copies it.
    let c_name = std::ffi::CString::new(name).ok()?;
    let mut attach_args = vmjni::JavaVMAttachArgs {
        version: vmjni::JNI_VERSION_1_2,
        name: c_name.as_ptr() as *mut std::ffi::c_char,
        group: std::ptr::null_mut(),
    };
    let mut env_ptr: *mut vmjni::JNIEnv = std::ptr::null_mut();
    let penv = &mut env_ptr as *mut *mut vmjni::JNIEnv as *mut *mut std::ffi::c_void;
    let args = &mut attach_args as *mut vmjni::JavaVMAttachArgs as *mut std::ffi::c_void;
    let rc = if daemon {
        ((**raw_vm).AttachCurrentThreadAsDaemon)(raw_vm, penv, args)
    } else {
        ((**raw_vm).AttachCurrentThread)(raw_vm, penv, args)
    };
    (rc == vmjni::JNI_OK && !env_ptr.is_null()).then_some(env_ptr)
}

/// RAII detach: created only when this thread attached itself, so
/// `DetachCurrentThread` runs exactly once per thread — on success AND on
/// panic unwinding.
struct DetachGuard(*mut vmjni::JavaVM);

impl Drop for DetachGuard {
    fn drop(&mut self) {
        unsafe {
            ((**self.0).DetachCurrentThread)(self.0);
        }
    }
}

/// Result of probing the VM for an existing attachment of the calling thread.
enum EnvFor {
    /// Already attached; do not detach afterwards.
    Attached(*mut vmjni::JNIEnv),
    /// Not attached; we may attach.
    Detached,
    /// Unexpected `GetEnv` return code.
    Error(i32),
}

fn env_for(raw_vm: *mut vmjni::JavaVM) -> EnvFor {
    let mut env_ptr: *mut vmjni::JNIEnv = std::ptr::null_mut();
    let rc = unsafe {
        ((**raw_vm).GetEnv)(
            raw_vm,
            &mut env_ptr as *mut *mut vmjni::JNIEnv as *mut *mut std::ffi::c_void,
            vmjni::JNI_VERSION_1_6,
        )
    };
    if rc == vmjni::JNI_OK && !env_ptr.is_null() {
        EnvFor::Attached(env_ptr)
    } else if rc == vmjni::JNI_EDETACHED {
        EnvFor::Detached
    } else {
        EnvFor::Error(rc)
    }
}

/// Rename the attached thread in the JVM to `crussty-<name>` via
/// `java.lang.Thread.currentThread().setName(...)`. The Java name is what
/// `jstack`/`jcmd Thread.print` show; the "crussty-" prefix makes platform
/// threads instantly recognizable next to kernel threads. Best-effort:
/// failures are logged, never fatal (the thread keeps its attach-time name).
fn set_java_thread_name(env: *mut vmjni::JNIEnv, name: &str) {
    let java_name = format!("crussty-{name}");
    let mut unowned = unsafe { EnvUnowned::from_raw(env as *mut jni::sys::JNIEnv) };
    let outcome = unowned.with_env(|env| -> jni::errors::Result<()> {
        let thread = env.call_static_method(
            jni::jni_str!("java/lang/Thread"),
            jni::jni_str!("currentThread"),
            jni::jni_sig!("()Ljava/lang/Thread;"),
            &[],
        )?;
        let thread = thread.l()?;
        let jname = JString::from_str(env, &java_name)?;
        env.call_method(
            &thread,
            jni::jni_str!("setName"),
            jni::jni_sig!("(Ljava/lang/String;)V"),
            &[JValue::Object(&jname)],
        )?;
        Ok(())
    });
    match outcome.into_outcome() {
        Outcome::Ok(()) => {}
        Outcome::Err(e) => {
            clear_pending_exception(&mut unowned);
            eprintln!("[crussty] thread {name}: java rename failed: {e:?}");
        }
        Outcome::Panic(p) => log_panic(name, p),
    }
}

/// Clear any pending Java exception on the thread, so a failed bookkeeping
/// call (e.g. `setName`) cannot poison the user closure's own JNI calls —
/// a pending exception makes every subsequent JNI call fail.
fn clear_pending_exception(unowned: &mut EnvUnowned) {
    let _ = unowned.with_env(|env| -> jni::errors::Result<()> {
        if env.exception_check() {
            env.exception_clear();
        }
        Ok(())
    });
}

/// Set the OS-level thread name ("crussty-<name>"). Linux: the kernel keeps
/// at most 15 bytes of `comm` (`TASK_COMM_LEN`), so truncate; elsewhere there
/// is no portable equivalent, so this is a no-op (std's Builder name already
/// set whatever the platform supports).
#[cfg(target_os = "linux")]
fn set_os_thread_name(name: &str) {
    let full = format!("crussty-{name}");
    let truncated = full.bytes().take(15).collect::<Vec<u8>>();
    if let Ok(c) = std::ffi::CString::new(truncated) {
        unsafe {
            libc::pthread_setname_np(libc::pthread_self(), c.as_ptr());
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn set_os_thread_name(_name: &str) {}

/// Log a caught panic payload, best-effort extracting the message.
fn log_panic(name: &str, payload: Box<dyn std::any::Any + Send>) {
    let msg = payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
        .unwrap_or("<non-string panic payload>");
    eprintln!("[crussty] thread {name} panicked: {msg}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    #[test]
    fn spawn_runs_and_joins() {
        let ran = Arc::new(AtomicBool::new(false));
        let r = Arc::clone(&ran);
        // No VM in tests — the thread runs host-side, closure still runs.
        let mut t = PlatformThread::spawn("t-test", move || {
            r.store(true, Ordering::SeqCst);
        })
        .unwrap();
        t.join();
        assert!(ran.load(Ordering::SeqCst));
    }

    #[test]
    fn spawn_daemon_runs_and_joins() {
        // Without a VM, daemon spawn behaves exactly like regular spawn.
        let ran = Arc::new(AtomicBool::new(false));
        let r = Arc::clone(&ran);
        let mut t = PlatformThread::spawn_daemon("t-daemon", move || {
            r.store(true, Ordering::SeqCst);
        })
        .unwrap();
        t.join();
        assert!(ran.load(Ordering::SeqCst));
    }

    #[test]
    fn names_are_kept() {
        let mut t = PlatformThread::spawn("named-thread", || {}).unwrap();
        assert_eq!(t.name(), "named-thread");
        t.join();
    }

    #[test]
    fn panic_is_caught_and_join_succeeds() {
        // Panic policy: panics are logged and the thread exits normally.
        let mut t = PlatformThread::spawn("panicky", || panic!("boom")).unwrap();
        t.join(); // must not unwind here
    }

    #[test]
    fn thread_tag_roundtrip() {
        assert_eq!(thread_tag(), None);
        set_thread_tag("region-42");
        assert_eq!(thread_tag(), Some("region-42"));
        // tag is per-thread: a spawned thread does not see the parent's tag
        let mut t = PlatformThread::spawn("tag-check", || {
            assert_eq!(thread_tag(), None);
            set_thread_tag("scheduler");
            assert_eq!(thread_tag(), Some("scheduler"));
        })
        .unwrap();
        t.join();
        assert_eq!(thread_tag(), Some("region-42"));
    }

    #[test]
    fn current_thread_info_is_none_without_vm() {
        assert_eq!(current_thread_info(), None);
    }
}
