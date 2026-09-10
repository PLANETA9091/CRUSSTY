//! Brick 3: scheduler interception — one choke point for task routing.
//!
//! The platform exposes the kernel's task scheduler as a routable queue:
//! modules may (a) intercept scheduled tasks, (b) redirect them to their own
//! platform threads (e.g. a regional tick loop), or (c) inject tasks into the
//! kernel's own queue from any thread. The concrete kernel methods are located
//! via transform rules (see the `transform` brick); the interface below is the
//! module-facing contract.
//!
//! # Wave 2: default rules + the tick boundary
//!
//! [`install_default_rules`] registers transform rules on the global engine so
//! class bytes flowing through the engine pick up static probe calls at the
//! kernel's scheduling entry points. A rule whose names do not match the
//! running kernel simply never fires, so registering best-guess surfaces is
//! safe. The surfaces below were researched against the shipped Purpur
//! 1.21.10 jar (`unzip` + `javap` on `versions/1.21.10/purpur-1.21.10.jar`)
//! and corroborated with upstream references:
//!
//! | Kernel surface (internal names) | Rule | Status |
//! |---|---|---|
//! | `net/minecraft/server/MinecraftServer.tickServer` `(Ljava/util/function/BooleanSupplier;)V` | `onTick` | verified exact (tick loop entry, see misode's `tick_order` gist; PaperMC deepwiki "Tick Loop and Performance Monitoring") |
//! | `net/minecraft/server/level/ServerLevel.tick` `(Ljava/util/function/BooleanSupplier;)V` | `onLevelTick` | verified exact (per-dimension tick: scheduled block/fluid ticks, entities, spawners) |
//! | `io/papermc/paper/threadedregions/scheduler/FallbackRegionScheduler.run/execute` | `onTaskScheduled` | verified names; descriptor wildcard (overloads) |
//! | `io/papermc/paper/threadedregions/scheduler/FoliaGlobalRegionScheduler.run/execute` | `onTaskScheduled` | verified names; descriptor wildcard |
//! | `org/bukkit/craftbukkit/scheduler/CraftScheduler.mainThreadHeartbeat` `()V` | `onTaskScheduled` | verified: called from the `MinecraftServer` tick loop (disassembled) |
//! | `net/minecraft/world/ticks/LevelTicks.tick` (scheduled block/fluid tick drain, what `ServerLevel.getBlockTicks()` returns) | `onBlockTicks` | verified name; descriptor wildcard |
//!
//! The Paper regionized schedulers are the Folia-compatible scheduling API
//! (Paper docs "Supporting Paper and Folia"); on non-Folia
//! `FallbackRegionScheduler` forwards every task to the main thread, so the
//! class exists and is exercised on this kernel. On a Folia kernel the same
//! API submits to region threads, which is exactly the surface a
//! module-owned regional tick loop wants to claim.
//!
//! # Java helper contract
//!
//! Every rule injects a single `invokestatic` of a public static `()V` method
//! on one hook class — `dev.crussty.hooks.SchedulerHooks` (the transform
//! engine only supports `()V` helpers). The Java bootstrap
//! (`dev.dist.launcher.Boot`) must provide this class; the patched kernel
//! bytecode resolves it lazily at first execution, so loading it during
//! bootstrap (e.g. `Class.forName`) is sufficient. Reference implementation:
//!
//! ```java
//! package dev.crussty.hooks;
//!
//! /** Native bridge required by the scheduler-interception transform rules. */
//! public final class SchedulerHooks {
//!     private SchedulerHooks() {}
//!
//!     /** Injected at the top of MinecraftServer.tickServer — one call per main tick. */
//!     public static native void onTick();
//!     /** Injected at the top of ServerLevel.tick — one call per dimension per tick. */
//!     public static native void onLevelTick();
//!     /** Injected at Paper regionized / CraftScheduler submission entries. */
//!     public static native void onTaskScheduled();
//!     /** Injected at the top of LevelTicks.tick — the scheduled block/fluid drain. */
//!     public static native void onBlockTicks();
//! }
//! ```
//!
//! The four natives map 1:1 onto [`on_tick_boundary`], [`on_level_tick`],
//! [`on_task_scheduled`] and [`on_block_ticks`]. The bootstrap registers them
//! (RegisterNatives against the runtime's exported
//! `Java_dev_crussty_hooks_SchedulerHooks_*` symbols, or its own JNI
//! library). All four are `()V`: the engine cannot pass arguments, so task
//! probes are synthesized on the Rust side, see below.
//!
//! # Routing probes and `take_routed`
//!
//! [`on_task_scheduled`] / [`on_block_ticks`] cannot see the concrete kernel
//! task (the `()V` probe carries no arguments). Instead they synthesize a
//! probe [`ScheduledTask`] stamped with the current server tick and a
//! synthetic token, and consult [`route_task`] with it. When a router returns
//! [`Routing::RunOnModule`] the probe is stashed into the module-owned queue
//! that [`take_routed`] drains; `KeepKernel`/`Drop` leave the kernel
//! untouched. Cancelling the kernel's own copy of a task needs a
//! `BeforeCall` cancel rule (future work); today the routing decision is a
//! claim notification the module loop acts on.
//!
//! Intended module-side loop (e.g. a regional tick thread):
//!
//! ```text
//! loop {
//!     for task in platform::scheduler::take_routed() {
//!         // task.tag            "kernel" | "blockticks" — which surface claimed it
//!         // task.scheduled_tick server tick at claim time
//!         // task.kernel_token   synthetic identity for the module's own registry
//!         region_queue.push(task);
//!     }
//!     run_one_region_tick(region_queue); // the module's own tick loop
//! }
//! ```
//!
//! # Test seam
//!
//! In test builds the tick-duration telemetry push records into a test-local
//! list instead of the process-global TPS window, so parallel test binaries
//! cannot corrupt other bricks' TPS assertions; production builds push into
//! the telemetry window as described on [`on_tick_boundary`].

