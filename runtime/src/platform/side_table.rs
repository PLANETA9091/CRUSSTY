//! Brick 2: side tables — module-owned data attached to JVM objects.
//!
//! JVMTI's RetransformClasses cannot add fields to already-loaded classes,
//! so modules that need per-object state (e.g. "which region owns this
//! entity") use a side table keyed by the object's identity. Implemented as
//! a Rust map keyed by a 64-bit object token; a process-wide registry holds a
//! JNI weak global reference per token so keys never dangle while the object
//! is alive, and a background sweep purges entries once the JVM collects the
//! object.
//!
//! # Keying strategy (why not raw handles)
//!
//! JNI object references are opaque: the JNI spec and CERT JNI02-J state that
//! reference values must never be treated as constant or unique, and
//! Android's JNI tips explicitly forbid using `jobject` values as keys —
//! `NewGlobalRef`/`NewWeakGlobalRef` may return different values for the same
//! object on consecutive calls, and a deleted reference's memory can be
//! reused by the VM. So we never key on a handle.
//!
//! Instead each object maps to a process-unique, monotonically increasing
//! `u64` token ([`ObjectKey`]) that can never collide and is never reused.
//! The token is bound to the object by the registry: one jni-rs
//! [`jni::objects::Weak`] (a JNI *weak global reference*) per token, held
//! alive for as long as the object lives. Because we hold the reference
//! open, the token→object identity mapping is stable.
//!
//! Same-object deduplication (the same object must always yield the same
//! token) uses `System.identityHashCode` as a first-level index — the
//! IdentityHashMap pattern — verified with `IsSameObject` against the bucket
//! candidates, since identity hash codes are 32-bit and can collide. The
//! candidate handle-address-as-key idea was rejected: a fresh weak ref for
//! the same object gets a fresh handle, so the same object would receive two
//! different keys on consecutive calls.
//!
//! # GC cleanup
//!
//! A single background thread (started lazily, only when a VM is present)
//! every 60 seconds attaches to the JVM, walks the registry calling
//! `IsSameObject(env, ref, NULL)` (via
//! [`jni::objects::Weak::is_garbage_collected`]), drops collected weak refs
//! while attached (so `DeleteWeakGlobalRef` runs — failure to do so is a
//! slow leak, per the JNI spec), then notifies every live table so it can
//! drop the dead entry and fire its optional `on_collect` callback.
//!
//! Without a VM (`crate::VM` unset, e.g. unit tests) every JNI-dependent
//! path degrades gracefully: [`key_from_jobject`] returns `None`.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock, Weak as StdWeak};
use std::time::Duration;

use jni::errors;
use jni::objects::{Global, JClass, JObject, Weak};
use jni::sys::jobject;
use jni::{Env, JValue, JavaVM, ScopeToken, jni_sig, jni_str};

/// Identity key for a JVM object. Modules obtain it by boxing the raw
/// `jobject`/`jlong` handle through [`key_from_jobject`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectKey(pub u64);

/// Seconds between garbage-collection sweeps of the reference registry.
const GC_INTERVAL: Duration = Duration::from_secs(60);

/// Next token handed out. Starts at 1 so `ObjectKey(0)` can never be issued.
static NEXT_KEY: AtomicU64 = AtomicU64::new(1);

fn next_key() -> ObjectKey {
    ObjectKey(NEXT_KEY.fetch_add(1, Ordering::Relaxed))
}

/// Access to the VM pointer stored by `lib.rs` (`crate::VM`), as a plain
/// `usize`. `None` when the runtime was not loaded into a JVM.
pub(crate) fn vm_raw() -> Option<usize> {
    crate::VM.get().copied().filter(|&p| p != 0)
}

/// The process-wide jni-rs [`JavaVM`] handle, built from [`vm_raw`] on first
/// use. This also initializes jni-rs's own `JavaVM` singleton, which
/// `Weak::drop` relies on to call `DeleteWeakGlobalRef` safely from any
/// thread.
fn jvm() -> Option<&'static JavaVM> {
    static JVM: OnceLock<JavaVM> = OnceLock::new();
    let raw = vm_raw()?;
    // Safety: `crate::VM` stores the non-null `JavaVM*` handed to
    // Agent_OnLoad/JNI_OnLoad, which is valid for the JVM's lifetime.
    Some(JVM.get_or_init(|| unsafe { JavaVM::from_raw(raw as *mut jni::sys::JavaVM) }))
}

