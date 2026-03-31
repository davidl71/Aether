#!/usr/bin/env python3
"""
Print remaining (non-Done) task IDs for a wave and the next batch of 10-15.
Reads .cursor/plans/parallel-execution-waves.json and uses exarp-go task_workflow
to get Todo/In Progress tasks; intersects with wave IDs.
Usage:
  python3 scripts/parallel_wave_remaining.py <wave_index> [batch_size]
  python3 scripts/parallel_wave_remaining.py <wave_index> [batch_size] --tag tui
  python3 scripts/parallel_wave_remaining.py <wave_index> [batch_size] --tag cli --strict-tag
  wave_index: 0, 1, or 2 (matches parallel-execution-waves.json "waves" keys)
  batch_size: default 15
Example:
  python3 scripts/parallel_wave_remaining.py 0
  python3 scripts/parallel_wave_remaining.py 1 10
"""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
from argparse import ArgumentParser


def repo_root() -> str:
    script_dir = os.path.dirname(os.path.abspath(__file__))
    return os.path.dirname(script_dir)


def load_waves(root: str) -> dict[str, list[str]]:
    path = os.path.join(root, ".cursor", "plans", "parallel-execution-waves.json")
    with open(path, encoding="utf-8") as f:
        data = json.load(f)
    if isinstance(data, dict) and "waves" in data and isinstance(data["waves"], dict):
        return data["waves"]
    return data if isinstance(data, dict) else {}


def resolve_exarp_invocation(root: str) -> tuple[list[str], dict[str, str]] | None:
    """Prefer a direct exarp-go binary; run_exarp_go.sh can fail in some CI/sandbox setups."""
    env = os.environ.copy()
    env.setdefault("PROJECT_ROOT", root)
    candidates: list[str] = []
    if os.environ.get("EXARP_GO_BIN"):
        candidates.append(os.environ["EXARP_GO_BIN"])
    for rel in (
        os.path.join(root, "..", "..", "mcp", "exarp-go", "bin", "exarp-go"),
        os.path.join(root, "..", "mcp", "exarp-go", "bin", "exarp-go"),
    ):
        candidates.append(os.path.normpath(rel))
    for exe in candidates:
        if exe and os.path.isfile(exe) and os.access(exe, os.X_OK):
            return ([exe], env)
    script = os.path.join(root, "scripts", "run_exarp_go.sh")
    if os.path.isfile(script) and os.access(script, os.X_OK):
        return ([script], env)
    return None


def parse_exarp_tool_stdout(stdout: str) -> dict | list | None:
    """Strip CLI banner lines; exarp-go -tool prints 'Result:\\n{...}' not raw JSON."""
    raw = (stdout or "").strip()
    if not raw:
        return None
    if "Result:" in raw:
        raw = raw.split("Result:", 1)[1].lstrip("\n\r").strip()
    try:
        out = json.loads(raw)
    except json.JSONDecodeError:
        start = raw.find("{")
        if start < 0:
            start = raw.find("[")
        if start < 0:
            return None
        try:
            out = json.loads(raw[start:])
        except json.JSONDecodeError:
            return None
    return out


