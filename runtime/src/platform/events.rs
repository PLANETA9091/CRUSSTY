//! Brick 6: module event bus — decoupled communication between modules and
//! with the platform itself.
//!
//! Events are stringly-typed payloads (serde_json::Value) to keep the ABI
//! stable while modules evolve independently. The platform emits lifecycle
//! events (class loaded, plugin loaded, tick boundary); modules publish and
//! subscribe freely.
//!
//! # Ordering
//!
//! * Sync handlers run on the publisher's thread, in subscription order, for
//!   exact-name matches first and pattern matches afterwards (patterns in the
//!   order their subscriptions were created).
//! * Async handlers run on a lazily-spawned dispatcher pool (2 threads) and
//!   may interleave with sync handlers. Handlers for a *single publish* of
//!   the same event still run in subscription order (one queued task per publish),
//!   but distinct publishes may be reordered across pool workers. Async
//!   delivery is fire-and-forget: `publish` never blocks on it.
//! * `publish` returns the number of sync handlers invoked; async dispatch is
//!   observed through [`EventBus::async_pending`] / [`EventBus::async_dropped`].
//!
//! # Backpressure
//!
//! The async dispatch queue is bounded ([`ASYNC_QUEUE_CAP`] = 4096 pending
//! tasks). The publisher must never block, so when the queue is full the
//! *oldest* pending event is dropped (load-shedding, the same drop-oldest
//! semantics as `tokio::sync::broadcast`'s `Lagged` slow-consumer handling),
//! never unbounded memory. A single `eprintln!` is emitted per drop burst.
//!
//! # Topic filter grammar
//!
//! Subscription names are dot-separated topic segments, following AMQP
//! topic-exchange routing-key semantics:
//!
//! * `a.b.c` — exact match on all segments.
//! * `platform.*` — `*` matches exactly one segment (any characters):
//!   matches `platform.save_complete`, not `platform.a.b` and not
//!   `platform`. Segment counts must be equal; there is no partial or
//!   trailing matching.
//! * `*` — the bare universal wildcard: matches every event (the event name
//!   is passed to the handler).
//!
//! # Error isolation
//!
//! Every handler invocation (sync and async) is wrapped in
//! `catch_unwind`; a panicking handler is logged and never propagates to
//! the publisher or kills a pool worker. A sync handler may publish again —
//! the registry lock is only held for mutation, never during invocation.
//!
//! # Hot path (TASK-164, TASK-167)
//!
//! `publish` / `has_subscribers` never take a lock. Each registry keeps an
//! immutable [`RegistryView`] published through an RCU [`rcu::ArcCell`];
//! resolved handler lists are cached in a process-global lock-free memo:
//! one acquire pointer load per probe, no TLS (measured 1.7-4.4ns per
//! thread_local access in this cdylib — TASK-166), no refcounts and no
//! allocation on hits. Records are immutable once published and never
//! freed, which is what makes the raw-pointer read sound without any
//! reader protocol.
//!
//! # Lifecycle events
//!
//! The bus publishes [`lifecycle::EVENT_SUBSCRIBED`] /
//! [`lifecycle::EVENT_UNSUBSCRIBED`] on itself. A re-entrancy guard prevents
//! infinite recursion: while a lifecycle event is being emitted, nested
//! subscribe/unsubscribe calls are applied but do not re-emit.
//!
//! # Design sources
//!
//! Bounded queues + drop-oldest load shedding (rustz2h "Backpressure in
//! Rust", rustfaq.org channel guides, Microsoft Rust Patterns book §5 —
//! "Always use bounded channels"), AMQP topic-exchange `*` segment matching
//! (LavinMQ topic-exchange rewrite, RabbitMQ docs), `catch_unwind` +
//! `AssertUnwindSafe` panic isolation for worker pools (std docs, Stanza
//! Concurrent Rust §thread-panics), generation counters for stale-handle
//! invalidation (slab/arena pattern), RCU snapshots (see [`rcu`]).

use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, OnceLock};

pub type Handler = Arc<dyn Fn(&str, &Value) + Send + Sync>;

/// Number of dispatcher threads draining the async queue.
const ASYNC_WORKERS: usize = 2;
/// Hard cap on pending async tasks; overflow drops the oldest (load shed).
pub const ASYNC_QUEUE_CAP: usize = 4096;
/// TASK-171 spin-then-park: how many `try_lock`+`spin_loop` probes a worker
/// makes before committing to the condvar park. Each probe is one CAS pair
/// (~30ns), so the whole window is a few microseconds: a publish that lands
/// inside it is absorbed without the structural ~0.3-3.5us futex wake
/// syscall. A truly idle pool pays the window once per idle episode, then
/// parks with zero ongoing CPU cost.
const ASYNC_SPIN_ITERS: usize = 1024;

/// A subscription handle. Tokens are only valid on the bus that issued them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Subscription {
    pub id: u64,
    gen: u64,
}

struct Entry {
    id: u64,
    gen: u64,
    /// `Some((module id, library generation))` when the subscription was
    /// created inside a module handshake: a hot reload purges exactly the
    /// replaced generation's subscriptions (see [`EventBus::purge_owner`]).
    owner: Option<(String, u64)>,
    handler: Handler,
}

/// One topic's subscriber list. `gen` is a per-list generation counter that
/// stamps every entry, so a token can never remove an entry it did not
/// create, even if the list was rebuilt (generation-counter invalidation).
#[derive(Default)]
struct HandlerList {
    gen: u64,
    entries: Vec<Entry>,
}

impl HandlerList {
    fn add(&mut self, handler: Handler, id: u64, owner: Option<(String, u64)>) -> Subscription {
        let gen = self.gen;
        self.entries.push(Entry { id, gen, owner, handler });
        self.gen += 1;
        Subscription { id, gen }
    }

    /// Drop every entry owned by `owner`; returns how many were removed.
    fn purge_owner(&mut self, owner: &(String, u64)) -> usize {
        let before = self.entries.len();
        self.entries
            .retain(|e| e.owner.as_ref() != Some(owner));
        before - self.entries.len()
    }

    fn remove(&mut self, token: &Subscription) -> bool {
        if let Some(pos) = self
            .entries
            .iter()
            .position(|e| e.id == token.id && e.gen == token.gen)
        {
            self.entries.remove(pos);
            true
        } else {
            false
        }
    }
}

/// Exact-name subscriptions plus glob-pattern subscriptions, in creation
/// order (used for both the sync and the async registries). Writer-side
/// source of truth — reads go through the RCU [`RegistryView`].
#[derive(Default)]
struct Registry {
    exact: HashMap<String, HandlerList>,
    patterns: Vec<(String, HandlerList)>,
}

impl Registry {
    fn insert(
        &mut self,
        event: &str,
        handler: Handler,
        id: u64,
        owner: Option<(String, u64)>,
    ) -> Subscription {
        let pos = self.patterns.iter().position(|(p, _)| p == event);
        let list = if let Some(pos) = pos {
            &mut self.patterns[pos].1
        } else if has_glob(event) {
            self.patterns
                .push((event.to_string(), HandlerList::default()));
            &mut self.patterns.last_mut().expect("just pushed").1
        } else {
            self.exact.entry(event.to_string()).or_default()
        };
        list.add(handler, id, owner)
    }

    /// Drop every entry owned by `owner` (exact and pattern lists alike).
    /// Empty lists are removed so existence checks report the truth.
    fn purge_owner(&mut self, owner: &(String, u64)) -> usize {
        let mut removed = 0usize;
        for list in self.exact.values_mut() {
            removed += list.purge_owner(owner);
        }
        self.exact.retain(|_, list| !list.entries.is_empty());
        for (_, list) in self.patterns.iter_mut() {
            removed += list.purge_owner(owner);
        }
        self.patterns.retain(|(_, list)| !list.entries.is_empty());
        removed
    }

    fn remove(&mut self, event: &str, token: &Subscription) -> bool {
        if let Some(list) = self.exact.get_mut(event) {
            if list.remove(token) {
                // Drop the now-empty entry so existence checks (e.g.
                // has_subscribers) report the truth.
                if list.entries.is_empty() {
                    self.exact.remove(event);
                }
                return true;
            }
        }
        let mut removed = false;
        self.patterns.retain_mut(|(p, list)| {
            if p != event || !list.remove(token) {
                return true;
            }
            removed = true;
            !list.entries.is_empty()
        });
        removed
    }
}

