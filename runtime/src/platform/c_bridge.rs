//! C bridge over the platform bricks (CPAPI 2.1).
//!
//! Modules are dlopened with RTLD_LOCAL, so they can never link the runtime
//! crate directly. Instead the agent hands every module a function table
//! (`CPlatformApi`, appended to `CPluginApi` as a trailing pointer). Every
//! entry below is a thin extern "C" trampoline into the owning brick's public
//! API; nothing is re-implemented here.
//!
//! Conventions:
//! - string args are NUL-terminated UTF-8; NULL is treated as empty.
//! - callbacks are fired on the brick's own thread (module context must not
//!   assume the JVM main thread).
//! - nothing here allocates JVM memory; payloads are caller-owned copies.

use crate::platform::{
    events, hot_reload, network, save_events, scheduler, side_table, signals, storage,
    telemetry, threads, transform,
};
use cplug_abi::{
    CPacket, CPlatformApi, EventCb, FaultCb, PacketHookCb, SaveCb, SchedulerTaskCb, StorageBeginSaveCb,
    StorageEndSaveCb, StorageNameCb, StorageReadCb, StorageWriteCb,
};
use serde_json::Value;
use std::collections::HashMap;
use std::ffi::{c_char, c_void, CStr, CString};
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex, OnceLock};

/// Table version. MUST match `CPB_VERSION` in cplug-abi.
pub const CPB_VERSION: u32 = 1;

/// TASK-179 A/B toggle: gate the C-entry JSON parse behind the
/// zero-subscriber check. `false` = pre-TASK-179 shape (the payload is
/// always parsed; publish's own gate then rejects it when nothing is
/// subscribed). Semantics are identical in both states — the payload is
/// unobservable without subscribers (publish drops it untouched).
pub const C_ENTRY_PREGATE: bool = true;

/// TASK-180 A/B toggle: gate the C telemetry entry's labels parse behind the
/// metrics-full check. `false` = pre-TASK-180 shape (labels always parsed;
/// publish_metric then drops the metric when the list is at MAX_METRICS).
/// When the list is full neither state publishes anything; the ONE
/// documented observable difference is the error code for a MALFORMED
/// labels string in the full state (0 instead of -2, the parse never ran);
/// the 0 / -1 / -2 contract is unchanged in every state where a metric
/// could actually be accepted.
pub const TELEM_PREGATE: bool = true;

/// TASK-181 A/B toggle: serialize the C snapshot entry's JSON in place.
/// `false` = pre-TASK-181 shape (every call deep-clones the whole Snapshot
/// under the data lock, serializes the clone into a fresh String, then
/// swaps a fresh CString into the static, freeing the old one). `true` =
/// the locked snapshot is serialized directly into a REUSED NUL-terminated
/// buffer (zero allocs in the steady state; the deep clone and the two
/// per-call allocations are gone). Semantics are identical in both states:
/// byte-identical JSON (asserted in-tree), the same pointer-valid-until-
/// next-call contract (both shapes overwrite the shared buffer every call;
/// serde JSON bytes never contain a raw 0x00, so the manual terminator is
/// the only NUL — CString semantics preserved).
pub const TELEM_SNAP_INPLACE: bool = true;

unsafe fn cstr(p: *const c_char) -> &'static str {
    if p.is_null() {
        return "";
    }
    CStr::from_ptr(p).to_str().unwrap_or("")
}

/// NULL-check for raw extern "C" function pointers (they have no is_null()).
#[inline]
fn fnull<T: Copy>(f: T) -> bool {
    // fn pointers are word-sized on all supported ABIs; compare by bytes.
    let mut word = 0usize;
    unsafe {
        std::ptr::copy_nonoverlapping(
            &f as *const T as *const u8,
            &mut word as *mut usize as *mut u8,
            std::mem::size_of::<T>(),
        );
    }
    word == 0
}

/// Opaque module context that may cross threads.
#[derive(Clone, Copy)]
struct Ctx(*mut c_void);
unsafe impl Send for Ctx {}
unsafe impl Sync for Ctx {}
impl Ctx {
    fn null() -> Self {
        Ctx(std::ptr::null_mut())
    }
}

/// (fn, ctx) pair that may cross threads; keeps the raw pointer out of the
/// closure capture so auto Send/Sync applies.
struct CbThunk<T> {
    f: T,
    ctx: Ctx,
}
unsafe impl<T: Send> Send for CbThunk<T> {}
unsafe impl<T: Sync> Sync for CbThunk<T> {}

// ---------------------------------------------------------------------------
// events (brick 6)
// ---------------------------------------------------------------------------

/// token -> event name, used by unsubscribe. The bus needs both the event
/// name and the (id, gen) — we keep the gen out of C, so unsubscribe is
/// simple: token only, event name looked up here.
static SUBSCRIPTIONS: OnceLock<Mutex<HashMap<u64, (String, events::Subscription)>>> = OnceLock::new();

fn subscriptions() -> &'static Mutex<HashMap<u64, (String, events::Subscription)>> {
    SUBSCRIPTIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

