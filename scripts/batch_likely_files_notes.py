#!/usr/bin/env python3
"""One-shot: add 'Likely files' note comments to Aether Todo2 tasks via exarp-go."""
from __future__ import annotations

import json
import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
EXARP_SH = os.path.join(ROOT, "scripts", "run_exarp_go.sh")

# Task ID -> markdown note body (no duplicate header)
NOTES: dict[str, str] = {
    # exarp-go SQLite / task_workflow (work lives in exarp-go repo)
    "T-1774888990930703000": "`exarp-go/internal/database/` (lock cleanup), `tasks_lock.go`",
    "T-1774888990962856000": "`exarp-go/internal/database/` (GetTask, FindNextClaimableTask, tags/deps queries)",
    "T-1774888990980406000": "`exarp-go/internal/database/` (AddComments batch insert)",
    "T-1774888990994244000": "`exarp-go/internal/database/` (UpdateTask, version), `internal/models/`",
    "T-1774888048357799000": "`exarp-go/Makefile`, `exarp-go/internal/database/*_test.go`, benches",
    "T-1774888048738176000": "`exarp-go/internal/database/` (BatchUpdateTaskStatus), `internal/tools/task_workflow*.go`",
    # Disk / ops
    "T-1774888010652362000": "`scripts/disk_pressure.sh`, operator runbook / `docs/`",
    "T-1774864299167793000": "`scripts/`, `Justfile`, `agents/backend/target` hygiene",
    "T-1774864299173551000": "`.cursor/commands.json`, `scripts/`, hooks if present",
    # Yield / market data
    "T-1774888010677335000": "`crates/market_data/`, `crates/api/`, tests under `agents/backend/`",
    "T-1774864280333760000": "`crates/market_data/` (Yahoo), `crates/tws_yield_curve/`, curve wiring",
    "T-1774864273104608000": "`crates/market_data/` (Polygon), yield curve inputs",
    "T-1774864273105669000": "`crates/market_data/`, `crates/api/src/finance_rates/`, comparison tests",
    "T-1774807919801005000": "`services/tws_yield_curve_daemon/`, `crates/api/src/finance_rates/`, TUI Yield tab",
    "T-1774465101831336000": "`services/tws_yield_curve_daemon/`, `crates/api/src/finance_rates/`",
    "T-1774285493600690000": "`crates/market_data/`, `crates/tws_yield_curve/`, yield tests",
    # NATS health
    "T-1774871452208254000": "`crates/nats_adapter/`, `services/backend_service/`, `crates/api/`, `tui_service`",
    "T-1774817606329653000": "`crates/api/` snapshot/health types, `services/backend_service/`",
    "T-1774817606330711000": "`crates/nats_adapter/`, collector paths in `backend_service`",
    "T-1774817606330858000": "`agents/backend/services/tui_service/src/` (Settings/health UI)",
    "T-1774817606331000000": "`docs/` (MARKET_DATA, NATS), manual QA checklist",
    "T-1774807919390383000": "`tui_service` + `api` snapshot fields for `nats_transport`",
    "T-1774814366283505000": "`docs/` health/NATS/snapshot wording",
    # Charts / risk
    "T-1774865358723034000": "`tui_service` charts, `crates/api/` risk_metrics, `docs/research/2026-03-30-research-wave/`",
    # TUI workspaces / layout / scroll (Aether tui_service)
    "T-1774864489431214000": "`tui_service/src/` positions views, `input_*.rs`, snapshot sort fields",
    "T-1774864489449178000": "`tui_service/src/ui/`, theme/palette modules, `app.rs`",
    "T-1774864489449128000": "`tui_service` table state + filter UI for pilot table",
    "T-1774864479161772000": "`tui_service` operations workspace panes, scroll state in `app.rs` / `ui/`",
    "T-1774864479163310000": "`tui_service/src/ui/`, `input_views.rs`, operations workspace layout",
    "T-1774864479163474000": "`tui_service` settings layout when embedded in operations",
    "T-1774864461100987000": "`tui_service` market workspace scroll state",
    "T-1774864455251981000": "`tui_service` market workspace banners, `ui/` polish",
    "T-1774864455252101000": "`tui_service/src/ui/`, market workspace split layout",
    # tui-input
    "T-1774864442697238000": "`tui_service/Cargo.toml`, `input*.rs`, ADR/decision doc if any",
    "T-1774864442696978000": "`tui_service` orders vs loans flows",
    "T-1774864442697310000": "`tui_service` pilot surface (orders filter or loans)",
    "T-1774807919643036000": "`tui_service` orders table + input buffer",
    # Feedback / toasts / in-flight
    "T-1774864312642150000": "`tui_service` command dispatch + status line",
    "T-1774864312642195000": "`tui_service` toast queue, `app.rs` tick/render",
    "T-1774864312642267000": "`tui_service` loans import path, HTTP client callbacks",
    "T-1774807919162623000": "`tui_service` error paths, NATS/snapshot failures",
    "T-1774807919195167000": "`tui_service` pending refresh / FMP / slow API surfaces",
    "T-1774817606441825000": "`tui_service` toast policy module",
    "T-1774817606442128000": "`tui_service` overlay render + timers",
    "T-1774817606442288000": "`tui_service` wire events from `api` client / backend responses",
    # ratatui-interact
    "T-1774862718741621000": "`tui_service`, upstream ratatui-interact docs",
    "T-1774862487010348000": "`tui_service/src/input_tabs.rs`, `input_views.rs`, region hit-testing",
    "T-1774862487049808000": "`tui_service` focus routing prototype",
    "T-1774862487064464000": "`tui_service/Cargo.toml`, isolated module boundary",
    # Pane model + helpers + perf (T-177481760*)
    "T-1774817606352358000": "`tui_service` inventory: `app.rs`, `input*.rs`, `ui/mod.rs`",
    "T-1774817606352659000": "`docs/` or `thoughts/` short design note + `tui_service`",
    "T-1774817606352847000": "`tui_service` central router module, incremental migration",
    "T-1774817606353005000": "`tui_service` manual QA; narrow terminal matrix",
    "T-1774817606373911000": "`tui_service/src/ui/`, duplicated layout/formatting helpers",
    "T-1774817606374238000": "`tui_service` new shared helper module",
    "T-1774817606374444000": "`tui_service` call-site migration",
    "T-1774817606395405000": "`tui_service` render/tick logging or profiling notes",
    "T-1774817606398792000": "`tui_service` `app.rs` draw path, dirty flags",
    "T-1774817606398986000": "`tui_service` validation + tests",
    "T-1774817606420467000": "`tui_service` table scroll types",
    "T-1774817606420764000": "`tui_service` one pilot table",
    "T-1774817606420931000": "`tui_service` remaining tables",
    "T-1774817606463474000": "`tui_service` column width helpers",
    "T-1774817606463744000": "`tui_service` positions/market tables",
    "T-1774817606463912000": "`tui_service` table sweep",
    "T-1774817606485091000": "`config/`, `nats_adapter::topics`, `strategy` crate",
    "T-1774817606485379000": "`tui_service` NATS client lifecycle",
    "T-1774817606485557000": "`tui_service` status/log snippet for strategy subjects",
    "T-1774817606507094000": "upstream docs (tickrs/tui-input/etc.)",
    "T-1774817606507387000": "scratch branch / `tui_service` spike",
    "T-1774817606507550000": "`docs/` decision note",
    "T-1774817606571913000": "`tui_service` settings layout breakpoints",
    "T-1774817606572191000": "`tui_service` settings responsive layout",
    "T-1774817606572367000": "`tui_service` QA checklist",
    # Deferred loans CSV/PDF
    "T-1774864120067507000": "`tui_service` loans + PDF parser crate choice, `crates/api` if ingest API",
    "T-1774864120067504000": "`tui_service` loans + CSV mapping, backend route if any",
    # CSV CLI
    "T-1774863006094962000": "`bin/cli`, `crates/discount_bank_parser/` or ingest path, `api`",
    "T-1774862964668253000": "`agents/backend/bin/cli`, snapshot export, `crates/api`",
    "T-1774862964678985000": "`bin/cli`, `tui_service` loans pipeline",
    # Discoverability / modes / epics
    "T-1774807919228082000": "`tui_service/src/input.rs`, `input_tabs.rs`, `app.rs` (AppMode)",
    "T-1774807919259741000": "`tui_service` status/hint rendering",
    "T-1774807919291227000": "`tui_service` help overlay + keymaps",
    "T-1774807919323158000": "`tui_service` command palette, `Command::available_in`",
    "T-1774807919431098000": "`tui_service` pane focus API",
    "T-1774807919484377000": "`tui_service` tests for workspace focus",
    "T-1774807919516323000": "`tui_service` market workspace (parent epic overlap)",
    "T-1774807919547664000": "`tui_service` operations/settings narrow column",
    "T-1774807919579082000": "`tui_service` settings_layout / responsive settings",
    "T-1774807919611398000": "`tui_service` keymaps, `docs/` terminal notes",
    "T-1774807919706789000": "`tui_service` shared layout helpers post-split",
    "T-1774807919738245000": "`tui_service` positions sort UI, `TUI_POSITIONS_SORT` / snapshot ordering fields",
    "T-1774807919769659000": "`tui_service` styles/theme spike, `ui/` palette",
    "T-1774469242259152000": "`tui_service` cross-cutting UX wiring",
    "T-1774469228746075000": "`tui_service` discoverability.rs, palette, help",
    "T-1774469215296702000": "`tui_service` AppMode, input gating",
    "T-1774463349681383000": "`tui_service` architecture: `app.rs`, `input*.rs`, `ui/`",
    "T-1774354599853391000": "umbrella: `tui_service` + linked epics",
    # Backend / api / health
    "T-1774457999256276000": "`services/backend_service/`, shared heartbeat publisher in crates",
    "T-1774356804404405000": "`services/backend_service/`, `crates/nats_adapter/`",
    "T-1774455853504374000": "`crates/api/src/mock_data/`",
    "T-1774455366591912000": "`crates/api/` (health, strategy_controller audit)",
    "T-1774480256501115000": "`services/backend_service/` naming, `crates/api` runtime types",
    # RUSTSEC
    "T-1774519668508061000": "`agents/backend/Cargo.toml`, crates using `lru`",
    "T-1774519655045104000": "`agents/backend/Cargo.toml`, NATS/TLS stack",
    "T-1774519641624967000": "`agents/backend/Cargo.toml`, `nalgebra` / dependents",
    "T-1774519628263316000": "`agents/backend/Cargo.toml`, `instant` users",
    # Misc
    "T-1774528154110853000": "`tui_service` keyboard maps, macOS modifiers",
    "T-1774478963997885000": "`tui_service/`, `docs/` tui-pantry setup",
    "T-1774478946391532000": "`tui_service` logs tab, `Cargo.toml` ansi-to-tui",
    "T-1774477098254544000": "`services/backend_service/`, dev-only load tests",
    "T-1774456788201680000": "`tui_service` settings tab responsive behavior",
    "T-1774352896296846000": "`tui_service` + `crates/ledger/` read models / future API",
    "T-1774352915703004000": "parent epic — `nats_adapter`, `tui_service`, `strategy`",
}

