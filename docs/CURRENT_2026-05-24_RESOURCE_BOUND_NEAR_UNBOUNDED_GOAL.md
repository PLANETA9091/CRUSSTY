# CURRENT 2026-05-24: Resource-bound near-unbounded core goal

Дата: 2026-05-24 CEST

Это активная большая цель для `/root/rust`.

Физически бесконечного сервера не существует: CPU, RAM, сеть, диск, JVM,
Minecraft protocol, клиенты и ОС конечны. Поэтому цель жёстче и полезнее, чем
просто сказать `unlimited`: ядро должно идти вверх по измеренным tiers до
потолка железа, а при нехватке ресурсов не разваливаться, а включать
bounded backpressure, budgets, graceful degradation и восстановление без
порчи данных.

Плагины и датапаки не обновляются, не патчатся и не оптимизируются ради
прохождения теста. Они остаются стресс-входом. Оптимизируется ядро:
Paper hot paths, Rust/JNI модули, chunk/entity/network scheduling,
worldgen/chunk IO, save/recovery, backpressure и evidence tooling.

## Allowed Future Claim

Эту формулировку можно будет заявлять только после закрытия checklist ниже:

> resource-bound near-unbounded production scaling for measured Minecraft
> player, mob, chunk, worldgen, plugin, network, IO and recovery tiers on a
> verified artifact, with bounded queues, bounded memory growth, graceful
> overload behavior, repeatable gates, restart/recovery evidence, and
> self-contained evidence bundles.

## Current Starting Truth

- [x] Literal infinite players is not a valid claim.
- [x] Literal infinite mobs is not a valid claim.
- [x] Literal infinite CPU/RAM/IO/network is not a valid claim.
- [x] Historical P500 evidence exists only for older artifacts and does not
  automatically certify the current artifact.
- [x] The current artifact identity must come from `reports/artifacts.json`
  and `reports/artifact-hashes.txt`.
- [x] `BLOCKED.md` currently says the current-artifact P500 claim is blocked
  until clean host/canary/preflight and fresh evidence are regenerated.
- [x] Stress corpus and mixed gameplay harnesses already exist.
- [x] Native Rust modules are currently mostly parity/benchmark/diagnostic
  models unless a guarded runtime hook is proven and enabled.
- [x] Gradle daemon heap is now repo-level via `org.gradle.jvmargs=-Xmx4g
  -Dfile.encoding=UTF-8`.
- [ ] Current patch stack applies cleanly with `applySourcePatches`.
- [ ] Current artifact P500 cold+warm/repeat/bundle claim is green again.

## Phase 0: Restore Build And Patch Truth

- [ ] `./gradlew applyPatches` passes on the current dirty patch stack.
- [ ] `./gradlew :paper-server:compileJava` passes after patch application.
- [ ] `./gradlew createMojmapBundlerJar` produces the current optimized jar.
- [ ] `scripts/build_optimized.sh` completes with artifact reports updated.
- [ ] `sha256sum -c reports/artifact-hashes.txt` passes.
- [ ] `reports/artifacts.json` records the exact current jar, runtime,
  native library, AppCDS, mappings and remap classpath.
- [ ] No accepted code path is based on a patch that only passed a synthetic
  microbench while failing the real gate.

## Phase 1: Re-Certify The P500 Floor

- [ ] Host synthetic canary passes before the run.
- [ ] Strict foreign-process preflight passes before the run.
- [ ] P500 cold/fresh 32 view / 32 simulation / creative block gate passes.
- [ ] P500 warm-source gate passes on the same artifact.
- [ ] Repeat quorum passes on the same artifact.
- [ ] Plugin matrix passes on the same artifact.
- [ ] Restart/recovery passes on the same artifact.
- [ ] Forced-ticket persistence passes on the same artifact.
- [ ] `reports/production-500-readiness-bundle-current` is regenerated.
- [ ] Bundle validation and claim assertion pass against current artifacts.

## Phase 2: Prove Bounded Overload Instead Of Collapse

