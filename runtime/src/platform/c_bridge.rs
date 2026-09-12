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
use serde_json::{Number, Value};
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

/// TASK-203 A/B toggle: gate-FIRST ordering for the C events publish
/// entry — the TASK-179 gate runs BEFORE the event string is
/// materialized, so the zero-subscriber steady state (the per-tick
/// default shape) skips the cstr walk entirely (strlen + UTF-8
/// validation): the entry is one global_ref + ONE acquire load + return
/// 0. Implies C_ENTRY_PREGATE (the gate itself is unchanged; only its
/// position moves). `false` = the verbatim pre-TASK-203 ordering
/// (cstr -> empty-check -> gate), kept as the A/B arm. Semantics are
/// identical in both states: behind the firing gate the entry returns 0 —
/// the same 0 the old shape returned there for empty AND non-empty
/// events (no dispatch runs; the payload and the event bytes are
/// unobservable without subscribers — the same no-new-guarantee argument
/// TASK-179 documents; a subscribe racing the window may or may not
/// observe the call, the gate only shortens it, now a few ns earlier).
/// A/B: bench_c_publish_gfirst_ab.
pub const C_ENTRY_PREGATE_FIRST: bool = true;

/// TASK-180 A/B toggle: gate the C telemetry entry's labels parse behind the
/// metrics-full check. `false` = pre-TASK-180 shape (labels always parsed;
/// publish_metric then drops the metric when the list is at MAX_METRICS).
/// When the list is full neither state publishes anything; the ONE
/// documented observable difference is the error code for a MALFORMED
/// labels string in the full state (0 instead of -2, the parse never ran);
/// the 0 / -1 / -2 contract is unchanged in every state where a metric
/// could actually be accepted. TASK-202: the full check itself is now
/// lock-free (one Relaxed load of telemetry's fullness flag — the
/// pre-202 shape paid a full data-lock pair here on every call); the
/// stale-false transition window is documented on telemetry's
/// METRICS_FULL_FLAG and is the same observable class as the difference
/// above.
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

/// TASK-200 A/B toggle: the C-entry snapshot buffer cycle under the ONE
/// cache lock. `true` = telemetry::snapshot_c_entry serves the whole
/// cycle (clear + assemble + NUL push) under the SNAP_JSON_CACHE cache
/// lock — the pre-200 SNAP_BUF Mutex pair is off the hot path. Writer
/// serialization is UNCHANGED: snapshot_json_write already held the
/// cache lock across its entire call body (hit and miss alike), so two
/// concurrent C-entry writes serialize on the cache lock exactly as they
/// serialized on the buffer mutex. The reader contract is UNCHANGED:
/// pointer valid until the next call (from any thread), consumed outside
/// any lock — exactly as the pre-200 shape, where the mutex guard was
/// released before the C consumer read the pointer (the mutex never
/// protected readers, only writers). `false` = the pre-TASK-200 shape
/// kept verbatim for the A/B bench (and as the byte-identity reference
/// implementation).
pub const TELEM_SNAP_ONELOCK: bool = true;

/// TASK-193 A/B toggle: hand-rolled fast-path parse for the flat payload
/// shapes C producers actually emit (null/true/false, integers, escape-free
/// strings, flat objects/arrays of exactly those leaves). ANYTHING the
/// scanner is not 100% sure about falls back to serde_json::from_str, so
/// the result is semantically identical to the pre-TASK-193 shape in every
/// case: input inside the proven subset -> the same Value serde builds;
/// everything else -> serde decides exactly as before (invalid input still
/// lands on Value::Null through unwrap_or). Excluded from the subset on
/// purpose: floats (serde's float resolution is not re-implemented),
/// leading zeros, "-0", escapes, raw control bytes, nested containers,
/// any whitespace inside containers — every one of those routes to serde.
pub const C_PUBLISH_FASTPARSE: bool = true;

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

/// TASK-193: C-entry payload parse = fast path first, serde fallback.
#[inline]
fn c_publish_parse(s: &str) -> Value {
    if C_PUBLISH_FASTPARSE {
        if let Some(v) = parse_payload_fast(s) {
            return v;
        }
    }
    serde_json::from_str(s).unwrap_or(Value::Null)
}

/// Fast-path scanner: Some = confidently parsed inside the proven subset;
/// None = the caller must fall back to serde (same result as pre-TASK-193).
fn parse_payload_fast(s: &str) -> Option<Value> {
    match s.as_bytes() {
        b"null" => return Some(Value::Null),
        b"true" => return Some(Value::Bool(true)),
        b"false" => return Some(Value::Bool(false)),
        _ => {}
    }
    if s.is_empty() {
        return None;
    }
    match s.as_bytes()[0] {
        b'"' => scan_simple_string(s).map(Value::String),
        b'{' => scan_flat_object(s),
        b'[' => scan_flat_array(s),
        b'-' | b'0'..=b'9' => scan_integer(s),
        _ => None,
    }
}

/// Whole-input string: quotes at both ends, no escapes, no control bytes,
/// no raw '"' inside (a raw inner quote means invalid JSON — serde rejects
/// it, so the fallback must decide it, not us).
fn scan_simple_string(s: &str) -> Option<String> {
    let b = s.as_bytes();
    if b.len() < 2 || b[0] != b'"' || b[b.len() - 1] != b'"' {
        return None;
    }
    let inner = &b[1..b.len() - 1];
    for &c in inner {
        if c == b'\\' || c == b'"' || c < 0x20 {
            return None;
        }
    }
    Some(s[1..s.len() - 1].to_string())
}

/// String token at cursor *i (which must point at an opening quote):
/// scans to the FIRST closing quote, rejecting escapes and control bytes.
/// A raw inner quote ends the token early and the caller's structural
/// check then rejects the input — parity is preserved through the fallback.
fn scan_string_at(s: &str, i: &mut usize) -> Option<String> {
    let b = s.as_bytes();
    let start = *i + 1;
    let mut j = start;
    loop {
        let c = *b.get(j)?;
        if c == b'"' {
            let t = s[start..j].to_string();
            *i = j + 1;
            return Some(t);
        }
        if c == b'\\' || c < 0x20 {
            return None;
        }
        j += 1;
    }
}

