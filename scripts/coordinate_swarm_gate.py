#!/usr/bin/env python3
"""Open a shared bot action gate after sharded swarm logs are globally ready."""

from __future__ import annotations

import argparse
import datetime as dt
import json
import math
import os
import pathlib
import re
import time
from dataclasses import dataclass


TOKEN_RE = re.compile(r"([A-Za-z][A-Za-z0-9]*)=([^ ]+)")


@dataclass
class ShardState:
    path: pathlib.Path
    position: int = 0
    partial: str = ""
    metrics: dict[str, str] | None = None
    metrics_timestamp: float = 0.0


def timestamp() -> str:
    return dt.datetime.now(dt.timezone.utc).isoformat(timespec="milliseconds").replace("+00:00", "Z")


def log(message: str) -> None:
    print(f"{timestamp()} {message}", flush=True)


def parse_tokens(line: str) -> dict[str, str]:
    return dict(TOKEN_RE.findall(line))


def parse_line_time(line: str) -> float:
    raw = line.split(" ", 1)[0]
    try:
        return dt.datetime.fromisoformat(raw.replace("Z", "+00:00")).timestamp()
    except ValueError:
        return time.time()


def as_int(tokens: dict[str, str], key: str) -> int:
    try:
        return int(float(tokens.get(key, "0")))
    except ValueError:
        return 0


def clamp(value: int, minimum: int, maximum: int) -> int:
    return min(maximum, max(minimum, value))


def required_count(args: argparse.Namespace) -> int:
    if args.action_mode == "all-ready":
        return args.count
    if args.action_mode == "ready-count":
        return clamp(args.ready_min_count or args.count, 1, args.count)
    if args.action_mode == "ready-fraction":
        return clamp(math.ceil(args.count * args.ready_min_fraction), 1, args.count)
    raise ValueError(f"unsupported action mode for shared gate: {args.action_mode}")


def poll_shard(state: ShardState) -> None:
    try:
        with state.path.open("r", encoding="utf-8", errors="replace") as handle:
            handle.seek(state.position)
            chunk = handle.read()
            state.position = handle.tell()
    except FileNotFoundError:
        return

    if not chunk:
        return

    data = state.partial + chunk
    state.partial = ""
    for part in data.splitlines(keepends=True):
        if not part.endswith(("\n", "\r")):
            state.partial = part
            continue
        line = part.rstrip("\r\n")
        if "swarm_metrics" not in line:
            continue
        state.metrics = parse_tokens(line)
        state.metrics_timestamp = parse_line_time(line)


def sum_field(metrics: list[dict[str, str]], *keys: str) -> int:
    for key in keys:
        if any(key in tokens for tokens in metrics):
            return sum(as_int(tokens, key) for tokens in metrics)
    return 0


def current_totals(states: list[ShardState], *, now: float, max_stale_seconds: float) -> tuple[dict[str, int], str]:
    ready_states = [state for state in states if state.metrics is not None]
    if len(ready_states) != len(states):
        return {}, f"metrics {len(ready_states)}/{len(states)}"

    stale = [
        state
        for state in ready_states
        if state.metrics_timestamp <= 0 or now - state.metrics_timestamp > max_stale_seconds
    ]
    if stale:
        return {}, f"freshMetrics {len(states) - len(stale)}/{len(states)}"

    metrics = [state.metrics for state in ready_states if state.metrics is not None]
    totals = {
        "created": sum_field(metrics, "created"),
        "connected": sum_field(metrics, "connected"),
        "ready": sum_field(metrics, "actionGateReady", "ready"),
        "active": sum_field(metrics, "actionGateActive", "active"),
        "settled": sum_field(metrics, "actionGateSettled", "settled"),
        "blockArmed": sum_field(metrics, "blockArmed"),
    }
    return totals, ""


def unmet_reason(totals: dict[str, int], *, required: int, requires_block_armed: bool) -> str:
    for key in ("created", "connected", "ready", "active", "settled"):
        if totals.get(key, 0) < required:
            return f"{key} {totals.get(key, 0)}/{required}"
    if requires_block_armed and totals.get("blockArmed", 0) < required:
        return f"blockArmed {totals.get('blockArmed', 0)}/{required}"
    return ""