unsafe extern "C" fn n_events_subscribe(event: *const c_char, cb: EventCb, ctx: *mut c_void) -> u64 {
    if fnull(cb) || event.is_null() {
        return 0;
    }
    let event = cstr(event).to_string();
    let th: Arc<(EventCb, Ctx)> = Arc::new((cb, Ctx(ctx)));
    let sub = events::global().subscribe(&event, Arc::new(move |_ev, payload| {
        let ev = CString::new(_ev).unwrap_or_default();
        let payload = serde_json::to_string(payload).unwrap_or_else(|_| "null".into());
        let payload = CString::new(payload).unwrap_or_default();
        (th.0)(ev.as_ptr(), payload.as_ptr(), th.1 .0);
    }));
    subscriptions()
        .lock()
        .unwrap()
        .insert(sub.id, (event.clone(), sub.clone()));
    sub.id
}

unsafe extern "C" fn n_events_publish(event: *const c_char, payload_json: *const c_char) -> usize {
    let event = cstr(event);
    if event.is_empty() {
        return 0;
    }
    // Borrowed global handle (TASK-163 pattern): the owned global() clone
    // costs ~5 Arc refcount pairs per call — hot C publishes pay it for
    // nothing, the &'static handle serves gate and publish alike.
    let bus = events::global_ref();
    // TASK-179 pre-parse gate: when nothing was ever subscribed, publish()
    // would drop the payload untouched and return 0 — skip the JSON parse
    // entirely. Same acquire load publish's own fast gate uses; semantics
    // identical (the payload is unobservable without subscribers), the
    // race window shortens exactly like the in-publish gate does.
    if C_ENTRY_PREGATE && !bus.may_have_subscribers() {
        return 0;
    }
    let payload: Value = if payload_json.is_null() {
        Value::Null
    } else {
        serde_json::from_str(cstr(payload_json)).unwrap_or(Value::Null)
    };
    bus.publish(event, &payload)
}

unsafe extern "C" fn n_events_unsubscribe(token: u64) -> i32 {
    let (event, sub) = {
        let mut map = subscriptions().lock().unwrap();
        let Some((event, sub)) = map.remove(&token) else {
            return -1;
        };
        (event, sub)
    };
    if events::global().unsubscribe(&event, &sub) {
        0
    } else {
        -2
    }
}

// ---------------------------------------------------------------------------
// scheduler (brick 3)
// ---------------------------------------------------------------------------

static INJECTED_COUNT: AtomicUsize = AtomicUsize::new(0);

unsafe extern "C" fn n_scheduler_inject(
    tag: *const c_char,
    cb: SchedulerTaskCb,
    ctx: *mut c_void,
) -> u64 {
    if fnull(cb) {
        return 0;
    }
    let tag = cstr(tag).to_string();
    INJECTED_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let th: Arc<(SchedulerTaskCb, Ctx)> = Arc::new((cb, Ctx(ctx)));
    scheduler::inject(&tag, move || (th.0)(th.1 .0))
}

unsafe extern "C" fn n_scheduler_current_tick() -> u64 {
    scheduler::current_tick()
}

unsafe extern "C" fn n_scheduler_injected_pending() -> usize {
    INJECTED_COUNT.load(std::sync::atomic::Ordering::Relaxed)
}

// ---------------------------------------------------------------------------
// telemetry (brick 7)
// ---------------------------------------------------------------------------

unsafe extern "C" fn n_telemetry_publish_metric(
    name: *const c_char,
    value: f64,
    unit: *const c_char,
    labels_json: *const c_char,
) -> i32 {
    let name = cstr(name);
    if name.is_empty() {
        return -1;
    }
    // TASK-180 pre-parse gate: once the metric list is full,
    // publish_metric drops every further metric — the parsed labels would
    // be unobservable. Skip the parse entirely (see the TELEM_PREGATE doc
    // for the one documented observable difference in the full state).
    if TELEM_PREGATE && telemetry::metrics_full() {
        return 0;
    }
    let unit = if unit.is_null() { None } else { Some(cstr(unit)) };
    let labels = if labels_json.is_null() {
        None
    } else {
        match serde_json::from_str::<HashMap<String, String>>(cstr(labels_json)) {
            Ok(l) => Some(l),
            Err(_) => return -2,
        }
    };
    telemetry::publish_metric(name, value, unit, labels);
    0
}

static SNAP: OnceLock<Mutex<Option<CString>>> = OnceLock::new();

/// TASK-181: reusable serialize target for the in-place path. The returned
/// pointer stays valid until the NEXT call to the entry (the same contract
/// the legacy CString swap had) — the Vec lives in this static, the NUL
/// terminator is appended after every write, and serde JSON output cannot
/// contain a raw 0x00 byte (control characters are escaped), so the
/// terminator is the only NUL in the buffer.
static SNAP_BUF: OnceLock<Mutex<Vec<u8>>> = OnceLock::new();

unsafe extern "C" fn n_telemetry_snapshot_json() -> *const c_char {
    if TELEM_SNAP_INPLACE {
        let buf = SNAP_BUF.get_or_init(|| Mutex::new(Vec::with_capacity(4096)));
        let mut guard = buf.lock().unwrap();
        guard.clear();
        telemetry::snapshot_json_write(guard.as_mut());
        guard.push(0);
        guard.as_ptr() as *const c_char
    } else {
        // Pre-TASK-181 shape, kept verbatim for the A/B toggle (the
        // byte-identity test re-derives this shape independently).
        let s = telemetry::snapshot();
        let json = serde_json::to_string(&s).unwrap_or_else(|_| "{}".into());
        let ptr = SNAP.get_or_init(|| Mutex::new(None));
        let mut guard = ptr.lock().unwrap();
        *guard = Some(CString::new(json).unwrap());
        guard.as_ref().unwrap().as_ptr()
    }
}