/// Topic filter matching — see the grammar in the module docs.
/// Zero-alloc (TASK-161): segment-by-segment iterator compare — the previous
/// version allocated two `Vec<&str>` per call, and this runs on hot publish /
/// has_subscribers paths.
fn glob_match(pattern: &str, event: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    let mut p = pattern.split('.');
    let mut e = event.split('.');
    loop {
        match (p.next(), e.next()) {
            (None, None) => return true,
            (Some(ps), Some(es)) => {
                if ps != "*" && ps != es {
                    return false;
                }
            }
            _ => return false,
        }
    }
}

fn has_glob(pattern: &str) -> bool {
    pattern.contains('*')
}

/// FNV-1a 64-bit — event-name hash for the flat exact map and the per-thread
/// memo key (shared chunked implementation in [`rcu`]).
use super::rcu::fnv1a;

/// Resolved dispatch list for one event name: exact matches first, then
/// pattern matches in insertion order — precomputed at mutation time so the
/// hot publish path never allocates or clones per subscriber.
type Resolved = Arc<[(Option<(Box<str>, u64)>, Handler)]>;

/// One exact topic in the view's open-addressed flat map (insert-only; the
/// whole view is rebuilt on every registry mutation).
struct ExactSlot {
    hash: u64,
    name: Box<str>,
    list: Resolved,
}

struct PatternSlot {
    pattern: Box<str>,
    list: Resolved,
}

/// Immutable compiled view of a [`Registry`] (TASK-164): exact topics in an
/// open-addressed flat map keyed by FNV-1a hash, patterns with their resolved
/// handler lists. Published through an RCU cell; readers never lock.
struct RegistryView {
    exact: Vec<Option<ExactSlot>>,
    mask: usize,
    patterns: Vec<PatternSlot>,
}

impl RegistryView {
    fn from_registry(reg: &Registry) -> Self {
        let cap = (reg.exact.len() * 2 + 2).next_power_of_two();
        let mut exact: Vec<Option<ExactSlot>> = (0..cap).map(|_| None).collect();
        for (name, list) in &reg.exact {
            let resolved: Resolved = Arc::from(
                list.entries
                    .iter()
                    .map(|e| {
                        (
                            e.owner
                                .clone()
                                .map(|(id, gen)| (id.into_boxed_str(), gen)),
                            Arc::clone(&e.handler),
                        )
                    })
                    .collect::<Vec<_>>(),
            );
            let hash = fnv1a(name.as_bytes());
            let mut i = (hash as usize) & (cap - 1);
            while exact[i].is_some() {
                i = (i + 1) & (cap - 1);
            }
            exact[i] = Some(ExactSlot { hash, name: name.clone().into_boxed_str(), list: resolved });
        }
        let patterns = reg
            .patterns
            .iter()
            .map(|(p, list)| PatternSlot {
                pattern: p.clone().into_boxed_str(),
                list: Arc::from(
                    list.entries
                        .iter()
                        .map(|e| {
                            (
                                e.owner
                                    .clone()
                                    .map(|(id, gen)| (id.into_boxed_str(), gen)),
                                Arc::clone(&e.handler),
                            )
                        })
                        .collect::<Vec<_>>(),
                ),
            })
            .collect();
        RegistryView { exact, mask: cap - 1, patterns }
    }

    fn exact_find(&self, hash: u64, name: &str) -> Option<&Resolved> {
        let mut i = (hash as usize) & self.mask;
        loop {
            match &self.exact[i] {
                None => return None,
                Some(slot) => {
                    if slot.hash == hash && &*slot.name == name {
                        return Some(&slot.list);
                    }
                    i = (i + 1) & self.mask;
                }
            }
        }
    }

    fn any_match(&self, hash: u64, event: &str) -> bool {
        if self.exact_find(hash, event).is_some() {
            return true;
        }
        self.patterns.iter().any(|p| glob_match(&p.pattern, event))
    }

    /// Exact matches first, then patterns in insertion order — the dispatch
    /// order guarantee.
    fn resolve(&self, hash: u64, event: &str) -> Resolved {
        let mut out: Vec<(Option<(Box<str>, u64)>, Handler)> = Vec::new();
        if let Some(list) = self.exact_find(hash, event) {
            out.extend(list.iter().cloned());
        }
        for p in &self.patterns {
            if glob_match(&p.pattern, event) {
                out.extend(p.list.iter().cloned());
            }
        }
        Arc::from(out)
    }
}

/// Writer-side registry + RCU-published immutable view (TASK-164). Every
/// mutation happens under `reg`'s mutex and republishes a freshly compiled
/// view (generation bump included).
struct RegistryCell {
    reg: Mutex<Registry>,
    view: super::rcu::ArcCell<RegistryView>,
    /// Per-cell identity for thread-local memo slots.
    id: u64,
    /// Bus-wide combined generation counter: sync mutations bump the high
    /// word, async mutations the low word (TASK-164: the hot gate reads ONE
    /// atomic for both registries).
    shared: Arc<AtomicU64>,
    shift: u32,
}

static NEXT_CELL_ID: AtomicU64 = AtomicU64::new(1);

impl RegistryCell {
    fn new(shared: Arc<AtomicU64>, shift: u32) -> Self {
        Self {
            reg: Mutex::new(Registry::default()),
            view: super::rcu::ArcCell::new(),
            id: NEXT_CELL_ID.fetch_add(1, Ordering::Relaxed),
            shared,
            shift,
        }
    }

    /// Apply `f` to the registry; when it reports a change, compile and
    /// publish a new view, then bump the bus-wide combined generation (the
    /// memo invalidation point — a publish that loads the new generation
    /// will resolve against the new view).
    fn mutate<T>(&self, f: impl FnOnce(&mut Registry) -> (bool, T)) -> T {
        let mut reg = self.reg.lock().unwrap_or_else(|p| p.into_inner());
        let (changed, out) = f(&mut reg);
        if changed {
            let view = Arc::new(RegistryView::from_registry(&reg));
            self.view.store(view);
            self.shared.fetch_add(1 << self.shift, Ordering::Release);
        }
        out
    }
}

/// Process-global lock-free resolved-event memo (TASK-167). Replaces the
/// per-thread TLS memo: the measured TLS law (TASK-166) prices each
/// thread_local access at 1.7-4.4ns in this cdylib (global-dynamic model),
/// which was the last ~4ns of `publish(1 sync)`. The global table reads
/// with ONE acquire pointer load per probe — records are immutable once
/// published and never freed (retired records move to a process-lifetime
/// retire list), so readers need no refcounts, no reader windows, no
/// epochs: zero RMW on the hot path.
///
/// Keying: the slot index comes from the event-name hash; each record
/// stores a 64-bit tag = fnv1a(bus_id || combined gens) plus the event
/// name, both verified on hit — the hash only ever proposes an index. A
/// registry mutation changes the combined generation, hence the tag: a
/// stale record can never verify (staleness is impossible by construction,
/// unlike the TLS put-back protocol). Bus identity is inside the tag, so
/// records from different bus instances coexist (the per-engine-id lesson,
/// TASK-165).
///
/// Memory: inserts claim the first empty slot they meet (CAS) and stop
/// there, so a probe chain never continues past an empty slot; a saturated
/// region overwrites a probed victim. Overwritten records are retired,
/// never freed — one ~100B record per (registry mutation, event) pair that
/// re-resolves; module reloads are human-scale events, so the retire list
/// stays churn-proportional and tiny in practice.
struct MemoRecord {
    tag: u64,
    name: Box<str>,
    resolved: Resolved,
    has_async: bool,
}

const MEMO_SLOTS: usize = 4096;
const MEMO_MASK: usize = MEMO_SLOTS - 1;
const MEMO_PROBES: usize = 8;

static MEMO_TABLE: [AtomicPtr<MemoRecord>; MEMO_SLOTS] =
    [const { AtomicPtr::new(std::ptr::null_mut()) }; MEMO_SLOTS];

/// Retired (overwritten) memo records. Never read, never freed — see the
/// memory note above.
static MEMO_RETIRED: Mutex<Vec<&'static MemoRecord>> = Mutex::new(Vec::new());

