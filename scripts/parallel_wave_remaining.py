#!/usr/bin/env python3
"""
Print remaining (non-Done) task IDs for a wave and the next batch of 10-15.
Reads .cursor/plans/parallel-execution-waves.json and uses exarp-go task_workflow
to get Todo/In Progress tasks; intersects with wave IDs.
Usage:
  python3 scripts/parallel_wave_remaining.py <wave_index> [batch_size]
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


def get_non_done_ids(root: str) -> set[str]:
    """Call exarp-go task_workflow list for Todo and In Progress; return union of task IDs."""
    inv = resolve_exarp_invocation(root)
    if not inv:
        return set()
    argv0, env = inv
    ids: set[str] = set()
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
            if isinstance(parsed, list):
                for t in parsed:
                    if isinstance(t, dict) and "id" in t:
                        ids.add(str(t["id"]).strip())
            elif isinstance(parsed, dict):
                tasks = parsed.get("tasks", parsed.get("items", []))
                for t in tasks if isinstance(tasks, list) else []:
                    if isinstance(t, dict) and "id" in t:
                        ids.add(str(t["id"]).strip())
                # Fallback: any key that looks like T-*
                for v in parsed.values():
                    if isinstance(v, str) and re.match(r"^T-\d+$", v.strip()):
                        ids.add(v.strip())
        except (json.JSONDecodeError, subprocess.TimeoutExpired, FileNotFoundError):
            continue
    # Also scrape T-* from stdout if JSON didn't work
    if not ids:
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
                    ids.add(m.group(0))
        except (subprocess.TimeoutExpired, FileNotFoundError):
            pass
    return ids


def main() -> int:
    root = repo_root()
    if len(sys.argv) < 2:
        print("Usage: parallel_wave_remaining.py <wave_index> [batch_size]", file=sys.stderr)
        print("  wave_index: 0, 1, or 2", file=sys.stderr)
        print("  batch_size: default 15", file=sys.stderr)
        return 1
    try:
        wave_index = int(sys.argv[1])
    except ValueError:
        print("wave_index must be 0, 1, or 2", file=sys.stderr)
        return 1
    if wave_index not in (0, 1, 2):
        print("wave_index must be 0, 1, or 2", file=sys.stderr)
        return 1
    batch_size = 15
    if len(sys.argv) > 2:
        try:
            batch_size = int(sys.argv[2])
        except ValueError:
            pass
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
    non_done = get_non_done_ids(root)
    remaining = [x for x in wave_list if x in non_done]
    if not non_done and wave_ids:
        print("Could not get Todo/In Progress from exarp-go. Wave IDs for manual intersect:", file=sys.stderr)
        print(" ".join(waves[wave_key]), file=sys.stderr)
        print("Run: exarp-go task list --status Todo (or use MCP task_workflow list) and intersect.", file=sys.stderr)
    print(f"# Wave {wave_index} remaining: {len(remaining)}")
    for tid in remaining:
        print(tid)
    batch = remaining[:batch_size]
    print(f"\n# Next batch (first {len(batch)}):")
    for tid in batch:
        print(tid)
    return 0


if __name__ == "__main__":
    sys.exit(main())