use crate::platform::events::lifecycle::TICK_BOUNDARY;
use crate::platform::transform::{global_engine, Injection, Rule};
use serde_json::json;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock, RwLock};
use std::time::Instant;

/// A unit of work the kernel scheduled (identified opaquely by the module).
#[derive(Debug, Clone)]
pub struct ScheduledTask {
    /// Opaque handle owned by the kernel adapter; not for direct use.
    pub kernel_token: u64,
    /// When the kernel scheduled it (server ticks, if known).
    pub scheduled_tick: Option<u64>,
    /// Optional module tag for routing decisions.
    pub tag: String,
}

/// Decision a module returns for an intercepted task.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Routing {
    /// Let the kernel run it on its main thread as usual.
    KeepKernel,
    /// Run it on this module's thread instead.
    RunOnModule,
    /// Drop it (cancel).
    Drop,
}

pub type RouterFn = Arc<dyn Fn(&ScheduledTask) -> Routing + Send + Sync>;

/// Hotpath note (TASK-161): `route_task` runs per task submission (Bukkit/
/// Paper schedulers submit hundreds of tasks per second). The router list is
/// an immutable snapshot behind an RwLock — the read path clones one `Arc`
/// (guard dropped before routers run, so a router may safely `add_router`),
/// instead of cloning a whole `Vec<RouterFn>` per submission.
static ROUTERS: OnceLock<RwLock<Arc<[RouterFn]>>> = OnceLock::new();

fn routers() -> &'static RwLock<Arc<[RouterFn]>> {
    ROUTERS.get_or_init(|| RwLock::new(Arc::from(Vec::new())))
}

/// Register a router; all routers are consulted in order until one returns
/// something other than KeepKernel (first non-keep wins).
pub fn add_router(f: RouterFn) {
    let mut w = routers().write().unwrap_or_else(|p| p.into_inner());
    let mut v: Vec<RouterFn> = w.to_vec();
    v.push(f);
    *w = v.into();
}

/// The kernel adapter calls this for every scheduled task (via transform).
pub fn route_task(task: &ScheduledTask) -> Routing {
    let snapshot: Arc<[RouterFn]> = routers().read().unwrap_or_else(|p| p.into_inner()).clone();
    for r in snapshot.iter() {
        let d = r(task);
        if d != Routing::KeepKernel {
            return d;
        }
    }
    Routing::KeepKernel
}

