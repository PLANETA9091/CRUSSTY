# Current 2026-05-23 CEST: current-artifact P500 recertification, then measured tiers

This is the active goal for `/root/rust`.

Literal unlimited is not a claim. The goal is to restore the measured P500
floor on the current artifact recorded in `reports/artifacts.json`, then move
upward only when each new tier has fresh same-artifact evidence.

Plugins and datapacks are compatibility and stress inputs, not the optimization
target. The optimization surface stays in core runtime, scheduling,
backpressure, IO, and evidence tooling.

Execution rule: run the fastest honest path available. That means parallel
checks, no waiting on sidecar work when the next local step is known, and no
production claim attempt while the host is contaminated.

## Current Artifact And Gate State

- [x] Current optimized Paper artifact:
  `d84e8c7a7e78fd46f286906029a673b2f973e1f4d9bf8695be88914a14a07989`.
- [x] Current optimized runtime launcher:
  `108e51a63a97739964438c2dcba169e3d66889d454b0f7e049beee4614568f6c`.
- [x] Current native runtime library:
  `2921d341ebe33a44fd572499f0b6fdb25920f5d11a17d8875ecd69a29a374051`.
- [x] Current AppCDS archive:
  `df6452f175dd9994efec0349aaa36da080dc12c384fd3184d1632c2a36a2cb81`.
- [x] Current runtime jar SHA256 file:
  `c339f735d95159a8e77c4d130a19f08844df6206986c18cacded8a759af8434e`.
- [x] Current native runtime library SHA256 file:
  `a4b59dee68265da1a7a078ea97276bfcc488a0f0f3701c78c3ade9be3978d42f`.
- [x] Current reversed mappings:
  `22ed1982f708a0526fc7d94ae1b5c4fbd99119fdb24fd0c82dc0ef28c479b086`
  under remap id
  `C997B50A9A660D45B81CDE45378185704EA319A244D471BB23CCC224B40E2BE0`.
- [x] Current remap classpath jar:
  `b9cdd5ac39c18d41a6971eff898797b1fe31172f51f77216c7429ee4f28da7e2`
  under remap classpath id
  `C997B50A9A660D45B81CDE45378185704EA319A244D471BB23CCC224B40E2BE0`.
- [x] Native NormalNoise direct JNI output is shipped in the current native
  runtime library.
- [x] Current artifact manifests are verified and green after the latest
  rebuild by `python3 scripts/update_artifact_reports.py`,
  `python3 -m json.tool reports/artifacts.json`, and
  `sha256sum -c reports/artifact-hashes.txt`.
- [x] Historical P500 production-ready evidence exists for older artifacts.
- [x] That historical P500 evidence is not a current-artifact claim after the
  current jar/native rebuilds.
- [x] The current host is not eligible for a P500 claim run:
  `reports/host-synthetic-canary-live-20260523-fresh.txt` has
  `host_synthetic_canary_ok=false` and `steal_percent_max=24.53`.
- [x] The strict foreign-process preflight is also red:
  `reports/load-preflight-accel-20260523-production500-preflight.txt` reports
  `strict_foreign_process_gate_pass=false` because a `server.jar` process is
  running at PID `2871654` from `/var/lib/pufferpanel/servers/6a11c76a`.
- [x] The old current bundle is rejected against the current artifact:
  `reports/production-500-readiness-bundle-current-validation.txt`
  has `bundle_validation_pass=false`.
- [x] The production claim assertion is also rejected against the current
  artifact:
  `reports/production-500-claim-verdict.txt` has
  `claim_assertion_pass=false`.
- [x] Fresh same-artifact plugin matrix is green:
  `reports/plugin-matrix-summary.txt` covers 11 initialized plugins,
  precomputed remaps, `CompatProbe`, scheduler ticks, join/quit, and command
  coverage.
- [x] Fresh same-artifact restart/recovery is green:
  `reports/restart-recovery-summary.txt` covers plugin startup,
  `compatprobe`, `save-all flush`, graceful stop, and persisted region files.
- [x] Fresh same-artifact forced-ticket persistence is green:
  `reports/forced-ticket-persistence-summary.txt` has
  `forced_ticket_persistence=PASS` and both runtime logs clean.
- [ ] Current artifact P500 cold+warm gate is green.
- [ ] Current artifact repeat quorum is green.
- [x] Current artifact plugin matrix is fresh and green.
- [x] Current artifact restart/recovery is fresh and green.
- [x] Current artifact forced-ticket persistence is fresh and green.
- [ ] Current artifact has a regenerated and validated self-contained P500
  evidence bundle.

## Roadmap

- [ ] Reconfirm the current P500 floor on this artifact before any larger tier
  is counted.
- [ ] Regenerate the P500 evidence bundle only from fresh cold+warm, repeat
  quorum, restart/recovery, and forced-ticket runs.
- [ ] Raise the player ceiling to `P750` on the same measurement contract.
- [ ] Raise the player ceiling to `P1000` on the same measurement contract.
- [ ] Add `M1k` and `M5k` mixed-mob tiers with bounded AI/pathfinding.
- [ ] Add `C10k` and `C25k` chunk/worldgen tiers with the same evidence
  discipline.
- [ ] Prove mixed gameplay at the highest accepted tier only after the isolated
  ladders are green.
- [ ] Carry plugin-matrix validation into every accepted tier.
- [ ] Publish every accepted tier with hashes, raw logs, manifests, exact
  non-claims, and a stable current bundle.

## Evidence Contract

- [x] Every accepted tier needs fresh soak, repeat quorum, restart/recovery,
  forced-ticket persistence, plugin-matrix evidence, and a self-contained
  bundle.
- [x] Every accepted tier needs an exact claim and exact non-claims.
- [x] Every accepted tier needs a stable publication file and hash record.

## Hard Limits

- [x] No literal unlimited players, mobs, chunks, ticks, plugins, or datapacks.
- [x] No full Rust Paper runtime claim.
- [x] No plugin/datapack optimization claim.
- [x] No multi-hour soak claim unless that exact soak was measured.
- [x] No higher-tier claim from a lower-tier pass.
- [x] No production claim from a stale or incomplete bundle.
