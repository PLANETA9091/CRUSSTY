# REAL GOAL 2026-05-23: Resource-aware near-unbounded core scaling

Дата: 2026-05-23

Literal `any RAM/CPU/unlimited` невозможно физически. Железо, сеть, диск,
JVM, протокол Minecraft и клиентские соединения конечны.

Реальная цель здесь другая: resource-aware near-unbounded scaling. То есть
ядро должно расти до измеряемых ceilings без падений, без порчи данных, без
неограниченных очередей и без скрытых деградаций. Плагины и датапаки здесь
только стресс-входы, а не мишень для «подгонки» результата.

## Current Artifact Truth

- [x] Current rebuilt artifact truth is the set recorded in
  `reports/artifacts.json`.
- [x] Exact current hashes: optimized Paper
  `d84e8c7a7e78fd46f286906029a673b2f973e1f4d9bf8695be88914a14a07989`,
  runtime launcher
  `108e51a63a97739964438c2dcba169e3d66889d454b0f7e049beee4614568f6c`,
  native runtime
  `2921d341ebe33a44fd572499f0b6fdb25920f5d11a17d8875ecd69a29a374051`,
  AppCDS
  `df6452f175dd9994efec0349aaa36da080dc12c384fd3184d1632c2a36a2cb81`,
  runtime jar SHA256 file
  `c339f735d95159a8e77c4d130a19f08844df6206986c18cacded8a759af8434e`,
  native runtime library SHA256 file
  `a4b59dee68265da1a7a078ea97276bfcc488a0f0f3701c78c3ade9be3978d42f`,
  reversed mappings
  `22ed1982f708a0526fc7d94ae1b5c4fbd99119fdb24fd0c82dc0ef28c479b086`,
  and remap classpath
  `b9cdd5ac39c18d41a6971eff898797b1fe31172f51f77216c7429ee4f28da7e2`
  under id
  `C997B50A9A660D45B81CDE45378185704EA319A244D471BB23CCC224B40E2BE0`.
- [x] Any older hash set is historical only and is not current-artifact
  evidence.

## Current Truths

- [x] Literal unlimited claims запрещены.
- [x] Цель должна формулироваться через измеряемые ceilings, а не через
  обещание бесконечности.
- [x] Плагины и датапаки не оптимизируются ради прохождения gate; они
  используются как нагрузка и compatibility surface.
- [x] Главный критерий качества — отсутствие crashes, corruption, runaway
  queues, watchdog loops и silent data loss.
- [x] Любой accepted tier обязан иметь свежие доказательства, а не только
  хорошее намерение или microbench.

## Real Work

- [ ] Поднять player tiers до стабильных измеряемых уровней без падения TPS,
  без протокольных ошибок и без unbounded backlog.
- [ ] Поднять mob tiers так, чтобы AI, pathfinding, despawn, collision and
  tracker budget были bounded and observable.
- [ ] Поднять worldgen/chunk tiers так, чтобы generation, load, send, ticket,
  and save paths оставались bounded under stress.
- [ ] Закрепить network/compression path: send budgets, packet bursts,
  compression pressure, reconnect storms, and slow-client backpressure must
  degrade gracefully.
- [ ] Закрепить IO/recovery path: region writes, flush behavior, restart,
  forced recovery, disk pressure, and resume must remain deterministic.
- [ ] Собрать evidence bundle на каждый accepted tier.
- [ ] Зафиксировать strict non-claims в публикации и в итоговой формулировке.

## Player Tiers

- [ ] P500 mixed gameplay with stress corpus.
- [ ] P750 mixed gameplay diagnostic tier.
- [ ] P1000 mixed gameplay diagnostic tier.
- [ ] P1500+ only as diagnostic until a real hardware ceiling is measured.
- [ ] For each tier: no crash, no corruption, no unbounded queue growth, no
  watchdog failures.

## Mob Tiers

- [ ] M1k mixed mobs on accepted player tier.
- [ ] M5k mixed mobs on accepted player tier.
- [ ] M10k diagnostic tier with bounded AI and tracker cost.
- [ ] M25k diagnostic tier only if the host still has measurable headroom.
- [ ] Mob scaling must preserve Bukkit-visible behavior where applicable.

## Worldgen And Chunk Tiers

- [ ] Chunk generation queue must stay bounded under load.
- [ ] Chunk send queue must stay bounded under load.
- [ ] Ticket pressure must not create runaway retention.
- [ ] Sync chunk loads must remain disallowed in accepted production tiers.
- [ ] Datapacks remain stress inputs only; do not tune them to fake success.

## Network And Compression

- [ ] Backpressure must trigger before the server or client enters collapse.
- [ ] Compression cost must be measurable and bounded per accepted tier.
- [ ] Packet bursts must not create unbounded send debt.
- [ ] Reconnect storms must recover cleanly without corruption or stall.
- [ ] Slow-client handling must degrade, not cascade.

## IO And Recovery

- [ ] Region IO writeback must stay bounded under sustained mutation.
- [ ] Flush and fsync behavior must be observable in the evidence bundle.
- [ ] Restart/recovery must preserve state correctness after stress.
- [ ] Forced tickets and high chunk load must not cause data inconsistency.
- [ ] Disk pressure must lead to controlled slowdown, not silent failure.

## Evidence Bundle

- [ ] bundle.json for each accepted tier.
- [ ] MANIFEST.txt with exact artifact identity.
- [ ] CLAIM.md with precise wording and exact non-claims.
- [ ] Raw logs, resource CSVs, and runner metadata.
- [ ] Hashes for artifact, runtime, and any native components.
- [ ] Fresh validation against the exact claimed artifact.

## Strict Non-Claims

- [ ] No literal infinite RAM.
- [ ] No literal infinite CPU.
- [ ] No literal unlimited players.
- [ ] No literal unlimited mobs.
- [ ] No literal unlimited chunks or ticks.
- [ ] No literal unlimited plugin compatibility.
- [ ] No literal unlimited datapack compatibility.
- [ ] No claim of real-player parity without real-client evidence.
- [ ] No claim of soak, repeat quorum, or recovery unless actually measured.

## Definition Of Done

- [ ] Each accepted tier has a fresh evidence bundle.
- [ ] Every remaining limit is tied to a measured hardware, network, disk, or
  policy ceiling.
- [ ] The system degrades predictably under stress instead of failing
  catastrophically.
- [ ] Publication wording is exact, conservative, and production-oriented.