/// Run `f` with an [`Env`] for the *current* thread, obtaining the env
/// without attaching when the thread is already attached (native method
/// frames, JVMTI callbacks — the [`EnvUnowned`](jni::EnvUnowned) semantics)
/// and attaching/detaching for the duration otherwise.
fn with_env_current<R>(f: impl FnOnce(&mut Env<'_>) -> errors::Result<R>) -> Option<R> {
    let vm = jvm()?;
    let mut scope = ScopeToken::default();
    // Safety: `scope` lives on this stack frame and we hold at most one
    // AttachGuard for the current thread, so the "one Env per top frame" rule
    // of jni-rs holds.
    if let Ok(mut guard) = unsafe { vm.get_env_attachment(&mut scope) } {
        return f(guard.borrow_env_mut()).ok();
    }
    vm.attach_current_thread_for_scope(|env| f(env)).ok()
}

/// Cached global reference to `java.lang.System` for `identityHashCode`.
/// The `Option` only turns into `None` if the very first lookup fails (e.g.
/// a pending exception), after which every call degrades to `None`/`Err`.
static SYSTEM_CLASS: OnceLock<Option<Global<JClass<'static>>>> = OnceLock::new();

fn system_class(env: &mut Env<'_>) -> errors::Result<&'static Global<JClass<'static>>> {
    let init = SYSTEM_CLASS.get_or_init(|| {
        env.find_class(jni_str!("java/lang/System"))
            .and_then(|cls| env.new_global_ref(cls))
            .ok()
    });
    init.as_ref()
        .ok_or(errors::Error::NullPtr("side table: java/lang/System lookup failed"))
}

/// `System.identityHashCode(obj)`: stable per-object 32-bit identity hash.
/// Used only as a first-level index; identity is always confirmed with
/// `IsSameObject`, so collisions are harmless.
fn identity_hash_code(env: &mut Env<'_>, obj: &JObject<'_>) -> errors::Result<i32> {
    let cls = system_class(env)?;
    let v = env.call_static_method(
        cls,
        jni_str!("identityHashCode"),
        jni_sig!("(Ljava/lang/Object;)I"),
        &[JValue::Object(obj)],
    )?;
    v.i()
}

/// One registry entry: the weak global reference that pins the token to its
/// object, plus the object's identity hash code for the dedupe index.
struct RegistryEntry {
    weak: Weak<JObject<'static>>,
    hc: i32,
}

/// Process-wide token registry. `by_key` maps token → weak ref; `by_hc` is
/// the identity-hash index used to dedupe [`key_from_jobject`] calls (bucket
/// candidates are confirmed with `IsSameObject`, see module docs).
#[derive(Default)]
struct Registry {
    by_key: HashMap<ObjectKey, RegistryEntry>,
    by_hc: HashMap<i32, Vec<ObjectKey>>,
}

impl Registry {
    fn rebuild_index(&mut self) {
        self.by_hc.clear();
        for (&k, e) in &self.by_key {
            self.by_hc.entry(e.hc).or_default().push(k);
        }
    }
}

static REGISTRY: OnceLock<Mutex<Registry>> = OnceLock::new();

fn registry() -> &'static Mutex<Registry> {
    REGISTRY.get_or_init(Default::default)
}

/// Create a stable identity token for a JVM object. Same object → same token
/// for as long as the object is alive; the registry keeps a weak global
/// reference open per token, so the mapping cannot dangle.
///
/// # Safety contract for the caller
///
/// `raw` must be a valid JNI reference (local, global or weak) to a live
/// object, owned by the current thread's JNI frame. Passing a stale or
/// deleted reference is undefined behavior at the JNI level.
///
/// Returns `None` when no VM is present (`crate::VM` unset) or any JNI call
/// fails (e.g. pending exception, OOM while creating the weak ref).
pub fn key_from_jobject(raw: jobject) -> Option<ObjectKey> {
    if raw.is_null() {
        return None;
    }
    jvm()?;
    ensure_gc_thread();
    with_env_current(|env| key_from_env(env, raw))
}

fn key_from_env(env: &mut Env<'_>, raw: jobject) -> errors::Result<ObjectKey> {
    // Safety: the caller guarantees `raw` is a valid reference of the current
    // thread's frame; it is only used to create a weak global ref, never
    // stored raw, and `env` bounds its lifetime.
    let obj = unsafe { JObject::from_raw(env, raw) };
    let hc = identity_hash_code(env, &obj)?;
    let mut reg = registry().lock().unwrap_or_else(|p| p.into_inner());
    if let Some(bucket) = reg.by_hc.get(&hc) {
        for &k in bucket {
            if let Some(e) = reg.by_key.get(&k) {
                if env.is_same_object(&e.weak, &obj)? {
                    return Ok(k);
                }
            }
        }
    }
    let key = next_key();
    let weak = env.new_weak_ref(obj)?;
    reg.by_key.insert(key, RegistryEntry { weak, hc });
    reg.by_hc.entry(hc).or_default().push(key);
    Ok(key)
}

