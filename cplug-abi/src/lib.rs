//! cplug-abi — the ONLY contract between the crussty-runtime and modules.
//!
//! Modules are native shared libraries in `modules/` (recursive scan). Each
//! must export:
//!
//! ```c
//! int32_t cplugin_init(const CPluginApi* api, JavaVM* vm, const char* options);
//! ```
//!
//! Everything else is raw JVM access (JNI/JVMTI via the JavaVM*): no Java API,
//! no limits. The api gives three tiny services: register a class-file hook
//! (the automatic hot-patch pipeline) and allocate replacement bytes through
//! the JVMTI allocator.
//!
//! Since CPAPI_VERSION 2.1 the struct carries one more trailing field:
//! `platform_api` — the C-visible surface of the runtime's platform bricks
//! (events, scheduler, telemetry, storage, ...). Old modules compiled against
//! the 3-field struct keep working unchanged: the field is appended at the
//! end, so offsets of the original fields never move.

#![allow(non_camel_case_types)]

use std::ffi::{c_char, c_void};

/// Wire version. Modules must accept `>= 2` and ignore unknown trailing fields.
pub const CPAPI_VERSION: u32 = 3;

/// Optional plugin hook, invoked for every class load before the class is
/// defined (JVMTI CLASS_FILE_LOAD_HOOK). The hot-patch pipeline: plugins get
/// the raw class bytes and may return patched bytes.
///
/// Contract:
/// - `out_data`/`out_len` must be set only when replacing the class; the
///   replacement buffer must come from `api.jvmti_allocate`.
/// - return 0  -> patched: use *out_data/*out_len as the new class bytes;
///   return != 0 -> keep original bytes (plugin error or no-op).
pub type ClassHookFn = unsafe extern "C" fn(
    ctx: *mut c_void,
    name: *const c_char,
    class_data: *const u8,
    class_data_len: usize,
    out_data: *mut *mut u8,
    out_len: *mut usize,
) -> i32;

// ---------------------------------------------------------------------------
// Platform bricks C bridge (CPAPI 2.1+)
//
// The runtime owns these bricks (`runtime/src/platform/*.rs`); modules get a
// function table instead of linking the runtime, because module `.so` files
// are dlopened with RTLD_LOCAL. Every brick below exposes its state-changing
// surface as extern "C" trampolines. NULL pointers mean "not available".
// ---------------------------------------------------------------------------

/// Callback fired on an event-bus publish. `event` and `payload_json` are
/// valid only for the duration of the call.
pub type EventCb = unsafe extern "C" fn(
    event: *const c_char,
    payload_json: *const c_char,
    ctx: *mut c_void,
);

/// Callback for scheduler-injected work; runs on the kernel main thread.
pub type SchedulerTaskCb = unsafe extern "C" fn(ctx: *mut c_void);

/// Callback for faults (crash isolation brick).
pub type FaultCb = unsafe extern "C" fn(
    signal: i32,
    timestamp_unix: u64,
    count: u64,
    ctx: *mut c_void,
);

/// Verdict for the network packet pipeline.
/// 0 = Pass, 1 = Drop, 2 = Disconnect.
pub const NET_PASS: i32 = 0;
pub const NET_DROP: i32 = 1;
pub const NET_DISCONNECT: i32 = 2;

/// Packet snapshot handed to the hook (valid for the call only).
#[repr(C)]
pub struct CPacket {
    pub direction: i32, // 0 = inbound, 1 = outbound
    pub state: u8,
    pub payload: *const u8,
    pub payload_len: usize,
    pub conn_id: u64,
}

/// Async callback handed to the hook; payload valid only during the call.
pub type PacketHookCb = unsafe extern "C" fn(
    pkt: *const CPacket,
    ctx: *mut c_void,
) -> i32;

/// Save-lifecycle callback (`save_events` brick).
/// kind: 0 = autosave, 1 = manual; status: 0 = ok, 1 = failed.
pub type SaveCb = unsafe extern "C" fn(
    kind: i32,
    status: i32,
    chunks_written: u64,
    duration_ms: u64,
    ctx: *mut c_void,
);