/// 64-bit record tag from the registry-cell id and the combined generation.
#[inline]
fn memo_tag(bus_key: u64, gens: u64) -> u64 {
    let mut buf = [0u8; 16];
    buf[..8].copy_from_slice(&bus_key.to_le_bytes());
    buf[8..].copy_from_slice(&gens.to_le_bytes());
    fnv1a(&buf)
}

/// Slot index for an event-name hash (FNV low bits are multiplication
/// truncated — fold and re-scatter before masking).
#[inline]
fn memo_index(hash: u64) -> usize {
    ((hash ^ (hash >> 27)).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 24) as usize & MEMO_MASK
}

/// Probe the memo for (bus, gens, event). On a hit the record is borrowed
/// with process lifetime: published records are immutable and never freed,
/// so the raw pointer is always dereferenceable.
#[inline]
fn memo_find(bus_key: u64, gens: u64, hash: u64, event: &str) -> Option<&'static MemoRecord> {
    let tag = memo_tag(bus_key, gens);
    let mut i = memo_index(hash);
    for _ in 0..MEMO_PROBES {
        let p = MEMO_TABLE[i].load(Ordering::Acquire);
        if p.is_null() {
            // Inserts claim the first empty slot they meet, so a chain never
            // continues past an empty slot: definite miss.
            return None;
        }
        // SAFETY: `p` was published by a release swap/CAS and records are
        // never mutated or freed afterwards.
        let rec = unsafe { &*p };
        if rec.tag == tag && &*rec.name == event {
            return Some(rec);
        }
        i = (i + 1) & MEMO_MASK;
    }
    None
}

/// Hot publish through the global memo (TASK-167): one acquire load per
/// probe, no TLS, no refcounts. Re-entrant publishes from a handler simply
/// take the same read path — there is no slot state to take out or restore
/// (the TLS take/match/put-back protocol is gone with it).
#[inline]
fn memo_dispatch(
    bus_key: u64,
    gens: u64,
    hash: u64,
    event: &str,
    payload: &Value,
) -> Option<(usize, bool)> {
    let rec = memo_find(bus_key, gens, hash, event)?;
    Some((dispatch(&rec.resolved, event, payload), rec.has_async))
}

/// Hot has_subscribers through the global memo (TASK-167) — the same
/// zero-RMW read shape as [`memo_dispatch`].
#[inline]
fn memo_has(bus_key: u64, gens: u64, hash: u64, event: &str) -> Option<bool> {
    let rec = memo_find(bus_key, gens, hash, event)?;
    Some(!rec.resolved.is_empty() || rec.has_async)
}

/// Fill the memo on a cold miss (registry-writer frequency). A record for
/// the exact (bus, gens, event) key is content-identical to any existing
/// one — resolution is a pure function of the view at `gens` — so a
/// matching record is left in place and repeated cold fills are free.
#[cold]
fn memo_fill(bus_key: u64, gens: u64, hash: u64, event: &str, resolved: Resolved, has_async: bool) {
    let tag = memo_tag(bus_key, gens);
    let mut victim: Option<usize> = None;
    let mut i = memo_index(hash);
    for _ in 0..MEMO_PROBES {
        let slot = &MEMO_TABLE[i];
        let cur = slot.load(Ordering::Acquire);
        if cur.is_null() {
            let rec = Box::into_raw(Box::new(MemoRecord {
                tag,
                name: event.into(),
                resolved: Arc::clone(&resolved),
                has_async,
            }));
            match slot.compare_exchange(
                std::ptr::null_mut(),
                rec,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return,
                // Lost the claim race: this record was never published, so
                // no reader can hold it — drop it and keep probing.
                Err(_) => drop(unsafe { Box::from_raw(rec) }),
            }
        } else {
            // SAFETY: published record — immutable, never freed.
            let rec = unsafe { &*cur };
            if rec.tag == tag && &*rec.name == event {
                return; // already correct content
            }
            if victim.is_none() {
                victim = Some(i);
            }
        }
        i = (i + 1) & MEMO_MASK;
    }
    // Saturated region: overwrite the first probed victim.
    if let Some(vi) = victim {
        let rec = Box::into_raw(Box::new(MemoRecord {
            tag,
            name: event.into(),
            resolved: Arc::clone(&resolved),
            has_async,
        }));
        let old = MEMO_TABLE[vi].swap(rec, Ordering::AcqRel);
        if !old.is_null() {
            // SAFETY: the record was published (immutable, never freed);
            // park it forever so any reader still borrowing it stays sound.
            MEMO_RETIRED
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .push(unsafe { &*old });
        }
    }
}

/// TASK-173 A/B toggle: when true, an AsyncTask event name up to
/// INLINE_EVENT_CAP bytes is stored inline in the task (heap-free build
/// path); when false, every name is boxed (round-12 allocation baseline).
/// Both modes compile to the same struct layout, so the A/B isolates the
/// allocation, not a type-size delta between variants.
const ASYNC_EVENT_INLINE: bool = true;
const INLINE_EVENT_CAP: usize = 23;

/// Heap-free event-name storage for queued async tasks (TASK-173): the
/// build path previously paid one `to_string()` allocation (and the worker
/// paid its deallocation) per task. Names up to 23 bytes live in the task's
/// own footprint; longer names fall back to a boxed `str` — never
/// truncated, handler-visible names are byte-exact in both variants.
enum InlineEvent {
    Inline { len: u8, buf: [u8; INLINE_EVENT_CAP] },
    Heap(Box<str>),
}

impl From<&str> for InlineEvent {
    #[inline]
    fn from(s: &str) -> Self {
        if ASYNC_EVENT_INLINE && s.len() <= INLINE_EVENT_CAP {
            let mut buf = [0u8; INLINE_EVENT_CAP];
            buf[..s.len()].copy_from_slice(s.as_bytes());
            InlineEvent::Inline { len: s.len() as u8, buf }
        } else {
            InlineEvent::Heap(Box::from(s))
        }
    }
}

impl InlineEvent {
    #[inline]
    fn as_str(&self) -> &str {
        match self {
            InlineEvent::Inline { len, buf } => {
                // SAFETY: the bytes were copied verbatim from a valid `&str`
                // (UTF-8 by construction) and are only ever read back through
                // this slice of exactly `len` bytes.
                unsafe { std::str::from_utf8_unchecked(&buf[..*len as usize]) }
            }
            InlineEvent::Heap(s) => s,
        }
    }
}

/// One queued unit of async work: the handler snapshot for a single publish.
struct AsyncTask {
    event: InlineEvent,
    payload: Value,
    /// Phantom guards keep the module mappings alive while their handlers
    /// sit in the queue or run: a reload cannot dlclose a module whose async
    /// handlers are still pending or in flight (active-count protocol).
    leaders: Vec<Option<super::hot_reload::ModuleGuard>>,
    handlers: Resolved,
}

/// Bounded dispatcher pool. Workers are spawned lazily on first use and run
/// forever (daemon threads); the queue is a `Mutex<VecDeque>` + `Condvar`
/// with drop-oldest overflow, since `std::sync::mpsc::sync_channel` would
/// block the publisher (backpressure by blocking) instead of shedding load.
struct AsyncPool {
    queue: Mutex<VecDeque<AsyncTask>>,
    condvar: Condvar,
    cap: usize,
    /// Set while a drop burst is in progress, so we log once per burst.
    dropping: AtomicBool,
    dropped: AtomicUsize,
    workers: OnceLock<()>,
    /// Registered-idle worker count (TASK-170). A worker increments this
    /// while HOLDING the queue lock in the instant before it commits to
    /// `condvar.wait`, and decrements right after wake (also under the
    /// lock). Every access is serialized by the queue mutex, so a pusher
    /// that observes `idle == 0` while holding the lock knows every worker
    /// is inside the pop loop and will re-check the queue under the same
    /// lock before sleeping — the notify can then be skipped with no
    /// missed-wakeup window: any thread counted in `idle` is guaranteed
    /// to re-check the queue before its next sleep, and that re-check
    /// sees the pushed task. Notifying a condvar with no waiter is still
    /// a futex wake syscall (~0.3-3.5us measured) — under saturation every
    /// push paid it for nothing.
    idle: AtomicUsize,
}

