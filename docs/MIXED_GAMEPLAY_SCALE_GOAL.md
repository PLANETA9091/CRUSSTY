# Mixed Gameplay Scale Goal

Date: 2026-05-17 CEST

This is the next concrete step after the 500 creative-block claim: make the
stress harness measure mixed gameplay, not just standing, moving, or pure
block spam.

## Target Claim Shape

Allowed only after the checklist below is green:

> production-ready for measured mixed-gameplay Minecraft load on the verified
> artifact, with movement, block, command, item, interact, mob, and plugin
> event pressure, plus repeatable evidence bundles and honest failure output.

This is not a claim for real-player parity, infinite scale, or a full Rust
Paper runtime.

## Current State

- [x] `mixed-gameplay` scenario exists in `scripts/run_load_test.sh`.
- [x] `scripts/run_stress_mixed_gameplay_gate.sh` exists.
- [x] `CompatProbe` now counts command preprocess, item-held, animation,
  interact, and other mixed workload counters.
- [x] Bot summaries now export mixed packet counters separately from block
  counters.
- [x] Mixed gameplay smoke passed with 4 bots, 0 kicks, 0
  `moved_too_quickly` warnings, 0 mixed action errors, and live server-side
  command/item/animation/interact counters.
- [x] Stress-corpus mixed-gameplay diagnostic exists on 50 bots with 26
  plugins, 10 datapacks, and 150 spawned mobs.
- [x] Same-artifact native-noise A/B exists. `ImprovedNoise` native is now the
  default guarded runtime path; `PerlinNoise` native remains opt-in because
  the combined native-noise run regressed.
- [x] Rejected `holdercache` visitor candidate is removed from the runtime
  patch stack after worsening the 50-bot mixed-gameplay gate.
- [x] Stress-corpus mixed-gameplay 50-bot gate passes on TPS/MSPT.
- [x] High-tier mixed-gameplay wrapper keeps failed runs evaluable by
  preserving summary/gate output instead of aborting before report capture.
- [x] Mixed-gameplay tier ladder runner exists:
  `scripts/run_mixed_gameplay_scale_ladder.sh`.
- [ ] Stress-corpus mixed-gameplay 250-bot diagnostic exists.
- [ ] Stress-corpus mixed-gameplay 500-bot diagnostic exists.
- [ ] Stress-corpus mixed-gameplay 500-bot gate passes.
- [ ] Mixed combat/entity-damage workload is validated without unsafe kicks.

## Current Pass

The current no-env default-ImprovedNoise 50-bot stress-corpus mixed-gameplay
gate now passes honestly on performance and stability:

- `online_max=50`
- `native_improved_noise_loaded=true`
- `native_perlin_noise_loaded=false`
- `load_window_tps1_avg=18.33`
- `load_window_tps1_min=15.88`
- `load_window_avg_tick_ms_avg=26.84`
- `load_window_avg_tick_ms_max=78.32`
- `watchdog_thread_dumps=0`
- `sync_load_stack_hits=0`
- `moved_too_quickly_warnings=0`
- `stability_failures=0`
- `process_rss_mib_max=5503.3`

That lifts the mixed-gameplay gate out of the failure bucket. The next
bottleneck is the next tier, not this 50-bot surface.

## Next Runner

The real next diagnostic runner is:

```bash
MC_EULA_AGREE=true ./scripts/run_mixed_gameplay_scale_ladder.sh
```

By default it runs `100 250 500` bot mixed-gameplay tiers with the full stress
corpus, 16/16 view/simulation distance, mob pressure, per-tier gate copies,
thread-sample summaries, and a stable current ladder report. A red tier is
still useful evidence, but it is not a claim.
