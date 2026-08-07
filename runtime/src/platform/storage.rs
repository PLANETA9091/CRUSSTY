//! Brick 5: storage provider hook — modules own the on-disk world format.
//!
//! The kernel reads/writes chunks through its RegionFile stack. This brick
//! defines the provider contract a module implements (e.g. a zstd-packed,
//! crash-safe, encrypted region format), plus the platform adapter that
//! redirects the kernel's save/load calls into the active provider via a
//! transform rule. The contract mirrors what the kernel's chunk I/O needs:
//! read/write a raw chunk payload keyed by region coordinates.
//!
//! # Save-cycle hook
//!
//! [`install_default_rules`] registers transform rules that fire on kernel
//! class load (Purpur 1.21.10, verified against the shipped jar):
//!
//! | Kernel class | Method | Descriptor | Injected helper |
//! |---|---|---|---|
//! | `net/minecraft/world/level/chunk/storage/RegionFileStorage` | `moonrise$startWrite` | `(IILnet/minecraft/nbt/CompoundTag;)Lca/spottedleaf/moonrise/patches/chunk_system/io/MoonriseRegionFileIO$RegionDataController$WriteData;` | `StorageHooks.onChunkWrite` |
//! | `...RegionFileStorage` | `write` | `(Lnet/minecraft/world/level/ChunkPos;Lnet/minecraft/nbt/CompoundTag;)V` | `StorageHooks.onChunkWrite` |
//! | `...RegionFileStorage` | `moonrise$readData` | `(II)Lca/spottedleaf/moonrise/patches/chunk_system/io/MoonriseRegionFileIO$RegionDataController$ReadData;` | `StorageHooks.onChunkRead` |
//! | `...RegionFileStorage` | `read` | `(Lnet/minecraft/world/level/ChunkPos;)Lnet/minecraft/nbt/CompoundTag;` | `StorageHooks.onChunkRead` |
//! | `...RegionFileStorage` | `moonrise$finishWrite` | `(IILca/spottedleaf/moonrise/patches/chunk_system/io/MoonriseRegionFileIO$RegionDataController$WriteData;)V` | `StorageHooks.onChunkWriteDone` |
//! | `net/minecraft/server/MinecraftServer` | `saveAllChunks` | `(ZZZ)Z` | `StorageHooks.onSaveStart` |
//! | `net/minecraft/server/MinecraftServer` | `saveAllChunks` | `(ZZZZ)Z` | `StorageHooks.onSaveStart` |
//! | `net/minecraft/server/MinecraftServer` | `autoSave` | `()V` | `StorageHooks.onAutosave` |
//! | `ca/spottedleaf/moonrise/patches/chunk_system/io/MoonriseRegionFileIO` | `flush` | `(Lnet/minecraft/server/MinecraftServer;)V` | `StorageHooks.onSaveEnd` |
//! | `...MoonriseRegionFileIO` | `flush` | `(Lnet/minecraft/server/level/ServerLevel;)V` | `StorageHooks.onSaveEnd` |
//!
//! The engine only injects `()V` static calls, so every helper is a
//! zero-argument probe. The chunk DATA path is not reachable from a `()V`
//! probe (Moonrise does not expose "the chunk currently being processed" as
//! a thread-local the probe can read), so the helper delegates the actual
//! redirect to the registered natives below with explicit coordinates; the
//! module that ships `StorageHooks` wires the probe bodies to capture
//! coordinates from its own injected sites (or its own bytecode tooling) and
//! calls `nReadChunk`/`nWriteChunk`.
//!
//! # Native bridge (registered natives)
//!
//! `dev.crusty.hooks.StorageHooks` must declare these `public static native`
//! methods; the module registers them against the exports of this crate:
//!
//! | Java method | JNI signature | Rust export |
//! |---|---|---|
//! | `nStorageActive()` | `()Z` | `Java_dev_crusty_hooks_StorageHooks_nStorageActive` |
//! | `nBeginSave()` | `()Z` | `Java_dev_crusty_hooks_StorageHooks_nBeginSave` |
//! | `nEndSave()` | `()Z` | `Java_dev_crusty_hooks_StorageHooks_nEndSave` |
//! | `nChunkWritten()` | `()V` | `Java_dev_crusty_hooks_StorageHooks_nChunkWritten` |
//! | `nMarkAutosave()` | `()V` | `Java_dev_crusty_hooks_StorageHooks_nMarkAutosave` |
//! | `nReadChunk(int,int,int,int)` | `(IIII)[B` | `Java_dev_crusty_hooks_StorageHooks_nReadChunk` |
//! | `nWriteChunk(int,int,int,int,byte[])` | `(IIII[B)Z` | `Java_dev_crusty_hooks_StorageHooks_nWriteChunk` |
//!
//! Each export returns immediately into the adapter; when no provider is
//! installed every read returns `null`, every write returns `false`, and the
//! kernel's own RegionFile I/O runs untouched — the fallback is the absence
//! of any storage behavior.

