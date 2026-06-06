# REAL GOAL 2026-05-25: recert текущего артефакта и bounded-overload P500

Дата: 2026-05-25 CEST

Это текущий практический goal для `/root/rust`: сначала восстановить честную
сертификацию текущего artifact bundle на P500 path, затем добавить измеримое
доказательство, что перегрузка остаётся bounded, а не превращается в
неограниченные очереди, watchdog churn, массовые disconnect/kick или порчу
состояния.

Цель не в том, чтобы написать красивую формулировку. Цель в том, чтобы свежий
текущий артефакт имел проверяемые хэши, свежие отчёты, минимальный smoke или
diagnostic после правки, и чтобы все claims оставались заблокированы до
появления нового evidence на этом же артефакте.

## Purpose

- [x] Зафиксировать честную текущую правду перед новой правкой.
- [x] Не переиспользовать старый P500 bundle как current-artifact claim.
- [x] Направить этот turn на маленькую core hot-path правку с низким риском.
- [x] После правки собрать, пересчитать artifact identity и прогнать самый
  дешёвый полезный smoke/diagnostic.
- [x] Держать claim text консервативным: P500 recertification плюс
  bounded-overload evidence, только если свежие gate files это подтверждают.

## Current Truth

- [x] Current artifact bundle сейчас invalidated для production claim.
  Исторические P500 результаты и старые bundle files не сертифицируют новый
  jar/native/runtime набор автоматически.
- [x] Latest contended P500 diagnostic доходит до поверхности `500 bots`, но
  не является pass: red-сигналы остаются по TPS, MSPT, host steal и
  kicks/connection stability.
- [x] Нет literal unlimited claim. Нельзя заявлять бесконечных players, mobs,
  chunks, ticks, CPU, RAM, IO или network.
- [x] Нет full Rust runtime claim. Rust/JNI части могут быть ускорителями,
  parity-модулями или диагностикой, но это не полный Paper runtime на Rust.
- [x] Plugins и datapacks остаются stress input и compatibility surface. Их не
  надо оптимизировать, упрощать или патчить ради прохождения P500.
- [x] Contended-host diagnostic может подсказать bottleneck, но сам по себе не
  восстанавливает production claim.
- [x] Любая новая формулировка должна ссылаться на exact artifact hashes,
  exact profile, raw logs, summaries, gate reports и claim validator output.

## Immediate Actions For This Turn

- [ ] Найти один low-risk core hot path, который можно поправить без изменения
  plugin/datapack behavior и без ослабления gate thresholds.
- [ ] Применить минимальную правку только в core/runtime path, где semantics
  очевидны или уже покрыты тестом/compile contract.
- [ ] Скомпилировать релевантный target, минимум `:paper-server:compileJava`
  или ближайший более дешёвый корректный compile gate.
- [ ] Пересобрать optimized/current artifact, если compile прошёл и правка
  должна попасть в bundle.
- [ ] Обновить и проверить artifact reports: `reports/artifacts.json`,
  `reports/artifact-hashes.txt`, hash check и related summaries.
- [ ] Запустить самый дешёвый честный smoke/diagnostic для P500 path, который
  подтверждает, что новый artifact стартует и не ломает базовый load harness.
- [ ] Обновить reports/hashes только свежими результатами текущего артефакта.
- [ ] Оставить production/P500 claims blocked, пока свежий evidence bundle и
  claim assertion не станут зелёными.

## P0: Current Artifact Recertification Gate

- [ ] `applyPatches` или эквивалентное состояние patch stack чистое для
  текущего working tree.
- [ ] `:paper-server:compileJava` проходит после новой правки.
- [ ] Optimized jar/native/runtime artifacts пересобраны из текущего source
  state.
- [ ] `reports/artifacts.json` отражает exact current jar, runtime launcher,
  native library, AppCDS, mappings и remap classpath.
- [ ] `sha256sum -c reports/artifact-hashes.txt` проходит.
- [ ] Старый `production-500-readiness-bundle-current` не считается валидным,
  если artifact hashes не совпадают с текущими.
- [ ] Claim assertion остаётся red/blocked до fresh cold+warm, repeat quorum и
  bundle validation на этом же current artifact.

## P1: P500 Path Fresh Evidence

- [ ] Clean-host или явно labeled contended-host status записан перед run.
- [ ] Host synthetic canary и strict foreign-process preflight не подменяются
  ручным override для production claim.
- [ ] P500 diagnostic или smoke достигает измеримой поверхности: bots created,
  connected, ready, active и action gate записаны в summary.
- [ ] TPS/MSPT thresholds указаны численно в gate report, а не описаны словами.
- [ ] Kicks/disconnect/protocol-error counters присутствуют в summary и gate.
- [ ] Steal/iowait/system load присутствуют в resource evidence.
- [ ] Failed diagnostic сохраняется как failure evidence, не как claim.
- [ ] Green P500 claim возможен только после same-artifact cold+warm,
  repeat quorum, plugin matrix, restart/recovery, forced-ticket persistence,
  self-contained bundle validation и claim assertion.

## P2: Bounded Overload Evidence

- [ ] Outbound packet send debt имеет bounded metric: queue depth, age,
  dropped/deferred policy или per-player cohort budget.
- [ ] Chunk send pressure имеет bounded metric: chunks per tick/window,
  backlog ceiling, throttling behavior и recovery after burst.
- [ ] Entity tracker/fanout pressure имеет bounded metric: recipients,
  refresh cost, packet fanout debt и tick impact.
- [ ] Region IO/writeback pressure имеет bounded metric: pending writes,
  flush latency, save pressure и post-restart validation.
- [ ] Slow-client или packet-burst scenario показывает throttling/backpressure
  до runaway memory growth.
- [ ] Chunk/worldgen pressure деградирует через explicit budget, а не через
  watchdog loops, sync-load storms или silent desync.
- [ ] Overload report называет ceiling как CPU, RAM, IO, network, JVM,
  protocol, host contention или explicit policy limit.
- [ ] Recovery path после overload проверяет world state, tickets, playerdata
  и claim-scope correctness на том же artifact.

## Explicit Non-Claims

- [x] No unlimited players.
- [x] No unlimited mobs.
- [x] No unlimited chunks, worlds or loaded regions.
- [x] No unlimited ticks, TPS, CPU, RAM, disk, IO or network.
- [x] No full Rust Paper runtime.
- [x] No real-player parity from bot data alone.
- [x] No multi-hour soak unless that exact soak was measured and reported.
- [x] No production claim from contended-host diagnostics.
- [x] No production claim from stale bundles or mismatched artifact hashes.
- [x] No plugin/datapack optimization claim.
- [x] No higher-tier claim from a P500-only pass.

## Definition Of Done

- [ ] Low-risk core hot-path patch is compiled.
- [ ] Current artifact is rebuilt or explicitly proven unchanged.
- [ ] Artifact hashes and reports are refreshed and verify cleanly.
- [ ] Cheapest useful P500 smoke/diagnostic has fresh output for this
  artifact.
- [ ] Reports state whether the run reached `500 bots` and whether it failed
  or passed TPS/MSPT/steal/kicks gates.
- [ ] Production claim stays blocked unless fresh same-artifact evidence
  satisfies every required gate.
- [ ] Bounded-overload work has at least one concrete metric path ready for the
  next P500 diagnostic, not just a prose promise.