impl AsyncPool {
    fn new(cap: usize) -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            condvar: Condvar::new(),
            cap,
            dropping: AtomicBool::new(false),
            dropped: AtomicUsize::new(0),
            workers: OnceLock::new(),
            idle: AtomicUsize::new(0),
        }
    }

    fn ensure_workers(self: &Arc<Self>) {
        self.workers.get_or_init(|| {
            for i in 0..ASYNC_WORKERS {
                let pool = Arc::clone(self);
                let spawned = std::thread::Builder::new()
                    .name(format!("crussty-events-{i}"))
                    .spawn(move || pool.worker_loop());
                if let Err(err) = spawned {
                    eprintln!("[crussty:events] failed to spawn dispatcher worker {i}: {err}");
                }
            }
        });
    }

    fn worker_loop(&self) {
        loop {
            let task = self.pop();
            self.run(task);
        }
    }

    /// TASK-170 hysteresis, shared by the spin and park pop paths: the
    /// drop-burst latch resets only when the queue genuinely drains (below
    /// half capacity) — one eprintln per overload episode, never a flood.
    fn latch_reset(&self, queue: &VecDeque<AsyncTask>) {
        if queue.len() < self.cap / 2 {
            self.dropping.store(false, Ordering::SeqCst);
        }
    }

    fn pop(&self) -> AsyncTask {
        // TASK-171 spin-then-park: before committing to the condvar park,
        // poll the queue through a bounded `try_lock` window. A publish that
        // lands inside the window is absorbed by one CAS pair instead of
        // waking a parked worker through the ~0.3-3.5us futex wake syscall;
        // the pusher's idle-gate cooperates for free — a spinning worker is
        // awake and NOT idle-counted, so the pusher skips the notify and this
        // loop observes the task on its next probe. A worker that finds
        // nothing falls through to the park path, which re-checks the queue
        // under the lock before registering idle, so no wakeup is lost.
        for _ in 0..ASYNC_SPIN_ITERS {
            if let Ok(mut queue) = self.queue.try_lock() {
                if let Some(task) = queue.pop_front() {
                    self.latch_reset(&queue);
                    return task;
                }
            }
            std::hint::spin_loop();
        }
        let mut queue = lock(&self.queue);
        loop {
            if let Some(task) = queue.pop_front() {
                self.latch_reset(&queue);
                return task;
            }
            // Register idle while still holding the lock (TASK-170), then
            // commit to the wait; decrement as soon as it returns so the
            // count never includes an awake worker for longer than the
            // wake-to-decrement window.
            self.idle.fetch_add(1, Ordering::SeqCst);
            queue = self.condvar.wait(queue).unwrap_or_else(|p| p.into_inner());
            self.idle.fetch_sub(1, Ordering::SeqCst);
        }
    }

    fn run(&self, mut task: AsyncTask) {
        // Release the guards as the handlers run, so a reload can proceed
        // once the last in-flight dispatch finishes.
        let mut leader_index = 0usize;
        for (owner, handler) in task.handlers.iter() {
            let _leader = task.leaders.get_mut(leader_index).and_then(Option::take);
            if owner.is_some() && _leader.is_none() {
                // Mid-swap: the module's mapping may be unmapped at any
                // moment; skip its queued handler instead of risking a call
                // into unloaded code.
                eprintln!(
                    "[crussty:events] async handler for '{}' skipped: module '{}' is being reloaded",
                    task.event.as_str(),
                    owner.as_ref().expect("owner checked").0
                );
                leader_index += 1;
                continue;
            }
            leader_index += 1;
            let result = catch_unwind(AssertUnwindSafe(|| handler(task.event.as_str(), &task.payload)));
            if let Err(panic) = result {
                eprintln!(
                    "[crussty:events] async handler panicked for '{}': {panic:?}",
                    task.event.as_str()
                );
            }
        }
    }
}

impl Default for AsyncPool {
    fn default() -> Self {
        Self::new(ASYNC_QUEUE_CAP)
    }
}

/// Lock with recovery from mutex poisoning: a panicked handler must never
/// wedge the bus, and our critical sections never hold user code.
fn lock<T>(guard: &Mutex<T>) -> MutexGuard<'_, T> {
    guard.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn dispatch(handlers: &[(Option<(Box<str>, u64)>, Handler)], event: &str, payload: &Value) -> usize {
    for (owner, handler) in handlers.iter() {
        let guard = owner
            .as_ref()
            .and_then(|(id, _)| super::hot_reload::guard_module(id));
        if owner.is_some() && guard.is_none() {
            report_skipped_reload(&owner.as_ref().expect("owner checked").0, event);
            continue;
        }
        let result = catch_unwind(AssertUnwindSafe(|| handler(event, payload)));
        if let Err(panic) = result {
            report_handler_panic(event, panic);
        }
    }
    handlers.len()
}

/// Panic-reporting cold tails (TASK-166): the eprintln formatting machinery
/// must not sit in the dispatch hot body.
#[cold]
#[inline(never)]
fn report_handler_panic(event: &str, panic: Box<dyn std::any::Any + Send>) {
    eprintln!("[crussty:events] handler panicked for '{event}': {panic:?}");
}

#[cold]
#[inline(never)]
fn report_skipped_reload(owner: &str, event: &str) {
    eprintln!(
        "[crussty:events] sync handler for '{event}' skipped: module '{owner}' is being reloaded"
    );
}

#[derive(Clone)]
pub struct EventBus {
    sync: Arc<RegistryCell>,
    async_cell: Arc<RegistryCell>,
    pool: Arc<AsyncPool>,
    next_id: Arc<AtomicU64>,
    emitting_lifecycle: Arc<AtomicBool>,
    /// Combined (sync<<32 | async) mutation generations — the ONE atomic the
    /// hot gate and the memo key read (TASK-164).
    gens: Arc<AtomicU64>,
}

impl Default for EventBus {
    fn default() -> Self {
        let gens = Arc::new(AtomicU64::new(0));
        Self {
            sync: Arc::new(RegistryCell::new(Arc::clone(&gens), 32)),
            async_cell: Arc::new(RegistryCell::new(Arc::clone(&gens), 0)),
            pool: Arc::new(AsyncPool::default()),
            next_id: Arc::new(AtomicU64::new(0)),
            emitting_lifecycle: Arc::new(AtomicBool::new(false)),
            gens,
        }
    }
}

static GLOBAL: OnceLock<EventBus> = OnceLock::new();

pub fn global() -> EventBus {
    GLOBAL.get_or_init(EventBus::default).clone()
}

/// Borrowed handle to the process-global bus (TASK-163): hot callers (the
/// per-class-load gate) must not pay the five-Arc clone `global()` costs to
/// consult a counter. Same bus, no clone.
pub fn global_ref() -> &'static EventBus {
    GLOBAL.get_or_init(EventBus::default)
}

impl EventBus {
    /// Subscribe to an event name. Handler runs synchronously on the
    /// publisher's thread, in subscription order. Returns a token usable
    /// with [`EventBus::unsubscribe`]. When called inside a module
    /// registration window the subscription is owned by that module
    /// generation (purged on its hot reload).
    pub fn subscribe(&self, event: &str, f: Handler) -> Subscription {
        let owner = crate::registration_owner();
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let token = self.sync.mutate(|r| {
            let tok = r.insert(event, f, id, owner);
            (true, tok)
        });
        self.emit_lifecycle(lifecycle::EVENT_SUBSCRIBED, event, token.id);
        token
    }

    /// Subscribe with the handler dispatched on the pool instead of the
    /// publisher's thread. Never blocks `publish`; see module docs for
    /// ordering and backpressure guarantees. Module-owned like
    /// [`EventBus::subscribe`].
    pub fn subscribe_async(&self, event: &str, f: Handler) -> Subscription {
        let owner = crate::registration_owner();
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let token = self.async_cell.mutate(|r| {
            let tok = r.insert(event, f, id, owner);
            (true, tok)
        });
        self.emit_lifecycle(lifecycle::EVENT_SUBSCRIBED, event, token.id);
        token
    }

    /// Drop every sync and async subscription owned by a module generation
    /// (id, library gen). Used by the hot-reload brick right before it
    /// dlcloses the replaced library, so no publish can ever invoke an
    /// unloaded callback. Returns the number of subscriptions removed.
    pub fn purge_owner(&self, owner: &(String, u64)) -> usize {
        let removed_sync = self.sync.mutate(|r| {
            let n = r.purge_owner(owner);
            (n > 0, n)
        });
        let removed_async = self.async_cell.mutate(|r| {
            let n = r.purge_owner(owner);
            (n > 0, n)
        });
        let removed = removed_sync + removed_async;
        if removed > 0 {
            eprintln!(
                "[crussty:events] purge: dropped {removed} subscription(s) owned by '{}' gen {}",
                owner.0, owner.1
            );
        }
        removed
    }