/// Integer token only: [-]digits with no leading zero ("0" alone is fine).
/// Floats (any '.'/'e'/'E'), overflow and "-0" are excluded — serde decides.
fn scan_integer(s: &str) -> Option<Value> {
    let b = s.as_bytes();
    let (neg, digits) = match b[0] {
        b'-' => (true, &b[1..]),
        _ => (false, b),
    };
    if digits.is_empty() || (digits[0] == b'0' && digits.len() > 1) {
        return None;
    }
    if !digits.iter().all(|d| d.is_ascii_digit()) {
        return None;
    }
    if neg {
        if digits == b"0" {
            return None;
        }
        s.parse::<i64>().ok().map(|v| Value::Number(v.into()))
    } else {
        s.parse::<u64>().ok().map(|v| Value::Number(v.into()))
    }
}

/// Leaf value at cursor *i: null/true/false, integer, or simple string.
/// Nested containers and whitespace return None (serde decides).
fn scan_leaf(s: &str, i: &mut usize) -> Option<Value> {
    let b = s.as_bytes();
    match b[*i] {
        b'n' if b.len() - *i >= 4 && &b[*i..*i + 4] == b"null" => {
            *i += 4;
            Some(Value::Null)
        }
        b't' if b.len() - *i >= 4 && &b[*i..*i + 4] == b"true" => {
            *i += 4;
            Some(Value::Bool(true))
        }
        b'f' if b.len() - *i >= 5 && &b[*i..*i + 5] == b"false" => {
            *i += 5;
            Some(Value::Bool(false))
        }
        b'"' => scan_string_at(s, i).map(Value::String),
        b'-' | b'0'..=b'9' => {
            let start = *i;
            let mut j = start;
            if b[j] == b'-' {
                j += 1;
            }
            while j < b.len() && b[j].is_ascii_digit() {
                j += 1;
            }
            if j == start + usize::from(b[start] == b'-') {
                return None; // sign with no digits
            }
            // Only a structural terminator may follow: a '.'/'e'/'E' or any
            // other byte means serde must decide (floats, garbage).
            match b.get(j) {
                None | Some(b',') | Some(b'}') | Some(b']') => {}
                _ => return None,
            }
            let tok = &s[start..j];
            let v = if tok.starts_with('-') {
                if tok == "-0" || (tok.len() > 2 && tok.starts_with("-0")) {
                    return None;
                }
                Value::Number(tok.parse::<i64>().ok()?.into())
            } else {
                if tok.len() > 1 && tok.starts_with('0') {
                    return None;
                }
                Value::Number(tok.parse::<u64>().ok()?.into())
            };
            *i = j;
            Some(v)
        }
        _ => None,
    }
}

/// Flat object: keys are simple strings, leaves from scan_leaf, no
/// whitespace inside (serde accepts whitespace; we just fall back to it).
fn scan_flat_object(s: &str) -> Option<Value> {
    let b = s.as_bytes();
    if b.len() < 2 || b[b.len() - 1] != b'}' {
        return None;
    }
    let mut map = serde_json::Map::new();
    let mut i = 1usize;
    if b[i] == b'}' {
        return Some(Value::Object(map));
    }
    loop {
        if b[i] != b'"' {
            return None;
        }
        let key = scan_string_at(s, &mut i)?;
        if b.get(i) != Some(&b':') {
            return None;
        }
        i += 1;
        let v = scan_leaf(s, &mut i)?;
        map.insert(key, v);
        match b.get(i) {
            Some(b',') => i += 1,
            Some(b'}') => return Some(Value::Object(map)),
            _ => return None,
        }
    }
}

/// Flat array of scan_leaf values (no nesting, no whitespace).
fn scan_flat_array(s: &str) -> Option<Value> {
    let b = s.as_bytes();
    if b.len() < 2 || b[b.len() - 1] != b']' {
        return None;
    }
    let mut vec = Vec::new();
    let mut i = 1usize;
    if b[i] == b']' {
        return Some(Value::Array(vec));
    }
    loop {
        let v = scan_leaf(s, &mut i)?;
        vec.push(v);
        match b.get(i) {
            Some(b',') => i += 1,
            Some(b']') => return Some(Value::Array(vec)),
            _ => return None,
        }
    }
}