HEADER = "**Likely files (implementer hint)**\n\n"


def add_comment(task_id: str, body: str) -> bool:
    payload = {
        "action": "add_comment",
        "task_id": task_id,
        "comment_type": "note",
        "content": HEADER + body,
        "output_format": "json",
    }
    args = json.dumps(payload, separators=(",", ":"))
    r = subprocess.run(
        [EXARP_SH, "-tool", "task_workflow", "-args", args],
        cwd=ROOT,
        env={**os.environ, "PROJECT_ROOT": ROOT},
        capture_output=True,
        text=True,
    )
    if r.returncode != 0:
        print(f"FAIL {task_id}: rc={r.returncode} stderr={r.stderr!r}", file=sys.stderr)
        return False
    try:
        data = json.loads(r.stdout)
        if not data.get("success", True):
            print(f"FAIL {task_id}: {data}", file=sys.stderr)
            return False
    except json.JSONDecodeError:
        if "error" in r.stdout.lower():
            print(f"FAIL {task_id}: {r.stdout[:500]}", file=sys.stderr)
            return False
    print(f"ok {task_id}")
    return True


def main() -> int:
    if not os.path.isfile(EXARP_SH):
        print(f"run_exarp_go.sh not found at {EXARP_SH}", file=sys.stderr)
        return 1
    ok = 0
    fail = 0
    for tid, body in sorted(NOTES.items()):
        if add_comment(tid, body):
            ok += 1
        else:
            fail += 1
    print(f"Done: {ok} ok, {fail} failed", file=sys.stderr)
    return 0 if fail == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
