# P250 Warm-Source Recovery Goal

Date: 2026-05-21 CEST

This is the next honest giant step for the current artifact. The last fresh
P250 mixed-gameplay run reached full online and opened the action gate, but it
still failed the load window and watchdog gate. The target here is to recover
that same artifact to a green P250 warm-source mixed-gameplay gate, then only
promote to P500 after repeat quorum, restart/recovery, forced-ticket
persistence, and a validated evidence bundle are all green on the same
artifact.

## Goal

Get the current artifact to a green P250 stress-corpus mixed-gameplay
warm-source gate, then prove repeat quorum, restart/recovery, forced-ticket
persistence, and a validated self-contained evidence bundle before stepping to
P500.

## Current Evidence

- [x] The current artifact already has a measured P100 mixed-gameplay gate on
  the current codebase.
- [x] The current artifact already has fresh 50-bot and 100-bot stress-corpus
  mixed-gameplay passes.
- [x] The current artifact already has a fresh P250 run that reached
  `250/250` online and opened the action gate.
- [x] That same fresh P250 run is still red on load-window and watchdog
  metrics, so it is not claim-ready.
- [x] A historical `500 bots / 32 view / 32 simulation / creative block`
  claim exists, but only for an older verified artifact.
- [ ] The current artifact does not yet have a fresh green P250 warm-source
  gate.
- [ ] The current artifact does not yet have a P250 warm-source repeat quorum.
- [ ] The current artifact does not yet have P250 restart/recovery evidence.
- [ ] The current artifact does not yet have P250 forced-ticket persistence
  evidence.
- [ ] The current artifact does not yet have a validated self-contained
  evidence bundle for this ladder step.

## Acceptance Gates

- [ ] P250 warm-source gate passes on the current artifact.
- [ ] P250 warm-source load window stays inside the declared TPS/MSPT budget.
- [ ] P250 warm-source repeat quorum passes on the same artifact.
- [ ] P250 restart/recovery passes on the same artifact.
- [ ] P250 forced-ticket persistence passes on the same artifact.
- [ ] P250 evidence bundle validates and contains logs, summaries, gate
  reports, resource data, artifact hashes, runtime hash, and native library
  hash.

## Promotion To P500

- [ ] P500 fresh-world gate reaches `500/500` online, ready, and active.
- [ ] P500 warm-source gate passes on the current artifact.
- [ ] P500 cold+warm repeat quorum exists.
- [ ] P500 2h cold+warm soak exists.
- [ ] P500 publication bundle exists.

## Longer Ladder

- [ ] P750 diagnostic exists only after P500 is green.
- [ ] P1000 diagnostic exists only after P750 shows headroom.
- [ ] M10k mixed mobs diagnostic exists only with an accepted player tier.
- [ ] C10k loaded chunks diagnostic exists only with forced-ticket evidence.
- [ ] 24h soak exists only after a leading tier is accepted.

## Scale Discipline

- [ ] Do not promote to P500 while the current P250 warm-source gate is red.
- [ ] Do not raise mob or chunk tier without an accepted player tier.
- [ ] Do not reuse stale artifact bundles for current-artifact claims.
- [ ] Do not merge diagnostic and accepted results in the same claim.
- [ ] Do not publish a higher tier if host contention makes the run
  non-reproducible.

## Non-Claims

- [x] No literal unlimited players, mobs, chunks, ticks, plugins, or
  datapacks.
- [x] No full Rust Paper runtime claim.
- [x] No arbitrary plugin compatibility without a matrix gate.
- [x] No arbitrary datapack/worldgen compatibility without a matrix gate.
- [x] No real-player parity without live client evidence.
- [x] No claim from stale evidence.