def get_non_done_tasks(root: str) -> dict[str, set[str]]:
    """Return map task_id -> normalized tag set for (Todo ∪ In Progress)."""
    inv = resolve_exarp_invocation(root)
    if not inv:
        return {}
    argv0, env = inv
    tasks_by_id: dict[str, set[str]] = {}
    for status in ("Todo", "In Progress"):
        try:
            cmd = argv0 + [
                "-tool",
                "task_workflow",
                "-args",
                json.dumps(
                    {
                        "action": "list",
                        "status": status,
                        "output_format": "json",
                        "compact": True,
                    }
                ),
            ]
            out = subprocess.run(
                cmd,
                cwd=root,
                env=env,
                capture_output=True,
                text=True,
                timeout=60,
            )
            if out.returncode != 0:
                continue
            parsed = parse_exarp_tool_stdout(out.stdout)
            if parsed is None:
                continue
            items = None
            if isinstance(parsed, list):
                items = parsed
            elif isinstance(parsed, dict):
                items = parsed.get("tasks", parsed.get("items", []))
                if not isinstance(items, list):
                    items = []

            for t in items:
                if not isinstance(t, dict):
                    continue
                tid = t.get("id")
                if not tid:
                    continue
                tid = str(tid).strip()
                raw_tags = t.get("tags") or []
                if isinstance(raw_tags, str):
                    tag_list = [x.strip() for x in raw_tags.split(",") if x.strip()]
                elif isinstance(raw_tags, list):
                    tag_list = [str(x).strip() for x in raw_tags if str(x).strip()]
                else:
                    tag_list = []
                tasks_by_id[tid] = {x.lower() for x in tag_list if x}
        except (json.JSONDecodeError, subprocess.TimeoutExpired, FileNotFoundError):
            continue

    # Last-resort: scrape T-* from stdout if JSON parsing didn't yield any IDs.
    if not tasks_by_id:
        try:
            cmd = argv0 + [
                "-tool",
                "task_workflow",
                "-args",
                json.dumps({"action": "list", "status": "Todo"}),
            ]
            out = subprocess.run(
                cmd,
                cwd=root,
                env=env,
                capture_output=True,
                text=True,
                timeout=60,
            )
            for line in (out.stdout or "").splitlines():
                for m in re.finditer(r"T-\d+", line):
                    tasks_by_id.setdefault(m.group(0), set())
        except (subprocess.TimeoutExpired, FileNotFoundError):
            pass

    return tasks_by_id


def parse_args(argv: list[str]) -> tuple[int, int, list[str], bool]:
    parser = ArgumentParser(add_help=True)
    parser.add_argument("wave_index", type=int, help="0, 1, or 2")
    parser.add_argument("batch_size", nargs="?", type=int, default=15)
    parser.add_argument(
        "--tag",
        dest="tags",
        action="append",
        default=[],
        help="Prefer batching tasks that include this tag (repeatable). Example: --tag tui",
    )
    parser.add_argument(
        "--strict-tag",
        action="store_true",
        help="If set, only include tasks that match --tag in the batch (may yield smaller batch).",
    )
    ns = parser.parse_args(argv)
    tags = [t.strip().lower() for t in (ns.tags or []) if t and t.strip()]
    return ns.wave_index, ns.batch_size, tags, bool(ns.strict_tag)


def main() -> int:
    root = repo_root()
    try:
        wave_index, batch_size, prefer_tags, strict_tag = parse_args(sys.argv[1:])
    except SystemExit:
        return 1
    if wave_index not in (0, 1, 2):
        print("wave_index must be 0, 1, or 2", file=sys.stderr)
        return 1
    wave_key = str(wave_index)
    waves = load_waves(root)
    if wave_key not in waves:
        print(
            f"Wave {wave_key} not in parallel-execution-waves.json (have: {sorted(waves.keys())})",
            file=sys.stderr,
        )
        return 1
    wave_list = waves[wave_key]
    wave_ids = set(wave_list)
    non_done_tasks = get_non_done_tasks(root)
    remaining = [x for x in wave_list if x in non_done_tasks]
    if not non_done_tasks and wave_ids:
        print("Could not get Todo/In Progress from exarp-go. Wave IDs for manual intersect:", file=sys.stderr)
        print(" ".join(waves[wave_key]), file=sys.stderr)
        print("Run: exarp-go task list --status Todo (or use MCP task_workflow list) and intersect.", file=sys.stderr)
    print(f"# Wave {wave_index} remaining: {len(remaining)}")
    for tid in remaining:
        print(tid)

    batch: list[str] = []
    if prefer_tags:
        tagged = [
            tid
            for tid in remaining
            if any(tag in non_done_tasks.get(tid, set()) for tag in prefer_tags)
        ]
        if strict_tag:
            batch = tagged[:batch_size]
        else:
            untagged = [tid for tid in remaining if tid not in set(tagged)]
            batch = (tagged + untagged)[:batch_size]
    else:
        batch = remaining[:batch_size]

    suffix = ""
    if prefer_tags:
        suffix = f" (prefer tags: {', '.join(prefer_tags)}{'; strict' if strict_tag else ''})"
    print(f"\n# Next batch (first {len(batch)}){suffix}:")
    for tid in batch:
        print(tid)
    return 0


if __name__ == "__main__":
    sys.exit(main())