/// Inject a task into the kernel scheduler from any thread. The adapter
/// implements the enqueue; the return value is the token it assigned (0
/// means "queued, no token"). The kernel adapter drains the queue on its
/// main thread each tick, see [`on_tick_boundary`].
pub fn inject<F>(_tag: &str, f: F) -> u64
where
    F: FnOnce() + Send + 'static,
{
    KERNEL_QUEUE.get_or_init(|| Mutex::new(Vec::new())).lock().unwrap().push(Box::new(f));
    // The kernel adapter drains KERNEL_QUEUE on its main thread each tick.
    0
}

type InjectedTask = Box<dyn FnOnce() + Send>;

static KERNEL_QUEUE: OnceLock<Mutex<Vec<InjectedTask>>> = OnceLock::new();

/// Called by the kernel adapter on the main thread each tick: runs all
/// injected tasks. Tasks run OUTSIDE the queue mutex (a long task must not
/// block `inject` callers, and a task may itself `inject` without
/// deadlocking); anything enqueued mid-drain is picked up next tick.
pub fn drain_injected() -> usize {
    let batch: Vec<InjectedTask> = match KERNEL_QUEUE.get() {
        Some(m) => {
            let mut q = m.lock().unwrap_or_else(|p| p.into_inner());
            if q.is_empty() {
                return 0;
            }
            std::mem::take(&mut *q)
        }
        None => return 0,
    };
    let n = batch.len();
    for f in batch {
        f();
    }
    n
}

/// Fully-qualified hook class every default rule injects calls into; the
/// Java bootstrap (`dev.dist.launcher.Boot`) provides it — see the module
/// docs for the exact contract.
pub const HOOK_CLASS: &str = "dev.crussty.hooks.SchedulerHooks";

/// (class_pattern, method, descriptor, helper) for every default rule, kept
/// in one table so the registration and the documented surface cannot drift.
/// Descriptors marked `*` match all overloads of the named method.
const DEFAULT_RULES_TABLE: [(&str, &str, &str, &str); 8] = [
    (
        "net/minecraft/server/MinecraftServer",
        "tickServer",
        "(Ljava/util/function/BooleanSupplier;)V",
        "dev.crussty.hooks.SchedulerHooks.onTick",
    ),
    (
        "net/minecraft/server/level/ServerLevel",
        "tick",
        "(Ljava/util/function/BooleanSupplier;)V",
        "dev.crussty.hooks.SchedulerHooks.onLevelTick",
    ),
    (
        "io/papermc/paper/threadedregions/scheduler/FallbackRegionScheduler",
        "run",
        "*",
        "dev.crussty.hooks.SchedulerHooks.onTaskScheduled",
    ),
    (
        "io/papermc/paper/threadedregions/scheduler/FallbackRegionScheduler",
        "execute",
        "*",
        "dev.crussty.hooks.SchedulerHooks.onTaskScheduled",
    ),
    (
        "io/papermc/paper/threadedregions/scheduler/FoliaGlobalRegionScheduler",
        "run",
        "*",
        "dev.crussty.hooks.SchedulerHooks.onTaskScheduled",
    ),
    (
        "io/papermc/paper/threadedregions/scheduler/FoliaGlobalRegionScheduler",
        "execute",
        "*",
        "dev.crussty.hooks.SchedulerHooks.onTaskScheduled",
    ),
    (
        "org/bukkit/craftbukkit/scheduler/CraftScheduler",
        "mainThreadHeartbeat",
        "*",
        "dev.crussty.hooks.SchedulerHooks.onTaskScheduled",
    ),
    (
        "net/minecraft/world/ticks/LevelTicks",
        "tick",
        "*",
        "dev.crussty.hooks.SchedulerHooks.onBlockTicks",
    ),
];

static DEFAULT_RULES: OnceLock<()> = OnceLock::new();

