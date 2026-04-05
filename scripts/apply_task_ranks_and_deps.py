#!/usr/bin/env python3
"""Assign priority_rank within high/medium/low from parallel-execution-waves order;
merge blocking dependencies for known chains.

Uses SQLite for priority_rank because the installed exarp-go task_workflow update
guard does not treat priority_rank as a standalone field (priority-only updates
succeed; rank must be written to DB). Then runs `task sync`. Dependencies are
applied via exarp task_workflow (merge with existing)."""
from __future__ import annotations

import json
import os
import sqlite3
import subprocess
import sys
from collections import defaultdict


def repo_root() -> str:
    return os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def load_ordered_ids(root: str) -> list[str]:
    path = os.path.join(root, ".cursor", "plans", "parallel-execution-waves.json")
    with open(path, encoding="utf-8") as f:
        data = json.load(f)
    return list(data.get("ordered_task_ids", []))


def exarp_task_workflow(args: dict) -> dict:
    root = repo_root()
    script = os.path.join(root, "scripts", "run_exarp_go.sh")
    proc = subprocess.run(
        [script, "-tool", "task_workflow", "-args", json.dumps(args)],
        capture_output=True,
        text=True,
        env={**os.environ, "PROJECT_ROOT": root},
    )
    raw = proc.stdout + proc.stderr
    if "Result:" in raw:
        raw = raw.split("Result:", 1)[1].strip()
    start = raw.find("{")
    if start < 0:
        print("exarp parse fail:", proc.stdout[:500], proc.stderr[:500], file=sys.stderr)
        sys.exit(1)
    decoder = json.JSONDecoder()
    obj, _ = decoder.raw_decode(raw[start:])
    return obj


def list_todo() -> list[dict]:
    out = exarp_task_workflow(
        {"action": "list", "status": "Todo", "output_format": "json", "compact": True}
    )
    return out.get("tasks") or []


def show_task(task_id: str) -> dict | None:
    out = exarp_task_workflow(
        {"action": "show", "task_id": task_id, "output_format": "json", "compact": True}
    )
    tasks = out.get("tasks") or []
    return tasks[0] if tasks else None


# Child -> additional blocking dependency IDs (merged with existing).
DEPENDENCY_MERGE: dict[str, list[str]] = {
    "T-1774817606398986000": ["T-1774817606395405000"],
    "T-1774817606420931000": ["T-1774817606420467000"],
    "T-1774817606442288000": ["T-1774817606441825000"],
    "T-1774817606463912000": ["T-1774817606463474000"],
    "T-1774817606507550000": ["T-1774817606507094000"],
    "T-1774817606572191000": ["T-1774817606571913000"],
    "T-1774817606572367000": ["T-1774817606572191000"],
    "T-1774965243379093000": ["T-1774862964668253000"],
    "T-1775403390235834000": ["T-1774817606571913000"],
}


def apply_sqlite_ranks(updates: list[tuple[str, int]]) -> None:
    root = repo_root()
    db = os.path.join(root, ".todo2", "todo2.db")
    if not os.path.isfile(db):
        print("missing", db, file=sys.stderr)
        sys.exit(1)
    con = sqlite3.connect(db)
    try:
        con.execute("BEGIN IMMEDIATE")
        for tid, rank in updates:
            con.execute(
                "UPDATE tasks SET priority_rank = ?, updated_at = strftime('%s', 'now') WHERE id = ?",
                (rank, tid),
            )
        con.commit()
    finally:
        con.close()


def run_sync() -> None:
    root = repo_root()
    script = os.path.join(root, "scripts", "run_exarp_go.sh")
    r = subprocess.run([script, "task", "sync"], cwd=root, env={**os.environ, "PROJECT_ROOT": root})
    if r.returncode != 0:
        sys.exit(r.returncode)


def main() -> None:
    root = repo_root()
    ordered = load_ordered_ids(root)
    pos = {tid: i for i, tid in enumerate(ordered)}
    todos = list_todo()
    pri_order = {"high": 0, "medium": 1, "low": 2}

    def sort_key(t: dict) -> tuple[int, int, str]:
        p = (t.get("priority") or "medium").lower()
        g = pri_order.get(p, 1)
        i = pos.get(t["id"], 10_000 + len(ordered))
        return (g, i, t["id"])

    by_pri: dict[str, list[dict]] = defaultdict(list)
    for t in sorted(todos, key=sort_key):
        p = (t.get("priority") or "medium").lower()
        if p not in pri_order:
            p = "medium"
        by_pri[p].append(t)

    rank_updates: list[tuple[str, int]] = []
    for p in ("high", "medium", "low"):
        for rank, t in enumerate(by_pri.get(p, [])):
            rank_updates.append((t["id"], rank))

    dry = "--dry-run" in sys.argv
    if dry:
        for tid, rank in rank_updates:
            extra = DEPENDENCY_MERGE.get(tid)
            print(f"{tid} priority_rank={rank} deps_merge={extra}")
        print(f"(dry-run) {len(rank_updates)} ranks, {len(DEPENDENCY_MERGE)} dep keys")
        return

    apply_sqlite_ranks(rank_updates)

    for tid, extra_deps in DEPENDENCY_MERGE.items():
        if not any(t["id"] == tid for t in todos):
            continue
        row = show_task(tid)
        existing = list(row.get("dependencies") or []) if row else []
        merged = list(dict.fromkeys(existing + extra_deps))
        out = exarp_task_workflow(
            {
                "action": "update",
                "task_ids": tid,
                "dependencies": merged,
                "auto_estimate": False,
                "output_format": "json",
                "compact": True,
            }
        )
        if not out.get("success") and out.get("updated_count", 0) == 0:
            print("FAIL deps", tid, out, file=sys.stderr)
            sys.exit(1)
        print("ok deps", tid)

    run_sync()
    print("done: sqlite priority_rank for", len(rank_updates), "tasks; deps merged;", "task sync")


if __name__ == "__main__":
    main()