/// Storage provider callbacks (all optional; a NULL pointer means "not
/// supported" for that operation). Buffer lifetimes: the runtime owns the
/// payload pointer during the call.
pub type StorageNameCb =
    unsafe extern "C" fn(ctx: *mut c_void) -> *const c_char;
/// Returns 1=found, 0=not found, -1=corrupt.
pub type StorageReadCb = unsafe extern "C" fn(
    ctx: *mut c_void,
    region_x: i32,
    region_z: i32,
    chunk_x: i32,
    chunk_z: i32,
    out_payload: *mut *const u8,
    out_len: *mut usize,
) -> i32;
/// Returns 0 on success, non-zero on error.
pub type StorageWriteCb = unsafe extern "C" fn(
    ctx: *mut c_void,
    region_x: i32,
    region_z: i32,
    chunk_x: i32,
    chunk_z: i32,
    payload: *const u8,
    payload_len: usize,
) -> i32;
pub type StorageBeginSaveCb = unsafe extern "C" fn(ctx: *mut c_void) -> i32;
pub type StorageEndSaveCb = unsafe extern "C" fn(ctx: *mut c_void) -> i32;

/// Complete brick-API table. Every field must stay at its current offset for
/// the lifetime of this major version; append new entries at the END only.
#[repr(C)]
pub struct CPlatformApi {
    pub version: u32, // = CPB_API_VERSION (see below)

    // --- events (brick 6) ------------------------------------------------
    pub events_subscribe:
        Option<unsafe extern "C" fn(event: *const c_char, cb: EventCb, ctx: *mut c_void) -> u64>,
    pub events_unsubscribe: Option<unsafe extern "C" fn(token: u64) -> i32>,
    pub events_publish: Option<unsafe extern "C" fn(event: *const c_char, payload_json: *const c_char) -> usize>,

    // --- scheduler (brick 3) ----------------------------------------------
    /// Queue `cb(ctx)` for the kernel main thread; returns a token (>0) or 0.
    pub scheduler_inject:
        Option<unsafe extern "C" fn(tag: *const c_char, cb: SchedulerTaskCb, ctx: *mut c_void) -> u64>,
    pub scheduler_current_tick: Option<unsafe extern "C" fn() -> u64>,
    pub scheduler_injected_pending: Option<unsafe extern "C" fn() -> usize>,

    // --- telemetry (brick 7) ----------------------------------------------
    /// `unit` and `labels_json` may be NULL. labels_json = `{"a":"b",...}`.
    pub telemetry_publish_metric: Option<
        unsafe extern "C" fn(
            name: *const c_char,
            value: f64,
            unit: *const c_char,
            labels_json: *const c_char,
        ) -> i32,
    >,
    pub telemetry_snapshot_json: Option<unsafe extern "C" fn() -> *const c_char>,

    // --- signals (brick 9) -------------------------------------------------
    pub signals_on_fault: Option<unsafe extern "C" fn(cb: FaultCb, ctx: *mut c_void) -> i32>,
    pub signals_fault_count: Option<unsafe extern "C" fn() -> u64>,
    pub signals_crash_log: Option<unsafe extern "C" fn(path: *const c_char) -> i32>,

    // --- network (brick 6b) -----------------------------------------------
    pub network_add_hook: Option<unsafe extern "C" fn(cb: PacketHookCb, ctx: *mut c_void) -> i32>,
    pub network_attach_conn: Option<unsafe extern "C" fn(conn_id: u64, player_uuid_hi: u64, player_uuid_lo: u64) -> i32>,
    pub network_detach_conn: Option<unsafe extern "C" fn(conn_id: u64) -> i32>,
    pub network_conn_state: Option<unsafe extern "C" fn(conn_id: u64, state_code: u8) -> i32>,
    pub network_conn_count: Option<unsafe extern "C" fn() -> usize>,

    // --- storage (brick 5) -------------------------------------------------
    /// One-shot installer; callbacks on the passed vtable + ctx are captured.
    pub storage_install: Option<unsafe extern "C" fn(
        ctx: *mut c_void,
        name: StorageNameCb,
        read_chunk: StorageReadCb,
        write_chunk: StorageWriteCb,
        begin_save: StorageBeginSaveCb,
        end_save: StorageEndSaveCb,
    ) -> i32>,
    pub storage_active: Option<unsafe extern "C" fn() -> i32>,