/// Tables that want to be notified when one of their keys is collected.
/// Implemented by [`SideTableInner`]; the sweep calls it outside any lock so
/// user callbacks never run under the registry or table-registry mutex.
trait GcNotifiable: Send + Sync {
    fn notify_collected(&self, key: ObjectKey);
}

type GcTableWeak = StdWeak<dyn GcNotifiable + Send + Sync>;

/// Per-table callback fired when one of its entries is garbage collected.
type OnCollect = Arc<dyn Fn(ObjectKey) + Send + Sync>;

/// Next table id handed out. Starts at 1 so table 0 is never issued.
static NEXT_TABLE_ID: AtomicU64 = AtomicU64::new(1);

/// Every live side table, weakly, so the sweep can auto-remove collected
/// keys and fire per-table `on_collect` callbacks.
static TABLES_REGISTRY: OnceLock<Mutex<HashMap<u64, GcTableWeak>>> = OnceLock::new();

fn tables_registry() -> &'static Mutex<HashMap<u64, GcTableWeak>> {
    TABLES_REGISTRY.get_or_init(Default::default)
}

/// Shared state of a [`SideTable`]. Kept behind an [`Arc`] so the sweep can
/// hold the table alive briefly while it notifies.
struct SideTableInner<V> {
    map: Mutex<HashMap<ObjectKey, V>>,
    on_collect: Mutex<Option<OnCollect>>,
}

impl<V: Send + 'static> GcNotifiable for SideTableInner<V> {
    fn notify_collected(&self, key: ObjectKey) {
        let removed = self.map.lock().unwrap_or_else(|p| p.into_inner()).remove(&key).is_some();
        if !removed {
            return;
        }
        if let Some(cb) = self.on_collect.lock().unwrap_or_else(|p| p.into_inner()).clone() {
            cb(key);
        }
    }
}

/// A side table of key -> value with cheap identity semantics. Keys come from
/// [`key_from_jobject`]; entries for objects the JVM has collected are
/// removed by the background sweep, which also fires the optional
/// `on_collect` callback if one is set.
pub struct SideTable<V> {
    id: u64,
    inner: Arc<SideTableInner<V>>,
}

impl<V: Clone + Send + 'static> SideTable<V> {
    pub fn new() -> Self {
        let inner: Arc<SideTableInner<V>> = Arc::new(SideTableInner {
            map: Mutex::new(HashMap::new()),
            on_collect: Mutex::new(None),
        });
        let id = NEXT_TABLE_ID.fetch_add(1, Ordering::Relaxed);
        // Coerce to the trait object first: `Arc::downgrade` does not coerce
        // its `&Arc<T>` argument, but a value binding does.
        let trait_arc: Arc<dyn GcNotifiable + Send + Sync> = inner.clone();
        let weak: GcTableWeak = Arc::downgrade(&trait_arc);
        tables_registry().lock().unwrap_or_else(|p| p.into_inner()).insert(id, weak);
        Self { id, inner }
    }

    /// Replace the callback fired when an entry's object is garbage
    /// collected (and the entry is removed). Replaces any previous callback.
    pub fn set_on_collect(&self, cb: OnCollect) {
        *self.inner.on_collect.lock().unwrap_or_else(|p| p.into_inner()) = Some(cb);
    }

    /// Return the value for `k`, or insert `default_fn()`'s result and
    /// return it. `default_fn` runs only when the key is absent.
    pub fn get_or_insert(&self, k: ObjectKey, default_fn: impl FnOnce() -> V) -> V {
        let mut map = self.inner.map.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(v) = map.get(&k) {
            return v.clone();
        }
        let v = default_fn();
        map.insert(k, v.clone());
        v
    }
}

impl<V: Clone> SideTable<V> {
    pub fn insert(&self, k: ObjectKey, v: V) {
        self.inner.map.lock().unwrap_or_else(|p| p.into_inner()).insert(k, v);
    }

    pub fn get(&self, k: &ObjectKey) -> Option<V> {
        self.inner.map.lock().unwrap_or_else(|p| p.into_inner()).get(k).cloned()
    }

    pub fn remove(&self, k: &ObjectKey) -> Option<V> {
        self.inner.map.lock().unwrap_or_else(|p| p.into_inner()).remove(k)
    }

    pub fn contains(&self, k: &ObjectKey) -> bool {
        self.inner.map.lock().unwrap_or_else(|p| p.into_inner()).contains_key(k)
    }