// ---------------------------------------------------------------------------
// signals (brick 9)
// ---------------------------------------------------------------------------

unsafe extern "C" fn n_signals_on_fault(cb: FaultCb, ctx: *mut c_void) -> i32 {
    if fnull(cb) {
        return -1;
    }
    let th: Arc<(FaultCb, Ctx)> = Arc::new((cb, Ctx(ctx)));
    signals::on_fault(Arc::new(move |info: signals::FaultInfo| {
        (th.0)(info.signal, info.timestamp_unix, info.count, th.1 .0);
    }));
    0
}

unsafe extern "C" fn n_signals_fault_count() -> u64 {
    signals::fault_count() as u64
}

unsafe extern "C" fn n_signals_crash_log(path: *const c_char) -> i32 {
    if path.is_null() {
        return -1;
    }
    signals::set_crash_log_path(Some(std::path::PathBuf::from(cstr(path))));
    0
}

// ---------------------------------------------------------------------------
// network (brick 6b)
// ---------------------------------------------------------------------------

static NET_HOOKS: OnceLock<Mutex<Vec<(PacketHookCb, Ctx)>>> = OnceLock::new();

#[repr(C)]
struct CPacketPtr {
    direction: i32,
    state: u8,
    payload: *const u8,
    payload_len: usize,
    conn_id: u64,
}

unsafe extern "C" fn n_network_add_hook(cb: PacketHookCb, ctx: *mut c_void) -> i32 {
    if fnull(cb) {
        return -1;
    }
    NET_HOOKS
        .get_or_init(|| Mutex::new(Vec::new()))
        .lock()
        .unwrap()
        .push((cb, Ctx(ctx)));
    if NET_ADAPTER.get().is_none() {
        let _ = NET_ADAPTER.set(());
        network::add_hook(Arc::new(|pkt: &mut network::Packet| -> network::Verdict {
            let hooks = NET_HOOKS
                .get_or_init(|| Mutex::new(Vec::new()))
                .lock()
                .unwrap()
                .clone();
            let direction = match pkt.direction {
                network::Direction::Inbound => 0i32,
                network::Direction::Outbound => 1i32,
            };
            let cpkt = CPacketPtr {
                direction,
                state: pkt.state,
                payload: pkt.payload.as_ptr(),
                payload_len: pkt.payload.len(),
                conn_id: pkt.conn_id,
            };
            for (cb, ctx) in &hooks {
                let v = unsafe { cb(&cpkt as *const CPacketPtr as *const CPacket, ctx.0) };
                match v {
                    0 => {}                 // pass
                    1 => return network::Verdict::Drop,
                    _ => return network::Verdict::Disconnect,
                }
            }
            network::Verdict::Pass
        }));
    }
    0
}

static NET_ADAPTER: OnceLock<()> = OnceLock::new();

unsafe extern "C" fn n_network_attach_conn(
    conn_id: u64,
    player_uuid_hi: u64,
    player_uuid_lo: u64,
) -> i32 {
    let uuid = if player_uuid_hi == 0 && player_uuid_lo == 0 {
        None
    } else {
        Some(((player_uuid_hi as u128) << 64) | player_uuid_lo as u128)
    };
    network::attach_conn(conn_id, uuid) as i32
}

unsafe extern "C" fn n_network_detach_conn(conn_id: u64) -> i32 {
    network::detach_conn(conn_id) as i32
}

unsafe extern "C" fn n_network_conn_state(conn_id: u64, state_code: u8) -> i32 {
    network::set_conn_state(conn_id, state_code) as i32
}

unsafe extern "C" fn n_network_conn_count() -> usize {
    network::conn_count()
}

// ---------------------------------------------------------------------------
// storage (brick 5) — vtable adapter
// ---------------------------------------------------------------------------

struct CStorageProvider {
    name: Option<StorageNameCb>,
    ctx: Ctx,
    read_chunk: Option<StorageReadCb>,
    write_chunk: Option<StorageWriteCb>,
    begin_save: Option<StorageBeginSaveCb>,
    end_save: Option<StorageEndSaveCb>,
}

impl storage::StorageProvider for CStorageProvider {
    fn name(&self) -> &str {
        match self.name {
            Some(f) => unsafe { cstr(f(self.ctx.0)) },
            None => "c",
        }
    }

    fn read_chunk(
        &self,
        region_x: i32,
        region_z: i32,
        chunk_x: i32,
        chunk_z: i32,
    ) -> storage::ReadResult {
        let Some(read) = self.read_chunk else {
            return storage::ReadResult::NotFound;
        };
        let mut out: *const u8 = std::ptr::null();
        let mut out_len: usize = 0;
        let rc =
            unsafe { read(self.ctx.0, region_x, region_z, chunk_x, chunk_z, &mut out, &mut out_len) };
        match rc {
            1 => {
                let payload = unsafe { std::slice::from_raw_parts(out, out_len).to_vec() };
                storage::ReadResult::Found(storage::ChunkData {
                    region_x,
                    region_z,
                    chunk_x,
                    chunk_z,
                    payload,
                })
            }
            0 => storage::ReadResult::NotFound,
            _ => storage::ReadResult::Corrupt(format!("c-provider rc={rc}")),
        }
    }