/// Register the default scheduling-surface rules on the global transform
/// engine. Idempotent: only the first call registers anything; later calls
/// are no-ops, so bootstrap paths can call it freely without duplicating
/// rules (the engine also dedupes helper calls per method, but a once-guard
/// is cheaper than re-parsing every kernel class).
pub fn install_default_rules() {
    DEFAULT_RULES.get_or_init(|| {
        let engine = global_engine();
        for (class, method, descriptor, helper) in DEFAULT_RULES_TABLE {
            engine.register(Rule::new(class, method, descriptor, Injection::MethodEntry, helper));
        }
    });
}

/// Monotonic estimate of the server tick, bumped by [`on_tick_boundary`].
/// Stamps [`ScheduledTask::scheduled_tick`] on probes and the TICK_BOUNDARY
/// payload.
static TICK_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Number of `ServerLevel.tick` boundaries observed (all dimensions).
static LEVEL_TICK_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Synthetic token for probe tasks stashed for modules.
static ROUTED_TOKEN: AtomicU64 = AtomicU64::new(1);

/// Wall clock of the previous tick boundary in nanoseconds since the
/// [`MONO_EPOCH`] instant; `u64::MAX` is the "no baseline yet" sentinel. The
/// duration between two boundaries is the tick duration fed to the telemetry
/// TPS estimator. A plain atomic: single-writer (main tick thread), zero
/// locking on the per-tick path.
static LAST_BOUNDARY_NS: AtomicU64 = AtomicU64::new(u64::MAX);
static MONO_EPOCH: OnceLock<Instant> = OnceLock::new();

/// The current server tick estimate (monotonic, bumped per main tick).
pub fn current_tick() -> u64 {
    TICK_COUNTER.load(Ordering::Relaxed)
}

/// Called by the injected Java helper at the start of every main tick
/// (`MinecraftServer.tickServer`). Runs all injected tasks ([`drain_injected`]),
/// publishes the `platform.tick_boundary` lifecycle event, and feeds the
/// wall-clock tick duration into the telemetry TPS estimator (the first
/// boundary only establishes the baseline). Returns the number of injected
/// tasks drained.
pub fn on_tick_boundary() -> usize {
    let tick = TICK_COUNTER.fetch_add(1, Ordering::Relaxed) + 1;
    let drained = drain_injected();
    // Fast path (TASK-161): skip the payload allocation and the publish
    // entirely when nothing listens — the steady state for `tick_boundary`
    // in default deployments. One cached bus lookup instead of a five-Arc
    // clone per tick.
    let bus = tick_bus();
    if bus.has_subscribers(TICK_BOUNDARY) {
        bus.publish(TICK_BOUNDARY, &json!({ "tick": tick, "drained": drained }));
    }
    let epoch = *MONO_EPOCH.get_or_init(Instant::now);
    // One clock read per tick (TASK-162): the boundary duration and the TPS
    // window timestamp share this instant instead of paying a second read.
    let at = now();
    let now_ns = u64::try_from(at.duration_since(epoch).as_nanos()).unwrap_or(u64::MAX);
    let prev = LAST_BOUNDARY_NS.swap(now_ns, Ordering::Relaxed);
    if prev != u64::MAX {
        push_tick_sample_at(at, now_ns.saturating_sub(prev));
    }
    drained
}

/// Cached clone of the global event bus (the bus is a bag of Arcs; cloning
/// it per tick costs five atomic increments).
fn tick_bus() -> &'static crate::platform::events::EventBus {
    static BUS: OnceLock<crate::platform::events::EventBus> = OnceLock::new();
    BUS.get_or_init(crate::platform::events::global)
}

/// Monotonic clock, factored for test seams.
fn now() -> Instant {
    Instant::now()
}

/// Feed one tick duration into the telemetry TPS window. Split from
/// [`on_tick_boundary`] so test builds record samples locally instead of
/// mutating the process-global window (see the module docs, "Test seam").
#[cfg(not(test))]
fn push_tick_sample_at(at: Instant, ns: u64) {
    crate::platform::telemetry::push_tick_time_at(at, ns);
}

#[cfg(test)]
fn push_tick_sample_at(at: Instant, ns: u64) {
    let _ = at; // the local window records durations only
    TICK_SAMPLES.get_or_init(|| Mutex::new(Vec::new())).lock().unwrap().push(ns);
}

#[cfg(test)]
pub(super) static TICK_SAMPLES: OnceLock<Mutex<Vec<u64>>> = OnceLock::new();