    pub fn len(&self) -> usize {
        self.inner.map.lock().unwrap_or_else(|p| p.into_inner()).len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Snapshot of all current values (clone).
    pub fn values(&self) -> Vec<V> {
        self.inner.map.lock().unwrap_or_else(|p| p.into_inner()).values().cloned().collect()
    }
}

impl<V: Clone + Send + 'static> Default for SideTable<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V> Drop for SideTable<V> {
    fn drop(&mut self) {
        tables_registry().lock().unwrap_or_else(|p| p.into_inner()).remove(&self.id);
    }
}

/// Process-wide registry of named side tables for inter-module sharing.
/// Modules should use their own SideTable instances instead of this global
/// unless cross-module access is genuinely needed.
static NAMED: OnceLock<Mutex<HashMap<String, ObjectKey>>> = OnceLock::new();

pub fn named_table(name: &str) -> Option<ObjectKey> {
    NAMED
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap()
        .get(name)
        .copied()
}

/// Start the background GC thread once. Only ever called with a VM present,
/// so the thread is only created when it has something to do.
fn ensure_gc_thread() {
    static STARTED: OnceLock<()> = OnceLock::new();
    STARTED.get_or_init(|| {
        let Some(vm) = jvm().cloned() else { return };
        if let Err(e) = std::thread::Builder::new()
            .name("crussty-side-table-gc".to_string())
            .spawn(move || gc_thread_loop(vm))
        {
            eprintln!("[crussty-runtime] side-table GC thread spawn failed: {e}");
        }
    });
}

fn gc_thread_loop(vm: JavaVM) {
    loop {
        std::thread::sleep(GC_INTERVAL);
        // Attach for the sweep; the attach guard detaches the thread when the
        // closure returns, so the thread is never left attached to the JVM.
        let _ = vm.attach_current_thread(|env| -> errors::Result<()> {
            gc_sweep(env);
            Ok(())
        });
    }
}

/// One sweep: probe every registered weak ref for collection and hand the
/// dead keys to [`gc_collect`]. Runs attached (see [`gc_thread_loop`]) so
/// dropping the collected `Weak`s issues `DeleteWeakGlobalRef` correctly.
fn gc_sweep(env: &mut Env<'_>) {
    let dead: Vec<ObjectKey> = {
        let reg = registry().lock().unwrap_or_else(|p| p.into_inner());
        reg.by_key
            .iter()
            .filter(|(_, e)| e.weak.is_garbage_collected(env).unwrap_or(false))
            .map(|(k, _)| *k)
            .collect()
    };
    gc_collect(&dead);
}

