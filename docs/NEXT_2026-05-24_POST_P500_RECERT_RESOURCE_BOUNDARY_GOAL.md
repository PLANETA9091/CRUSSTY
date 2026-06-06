# NEXT 2026-05-24: Post-P500 Recert Resource Boundary Goal

This is the next goal after the current P500 recertification work, not a
replacement for it.

The target is narrow on purpose: once the current artifact is recertified at
the measured P500 workload, the next frontier is controlled degradation under
resource pressure. The work here is about outbound send backpressure, chunk
send budgeting, region IO/recovery, and self-contained evidence bundles.

## Starting Truth

- [x] Literal unlimited players is not a claim.
- [x] Literal unlimited mobs is not a claim.
- [x] Literal unlimited CPU is not a claim.
- [x] Literal unlimited RAM is not a claim.
- [x] The current blocking milestone is still P500 recertification on the
  current artifact.
- [x] Higher-scale wording is invalid unless the send path, chunk send path,
  and region IO path stay bounded and recoverable.
- [x] The next accepted statement must be resource-aware and evidence-bound,
  not marketing language.

## Work Ladder

- [ ] P500 recertification is green on the current artifact and host.
- [ ] Add fresh measurement for outbound send debt under the accepted P500
  workload.
- [ ] Prove slow-client or burst-client pressure triggers bounded
  backpressure before runaway queue growth.
- [ ] Establish a chunk send budget with observable limits per tick, per
  player cohort, and per stress window.
- [ ] Prove chunk send pressure degrades through budgeting or throttling
  instead of collapse, watchdog churn, or silent desync.
- [ ] Measure region writeback pressure under sustained chunk mutation on the
  accepted tier.
- [ ] Prove restart/recovery after stressed region IO preserves world state,
  ticket state, and claim-scope correctness.
- [ ] Build one self-contained evidence bundle for the accepted P500 tier with
  the new backpressure and IO proofs included.
- [ ] Only after the above is green, open the next diagnostic tier for higher
  players, mobs, or chunk pressure.

## Evidence Contract

- [ ] `bundle.json` names the exact artifact, runtime, host window, and load
  profile.
- [ ] `MANIFEST.txt` lists every required evidence file and hash.
- [ ] `CLAIM.md` states the accepted tier and the explicit non-claims.
- [ ] Raw logs for send path, chunk send path, and region IO are present.
- [ ] Resource CSVs or equivalent summaries show queue depth, tick cost, IO
  pressure, and recovery outcome.
- [ ] Recovery evidence includes pre-restart state, restart logs, and
  post-restart validation on the same artifact.
- [ ] The bundle is self-contained enough to re-check the claim without
  guessing missing context from the working tree.

## Strict Non-Claims

- [ ] No claim of literal unlimited players.
- [ ] No claim of literal unlimited mobs.
- [ ] No claim of literal unlimited CPU headroom.
- [ ] No claim of literal unlimited RAM headroom.
- [ ] No claim that chunk sends are free or unbounded.
- [ ] No claim that region IO cannot stall or fail.
- [ ] No claim of recovery correctness without fresh restart evidence.
- [ ] No claim beyond the exact measured artifact, workload, and host-quality
  window.

## Definition Of Done

- [ ] P500 recertification is complete on the current artifact.
- [ ] Outbound send backpressure is measured and shown bounded at the accepted
  tier.
- [ ] Chunk send budgeting is measured, explicit, and shown to prevent runaway
  send debt at the accepted tier.
- [ ] Region IO stress and restart/recovery are measured and shown correct at
  the accepted tier.
- [ ] One self-contained evidence bundle exists and validates the exact claim
  wording.
- [ ] The final wording stays conservative: resource-aware scaling on a
  measured tier, not literal unlimited scale.