    fn write_chunk(&self, data: storage::ChunkData) -> Result<(), String> {
        let Some(write) = self.write_chunk else {
            return Err("write_chunk unsupported".into());
        };
        let rc = unsafe {
            write(
                self.ctx.0,
                data.region_x,
                data.region_z,
                data.chunk_x,
                data.chunk_z,
                data.payload.as_ptr(),
                data.payload.len(),
            )
        };
        if rc == 0 {
            Ok(())
        } else {
            Err(format!("c-provider write rc={rc}"))
        }
    }

    fn begin_save(&self) -> Result<(), String> {
        match self.begin_save {
            Some(f) if unsafe { f(self.ctx.0) } == 0 => Ok(()),
            Some(_) => Err("c-provider begin_save failed".into()),
            None => Ok(()),
        }
    }

    fn end_save(&self) -> Result<(), String> {
        match self.end_save {
            Some(f) if unsafe { f(self.ctx.0) } == 0 => Ok(()),
            Some(_) => Err("c-provider end_save failed".into()),
            None => Ok(()),
        }
    }
}

static STORAGE_KEEP: OnceLock<Mutex<Option<Arc<CStorageProvider>>>> = OnceLock::new();

unsafe extern "C" fn n_storage_install(
    ctx: *mut c_void,
    name: StorageNameCb,
    read_chunk: StorageReadCb,
    write_chunk: StorageWriteCb,
    begin_save: StorageBeginSaveCb,
    end_save: StorageEndSaveCb,
) -> i32 {
    static STORAGE_ENTRY: OnceLock<()> = OnceLock::new();
    if STORAGE_ENTRY.get().is_some() || storage::storage_active() {
        return -1;
    }
    let _ = STORAGE_ENTRY.set(());
    let provider = Arc::new(CStorageProvider {
        name: if fnull(name) { None } else { Some(name) },
        ctx: Ctx(ctx),
        read_chunk: if fnull(read_chunk) { None } else { Some(read_chunk) },
        write_chunk: if fnull(write_chunk) { None } else { Some(write_chunk) },
        begin_save: if fnull(begin_save) { None } else { Some(begin_save) },
        end_save: if fnull(end_save) { None } else { Some(end_save) },
    });
    *STORAGE_KEEP.get_or_init(|| Mutex::new(None)).lock().unwrap() = Some(provider.clone());
    match storage::install(provider) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("[c-bridge] storage install: {e}");
            1
        }
    }
}

unsafe extern "C" fn n_storage_active() -> i32 {
    storage::storage_active() as i32
}

// ---------------------------------------------------------------------------
// threads (brick threads)
// ---------------------------------------------------------------------------

unsafe extern "C" fn n_threads_spawn(name: *const c_char, f: SchedulerTaskCb, ctx: *mut c_void) -> i32 {
    if fnull(f) {
        return -1;
    }
    let name = cstr(name).to_string();
    let th: Arc<(SchedulerTaskCb, Ctx)> = Arc::new((f, Ctx(ctx)));
    threads::spawn_attached(&name, move || (th.0)(th.1 .0));
    0
}

unsafe extern "C" fn n_threads_spawn_daemon(
    name: *const c_char,
    f: SchedulerTaskCb,
    ctx: *mut c_void,
) -> i32 {
    if fnull(f) {
        return -1;
    }
    let name = cstr(name).to_string();
    let t: Arc<(SchedulerTaskCb, Ctx)> = Arc::new((f, Ctx(ctx)));
    match threads::PlatformThread::spawn_daemon(&name, move || (t.0)(t.1 .0)) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

unsafe extern "C" fn n_threads_current_name(out: *mut c_char, out_len: usize) -> i32 {
    if out.is_null() || out_len == 0 {
        return -1;
    }
    let name = threads::current_thread_info().unwrap_or_default();
    let bytes = name.as_bytes();
    let n = bytes.len().min(out_len - 1);
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), out as *mut u8, n);
    *out.add(n) = 0;
    0
}

// ---------------------------------------------------------------------------
// transform (brick 1)
// ---------------------------------------------------------------------------

unsafe extern "C" fn n_transform_register_rule(
    class_pattern: *const c_char,
    method: *const c_char,
    descriptor: *const c_char,
    injection: i32,
    helper: *const c_char,
) -> i32 {
    if class_pattern.is_null() || method.is_null() || descriptor.is_null() {
        return -1;
    }
    let injection = match injection {
        0 => transform::Injection::MethodEntry,
        1 => transform::Injection::BeforeCall(cstr(helper).to_string()),
        _ => return -2,
    };
    let engine = transform::global_engine();
    engine.register(transform::Rule::new(
        cstr(class_pattern),
        cstr(method),
        cstr(descriptor),
        injection,
        if helper.is_null() { "" } else { cstr(helper) },
    ));
    0
}

// ---------------------------------------------------------------------------
// save_events (brick 8)
// ---------------------------------------------------------------------------

