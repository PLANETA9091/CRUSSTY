# Next Measured Giant Goal

Date: 2026-05-20 CEST

This is the next honest step after the current P250 fresh-world run reached
250/250 online and opened the action gate, but still failed the load-window
and watchdog gate on the first full-online window. The next goal is to prove
the same artifact on a warm-source P250 profile, then carry it through
repeat, restart/recovery, forced-ticket, and evidence-bundle checks before
moving to P500.

## Goal

Get the current artifact to a green P250 stress-corpus mixed-gameplay gate on
a warm source world, then carry that same artifact through repeat,
restart/recovery, forced-ticket persistence, and evidence-bundle checks
before stepping to P500.

## Checklist

- [x] P250 fresh-world gate reaches `250/250` online, ready, and active.
- [x] P250 action gate opens on the full tier.
- [x] P250 fresh-world load-window evidence exists.
- [ ] P250 warm-source gate passes on the current artifact.
- [ ] P250 warm-source repeat quorum exists.
- [ ] P250 restart/recovery and forced-ticket persistence evidence exists.
- [ ] P250 self-contained evidence bundle exists.
- [ ] P500 fresh-world gate reaches `500/500` online, ready, and active.
- [ ] P500 warm-source gate passes.
- [ ] P500 cold+warm repeat quorum exists.
- [ ] P500 2h cold+warm soak exists.
- [ ] P500 publication bundle exists.

## Fresh Blocker

The current fresh-world P250 run is still red:

- `gate_pass=false`
- `observed_tps1_avg=14.31`
- `observed_tps1_min=0.61`
- `observed_avg_tick_ms_avg=79.31`
- `observed_avg_tick_ms_max=915.73`
- `observed_watchdog_thread_dumps=12`
- `observed_external_thread_prints=10`

Thread samples point at the server thread in `ServerGamePacketListenerImpl`
movement handling, with `WaypointTransmitter` checks appearing inside the
same stall window. The log also shows `squaremap` background render threads
waiting on chunk snapshots, so the next run must separate cold preload from
steady-state pressure instead of pretending the fresh run was enough.

## Non-Claims

- [ ] literal unlimited scale
- [ ] full Rust Paper runtime
- [ ] arbitrary plugin compatibility without a matrix gate
- [ ] arbitrary datapack/worldgen compatibility without a matrix gate
- [ ] real-player parity without live client evidence
- [ ] multi-hour soak without fresh soak evidence