/// Remove `dead` keys from the registry and notify every live table
/// (auto-removal + `on_collect` callback). The JNI-free core of the sweep,
/// also usable as a test injection point. Callers that removed registry
/// entries must be JVM-attached so the `Weak` drops can call
/// `DeleteWeakGlobalRef`.
fn gc_collect(dead: &[ObjectKey]) {
    if dead.is_empty() {
        return;
    }
    {
        let mut reg = registry().lock().unwrap_or_else(|p| p.into_inner());
        for k in dead {
            reg.by_key.remove(k);
        }
        reg.rebuild_index();
    }
    let mut tables: Vec<Arc<dyn GcNotifiable + Send + Sync>> = Vec::new();
    {
        let mut ts = tables_registry().lock().unwrap_or_else(|p| p.into_inner());
        ts.retain(|_, w| match w.upgrade() {
            Some(t) => {
                tables.push(t);
                true
            }
            None => false,
        });
    }
    for t in &tables {
        for k in dead {
            t.notify_collected(*k);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    #[test]
    fn insert_get_remove() {
        let t = SideTable::<u64>::new();
        let k = ObjectKey(42);
        assert!(!t.contains(&k));
        t.insert(k, 7);
        assert!(t.contains(&k));
        assert_eq!(t.get(&k), Some(7));
        assert_eq!(t.remove(&k), Some(7));
        assert!(!t.contains(&k));
    }

    #[test]
    fn key_tokens_are_unique_and_nonzero() {
        let a = next_key();
        let b = next_key();
        let c = next_key();
        assert_ne!(a.0, 0);
        assert_ne!(a, b);
        assert_ne!(b, c);
        assert_ne!(a, c);
    }

    #[test]
    fn key_from_jobject_without_vm_returns_none() {
        // No JVM in unit tests: crate::VM is unset, so every JNI path must
        // degrade to None, including for a non-null bogus pointer.
        assert!(crate::VM.get().is_none());
        let fake: jobject = 0x1234usize as *mut _;
        assert!(key_from_jobject(fake).is_none());
        assert!(key_from_jobject(std::ptr::null_mut()).is_none());
    }

    #[test]
    fn registry_identity_index_rebuilds_after_removal() {
        let mut reg = Registry::default();
        let k1 = ObjectKey(1);
        let k2 = ObjectKey(2);
        // Weak::null() needs no JVM and performs no JNI calls on drop.
        reg.by_key.insert(k1, RegistryEntry { weak: Weak::null(), hc: 7 });
        reg.by_key.insert(k2, RegistryEntry { weak: Weak::null(), hc: 7 });
        reg.rebuild_index();
        assert_eq!(reg.by_hc.get(&7).unwrap().len(), 2);
        reg.by_key.remove(&k1);
        reg.rebuild_index();
        assert_eq!(reg.by_hc.get(&7).unwrap(), &vec![k2]);
        assert!(!reg.by_hc.contains_key(&8));
    }

    #[test]
    fn gc_collect_fires_callback_and_removes_entries() {
        let t = SideTable::<u64>::new();
        let fired = Arc::new(AtomicUsize::new(0));
        let seen = Arc::new(Mutex::new(Vec::new()));
        let fired2 = fired.clone();
        let seen2 = seen.clone();
        t.set_on_collect(Arc::new(move |k| {
            fired2.fetch_add(1, Ordering::SeqCst);
            seen2.lock().unwrap().push(k);
        }));
        let k1 = ObjectKey(11);
        let k2 = ObjectKey(22);
        t.insert(k1, 1);
        t.insert(k2, 2);

        gc_collect(&[k1]);

        assert_eq!(fired.load(Ordering::SeqCst), 1);
        assert_eq!(seen.lock().unwrap().as_slice(), &[k1]);
        assert_eq!(t.len(), 1);
        assert!(!t.contains(&k1));
        assert!(t.contains(&k2));
    }

    #[test]
    fn gc_collect_without_callback_still_removes_entries() {
        let t = SideTable::<u64>::new();
        t.insert(ObjectKey(1), 5);
        gc_collect(&[ObjectKey(1)]);
        assert!(t.is_empty());
    }

    #[test]
    fn set_on_collect_replaces_previous_callback() {
        let t = SideTable::<u64>::new();
        let a = Arc::new(AtomicUsize::new(0));
        let b = Arc::new(AtomicUsize::new(0));
        let a2 = a.clone();
        let b2 = b.clone();
        t.set_on_collect(Arc::new(move |_| {
            a2.fetch_add(1, Ordering::SeqCst);
        }));
        t.set_on_collect(Arc::new(move |_| {
            b2.fetch_add(1, Ordering::SeqCst);
        }));
        t.insert(ObjectKey(3), 1);
        gc_collect(&[ObjectKey(3)]);
        assert_eq!(a.load(Ordering::SeqCst), 0);
        assert_eq!(b.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn table_drop_unregisters_and_sweep_prunes_dangling() {
        let id;
        {
            let t = SideTable::<u64>::new();
            id = t.id;
            assert!(tables_registry().lock().unwrap().contains_key(&id));
        }
        assert!(!tables_registry().lock().unwrap().contains_key(&id));

        // A dangling weak (as if a table vanished without Drop) is pruned by
        // the next sweep and never notified.
        struct Dead;
        impl GcNotifiable for Dead {
            fn notify_collected(&self, _key: ObjectKey) {}
        }
        {
            // `dead_arc` drops at the end of this block, leaving the
            // registered weak dangling.
            let dead_arc: Arc<dyn GcNotifiable + Send + Sync> = Arc::new(Dead);
            tables_registry().lock().unwrap().insert(id, Arc::downgrade(&dead_arc));
        }
        gc_collect(&[ObjectKey(99)]);
        assert!(!tables_registry().lock().unwrap().contains_key(&id));
    }

    #[test]
    fn get_or_insert_returns_existing_or_inserts() {
        let t = SideTable::<String>::new();
        let k = ObjectKey(5);
        assert_eq!(t.get_or_insert(k, || "first".to_string()), "first");
        assert_eq!(t.get_or_insert(k, || "second".to_string()), "first");
        assert_eq!(t.len(), 1);
    }

    #[test]
    fn values_clones_current_entries() {
        let t = SideTable::<u64>::new();
        t.insert(ObjectKey(1), 10);
        t.insert(ObjectKey(2), 20);
        let mut v = t.values();
        v.sort_unstable();
        assert_eq!(v, vec![10, 20]);
    }

    #[test]
    fn named_table_returns_none_when_unregistered() {
        assert_eq!(named_table("entities"), None);
    }
}
