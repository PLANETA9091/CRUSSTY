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
        let exact = self.exact_find(hash, event);
        // TASK-175 fast path (RESOLVE_SHARED_EXACT A/B toggle): with no
        // patterns registered the dispatch list is EXACTLY the precomputed
        // exact-topic slice (built once at mutation time, TASK-164) — share
        // it with one atomic increment instead of rebuilding a
        // byte-identical Vec + Arc on every call. queue_async bypasses the
        // sync-path memo, so every async publish paid a Vec alloc + Arc
        // inner alloc here plus the worker-side Arc dealloc; a zero match
        // additionally returns a shared static empty list (no Arc-header
        // alloc). The patterned case keeps the full build: mixed
        // exact+pattern concat order is unchanged.
        if RESOLVE_SHARED_EXACT && self.patterns.is_empty() {
            return match exact {
                Some(list) => Arc::clone(list),
                None => empty_resolved(),
            };
        }
        let mut out: Vec<(Option<(Box<str>, u64)>, Handler)> = Vec::new();
        if let Some(list) = exact {
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

/// Shared empty dispatch list: zero-match resolves clone one static Arc
/// instead of building and dropping an Arc header per call (TASK-175).
fn empty_resolved() -> Resolved {
    static EMPTY: OnceLock<Resolved> = OnceLock::new();
    EMPTY.get_or_init(|| Arc::from(Vec::new())).clone()
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
    /// TASK-197: the async resolved list at this record's generation, so a
    /// memo hit enqueues without re-walking the async view (no per-publish
    /// `load_arc` inc/dec pair, no exact-slot probe). Same guarantee class
    /// as `resolved`: resolution is a pure function of the view at `gens`,
    /// and any registry mutation (sync OR async — both bump the shared
    /// combined counter) changes the tag, so a stale list can never verify.
    /// `None` = no async match; `has_async` stays the verbatim any_match
    /// verdict and a None-with-true combination (defensively impossible:
    /// any_match implies a non-empty resolve) falls back to the legacy
    /// re-resolve path, preserving the old behavior bit-for-bit.
    async_list: Option<Resolved>,
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

/// Hot has_subscribers through the global memo (TASK-167) — the same
/// zero-RMW read shape the publish path takes via [`memo_find`]: one
/// acquire load per probe, no TLS, no refcounts.
#[inline]
fn memo_has(bus_key: u64, gens: u64, hash: u64, event: &str) -> Option<bool> {
    let rec = memo_find(bus_key, gens, hash, event)?;
    Some(!rec.resolved.is_empty() || rec.has_async)
}

/// Fill the memo on a cold miss (registry-writer frequency). A record for
/// the exact (bus, gens, event) key is content-identical to any existing
/// one — resolution (sync AND async, TASK-197) is a pure function of the
/// view at `gens` — so a matching record is left in place and repeated
/// cold fills are free.
#[cold]
fn memo_fill(
    bus_key: u64,
    gens: u64,
    hash: u64,
    event: &str,
    resolved: Resolved,
    has_async: bool,
    async_list: &Option<Resolved>,
) {
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
                async_list: async_list.clone(),
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
                // TASK-197: a record without the carried list (has_subscribers
                // fills incomplete — the resolve costs more than the whole
                // probe) is not WRONG, only poorer. A fill that carries a
                // list UPGRADES it (overwrite below); otherwise the record is
                // content-identical — leave in place, repeated fills free.
                if rec.async_list.is_some() || async_list.is_none() {
                    return; // already correct content
                }
                // Prefer clobbering the incomplete record over retiring an
                // unrelated one; nothing further down the chain can be a
                // better target (inserts never chain past an empty slot).
                victim = Some(i);
                break;
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
            async_list: async_list.clone(),
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

/// TASK-174 A/B toggle: when true, an all-unowned async dispatch (no
/// queued handler belongs to a module) pushes an empty leaders Vec —
/// zero heap allocs; when false, the full parallel Vec is collected
/// unconditionally (round-13 baseline). See queue_async for the
/// correctness argument.
const ASYNC_LEADERS_ZEROALLOC: bool = true;

/// TASK-175 A/B toggle: when true, resolve() on a patternless view shares
/// the precomputed exact-topic Arc (one atomic increment) instead of
/// rebuilding an identical Vec + Arc per call; when false, the full build
/// runs unconditionally (round-14 baseline). See RegistryView::resolve.
const RESOLVE_SHARED_EXACT: bool = true;

/// TASK-176 A/B toggle: when true, `publish_shared` hands the caller's
/// `Arc<Value>` straight to the queued task — one atomic increment, zero
/// deep clone; when false, the shared publish path deep-clones the payload
/// exactly like `publish` (round-15 baseline). Handler-visible payloads
/// are byte-identical in both modes: `TaskPayload` derefs to `Value`.
const ASYNC_PAYLOAD_SHARED: bool = true;

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

/// Payload storage for queued async tasks (TASK-176). `publish` deep-clones
/// the caller's borrowed Value (the borrow may die before the task runs);
/// `publish_shared` moves the caller's `Arc<Value>` handle into the task —
/// one atomic increment instead of cloning the map root, every entry node
/// and every String key. Both variants hand handlers the same `&Value`:
/// the enum derefs, so dispatch code and handler signatures are unchanged.
enum TaskPayload {
    Owned(Value),
    Shared(Arc<Value>),
}

impl std::ops::Deref for TaskPayload {
    type Target = Value;
    #[inline]
    fn deref(&self) -> &Value {
        match self {
            TaskPayload::Owned(v) => v,
            TaskPayload::Shared(v) => v,
        }
    }
}

/// One queued unit of async work: the handler snapshot for a single publish.
struct AsyncTask {
    event: InlineEvent,
    payload: TaskPayload,
    /// Phantom guards keep the module mappings alive while their handlers
    /// sit in the queue or run: a reload cannot dlclose a module whose async
    /// handlers are still pending or in flight (active-count protocol).
    leaders: Vec<Option<crate::platform::hot_reload::ModuleGuard>>,
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

/// Quiescence: every queued handler keeps its module's guard alive until
/// the pool has run it (see AsyncTask::leaders), so a hot reload waits
/// instead of dlclosing under pending handlers.
/// TASK-174: the parallel leaders array exists for the reload-safety
/// protocol — run() consults leaders.get_mut(i) only to release a module
/// guard for OWNED handlers (the skip branch requires owner.is_some()), so
/// when no queued handler belongs to a module there is nothing to guard
/// and nothing to release: an out-of-range get_mut yields None exactly
/// like a Some(None) slot. The all-unowned shape (global subscribe_async)
/// then pushes a Vec::new() — zero heap allocs on the build path. The
/// mixed / some-owned shape keeps the full parallel collect: an owned
/// handler whose slot read None would be skipped as mid-reload, so
/// leaders.len()==list.len() must hold whenever any owner exists.
fn build_leaders(
    list: &[(Option<(Box<str>, u64)>, Handler)],
) -> Vec<Option<crate::platform::hot_reload::ModuleGuard>> {
    if ASYNC_LEADERS_ZEROALLOC && !list.iter().any(|(owner, _)| owner.is_some()) {
        Vec::new()
    } else {
        list.iter()
            .map(|(owner, _)| {
                owner
                    .as_ref()
                    .and_then(|(id, _)| crate::platform::hot_reload::guard_module(id))
            })
            .collect()
    }
}

fn dispatch(handlers: &[(Option<(Box<str>, u64)>, Handler)], event: &str, payload: &Value) -> usize {
    for (owner, handler) in handlers.iter() {
        let guard = owner
            .as_ref()
            .and_then(|(id, _)| crate::platform::hot_reload::guard_module(id));
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
        // has_subscribers never resolves the async list (the resolve costs
        // more than this whole probe) — the record is filled INCOMPLETE
        // (async_list=None); a later publish-driven fill upgrades it (see
        // memo_fill). Until then, publish hits on this record take the
        // legacy re-resolve fallback — exactly the pre-197 behavior.
        memo_fill(self.sync.id, gens, hash, event, resolved, has_async, &None);
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

    /// Pre-parse gate for foreign entries (TASK-179): ONE acquire load of
    /// the combined generation counter — the same word [`publish`]'s own
    /// fast gate reads. `false` proves nothing was EVER subscribed on this
    /// bus, so a publish would drop the payload untouched and return 0;
    /// a caller that still has to BUILD or parse its payload may skip that
    /// work entirely. Concurrency contract is identical to the in-publish
    /// gate: a subscriber racing the publish window may or may not observe
    /// the call (no ordering is promised) — the check only shortens the
    /// window; it grants no new guarantee and removes none.
    #[inline]
    pub fn may_have_subscribers(&self) -> bool {
        self.gens.load(Ordering::Acquire) != 0
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
        // resolution path and the async queue below. TASK-197: the hit path
        // enqueues from the record's precomputed async list — the tag check
        // already proves the list matches THIS generation, so no view walk.
        let hash = fnv1a(event.as_bytes());
        if let Some(rec) = memo_find(self.sync.id, gens, hash, event) {
            let invoked = dispatch(&rec.resolved, event, payload);
            if rec.has_async {
                match &rec.async_list {
                    Some(list) => self.queue_async_with(event, payload, Arc::clone(list)),
                    // Defensive (any_match implies non-empty): legacy shape.
                    None => self.queue_async(event, payload, hash),
                }
            }
            return invoked;
        }
        let (resolved, has_async) = self.resolve_event(event, hash, gens >> 32, gens & 0xFFFF_FFFF);
        // Resolve the async list ONCE on the cold path: the memo record
        // stores it (hits clone from there) and this publish's own task
        // consumes the local handle — the queue no longer re-resolves.
        let async_list = if has_async {
            self.async_targets(event, hash)
        } else {
            None
        };
        memo_fill(
            self.sync.id,
            gens,
            hash,
            event,
            Arc::clone(&resolved),
            has_async,
            &async_list,
        );
        let invoked = dispatch(&resolved, event, payload);
        if has_async {
            match async_list {
                Some(list) => self.queue_async_with(event, payload, list),
                None => self.queue_async(event, payload, hash),
            }
        }
        invoked
    }

    /// Publish an event whose payload the caller holds behind an `Arc`
    /// (TASK-176). Identical to [`publish`] in every observable way —
    /// same fast gate, same memo, same sync dispatch — except the async
    /// queue takes the caller's handle instead of deep-cloning the Value
    /// per publish.
    ///
    /// Preferred call shape: build a FRESH payload and MOVE the handle in
    /// (`publish_shared(topic, Arc::new(json!(...)))`) — the task then owns
    /// the value outright (count 1) and no counter line is ever contended.
    /// Re-cloning one long-lived handle per call
    /// (`publish_shared(topic, Arc::clone(&shared))`) still works, but the
    /// publisher's inc and the worker's dec then bounce the same count line
    /// across cores every publish — measurably worse than a fresh build.
    pub fn publish_shared(&self, event: &str, payload: Arc<Value>) -> usize {
        // Lock-free fast gate (TASK-164): combined generation 0 = nothing
        // ever subscribed — ONE acquire load, no locks.
        let gens = self.gens.load(Ordering::Acquire);
        if gens == 0 {
            return 0;
        }
        // Global memo hit (TASK-167) — same shape as publish; the hash is
        // computed once and reused by the cold resolution path and the
        // async queue below. TASK-197: the hit path enqueues from the
        // record's precomputed async list (tag-checked against THIS gens).
        let hash = fnv1a(event.as_bytes());
        if let Some(rec) = memo_find(self.sync.id, gens, hash, event) {
            let invoked = dispatch(&rec.resolved, event, &payload);
            if rec.has_async {
                match &rec.async_list {
                    Some(list) => {
                        self.queue_async_shared_with(event, payload, Arc::clone(list))
                    }
                    // Defensive (any_match implies non-empty): legacy shape.
                    None => self.queue_async_shared(event, payload, hash),
                }
            }
            return invoked;
        }
        let (resolved, has_async) = self.resolve_event(event, hash, gens >> 32, gens & 0xFFFF_FFFF);
        // Cold path: one async resolve for both the memo record and this
        // publish's own task (same generation snapshot — the sync dispatch
        // and the async task now share one coherent view state).
        let async_list = if has_async {
            self.async_targets(event, hash)
        } else {
            None
        };
        memo_fill(
            self.sync.id,
            gens,
            hash,
            event,
            Arc::clone(&resolved),
            has_async,
            &async_list,
        );
        let invoked = dispatch(&resolved, event, &payload);
        if has_async {
            match async_list {
                Some(list) => self.queue_async_shared_with(event, payload, list),
                None => self.queue_async_shared(event, payload, hash),
            }
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
        let Some(list) = self.async_targets(event, hash) else {
            return;
        };
        self.queue_async_with(event, payload, list);
    }

    /// Queue with the resolved list already in hand (TASK-197). The task
    /// shape is byte-identical to the resolve-inside shape — the only
    /// difference is WHERE the list came from (memo record vs view walk).
    /// The borrowed payload may die before the task runs: deep-clone it
    /// into the task (publish has no handle to share).
    fn queue_async_with(&self, event: &str, payload: &Value, list: Resolved) {
        self.pool.push(AsyncTask {
            event: event.into(),
            payload: TaskPayload::Owned(payload.clone()),
            leaders: build_leaders(&list),
            handlers: list,
        });
    }

    /// Queue the async handlers for one shared-arc publish (TASK-176): the
    /// caller hands over its `Arc<Value>` handle, so the task stores the
    /// payload without any deep clone when ASYNC_PAYLOAD_SHARED is on —
    /// one atomic increment instead of cloning the map root, every entry
    /// node and every String key. The OFF mode deep-clones exactly like
    /// queue_async (round-15 baseline), which keeps the A/B honest about
    /// the enqueue representation and nothing else.
    fn queue_async_shared(&self, event: &str, payload: Arc<Value>, hash: u64) {
        let Some(list) = self.async_targets(event, hash) else {
            return;
        };
        self.queue_async_shared_with(event, payload, list);
    }

    /// Shared-arc enqueue with the resolved list already in hand
    /// (TASK-197). Identical to queue_async_shared minus the re-resolve:
    /// the caller's `Arc<Value>` handle moves into the task (zero deep
    /// clone when ASYNC_PAYLOAD_SHARED is on), the list Arc is consumed
    /// outright (the one inherent inc happened at the clone site).
    fn queue_async_shared_with(&self, event: &str, payload: Arc<Value>, list: Resolved) {
        let payload = if ASYNC_PAYLOAD_SHARED {
            TaskPayload::Shared(payload)
        } else {
            TaskPayload::Owned((*payload).clone())
        };
        self.pool.push(AsyncTask {
            event: event.into(),
            payload,
            leaders: build_leaders(&list),
            handlers: list,
        });
    }

    /// Resolve the async handler snapshot for one publish; `None` = nothing
    /// to queue. Shared by the borrowed (queue_async) and shared-arc
    /// (queue_async_shared) enqueue paths — identical semantics, callers
    /// differ only in how the payload reaches the task.
    fn async_targets(&self, event: &str, hash: u64) -> Option<Resolved> {
        self.async_cell
            .view
            .load_arc()
            .map(|v| v.resolve(hash, event))
            .filter(|l| !l.is_empty())
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

    /// TASK-197 parity: the memo-carried async list must dispatch exactly
    /// like the view-walk path — one task per publish across the cold-fill
    /// boundary and every memo hit, for both publish shapes; unsubscribe
    /// bumps the shared combined generation so the stale record (and its
    /// carried list) can never verify, and the sync return count stays 0
    /// (async handlers report through the pool, not the return value).
    #[test]
    fn async_memo_hit_dispatch_parity() {
        let bus = with_cap(64);
        let n = Arc::new(AtomicUsize::new(0));
        let n2 = Arc::clone(&n);
        let tok = bus.subscribe_async("memo.parity", Arc::new(move |_, _| {
            n2.fetch_add(1, Ordering::SeqCst);
        }));
        let payload = serde_json::json!({ "k": 1u64 });
        // 1 cold fill + 9 memo hits through the legacy publish shape.
        for _ in 0..10 {
            assert_eq!(bus.publish("memo.parity", &payload), 0);
        }
        assert!(
            wait_until(|| n.load(Ordering::SeqCst) == 10),
            "async handler must fire once per publish across the memo boundary"
        );
        // The record must actually CARRY the list (the hit path's subject).
        let hash = fnv1a(b"memo.parity");
        let gens = bus.gens.load(Ordering::Acquire);
        let rec = memo_find(bus.sync.id, gens, hash, "memo.parity")
            .expect("memo record after publish");
        assert!(rec.has_async, "async sub must mark the record");
        assert!(
            rec.async_list.is_some(),
            "TASK-197: record must carry the async resolved list"
        );
        // Same shape through publish_shared (the hot-path caller).
        for _ in 0..10 {
            assert_eq!(
                bus.publish_shared("memo.parity", Arc::new(payload.clone())),
                0
            );
        }
        assert!(
            wait_until(|| n.load(Ordering::SeqCst) == 20),
            "publish_shared memo hits must queue from the carried list too"
        );
        // Unsubscribe bumps gens: the stale record can never verify; the
        // next publish takes the cold path with has_async=false — no more
        // tasks, and a fresh record (if slotted) carries no async list.
        assert!(bus.unsubscribe("memo.parity", &tok));
        let gens_before = gens;
        assert_eq!(bus.publish("memo.parity", &payload), 0);
        let gens_after = bus.gens.load(Ordering::Acquire);
        assert_ne!(gens_after, gens_before, "unsubscribe must bump gens");
        thread::sleep(Duration::from_millis(20));
        assert_eq!(
            n.load(Ordering::SeqCst),
            20,
            "no async dispatch after unsubscribe"
        );
        if let Some(r) = memo_find(bus.sync.id, gens_after, hash, "memo.parity") {
            assert!(
                !r.has_async && r.async_list.is_none(),
                "post-unsubscribe record must be async-free"
            );
        }
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
    fn publish_shared_matches_publish_visibility() {
        // TASK-176: publish_shared must be observationally identical to
        // publish — same sync return value, byte-exact handler-visible
        // payloads on the async path, both via the shared Arc handle.
        let bus = EventBus::default();

        // sync path: same count as publish
        let n = Arc::new(AtomicUsize::new(0));
        let n2 = Arc::clone(&n);
        bus.subscribe("sh.sync", Arc::new(move |_, payload| {
            n2.fetch_add(1, Ordering::SeqCst);
            assert_eq!(payload["tick"].as_u64(), Some(7), "sync payload via publish_shared");
        }));
        assert_eq!(bus.publish_shared("sh.sync", Arc::new(serde_json::json!({ "tick": 7u64 }))), 1);
        assert_eq!(n.load(Ordering::SeqCst), 1);

        // async path: the queued task derefs to the same payload
        let seen = Arc::new(Mutex::new(Vec::new()));
        let s = Arc::clone(&seen);
        bus.subscribe_async("sh.async", Arc::new(move |event, payload| {
            s.lock().unwrap().push((event.to_string(), payload["tick"].as_u64().unwrap()));
        }));
        bus.publish_shared("sh.async", Arc::new(serde_json::json!({ "tick": 41u64 })));
        assert!(
            wait_until(|| seen.lock().unwrap().len() == 1),
            "publish_shared async handler never ran"
        );
        assert_eq!(seen.lock().unwrap()[0], ("sh.async".to_string(), 41u64));

        // zero subscribers: same gate as publish (0 invoked, nothing queued)
        assert_eq!(bus.publish_shared("sh.none", Arc::new(serde_json::json!(null))), 0);
        assert_eq!(bus.async_pending(), 0);
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
    use std::time::{Duration, Instant};

    /// TASK-210 regime instrumentation: the three async/enqueue arms
    /// (publish(1 async sub), task build+push, publish(1 async saturated))
    /// print their regime inline — shed count over the timed rounds and
    /// worst-of-rounds spread — ADDED ALONGSIDE unchanged headline
    /// statistics (same warmups, same sample counts, same min-of
    /// methodology: historical bands stay comparable). Motivating study
    /// (3 fresh-process solo runs on unmodified HEAD): the deterministic
    /// arms of this bench reproduce exactly (no subs / 1 sync / glob /
    /// zero-subs 2/9/7/2 ns flat, task build 90-91 = 1.1%) while the
    /// async arms are the TASK-208 chaotic regime printing raw scalars —
    /// publish(1 async sub) 617-956 = 55% spread, build+push 460-528 =
    /// 15%, saturated 503-644 = 28%. The sibling benches already quote
    /// their regime (TASK-208 enqueue-iso, Round-38 pattern); this bench
    /// was the last headline surface printing bare scalars.
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
        // TASK-210: regime fields for the distributional arms — the shed
        // count over the timed rounds and worst-of-rounds travel inline
        // with the unchanged min (same warmups, same rounds, same min-of
        // methodology: historical bands stay comparable; the ledger quotes
        // the regime, not a fake scalar — TASK-208 finding).
        let sheds_before = bus.pool.dropped.load(Ordering::SeqCst);
        let mut worst_async = f64::MIN;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                let _ = bus.publish("bench.async", &payload);
            }
            let per = start.elapsed().as_secs_f64() / f64::from(iters);
            best_async = best_async.min(per);
            worst_async = worst_async.max(per);
        }
        let sheds_async = bus.pool.dropped.load(Ordering::SeqCst) - sheds_before;
        // pool.push isolated from queue_async (TASK-169): (a) build-only
        // per-op cost of one AsyncTask (to_string + Value clone + allocs),
        // (b) build + push — the delta is the enqueue critical section
        // (Mutex + push_back + condvar notify, including the worker wake).
        let pool = Arc::clone(&bus.pool);
        let build_task = || AsyncTask {
            event: "bench.async".into(),
            payload: TaskPayload::Owned(payload.clone()),
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
        let sheds_before = pool.dropped.load(Ordering::SeqCst);
        let mut worst_push = f64::MIN;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                pool.push(build_task());
            }
            let per = start.elapsed().as_secs_f64() / f64::from(iters);
            best_push = best_push.min(per);
            worst_push = worst_push.max(per);
        }
        let sheds_push = pool.dropped.load(Ordering::SeqCst) - sheds_before;
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
        let sheds_before = pool.dropped.load(Ordering::SeqCst);
        let mut worst_sat = f64::MIN;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                let _ = bus.publish("bench.async.slow", &payload);
            }
            let per = start.elapsed().as_secs_f64() / f64::from(iters);
            best_sat = best_sat.min(per);
            worst_sat = worst_sat.max(per);
        }
        let sheds_sat = pool.dropped.load(Ordering::SeqCst) - sheds_before;
        println!(
            "BENCH events: publish(no subs) {:.0} ns/op, publish(1 sync sub) {:.0} ns/op, has_subscribers(1 pattern) {:.0} ns/op, has_subscribers(zero subs) {:.0} ns/op, publish(1 async sub) {:.0} ns/op (sheds {}, rounds spread {:.2}x worst {:.0}), task build {:.0} ns/op, task build+push {:.0} ns/op (sheds {}, rounds spread {:.2}x worst {:.0}), publish(1 async saturated) {:.0} ns/op (sheds {}, rounds spread {:.2}x worst {:.0}) (min of {rounds}x{iters})",
            best_none * 1e9,
            best_one * 1e9,
            best_glob * 1e9,
            best_empty * 1e9,
            best_async * 1e9,
            sheds_async,
            worst_async / best_async,
            worst_async * 1e9,
            best_build * 1e9,
            best_push * 1e9,
            sheds_push,
            worst_push / best_push,
            worst_push * 1e9,
            best_sat * 1e9,
            sheds_sat,
            worst_sat / best_sat,
            worst_sat * 1e9
        );
    }

    /// TASK-176 A/B: the shared-arc enqueue path. The integral subject is
    /// publish_shared(1 async sub) in the PRODUCTION call shape (scheduler
    /// tick boundary): a FRESH Arc<Value> is built per publish and MOVED
    /// into the call, so the task takes sole ownership (count 1) — under
    /// ASYNC_PAYLOAD_SHARED=true the queue stores that handle with zero
    /// deep clone; under false it deep-clones exactly like publish
    /// (round-15 baseline). Cloning one long-lived handle per call instead
    /// would put the Arc counter on a cross-core contended line (inc on
    /// the publisher, dec on the worker) — a shape production callers do
    /// not produce and the A/B must not measure.
    /// The build(shared) isolate is toggle-independent (it constructs the
    /// Shared variant directly), so it doubles as a cross-build noise
    /// check against the owned task build line in bench_event_bus_publish.
    /// TASK-210 regime instrumentation: publish_shared(1 async sub) — the
    /// one distributional arm of this bench — prints its regime inline
    /// (shed count over the timed rounds + worst-of-rounds spread) beside
    /// the unchanged min; the deterministic arms (sync 19 ns, build(shared)
    /// 34 ns — 0% across 3 fresh solo runs on unmodified HEAD) stay bare.
    /// The async arm measured 785-947 ns = 21% spread across the same 3
    /// runs while printing a raw scalar — the Round-38 pattern extended to
    /// the last headline surface (headline methodology untouched).
    #[test]
    #[ignore]
    fn bench_event_publish_shared() {
        let bus = with_cap(64);
        let payload = serde_json::json!({ "tick": 1u64, "drained": 0u64 });
        let payload_arc = Arc::new(payload.clone());
        let iters = 200_000u32;
        let rounds = 5;

        // one sync subscriber — publish_shared's sync path must sit at the
        // publish(1 sync sub) floor (dispatch + memo, no queue work); the
        // handle clone here isolates the bus path from payload construction
        let _stok = bus.subscribe("bench.shared.sync", Arc::new(|_, _| {}));
        for _ in 0..10_000u32 {
            let _ = bus.publish_shared("bench.shared.sync", Arc::clone(&payload_arc));
        }
        let mut best_sync = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                let _ = bus.publish_shared("bench.shared.sync", Arc::clone(&payload_arc));
            }
            best_sync = best_sync.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }

        // one async subscriber — the A/B subject line, production shape:
        // fresh payload per publish, handle MOVED into the call
        let _atok = bus.subscribe_async("bench.shared", Arc::new(|_, _| {}));
        for _ in 0..10_000u32 {
            let _ = bus.publish_shared(
                "bench.shared",
                Arc::new(serde_json::json!({ "tick": 1u64, "drained": 0u64 })),
            );
        }
        let mut best_shared = f64::MAX;
        // TASK-210: regime fields for the distributional arm (same pattern
        // as the sibling benches — TASK-208/TASK-210: the ledger quotes the
        // regime, the min stays methodology-identical).
        let sheds_before = bus.pool.dropped.load(Ordering::SeqCst);
        let mut worst_shared = f64::MIN;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                let _ = bus.publish_shared(
                    "bench.shared",
                    Arc::new(serde_json::json!({ "tick": 1u64, "drained": 0u64 })),
                );
            }
            let per = start.elapsed().as_secs_f64() / f64::from(iters);
            best_shared = best_shared.min(per);
            worst_shared = worst_shared.max(per);
        }
        let sheds_shared = bus.pool.dropped.load(Ordering::SeqCst) - sheds_before;

        // build isolate: shared-arc payload (one atomic inc) vs the owned
        // deep-clone build line in bench_event_bus_publish; the handlers
        // form (Arc::from(Vec::new())) matches the owned isolate so the
        // ONLY delta between the two build lines is the payload repr.
        let build_shared = || AsyncTask {
            event: "bench.shared".into(),
            payload: TaskPayload::Shared(Arc::clone(&payload_arc)),
            leaders: Vec::new(),
            handlers: Arc::from(Vec::new()),
        };
        for _ in 0..10_000u32 {
            drop(build_shared());
        }
        let mut best_build = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                drop(build_shared());
            }
            best_build = best_build.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }
        println!(
            "BENCH publish_shared: publish_shared(1 sync sub) {:.0} ns/op, publish_shared(1 async sub) {:.0} ns/op (sheds {}, rounds spread {:.2}x worst {:.0}), task build(shared) {:.0} ns/op (min of {rounds}x{iters})",
            best_sync * 1e9,
            best_shared * 1e9,
            sheds_shared,
            worst_shared / best_shared,
            worst_shared * 1e9,
            best_build * 1e9
        );
    }

    /// TASK-205 iso decomposition: publish_shared(1 async sub) 540-632 ns —
    /// the last ledger line never split into build / push / wake terms.
    /// The headline shape (bench_event_publish_shared) times the WHOLE
    /// production call: a fresh `Arc<Value>` per publish (scheduler.rs:396
    /// builds one per tick boundary) MOVED into publish_shared, which
    /// memo-hits, sync-dispatches nothing, builds the AsyncTask and pushes
    /// it into the pool where a worker drains it. This bench splits those
    /// terms. All loop arms are context-independent (local pools/buses,
    /// min of 5x200k, one run); the notify reference lines are
    /// min-of-samples — a futex wake cannot be driven at a 200k rate
    /// against a worker that must re-park between samples.
    ///   framing_sync  — publish_shared(1 sync sub): the whole non-queue
    ///                   part through the same memo/dispatch machinery
    ///                   (the handle-clone shape of the sibling bench)
    ///   payload_build — the caller-side term: fresh Arc<Value> per
    ///                   publish, no bus involved
    ///   enq_e2e       — real pool.push at the headline composition (fresh
    ///                   payload + task build + push, workers active,
    ///                   back-to-back SATURATED regime). REGIME FINDING
    ///                   banked by this bench: on the 2-core box the
    ///                   publisher and the workers are CPU-saturating
    ///                   threads competing for both cores — the queue
    ///                   reaches the cap, the load-shed path runs, and the
    ///                   line is distributional by construction (the
    ///                   mechanism behind the ledger's "wake-dominated
    ///                   noisy class"). Zero-shed is NOT asserted; the
    ///                   shed count is printed instead.
    ///   enq_prebuilt  — ANTI-shape reference: one long-lived handle
    ///                   cloned per push — the cross-core counter line the
    ///                   TASK-176 doc warns about, quantified (same
    ///                   saturated regime, shed count printed)
    ///   enq_cadence   — the PRODUCTION-cadence arm: one task in flight,
    ///                   workers parked before every push (the crossing
    ///                   detector fires the real notify), per-iteration
    ///                   latency push -> drained -> re-parked sampled min
    ///                   and median — the per-publish cost at tick
    ///                   cadence including the futex wake + schedule
    ///   enq_serial    — single-thread replica of push+pop+run+drop: the
    ///                   total per-task WORK with zero cross-core
    ///                   coordination (replica skips ensure_workers, so
    ///                   no worker exists; idle stays 0 and the notify
    ///                   gate is the verbatim-off branch)
    ///   worker_drain  — drain-bound per-task cost: 16384 tasks
    ///                   replica-prefilled while both workers are parked
    ///                   (replica pushes never notify), then ONE manual
    ///                   notify_one wakes a single worker and the serial
    ///                   drain is timed end-to-end
    ///   notify lines  — the wake term: notify_one against a condvar
    ///                   nobody ever waited on vs against a registered-
    ///                   idle worker (the futex wake the TASK-170 doc
    ///                   prices at 0.3-3.5us — confirmed or refuted with
    ///                   numbers; the no-waiter line includes two
    ///                   Instant reads ~40ns and is overhead-bound if the
    ///                   glibc fast path skips the syscall)
    /// TASK-208 reproducibility pass: a 5-fresh-process study on
    /// unmodified HEAD banked the run-to-run truth — deterministic arms
    /// are stable to <5% (framing 0%, payload 1.7%, serial 4.7%, notify
    /// no-waiter 0.8%) while the distributional arms are a CHAOTIC
    /// regime, not a noisy scalar (e2e min 565-754 ns = 33% spread,
    /// prebuilt 525-764 = 45% with the shed count itself random-walking
    /// 108k-337k, cadence min 650-971 / med 3935-5676, parked-notify min
    /// 143-358 = 150%): the shed rate is an emergent oscillator state,
    /// so scalar reproducibility is structurally impossible. The
    /// hardening is INLINE REGIME INSTRUMENTATION — worst-of-rounds
    /// spread for the enqueue arms, p10/p90 for cadence, worst-of-3 for
    /// drain, med for parked — ADDED ALONGSIDE unchanged headline
    /// statistics (same warmups, same sample counts, same min-of
    /// methodology: historical bands stay comparable; the ledger quotes
    /// the regime, not a fake scalar). A pre-registered WARMUP BURN-IN
    /// lever (10k -> 60k pre-timing pushes) was probed and REFUTED with
    /// numbers: e2e spread 29% vs 33% baseline (noise), prebuilt 73%,
    /// worker_drain 76% (worse), band centers drifted +30-50 ns — the
    /// regime is a sustained oscillator, not a startup-phase artifact;
    /// the warmup stays 10k.
    #[test]
    #[ignore]
    fn bench_publish_shared_enqueue_iso() {
        // Byte-identical to the headline arm's event (Inline variant, 12B
        // <= INLINE_EVENT_CAP) — no repr drift between the shapes.
        const EVENT: &str = "bench.shared";
        let iters = 200_000u32;
        let rounds = 5;
        let mut fold = 0u64; // observability guard against elision

        // TASK-208: returns (name, best, worst) — worst is the max
        // per-round average, reported for the DISTRIBUTIONAL arms so the
        // regime spread travels inline with the headline (which stays
        // methodology-identical: min of {rounds}x{iters}).
        fn time_arm(
            name: &'static str,
            mut op: impl FnMut() -> u64,
            fold: &mut u64,
            iters: u32,
            rounds: u32,
        ) -> (&'static str, f64, f64) {
            for _ in 0..10_000u32 {
                *fold = fold.wrapping_add(op());
            }
            let mut best = f64::MAX;
            let mut worst = f64::MIN;
            for _ in 0..rounds {
                let start = Instant::now();
                for _ in 0..iters {
                    *fold = fold.wrapping_add(op());
                }
                let per = start.elapsed().as_secs_f64() / f64::from(iters);
                best = best.min(per);
                worst = worst.max(per);
            }
            (name, best * 1e9, worst * 1e9)
        }

        // framing_sync — the non-queue part through the memo machinery.
        let bus = with_cap(64);
        let _stok = bus.subscribe("bench.iso.frame", Arc::new(|_, _| {}));
        let frame_arc = Arc::new(serde_json::json!({ "tick": 1u64, "drained": 0u64 }));
        for _ in 0..10_000u32 {
            let _ = bus.publish_shared("bench.iso.frame", Arc::clone(&frame_arc));
        }
        let framing = time_arm(
            "framing publish_shared(1 sync sub)",
            || {
                let _ = bus.publish_shared("bench.iso.frame", Arc::clone(&frame_arc));
                0
            },
            &mut fold,
            iters,
            rounds,
        );

        // The production task shape, built fresh per push — verbatim with
        // what queue_async_shared_with assembles for the scheduler call
        // (owner-less list => build_leaders is the zeroalloc Vec::new).
        let iso_handler: Handler = Arc::new(|_: &str, _: &Value| {});
        let iso_list: Resolved = Arc::from(vec![(None, iso_handler)]);
        let mk_task = |payload: Arc<Value>| AsyncTask {
            event: EVENT.into(),
            payload: TaskPayload::Shared(payload),
            leaders: build_leaders(&iso_list),
            handlers: Arc::clone(&iso_list),
        };

        // payload_build — the caller-side term (scheduler.rs:396 shape).
        let payload_build = time_arm(
            "payload build (fresh Arc<Value>)",
            || {
                drop(Arc::new(serde_json::json!({ "tick": 1u64, "drained": 0u64 })));
                0
            },
            &mut fold,
            iters,
            rounds,
        );

        // enq_e2e — real pool.push, workers active, headline composition.
        let pool = Arc::new(AsyncPool::new(64));
        for _ in 0..10_000u32 {
            pool.push(mk_task(Arc::new(serde_json::json!({ "tick": 1u64, "drained": 0u64 }))));
        }
        let sheds_before = pool.dropped.load(Ordering::SeqCst);
        let enq_e2e = time_arm(
            "enqueue e2e (real push, workers active)",
            || {
                pool.push(mk_task(Arc::new(serde_json::json!({ "tick": 1u64, "drained": 0u64 }))));
                0
            },
            &mut fold,
            iters,
            rounds,
        );
        let sheds_e2e = pool.dropped.load(Ordering::SeqCst) - sheds_before;

        // enq_prebuilt — the TASK-176 anti-shape, quantified: the same
        // pool, the same workers, but the payload handle is ONE long-lived
        // Arc cloned per push (publisher-side inc, worker-side dec on the
        // same counter line).
        let prebuilt = Arc::new(serde_json::json!({ "tick": 1u64, "drained": 0u64 }));
        for _ in 0..10_000u32 {
            pool.push(mk_task(Arc::clone(&prebuilt)));
        }
        let sheds_before = pool.dropped.load(Ordering::SeqCst);
        let enq_prebuilt = time_arm(
            "enqueue prebuilt handle (anti-shape)",
            || {
                pool.push(mk_task(Arc::clone(&prebuilt)));
                0
            },
            &mut fold,
            iters,
            rounds,
        );
        let sheds_prebuilt = pool.dropped.load(Ordering::SeqCst) - sheds_before;

        // enq_serial — the push body VERBATIM (shed branch included though
        // unreachable at depth <= 1; idle == 0 keeps the notify gate off)
        // with ensure_workers skipped, plus the worker-side pop + run + drop
        // serialized onto the same thread: the total per-task work with
        // zero cross-core coordination. `gate=false` mutes ONLY the notify
        // branch — used by the drain refill below, where idle == 2 would
        // make the verbatim crossing fire on the FIRST refill push
        // (len == 1 == 2 - 2 + 1) and let a worker steal tasks mid-fill.
        #[inline(never)]
        fn push_replica(pool: &AsyncPool, task: AsyncTask, gate: bool) {
            let mut queue = lock(&pool.queue);
            if queue.len() >= pool.cap {
                queue.pop_front();
                pool.dropped.fetch_add(1, Ordering::SeqCst);
                if !pool.dropping.swap(true, Ordering::SeqCst) {
                    eprintln!(
                        "[crussty:events] async dispatch queue at capacity ({}) — dropping oldest pending events",
                        pool.cap
                    );
                }
            }
            queue.push_back(task);
            if gate {
                let idle = pool.idle.load(Ordering::SeqCst);
                if idle > 0 && queue.len() == ASYNC_WORKERS - idle + 1 {
                    pool.condvar.notify_one();
                }
            }
        }
        let ser_pool = AsyncPool::new(64);
        let ser_prebuilt = Arc::new(serde_json::json!({ "tick": 1u64, "drained": 0u64 }));
        for _ in 0..10_000u32 {
            push_replica(&ser_pool, mk_task(Arc::clone(&ser_prebuilt)), true);
            let t = ser_pool.pop();
            ser_pool.run(t);
        }
        let enq_serial = time_arm(
            "enqueue serial (push+pop+run+drop, one thread)",
            || {
                push_replica(&ser_pool, mk_task(Arc::clone(&ser_prebuilt)), true);
                let t = ser_pool.pop();
                ser_pool.run(t);
                0
            },
            &mut fold,
            iters,
            rounds,
        );

        // worker_drain — drain-bound per-task cost, one worker, deep queue,
        // zero spawn pollution and zero push interference: the workers are
        // spawned by one real push and left to park; every round replica-
        // prefills the queue while idle == ASYNC_WORKERS (replica never
        // notifies), then ONE manual notify_one wakes exactly one worker
        // and the drain to empty is timed. The divisor is the length
        // observed at the notify instant, so a stray steal cannot skew the
        // number.
        const DRAIN_TASKS: usize = 16384;
        let drain_pool = Arc::new(AsyncPool::new(DRAIN_TASKS * 2));
        drain_pool.push(mk_task(Arc::new(serde_json::json!({ "tick": 1u64, "drained": 0u64 }))));
        let parked_deadline = Instant::now() + Duration::from_secs(5);
        while drain_pool.idle.load(Ordering::SeqCst) != ASYNC_WORKERS {
            assert!(
                Instant::now() < parked_deadline,
                "drain workers must reach the parked state before the refill"
            );
            std::hint::spin_loop();
        }
        let mut best_drain = f64::MAX;
        let mut worst_drain = f64::MIN;
        let mut drain_len = DRAIN_TASKS;
        for _ in 0..3 {
            while drain_pool.idle.load(Ordering::SeqCst) != ASYNC_WORKERS {
                std::hint::spin_loop();
            }
            for _ in 0..DRAIN_TASKS {
                push_replica(
                    &drain_pool,
                    mk_task(Arc::new(serde_json::json!({ "tick": 1u64, "drained": 0u64 }))),
                    false,
                );
            }
            let start_len = lock(&drain_pool.queue).len();
            let t0 = Instant::now();
            // The manual wake MUST be issued under the queue lock — the
            // exact discipline the real push uses. A bare notify can land
            // inside the worker's fetch_add -> wait-commit window (the
            // worker is idle-counted but not yet futex-registered), be
            // lost, and leave the worker asleep on a full queue while this
            // loop reads idle == ASYNC_WORKERS instantly (observed as a
            // ~0 ns/task artifact pre-fix). Under the lock the window
            // cannot overlap: the park path holds the lock from fetch_add
            // until wait() releases it, so a lock-held notify always finds
            // a registered waiter (or an awake worker that will re-check).
            {
                let _q = lock(&drain_pool.queue);
                drain_pool.condvar.notify_one();
            }
            // Drain-complete signal WITHOUT touching the queue lock, in TWO
            // phases: idle == ASYNC_WORKERS is AMBIGUOUS (both parked OR
            // the woken worker not yet scheduled to run its fetch_sub —
            // observed as a ~4us "drain" of 16384 tasks). Phase 1 waits for
            // the dip (the worker's fetch_sub proves it actually woke);
            // phase 2 waits for the return (it parks only after finding
            // the queue empty under the lock — the drain is done). A
            // try_lock poller here would steal probes from the worker and
            // inflate the line; the atomic poll costs it nothing. The one
            // spin-window the worker runs before re-parking is a fixed
            // ~us tail over 16384 tasks — noise at ns/task scale.
            while drain_pool.idle.load(Ordering::SeqCst) == ASYNC_WORKERS {
                std::hint::spin_loop();
            }
            while drain_pool.idle.load(Ordering::SeqCst) != ASYNC_WORKERS {
                std::hint::spin_loop();
            }
            let per = t0.elapsed().as_secs_f64() * 1e9 / start_len.max(1) as f64;
            if per < best_drain {
                best_drain = per;
                drain_len = start_len;
            }
            worst_drain = worst_drain.max(per);
        }

        // enq_cadence — the production-cadence arm: exactly one task in
        // flight, both workers parked before every push, so THIS push is
        // the crossing (len == ASYNC_WORKERS - idle + 1) and fires the
        // real notify; the sample covers push + futex wake + scheduler
        // dispatch + drain + the worker's spin-window re-park. This is
        // the per-publish cost a tick-boundary caller actually pays.
        let cad_pool = Arc::new(AsyncPool::new(64));
        cad_pool.push(mk_task(Arc::new(serde_json::json!({ "tick": 1u64, "drained": 0u64 }))));
        let cad_deadline = Instant::now() + Duration::from_secs(5);
        while cad_pool.idle.load(Ordering::SeqCst) != ASYNC_WORKERS {
            assert!(
                Instant::now() < cad_deadline,
                "cadence workers must reach the parked state"
            );
            std::hint::spin_loop();
        }
        let cad_iters = 2000usize;
        let mut cad_samples = Vec::with_capacity(cad_iters);
        for _ in 0..cad_iters {
            let deadline = Instant::now() + Duration::from_secs(5);
            while cad_pool.idle.load(Ordering::SeqCst) != ASYNC_WORKERS {
                assert!(
                    Instant::now() < deadline,
                    "cadence worker must re-park between iterations"
                );
                std::hint::spin_loop();
            }
            let t0 = Instant::now();
            cad_pool.push(mk_task(Arc::new(serde_json::json!({ "tick": 1u64, "drained": 0u64 }))));
            loop {
                let mut settled = cad_pool.idle.load(Ordering::SeqCst) == ASYNC_WORKERS;
                if settled {
                    if let Ok(queue) = cad_pool.queue.try_lock() {
                        settled = queue.is_empty();
                    }
                }
                if settled {
                    break;
                }
                std::hint::spin_loop();
            }
            cad_samples.push(t0.elapsed().as_secs_f64() * 1e9);
        }
        cad_samples.sort_by(|a, b| a.partial_cmp(b).expect("no NaN in elapsed"));
        let cad_min = cad_samples[0];
        let cad_med = cad_samples[cad_iters / 2];
        // TASK-208: the 5-fresh-process study showed the min/med pair
        // alone reads as a stable scalar while the whole distribution
        // shifts per process (med 3935-5676 ns across 5 runs) — p10/p90
        // are printed so the regime SHAPE travels with the line.
        let cad_p10 = cad_samples[cad_iters / 10];
        let cad_p90 = cad_samples[cad_iters * 9 / 10];

        // notify lines — the wake term, min of samples. no_waiter: a
        // condvar nobody ever waited on (glibc __wrefs fast path?).
        // parked: one registered-idle worker re-parked before every sample
        // (the sample is the futex(WAKE) syscall on the publisher side).
        let bare_pool = AsyncPool::new(64);
        let mut best_no_waiter = f64::MAX;
        for _ in 0..4096usize {
            let t0 = Instant::now();
            bare_pool.condvar.notify_one();
            let ns = t0.elapsed().as_secs_f64() * 1e9;
            if ns < best_no_waiter {
                best_no_waiter = ns;
            }
        }
        let np_pool = Arc::new(AsyncPool::new(64));
        np_pool.push(mk_task(Arc::new(serde_json::json!({ "tick": 1u64, "drained": 0u64 }))));
        let np_deadline = Instant::now() + Duration::from_secs(5);
        while np_pool.idle.load(Ordering::SeqCst) != ASYNC_WORKERS {
            assert!(
                Instant::now() < np_deadline,
                "notify workers must reach the parked state"
            );
            std::hint::spin_loop();
        }
        let mut park_samples = Vec::with_capacity(256);
        for _ in 0..256usize {
            let deadline = Instant::now() + Duration::from_secs(5);
            while np_pool.idle.load(Ordering::SeqCst) != ASYNC_WORKERS {
                assert!(
                    Instant::now() < deadline,
                    "worker must re-park between notify samples"
                );
                std::hint::spin_loop();
            }
            let t0 = Instant::now();
            // Lock-held notify — the production pusher shape (the crossing
            // decision runs under the queue lock; a bare notify could be
            // lost in the fetch_add -> wait-commit window).
            {
                let _q = lock(&np_pool.queue);
                np_pool.condvar.notify_one();
            }
            let ns = t0.elapsed().as_secs_f64() * 1e9;
            park_samples.push(ns);
        }
        park_samples.sort_by(|a, b| a.partial_cmp(b).expect("no NaN in elapsed"));
        let best_parked = park_samples[0];
        let med_parked = park_samples[128];

        println!(
            "BENCH publish_shared_enqueue_iso: [{}] {:.0} ns/op, [{}] {:.0} ns/op, [{}] {:.0} ns/op (sheds {}, rounds spread {:.2}x worst {:.0}), [{}] {:.0} ns/op (sheds {}, rounds spread {:.2}x worst {:.0}), [{}] {:.0} ns/op; worker_drain {:.0} ns/task min (worst-of-3 {:.0}; {} tasks, single worker); cadence push->drained min {:.0} / med {:.0} / p10 {:.0} / p90 {:.0} ns ({} iters); notify no-waiter {:.0} ns / parked min {:.0} / med {:.0} ns (min/med of samples); fold {} (loops min of {rounds}x{iters})",
            framing.0,
            framing.1,
            payload_build.0,
            payload_build.1,
            enq_e2e.0,
            enq_e2e.1,
            sheds_e2e,
            enq_e2e.2 / enq_e2e.1,
            enq_e2e.2,
            enq_prebuilt.0,
            enq_prebuilt.1,
            sheds_prebuilt,
            enq_prebuilt.2 / enq_prebuilt.1,
            enq_prebuilt.2,
            enq_serial.0,
            enq_serial.1,
            best_drain,
            worst_drain,
            drain_len,
            cad_min,
            cad_med,
            cad_p10,
            cad_p90,
            cad_iters,
            best_no_waiter,
            best_parked,
            med_parked,
            fold,
        );
        assert!(fold != u64::MAX);
    }

    /// TASK-196 A/B: what does building one AsyncTask actually cost in
    /// PRODUCTION, and what part of the legacy `task build` line (92 ns)
    /// is bench fixture, not code? The two fixture isolates above time
    /// `handlers: Arc::from(Vec::new())` — a fresh Arc-header malloc AND
    /// its dealloc (both inside the timed build+drop) per op. Production
    /// never pays that on the enqueue path: `async_targets` resolves the
    /// handler list from the immutable compiled view, and under
    /// RESOLVE_SHARED_EXACT=true (TASK-175) the no-patterns shape returns
    /// `Arc::clone(slot.list)` — one atomic increment, zero mallocs; the
    /// empty case is a shared static (`empty_resolved`). Payload side:
    /// under ASYNC_PAYLOAD_SHARED=true (TASK-176) `queue_async_shared`
    /// MOVES the caller's handle in — zero atomics at build (the dec
    /// happens worker-side after the handler ran); the isolate below pays
    /// one inc/dec pair the production move avoids, so it is a
    /// CONSERVATIVE upper bound. Leaders: ASYNC_LEADERS_ZEROALLOC=true +
    /// owner-less handlers = `Vec::new()`, verbatim `build_leaders` shape
    /// (the scan is timed, matching the real caller). Event: InlineEvent
    /// (TASK-173), 11 bytes inline, no malloc. Consequence: if the
    /// prod-shape shared arm lands <=10 ns, production task construction
    /// is PROVEN <=10 ns (it does strictly less work than this arm).
    /// Micro-isolates decompose the legacy 92 ns ledger: fixture
    /// empty-Arc malloc/free, view-share inc/dec, InlineEvent memcpy,
    /// Value deep clone (the legacy `publish` repr — the only real,
    /// non-fixture term, and the reason the legacy path costs what it
    /// costs). Existing fixture lines above are kept verbatim as the
    /// cross-build noise check; this bench adds the production-shape
    /// lines, it does not redefine history.
    #[test]
    #[ignore]
    fn bench_task_build_ab() {
        let payload = serde_json::json!({ "tick": 1u64, "drained": 0u64 });
        let payload_arc = Arc::new(payload.clone());
        // Production-faithful resolved list: one owner-less handler behind
        // the Arc<[(owner, Handler)]> slice the view shares per publish
        // (RESOLVE_SHARED_EXACT shape). Built once, outside the loops.
        let bench_handler: Handler = Arc::new(|_: &str, _: &Value| {});
        let prod_list: Resolved = Arc::from(vec![(None, bench_handler)]);
        let iters = 200_000u32;
        let rounds = 5;
        const TASK_EVENT: &str = "bench.async";
        let mut fold = 0u64; // observability guard against elision

        fn time_arm(
            name: &'static str,
            mut op: impl FnMut() -> u64,
            fold: &mut u64,
            iters: u32,
            rounds: u32,
        ) -> (&'static str, f64) {
            for _ in 0..10_000u32 {
                *fold = fold.wrapping_add(op());
            }
            let mut best = f64::MAX;
            for _ in 0..rounds {
                let start = Instant::now();
                for _ in 0..iters {
                    *fold = fold.wrapping_add(op());
                }
                best = best.min(start.elapsed().as_secs_f64() / f64::from(iters));
            }
            (name, best * 1e9)
        }
        let results = [
            // A — legacy fixture line (verbatim bench_event_bus_publish arm)
            time_arm(
                "owned fixture (Arc::from(Vec::new()))",
                || {
                    drop(AsyncTask {
                        event: TASK_EVENT.into(),
                        payload: TaskPayload::Owned(payload.clone()),
                        leaders: Vec::new(),
                        handlers: Arc::from(Vec::new()),
                    });
                    0
                },
                &mut fold,
                iters,
                rounds,
            ),
            // B — shared fixture line (verbatim bench_event_publish_shared arm)
            time_arm(
                "shared fixture (Arc::from(Vec::new()))",
                || {
                    drop(AsyncTask {
                        event: TASK_EVENT.into(),
                        payload: TaskPayload::Shared(Arc::clone(&payload_arc)),
                        leaders: Vec::new(),
                        handlers: Arc::from(Vec::new()),
                    });
                    0
                },
                &mut fold,
                iters,
                rounds,
            ),
            // C — legacy publish() task construction, production shape:
            // deep-cloned payload (the repr legacy publish dictates) +
            // view-shared handler list + zero-alloc leaders scan
            time_arm(
                "owned prod-shape (view-shared list)",
                || {
                    drop(AsyncTask {
                        event: TASK_EVENT.into(),
                        payload: TaskPayload::Owned(payload.clone()),
                        leaders: build_leaders(&prod_list),
                        handlers: Arc::clone(&prod_list),
                    });
                    0
                },
                &mut fold,
                iters,
                rounds,
            ),
            // D — publish_shared task construction, production shape,
            // CONSERVATIVE: the isolate clones the handle (inc/dec) where
            // the production call MOVES it in (zero atomics) — production
            // is strictly cheaper than this arm by that pair
            time_arm(
                "shared prod-shape (conservative move-bound)",
                || {
                    drop(AsyncTask {
                        event: TASK_EVENT.into(),
                        payload: TaskPayload::Shared(Arc::clone(&payload_arc)),
                        leaders: build_leaders(&prod_list),
                        handlers: Arc::clone(&prod_list),
                    });
                    0
                },
                &mut fold,
                iters,
                rounds,
            ),
            // Micro-isolates: the legacy 92 ns ledger, term by term
            time_arm(
                "iso: fixture empty-Arc malloc+free",
                || {
                    let empty: Resolved = Arc::from(Vec::new());
                    drop(empty);
                    0
                },
                &mut fold,
                iters,
                rounds,
            ),
            time_arm(
                "iso: view-share inc/dec (resolved list)",
                || {
                    drop(Arc::clone(&prod_list));
                    0
                },
                &mut fold,
                iters,
                rounds,
            ),
            time_arm(
                "iso: InlineEvent::from (11B inline) + read",
                || {
                    let e: InlineEvent = TASK_EVENT.into();
                    let n = e.as_str().len();
                    drop(e);
                    n as u64
                },
                &mut fold,
                iters,
                rounds,
            ),
            time_arm(
                "iso: Value deep clone (owned payload repr)",
                || {
                    drop(payload.clone());
                    0
                },
                &mut fold,
                iters,
                rounds,
            ),
        ];
        let mut line = String::from("BENCH task_build_ab:");
        for (name, ns) in &results {
            line.push_str(&format!(" [{name}] {:.0} ns/op,", ns));
        }
        line.push_str(&format!(
            " fold {fold} (min of {rounds}x{iters})"
        ));
        println!("{line}");
        // Fold guard is never expected to trip (arms return 0 or tiny lens);
        // it exists so the compiler cannot prove the loops side-effect-free.
        assert!(fold != u64::MAX);
    }

    /// TASK-197 A/B: the async-side enqueue resolution, per publish.
    /// Arm A = the verbatim pre-197 work (`async_targets`: view load_arc
    /// inc/dec pair + exact-slot probe walk + list clone). Arm B = the
    /// memo-hit shape TASK-197 enables (one acquire-load probe via
    /// memo_find + the record-carried list clone; the tag check proves the
    /// list matches this generation). The iso delta is the banked number;
    /// the end-to-end async lines ride their wake-dominated noisy class
    /// (recorded by bench_event_bus_publish / bench_event_publish_shared).
    /// Solo protocol: the memo table is process-global.
    #[test]
    #[ignore]
    fn bench_async_targets_ab() {
        let bus = with_cap(64);
        let _atok = bus.subscribe_async("bench.async", Arc::new(|_, _| {}));
        let payload = serde_json::json!({ "tick": 1u64, "drained": 0u64 });
        // Fill the memo the way a real publisher would (cold fill + hits),
        // so arm B reads the record TASK-197 stores.
        for _ in 0..10_000u32 {
            let _ = bus.publish_shared("bench.async", Arc::new(payload.clone()));
        }
        let iters = 200_000u32;
        let rounds = 5;
        const EVENT: &str = "bench.async";
        let hash = fnv1a(EVENT.as_bytes());
        let gens = bus.gens.load(Ordering::Acquire);
        let mut fold = 0u64;

        fn time_arm(
            name: &'static str,
            mut op: impl FnMut() -> u64,
            fold: &mut u64,
            iters: u32,
            rounds: u32,
        ) -> (&'static str, f64) {
            for _ in 0..10_000u32 {
                *fold = fold.wrapping_add(op());
            }
            let mut best = f64::MAX;
            for _ in 0..rounds {
                let start = Instant::now();
                for _ in 0..iters {
                    *fold = fold.wrapping_add(op());
                }
                best = best.min(start.elapsed().as_secs_f64() / f64::from(iters));
            }
            (name, best * 1e9)
        }

        // A — pre-197 verbatim: resolve through the async view per publish.
        let a = time_arm(
            "async_targets view walk (pre-197)",
            || {
                let l = bus.async_targets(EVENT, hash);
                std::hint::black_box(l.is_some());
                0
            },
            &mut fold,
            iters,
            rounds,
        );
        // B — TASK-197 memo-hit shape: one probe + carried list clone.
        let b = time_arm(
            "memo_find + carried list (TASK-197)",
            || {
                if let Some(rec) = memo_find(bus.sync.id, gens, hash, EVENT) {
                    let l = rec.async_list.clone();
                    std::hint::black_box(l.is_some());
                }
                0
            },
            &mut fold,
            iters,
            rounds,
        );
        // C — record-presence sanity inside the bench itself: a miss here
        // would mean the warmup did not fill the memo (arms are invalid).
        let rec_present = memo_find(bus.sync.id, gens, hash, EVENT)
            .is_some_and(|r| r.async_list.is_some());
        println!(
            "BENCH async_targets_ab: [{a}] {:.0} ns/op vs [{b}] {:.0} ns/op (min of {rounds}x{iters}) record-carried {rec_present} fold {fold}",
            a.1, b.1,
            a = a.0,
            b = b.0,
        );
        assert!(rec_present, "memo record must carry the async list for arm B to be the hit shape");
        assert!(fold != u64::MAX);
    }

    /// TASK-177 integral: the dispatcher pays one module guard per owned
    /// handler. Seed a registry entry, subscribe one sync handler inside a
    /// registration window (owner = ("bench-own-mod", 1)), then publish —
    /// each publish walks the memo/dispatch path and calls guard_module
    /// exactly once. The handler is a no-op, so the ON/OFF delta isolates
    /// the guard cost (registry Mutex + String alloc + HashMap lookups vs
    /// the lock-free gate).
    #[test]
    #[ignore]
    fn bench_event_dispatch_owned() {
        crate::platform::hot_reload::test_seed_module("bench-own-mod", 0, false);
        let bus = with_cap(64);
        {
            let _owner = crate::begin_registration("bench-own-mod", 1);
            let _tok = bus.subscribe("bench.own.tick", Arc::new(|_, _| {}));
        }
        let payload = serde_json::json!({ "n": 1u64 });
        let iters = 200_000u32;
        let rounds = 5;

        for _ in 0..10_000u32 {
            let _ = bus.publish("bench.own.tick", &payload);
        }
        let mut best = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                let _ = bus.publish("bench.own.tick", &payload);
            }
            best = best.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }
        println!(
            "BENCH dispatch_owned: publish(1 owned sync sub) {:.0} ns/op (min of {rounds}x{iters})",
            best * 1e9
        );
    }
}