    /// True when at least one handler (sync or async) is subscribed to
    /// `event` (exact name or matching pattern). A cheap existence check —
    /// no handler snapshot — so hot paths (e.g. the class-load hook) can
    /// skip building a payload entirely when nobody listens.
    pub fn has_subscribers(&self, event: &str) -> bool {
        // Lock-free fast gate (TASK-164): combined generation 0 = never
        // mutated = both registries empty — ONE acquire load, no locks.
        let gens = self.gens.load(Ordering::Acquire);
        if gens == 0 {
            return false;
        }
        // Global memo hit (TASK-167): one acquire load per probe, no TLS.
        // The hash is computed once and reused by the cold path below.
        let hash = fnv1a(event.as_bytes());
        if let Some(any) = memo_has(self.sync.id, gens, hash, event) {
            return any;
        }
        let (resolved, has_async) = self.resolve_event(event, hash, gens >> 32, gens & 0xFFFF_FFFF);
        let any = !resolved.is_empty() || has_async;
        memo_fill(self.sync.id, gens, hash, event, resolved, has_async);
        any
    }

    /// Remove a subscription. Returns `false` if the token is unknown or
    /// stale (never emitted on this bus, already removed, or its list was
    /// mutated since).
    pub fn unsubscribe(&self, event: &str, token: &Subscription) -> bool {
        let removed = self.sync.mutate(|r| {
            let x = r.remove(event, token);
            (x, x)
        });
        let removed = if removed {
            true
        } else {
            self.async_cell.mutate(|r| {
                let x = r.remove(event, token);
                (x, x)
            })
        };
        if removed {
            self.emit_lifecycle(lifecycle::EVENT_UNSUBSCRIBED, event, token.id);
        }
        removed
    }

    /// Publish an event; returns the number of sync handlers invoked.
    /// Async handlers for this event are queued as one task and dispatched
    /// on the pool.
    pub fn publish(&self, event: &str, payload: &Value) -> usize {
        // Lock-free fast gate (TASK-164): combined generation 0 = nothing
        // ever subscribed — ONE acquire load, no locks. This is the
        // per-tick / per-class-load default shape.
        let gens = self.gens.load(Ordering::Acquire);
        if gens == 0 {
            return 0;
        }
        // Global memo hit (TASK-167): no TLS, no refcounts — one acquire
        // load per probe. The hash is computed once and reused by the cold
        // resolution path and the async queue below.
        let hash = fnv1a(event.as_bytes());
        if let Some((invoked, has_async)) =
            memo_dispatch(self.sync.id, gens, hash, event, payload)
        {
            if has_async {
                self.queue_async(event, payload, hash);
            }
            return invoked;
        }
        let (resolved, has_async) = self.resolve_event(event, hash, gens >> 32, gens & 0xFFFF_FFFF);
        memo_fill(self.sync.id, gens, hash, event, Arc::clone(&resolved), has_async);
        let invoked = dispatch(&resolved, event, payload);
        if has_async {
            self.queue_async(event, payload, hash);
        }
        invoked
    }

    /// Cold resolution path: load the RCU views, build the resolved lists.
    fn resolve_event(&self, event: &str, hash: u64, sg: u64, ag: u64) -> (Resolved, bool) {
        let resolved: Resolved = if sg > 0 {
            self.sync
                .view
                .load_arc()
                .map(|v| v.resolve(hash, event))
                .unwrap_or_else(|| Arc::from(Vec::new()))
        } else {
            Arc::from(Vec::new())
        };
        let has_async = ag > 0
            && self
                .async_cell
                .view
                .load_arc()
                .is_some_and(|v| v.any_match(hash, event));
        (resolved, has_async)
    }

    /// Queue the async handlers for one publish (cold: only when an async
    /// subscription matches).
    fn queue_async(&self, event: &str, payload: &Value, hash: u64) {
        let Some(list) = self
            .async_cell
            .view
            .load_arc()
            .map(|v| v.resolve(hash, event))
            .filter(|l| !l.is_empty())
        else {
            return;
        };
        // Quiescence: every queued handler keeps its module's guard alive
        // until the pool has run it (see AsyncTask::leaders), so a hot
        // reload waits instead of dlclosing under pending handlers.
        let leaders = list
            .iter()
            .map(|(owner, _)| {
                owner
                    .as_ref()
                    .and_then(|(id, _)| super::hot_reload::guard_module(id))
            })
            .collect();
        self.pool.push(AsyncTask {
            event: event.into(),
            payload: payload.clone(),
            leaders,
            handlers: list,
        });
    }

    /// Number of events currently queued for async dispatch (queue depth).
    pub fn async_pending(&self) -> usize {
        lock(&self.pool.queue).len()
    }

    /// Total events dropped by the backpressure cap since bus creation.
    pub fn async_dropped(&self) -> usize {
        self.pool.dropped.load(Ordering::SeqCst)
    }

    /// Emit a lifecycle event on the bus itself, guarded against re-entrant
    /// emission: subscribe/unsubscribe calls made from a lifecycle handler
    /// apply silently (no infinite recursion, no event flood).
    fn emit_lifecycle(&self, event: &str, subject: &str, subscription: u64) {
        if self.emitting_lifecycle.swap(true, Ordering::SeqCst) {
            return;
        }
        self.publish(event, &serde_json::json!({ "event": subject, "subscription": subscription }));
        self.emitting_lifecycle.store(false, Ordering::SeqCst);
    }
}

impl AsyncPool {
    /// Enqueue one dispatch task. Never blocks the publisher: when the queue
    /// is at capacity, the oldest pending task is dropped (load shedding)
    /// and logged once per burst.
    fn push(self: &Arc<Self>, task: AsyncTask) {
        self.ensure_workers();
        let mut queue = lock(&self.queue);
        if queue.len() >= self.cap {
            queue.pop_front();
            self.dropped.fetch_add(1, Ordering::SeqCst);
            if !self.dropping.swap(true, Ordering::SeqCst) {
                eprintln!(
                    "[crussty:events] async dispatch queue at capacity ({}) — dropping oldest pending events",
                    self.cap
                );
            }
        }
        queue.push_back(task);
        // Depth-, idle- and TRANSITION-aware notify (TASK-170 gate
        // generalized by TASK-171; 02a891e shipped the OFF variant of the
        // round-10 A/B — unconditional notify behind a dead `idle.load`
        // under a stale TEMP comment — so the measured winner never reached
        // production). Fire exactly when THIS push creates the first
        // backlog unit beyond the awake workforce (len == awake + 1, where
        // awake = ASYNC_WORKERS - idle): awake workers either hold the pop
        // loop or sit in the TASK-171 spin window and are guaranteed to
        // observe the queue under the lock before they can sleep (the park
        // path re-checks first), so a shallow queue is covered without any
        // syscall. Later pushes while the backlog persists must NOT
        // re-notify: the woken worker drains serially at ~100ns/task, so a
        // per-push wake against it is a 3.5us-per-task syscall flood (the
        // preemption scenario that dominated the naive depth gate).
        // `idle` changes only under the queue lock and the pusher holds
        // that lock here, so this snapshot is exact for the decision
        // instant; len evolves by +-1 under the same lock, so the equality
        // is the crossing detector, not a coincidence filter.
        let idle = self.idle.load(Ordering::SeqCst);
        if idle > 0 && queue.len() == ASYNC_WORKERS - idle + 1 {
            self.condvar.notify_one();
        }
    }
}

/// Platform lifecycle events (published by the runtime itself).
pub mod lifecycle {
    pub const CLASS_LOADED: &str = "platform.class_loaded";
    pub const PLUGIN_LOADED: &str = "platform.plugin_loaded";
    pub const PLUGIN_UNLOADED: &str = "platform.plugin_unloaded";
    pub const TICK_BOUNDARY: &str = "platform.tick_boundary";
    pub const SAVE_COMPLETE: &str = "platform.save_complete";
    /// Emitted on the bus itself whenever a subscription is added.
    pub const EVENT_SUBSCRIBED: &str = "platform.event_subscribed";
    /// Emitted on the bus itself whenever a subscription is removed.
    pub const EVENT_UNSUBSCRIBED: &str = "platform.event_unsubscribed";
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::{Duration, Instant};