unsafe extern "C" fn n_save_events_on_save(cb: SaveCb, ctx: *mut c_void) -> i32 {
    if fnull(cb) {
        return -1;
    }
    let th: Arc<(SaveCb, Ctx)> = Arc::new((cb, Ctx(ctx)));
    save_events::on_save(Arc::new(move |outcome: save_events::SaveOutcome| {
        let kind = match outcome.kind {
            save_events::SaveKind::Autosave => 0,
            save_events::SaveKind::Manual => 1,
        };
        let status = match outcome.status {
            save_events::SaveStatus::Ok => 0,
            save_events::SaveStatus::Failed => 1,
        };
        (th.0)(kind, status, outcome.chunks_written, outcome.duration_ms, th.1 .0);
    }));
    0
}

// ---------------------------------------------------------------------------
// hot_reload (brick 10)
// ---------------------------------------------------------------------------

unsafe extern "C" fn n_hot_reload_module(id: *const c_char) -> i32 {
    if id.is_null() {
        return -1;
    }
    match hot_reload::reload_module(cstr(id)) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("[c-bridge] hot_reload: {e}");
            1
        }
    }
}

unsafe extern "C" fn n_hot_reload_enter(id: *const c_char) -> i32 {
    if id.is_null() {
        return -1;
    }
    hot_reload::enter_module(cstr(id)) as i32
}

unsafe extern "C" fn n_hot_reload_leave(id: *const c_char) -> i32 {
    if id.is_null() {
        return -1;
    }
    hot_reload::leave_module(cstr(id));
    0
}

// ---------------------------------------------------------------------------
// side_table (brick 2)
// ---------------------------------------------------------------------------

unsafe extern "C" fn n_side_table_key(obj: *mut c_void, out: *mut u64) -> i32 {
    if obj.is_null() || out.is_null() {
        return -1;
    }
    match side_table::key_from_jobject(obj as *mut _) {
        Some(k) => {
            *out = k.0;
            0
        }
        None => -2,
    }
}

unsafe extern "C" fn n_side_table_named(name: *const c_char, out: *mut u64) -> i32 {
    if fnull(name) || out.is_null() {
        return -1;
    }
    match side_table::named_table(cstr(name)) {
        Some(k) => {
            *out = k.0;
            0
        }
        None => -2,
    }
}

// ---------------------------------------------------------------------------
// table
// ---------------------------------------------------------------------------

/// The full C-visible brick surface. `version` stays the same across binary
/// table layouts; new entries are appended at the END of `CPlatformApi`.
pub static PLATFORM_API: CPlatformApi = CPlatformApi {
    version: CPB_VERSION,
    events_subscribe: Some(n_events_subscribe),
    events_unsubscribe: Some(n_events_unsubscribe),
    events_publish: Some(n_events_publish),
    scheduler_inject: Some(n_scheduler_inject),
    scheduler_current_tick: Some(n_scheduler_current_tick),
    scheduler_injected_pending: Some(n_scheduler_injected_pending),
    telemetry_publish_metric: Some(n_telemetry_publish_metric),
    telemetry_snapshot_json: Some(n_telemetry_snapshot_json),
    signals_on_fault: Some(n_signals_on_fault),
    signals_fault_count: Some(n_signals_fault_count),
    signals_crash_log: Some(n_signals_crash_log),
    network_add_hook: Some(n_network_add_hook),
    network_attach_conn: Some(n_network_attach_conn),
    network_detach_conn: Some(n_network_detach_conn),
    network_conn_state: Some(n_network_conn_state),
    network_conn_count: Some(n_network_conn_count),
    storage_install: Some(n_storage_install),
    storage_active: Some(n_storage_active),
    threads_spawn: Some(n_threads_spawn),
    threads_spawn_daemon: Some(n_threads_spawn_daemon),
    threads_current_name: Some(n_threads_current_name),
    transform_register_rule: Some(n_transform_register_rule),
    save_events_on_save: Some(n_save_events_on_save),
    hot_reload_module: Some(n_hot_reload_module),
    hot_reload_enter: Some(n_hot_reload_enter),
    hot_reload_leave: Some(n_hot_reload_leave),
    side_table_key: Some(n_side_table_key),
    side_table_named: Some(n_side_table_named),
};

