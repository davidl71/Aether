#!/usr/bin/env python3
"""Batch-create subtasks for complex open TUI work via exarp-go task_workflow (action=create)."""
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

COMMON_TAGS = "tui,frontend"


def run_tw(params: dict) -> dict:
    proc = subprocess.run(
        [
            str(ROOT / "scripts" / "run_exarp_go.sh"),
            "-tool",
            "task_workflow",
            "-args",
            json.dumps(params, separators=(",", ":")),
        ],
        cwd=str(ROOT),
        capture_output=True,
        text=True,
    )
    raw = proc.stdout + proc.stderr
    if proc.returncode != 0:
        print(raw, file=sys.stderr)
        raise SystemExit(proc.returncode)
    marker = "\nResult:\n"
    pos = raw.find(marker)
    if pos >= 0:
        raw = raw[pos + len(marker) :]
    idx = raw.find("{")
    if idx < 0:
        raise RuntimeError(f"No JSON in output:\n{raw}")
    return json.JSONDecoder().raw_decode(raw[idx:])[0]


def tag_children(items: list[dict]) -> list[dict]:
    out = []
    for d in items:
        c = dict(d)
        if "tags" not in c:
            c["tags"] = COMMON_TAGS
        out.append(c)
    return out


def create_under(parent_id: str, children: list[dict]) -> dict:
    return run_tw(
        {
            "action": "create",
            "parent_id": parent_id,
            "auto_estimate": False,
            "tasks": json.dumps(tag_children(children)),
        }
    )