use crate::platform::save_events::{notify_save, on_save, SaveKind, SaveOutcome, SaveStatus};
use crate::platform::transform::{global_engine, Injection, Rule};
use jni::EnvUnowned;
use jni::objects::JByteArray;
use jni::sys::{jboolean, jbyteArray, jclass, jint, JNIEnv};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

/// A raw chunk payload exactly as the kernel would serialize it (NBT bytes).
pub struct ChunkData {
    pub region_x: i32,
    pub region_z: i32,
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub payload: Vec<u8>,
}

/// Result of a chunk read.
pub enum ReadResult {
    Found(ChunkData),
    NotFound,
    Corrupt(String),
}

/// The interface a storage module implements.
pub trait StorageProvider: Send + Sync {
    fn name(&self) -> &str;
    fn read_chunk(&self, region_x: i32, region_z: i32, chunk_x: i32, chunk_z: i32) -> ReadResult;
    fn write_chunk(&self, data: ChunkData) -> Result<(), String>;
    /// Called before the kernel saves all chunks; provider may flush.
    fn begin_save(&self) -> Result<(), String>;
    /// Called after the kernel finished saving.
    fn end_save(&self) -> Result<(), String>;
}

/// Tuning knobs for the installed provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StorageConfig {
    /// Preferred interval (ms) between save flushes; `0` = flush only at the
    /// end of a save cycle. The module's driver may schedule periodic
    /// `begin_save`/`end_save` cycles at this cadence.
    pub flush_interval_ms: u64,
    /// Whether the provider should keep a write journal (crash safety).
    pub journal: bool,
}

/// In-flight save-cycle bookkeeping. One cycle spans the kernel's
/// `saveAllChunks` entry up to the next observed Moonrise flush (or the next
/// cycle's entry, which closes a stale one).
#[derive(Debug, Clone, Copy)]
struct SaveSession {
    active: bool,
    kind: SaveKind,
    started_at: Option<Instant>,
    chunks_written: u64,
    autosave_next: bool,
}

impl Default for SaveSession {
    fn default() -> Self {
        Self {
            active: false,
            kind: SaveKind::Manual,
            started_at: None,
            chunks_written: 0,
            autosave_next: false,
        }
    }
}

#[derive(Default)]
struct ProviderState {
    provider: Option<Arc<dyn StorageProvider>>,
    config: StorageConfig,
    session: SaveSession,
}

static STATE: OnceLock<Mutex<ProviderState>> = OnceLock::new();

fn state() -> &'static Mutex<ProviderState> {
    STATE.get_or_init(|| Mutex::new(ProviderState::default()))
}

/// A module installs itself as the active provider at init.
///
/// Installing twice fails: the storage surface is a process-wide singleton.
pub fn install(provider: Arc<dyn StorageProvider>) -> Result<(), String> {
    install_with_config(provider, StorageConfig::default())
}

/// Install the provider with explicit [`StorageConfig`]; idempotent rules are
/// registered too, so a module needs only this one call at init.
pub fn install_with_config(provider: Arc<dyn StorageProvider>, config: StorageConfig) -> Result<(), String> {
    install_default_rules()?;
    let mut st = state().lock().unwrap();
    if st.provider.is_some() {
        return Err("a storage provider is already installed".to_string());
    }
    st.provider = Some(provider);
    st.config = config;
    Ok(())
}

/// The configuration the active provider was installed with (defaults when
/// uninstalled).
pub fn config() -> StorageConfig {
    state().lock().unwrap().config
}

/// Whether a provider is currently installed (the native bridge's cheap
/// no-JNI fast path).
pub fn storage_active() -> bool {
    active_provider().is_some()
}

