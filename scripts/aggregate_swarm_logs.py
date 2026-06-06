#!/usr/bin/env python3
"""Aggregate sharded mc_bot_swarm logs into one gate-readable bot log."""

from __future__ import annotations

import argparse
import pathlib
import re
from dataclasses import dataclass


TOKEN_RE = re.compile(r"([A-Za-z][A-Za-z0-9]*)=([^ ]+)")


INT_SUM_FIELDS = (
    "created",
    "connected",
    "ready",
    "active",
    "ended",
    "kicked",
    "errors",
    "positions",
    "chunks",
    "blockArmed",
    "blockPrimed",
    "blockCreativeSlotPackets",
    "blockPlacePackets",
    "blockDigPackets",
    "blockActionErrors",
    "blockActionReady",
    "mixedActionTicks",
    "mixedHeldItemPackets",
    "mixedArmAnimationPackets",
    "mixedPlayerInputPackets",
    "mixedUseItemPackets",
    "mixedCommandPackets",
    "mixedBlockPlacePackets",
    "mixedBlockDigPackets",
    "mixedAttackPackets",
    "mixedActionErrors",
)
FLOAT_SUM_FIELDS = (
    "positionsPerSec",
    "chunksPerSec",
    "blockActionsPerSec",
    "mixedActionsPerSec",
)


@dataclass(frozen=True)
class Event:
    timestamp: str
    shard: int
    kind: str
    line: str
    tokens: dict[str, str]


def parse_tokens(line: str) -> dict[str, str]:
    return dict(TOKEN_RE.findall(line))


def as_int(tokens: dict[str, str], key: str) -> int:
    try:
        return int(float(tokens.get(key, "0")))
    except ValueError:
        return 0


def as_float(tokens: dict[str, str], key: str) -> float:
    try:
        return float(tokens.get(key, "0"))
    except ValueError:
        return 0.0


def read_events(paths: list[pathlib.Path]) -> list[Event]:
    events: list[Event] = []
    for shard, path in enumerate(paths):
        try:
            lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
        except FileNotFoundError:
            continue
        for line in lines:
            if not line:
                continue
            timestamp = line.split(" ", 1)[0]
            if "swarm_metrics" in line:
                events.append(Event(timestamp, shard, "metrics", line, parse_tokens(line)))
            elif "swarm_action_gate_ready" in line:
                events.append(Event(timestamp, shard, "gate_ready", line, parse_tokens(line)))
            elif "swarm_action_gate_open" in line:
                events.append(Event(timestamp, shard, "gate_open", line, parse_tokens(line)))
            elif "swarm_action_gate_reset" in line:
                events.append(Event(timestamp, shard, "gate_reset", line, {}))
            elif (
                "bot_error" in line
                or "bot_kick" in line
                or "bot_end" in line
                or "swarm_shutdown" in line
            ):
                events.append(Event(timestamp, shard, "tail", line, {}))
    return sorted(events, key=lambda event: (event.timestamp, event.shard, event.kind))


def sum_int(latest: dict[int, dict[str, str]], key: str) -> int:
    return sum(as_int(tokens, key) for tokens in latest.values())


def sum_float(latest: dict[int, dict[str, str]], key: str) -> float:
    return sum(as_float(tokens, key) for tokens in latest.values())


def emit_metrics(
    timestamp: str,
    latest: dict[int, dict[str, str]],
    *,
    mode: str,
    action_mode: str,
    required: int,
) -> str:
    opened_values = [as_int(tokens, "actionGateOpenedMs") for tokens in latest.values()]
    ready_since_values = [as_int(tokens, "actionGateReadySinceMs") for tokens in latest.values()]
    gate_open = bool(latest) and all(tokens.get("actionGate") == "open" for tokens in latest.values())
    opened_ms = max([value for value in opened_values if value >= 0], default=-1) if gate_open else -1
    ready_since_ms = min([value for value in ready_since_values if value >= 0], default=-1)
    fields = [
        "swarm_metrics",
        f"mode={mode}",
        f"created={sum_int(latest, 'created')}",
        f"connected={sum_int(latest, 'connected')}",
        f"ready={sum_int(latest, 'ready')}",
        f"active={sum_int(latest, 'active')}",
        f"ended={sum_int(latest, 'ended')}",
        f"kicked={sum_int(latest, 'kicked')}",
        f"errors={sum_int(latest, 'errors')}",
        f"positions={sum_int(latest, 'positions')}",
        f"positionsPerSec={sum_float(latest, 'positionsPerSec'):.1f}",
        f"chunks={sum_int(latest, 'chunks')}",
        f"chunksPerSec={sum_float(latest, 'chunksPerSec'):.1f}",
        f"actionGateMode={action_mode}",
        f"actionGate={'open' if gate_open else 'waiting'}",
        f"actionGateRequired={required}",
        f"actionGateReady={sum_int(latest, 'actionGateReady')}",
        f"actionGateActive={sum_int(latest, 'actionGateActive')}",
        f"actionGateSettled={sum_int(latest, 'actionGateSettled')}",
        f"actionGateOpenedMs={opened_ms}",
        f"actionGateReadySinceMs={ready_since_ms}",
    ]
    if any("blockArmed" in tokens for tokens in latest.values()):
        fields.extend(
            [
                f"blockArmed={sum_int(latest, 'blockArmed')}",
                f"blockPrimed={sum_int(latest, 'blockPrimed')}",
                f"blockCreativeSlotPackets={sum_int(latest, 'blockCreativeSlotPackets')}",
                f"blockPlacePackets={sum_int(latest, 'blockPlacePackets')}",
                f"blockDigPackets={sum_int(latest, 'blockDigPackets')}",
                f"blockActionErrors={sum_int(latest, 'blockActionErrors')}",
                f"blockActionsPerSec={sum_float(latest, 'blockActionsPerSec'):.1f}",
                f"blockActionReady={sum_int(latest, 'blockActionReady')}",
            ]
        )
    if any("mixedActionTicks" in tokens for tokens in latest.values()):
        fields.extend(
            [
                f"mixedActionTicks={sum_int(latest, 'mixedActionTicks')}",
                f"mixedHeldItemPackets={sum_int(latest, 'mixedHeldItemPackets')}",
                f"mixedArmAnimationPackets={sum_int(latest, 'mixedArmAnimationPackets')}",
                f"mixedPlayerInputPackets={sum_int(latest, 'mixedPlayerInputPackets')}",
                f"mixedUseItemPackets={sum_int(latest, 'mixedUseItemPackets')}",
                f"mixedCommandPackets={sum_int(latest, 'mixedCommandPackets')}",
                f"mixedBlockPlacePackets={sum_int(latest, 'mixedBlockPlacePackets')}",
                f"mixedBlockDigPackets={sum_int(latest, 'mixedBlockDigPackets')}",
                f"mixedAttackPackets={sum_int(latest, 'mixedAttackPackets')}",
                f"mixedActionErrors={sum_int(latest, 'mixedActionErrors')}",
                f"mixedActionsPerSec={sum_float(latest, 'mixedActionsPerSec'):.1f}",
            ]
        )
    return f"{timestamp} {' '.join(fields)}"