def main() -> None:
    results: list[tuple[str, dict]] = []

    results.append(
        (
            "T-1774459943054837000",
            create_under(
                "T-1774459943054837000",
                [
                    {
                        "name": "NATS health: extend snapshot/API with transport metrics fields",
                        "priority": "high",
                        "long_description": "Define and plumb metrics (connected, reconnect count, last error, latency if available) into existing snapshot or health payload consumed by TUI.",
                    },
                    {
                        "name": "NATS health: populate metrics from nats_adapter / collector",
                        "priority": "high",
                        "long_description": "Instrument the live NATS client path to fill the new fields; keep read-only and safe when NATS disabled.",
                    },
                    {
                        "name": "NATS health: TUI surfaces (Settings and/or status)",
                        "priority": "high",
                        "long_description": "Render transport health clearly without noise; align with existing health tab patterns.",
                    },
                    {
                        "name": "NATS health: verification checklist (manual + docs touchpoint)",
                        "priority": "medium",
                        "long_description": "Short checklist: backend off, NATS down, happy path; note in MARKET_DATA or ops doc if needed.",
                    },
                ],
            ),
        )
    )

    pane_dep = "T-1774459957300872000"
    results.append(
        (
            "T-1774463349681383000",
            create_under(
                "T-1774463349681383000",
                [
                    {
                        "name": "Pane model: inventory current pane/focus/split state",
                        "priority": "high",
                        "long_description": "Map where focus, splits, and workspace state live in tui_service today; note edge cases.",
                        "dependencies": pane_dep,
                    },
                    {
                        "name": "Pane model: document target model (types + transitions)",
                        "priority": "high",
                        "long_description": "One short design note: entities, invariants, how events change focus/pane.",
                        "dependencies": pane_dep,
                    },
                    {
                        "name": "Pane model: refactor shell/router toward central model",
                        "priority": "high",
                        "long_description": "Implement model + migrate routing incrementally; avoid big-bang.",
                        "dependencies": pane_dep,
                    },
                    {
                        "name": "Pane model: regression checklist (tabs, workspaces, resize)",
                        "priority": "medium",
                        "long_description": "Manual pass for narrow terminals and workspace switches.",
                        "dependencies": pane_dep,
                    },
                ],
            ),
        )
    )

    dedupe_dep = "T-1774463349681383000"
    results.append(
        (
            "T-1774377178601506000",
            create_under(
                "T-1774377178601506000",
                [
                    {
                        "name": "TUI helpers: audit duplicate patterns post workspace/settings split",
                        "priority": "medium",
                        "long_description": "List files/helpers that overlap; group by theme (layout, formatting, keys).",
                        "dependencies": dedupe_dep,
                    },
                    {
                        "name": "TUI helpers: extract shared module for highest-duplication cluster",
                        "priority": "medium",
                        "long_description": "Pick one cluster; create shared API; keep behavior identical.",
                        "dependencies": dedupe_dep,
                    },
                    {
                        "name": "TUI helpers: remove call-site duplicates + fmt/clippy",
                        "priority": "medium",
                        "long_description": "Migrate call sites; run cargo fmt/clippy on touched crates.",
                        "dependencies": dedupe_dep,
                    },
                ],
            ),
        )
    )

    results.append(
        (
            "T-1774285312494816000",
            create_under(
                "T-1774285312494816000",
                [
                    {
                        "name": "DirtyFlags: baseline render/tick cost notes",
                        "priority": "medium",
                        "long_description": "Quick profile or log-based estimate of redraw frequency/cost before changes.",
                    },
                    {
                        "name": "DirtyFlags: implement granular dirty tracking in render path",
                        "priority": "medium",
                        "long_description": "Track which regions/widgets need draw; skip full frame when clean.",
                    },
                    {
                        "name": "DirtyFlags: validate behavior (manual + reduced redraw sanity)",
                        "priority": "medium",
                        "long_description": "Confirm UI correctness and observable fewer full redraws.",
                    },
                ],
            ),
        )
    )

    results.append(
        (
            "T-1774285315059562000",
            create_under(
                "T-1774285315059562000",
                [
                    {
                        "name": "ScrollableTableState: API + ownership design",
                        "priority": "low",
                        "long_description": "Define state struct, who owns scroll offset, interaction with widgets.",
                    },
                    {
                        "name": "ScrollableTableState: implement + pilot one table",
                        "priority": "low",
                        "long_description": "Integrate with one real table; fix rough edges.",
                    },
                    {
                        "name": "ScrollableTableState: migrate remaining tables",
                        "priority": "low",
                        "long_description": "Replace ad-hoc scrolling; delete dead code paths.",
                    },
                ],
            ),
        )
    )

    results.append(
        (
            "T-1774285317440018000",
            create_under(
                "T-1774285317440018000",
                [
                    {
                        "name": "Toasts: queue, policy, and concurrency",
                        "priority": "low",
                        "long_description": "Max visible, dedupe, thread/async boundaries vs UI thread.",
                    },
                    {
                        "name": "Toasts: rendering + timeout behavior",
                        "priority": "low",
                        "long_description": "Styles, duration, dismissal, stacking.",
                    },
                    {
                        "name": "Toasts: wire pilot events from backend/UI",
                        "priority": "low",
                        "long_description": "Pick 1–2 user-visible failures/successes to surface first.",
                    },
                ],
            ),
        )
    )

    results.append(
        (
            "T-1774285319951400000",
            create_under(
                "T-1774285319951400000",
                [
                    {
                        "name": "Numeric columns: measurement helper API",
                        "priority": "low",
                        "long_description": "Width from content sample or fixed rules; test with wide values.",
                    },
                    {
                        "name": "Numeric columns: apply to primary positions/market tables",
                        "priority": "low",
                        "long_description": "Right-align numeric columns; verify headers.",
                    },
                    {
                        "name": "Numeric columns: sweep remaining tables",
                        "priority": "low",
                        "long_description": "Consistency pass; document convention in code comment if needed.",
                    },
                ],
            ),
        )
    )

    results.append(
        (
            "T-1774352915703004000",
            create_under(
                "T-1774352915703004000",
                [
                    {
                        "name": "Strategy NATS: config for strategy.signal / strategy.decision subjects",
                        "priority": "low",
                        "long_description": "Align with config.example.json and read-only exploration defaults.",
                    },
                    {
                        "name": "Strategy NATS: subscription lifecycle in tui_service",
                        "priority": "low",
                        "long_description": "Connect/disconnect, backoff, error surfacing without spam.",
                    },
                    {
                        "name": "Strategy NATS: minimal UI surfacing (log or status snippet)",
                        "priority": "low",
                        "long_description": "Operator-visible last message or count; keep low-noise.",
                    },
                ],
            ),
        )
    )

    eval_children = [
        {
            "name": "Research phase 1: docs/README survey",
            "priority": "low",
            "long_description": "Skim upstream docs, examples, MSRV, maintenance.",
        },
        {
            "name": "Research phase 2: minimal spike",
            "priority": "low",
            "long_description": "Optional scratch branch or isolated module; time-box.",
        },
        {
            "name": "Research phase 3: decision note (adopt / hybrid / pass)",
            "priority": "low",
            "long_description": "Short rationale + follow-up tasks if adopt.",
        },
    ]

    for tid in (
        "T-1774478911044499000",
        "T-1774479836566898000",
        "T-1774479844352731000",
    ):
        results.append((tid, create_under(tid, eval_children)))

    results.append(
        (
            "T-1774807919579082000",
            create_under(
                "T-1774807919579082000",
                [
                    {
                        "name": "Settings short terminal: reproduce layouts (height breakpoints)",
                        "priority": "medium",
                        "long_description": "Capture minimum viable heights where layout breaks.",
                    },
                    {
                        "name": "Settings short terminal: fix layout/truncation issues",
                        "priority": "medium",
                        "long_description": "Implement fixes; prefer ratatui-idiomatic constraints.",
                    },
                    {
                        "name": "Settings short terminal: QA checklist",
                        "priority": "medium",
                        "long_description": "Quick manual matrix: widths x heights.",
                    },
                ],
            ),
        )
    )

    epic_ld = """Epic: TUI UX foundation program (discoverability, modes, terminal feedback).

Scope: This task stays the umbrella. Executable work is tracked in:
- T-1774469201593212000 terminal feedback primitives
- T-1774469215296702000 mode system
- T-1774469228746075000 discoverability layer
- Child tasks under this epic: pane model (T-1774463349681383000), helper dedupe (T-1774377178601506000)

Done when: the three pillar tasks above are Done, pane model + dedupe epics are Done, and integration task T-1774469242259152000 can proceed per its own dependencies."""

    results.append(
        (
            "T-1774354599853391000 update",
            run_tw(
                {
                    "action": "update",
                    "task_id": "T-1774354599853391000",
                    "long_description": epic_ld,
                }
            ),
        )
    )

    results.append(
        (
            "T-1774354599853391000 note",
            run_tw(
                {
                    "action": "add_comment",
                    "task_id": "T-1774354599853391000",
                    "comment_type": "note",
                    "content": "2026-03-29: Clarified epic scope in long_description; split complex open TUI tasks into child tasks (batch create via scripts/tui_complex_split_batch.py).",
                }
            ),
        )
    )

    print(json.dumps({"results": results}, indent=2))


if __name__ == "__main__":
    main()
