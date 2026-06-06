# Current Maximum Scale Goal

Date: 2026-05-20 CEST

This file is the active goal for the next push. The target is not a literal
unlimited server. Literal unlimited players, mobs, chunks, ticks, plugins, or
datapacks cannot be a true engineering claim. The real goal is stricter:

raise the current Paper 1.21.10 artifact through measured player, mob, chunk,
worldgen, plugin, datapack, network, restart, recovery, and soak gates until
the next limit is a proven hardware, network, disk, or policy ceiling.

## Allowed Claim Shape

Only claim a tier after its fresh summary, gate report, logs, resource CSV,
artifact hashes, repeat evidence, restart/recovery evidence, forced-ticket
evidence, and self-contained evidence bundle are present and green:

> production-ready for the measured tier on the verified artifact, with exact
> bot count, view distance, simulation distance, workload, plugin/datapack
> corpus, mob pressure, world source, restart/recovery scope, soak duration,
> artifact hashes, and non-claims stated.

Do not claim:

- literal unlimited scale
- full Rust Paper runtime
- arbitrary plugin compatibility without a matrix gate
- arbitrary datapack/worldgen compatibility without a matrix gate
- real-player gameplay parity without live client evidence
- multi-hour soak without fresh soak evidence
- a higher tier because a lower tier passed
- a clean production claim from a noisy or incomplete run

## Verified Floor

- [x] A narrow `500 bots / 32 view / 32 simulation / creative block`
  production-ready claim exists on a verified artifact with cold+warm soak,
  repeat quorum, plugin matrix, restart/recovery, forced-ticket persistence,
  artifact hashes, and a self-contained evidence bundle.
- [x] Heavy stress corpus exists with `26` total plugin jars and `10`
  datapacks.
- [x] Mixed-gameplay workload exists with movement, block place/break, held
  item switches, arm animation, player input, use-item, client commands,
  stress plugins, stress datapacks, and mobstorm pressure.
- [x] Radius-based arena preload exists and is wired through the load harness
  with configurable `radiusChunks` and `maxInFlight`.
- [x] Radius-preload smoke evidence exists with `radiusChunks=2`,
  `maxInFlight=12`, zero preload failures, zero bot kicks, zero bot errors,
  zero watchdog dumps, zero sync-load hits, and zero stability failures.
- [x] The current P100 radius-preload mixed-gameplay run has a final summary.
- [x] The current P100 radius-preload mixed-gameplay run has a final gate
  report.
- [x] The current artifact has a fresh accepted P100 mixed-gameplay gate.

## Current Live Checkpoint

Current live run:

```text
current-0127-radiuspreload-joins20-p250-ramp180-20260520-053648
```

Rules for this checkpoint:

- [x] Do not start a second P250 run in parallel.
- [x] Let the live run finish, or classify it as stalled with log evidence.
- [x] Parse its summary and gate report.
- [x] The fresh result is red, so the next move is not a blind larger tier.
- [x] Keep `watchdog_thread_dumps=0` as a target line, not a claim.
- [x] Keep `sync_load_stack_hits=0` as a target line, not a claim.
- [x] Keep `stability_failures=0` as a target line, not a claim.

Current bottleneck:

- `observed_online_max=250`
- `observed_bot_connected_max=250`
- `observed_bot_ready_max=250`
- `observed_bot_active_max=250`
- `observed_bot_errors_max=14`
- `observed_load_window_reached_full_online=true`
- `observed_bot_action_gate_opened=true`
- `observed_tps1_avg=14.31`
- `observed_tps1_min=0.61`
- `observed_avg_tick_ms_avg=79.31`
- `observed_avg_tick_ms_max=915.73`
- `observed_watchdog_thread_dumps=12`
- `observed_external_thread_prints=10`

The join/ready admission path is no longer the main blocker. The explicit
`misc.max-joins-per-tick=20` knob let the tier reach full online and open
the action gate, but the first full-online window is still unstable on the
fresh world with the full stress corpus. The next measured move is a warm-
source P250 diagnostic on the same artifact. The checklist for that work is
in `docs/NEXT_2026-05-20_P250_POSTFULLONLINE_GOAL.md`.

## Player Tier Ladder

These tiers are for stress-corpus mixed gameplay, not the older creative-block
release profile.

- [x] P100 fresh-world gate passes on the current artifact.
- [ ] P100 warm-world gate passes on the current artifact.
- [ ] P100 cold+warm repeat quorum exists.
- [ ] P100 restart/recovery and forced-ticket persistence evidence exists.
- [ ] P100 self-contained evidence bundle exists.
- [ ] P250 fresh-world gate passes.
- [ ] P250 warm-world gate passes.
- [ ] P250 cold+warm repeat quorum exists.
- [ ] P500 fresh-world gate passes.
- [ ] P500 warm-world gate passes.
- [ ] P500 cold+warm repeat quorum exists.
- [ ] P500 2h cold+warm soak exists.
- [ ] P750 diagnostic exists.
- [ ] P1000 diagnostic exists.
- [ ] P1500 diagnostic exists if hardware allows.
- [ ] P2000 diagnostic exists if hardware allows.
- [ ] P3000 diagnostic exists if hardware allows.
- [ ] P5000 diagnostic exists only if hardware and network allow it.