def write_gate_file(path: pathlib.Path, payload: dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_name(f".{path.name}.tmp")
    tmp.write_text(json.dumps(payload, sort_keys=True) + "\n", encoding="utf-8")
    os.replace(tmp, path)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--count", type=int, required=True)
    parser.add_argument("--action-mode", required=True, choices=("all-ready", "ready-count", "ready-fraction"))
    parser.add_argument("--ready-min-count", type=int, required=True)
    parser.add_argument("--ready-min-fraction", type=float, required=True)
    parser.add_argument("--settle-ms", type=int, required=True)
    parser.add_argument("--requires-block-armed", choices=("true", "false"), required=True)
    parser.add_argument("--gate-file", type=pathlib.Path, required=True)
    parser.add_argument("--poll-ms", type=int, default=250)
    parser.add_argument("--metric-stale-ms", type=int, default=20000)
    parser.add_argument("--timeout-ms", type=int, default=0)
    parser.add_argument("logs", nargs="+", type=pathlib.Path)
    args = parser.parse_args()

    if args.count < 1:
        raise SystemExit("--count must be positive")
    if args.poll_ms < 50:
        raise SystemExit("--poll-ms must be >= 50")
    if args.metric_stale_ms < args.poll_ms:
        raise SystemExit("--metric-stale-ms must be >= --poll-ms")

    required = required_count(args)
    requires_block_armed = args.requires_block_armed == "true"
    states = [ShardState(path) for path in args.logs]
    ready_since: float | None = None
    last_status_log = 0.0
    started = time.time()
    deadline = started + (args.timeout_ms / 1000) if args.timeout_ms > 0 else None

    try:
        args.gate_file.unlink()
    except FileNotFoundError:
        pass

    log(
        "swarm_global_action_gate_start "
        f"count={args.count} shards={len(states)} required={required} mode={args.action_mode} "
        f"settleMs={args.settle_ms} requiresBlockArmed={requires_block_armed}"
    )

    while True:
        now = time.time()
        if deadline is not None and now >= deadline:
            log("swarm_global_action_gate_timeout")
            return 1

        for state in states:
            poll_shard(state)

        totals, reason = current_totals(
            states,
            now=now,
            max_stale_seconds=args.metric_stale_ms / 1000,
        )
        if not reason:
            reason = unmet_reason(totals, required=required, requires_block_armed=requires_block_armed)

        if reason:
            if ready_since is not None:
                log(f"swarm_global_action_gate_reset reason={reason} readySinceMs={int((now - ready_since) * 1000)}")
                ready_since = None
            if now - last_status_log >= 5:
                if totals:
                    log(
                        "swarm_global_action_gate_wait "
                        f"reason={reason} created={totals['created']} connected={totals['connected']} "
                        f"ready={totals['ready']} active={totals['active']} settled={totals['settled']} "
                        f"blockArmed={totals['blockArmed']} required={required}"
                    )
                else:
                    log(f"swarm_global_action_gate_wait reason={reason} required={required}")
                last_status_log = now
            time.sleep(args.poll_ms / 1000)
            continue

        if ready_since is None:
            ready_since = now
            log(
                "swarm_global_action_gate_ready "
                f"created={totals['created']} connected={totals['connected']} ready={totals['ready']} "
                f"active={totals['active']} settled={totals['settled']} blockArmed={totals['blockArmed']} "
                f"required={required} settleMs={args.settle_ms}"
            )

        if (now - ready_since) * 1000 >= args.settle_ms:
            payload = {
                "open": True,
                "openedAtUnixMs": int(now * 1000),
                "readySinceUnixMs": int(ready_since * 1000),
                "count": args.count,
                "required": required,
                "shards": len(states),
                "settleMs": args.settle_ms,
                "requiresBlockArmed": requires_block_armed,
                "counts": totals,
            }
            write_gate_file(args.gate_file, payload)
            log(
                "swarm_global_action_gate_open "
                f"created={totals['created']} connected={totals['connected']} ready={totals['ready']} "
                f"active={totals['active']} settled={totals['settled']} blockArmed={totals['blockArmed']} "
                f"required={required} settleMs={args.settle_ms}"
            )
            return 0

        time.sleep(args.poll_ms / 1000)


if __name__ == "__main__":
    raise SystemExit(main())