// ---------------------------------------------------------------------------
// tests (TASK-179: C-entry pre-parse gate)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::hint::black_box;
    use std::time::Instant;

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    /// Semantics of the C entry must be identical in both A/B states: the
    /// gate only skips work whose result publish would drop untouched.
    #[test]
    fn c_entry_pregate_matches_publish_visibility() {
        let _guard = TEST_LOCK.lock().unwrap();

        // zero subscribers: valid JSON, malformed JSON and a NULL payload
        // pointer all report 0 (gate ON: the parse never runs; gate OFF:
        // the parsed value reaches publish's fast gate and is dropped —
        // the returned count is the same 0).
        let ev = CString::new("c.gate.nosub").unwrap();
        let pj = CString::new("{\"tick\":1,\"drained\":0}").unwrap();
        let bad = CString::new("{not json").unwrap();
        assert_eq!(unsafe { n_events_publish(ev.as_ptr(), pj.as_ptr()) }, 0);
        assert_eq!(unsafe { n_events_publish(ev.as_ptr(), bad.as_ptr()) }, 0);
        assert_eq!(unsafe { n_events_publish(ev.as_ptr(), std::ptr::null()) }, 0);

        // one sync subscriber: delivery identical to publish — valid JSON
        // parsed into the payload, malformed JSON -> Null, NULL pointer ->
        // Null; an empty (NULL) event name is rejected by the entry itself.
        let seen = Arc::new(Mutex::new(Vec::<Value>::new()));
        let s2 = Arc::clone(&seen);
        let sub = events::global().subscribe("c.gate.sync", Arc::new(move |_, payload| {
            s2.lock().unwrap().push(payload.clone());
        }));
        let evs = CString::new("c.gate.sync").unwrap();
        assert_eq!(unsafe { n_events_publish(evs.as_ptr(), pj.as_ptr()) }, 1);
        assert_eq!(unsafe { n_events_publish(evs.as_ptr(), bad.as_ptr()) }, 1);
        assert_eq!(unsafe { n_events_publish(evs.as_ptr(), std::ptr::null()) }, 1);
        assert_eq!(unsafe { n_events_publish(std::ptr::null(), pj.as_ptr()) }, 0);
        assert_eq!(
            *seen.lock().unwrap(),
            vec![
                serde_json::json!({ "tick": 1u64, "drained": 0u64 }),
                Value::Null,
                Value::Null,
            ],
            "payload delivery through the C entry matches publish"
        );
        assert!(events::global().unsubscribe("c.gate.sync", &sub));
        assert_eq!(unsafe { n_events_publish(evs.as_ptr(), pj.as_ptr()) }, 0);
    }

    /// TASK-179 A/B subject: the full C publish entry. Line 1 is the gate
    /// case — nothing ever subscribed on the global bus, so with the gate
    /// ON the JSON parse never runs; with it OFF every call parses the
    /// payload and drops it in publish's fast gate. Line 2 (one sync
    /// subscriber) must be toggle-independent within noise: the parse is
    /// required for delivery in both states, the gate adds one acquire
    /// load. Global-bus state evolves identically in both builds (the
    /// subscribe happens after line 1), so the pair is directly
    /// comparable; line 1 is deterministic because a filtered bench run
    /// executes only this test (fresh global bus, gens == 0).
    #[test]
    #[ignore]
    fn bench_c_events_publish() {
        let ev = CString::new("c.bench.nosub").unwrap();
        let pj = CString::new("{\"tick\":1,\"drained\":0}").unwrap();
        let iters = 200_000u32;
        let rounds = 5;

        // line 1: zero subscribers on the global bus
        for _ in 0..10_000u32 {
            black_box(unsafe { n_events_publish(ev.as_ptr(), pj.as_ptr()) });
        }
        let mut best_none = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                black_box(unsafe { n_events_publish(ev.as_ptr(), pj.as_ptr()) });
            }
            best_none = best_none.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }

        // line 2: one sync subscriber — parse required for delivery in
        // both states; the trivial handler matches the publish(1 sync sub)
        // harness shape in events.rs so the lines stay comparable.
        let sub = events::global().subscribe("c.bench.sync", Arc::new(|_, _| {}));
        let evs = CString::new("c.bench.sync").unwrap();
        for _ in 0..10_000u32 {
            black_box(unsafe { n_events_publish(evs.as_ptr(), pj.as_ptr()) });
        }
        let mut best_sync = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                black_box(unsafe { n_events_publish(evs.as_ptr(), pj.as_ptr()) });
            }
            best_sync = best_sync.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }
        let _ = events::global().unsubscribe("c.bench.sync", &sub);

        println!(
            "BENCH c_publish: c_publish(no subs) {:.0} ns/op, c_publish(1 sync sub) {:.0} ns/op (min of {rounds}x{iters})",
            best_none * 1e9,
            best_sync * 1e9
        );
    }

    /// TASK-180: C telemetry entry state-independent contract. These two
    /// asserts hold regardless of the metric list state (parallel test
    /// processes may have it empty, partially filled or full): an empty
    /// name is rejected with -1, a NULL labels pointer is accepted with 0.
    /// State-DEPENDENT semantics (malformed labels -> -2 with room, -> 0
    /// once the list is full) live in bench_telemetry_c_publish — they
    /// require the controlled solo state a parallel full-suite run cannot
    /// provide (the metric list has no removal API and other tests read
    /// it concurrently).
    #[test]
    fn telemetry_c_entry_pregate_semantics() {
        let _guard = TEST_LOCK.lock().unwrap();

        let name_ok = CString::new("c.gate.metric").unwrap();
        let labels_ok = CString::new("{\"region\":\"eu\"}").unwrap();

        assert_eq!(
            unsafe { n_telemetry_publish_metric(std::ptr::null(), 1.0, std::ptr::null(), labels_ok.as_ptr()) },
            -1,
            "empty name is rejected in any state"
        );
        assert_eq!(
            unsafe { n_telemetry_publish_metric(name_ok.as_ptr(), 1.0, std::ptr::null(), std::ptr::null()) },
            0,
            "NULL labels are accepted in any state (room: published, full: capped)"
        );
    }

    /// TASK-180 A/B subject: the full C telemetry entry in the steady state
    /// (list at MAX_METRICS — periodic metric publishers live here forever,
    /// nothing is ever removed). OFF: the labels JSON is parsed into a
    /// HashMap and then dropped by the cap on every call; ON: the entry
    /// returns after the cheap full-check. Control line: NULL labels (no
    /// parse in either state — the gate must not cost anything there).
    /// Runs SOLO (filtered run only): it resets the shared snapshot,
    /// asserts the room-state contract (malformed labels -> -2), fills the
    /// list to the cap and asserts the full-state contract (nothing
    /// publishes; malformed labels report 0 — the documented decision),
    /// then benches the full state. The parallel full-suite run must never
    /// see the list filled by this bench — hence #[ignore].
    #[test]
    #[ignore]
    fn bench_telemetry_c_publish() {
        telemetry::test_reset_snapshot();

        // room state: the pre-TASK-180 0 / -2 contract is intact
        let nm = CString::new("c.bench.metric").unwrap();
        let lb1 = CString::new("{\"region\":\"eu\"}").unwrap();
        let bad = CString::new("{not json").unwrap();
        assert_eq!(
            unsafe { n_telemetry_publish_metric(nm.as_ptr(), 1.0, std::ptr::null(), lb1.as_ptr()) },
            0,
            "with room: valid labels accepted"
        );
        assert_eq!(
            unsafe { n_telemetry_publish_metric(nm.as_ptr(), 1.0, std::ptr::null(), bad.as_ptr()) },
            -2,
            "with room: malformed labels rejected before any cap logic"
        );

        // fill to the cap (unique names, no labels)
        let mut i = 0u64;
        while !telemetry::metrics_full() {
            let n = CString::new(format!("c.bench.fill.{i}")).unwrap();
            telemetry::publish_metric(n.to_str().unwrap(), i as f64, None, None);
            i += 1;
            assert!(i < 10_000, "metric list never filled");
        }
        assert!(telemetry::metrics_full(), "bench requires the full state");

        // full state: nothing publishes in either toggle state; the
        // malformed-labels CODE follows the gate decision (documented):
        // ON skips the parse -> 0, OFF parses and rejects -> -2
        assert_eq!(
            unsafe { n_telemetry_publish_metric(nm.as_ptr(), 2.0, std::ptr::null(), lb1.as_ptr()) },
            0,
            "full: the metric is capped in both states"
        );
        assert_eq!(
            unsafe { n_telemetry_publish_metric(nm.as_ptr(), 2.0, std::ptr::null(), bad.as_ptr()) },
            if TELEM_PREGATE { 0 } else { -2 },
            "full: malformed-labels code follows the gate decision (documented)"
        );

        let lb4 = CString::new(
            "{\"region\":\"eu-central-1\",\"world\":\"overworld\",\"dim\":\"nether\",\"tier\":2}",
        )
        .unwrap();
        let iters = 200_000u32;
        let rounds = 5;

        // line 1: full + labeled (4 pairs) — the parse-then-drop shape
        for _ in 0..10_000u32 {
            black_box(unsafe { n_telemetry_publish_metric(nm.as_ptr(), 1.0, std::ptr::null(), lb4.as_ptr()) });
        }
        let mut best_labeled = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                black_box(unsafe { n_telemetry_publish_metric(nm.as_ptr(), 1.0, std::ptr::null(), lb4.as_ptr()) });
            }
            best_labeled = best_labeled.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }

        // control line: full + NULL labels (no parse in either state)
        for _ in 0..10_000u32 {
            black_box(unsafe { n_telemetry_publish_metric(nm.as_ptr(), 1.0, std::ptr::null(), std::ptr::null()) });
        }
        let mut best_null = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                black_box(unsafe { n_telemetry_publish_metric(nm.as_ptr(), 1.0, std::ptr::null(), std::ptr::null()) });
            }
            best_null = best_null.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }

        println!(
            "BENCH c_telemetry: c_metric(full, labels4) {:.0} ns/op, c_metric(full, no labels) {:.0} ns/op (min of {rounds}x{iters})",
            best_labeled * 1e9,
            best_null * 1e9
        );
    }

    /// TASK-181: C snapshot-entry state-independent contract (parallel
    /// test): the entry returns a non-NULL, NUL-terminated JSON object
    /// carrying the base snapshot fields in ANY shared state (empty /
    /// partial / full metric list; parallel tests may publish metrics
    /// concurrently). Two successive calls must both yield a valid,
    /// parseable object — each call refreshes the buffer, and the returned
    /// pointer is valid until the NEXT call (the pre-TASK-181 contract,
    /// unchanged by the round).