/// Called by the injected Java helper at the start of every dimension tick
/// (`ServerLevel.tick`). Returns the new level-tick counter value.
pub fn on_level_tick() -> u64 {
    LEVEL_TICK_COUNTER.fetch_add(1, Ordering::Relaxed) + 1
}

/// Queue of tasks the routing probes stashed for modules (router returned
/// [`Routing::RunOnModule`]); drained by [`take_routed`].
static MODULE_QUEUE: OnceLock<Mutex<Vec<ScheduledTask>>> = OnceLock::new();

fn route_probe(tag: &str) -> Option<ScheduledTask> {
    // Zero-alloc fast path (TASK-161): with no router registered — the
    // production default — a scheduling-surface probe must not allocate a
    // task, a tag String, or touch any queue.
    if routers().read().unwrap_or_else(|p| p.into_inner()).is_empty() {
        return None;
    }
    let task = ScheduledTask {
        kernel_token: ROUTED_TOKEN.fetch_add(1, Ordering::Relaxed),
        scheduled_tick: Some(current_tick()),
        tag: tag.to_string(),
    };
    match route_task(&task) {
        Routing::RunOnModule => {
            MODULE_QUEUE.get_or_init(|| Mutex::new(Vec::new())).lock().unwrap().push(task.clone());
            Some(task)
        }
        _ => None,
    }
}

/// Called by the injected Java helper whenever a task hits a scheduling
/// surface (Paper regionized / Bukkit scheduler submission). Consults the
/// routers with a probe stamped with the current tick; when a router returns
/// [`Routing::RunOnModule`] the probe is stashed for the module-side loop
/// (see [`take_routed`]) and returned as `Some`. `None` means the kernel
/// keeps (or drops) the task.
pub fn on_task_scheduled() -> Option<ScheduledTask> {
    route_probe("kernel")
}

/// Like [`on_task_scheduled`], but fired at the scheduled block/fluid tick
/// drain (`LevelTicks.tick`) — modules that run region ticks can claim the
/// drain itself instead of individual tasks.
pub fn on_block_ticks() -> Option<ScheduledTask> {
    route_probe("blockticks")
}