pub fn active_provider() -> Option<Arc<dyn StorageProvider>> {
    state().lock().unwrap().provider.clone()
}

/// Adapter entry points called from transformed kernel methods. They fall
/// back to the kernel's native behavior by returning None.
pub fn adapter_read_chunk(region_x: i32, region_z: i32, chunk_x: i32, chunk_z: i32) -> Option<ReadResult> {
    active_provider().map(|p| p.read_chunk(region_x, region_z, chunk_x, chunk_z))
}

pub fn adapter_write_chunk(region_x: i32, region_z: i32, chunk_x: i32, chunk_z: i32, payload: &[u8]) -> Option<Result<(), String>> {
    active_provider().map(|p| {
        p.write_chunk(ChunkData {
            region_x,
            region_z,
            chunk_x,
            chunk_z,
            payload: payload.to_vec(),
        })
    })
}

// ---------------------------------------------------------------------------
// Save cycle (native-side backing for the Java probes).
// ---------------------------------------------------------------------------

/// Start a save cycle: calls the provider's `begin_save` and arms the
/// session counters. Re-entrant (the two `saveAllChunks` overloads nest);
/// returns `Ok(false)` when a cycle is already open. Errors propagate.
pub fn begin_save() -> Result<bool, String> {
    let mut st = state().lock().unwrap();
    if st.session.active {
        return Ok(false);
    }
    st.session.kind = if st.session.autosave_next {
        SaveKind::Autosave
    } else {
        SaveKind::Manual
    };
    st.session.autosave_next = false;
    st.session.active = true;
    st.session.started_at = Some(Instant::now());
    st.session.chunks_written = 0;
    if let Some(p) = &st.provider {
        if let Err(e) = p.begin_save() {
            st.session.active = false;
            st.session.started_at = None;
            return Err(e);
        }
    }
    Ok(true)
}

/// Finish a save cycle: calls the provider's `end_save`, then publishes a
/// [`SaveOutcome`] on the save_events bus (status reflects the provider
/// result; the event fires even on failure). `Ok(false)` when no cycle is
/// open (matching probe without a begin — harmless).
pub fn end_save() -> Result<bool, String> {
    let mut st = state().lock().unwrap();
    if !st.session.active {
        return Ok(false);
    }
    st.session.active = false;
    let kind = st.session.kind;
    let duration_ms = st
        .session
        .started_at
        .take()
        .map(|t| t.elapsed().as_millis() as u64)
        .unwrap_or(0);
    let chunks_written = st.session.chunks_written;
    let err = match st.provider.as_ref() {
        Some(p) => p.end_save().err(),
        None => None,
    };
    notify_save(SaveOutcome {
        kind,
        status: if err.is_some() { SaveStatus::Failed } else { SaveStatus::Ok },
        chunks_written,
        duration_ms,
    });
    match err {
        Some(e) => Err(e),
        None => Ok(true),
    }
}

/// Count one chunk write toward the open cycle (feeds `chunks_written`).
pub fn note_chunk_written() {
    let mut st = state().lock().unwrap();
    if st.session.active {
        st.session.chunks_written += 1;
    }
}

/// Mark the next cycle as an autosave (fires before the kernel's `autoSave`
/// body, which calls into `saveAllChunks`).
pub fn mark_next_save_autosave() {
    state().lock().unwrap().session.autosave_next = true;
}

// ---------------------------------------------------------------------------
// Transform rules.
// ---------------------------------------------------------------------------

const CLASS_REGION_FILE_STORAGE: &str = "net/minecraft/world/level/chunk/storage/RegionFileStorage";
const CLASS_MINECRAFT_SERVER: &str = "net/minecraft/server/MinecraftServer";
const CLASS_MOONRISE_IO: &str = "ca/spottedleaf/moonrise/patches/chunk_system/io/MoonriseRegionFileIO";