unsafe extern "C" fn n_events_publish(event: *const c_char, payload_json: *const c_char) -> usize {
    // TASK-203: gate-first ordering (C_ENTRY_PREGATE_FIRST). With the
    // toggle on, the TASK-179 gate consumes the event pointer FIRST —
    // the zero-subscriber steady state never walks the event string.
    // The fall-through body is the pre-203 shape minus the now-redundant
    // second gate load (publish's own fast gate still re-reads gens).
    if C_ENTRY_PREGATE_FIRST {
        // Borrowed global handle (TASK-163 pattern): the owned global()
        // clone costs ~5 Arc refcount pairs per call — hot C publishes
        // pay it for nothing, the &'static handle serves gate and publish
        // alike.
        let bus = events::global_ref();
        // TASK-179 pre-parse gate, TASK-203 position: when nothing was
        // ever subscribed, publish() would drop the payload untouched and
        // return 0 — skip the cstr walk AND the JSON parse entirely. Same
        // acquire load publish's own fast gate uses; semantics identical
        // (return 0 here matches the old shape's 0 for empty and
        // non-empty events alike — nothing behind the gate is
        // observable), the race window shortens exactly like the
        // in-publish gate does.
        if !bus.may_have_subscribers() {
            return 0;
        }
        let event = cstr(event);
        if event.is_empty() {
            return 0;
        }
        let payload: Value = if payload_json.is_null() {
            Value::Null
        } else {
            // TASK-193: fast path for the flat producer shapes, serde
            // fallback for everything else — same Value in every case
            // (see C_PUBLISH_FASTPARSE for the parity argument).
            c_publish_parse(cstr(payload_json))
        };
        return bus.publish(event, &payload);
    }
    // Verbatim pre-TASK-203 shape (C_ENTRY_PREGATE_FIRST = false arm).
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
        // TASK-193: fast path for the flat producer shapes, serde fallback
        // for everything else — same Value in every case (see
        // C_PUBLISH_FASTPARSE for the parity argument).
        c_publish_parse(cstr(payload_json))
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
/// TASK-200: this static now serves ONLY the TELEM_SNAP_ONELOCK=false A/B
/// arm below (and the A/B bench arm that re-derives the same shape); the
/// default path keeps the buffer inside the telemetry cache struct
/// (SnapshotJsonCache::c_buf) under the ONE cache lock.
static SNAP_BUF: OnceLock<Mutex<Vec<u8>>> = OnceLock::new();

unsafe extern "C" fn n_telemetry_snapshot_json() -> *const c_char {
    if TELEM_SNAP_INPLACE {
        if TELEM_SNAP_ONELOCK {
            // TASK-200: the whole buffer cycle (clear + assemble + NUL)
            // runs under the ONE cache lock inside
            // telemetry::snapshot_c_entry — the pre-200 SNAP_BUF mutex
            // pair is off the hot path. Writer serialization unchanged
            // (the cache lock already spanned every snapshot_json_write
            // call); reader exposure unchanged (the pointer is consumed
            // outside any lock under the TASK-181 valid-until-next-call
            // contract); lock nesting SHALLOWER (the mutex -> cache ->
            // data chain loses its first level).
            telemetry::snapshot_c_entry()
        } else {
            // Pre-TASK-200 shape, kept verbatim for the A/B toggle (the
            // byte-identity test re-derives this shape independently).
            let buf = SNAP_BUF.get_or_init(|| Mutex::new(Vec::with_capacity(4096)));
            let mut guard = buf.lock().unwrap();
            guard.clear();
            telemetry::snapshot_json_write(guard.as_mut());
            guard.push(0);
            guard.as_ptr() as *const c_char
        }
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
    use std::sync::atomic::{AtomicBool, Ordering};
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

    /// TASK-202: the lock-free pre-parse gate contract — the flag is
    /// maintained by publish_metric (the Rust path included), exact in
    /// both steady states, cleared by the test resets, and the C entry's
    /// 0/-2 codes follow the state through the transition. Holds the
    /// module TEST_LOCK (the metric list is shared global state); note
    /// telemetry-module tests may still interleave (cross-module locks),
    /// so no ABSOLUTE list-length asserts — only the flag and the codes.
    #[test]
    fn cmetric_gate_flag_contract() {
        let _guard = TEST_LOCK.lock().unwrap();

        telemetry::test_reset_snapshot();
        assert!(!telemetry::metrics_full(), "fresh state: room");

        // room contract via the C entry: accepted / malformed rejected.
        let nm = CString::new("c.gate.contract").unwrap();
        let lb = CString::new("{\"k\":\"v\"}").unwrap();
        let bad = CString::new("{nope").unwrap();
        assert_eq!(
            unsafe { n_telemetry_publish_metric(nm.as_ptr(), 1.0, std::ptr::null(), lb.as_ptr()) },
            0,
            "room: valid labels accepted"
        );
        assert_eq!(
            unsafe { n_telemetry_publish_metric(nm.as_ptr(), 1.0, std::ptr::null(), bad.as_ptr()) },
            -2,
            "room: malformed labels rejected before any cap logic"
        );
        assert!(!telemetry::metrics_full(), "one metric published: still room");

        // fill DIRECTLY via the Rust publish path (it maintains the flag
        // too) — the flag must land exactly when the list hits the cap.
        let mut i = 0u64;
        while !telemetry::metrics_full() {
            let n = format!("c.gate.fill.{i}");
            telemetry::publish_metric(&n, i as f64, None, None);
            i += 1;
            assert!(i < 10_000, "metric list never filled");
        }

        // full contract via the C entry: capped 0 for valid labels, and
        // the documented gate decision 0 for malformed (parse skipped).
        assert_eq!(
            unsafe { n_telemetry_publish_metric(nm.as_ptr(), 2.0, std::ptr::null(), lb.as_ptr()) },
            0,
            "full: the metric is capped"
        );
        assert_eq!(
            unsafe { n_telemetry_publish_metric(nm.as_ptr(), 2.0, std::ptr::null(), bad.as_ptr()) },
            0,
            "full: the gate skips the parse (documented)"
        );

        // concurrent: Rust publishers racing the gate readers — the list
        // is already full here, so the publishers all drop early (no gen
        // churn) and every reader code must be 0 (valid labels: 0 in the
        // room state via accept, 0 in the full state via the cap).
        let stop = Arc::new(AtomicBool::new(false));
        let mut handles = Vec::new();
        for t in 0..2u32 {
            let stop = Arc::clone(&stop);
            handles.push(std::thread::spawn(move || {
                let mut i = 0u64;
                while !stop.load(Ordering::Relaxed) {
                    let n = format!("c.gate.race.{t}.{i}");
                    telemetry::publish_metric(&n, i as f64, None, None);
                    i += 1;
                }
            }));
        }
        for _ in 0..2000u32 {
            let rc =
                unsafe { n_telemetry_publish_metric(nm.as_ptr(), 1.0, std::ptr::null(), lb.as_ptr()) };
            assert_eq!(rc, 0, "gate reader code must be 0 in every state");
        }
        stop.store(true, Ordering::Relaxed);
        for h in handles {
            h.join().expect("publisher thread must not panic");
        }

        // reset clears the flag -> room contract again.
        telemetry::test_reset_snapshot();
        assert!(!telemetry::metrics_full(), "reset must clear the flag");
        assert_eq!(
            unsafe { n_telemetry_publish_metric(nm.as_ptr(), 1.0, std::ptr::null(), lb.as_ptr()) },
            0,
            "room again: accepted"
        );
        assert_eq!(
            unsafe { n_telemetry_publish_metric(nm.as_ptr(), 1.0, std::ptr::null(), bad.as_ptr()) },
            -2,
            "room again: malformed rejected"
        );
        telemetry::test_reset_snapshot(); // leave a clean room state
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

    /// TASK-203 A/B subject: the C events publish entry's gate-vs-cstr
    /// ordering. Arm A = the verbatim pre-203 shape replicated locally
    /// (cstr -> empty-check -> gate -> parse -> publish); arm B = the
    /// REAL entry (C_ENTRY_PREGATE_FIRST = true: gate -> cstr ->
    /// empty-check -> parse -> publish). Line 1 is the zero-subscriber
    /// steady state (the per-tick default shape; requires a fresh global
    /// bus — hence a filtered SOLO run, gens == 0; the "gfirst" filter
    /// matches only this bench). NAMED to sort after bench_c_events_
    /// publish: this bench's subscribe/unsubscribe cycle leaves the
    /// global gens nonzero forever, and bench_c_events_publish's no-subs
    /// line is the one ledger line that needs a never-subscribed bus —
    /// this bench must never run before it in a shared sweep. Line 2 is
    /// the one-sync-subscriber control: both arms parse and dispatch
    /// there, so the pair must match within noise — the reorder must not
    /// tax the with-subscriber path.
    #[test]
    #[ignore]
    fn bench_c_publish_gfirst_ab() {
        // Arm A: the pre-TASK-203 entry body, verbatim.
        #[inline(never)]
        unsafe fn arm_a_pre203(event: *const c_char, payload_json: *const c_char) -> usize {
            let event = cstr(event);
            if event.is_empty() {
                return 0;
            }
            let bus = events::global_ref();
            if C_ENTRY_PREGATE && !bus.may_have_subscribers() {
                return 0;
            }
            let payload: Value = if payload_json.is_null() {
                Value::Null
            } else {
                c_publish_parse(cstr(payload_json))
            };
            bus.publish(event, &payload)
        }

        let ev = CString::new("c.bench.gfirst").unwrap();
        let pj = CString::new("{\"tick\":1,\"drained\":0}").unwrap();
        let iters = 200_000u32;
        let rounds = 5;

        // line 1: zero subscribers on the global bus (fresh in a solo run)
        let mut lines = [(f64::MAX, f64::MAX); 2]; // (arm A, arm B) per line
        for _ in 0..10_000u32 {
            black_box(unsafe { arm_a_pre203(ev.as_ptr(), pj.as_ptr()) });
            black_box(unsafe { n_events_publish(ev.as_ptr(), pj.as_ptr()) });
        }
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                black_box(unsafe { arm_a_pre203(ev.as_ptr(), pj.as_ptr()) });
            }
            lines[0].0 = lines[0].0.min(start.elapsed().as_secs_f64() / f64::from(iters));
            let start = Instant::now();
            for _ in 0..iters {
                black_box(unsafe { n_events_publish(ev.as_ptr(), pj.as_ptr()) });
            }
            lines[0].1 = lines[0].1.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }

        // line 2: one sync subscriber — the parse + dispatch control
        let sub = events::global().subscribe("c.bench.gfirst", Arc::new(|_, _| {}));
        for _ in 0..10_000u32 {
            black_box(unsafe { arm_a_pre203(ev.as_ptr(), pj.as_ptr()) });
            black_box(unsafe { n_events_publish(ev.as_ptr(), pj.as_ptr()) });
        }
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                black_box(unsafe { arm_a_pre203(ev.as_ptr(), pj.as_ptr()) });
            }
            lines[1].0 = lines[1].0.min(start.elapsed().as_secs_f64() / f64::from(iters));
            let start = Instant::now();
            for _ in 0..iters {
                black_box(unsafe { n_events_publish(ev.as_ptr(), pj.as_ptr()) });
            }
            lines[1].1 = lines[1].1.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }
        let _ = events::global().unsubscribe("c.bench.gfirst", &sub);

        println!(
            "BENCH c_publish_gfirst_ab: no-subs pre-203 {:.0} vs gate-first {:.0} ns/op; 1-sync-sub pre-203 {:.0} vs gate-first {:.0} ns/op (min of {rounds}x{iters})",
            lines[0].0 * 1e9,
            lines[0].1 * 1e9,
            lines[1].0 * 1e9,
            lines[1].1 * 1e9,
        );
    }

    /// TASK-204 decomposition: where do the ~147 ns of
    /// c_publish(1 sync sub) go? The parse layer is TASK-193's floor
    /// (bench_c_publish_parse_ab: ~140, representation-locked — the
    /// output Value MUST be serde_json's Map, so the key Strings and the
    /// BTree build/drop are semantic parity costs); the dispatch layer
    /// rides the Rust publish line (~9: gens load + fnv + memo probes +
    /// guard-check + catch_unwind + handler call). The C framing between
    /// them (the gate load + TWO cstr walks — event and payload) has
    /// never been isolated. Iso lines, all in ONE run for comparability:
    ///   framing      — the post-203 fall-through shape up to but
    ///                  excluding the parse (gate + cstr(event) +
    ///                  empty-check), one sync subscriber present (the
    ///                  gate must pass);
    ///   payload_cstr — the second cstr walk (the payload string) that
    ///                  bench_c_publish_parse_ab excludes (it receives a
    ///                  ready &str);
    ///   parse        — c_publish_parse on the payload &str (the
    ///                  TASK-193 shape, Value drop inside the loop);
    ///   dispatch     — bus.publish(event, &payload) on the memo-hit
    ///                  path, payload PRE-BUILT outside the loop (pure
    ///                  delivery cost, no per-iter Value build/drop);
    ///   e2e          — the real entry (the sum check).
    /// If framing + payload_cstr + parse + dispatch closes on e2e within
    /// measurement overlap, the line's true zero-semantic-change floor is
    /// BANKED with numbers: every remaining component is either
    /// representation-locked (the Map), contract-locked (the cstr walks
    /// feeding utf8-checked &str slicing + the empty-check), or
    /// floor-class (the memo-hit dispatch). Context-independent: every
    /// line is with-subscriber or bus-free (no line needs a
    /// never-subscribed bus; the bench's own subscribe cycle bumps gens
    /// exactly like every other global-bus bench).
    #[test]
    #[ignore]
    fn bench_c_publish_delivery_iso() {
        // The post-203 fall-through shape up to (excluding) the parse.
        #[inline(never)]
        unsafe fn iso_framing(event: *const c_char) -> usize {
            let bus = events::global_ref();
            if !bus.may_have_subscribers() {
                return 0;
            }
            let event = cstr(event);
            if event.is_empty() {
                return 0;
            }
            black_box(event);
            0
        }
        // The payload cstr walk (strlen + UTF-8) the parse bench excludes.
        #[inline(never)]
        unsafe fn iso_payload_cstr(payload_json: *const c_char) -> usize {
            let p = cstr(payload_json);
            black_box(p);
            p.len()
        }

        let ev = CString::new("c.bench.delivery").unwrap();
        let pj = CString::new("{\"tick\":1,\"drained\":0}").unwrap();
        let ev_str = ev.to_str().unwrap();
        let pj_str = pj.to_str().unwrap();
        let iters = 200_000u32;
        let rounds = 5;

        let sub = events::global().subscribe("c.bench.delivery", Arc::new(|_, _| {}));
        let bus = events::global_ref();
        let payload = c_publish_parse(pj_str); // pre-built for the dispatch iso

        // warm-up: memo fill + code paths (10k each)
        for _ in 0..10_000u32 {
            black_box(unsafe { iso_framing(ev.as_ptr()) });
            black_box(unsafe { iso_payload_cstr(pj.as_ptr()) });
            black_box(c_publish_parse(pj_str));
            black_box(bus.publish(ev_str, &payload));
            black_box(unsafe { n_events_publish(ev.as_ptr(), pj.as_ptr()) });
        }

        let mut best = [f64::MAX; 5]; // framing, payload_cstr, parse, dispatch, e2e
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                black_box(unsafe { iso_framing(ev.as_ptr()) });
            }
            best[0] = best[0].min(start.elapsed().as_secs_f64() / f64::from(iters));

            let start = Instant::now();
            for _ in 0..iters {
                black_box(unsafe { iso_payload_cstr(pj.as_ptr()) });
            }
            best[1] = best[1].min(start.elapsed().as_secs_f64() / f64::from(iters));

            let start = Instant::now();
            for _ in 0..iters {
                black_box(c_publish_parse(pj_str));
            }
            best[2] = best[2].min(start.elapsed().as_secs_f64() / f64::from(iters));

            let start = Instant::now();
            for _ in 0..iters {
                black_box(bus.publish(ev_str, &payload));
            }
            best[3] = best[3].min(start.elapsed().as_secs_f64() / f64::from(iters));

            let start = Instant::now();
            for _ in 0..iters {
                black_box(unsafe { n_events_publish(ev.as_ptr(), pj.as_ptr()) });
            }
            best[4] = best[4].min(start.elapsed().as_secs_f64() / f64::from(iters));
        }
        let _ = events::global().unsubscribe("c.bench.delivery", &sub);

        println!(
            "BENCH c_publish_delivery_iso: framing {:.0} + payload_cstr {:.0} + parse {:.0} + dispatch {:.0} = {:.0} vs e2e {:.0} ns/op (min of {rounds}x{iters})",
            best[0] * 1e9,
            best[1] * 1e9,
            best[2] * 1e9,
            best[3] * 1e9,
            (best[0] + best[1] + best[2] + best[3]) * 1e9,
            best[4] * 1e9,
        );
    }

    /// TASK-203 in-suite contract: the gate-first ordering preserves the
    /// C publish entry's observable contract in every state reachable
    /// from a parallel suite run. State-INDEPENDENT lines (no assumption
    /// about other tests' subscriptions on the global bus): a NULL event
    /// and an empty event return 0 with AND without subscribers (behind
    /// the firing gate the return is 0; on the fall-through path the
    /// empty-check returns 0). The state-DEPENDENT lines use a UNIQUE
    /// event name owned by this test: one sync subscriber -> 1, NULL
    /// payload -> 1, malformed payload -> 1 (the TASK-193 fallback
    /// parses to Value::Null and dispatch still runs), an empty event
    /// name while subscribed -> 0, and after unsubscribe -> 0 (the gens
    /// bump invalidates the dispatch memo). The empty-name-with-subs
    /// line is the ordering-critical one: the gate must FALL THROUGH to
    /// the empty-check when subscribers exist.
    #[test]
    fn c_events_pregate_first_contract() {
        let _guard = TEST_LOCK.lock().unwrap();

        let ev = CString::new("c.bench.gfirst.contract").unwrap();
        let pj = CString::new("{\"tick\":1,\"drained\":0}").unwrap();

        // state-independent: empty / NULL events are 0 in any bus state
        assert_eq!(unsafe { n_events_publish(std::ptr::null(), pj.as_ptr()) }, 0);
        assert_eq!(unsafe { n_events_publish(ev.as_ptr(), pj.as_ptr()) }, 0);

        // state-dependent, unique name: the full dispatch path
        let sub = events::global().subscribe("c.bench.gfirst.contract", Arc::new(|_, _| {}));
        assert_eq!(
            unsafe { n_events_publish(ev.as_ptr(), pj.as_ptr()) },
            1,
            "one sync subscriber must be invoked"
        );
        assert_eq!(
            unsafe { n_events_publish(ev.as_ptr(), std::ptr::null()) },
            1,
            "NULL payload publishes as Value::Null"
        );
        let bad = CString::new("{not json").unwrap();
        assert_eq!(
            unsafe { n_events_publish(ev.as_ptr(), bad.as_ptr()) },
            1,
            "malformed payload falls back to Null and dispatches"
        );
        assert_eq!(
            unsafe { n_events_publish(std::ptr::null(), pj.as_ptr()) },
            0,
            "NULL event stays 0 while subscribers exist"
        );
        let empty = CString::new("").unwrap();
        assert_eq!(
            unsafe { n_events_publish(empty.as_ptr(), pj.as_ptr()) },
            0,
            "empty event name stays 0 while subscribers exist (the gate must fall through to the empty-check)"
        );
        assert!(
            events::global().unsubscribe("c.bench.gfirst.contract", &sub),
            "unsubscribe must remove the test's own subscription"
        );
        assert_eq!(
            unsafe { n_events_publish(ev.as_ptr(), pj.as_ptr()) },
            0,
            "after unsubscribe the unique name dispatches nothing"
        );
    }

    /// TASK-193 parity corpus: for every input, the C-entry parse must
    /// produce EXACTLY the Value the pre-TASK-193 shape produced
    /// (serde_json::from_str(..).unwrap_or(Value::Null)). Fast-path
    /// acceptances are checked for equality with serde; everything the
    /// scanner declines routes to serde itself, so equality is structural.
    #[test]
    fn c_publish_parse_parity_corpus() {
        // fast-path subset: must equal serde AND exercise the scanner
        let fast_subset = [
            "null",
            "true",
            "false",
            "0",
            "42",
            "-7",
            "18446744073709551615",  // u64::MAX
            "9223372036854775808",   // i64::MAX+1 -> u64 range
            "-9223372036854775808",  // i64::MIN
            "\"hello\"",
            "\"\"",
            "{}",
            "[]",
            "{\"tick\":1,\"drained\":0}",
            "{\"a\":\"x\",\"b\":2,\"c\":null,\"d\":true}",
            "[1,2,3]",
            "[true,false,null]",
            "[\"a\",\"b\"]",
        ];
        for s in fast_subset {
            assert!(
                parse_payload_fast(s).is_some(),
                "scanner declined a subset shape: {s}"
            );
            assert_eq!(
                c_publish_parse(s),
                serde_json::from_str::<Value>(s).unwrap_or(Value::Null),
                "fast-path result diverged from serde for: {s}"
            );
        }
        // fallback shapes: scanner declines, serde decides — equality is
        // the contract (invalid inputs land on Null exactly as before)
        let fallback = [
            "",
            "1.5",
            "1e3",
            "-0",
            "01",
            "-01",
            "18446744073709551616",  // u64 overflow -> serde error -> Null
            "-9223372036854775809",  // i64 underflow -> serde error -> Null
            "\"a\\\"b\"",
            "\"a\\u0041b\"",
            "\"a\"b\"",
            "{\"a\":{\"b\":1}}",
            "[{\"a\":1}]",
            "[1,]",
            "{\"a\":1,}",
            "{a:1}",
            "  {\"a\":1}",
            "{\"a\":1} ",
            " {\"a\": 1}",
            "{\"a\": 1}",
            "{\"a\":1.5}",
            "nul",
            "nullx",
            "tru",
            "[1,2",
            "{\"a\"",
            "12x",
            "12 ",
            " 12",
        ];
        for s in fallback {
            let via_entry = c_publish_parse(s);
            let via_serde = serde_json::from_str::<Value>(s).unwrap_or(Value::Null);
            assert_eq!(
                via_entry, via_serde,
                "C-entry parse diverged from serde for: {s}"
            );
        }
    }

    /// TASK-193 A/B subject: payload parse cost, pre-shape (serde direct)
    /// vs fast-path+fallback. Line 1 is the BEFORE arm (the exact
    /// pre-TASK-193 expression), line 2 the AFTER arm; the end-to-end
    /// effect shows up in bench_c_events_publish line 2.
    #[test]
    #[ignore]
    fn bench_c_publish_parse_ab() {
        let payload = "{\"tick\":1,\"drained\":0}";
        assert!(
            parse_payload_fast(payload).is_some(),
            "fast path misses the bench shape — round would be refuted"
        );
        let iters = 200_000u32;
        let rounds = 5;

        let mut best_serde = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                black_box(serde_json::from_str::<Value>(payload).unwrap_or(Value::Null));
            }
            best_serde = best_serde.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }

        let mut best_fast = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                black_box(c_publish_parse(payload));
            }
            best_fast = best_fast.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }

        println!(
            "BENCH c_publish_parse: serde-direct {:.0} ns/op, fast-path+fallback {:.0} ns/op (min of {rounds}x{iters})",
            best_serde * 1e9,
            best_fast * 1e9
        );
    }

    /// TASK-211 decomposition: the c_publish parse term (fast path
    /// 142-148 ns in the delivery-iso split) is the LAST >100 ns hot-path
    /// term with no internal mechanism split — TASK-193 landed the
    /// scanner + the parity corpus but priced parse as ONE number. Arms
    /// (min of 5x200k; drop included wherever a Value/String is built,
    /// matching the e2e shape):
    ///   e2e_fast       — verbatim c_publish_parse(fixture): scan +
    ///                    construct + drop
    ///   scan_only      — fold-replica of the scanner's accept-path
    ///                    control flow (scan_flat_object + scan_leaf for
    ///                    the fixture's leaf class): walks the SAME bytes
    ///                    with the same bounds checks, folds accepted
    ///                    payload bytes into a u64 — NO Value built, NO
    ///                    allocation. The input goes through black_box
    ///                    per iteration so LLVM cannot constant-fold the
    ///                    walk (the TASK-209 phantom-term lesson). The
    ///                    replica prices the ACCEPT path only — the
    ///                    rejection subtleties (leading zeros, bare '-',
    ///                    non-integer leaves) are not replicated; they
    ///                    cost nothing on the accepted shape.
    ///   construct_only — verbatim fixture Value built directly
    ///                    (Map::new + two owned key Strings + two
    ///                    Numbers), dropped per iteration — construct +
    ///                    drop with zero scanning.
    ///   strings_only   — the two owned key Strings built+dropped per
    ///                    iteration: prices the key malloc/free pairs.
    ///   map_only       — Map::new + two Number inserts + Value wrap
    ///                    with the SAME empty key (no key mallocs; the
    ///                    second insert is the in-place value update, so
    ///                    the walk is two inserts and the ONLY malloc is
    ///                    the BTreeMap node): prices node alloc/free +
    ///                    insert machinery + wrap + drop. TASK-218: this
    ///                    arm is LEGACY-RECORD only — its same-key-replace
    ///                    shape is NOT the production path (two DISTINCT
    ///                    keys into a fresh map) and its number exceeds the
    ///                    whole construct it decomposes (131 vs 80), so
    ///                    the additive check strings+map cannot close; the
    ///                    arithmetic contradiction alone proves the arm
    ///                    misprices (the construct contains exactly ONE
    ///                    node malloc — a 131 ns node term cannot fit in an
    ///                    80 ns construct).
    ///   map_node       — TASK-218 production-shaped node arm: clone+drop
    ///                    of a prebuilt single-entry empty-key map per
    ///                    iteration = the node malloc/free + wrap/drop
    ///                    profile with ZERO key mallocs (an empty String
    ///                    clone does not allocate) and no insert machinery.
    ///   map_machinery  — TASK-218 production-shaped machinery arm: a
    ///                    prebuilt two-entry map (real keys tick/drained,
    ///                    asserted equal to the fast path's Value) hit with
    ///                    2x get_mut + value write per iteration = pure
    ///                    search/update machinery, zero allocs of any kind.
    ///   serde_direct   — the fallback arm, context line.
    /// Gates asserted BEFORE any timing: (1) PARITY — the fast path's
    /// Value equals serde's on the fixture; (2) replica agreement — the
    /// fold-replica folds exactly the alphanumeric bytes of the fixture
    /// (independent runtime sum, not a hand constant); (3) construct
    /// equality — construct_only's Value equals the fast path's; (4) the
    /// machinery map equals the fast path's Value. Sum checks printed:
    /// scan + construct vs e2e; strings + node + machinery vs construct
    /// (TASK-218 additive check; the legacy strings+map sum is dropped
    /// from the print as broken-by-shape).
    #[test]
    #[ignore]
    fn bench_c_publish_parse_stages_iso() {
        let payload = "{\"tick\":1,\"drained\":0}";

        // Walk-replica of the scanner's accept path: fold instead of
        // alloc. Bench-only (not production code): prices the fixture's
        // scan — key spans via the scan_string_at walk (b.get bounds
        // check per byte), integer leaves via the scan_leaf digit walk
        // with the same structural-terminator check.
        fn scan_fold_replica(s: &str) -> Option<u64> {
            let b = s.as_bytes();
            if b.len() < 2 || b[b.len() - 1] != b'}' {
                return None;
            }
            let mut fold = 0u64;
            let mut i = 1usize;
            if b[i] == b'}' {
                return Some(fold);
            }
            loop {
                if b[i] != b'"' {
                    return None;
                }
                let start = i + 1;
                let mut j = start;
                loop {
                    let c = *b.get(j)?;
                    if c == b'"' {
                        break;
                    }
                    if c == b'\\' || c < 0x20 {
                        return None;
                    }
                    fold = fold.wrapping_add(u64::from(c));
                    j += 1;
                }
                i = j + 1;
                if b.get(i) != Some(&b':') {
                    return None;
                }
                i += 1;
                match b[i] {
                    b'-' | b'0'..=b'9' => {
                        let mut j = i;
                        if b[j] == b'-' {
                            j += 1;
                        }
                        while j < b.len() && b[j].is_ascii_digit() {
                            fold = fold.wrapping_add(u64::from(b[j]));
                            j += 1;
                        }
                        match b.get(j) {
                            None | Some(b',') | Some(b'}') | Some(b']') => {}
                            _ => return None,
                        }
                        i = j;
                    }
                    _ => return None,
                }
                match b.get(i) {
                    Some(b',') => i += 1,
                    Some(b'}') => return Some(fold),
                    _ => return None,
                }
            }
        }

        // Construct replica: the exact fixture Value, no scanning.
        fn construct_fixture_value() -> Value {
            let mut map = serde_json::Map::new();
            map.insert(String::from("tick"), serde_json::Number::from(1u64).into());
            map.insert(
                String::from("drained"),
                serde_json::Number::from(0u64).into(),
            );
            Value::Object(map)
        }

        // Gate 1: parity (the TASK-193 contract on the bench shape).
        let fast = parse_payload_fast(payload).expect("fast path misses the bench shape");
        let via_serde = serde_json::from_str::<Value>(payload).unwrap_or(Value::Null);
        assert_eq!(fast, via_serde, "fast path diverged from serde on the bench shape");
        // Gate 2: the fold-replica folds exactly the alphanumeric bytes.
        let expected: u64 = payload
            .bytes()
            .filter(|b| b.is_ascii_alphanumeric())
            .map(u64::from)
            .sum();
        assert_eq!(
            scan_fold_replica(payload),
            Some(expected),
            "fold-replica does not agree with the independent byte sum"
        );
        // Gate 3: the construct replica equals the fast path's Value.
        assert_eq!(
            construct_fixture_value(),
            fast,
            "construct replica diverges from the fast path"
        );

        // TASK-218 arm fixtures. node_map: single entry, empty key — the
        // clone prices node malloc/free + wrap/drop with zero key mallocs.
        // mach_map: the production-shaped two-entry map; its equality with
        // the fast path's Value is gate 4.
        let node_map: serde_json::Map<String, Value> = {
            let mut m = serde_json::Map::new();
            m.insert(String::new(), serde_json::Number::from(1u64).into());
            m
        };
        let mut mach_map: serde_json::Map<String, Value> = {
            let mut m = serde_json::Map::new();
            m.insert(String::from("tick"), serde_json::Number::from(1u64).into());
            m.insert(String::from("drained"), serde_json::Number::from(0u64).into());
            m
        };
        // Gate 4: the machinery map equals the fast path's Value.
        assert_eq!(
            Value::Object(mach_map.clone()),
            fast,
            "machinery map diverges from the fast path"
        );

        let iters = 200_000u32;
        let rounds = 5;

        let mut best_e2e = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                black_box(c_publish_parse(payload));
            }
            best_e2e = best_e2e.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }

        let mut best_scan = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                black_box(scan_fold_replica(black_box(payload)));
            }
            best_scan = best_scan.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }

        let mut best_construct = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                black_box(construct_fixture_value());
            }
            best_construct = best_construct.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }

        let mut best_strings = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                black_box(String::from("tick"));
                black_box(String::from("drained"));
            }
            best_strings = best_strings.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }

        let mut best_map = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                let mut m = serde_json::Map::new();
                m.insert(String::new(), serde_json::Number::from(1u64).into());
                m.insert(String::new(), serde_json::Number::from(0u64).into());
                black_box(Value::Object(m));
            }
            best_map = best_map.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }

        let mut best_serde = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                black_box(serde_json::from_str::<Value>(payload).unwrap_or(Value::Null));
            }
            best_serde = best_serde.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }

        // TASK-218: node arm — clone+drop of the single-entry empty-key map.
        let mut best_node = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                black_box(node_map.clone());
            }
            best_node = best_node.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }

        // TASK-218: machinery arm — 2x get_mut + value write on the prebuilt
        // two-entry map (values rewritten to themselves: idempotent, zero
        // allocs, the map persists across iterations).
        let mut best_mach = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                if let Some(v) = mach_map.get_mut("tick") {
                    *v = serde_json::Number::from(1u64).into();
                }
                if let Some(v) = mach_map.get_mut("drained") {
                    *v = serde_json::Number::from(0u64).into();
                }
                black_box(mach_map.len());
            }
            best_mach = best_mach.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }

        println!(
            "BENCH c_publish_parse_stages: e2e_fast {:.0} = scan {:.0} + construct {:.0} (sum {:.0}); construct = strings {:.0} + node {:.0} + machinery {:.0} (sum {:.0}); legacy map-replace {:.0}; serde-direct {:.0} ns/op (min of {rounds}x{iters})",
            best_e2e * 1e9,
            best_scan * 1e9,
            best_construct * 1e9,
            (best_scan + best_construct) * 1e9,
            best_strings * 1e9,
            best_node * 1e9,
            best_mach * 1e9,
            (best_strings + best_node + best_mach) * 1e9,
            best_map * 1e9,
            best_serde * 1e9
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

    /// TASK-202 A/B subject: the C metric entry's pre-parse gate. Arm A =
    /// the verbatim pre-202 gate (cstr(name) + the fullness recomputed
    /// UNDER THE DATA LOCK via metrics_full_checked + return); arm B =
    /// the real entry (the fullness is one Relaxed flag load). Lines:
    /// labels4 (the parse payload is present but the gate fires before
    /// it in the full state) and NULL labels (control). The list is
    /// filled to the cap first — the production steady state (append-only,
    /// periodic publishers live here forever). Runs SOLO (filtered run
    /// only): resets the shared snapshot and fills the list to the cap;
    /// the parallel suite must never see that state — hence #[ignore].
    #[test]
    #[ignore]
    fn bench_cmetric_gate_ab() {
        telemetry::test_reset_snapshot();

        let nm = CString::new("c.bench.gate").unwrap();
        // fill to the cap (unique names, no labels)
        let mut i = 0u64;
        while !telemetry::metrics_full() {
            let n = CString::new(format!("c.gate.fill.{i}")).unwrap();
            telemetry::publish_metric(n.to_str().unwrap(), i as f64, None, None);
            i += 1;
            assert!(i < 10_000, "metric list never filled");
        }

        // Arm A: the pre-202 gate body, verbatim (data lock per call).
        let entry_pre202 = || -> i32 {
            let name = unsafe { cstr(nm.as_ptr()) };
            if name.is_empty() {
                return -1;
            }
            if TELEM_PREGATE && telemetry::metrics_full_checked() {
                return 0;
            }
            0
        };

        let lb4 = CString::new(
            "{\"region\":\"eu-central-1\",\"world\":\"overworld\",\"dim\":\"nether\",\"tier\":2}",
        )
        .unwrap();
        let iters = 200_000u32;
        let rounds = 5;

        // ---- line 1: full + labeled ----
        for _ in 0..10_000u32 {
            black_box(entry_pre202());
            black_box(unsafe {
                n_telemetry_publish_metric(nm.as_ptr(), 1.0, std::ptr::null(), lb4.as_ptr())
            });
        }
        let mut best_labeled_a = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                black_box(entry_pre202());
            }
            best_labeled_a = best_labeled_a.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }
        let mut best_labeled_b = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                black_box(unsafe {
                    n_telemetry_publish_metric(nm.as_ptr(), 1.0, std::ptr::null(), lb4.as_ptr())
                });
            }
            best_labeled_b = best_labeled_b.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }

        // ---- line 2: full + NULL labels (control) ----
        for _ in 0..10_000u32 {
            black_box(entry_pre202());
            black_box(unsafe {
                n_telemetry_publish_metric(nm.as_ptr(), 1.0, std::ptr::null(), std::ptr::null())
            });
        }
        let mut best_null_a = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                black_box(entry_pre202());
            }
            best_null_a = best_null_a.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }
        let mut best_null_b = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                black_box(unsafe {
                    n_telemetry_publish_metric(nm.as_ptr(), 1.0, std::ptr::null(), std::ptr::null())
                });
            }
            best_null_b = best_null_b.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }

        println!(
            "BENCH cmetric gate A/B: lock-gate labels4 {:.0} -> flag-gate {:.0} ns/op, lock-gate null {:.0} -> flag-gate {:.0} ns/op (min of {rounds}x{iters})",
            best_labeled_a * 1e9,
            best_labeled_b * 1e9,
            best_null_a * 1e9,
            best_null_b * 1e9
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
        assert!(v2["server_name"].is_string(), "server_name always present on call 2");
        assert!(v2["started_at"].is_u64(), "started_at always present on call 2");
        assert!(v2["tps"].is_number(), "tps always present on call 2");
        assert!(v2["metrics"].is_array(), "metrics always serialized on call 2");
        // NOTE (TASK-201): cross-call FIELD EQUALITY is deliberately not
        // asserted here. The doc above allows concurrent mutators, and the
        // per-module test locks do not serialize across modules — telemetry
        // tests legitimately mutate server_name / started_at (setters test,
        // reset_state) under the telemetry TEST_LOCK while this test holds
        // the c_bridge one; the contract is that EVERY call serves a
        // complete snapshot of the CURRENT state. The equality form was a
        // pre-existing flake under the filtered `snapshot` cluster (2/5
        // runs on unmodified HEAD, same assert).
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

    /// TASK-200 A/B subject: the C snapshot entry's buffer locking.
    /// Arm A = the verbatim pre-200 shape (SNAP_BUF OnceLock<Mutex> load
    /// + mutex pair around clear + snapshot_json_write + NUL push); arm
    /// B = telemetry::snapshot_c_entry (the whole cycle under the ONE
    /// cache lock — the buffer lives in the cache struct, no second lock
    /// acquisition, no OnceLock hop). Lines: empty metric list (control)
    /// and 32 labeled metrics. Byte-identity between the shapes is
    /// asserted by snapshot_c_entry_single_lock_parity (telemetry) and
    /// the solo telemetry_snapshot_inplace_byte_identical (which runs on
    /// the default arm-B entry). Runs SOLO (filtered run only): resets
    /// the shared snapshot and leaves 32 metrics published at the end
    /// (harmless — every solo bench resets its own starting state).
    #[test]
    #[ignore]
    fn bench_snapshot_centry_ab() {
        telemetry::test_reset_snapshot();

        let iters = 200_000u32;
        let rounds = 5;

        // Arm A: the pre-200 entry body, verbatim (the TELEM_SNAP_ONELOCK
        // = false arm re-derives exactly this shape).
        let entry_legacy = || -> *const c_char {
            let buf = SNAP_BUF.get_or_init(|| Mutex::new(Vec::with_capacity(4096)));
            let mut guard = buf.lock().unwrap();
            guard.clear();
            telemetry::snapshot_json_write(guard.as_mut());
            guard.push(0);
            guard.as_ptr() as *const c_char
        };

        // ---- line 1: empty metric list (control) ----
        for _ in 0..10_000u32 {
            black_box(entry_legacy());
            black_box(telemetry::snapshot_c_entry());
        }
        let mut best_empty_a = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                black_box(entry_legacy());
            }
            best_empty_a = best_empty_a.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }
        let mut best_empty_b = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                black_box(telemetry::snapshot_c_entry());
            }
            best_empty_b = best_empty_b.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }

        // ---- 32 labeled metrics (4 label pairs each) ----
        for i in 0..32u32 {
            let mut labels = HashMap::new();
            labels.insert("region".to_string(), "eu".to_string());
            labels.insert("host".to_string(), format!("host-{i}"));
            labels.insert("module".to_string(), format!("mod-{i}"));
            labels.insert("idx".to_string(), i.to_string());
            telemetry::publish_metric(
                &format!("c.bench.centry.{i}"),
                i as f64,
                Some("ms"),
                Some(labels),
            );
        }
        for _ in 0..10_000u32 {
            black_box(entry_legacy());
            black_box(telemetry::snapshot_c_entry());
        }
        let mut best_32_a = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                black_box(entry_legacy());
            }
            best_32_a = best_32_a.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }
        let mut best_32_b = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                black_box(telemetry::snapshot_c_entry());
            }
            best_32_b = best_32_b.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }

        println!(
            "BENCH centry A/B: mutex empty {:.0} ns/op -> single-lock empty {:.0} ns/op, mutex 32-labeled {:.0} ns/op -> single-lock 32-labeled {:.0} ns/op (min of {rounds}x{iters})",
            best_empty_a * 1e9,
            best_empty_b * 1e9,
            best_32_a * 1e9,
            best_32_b * 1e9
        );
    }
}