def sum_gate_tokens(events: dict[int, Event], key: str) -> int:
    return sum(as_int(event.tokens, key) for event in events.values())


def emit_gate_event(
    timestamp: str,
    events: dict[int, Event],
    *,
    kind: str,
    mode: str,
    required: int,
    settle_ms: int,
) -> str:
    opened_after_ms = max((as_int(event.tokens, "openedAfterMs") for event in events.values()), default=0)
    ready_since_ms = max((as_int(event.tokens, "readySinceMs") for event in events.values()), default=0)
    fields = [f"mode={mode}"]
    if kind == "gate_open":
        fields.append(f"openedAfterMs={opened_after_ms}")
        fields.append(f"readySinceMs={ready_since_ms}")
    fields.extend(
        [
            f"created={sum_gate_tokens(events, 'created')}",
            f"connected={sum_gate_tokens(events, 'connected')}",
            f"ready={sum_gate_tokens(events, 'ready')}",
            f"active={sum_gate_tokens(events, 'active')}",
            f"settled={sum_gate_tokens(events, 'settled')}",
            f"required={required}",
            f"settleMs={settle_ms}",
        ]
    )
    if kind == "gate_open":
        min_delay_ms = max((as_int(event.tokens, "minDelayMs") for event in events.values()), default=0)
        fields.append(f"minDelayMs={min_delay_ms}")
        name = "swarm_action_gate_open"
    else:
        name = "swarm_action_gate_ready"
    return f"{timestamp} {name} {' '.join(fields)}"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--count", type=int, required=True)
    parser.add_argument("--mode", required=True)
    parser.add_argument("--action-mode", required=True)
    parser.add_argument("--settle-ms", type=int, required=True)
    parser.add_argument("logs", nargs="+", type=pathlib.Path)
    args = parser.parse_args()

    events = read_events(args.logs)
    latest_metrics: dict[int, dict[str, str]] = {}
    gate_ready_events: dict[int, Event] = {}
    gate_open_events: dict[int, Event] = {}
    emitted_ready = False
    emitted_open = False
    tail_lines: list[str] = []

    print(
        f"{events[0].timestamp if events else ''} swarm_start "
        f"count={args.count} shards={len(args.logs)} mode={args.mode} actionStartMode={args.action_mode}"
    )
    for event in events:
        if event.kind == "metrics":
            latest_metrics[event.shard] = event.tokens
            print(
                emit_metrics(
                    event.timestamp,
                    latest_metrics,
                    mode=args.mode,
                    action_mode=args.action_mode,
                    required=args.count,
                )
            )
        elif event.kind == "gate_ready":
            gate_ready_events[event.shard] = event
            if not emitted_ready and len(gate_ready_events) == len(args.logs):
                print(
                    emit_gate_event(
                        event.timestamp,
                        gate_ready_events,
                        kind="gate_ready",
                        mode=args.action_mode,
                        required=args.count,
                        settle_ms=args.settle_ms,
                    )
                )
                emitted_ready = True
        elif event.kind == "gate_open":
            gate_open_events[event.shard] = event
            if not emitted_open and len(gate_open_events) == len(args.logs):
                print(
                    emit_gate_event(
                        event.timestamp,
                        gate_open_events,
                        kind="gate_open",
                        mode=args.action_mode,
                        required=args.count,
                        settle_ms=args.settle_ms,
                    )
                )
                emitted_open = True
        elif event.kind == "gate_reset":
            print(event.line)
        elif event.kind == "tail":
            tail_lines.append(event.line)

    for line in tail_lines[-500:]:
        print(line)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