- [ ] Add or prove outbound packet send debt metrics per player cohort.
- [ ] Add or prove chunk send queue depth metrics per tick/window.
- [ ] Add or prove chunk generation queue depth metrics under fresh worldgen.
- [ ] Add or prove entity tracker fanout debt metrics under high player count.
- [ ] Add or prove mob AI/pathfinding budget metrics under high mob count.
- [ ] Add or prove region IO writeback pressure metrics under sustained block
  mutation.
- [ ] Slow-client pressure triggers bounded backpressure before runaway send
  debt.
- [ ] Packet burst pressure recovers without OOM, watchdog loops or silent
  disconnect storms.
- [ ] Chunk send/generation pressure degrades through explicit budgets, not
  unbounded queues.
- [ ] Region IO pressure slows safely and restart/recovery preserves data.

## Phase 3: Player, Mob, Chunk And Worldgen Tiers

- [ ] P500 stress-corpus mixed gameplay passes.
- [ ] P750 stress-corpus mixed gameplay diagnostic exists.
- [ ] P1000 stress-corpus mixed gameplay diagnostic exists.
- [ ] Highest accepted player tier has cold+warm evidence.
- [ ] M1k mixed mobs passes with an accepted player tier.
- [ ] M5k mixed mobs diagnostic exists.
- [ ] M10k mixed mobs diagnostic exists with bounded AI/pathfinding cost.
- [ ] C10k loaded/generating chunks diagnostic exists.
- [ ] C25k loaded/generating chunks diagnostic exists if hardware allows.
- [ ] Heavy datapack worldgen remains compatible with generation hooks.
- [ ] Combined player+mob+chunk+plugin tier exists only after isolated
  player, mob and chunk ladders are understood.

## Phase 4: Core Optimization Frontier

- [ ] Remove allocation-heavy hot paths proven by profiler, gate logs or
  focused parity benches.
- [ ] Replace stream/collector hot paths in mob/player/chunk loops where
  semantics are preserved.
- [ ] Keep chunk wait/load paths allocation-light and avoid sync-load spikes.
- [ ] Expand Rust/JNI runtime hooks only when JNI overhead, fallback behavior
  and strict load gates prove the hook is worth enabling.
- [ ] Rust parity modules must keep Java-equivalent behavior tests before any
  production hook.
- [ ] Any native hook has a strict load gate, fallback path, runtime flag, and
  rollback plan.
- [ ] Rejected candidates remain diagnostic only and are not counted as
  production speedups.

## Phase 5: Evidence And Publication

- [ ] Every accepted tier has raw logs, summaries, resource CSVs and gate
  reports.
- [ ] Every accepted tier has exact artifact hashes and source freshness proof.
- [ ] Every accepted tier has `bundle.json`, `MANIFEST.txt`, `CLAIM.md` and
  copied evidence files.
- [ ] Bundle validators pass without hidden working-tree context.
- [ ] Publication files state exact measured tier and exact non-claims.
- [ ] A failed tier is documented as failure evidence, not a claim.
- [ ] The final wording never claims more than the measured artifact/profile.

## Strict Non-Claims

- [x] No full Rust Paper runtime claim.
- [x] No literal unlimited players.
- [x] No literal unlimited mobs.
- [x] No literal unlimited chunks, ticks or worlds.
- [x] No unlimited plugin compatibility.
- [x] No unlimited datapack compatibility.
- [x] No real-player gameplay parity without real-client evidence.
- [x] No multi-hour soak unless that exact soak passed.
- [x] No production claim on a noisy host or stale bundle.

## Definition Of Done

- [ ] The current artifact has a fresh green P500 production bundle again.
- [ ] The next higher measured tier has its own green bundle.
- [ ] Overload behavior is bounded and observable across network, chunk,
  entity, mob AI, worldgen and IO paths.
- [ ] Remaining ceilings are named as hardware, network, disk, JVM, protocol
  or explicit policy ceilings with evidence.
- [ ] The server fails closed or degrades predictably under resource pressure
  instead of crashing, corrupting state, or growing unbounded queues.