/// Drain every task the scheduling probes stashed for modules (a router
/// returned [`Routing::RunOnModule`]). The module-side loop owns these tasks
/// from here on; see the module docs for the intended loop shape.
pub fn take_routed() -> Vec<ScheduledTask> {
    let Some(queue) = MODULE_QUEUE.get() else {
        return Vec::new();
    };
    queue.lock().unwrap_or_else(|p| p.into_inner()).drain(..).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    /// Serializes the scheduler tests: they share process-global state
    /// (router list, tick counters, queues), the same pattern as
    /// telemetry's `TEST_LOCK`.
    pub(super) static TEST_LOCK: Mutex<()> = Mutex::new(());

    pub(super) fn reset_routers() {
        if let Some(m) = ROUTERS.get() {
            *m.write().unwrap_or_else(|p| p.into_inner()) = Arc::from(Vec::new());
        }
    }

    pub(super) fn reset_queues() {
        if let Some(q) = KERNEL_QUEUE.get() {
            q.lock().unwrap_or_else(|p| p.into_inner()).clear();
        }
        if let Some(q) = MODULE_QUEUE.get() {
            q.lock().unwrap_or_else(|p| p.into_inner()).clear();
        }
        if let Some(s) = TICK_SAMPLES.get() {
            s.lock().unwrap_or_else(|p| p.into_inner()).clear();
        }
    }

    #[test]
    fn routing_decision_order() {
        let _guard = TEST_LOCK.lock().unwrap();
        let d1 = Arc::new(|_: &ScheduledTask| Routing::KeepKernel);
        let d2 = Arc::new(|_: &ScheduledTask| Routing::RunOnModule);
        add_router(d1);
        add_router(d2);
        let t = ScheduledTask { kernel_token: 1, scheduled_tick: None, tag: "x".into() };
        assert_eq!(route_task(&t), Routing::RunOnModule);
    }

    #[test]
    fn injected_tasks_drain() {
        let _guard = TEST_LOCK.lock().unwrap();
        let n = Arc::new(AtomicUsize::new(0));
        let n2 = Arc::clone(&n);
        inject("t", move || {
            n2.fetch_add(1, Ordering::SeqCst);
        });
        assert_eq!(drain_injected(), 1);
        assert_eq!(n.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn routers_consulted_in_order_first_non_keep_wins() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_routers();
        let tag = "order-probe";
        let drop_first = Arc::new(move |t: &ScheduledTask| {
            if t.tag == tag { Routing::Drop } else { Routing::KeepKernel }
        });
        let route_second = Arc::new(move |t: &ScheduledTask| {
            if t.tag == tag { Routing::RunOnModule } else { Routing::KeepKernel }
        });
        add_router(drop_first);
        add_router(route_second);
        let t = ScheduledTask { kernel_token: 1, scheduled_tick: None, tag: tag.to_string() };
        assert_eq!(route_task(&t), Routing::Drop, "the earlier non-keep decision wins");

        let other = ScheduledTask { kernel_token: 2, scheduled_tick: None, tag: "unrelated".into() };
        assert_eq!(route_task(&other), Routing::KeepKernel, "tag-scoped routers defer");

        // All routers deferring falls through to KeepKernel.
        reset_routers();
        add_router(Arc::new(|_: &ScheduledTask| Routing::KeepKernel));
        assert_eq!(route_task(&t), Routing::KeepKernel);
    }

    #[test]
    fn install_default_rules_is_idempotent() {
        let _guard = TEST_LOCK.lock().unwrap();
        let engine = global_engine();
        install_default_rules();
        let first = engine.rules().len();
        install_default_rules();
        install_default_rules();
        let second = engine.rules().len();
        assert_eq!(first, second, "re-registration must not add rules");
        let rules = engine.rules();
        let on_tick = rules
            .iter()
            .filter(|r| r.helper == "dev.crussty.hooks.SchedulerHooks.onTick")
            .collect::<Vec<_>>();
        assert_eq!(on_tick.len(), 1, "the main-tick rule is registered exactly once");
        assert_eq!(on_tick[0].class_pattern, "net/minecraft/server/MinecraftServer");
        assert_eq!(on_tick[0].method, "tickServer");
        assert_eq!(on_tick[0].injection, Injection::MethodEntry);
        // every default table entry made it into the engine
        let helpers = engine.rules().iter().map(|r| r.helper.clone()).collect::<Vec<_>>();
        for (_, _, _, helper) in DEFAULT_RULES_TABLE {
            assert!(helpers.contains(&helper.to_string()), "missing rule for {helper}");
        }
    }

    #[test]
    fn task_probe_stashed_and_drained() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_routers();
        reset_queues();
        add_router(Arc::new(|t: &ScheduledTask| {
            if t.tag == "kernel" { Routing::RunOnModule } else { Routing::KeepKernel }
        }));

        let tick_at_probe = current_tick();
        let task = on_task_scheduled().expect("router must claim kernel probes");
        assert_eq!(task.tag, "kernel");
        assert_eq!(task.scheduled_tick, Some(tick_at_probe));

        let drained = take_routed();
        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0].kernel_token, task.kernel_token);
        assert!(take_routed().is_empty(), "the queue drains fully");

        // A surface no router claims is not stashed.
        assert!(on_block_ticks().is_none(), "unclaimed probes stay in the kernel");
        assert!(take_routed().is_empty());
    }

    #[test]
    fn on_tick_boundary_drains_injected_and_publishes() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_queues();
        let bus = crate::platform::events::global();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let s = Arc::clone(&seen);
        bus.subscribe(TICK_BOUNDARY, Arc::new(move |_, payload| {
            s.lock().unwrap().push((
                payload["tick"].as_u64().unwrap_or(0),
                payload["drained"].as_u64().unwrap_or(u64::MAX),
            ));
        }));

        let done = Arc::new(AtomicUsize::new(0));
        let d1 = Arc::clone(&done);
        inject("t", move || {
            d1.fetch_add(1, Ordering::SeqCst);
        });
        let d2 = Arc::clone(&done);
        inject("u", move || {
            d2.fetch_add(1, Ordering::SeqCst);
        });
        let before = current_tick();
        assert_eq!(on_tick_boundary(), 2, "both injected tasks run at the boundary");
        assert_eq!(done.load(Ordering::SeqCst), 2);
        assert_eq!(current_tick(), before + 1, "the boundary bumps the tick estimate");
        assert_eq!(on_tick_boundary(), 0, "empty queue drains to zero");
        assert_eq!(current_tick(), before + 2);
        assert_eq!(
            *seen.lock().unwrap(),
            vec![(before + 1, 2u64), (before + 2, 0u64)],
            "TICK_BOUNDARY published once per boundary with tick and drained count"
        );

        // Telemetry: the first boundary only establishes the baseline, the
        // second records one tick-duration sample.
        let samples = TICK_SAMPLES
            .get()
            .map(|m| m.lock().unwrap_or_else(|p| p.into_inner()).len())
            .unwrap_or(0);
        assert_eq!(samples, 1, "one sample after the baseline boundary");
    }

    #[test]
    fn on_level_tick_counts_dimensions() {
        let _guard = TEST_LOCK.lock().unwrap();
        let before = LEVEL_TICK_COUNTER.load(Ordering::SeqCst);
        assert_eq!(on_level_tick(), before + 1);
        assert_eq!(on_level_tick(), before + 2);
    }
}