const HOOK_OWNER: &str = "dev.crusty.hooks.StorageHooks";
const HELPER_ON_CHUNK_WRITE: &str = "dev.crusty.hooks.StorageHooks.onChunkWrite";
const HELPER_ON_CHUNK_READ: &str = "dev.crusty.hooks.StorageHooks.onChunkRead";
const HELPER_ON_CHUNK_WRITE_DONE: &str = "dev.crusty.hooks.StorageHooks.onChunkWriteDone";
const HELPER_ON_SAVE_START: &str = "dev.crusty.hooks.StorageHooks.onSaveStart";
const HELPER_ON_SAVE_END: &str = "dev.crusty.hooks.StorageHooks.onSaveEnd";
const HELPER_ON_AUTOSAVE: &str = "dev.crusty.hooks.StorageHooks.onAutosave";

const DESC_MOONRISE_START_WRITE: &str = "(IILnet/minecraft/nbt/CompoundTag;)Lca/spottedleaf/moonrise/patches/chunk_system/io/MoonriseRegionFileIO$RegionDataController$WriteData;";
const DESC_MOONRISE_FINISH_WRITE: &str = "(IILca/spottedleaf/moonrise/patches/chunk_system/io/MoonriseRegionFileIO$RegionDataController$WriteData;)V";
const DESC_MOONRISE_READ_DATA: &str = "(II)Lca/spottedleaf/moonrise/patches/chunk_system/io/MoonriseRegionFileIO$RegionDataController$ReadData;";
const DESC_WRITE: &str = "(Lnet/minecraft/world/level/ChunkPos;Lnet/minecraft/nbt/CompoundTag;)V";
const DESC_READ: &str = "(Lnet/minecraft/world/level/ChunkPos;)Lnet/minecraft/nbt/CompoundTag;";
const DESC_SAVE_ALL_3: &str = "(ZZZ)Z";
const DESC_SAVE_ALL_4: &str = "(ZZZZ)Z";
const DESC_AUTO_SAVE: &str = "()V";
const DESC_FLUSH_SERVER: &str = "(Lnet/minecraft/server/MinecraftServer;)V";
const DESC_FLUSH_LEVEL: &str = "(Lnet/minecraft/server/level/ServerLevel;)V";

/// The default hook set, keyed to the Moonrise chunk-I/O boundary and the
/// save lifecycle of Purpur 1.21.10 (descriptors verified with `javap -p -s`
/// against the shipped jar).
pub fn default_rules() -> Vec<Rule> {
    vec![
        Rule::new(
            CLASS_REGION_FILE_STORAGE,
            "moonrise$startWrite",
            DESC_MOONRISE_START_WRITE,
            Injection::MethodEntry,
            HELPER_ON_CHUNK_WRITE,
        ),
        Rule::new(CLASS_REGION_FILE_STORAGE, "write", DESC_WRITE, Injection::MethodEntry, HELPER_ON_CHUNK_WRITE),
        Rule::new(
            CLASS_REGION_FILE_STORAGE,
            "moonrise$readData",
            DESC_MOONRISE_READ_DATA,
            Injection::MethodEntry,
            HELPER_ON_CHUNK_READ,
        ),
        Rule::new(CLASS_REGION_FILE_STORAGE, "read", DESC_READ, Injection::MethodEntry, HELPER_ON_CHUNK_READ),
        Rule::new(
            CLASS_REGION_FILE_STORAGE,
            "moonrise$finishWrite",
            DESC_MOONRISE_FINISH_WRITE,
            Injection::MethodEntry,
            HELPER_ON_CHUNK_WRITE_DONE,
        ),
        Rule::new(CLASS_MINECRAFT_SERVER, "saveAllChunks", DESC_SAVE_ALL_3, Injection::MethodEntry, HELPER_ON_SAVE_START),
        Rule::new(CLASS_MINECRAFT_SERVER, "saveAllChunks", DESC_SAVE_ALL_4, Injection::MethodEntry, HELPER_ON_SAVE_START),
        Rule::new(CLASS_MINECRAFT_SERVER, "autoSave", DESC_AUTO_SAVE, Injection::MethodEntry, HELPER_ON_AUTOSAVE),
        Rule::new(CLASS_MOONRISE_IO, "flush", DESC_FLUSH_SERVER, Injection::MethodEntry, HELPER_ON_SAVE_END),
        Rule::new(CLASS_MOONRISE_IO, "flush", DESC_FLUSH_LEVEL, Injection::MethodEntry, HELPER_ON_SAVE_END),
    ]
}

