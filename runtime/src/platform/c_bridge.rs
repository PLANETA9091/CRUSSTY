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
    let payload: Value = if payload_json.is_null() {
        Value::Null
    } else {
        serde_json::from_str(cstr(payload_json)).unwrap_or(Value::Null)
    };
    events::global().publish(event, &payload)
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

unsafe extern "C" fn n_telemetry_snapshot_json() -> *const c_char {
    let s = telemetry::snapshot();
    let json = serde_json::to_string(&s).unwrap_or_else(|_| "{}".into());
    let ptr = SNAP.get_or_init(|| Mutex::new(None));
    let mut guard = ptr.lock().unwrap();
    *guard = Some(CString::new(json).unwrap());
    guard.as_ref().unwrap().as_ptr()
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