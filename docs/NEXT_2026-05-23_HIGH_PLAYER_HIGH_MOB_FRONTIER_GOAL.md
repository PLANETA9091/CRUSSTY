# Next 2026-05-23: high-player / high-mob core frontier

This is the next real goal after the current P500 recovery line on the rebuilt
artifact set recorded in `reports/artifacts.json`.

Literal unlimited is not a valid claim. The target is to remove the next core
bottlenecks so the ceiling is defined by measured hardware, network, disk, or
explicit policy, not by obvious tracker, packet, lookup, or queue churn.

Plugins and datapacks stay stress inputs. Do not update, remove, patch, or
optimize them to make a gate pass. The optimization surface is the core:
Paper runtime, Java hot paths, Rust/JNI native modules, entity tracking,
packet fanout, collision/nearby lookup, chunk send backpressure, and
validation tooling.

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

## Claim Shape

Only claim a tier after its summary, gate report, logs, resource data, artifact
hashes, and evidence bundle are present and green on the current artifact
recorded in `reports/artifacts.json`.

> production-ready for the measured high-player / high-mob tier on the
> verified current artifact recorded in `reports/artifacts.json`, with the
> exact bot count, view/simulation distance, mob pressure, chunk pressure,
> restart/recovery scope, plugin and datapack corpus, and exact non-claims
> stated.

Do not claim:

- literal unlimited players, mobs, chunks, ticks, plugins, or datapacks
- full Rust Paper runtime
- real-player parity without live client evidence
- plugin compatibility beyond the tested matrix
- datapack compatibility beyond the tested corpus
- a higher tier because a lower tier passed
- a benchmark result from a noisy host as a clean production claim

## Current Frontier From Audit

- [x] `ChunkMap.newTrackerTick` / tracker membership refresh and purge are hot
  frontier candidates.
- [x] `ServerEntity.sendChanges` / packet fanout from tracked entities is a hot
  frontier candidate.
- [x] `NearbyPlayers` / nearest-player and spawn-radius lookup is a hot
  frontier candidate.
- [x] `ChunkEntitySlices.getEntities` / `CollisionUtil.merge` / collision and
  nearby-entity lookup are hot frontier candidates.
- [x] Current P500 evidence is still blocked by bundle freshness and host
  contention, not by a proven tracker bug.

## Immediate Ladder

- [ ] P500 current-artifact claim is restored and published.
- [ ] Tracker membership refresh/purge cost is bounded and measured.
- [ ] Dirty tracked-entity packet fanout is bounded and measured.
- [ ] Nearby-player and collision lookup cost is bounded and measured.
- [ ] Chunk send queue and backpressure stay bounded under spread players.
- [ ] P750 mixed gameplay cold+warm gate passes on the verified current artifact.
- [ ] P1000 mixed gameplay diagnostic passes if host capacity allows.
- [ ] M10k mixed mobs pass with bounded AI/pathfinding.
- [ ] M25k mixed mobs diagnostic passes if host capacity allows.
- [ ] Combined player + mob + chunk + plugin gate passes on the leading tier.
- [ ] Restart/recovery and forced-ticket persistence pass on the leading tier.
- [ ] Long soak passes on the leading tier.
- [ ] A self-contained evidence bundle is regenerated and validated.
- [ ] A stable current publication file exists for the accepted tier.

## Working Rule

- [x] No claim without gate.
- [x] No broad claim from microbench alone.
- [x] No plugin/datapack optimization to pass the core gate.
- [x] No "real players" claim without live client evidence.
- [x] No "full Rust Paper runtime" claim.
- [ ] Every new scale claim must have a timestamped evidence bundle.
- [ ] Every new scale claim must have a stable current publication file.
- [ ] Every new scale claim must include exact non-claims.

## Core Optimization Frontier

- [ ] entity tracker membership refresh / purge
- [ ] packet fanout for tracked entities
- [ ] nearby-player and spawn-radius lookup
- [ ] collision lookup and nearby-entity scan path
- [ ] chunk send backpressure and queue budgeting
- [ ] restart / recovery under high entity churn
- [ ] forced-ticket persistence under high chunk load
- [ ] worldgen / density work only where it is still a proven hot path

## Definition Of Done

This goal is real only when the leading accepted tier can be defended with
fresh evidence and the next limiter is clearly outside the current core hot
paths.

- [ ] The leading accepted tier has a fresh green cold+warm gate.
- [ ] The leading accepted tier has repeat quorum evidence.
- [ ] The leading accepted tier has plugin matrix evidence.
- [ ] The leading accepted tier has restart/recovery evidence.
- [ ] The leading accepted tier has forced-ticket persistence evidence.
- [ ] The leading accepted tier has a validated self-contained bundle.
- [ ] The next ceiling is a measured hardware, network, disk, or policy limit.