    // --- threads (brick 9) --------------------------------------------------
    pub threads_spawn: Option<unsafe extern "C" fn(name: *const c_char, f: SchedulerTaskCb, ctx: *mut c_void) -> i32>,
    pub threads_spawn_daemon: Option<unsafe extern "C" fn(name: *const c_char, f: SchedulerTaskCb, ctx: *mut c_void) -> i32>,
    pub threads_current_name: Option<unsafe extern "C" fn(out: *mut c_char, out_len: usize) -> i32>,

    // --- transform (brick 1) ------------------------------------------------
    /// injection: 0 = methodEntry, 1 = before_call(helper).
    pub transform_register_rule: Option<unsafe extern "C" fn(
        class_pattern: *const c_char,
        method: *const c_char,
        descriptor: *const c_char,
        injection: i32,
        helper: *const c_char,
    ) -> i32>,

    // --- save_events (brick 8) ----------------------------------------------
    pub save_events_on_save: Option<unsafe extern "C" fn(cb: SaveCb, ctx: *mut c_void) -> i32>,

    // --- hot_reload (brick 10) ----------------------------------------------
    pub hot_reload_module: Option<unsafe extern "C" fn(id: *const c_char) -> i32>,
    pub hot_reload_enter: Option<unsafe extern "C" fn(id: *const c_char) -> i32>,
    pub hot_reload_leave: Option<unsafe extern "C" fn(id: *const c_char) -> i32>,

    // --- barriers (brick 4a) -------------------------------------------------
    /// NOTE: a barrier has no global state and is therefore not exposed;
    /// modules implementing barriers embed them in their own processes.
    /// (Reserved — keep alignment stable.)

    // --- side_table (brick 2) -------------------------------------------------
    pub side_table_key: Option<unsafe extern "C" fn(obj: *mut c_void, out: *mut u64) -> i32>,
    pub side_table_named: Option<unsafe extern "C" fn(name: *const c_char, out: *mut u64) -> i32>,
}

pub const CPB_VERSION: u32 = 1;

/// Agent -> plugin services.
#[repr(C)]
pub struct CPluginApi {
    pub version: u32,
    /// Register `hook` (with plugin-owned `ctx`) in the runtime's patch pipeline.
    pub register_class_hook:
        Option<unsafe extern "C" fn(ctx: *mut c_void, hook: ClassHookFn) -> i32>,
    /// Allocate a buffer via JVMTI (freed by the VM with Deallocate).
    pub jvmti_allocate: Option<unsafe extern "C" fn(size: usize) -> *mut u8>,
    /// Retransform a loaded class by internal name; re-enters the plugin hook
    /// pipeline with its current bytes so a hook may patch it post-load.
    /// Returns 0 on success, negative on failure (class not loaded, etc.).
    pub retransform_class: Option<unsafe extern "C" fn(name: *const c_char) -> i32>,
    /// Claim a unique global resource key before registering it with the
    /// JVM — e.g. "class:a/b/C" for DefineClass and "native:a/b/C#name:sig"
    /// for RegisterNatives. Guarantees two modules never collide: returns 0
    /// when the key is free (or already owned by the same `owner`) and -1
    /// when another module owns it (the caller must skip the registration).
    /// `owner` is the caller's plugin handle (e.g. the CPluginApi pointer).
    pub claim: Option<unsafe extern "C" fn(owner: usize, key: *const c_char) -> i32>,
    /// (v3.+) Pointer to the platform-bricks function table; NULL on runtimes
    /// older than this addition. Check for non-NULL at runtime.
    pub platform: *const CPlatformApi,
}

/// Opaque JavaVM* (cast to your JNI bindings' JavaVM type on either side).
pub type JavaVmPtr = *mut c_void;

/// `cplugin_init(api, vm, options) -> i32` — the single required export.
/// options carries runtime info, e.g. "modules=<dir>;versions=<dir>;kernel=<jar>".
pub type CPluginInit =
    unsafe extern "C" fn(api: *const CPluginApi, vm: JavaVmPtr, options: *const c_char) -> i32;