#!/usr/bin/env python3
"""
TUI-scoped parallel execution plan: intersect exarp-go task_analysis parallelization
groups with a frozen TUI task ID list, then order remaining (orphan) TUI IDs by
global execution_plan order.

Regenerate TUI ID list (Todo, keyword/tag heuristic):
  cd repo_root && ./scripts/run_exarp_go.sh task list --status Todo --json --quiet | \\
    python3 -c \"...\"  # see scripts/tui_backlog_ids.txt header

Usage:
  python3 scripts/tui_parallel_execution_plan.py [--ids-file PATH] [--out PATH]
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
from datetime import datetime, timezone

# Reuse exarp invocation + JSON parse from parallel_wave_remaining
_SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
_REPO = os.path.dirname(_SCRIPT_DIR)
if _SCRIPT_DIR not in sys.path:
    sys.path.insert(0, _SCRIPT_DIR)

from parallel_wave_remaining import parse_exarp_tool_stdout, resolve_exarp_invocation  # noqa: E402


def load_tui_ids(path: str) -> set[str]:
    ids: set[str] = set()
    with open(path, encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            if line.startswith("T-"):
                ids.add(line)
    return ids


def run_tool(root: str, tool: str, args: dict) -> dict | None:
    inv = resolve_exarp_invocation(root)
    if not inv:
        return None
    argv0, env = inv
    cmd = argv0 + [
        "-tool",
        tool,
        "-args",
        json.dumps(args),
    ]
    out = subprocess.run(
        cmd,
        cwd=root,
        env=env,
        capture_output=True,
        text=True,
        timeout=120,
    )
    if out.returncode != 0:
        sys.stderr.write(out.stderr or "")
        sys.stderr.write(out.stdout or "")
        return None
    return parse_exarp_tool_stdout(out.stdout)


def parallel_groups(data: dict | None) -> list[dict]:
    if not isinstance(data, dict):
        return []
    groups = data.get("parallel_groups")
    if not isinstance(groups, list):
        return []
    return [g for g in groups if isinstance(g, dict)]


def main() -> int:
    root = _REPO
    ids_path = os.path.join(root, "scripts", "tui_backlog_ids.txt")
    out_path = os.path.join(root, "out", "TUI_PARALLEL_EXECUTION_PLAN.md")

    if "--help" in sys.argv or "-h" in sys.argv:
        print(__doc__)
        return 0
    args = sys.argv[1:]
    if "--ids-file" in args:
        i = args.index("--ids-file")
        ids_path = os.path.normpath(os.path.join(root, args[i + 1]))
        del args[i : i + 2]
    if "--out" in args:
        i = args.index("--out")
        out_path = os.path.normpath(os.path.join(root, args[i + 1]))
        del args[i : i + 2]

    tui_ids = load_tui_ids(ids_path)
    if not tui_ids:
        print(f"No TUI IDs loaded from {ids_path}", file=sys.stderr)
        return 1

    par = run_tool(
        root,
        "task_analysis",
        {"action": "parallelization", "output_format": "json"},
    )
    if par is None:
        print("task_analysis parallelization failed", file=sys.stderr)
        return 1

    groups = parallel_groups(par)
    tui_batches: list[dict] = []
    tui_with_parallel_partner: set[str] = set()

    for g in groups:
        tasks = g.get("tasks")
        if not isinstance(tasks, list):
            continue
        tid_list = [str(t).strip() for t in tasks if t]
        batch = [x for x in tid_list if x in tui_ids]
        if len(batch) >= 2:
            for x in batch:
                tui_with_parallel_partner.add(x)
            tui_batches.append(
                {
                    "priority": g.get("priority", ""),
                    "reason": g.get("reason", ""),
                    "tasks": batch,
                }
            )

    orphans = sorted(tui_ids - tui_with_parallel_partner)

    plan = run_tool(
        root,
        "task_analysis",
        {"action": "execution_plan", "output_format": "json"},
    )
    ordered: list[str] = []
    if isinstance(plan, dict):
        raw_order = plan.get("ordered_task_ids")
        if isinstance(raw_order, list):
            ordered = [str(x).strip() for x in raw_order if x]

    orphan_set = set(orphans)
    orphans_ordered = [x for x in ordered if x in orphan_set]
    tail = [x for x in orphans if x not in set(orphans_ordered)]
    orphans_ordered.extend(sorted(tail))

    os.makedirs(os.path.dirname(out_path), exist_ok=True)
    now = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M UTC")

    lines = [
        "# TUI parallel execution plan",
        "",
        f"Generated: {now}",
        "",
        "Source TUI ID list: `scripts/tui_backlog_ids.txt`",
        "",
        "Parallel groups are **global** Todo dependency levels from exarp `task_analysis` "
        "`parallelization` (Todo-only), intersected with TUI IDs. "
        "See [docs/EXARP_TODO2_BACKLOG.md](../docs/EXARP_TODO2_BACKLOG.md).",
        "",
        "## Parallel batches (run together)",
        "",
    ]

    if not tui_batches:
        lines.append("_No batch has two or more TUI tasks at the same dependency level._")
        lines.append("")
    else:
        for i, b in enumerate(tui_batches, 1):
            lines.append(f"### Batch {i} ({b.get('priority', '')} priority)")
            lines.append("")
            lines.append(f"_{b.get('reason', '')}_")
            lines.append("")
            for tid in b["tasks"]:
                lines.append(f"- {tid}")
            lines.append("")

    lines.extend(
        [
            "## Orphans (no other TUI task in the same global parallel group)",
            "",
            "Ordered by global `execution_plan` where possible, then alphabetically.",
            "",
        ]
    )
    for tid in orphans_ordered:
        lines.append(f"- {tid}")
    lines.append("")

    lines.extend(
        [
            "## Regenerate",
            "",
            "```bash",
            "python3 scripts/tui_parallel_execution_plan.py",
            "```",
            "",
        ]
    )

    with open(out_path, "w", encoding="utf-8") as f:
        f.write("\n".join(lines))

    print(out_path)
    return 0


if __name__ == "__main__":
    sys.exit(main())
