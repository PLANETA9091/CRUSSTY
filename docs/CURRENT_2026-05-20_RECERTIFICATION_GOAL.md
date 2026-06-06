# Current Recertification Goal

Date: 2026-05-20 CEST

This is the current working goal for the Paper 1.21.10 optimization loop.
Do not claim "unlimited" anything. Keep the language tied to measured gates,
artifact hashes, and repeatable evidence.

## Objective

Raise the current artifact from a verified protocol-safe smoke to a fresh
mixed-gameplay P100 gate, then continue the ladder only after the numbers stay
green.

## Verified Facts

- [x] Optimized runtime artifact was rebuilt and hashed.
- [x] The direct join regression was fixed by using `HandlerNames.OUTBOUND_CONFIG`
  instead of looking for a missing `encoder` handler.
- [x] Fresh 10-bot mixed-gameplay smoke reached `10/10` online with
  `0` kicks, `0` bot errors, and no `Missing outbound protocol handler`.
- [x] Fresh 10-bot mixed-gameplay smoke opened the action gate and exercised
  block place/break plus mixed command/item/animation paths.
- [x] The current 10-bot smoke still fails the strict TPS gate, so it is
  evidence, not a claim.

## Current Open Work

- [ ] Fresh P100 mixed-gameplay gate passes on the current artifact.
- [ ] P250 mixed-gameplay diagnostic exists on the current artifact.
- [ ] P500 mixed-gameplay diagnostic exists on the current artifact.
- [ ] P500 mixed-gameplay gate passes on the current artifact.
- [ ] Cold + warm repeat quorum exists for the leading accepted mixed tier.
- [ ] Restart/recovery under load exists for the leading accepted mixed tier.
- [ ] Forced-ticket persistence is validated under load.
- [ ] Self-contained evidence bundle exists for the leading accepted tier.

## Entity And Worldgen Ladder

- [ ] Mixed mobs scale beyond the current 150-mob stress corpus.
- [ ] Pathfinding budget stays inside the accepted tier.
- [ ] Chunk generation queue backpressure is measured at the next tier.
- [ ] Chunk send queue backpressure is measured at the next tier.
- [ ] Datapack worldgen compatibility stays intact at the next tier.

## Claim Rule

Only write a production-ready claim after the fresh summary, gate report, logs,
resource data, artifact hashes, and evidence bundle are all present and green.
Anything else is a work-in-progress.