    #[test]
    fn inline_event_stores_byte_exact_names() {
        // Round-trip through both variants: short names take the inline
        // path under the A/B toggle, longer names (24 bytes) must fall
        // back to the heap variant without truncation — handler-visible
        // names are byte-exact in every case.
        let short = InlineEvent::from("bench.async");
        let boundary = InlineEvent::from("x".repeat(INLINE_EVENT_CAP).as_str());
        let overflow = InlineEvent::from("platform.plugin_unloaded");
        assert_eq!(short.as_str(), "bench.async");
        assert_eq!(boundary.as_str().len(), INLINE_EVENT_CAP);
        assert_eq!(overflow.as_str(), "platform.plugin_unloaded");
        // Multibyte UTF-8 must survive the byte copy (len is bytes).
        let multibyte = InlineEvent::from("событие");
        assert_eq!(multibyte.as_str(), "событие");
    }

    pub(super) fn with_cap(cap: usize) -> EventBus {
        let gens = Arc::new(AtomicU64::new(0));
        EventBus {
            sync: Arc::new(RegistryCell::new(Arc::clone(&gens), 32)),
            async_cell: Arc::new(RegistryCell::new(Arc::clone(&gens), 0)),
            pool: Arc::new(AsyncPool::new(cap)),
            next_id: Arc::new(AtomicU64::new(0)),
            emitting_lifecycle: Arc::new(AtomicBool::new(false)),
            gens,
        }
    }

    fn wait_until(mut cond: impl FnMut() -> bool) -> bool {
        let deadline = Instant::now() + Duration::from_secs(5);
        while !cond() {
            if Instant::now() >= deadline {
                return false;
            }
            thread::sleep(Duration::from_millis(5));
        }
        true
    }