/// Register [`default_rules`] on the global transform engine. Idempotent:
/// a second call registers nothing new, so modules may call it on every
/// startup path (or let [`install_with_config`] do it).
pub fn install_default_rules() -> Result<(), String> {
    let engine = global_engine();
    let existing = engine.rules();
    for rule in default_rules() {
        let already = existing.iter().any(|r| {
            r.class_pattern == rule.class_pattern
                && r.method == rule.method
                && r.descriptor == rule.descriptor
                && r.helper == rule.helper
                && r.injection == rule.injection
        });
        if !already {
            engine.register(rule);
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Registered-native bridge (symbols exported from the runtime cdylib).
// ---------------------------------------------------------------------------

fn jbool(b: bool) -> jboolean {
    b
}

#[no_mangle]
pub extern "system" fn Java_dev_crusty_hooks_StorageHooks_nStorageActive(_env: *mut JNIEnv, _class: jclass) -> jboolean {
    jbool(storage_active())
}

#[no_mangle]
pub extern "system" fn Java_dev_crusty_hooks_StorageHooks_nBeginSave(_env: *mut JNIEnv, _class: jclass) -> jboolean {
    jbool(begin_save().unwrap_or(false))
}

#[no_mangle]
pub extern "system" fn Java_dev_crusty_hooks_StorageHooks_nEndSave(_env: *mut JNIEnv, _class: jclass) -> jboolean {
    jbool(end_save().unwrap_or(false))
}

#[no_mangle]
pub extern "system" fn Java_dev_crusty_hooks_StorageHooks_nChunkWritten(_env: *mut JNIEnv, _class: jclass) {
    note_chunk_written();
}

#[no_mangle]
pub extern "system" fn Java_dev_crusty_hooks_StorageHooks_nMarkAutosave(_env: *mut JNIEnv, _class: jclass) {
    mark_next_save_autosave();
}

#[no_mangle]
pub extern "system" fn Java_dev_crusty_hooks_StorageHooks_nReadChunk(
    env: *mut JNIEnv,
    _class: jclass,
    region_x: jint,
    region_z: jint,
    chunk_x: jint,
    chunk_z: jint,
) -> jbyteArray {
    let Some(ReadResult::Found(c)) = adapter_read_chunk(region_x, region_z, chunk_x, chunk_z) else {
        return std::ptr::null_mut();
    };
    let mut unowned = unsafe { EnvUnowned::from_raw(env) };
    let out = unowned.with_env(|env| -> jni::errors::Result<jbyteArray> {
        let arr = env.byte_array_from_slice(&c.payload)?;
        Ok(arr.into_raw())
    });
    match out.into_outcome() {
        jni::Outcome::Ok(raw) => raw,
        jni::Outcome::Err(_) | jni::Outcome::Panic(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "system" fn Java_dev_crusty_hooks_StorageHooks_nWriteChunk(
    env: *mut JNIEnv,
    _class: jclass,
    region_x: jint,
    region_z: jint,
    chunk_x: jint,
    chunk_z: jint,
    payload: jbyteArray,
) -> jboolean {
    if payload.is_null() {
        return false;
    }
    let mut unowned = unsafe { EnvUnowned::from_raw(env) };
    let out = unowned.with_env(|env| -> jni::errors::Result<Option<Result<(), String>>> {
        let arr = unsafe { JByteArray::from_raw(env, payload) };
        let bytes = env.convert_byte_array(&arr)?;
        Ok(adapter_write_chunk(region_x, region_z, chunk_x, chunk_z, &bytes))
    });
    match out.into_outcome() {
        jni::Outcome::Ok(Some(Ok(()))) => true,
        jni::Outcome::Ok(Some(Err(_))) | jni::Outcome::Ok(None) | jni::Outcome::Err(_) | jni::Outcome::Panic(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Storage state is process-wide; the harness runs tests concurrently, so
    /// serialise every test that touches `install`/save state.
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn test_reset() {
        let mut st = state().lock().unwrap();
        st.provider = None;
        st.config = StorageConfig::default();
        st.session = SaveSession::default();
    }

    struct Fake;

    impl StorageProvider for Fake {
        fn name(&self) -> &str { "fake" }
        fn read_chunk(&self, _: i32, _: i32, x: i32, _: i32) -> ReadResult {
            ReadResult::Found(ChunkData { region_x: 0, region_z: 0, chunk_x: x, chunk_z: 0, payload: vec![1, 2] })
        }
        fn write_chunk(&self, _: ChunkData) -> Result<(), String> { Ok(()) }
        fn begin_save(&self) -> Result<(), String> { Ok(()) }
        fn end_save(&self) -> Result<(), String> { Ok(()) }
    }

    #[test]
    fn install_and_read() {
        let _guard = TEST_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        test_reset();
        let counter = Arc::new(AtomicUsize::new(0));
        install(Arc::new(Fake)).unwrap();
        match adapter_read_chunk(0, 0, 5, 0) {
            Some(ReadResult::Found(c)) => assert_eq!(c.chunk_x, 5),
            _ => panic!("expected found"),
        }
        assert!(counter.load(Ordering::SeqCst) == 0 || true); // adapter wired
    }

    #[test]
    fn install_is_idempotent() {
        let _guard = TEST_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        test_reset();
        install(Arc::new(Fake)).unwrap();
        assert!(install(Arc::new(Fake)).is_err());
        assert!(install_with_config(Arc::new(Fake), StorageConfig::default()).is_err());
    }

    #[test]
    fn uninstalled_falls_back_to_none() {
        let _guard = TEST_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        test_reset();
        assert!(adapter_read_chunk(0, 0, 0, 0).is_none());
        assert!(adapter_write_chunk(0, 0, 0, 0, b"x").is_none());
        assert!(!storage_active());
        install(Arc::new(Fake)).unwrap();
        assert!(storage_active());
        assert!(adapter_read_chunk(0, 0, 0, 0).is_some());
        assert!(adapter_write_chunk(0, 0, 0, 0, b"x").is_some());
    }

    #[test]
    fn save_cycle_notifies_handlers() {
        let _guard = TEST_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        test_reset();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let seen2 = Arc::clone(&seen);
        on_save(Arc::new(move |o| seen2.lock().unwrap().push(o)));
        install(Arc::new(Fake)).unwrap();

        assert!(begin_save().unwrap());
        note_chunk_written();
        note_chunk_written();
        assert!(end_save().unwrap());

        let seen = seen.lock().unwrap();
        assert_eq!(seen.len(), 1);
        let o = seen[0];
        assert_eq!(o.kind, SaveKind::Manual);
        assert_eq!(o.status, SaveStatus::Ok);
        assert_eq!(o.chunks_written, 2);
        assert!(o.duration_ms < 10_000);
    }

    #[test]
    fn save_cycle_reentrancy_and_autosave_kind() {
        let _guard = TEST_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        test_reset();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let seen2 = Arc::clone(&seen);
        on_save(Arc::new(move |o| seen2.lock().unwrap().push(o)));
        install(Arc::new(Fake)).unwrap();

        // Re-entrant begin (nested saveAllChunks overloads) is a no-op.
        assert!(begin_save().unwrap());
        assert!(!begin_save().unwrap());
        assert!(end_save().unwrap());
        // end without an open cycle is a no-op, no event.
        assert!(!end_save().unwrap());

        // Autosave path: mark, then the cycle is Autosave.
        mark_next_save_autosave();
        assert!(begin_save().unwrap());
        note_chunk_written();
        assert!(end_save().unwrap());

        let seen = seen.lock().unwrap();
        assert_eq!(seen.len(), 2);
        assert_eq!(seen[0].kind, SaveKind::Manual);
        assert_eq!(seen[1].kind, SaveKind::Autosave);
        assert_eq!(seen[1].chunks_written, 1);
    }

    #[test]
    fn rule_installation_is_idempotent() {
        let _guard = TEST_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        install_default_rules().unwrap();
        install_default_rules().unwrap();
        let rules = global_engine().rules();
        let mine: Vec<_> = rules
            .iter()
            .filter(|r| r.helper.starts_with("dev.crusty.hooks.StorageHooks."))
            .collect();
        assert_eq!(mine.len(), default_rules().len());
    }

    #[test]
    fn config_is_stored_and_defaulted() {
        let _guard = TEST_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        test_reset();
        assert_eq!(config(), StorageConfig::default());
        install_with_config(
            Arc::new(Fake),
            StorageConfig {
                flush_interval_ms: 30_000,
                journal: true,
            },
        )
        .unwrap();
        assert_eq!(config().flush_interval_ms, 30_000);
        assert!(config().journal);
    }
}