#[cfg(test)]
mod bench_hotpath {
    //! Release-only A/B benches: `cargo test --release -- --ignored --nocapture bench_scheduler`.
    //! Per-task-submission probe path + the steady-state main-tick boundary.
    use super::*;
    use std::time::Instant;

    #[test]
    #[ignore]
    fn bench_scheduler_hotpath() {
        let _guard = tests::TEST_LOCK.lock().unwrap();
        tests::reset_routers();
        tests::reset_queues();
        let iters = 200_000u32;
        let rounds = 5;

        // route_task with 2 registered routers, both deferring (steady state)
        add_router(Arc::new(|_: &ScheduledTask| Routing::KeepKernel));
        add_router(Arc::new(|_: &ScheduledTask| Routing::KeepKernel));
        let t = ScheduledTask { kernel_token: 7, scheduled_tick: None, tag: "kernel".into() };
        for _ in 0..10_000u32 {
            let _ = route_task(&t);
        }
        let mut best_route = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                let _ = route_task(&t);
            }
            best_route = best_route.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }

        // probe path with routers registered
        for _ in 0..10_000u32 {
            let _ = on_task_scheduled();
        }
        let mut best_probe = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                let _ = on_task_scheduled();
            }
            best_probe = best_probe.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }
        tests::reset_routers();

        // probe path, empty router table (production default — must be free)
        for _ in 0..10_000u32 {
            let _ = on_task_scheduled();
        }
        let mut best_probe_empty = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                let _ = on_task_scheduled();
            }
            best_probe_empty = best_probe_empty.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }

        // steady-state tick boundary: empty injected queue, no subscribers
        for _ in 0..1_000u32 {
            let _ = on_tick_boundary();
        }
        let mut best_tick = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                let _ = on_tick_boundary();
            }
            best_tick = best_tick.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }

        // teardown: restore the pristine baseline state so the unit tests'
        // assertions (TICK_SAMPLES len, baseline-first-sample) hold even under
        // --include-ignored runs.
        tests::reset_routers();
        tests::reset_queues();
        if let Some(s) = TICK_SAMPLES.get() {
            s.lock().unwrap_or_else(|p| p.into_inner()).clear();
        }
        reset_last_boundary_for_tests();
        println!(
            "BENCH scheduler: route_task(2 defer) {:.0} ns/op, probe(2 routers) {:.0} ns/op, probe(empty) {:.0} ns/op, tick_boundary {:.0} ns/op (min of {rounds}x{iters})",
            best_route * 1e9,
            best_probe * 1e9,
            best_probe_empty * 1e9,
            best_tick * 1e9
        );
    }

    /// Reset the last-boundary baseline (u64::MAX sentinel = no baseline).
    fn reset_last_boundary_for_tests() {
        LAST_BOUNDARY_NS.store(u64::MAX, Ordering::Relaxed);
    }
}