Minimum gate floor for accepted stress-mixed-gameplay player tiers:

- [ ] requested tier reaches `online_max`, `bot_ready_max`, and
  `bot_active_max`
- [ ] `bot_kicked_max = 0`
- [ ] `bot_errors_max = 0`
- [ ] `load_window_tps1_avg >= 18.00`
- [ ] `load_window_tps1_min >= 15.00`
- [ ] `load_window_avg_tick_ms_avg <= 75.00`
- [ ] `load_window_avg_tick_ms_max <= 150.00`
- [ ] `process_rss_mib_max <= 28672`
- [ ] `watchdog_thread_dumps = 0`
- [ ] `sync_load_stack_hits = 0`
- [ ] `stability_failures = 0`
- [ ] no unbounded heap, packet queue, chunk queue, region IO, or disk growth

## Mob And Entity Ladder

Mob tiers must preserve Bukkit/plugin-visible semantics. Optimization is
allowed only where behavior stays compatible or the gate explicitly proves the
compatibility surface.

- [ ] M1k mixed mobs with an accepted player tier.
- [ ] M5k mixed mobs with an accepted player tier.
- [ ] M10k mixed mobs with an accepted player tier.
- [ ] M25k mixed mobs diagnostic.
- [ ] M50k mixed mobs diagnostic if hardware allows.
- [ ] Pathfinding budget evidence.
- [ ] Goal-selector budget evidence.
- [ ] Collision lookup budget evidence.
- [ ] Entity tracker budget evidence.
- [ ] Target acquisition budget evidence.
- [ ] Despawn/removal cleanup evidence.
- [ ] Entity save/load persistence evidence.
- [ ] AI backpressure evidence that degrades gracefully instead of killing TPS.

## Chunk, Worldgen, And IO Ladder

World generation must stay compatible with vanilla semantics, datapacks, and
plugin hooks. Do not change generation output to win a benchmark unless a
separate compatibility gate proves it is safe.

- [ ] C10k loaded chunks with accepted player/mob pressure.
- [ ] C25k loaded chunks with accepted player/mob pressure.
- [ ] C50k loaded chunks diagnostic if hardware allows.
- [ ] Fresh-world generation gate with heavy datapacks.
- [ ] Warm-world / pregenerated gate with the same workload.
- [ ] Datapack worldgen matrix gate.
- [ ] Plugin worldgen-hook matrix gate.
- [ ] Chunk generation queue backpressure gate.
- [ ] Chunk send queue backpressure gate.
- [ ] Region read/write latency gate.
- [ ] Save-all under load gate.
- [ ] Restart after high-chunk load gate.
- [ ] Forced-ticket persistence under high load gate.
- [ ] Crash consistency / world save correctness evidence.

## Plugin, Datapack, And Network Ladder

- [ ] Full stress corpus passes at P100.
- [ ] Full stress corpus passes at P250.
- [ ] Full stress corpus passes at P500.
- [ ] Plugin matrix includes the currently downloaded heavy jars.
- [ ] Datapack matrix includes the currently downloaded heavy packs.
- [ ] Login burst gate.
- [ ] Disconnect storm gate.
- [ ] Slow-client backpressure gate.
- [ ] Packet budget per player.
- [ ] Chunk packet budget.
- [ ] Entity tracker packet budget.
- [ ] Compression/encode hot-path profile.
- [ ] No OOM under packet burst.
- [ ] No main-thread death under packet burst.

## Rust And Native Runtime Ladder

Paper runtime is not "rewritten in Rust" until the server actually uses a
strictly gated native replacement in production paths. Native modules must
prove parity, load gating, fallback behavior, and benchmark value.

- [ ] Every native candidate has a Java fallback.
- [ ] Every native candidate has a strict load gate.
- [ ] Every native candidate has parity tests.
- [ ] Every native candidate has microbench evidence.
- [ ] Every native candidate has a load-test gate on the target profile.
- [ ] No native candidate is enabled by default after a regression.
- [ ] Accepted native paths are listed in `docs/PARITY_MATRIX.md`.
- [ ] Rejected native paths stay documented with the failing evidence.

## Soak, Recovery, And Publication

- [ ] 2h cold+warm soak for every accepted player tier.
- [ ] 24h soak for the leading accepted tier.
- [ ] Restart during load.
- [ ] Restart after load.
- [ ] Recovery after disk pressure.
- [ ] Recovery after plugin churn.
- [ ] Recovery after worldgen churn.
- [ ] Recovery after forced-ticket heavy load.
- [ ] Timestamped evidence bundle for every accepted tier.
- [ ] Hash manifest for every accepted bundle.
- [ ] Stable `current` publication file for every accepted tier.
- [ ] Independent validation script for every accepted bundle.

## Working Rule

If a tier fails, the next task is not a bigger number. The next task is the
specific bottleneck shown by that failed tier. A checkbox changes to `[x]`
only after the current artifact has fresh green evidence for that exact line.
