# P500 Recovery And Scale Goal

Date: 2026-05-20 CEST

This is the next honest goal after the failed current-artifact 500 cold soak.
Literal unlimited players, mobs, chunks, ticks, plugins, or datapacks are not
a real claim. The real target is measured scale with explicit gate evidence.

## Allowed Claim Shape

Only claim a tier after fresh summary, gate report, logs, resource data,
artifact hashes, repeat evidence, restart/recovery evidence, forced-ticket
evidence, and a self-contained evidence bundle are all green.

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

## Current Verified State

- [x] Historical `500 bots / 32 view / 32 simulation / creative block`
  production-ready claim exists on a verified artifact.
- [x] Heavy stress corpus exists with `26` total plugin jars and `10`
  datapacks.
- [x] Mixed-gameplay workload exists with movement, block place/break, held
  item switches, arm animation, player input, use-item, commands, plugin
  counters, datapacks, and mob pressure.
- [x] Current artifact cold soak run was recorded and failed honestly.
- [x] Current failed run reached `500` created / `500` ready, then fell to
  `364` active max with keepalive timeouts and socket closes.
- [x] The latest regression was localized to the configuration-finish protocol
  switch path in `0125`.
- [x] A direct `0125` p100 run already showed `Missing outbound protocol
  handler` on the async protocol-switch path.

## Current Active Ladder

- [ ] Restore a fresh green `500 bots / 32 view / 32 simulation / creative
  block` cold soak on the current artifact.
- [ ] Restore warm soak on the current artifact.
- [ ] Restore repeat quorum on the current artifact.
- [ ] Restore plugin matrix on the current artifact.
- [ ] Restore restart/recovery on the current artifact.
- [ ] Restore forced-ticket persistence on the current artifact.
- [ ] Export and validate a fresh self-contained evidence bundle.
- [ ] Publish the refreshed current claim only after all of the above are
  green.
- [ ] Raise to `P500` mixed-gameplay gate on the same artifact.
- [ ] Raise to `P750` mixed-gameplay diagnostic if `P500` goes green.
- [ ] Raise to `M10k` mixed mobs with an accepted player tier.
- [ ] Raise to `C10k` loaded chunks with an accepted player tier.

## Working Rule

If a tier fails, the next task is to use the failure evidence to fix the real
bottleneck. Do not skip from a red 500 cold soak to a broader claim.