    #[test]
    fn pub_sub_roundtrip() {
        let bus = EventBus::default();
        let n = Arc::new(AtomicUsize::new(0));
        let n2 = Arc::clone(&n);
        bus.subscribe("test.evt", Arc::new(move |_, _| {
            n2.fetch_add(1, Ordering::SeqCst);
        }));
        let count = bus.publish("test.evt", &serde_json::json!({"a": 1}));
        assert_eq!(count, 1);
        assert_eq!(n.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn has_subscribers_tracks_exact_and_pattern_subscriptions() {
        let bus = EventBus::default();
        assert!(!bus.has_subscribers("platform.class_loaded"), "no handlers yet");

        bus.subscribe(lifecycle::CLASS_LOADED, Arc::new(|_, _| {}));
        assert!(bus.has_subscribers(lifecycle::CLASS_LOADED));
        assert!(!bus.has_subscribers("platform.tick_boundary"), "different event");

        // Pattern subscriptions count too (the class-load path skips payload
        // construction only when *nothing* can receive the event).
        let bus2 = EventBus::default();
        bus2.subscribe("platform.*", Arc::new(|_, _| {}));
        assert!(bus2.has_subscribers(lifecycle::CLASS_LOADED));
        assert!(!bus2.has_subscribers("other.event"));

        // Unsubscribing flips the flag back.
        let bus3 = EventBus::default();
        let token = bus3.subscribe(lifecycle::CLASS_LOADED, Arc::new(|_, _| {}));
        assert!(bus3.has_subscribers(lifecycle::CLASS_LOADED));
        bus3.unsubscribe(lifecycle::CLASS_LOADED, &token);
        assert!(!bus3.has_subscribers(lifecycle::CLASS_LOADED));
    }

    #[test]
    fn sync_handlers_run_in_subscription_order() {
        let bus = EventBus::default();
        let order = Arc::new(Mutex::new(Vec::new()));
        for i in 0..3 {
            let order = Arc::clone(&order);
            bus.subscribe("order.evt", Arc::new(move |_, _| order.lock().unwrap().push(i)));
        }
        assert_eq!(bus.publish("order.evt", &serde_json::json!(null)), 3);
        assert_eq!(*order.lock().unwrap(), vec![0, 1, 2]);
    }

    #[test]
    fn wildcard_receives_all_events_with_name() {
        let bus = EventBus::default();
        let got = Arc::new(Mutex::new(Vec::new()));
        let exact = Arc::clone(&got);
        bus.subscribe("a.b", Arc::new(move |event, _| exact.lock().unwrap().push(event.to_string())));
        let wild = Arc::clone(&got);
        bus.subscribe("*", Arc::new(move |event, _| wild.lock().unwrap().push(event.to_string())));

        assert_eq!(bus.publish("a.b", &serde_json::json!(1)), 2);
        assert_eq!(bus.publish("x.y.z", &serde_json::json!(2)), 1);

        let got = got.lock().unwrap();
        // The wildcard handler also receives the lifecycle event emitted
        // when its own subscription was registered.
        assert_eq!(*got, vec!["platform.event_subscribed", "a.b", "a.b", "x.y.z"]);
    }

    #[test]
    fn glob_pattern_matching() {
        let bus = EventBus::default();
        let got = Arc::new(Mutex::new(Vec::new()));
        let g = Arc::clone(&got);
        bus.subscribe(
            "platform.*",
            Arc::new(move |event, _| g.lock().unwrap().push(event.to_string())),
        );
        let g = Arc::clone(&got);
        bus.subscribe(
            "platform.save_complete",
            Arc::new(move |event, _| g.lock().unwrap().push(event.to_string())),
        );

        assert_eq!(bus.publish("platform.save_complete", &serde_json::json!(null)), 2);
        assert_eq!(bus.publish("platform.plugin_loaded", &serde_json::json!(null)), 1);
        // "platform.*" is exactly two segments: no partial/trailing matching.
        assert_eq!(bus.publish("platform.a.b", &serde_json::json!(null)), 0);
        assert_eq!(bus.publish("other.evt", &serde_json::json!(null)), 0);

        let got = got.lock().unwrap();
        // `platform.*` matches the lifecycle event `platform.event_subscribed`
        // (emitted during subscription registration) and `platform.plugin_loaded`.
        assert_eq!(
            *got,
            vec![
                "platform.event_subscribed",
                "platform.event_subscribed",
                "platform.save_complete",
                "platform.save_complete",
                "platform.plugin_loaded"
            ]
        );
    }

    #[test]
    fn async_handlers_eventually_run() {
        let bus = EventBus::default();
        let n = Arc::new(AtomicUsize::new(0));
        let n2 = Arc::clone(&n);
        bus.subscribe_async("async.evt", Arc::new(move |_, _| {
            n2.fetch_add(1, Ordering::SeqCst);
        }));
        // Async-only publish: no sync handlers invoked...
        assert_eq!(bus.publish("async.evt", &serde_json::json!(1)), 0);
        // ...but the handler runs on the pool shortly after.
        assert!(wait_until(|| n.load(Ordering::SeqCst) == 1), "async handler never ran");
    }

    #[test]
    fn async_handlers_run_in_subscription_order_on_pool() {
        let bus = EventBus::default();
        let order = Arc::new(Mutex::new(Vec::new()));
        for i in 0..3 {
            let order = Arc::clone(&order);
            bus.subscribe_async("async.order", Arc::new(move |_, _| order.lock().unwrap().push(i)));
        }
        bus.publish("async.order", &serde_json::json!(null));
        assert!(
            wait_until(|| order.lock().unwrap().len() == 3),
            "async handlers never ran"
        );
        assert_eq!(*order.lock().unwrap(), vec![0, 1, 2]);
    }

    #[test]
    fn sync_handler_panic_does_not_propagate() {
        let bus = EventBus::default();
        bus.subscribe("panic.evt", Arc::new(|_, _| panic!("sync boom")));
        let n = Arc::new(AtomicUsize::new(0));
        let n2 = Arc::clone(&n);
        bus.subscribe("panic.evt", Arc::new(move |_, _| { n2.fetch_add(1, Ordering::SeqCst); }));

        assert_eq!(bus.publish("panic.evt", &serde_json::json!(null)), 2);
        assert_eq!(n.load(Ordering::SeqCst), 1, "second handler must still run");
        // The bus keeps working after a panicked handler.
        assert_eq!(bus.publish("panic.evt", &serde_json::json!(null)), 2);
        assert_eq!(n.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn async_handler_panic_is_isolated() {
        let bus = EventBus::default();
        bus.subscribe_async("ap.evt", Arc::new(|_, _| panic!("async boom")));
        let done = Arc::new(AtomicUsize::new(0));
        let d = Arc::clone(&done);
        bus.subscribe_async("ap.evt", Arc::new(move |_, _| { d.fetch_add(1, Ordering::SeqCst); }));

        bus.publish("ap.evt", &serde_json::json!(null));
        assert!(
            wait_until(|| done.load(Ordering::SeqCst) >= 1),
            "good async handler never ran after a panic"
        );
        // The pool worker survived the panic: dispatch still works.
        bus.publish("ap.evt", &serde_json::json!(null));
        assert!(wait_until(|| done.load(Ordering::SeqCst) >= 2), "worker died after panic");
    }

    #[test]
    fn unsubscribe_removes_handler() {
        let bus = EventBus::default();
        let n = Arc::new(AtomicUsize::new(0));
        let n2 = Arc::clone(&n);
        let token = bus.subscribe("unsub.evt", Arc::new(move |_, _| { n2.fetch_add(1, Ordering::SeqCst); }));

        assert_eq!(bus.publish("unsub.evt", &serde_json::json!(null)), 1);
        assert!(bus.unsubscribe("unsub.evt", &token));
        assert!(!bus.unsubscribe("unsub.evt", &token), "double unsubscribe must fail");
        assert_eq!(bus.publish("unsub.evt", &serde_json::json!(null)), 0);
        assert_eq!(n.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn unsubscribe_works_for_async_and_glob_subscriptions() {
        let bus = EventBus::default();
        let n = Arc::new(AtomicUsize::new(0));
        let n2 = Arc::clone(&n);
        let token = bus.subscribe_async("ua.evt", Arc::new(move |_, _| { n2.fetch_add(1, Ordering::SeqCst); }));
        assert!(bus.unsubscribe("ua.evt", &token));
        bus.publish("ua.evt", &serde_json::json!(null));
        thread::sleep(Duration::from_millis(100));
        assert_eq!(n.load(Ordering::SeqCst), 0, "unsubscribed async handler still ran");

        let n = Arc::new(AtomicUsize::new(0));
        let n2 = Arc::clone(&n);
        let token = bus.subscribe("metrics.*", Arc::new(move |_, _| { n2.fetch_add(1, Ordering::SeqCst); }));
        assert_eq!(bus.publish("metrics.cpu", &serde_json::json!(null)), 1);
        assert!(bus.unsubscribe("metrics.*", &token));
        assert_eq!(bus.publish("metrics.cpu", &serde_json::json!(null)), 0);
    }

    #[test]
    fn backpressure_drops_oldest_events() {
        let cap = 64usize;
        let total = cap + 10;
        let bus = with_cap(cap);

        let seen = Arc::new(Mutex::new(Vec::new()));
        let gate = Arc::new(Mutex::new(()));
        let g = Arc::clone(&gate);
        let s = Arc::clone(&seen);
        bus.subscribe_async("flood.evt", Arc::new(move |_, payload| {
            let _guard = g.lock().unwrap();
            s.lock().unwrap().push(payload["i"].as_u64().unwrap());
        }));

        // Hold the gate so both pool workers block inside the handler and the
        // queue fills deterministically.
        let gate_guard = gate.lock().unwrap();
        for i in 0..total {
            bus.publish("flood.evt", &serde_json::json!({ "i": i }));
        }

        // Queue length is capped exactly at `cap`, and the oldest events were
        // shed: the queue holds only the newest tasks {10..=73}.
        {
            let queue = lock(&bus.pool.queue);
            assert_eq!(queue.len(), cap, "queue must be capped");
            assert_eq!(queue.front().unwrap().payload["i"].as_u64(), Some(10));
            assert_eq!(queue.back().unwrap().payload["i"].as_u64(), Some(73));
        }
        assert!(bus.async_dropped() > 0, "overflow must drop events");

        drop(gate_guard); // release the workers
        assert!(
            wait_until(|| seen.lock().unwrap().len() == total - bus.async_dropped()),
            "queued events never drained"
        );

        let mut received = seen.lock().unwrap().clone();
        received.sort_unstable();
        received.dedup();
        assert_eq!(received.len(), total - bus.async_dropped());
        assert_eq!(*received.last().unwrap(), (total - 1) as u64, "newest event must be delivered");
        // Every task that stayed in the queue is delivered exactly once.
        assert_eq!(received.iter().filter(|i| **i >= 10).count(), cap);
    }

    #[test]
    fn lifecycle_events_are_published_and_not_redispatched() {
        let bus = EventBus::default();
        let subscribed = Arc::new(AtomicUsize::new(0));
        let s1 = Arc::clone(&subscribed);
        bus.subscribe(lifecycle::EVENT_SUBSCRIBED, Arc::new(move |_, _| {
            s1.fetch_add(1, Ordering::SeqCst);
        }));
        assert_eq!(subscribed.load(Ordering::SeqCst), 1, "subscribing fires EVENT_SUBSCRIBED once");

        let unsubscribed = Arc::new(AtomicUsize::new(0));
        let u1 = Arc::clone(&unsubscribed);
        bus.subscribe(lifecycle::EVENT_UNSUBSCRIBED, Arc::new(move |_, _| {
            u1.fetch_add(1, Ordering::SeqCst);
        }));
        assert_eq!(subscribed.load(Ordering::SeqCst), 2);

        bus.subscribe("some.evt", Arc::new(|_, _| {}));
        assert_eq!(subscribed.load(Ordering::SeqCst), 3);

        let token = bus.subscribe("another.evt", Arc::new(|_, _| {}));
        assert_eq!(subscribed.load(Ordering::SeqCst), 4);
        assert!(bus.unsubscribe("another.evt", &token));
        assert_eq!(subscribed.load(Ordering::SeqCst), 4, "no EVENT_SUBSCRIBED on unsubscribe");
        assert_eq!(unsubscribed.load(Ordering::SeqCst), 1, "unsubscribe fires EVENT_UNSUBSCRIBED");
    }

    #[test]
    fn purge_owner_removes_module_subscriptions_only() {
        let bus = EventBus::default();

        // Sync + async subscriptions inside a module window (gen 1).
        {
            let _g = crate::begin_registration("hello", 1);
            bus.subscribe("a.evt", Arc::new(|_, _| {}));
            bus.subscribe_async("b.evt", Arc::new(|_, _| {}));
        }
        // A newer generation of the same module subscribes too.
        {
            let _g = crate::begin_registration("hello", 2);
            bus.subscribe("a.evt", Arc::new(|_, _| {}));
        }
        // A different module, and an unowned (platform) subscriber.
        {
            let _g = crate::begin_registration("dist", 1);
            bus.subscribe("c.evt", Arc::new(|_, _| {}));
        }
        bus.subscribe("d.evt", Arc::new(|_, _| {}));

        assert_eq!(bus.publish("a.evt", &serde_json::json!(null)), 2);
        assert_eq!(bus.publish("b.evt", &serde_json::json!(null)), 0, "async-only");

        // Purging hello gen 1 removes exactly its sync + async handlers.
        assert_eq!(
            bus.purge_owner(&("hello".to_string(), 1)),
            2,
            "hello gen 1 held sync 'a.evt' + async 'b.evt'"
        );
        assert_eq!(bus.publish("a.evt", &serde_json::json!(null)), 1, "hello gen 2 stays");
        assert_eq!(bus.publish("b.evt", &serde_json::json!(null)), 0, "async purged");
        assert_eq!(bus.publish("c.evt", &serde_json::json!(null)), 1, "dist untouched");
        assert_eq!(bus.publish("d.evt", &serde_json::json!(null)), 1, "unowned untouched");

        // Purging gen that owns nothing is a no-op.
        assert_eq!(bus.purge_owner(&("hello".to_string(), 3)), 0);
    }

    #[test]
    fn lifecycle_recursion_is_guarded() {
        let bus = EventBus::default();
        let n = Arc::new(AtomicUsize::new(0));
        let count = Arc::clone(&n);
        let bus2 = bus.clone();
        // Subscribing inside a lifecycle handler must not re-emit: without
        // the guard this would recurse forever.
        bus.subscribe(lifecycle::EVENT_SUBSCRIBED, Arc::new(move |_, _| {
            count.fetch_add(1, Ordering::SeqCst);
            bus2.subscribe("dummy.evt", Arc::new(|_, _| {}));
        }));
        assert_eq!(n.load(Ordering::SeqCst), 1, "nested subscribe must not re-emit");

        bus.subscribe("x.evt", Arc::new(|_, _| {}));
        assert_eq!(n.load(Ordering::SeqCst), 2, "later subscriptions emit exactly once");
    }

    #[test]
    fn global_memo_reentrant_publish_from_handler() {
        let bus = EventBus::default();
        let inner = Arc::new(AtomicUsize::new(0));
        let i2 = Arc::clone(&inner);
        bus.subscribe("r.inner", Arc::new(move |_, _| {
            i2.fetch_add(1, Ordering::SeqCst);
        }));
        let bus2 = bus.clone();
        bus.subscribe("r.outer", Arc::new(move |_, _| {
            // Nested publish while the outer dispatch is live: with the
            // global memo there is no slot state to take out — the nested
            // publish just probes the same table.
            bus2.publish("r.inner", &serde_json::json!(null));
        }));
        assert_eq!(bus.publish("r.outer", &serde_json::json!(null)), 1);
        assert_eq!(inner.load(Ordering::SeqCst), 1);
        // Second publish rides the memo for both keys, nested dispatch included.
        assert_eq!(bus.publish("r.outer", &serde_json::json!(null)), 1);
        assert_eq!(inner.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn global_memo_concurrent_publish_and_mutate() {
        let bus = EventBus::default();
        let hits = Arc::new(AtomicUsize::new(0));
        let pat_hits = Arc::new(AtomicUsize::new(0));
        let h2 = Arc::clone(&hits);
        bus.subscribe("g.evt", Arc::new(move |event, _| {
            assert_eq!(event, "g.evt", "memo must never dispatch a wrong list");
            h2.fetch_add(1, Ordering::SeqCst);
        }));

        // Warm the memo on this thread (fill), then account the baseline.
        assert_eq!(bus.publish("g.evt", &serde_json::json!(null)), 1);
        let mut accounted = hits.load(Ordering::SeqCst) + pat_hits.load(Ordering::SeqCst);

        let stop = Arc::new(AtomicBool::new(false));
        let mut handles = Vec::new();
        for _ in 0..4 {
            let bus = bus.clone();
            let stop = Arc::clone(&stop);
            handles.push(std::thread::spawn(move || {
                let mut returned = 0usize;
                while !stop.load(Ordering::Relaxed) {
                    returned += bus.publish("g.evt", &serde_json::json!(null));
                }
                returned
            }));
        }
        // Mutator: churn the registry so the combined generation (and hence
        // the memo tags) keeps moving under the concurrent publishers.
        for _ in 0..2_000 {
            let p2 = Arc::clone(&pat_hits);
            let tok = bus.subscribe("g.*", Arc::new(move |_, _| { p2.fetch_add(1, Ordering::SeqCst); }));
            bus.unsubscribe("g.*", &tok);
        }
        stop.store(true, Ordering::Relaxed);
        let total_returned: usize = handles.into_iter().map(|h| h.join().unwrap()).sum();
        accounted += total_returned;
        // Strong coherence invariant: `publish` returns the exact number of
        // sync handler invocations, so the handler atomics must sum to every
        // returned count (each sync dispatch runs to completion on the
        // publishing thread before the return value is produced).
        assert_eq!(
            hits.load(Ordering::SeqCst) + pat_hits.load(Ordering::SeqCst),
            accounted,
            "handler invocations must exactly match publish return counts"
        );
    }
}

#[cfg(test)]
mod bench_hotpath {
    //! Release-only A/B benches: `cargo test --release -- --ignored --nocapture bench_events`.
    use super::*;
    use super::tests::with_cap;
    use std::time::Instant;

    #[test]
    #[ignore]
    fn bench_event_bus_publish() {
        let bus = with_cap(64);
        let payload = serde_json::json!({ "tick": 1u64, "drained": 0u64 });
        let iters = 200_000u32;
        let rounds = 5;

        // publish with zero subscribers (steady-state tick boundary shape)
        for _ in 0..10_000u32 {
            let _ = bus.publish("bench.none", &payload);
        }
        let mut best_none = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                let _ = bus.publish("bench.none", &payload);
            }
            best_none = best_none.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }

        // one sync subscriber
        let _tok = bus.subscribe("bench.one", Arc::new(|_, _| {}));
        for _ in 0..10_000u32 {
            let _ = bus.publish("bench.one", &payload);
        }
        let mut best_one = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                let _ = bus.publish("bench.one", &payload);
            }
            best_one = best_one.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }

        // one glob-pattern subscriber, then has_subscribers (exact miss + 1 pattern scan)
        let _pat = bus.subscribe("platform.*", Arc::new(|_, _| {}));
        for _ in 0..10_000u32 {
            let _ = bus.has_subscribers("platform.tick_boundary");
        }
        let mut best_glob = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                let _ = bus.has_subscribers("platform.tick_boundary");
            }
            best_glob = best_glob.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }
        // has_subscribers on a completely empty bus — the production global
        // default shape (TASK-162 gate subject). Fresh bus so the gate, not
        // the registry scan, is what the number reflects.
        let empty = with_cap(64);
        for _ in 0..10_000u32 {
            let _ = empty.has_subscribers("platform.tick_boundary");
        }
        let mut best_empty = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                let _ = empty.has_subscribers("platform.tick_boundary");
            }
            best_empty = best_empty.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }
        // one async subscriber — characterizes the full enqueue path
        // (TASK-169): resolve + guards + task alloc + pool.push (Mutex +
        // condvar notify). The number includes the worker wake cost: the
        // honest per-publish price of the async pool as built.
        let _atok = bus.subscribe_async("bench.async", Arc::new(|_, _| {}));
        for _ in 0..10_000u32 {
            let _ = bus.publish("bench.async", &payload);
        }
        let mut best_async = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                let _ = bus.publish("bench.async", &payload);
            }
            best_async = best_async.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }
        // pool.push isolated from queue_async (TASK-169): (a) build-only
        // per-op cost of one AsyncTask (to_string + Value clone + allocs),
        // (b) build + push — the delta is the enqueue critical section
        // (Mutex + push_back + condvar notify, including the worker wake).
        let pool = Arc::clone(&bus.pool);
        let build_task = || AsyncTask {
            event: "bench.async".into(),
            payload: payload.clone(),
            leaders: Vec::new(),
            handlers: Arc::from(Vec::new()),
        };
        for _ in 0..10_000u32 {
            drop(build_task());
        }
        let mut best_build = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                drop(build_task());
            }
            best_build = best_build.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }
        for _ in 0..10_000u32 {
            pool.push(build_task());
        }
        let mut best_push = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                pool.push(build_task());
            }
            best_push = best_push.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }
        // Saturated regime (TASK-170): the handler does real work so the
        // publisher outpaces the pool; the queue sits at capacity and
        // drop-oldest sheds. This is the shape where the notify cost
        // (empty-waiter futex wake per push) and the drop-latch log policy
        // show their true per-op price. Warmup must reach saturation
        // (queue full) before the timed rounds start.
        let _slok = bus.subscribe_async("bench.async.slow", Arc::new(|_, _| {
            let mut x = 0u64;
            for i in 0..20_000u64 {
                x = x.wrapping_add(i.wrapping_mul(7));
            }
            std::hint::black_box(x);
        }));
        for _ in 0..30_000u32 {
            let _ = bus.publish("bench.async.slow", &payload);
        }
        let mut best_sat = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                let _ = bus.publish("bench.async.slow", &payload);
            }
            best_sat = best_sat.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }
        println!(
            "BENCH events: publish(no subs) {:.0} ns/op, publish(1 sync sub) {:.0} ns/op, has_subscribers(1 pattern) {:.0} ns/op, has_subscribers(zero subs) {:.0} ns/op, publish(1 async sub) {:.0} ns/op, task build {:.0} ns/op, task build+push {:.0} ns/op, publish(1 async saturated) {:.0} ns/op (min of {rounds}x{iters})",
            best_none * 1e9,
            best_one * 1e9,
            best_glob * 1e9,
            best_empty * 1e9,
            best_async * 1e9,
            best_build * 1e9,
            best_push * 1e9,
            best_sat * 1e9
        );
    }
}



