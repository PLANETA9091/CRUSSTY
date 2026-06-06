#!/usr/bin/env python3
"""Summarize jstack thread samples from run_load_test diagnostics."""

from __future__ import annotations

import argparse
import collections
import json
import pathlib
import re
from dataclasses import dataclass, field


THREAD_RE = re.compile(r'^"([^"]+)".*')
STATE_RE = re.compile(r"^\s+java\.lang\.Thread\.State:\s+([A-Z_]+)")
FRAME_RE = re.compile(r"^\s+at\s+(.+)$")


@dataclass
class ThreadStack:
    name: str
    state: str | None = None
    frames: list[str] = field(default_factory=list)


def read_sample(path: pathlib.Path) -> list[ThreadStack]:
    stacks: list[ThreadStack] = []
    current: ThreadStack | None = None

    def flush() -> None:
        nonlocal current
        if current is not None and (current.state or current.frames):
            stacks.append(current)
        current = None

    with path.open(encoding="utf-8", errors="replace") as handle:
        for raw in handle:
            thread_match = THREAD_RE.match(raw)
            if thread_match:
                flush()
                current = ThreadStack(name=thread_match.group(1))
                continue
            if current is None:
                continue
            state_match = STATE_RE.match(raw)
            if state_match:
                current.state = state_match.group(1)
                continue
            frame_match = FRAME_RE.match(raw)
            if frame_match:
                current.frames.append(frame_match.group(1))
    flush()
    return stacks


def interesting_top_frame(frames: list[str]) -> str | None:
    for frame in frames:
        if frame.startswith(("java.", "jdk.", "sun.", "com.sun.")):
            continue
        return frame
    return None


def summarize(sample_dir: pathlib.Path, pattern: str) -> dict[str, object]:
    files = sorted(sample_dir.glob(pattern))
    state_counts: collections.Counter[str] = collections.Counter()
    thread_counts: collections.Counter[str] = collections.Counter()
    top_frame_counts: collections.Counter[str] = collections.Counter()
    all_frame_counts: collections.Counter[str] = collections.Counter()
    stack_count = 0

    for path in files:
        for stack in read_sample(path):
            stack_count += 1
            state_counts[stack.state or "UNKNOWN"] += 1
            thread_counts[stack.name] += 1
            top_frame = interesting_top_frame(stack.frames)
            if top_frame:
                top_frame_counts[top_frame] += 1
            all_frame_counts.update(stack.frames)

    return {
        "sample_dir": str(sample_dir),
        "file_pattern": pattern,
        "sample_count": len(files),
        "thread_stack_count": stack_count,
        "state_counts": dict(state_counts.most_common()),
        "thread_counts": dict(thread_counts.most_common()),
        "top_frame_counts": dict(top_frame_counts.most_common()),
        "all_frame_counts": dict(all_frame_counts.most_common()),
    }


def text_report(summary: dict[str, object], top: int) -> str:
    lines = [
        "thread_sample_summary=true",
        f"sample_dir={summary['sample_dir']}",
        f"file_pattern={summary['file_pattern']}",
        f"sample_count={summary['sample_count']}",
        f"thread_stack_count={summary['thread_stack_count']}",
        "",
        "[states]",
    ]
    for key, value in list(summary["state_counts"].items())[:top]:  # type: ignore[union-attr]
        lines.append(f"{value}\t{key}")

    lines.append("")
    lines.append("[top_frames]")
    for key, value in list(summary["top_frame_counts"].items())[:top]:  # type: ignore[union-attr]
        lines.append(f"{value}\t{key}")

    lines.append("")
    lines.append("[all_frames]")
    for key, value in list(summary["all_frame_counts"].items())[:top]:  # type: ignore[union-attr]
        lines.append(f"{value}\t{key}")

    lines.append("")
    lines.append("[threads]")
    for key, value in list(summary["thread_counts"].items())[:top]:  # type: ignore[union-attr]
        lines.append(f"{value}\t{key}")

    return "\n".join(lines) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("sample_dir", type=pathlib.Path)
    parser.add_argument("--top", type=int, default=30)
    parser.add_argument("--pattern", default="thread-sample-*.txt")
    parser.add_argument("--report", type=pathlib.Path)
    parser.add_argument("--json-report", type=pathlib.Path)
    args = parser.parse_args()

    if not args.sample_dir.is_dir():
        raise SystemExit(f"Missing sample directory: {args.sample_dir}")
    if args.top < 1:
        raise SystemExit("--top must be >= 1")

    summary = summarize(args.sample_dir, args.pattern)
    report = text_report(summary, args.top)
    if args.report:
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(report, encoding="utf-8")
    if args.json_report:
        args.json_report.parent.mkdir(parents=True, exist_ok=True)
        args.json_report.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(report, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