#[test]
    fn telemetry_snapshot_inplace_contract() {
        let _guard = TEST_LOCK.lock().unwrap();

        let p1 = unsafe { n_telemetry_snapshot_json() };
        assert!(!p1.is_null(), "snapshot entry returns a pointer in any state");
        let b1 = unsafe { CStr::from_ptr(p1) }.to_bytes();
        let v1: serde_json::Value = serde_json::from_slice(b1).expect("call 1: valid JSON");
        assert_eq!(
            v1["runtime_version"].as_str(),
            Some(env!("CARGO_PKG_VERSION")),
            "runtime_version is the crate version in any state"
        );
        assert!(v1["server_name"].is_string(), "server_name always present");
        assert!(v1["started_at"].is_u64(), "started_at always present");
        assert!(v1["tps"].is_number(), "tps always present");
        assert!(v1["metrics"].is_array(), "metrics always serialized");

        let p2 = unsafe { n_telemetry_snapshot_json() };
        assert!(!p2.is_null());
        let b2 = unsafe { CStr::from_ptr(p2) }.to_bytes();
        let v2: serde_json::Value = serde_json::from_slice(b2).expect("call 2: valid JSON");
        assert_eq!(v2["server_name"], v1["server_name"], "stable across calls");
        assert_eq!(v2["started_at"], v1["started_at"], "stable across calls");
    }

    /// TASK-181: byte-identity between the two toggle paths in ONE build
    /// (SOLO, filtered run — resets the shared snapshot and publishes a
    /// controlled fixture; the parallel suite may publish concurrently,
    /// which would change the JSON between the two calls): the legacy
    /// shape (deep clone -> to_string -> CString swap) and the in-place
    /// shape (serialize under the lock into the reused buffer) must emit
    /// byte-identical JSON in the empty state and in the labeled state.
    /// The ring is empty in tests, so tps is deterministic (0.0) and both
    /// read points agree.
