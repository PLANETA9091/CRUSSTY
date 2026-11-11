# Platform bricks

In plain words: the runtime itself has a toolbox of ready-made parts —
events, timers, storage, threads, crash protection. Your module doesn't
need to reinvent them: the **platform bridge** hands the toolbox to any
module that asks. This page lists what's in the toolbox and the rules for
using it. Module developers only; end users can skip it.

The runtime ships twelve native primitives under `runtime/src/platform/`.
They are the building blocks every module builds on: instead of reinventing
event buses, tick routing or persistence in each module, a module uses the
brick's public API (re-exported from `platform/mod.rs`). Each file is owned
by exactly one concern; bricks do not depend on each other's internals.

| Brick | File | What it gives |
|---|---|---|
| transform | `transform.rs` | ClassFileLoadHook patch pipeline (bytecode rewriting) |
| events | `events.rs` | async pub/sub event bus with backpressure and lifecycle events |
| signals | `signals.rs` | crash isolation: native backtrace + fault stats, watchdog, fault hooks |
| network | `network.rs` | connection registry (LRU, UUID-keyed), hooks, traffic counters |
| scheduler | `scheduler.rs` | tick routing on the kernel main thread, injected tasks |
| storage | `storage.rs` | key-value persistence for modules |
| side_table | `side_table.rs` | O(1) metadata beside kernel objects |
| hot_reload | `hot_reload.rs` | swap a module's library without a server restart |
| threads | `threads.rs` | managed worker threads |
| barriers | `barriers.rs` | multi-phase synchronization between module threads |
| telemetry | `telemetry.rs` | metrics registry + UDP export |
| save_events | `save_events.rs` | world-save lifecycle hooks |

## Example: events

```rust
use crussty_runtime::platform::{events, lifecycle};

let _token = events::global().subscribe(
    lifecycle::PLUGIN_LOADED,
    Arc::new(|_, payload| { /* ... */ }),
);
events::global().publish(lifecycle::PLUGIN_LOADED, &serde_json::json!({ "phase": "ready" }));
```

## Example: crash isolation (signals)

```rust
use crussty_runtime::platform::signals;

let handled = signals::install_handlers();  // returns count of signals taken over
signals::set_crash_log_path("/var/log/crussty/faults.log");
```

Handlers chain to whatever the JVM had installed (`sigaction` with
`SA_SIGINFO`): if the JVM has its own handler for a signal, the platform
records fault counters and forwards the fault to it, so the JVM's own
machinery (hs_err dumps, internal null-check/stack-bang signals) keeps
working. Only when there is no previous handler does the platform dump a
backtrace itself and re-raise. Disable entirely with `CRUSSTY_NO_SIGNALS=1`.

## Example: telemetry

```rust
use crussty_runtime::platform::telemetry;

telemetry::init("/tmp/crussty-telemetry.sock")?;  // UDP export
telemetry::set_uptime(started_at);
telemetry::snapshot();  // metrics registry readout
```