#[test]
    #[ignore]
    fn telemetry_snapshot_inplace_byte_identical() {
        telemetry::test_reset_snapshot();

        // legacy shape re-derived here independently (the dispatch fn is a
        // single unit — no extracted legacy fn; TASK-181 iteration 2 showed
        // that extracting one shifts unrelated CPU-bound loop placement)
        let legacy_shape = || {
            let s = telemetry::snapshot();
            serde_json::to_string(&s).unwrap_or_else(|_| "{}".into())
        };
        let legacy1 = legacy_shape().into_bytes();
        let inplace1 =
            unsafe { CStr::from_ptr(n_telemetry_snapshot_json()) }.to_bytes().to_vec();
        assert_eq!(legacy1, inplace1, "empty state: byte-identical JSON");
        let v1: serde_json::Value = serde_json::from_slice(&inplace1).unwrap();
        assert_eq!(v1["metrics"].as_array().unwrap().len(), 0, "empty state");

        for i in 0..32u32 {
            let mut labels = HashMap::new();
            labels.insert("region".to_string(), "eu".to_string());
            labels.insert("idx".to_string(), i.to_string());
            telemetry::publish_metric(
                &format!("c.bench.ident.{i}"),
                i as f64,
                Some("ms"),
                Some(labels),
            );
        }

        let legacy2 = legacy_shape().into_bytes();
        let inplace2 =
            unsafe { CStr::from_ptr(n_telemetry_snapshot_json()) }.to_bytes().to_vec();
        assert_eq!(legacy2, inplace2, "32-metric state: byte-identical JSON");
        let v2: serde_json::Value = serde_json::from_slice(&inplace2).unwrap();
        assert_eq!(v2["metrics"].as_array().unwrap().len(), 32);
        assert_eq!(v2["metrics"][0]["name"], "c.bench.ident.0");
        assert_eq!(v2["metrics"][0]["labels"]["region"], "eu");
    }

    /// TASK-181 A/B subject: the C telemetry snapshot entry. OFF (legacy
    /// shape): every call deep-clones the whole Snapshot under the data
    /// lock (every String / labels map — up to MAX_METRICS metrics),
    /// serializes the clone into a fresh String, then swaps a fresh CString
    /// into the static (freeing the old one) — the clone is dropped
    /// untouched by the serializer. ON: serializes the locked snapshot in
    /// place into a REUSED buffer (transient tps override, zero allocs in
    /// the steady state). Line 1: empty metric list (control — base fields
    /// only). Line 2: 32 labeled metrics (the clone tax scales with the
    /// list; a realistic active-module fleet). Runs SOLO (filtered run
    /// only): resets the shared snapshot and leaves 32 metrics published
    /// at the end (harmless — every solo bench resets its own starting
    /// state; the parallel suite never depends on the list being empty).
    /// Byte-identity between the toggle paths is asserted by the separate
    /// solo test telemetry_snapshot_inplace_byte_identical.
#[test]
    #[ignore]
    fn bench_telemetry_snapshot_json() {
        telemetry::test_reset_snapshot();

        let iters = 200_000u32;
        let rounds = 5;

        // line 1: empty metric list (control)
        for _ in 0..10_000u32 {
            black_box(unsafe { n_telemetry_snapshot_json() });
        }
        let mut best_empty = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                black_box(unsafe { n_telemetry_snapshot_json() });
            }
            best_empty = best_empty.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }

        // 32 labeled metrics (4 label pairs each)
        for i in 0..32u32 {
            let mut labels = HashMap::new();
            labels.insert("region".to_string(), "eu".to_string());
            labels.insert("host".to_string(), format!("host-{i}"));
            labels.insert("module".to_string(), format!("mod-{i}"));
            labels.insert("idx".to_string(), i.to_string());
            telemetry::publish_metric(
                &format!("c.bench.snap.{i}"),
                i as f64,
                Some("ms"),
                Some(labels),
            );
        }

        for _ in 0..10_000u32 {
            black_box(unsafe { n_telemetry_snapshot_json() });
        }
        let mut best_32 = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                black_box(unsafe { n_telemetry_snapshot_json() });
            }
            best_32 = best_32.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }

        println!(
            "BENCH snapshot_json: snapshot_json(0 metrics) {:.0} ns/op, snapshot_json(32 labeled) {:.0} ns/op (min of {rounds}x{iters})",
            best_empty * 1e9,
            best_32 * 1e9
        );
    }
}