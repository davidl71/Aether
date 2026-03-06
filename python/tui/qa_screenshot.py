"""
QA screenshot helper for the TUI.

Runs the TUI with a mock provider, waits for the first paint, saves an SVG
screenshot to a configurable path, then exits. Used by sanity/QA scripts to
capture a reference screenshot without a display.

Display each screen without simulating interaction: use --tab TAB_ID to switch
to that tab before capturing (tab ids from TUI_TAB_IDS), or --all-tabs to
capture every tab to separate files.

Usage:
    python -m python.tui.qa_screenshot [--output-dir DIR] [--output-name NAME]
    python -m python.tui.qa_screenshot --tab dashboard-tab
    python -m python.tui.qa_screenshot --all-tabs
    TUI_QA_SCREENSHOT_DIR=build/qa/tui python -m python.tui.qa_screenshot

Output defaults to build/qa/tui/tui-screenshot-<timestamp>.svg (build/ is gitignored).
"""

from __future__ import annotations

import argparse
import asyncio
import json
import logging
import os
import subprocess
import sys
from pathlib import Path
from datetime import datetime
from typing import Any

# Configure logging before importing app (reduces TUI log noise during QA run)
logging.basicConfig(
    level=logging.WARNING,
    format="%(message)s",
)
logger = logging.getLogger(__name__)


def _parse_args() -> argparse.Namespace:
    from python.tui.app import TUI_TAB_IDS

    parser = argparse.ArgumentParser(
        description="Run TUI briefly and save an SVG screenshot for QA."
    )
    parser.add_argument(
        "--output-dir",
        default=os.environ.get("TUI_QA_SCREENSHOT_DIR", "build/qa/tui"),
        help="Directory to write screenshot (default: build/qa/tui or TUI_QA_SCREENSHOT_DIR)",
    )
    parser.add_argument(
        "--output-name",
        default=None,
        help="Basename for the screenshot file (default: tui-screenshot-<timestamp>.svg)",
    )
    parser.add_argument(
        "--delay",
        type=float,
        default=1.5,
        help="Seconds to wait before taking screenshot (default: 1.5)",
    )
    parser.add_argument(
        "--tab",
        choices=TUI_TAB_IDS,
        default=None,
        metavar="TAB_ID",
        help="Switch to this tab before capturing (no user interaction). Use --list-tabs to see ids.",
    )
    parser.add_argument(
        "--all-tabs",
        action="store_true",
        help="Capture each tab to a separate file (tui-<tab_id>.svg) without simulating interaction.",
    )
    parser.add_argument(
        "--list-tabs",
        action="store_true",
        help="Print tab ids and exit (for use with --tab).",
    )
    return parser.parse_args()


def _seed_logs_tab_for_screenshot(app: Any) -> None:
    """Seed the Logs tab with sample lines so the QA screenshot shows multiple log lines.

    QA runs with logging at WARNING, so the real buffer often has only one line.
    """
    try:
        from python.tui.components.logs_tab import LogsTab
        logs_tab = app.query_one(LogsTab)
        now = datetime.utcnow().strftime("%Y-%m-%d %H:%M:%S")
        sample_lines = [
            f"{now},123 [INFO] python.tui.app: TUI application mounted",
            f"{now},124 [DEBUG] python.tui.providers: BackendHealthAggregator started for backends: ['ib', 'tws', 'alpaca', 'tastytrade']",
            f"{now},125 [INFO] python.tui.app: Snapshot updated from mock provider",
            f"{now},126 [DEBUG] python.tui.components.dashboard: Dashboard tab updated with 3 symbols",
            f"{now},127 [WARNING] python.tui.providers: Backend alpaca unreachable (Missing API key)",
            f"{now},128 [WARNING] python.tui.providers: Backend tastytrade unreachable (Missing credentials)",
            f"{now},129 [INFO] python.tui.app: Config file watch active",
        ]
        logs_tab.load_buffer(sample_lines)
    except Exception:
        pass


def _git_rev() -> str | None:
    """Return short git rev if repo available, else None."""
    try:
        r = subprocess.run(
            ["git", "rev-parse", "--short", "HEAD"],
            cwd=Path(__file__).resolve().parents[2],
            capture_output=True,
            text=True,
            timeout=2,
        )
        return r.stdout.strip() or None if r.returncode == 0 else None
    except Exception:
        return None


def _debug_metadata(tab_id: str | None, **extra: Any) -> dict:
    """Build debug metadata dict for injection into SVG."""
    import textual
    data: dict = {
        "tab_id": tab_id or "single",
        "timestamp": datetime.utcnow().strftime("%Y-%m-%dT%H:%M:%SZ"),
        "provider": "mock",
        "textual_version": getattr(textual, "__version__", "unknown"),
        "python_version": f"{sys.version_info.major}.{sys.version_info.minor}.{sys.version_info.micro}",
    }
    rev = _git_rev()
    if rev:
        data["git_rev"] = rev
    data.update(_json_safe(extra))
    return data


def _json_safe(obj: Any) -> Any:
    """Return a JSON-serializable copy of obj (dict/list/str/bool/int/float, None)."""
    if obj is None or isinstance(obj, (bool, int, float, str)):
        return obj
    if isinstance(obj, dict):
        return {k: _json_safe(v) for k, v in obj.items()}
    if isinstance(obj, (list, tuple)):
        return [_json_safe(v) for v in obj]
    return str(obj)


def inject_svg_debug_metadata(path: Path, tab_id: str | None = None, **extra: Any) -> None:
    """Inject hidden debug metadata into an SVG file written by save_screenshot.

    Inserts a single-line comment after the opening <svg> tag so the file remains
    valid and the metadata is easy to parse (e.g. grep for 'tui-qa:' then parse JSON).
    When status_line or backend_health is in extra, draws status line at the bottom
    of the SVG so the provider/backend indicators are visible (Textual may not
    render docked status bar into the export). Uses one-word colored pills when
    backend_health is provided.
    """
    if not path.exists():
        return
    raw = path.read_text(encoding="utf-8", errors="replace")
    # Insert after first '>\n' of the root <svg ...>
    marker = ">\n"
    idx = raw.find(marker)
    if idx == -1:
        return
    insert_at = idx + len(marker)
    meta = _debug_metadata(tab_id=tab_id, **extra)
    comment = f"<!-- tui-qa: {json.dumps(meta)} -->\n"
    new_content = raw[:insert_at] + comment + raw[insert_at:]

    # Draw status line at bottom so health bar / pills are visible in screenshot
    import re as _re
    environment = (extra.get("environment") or "").strip().lower()
    if environment not in ("mock", "paper", "live"):
        environment = "mock"
    backend_health = extra.get("backend_health")
    status_line = extra.get("status_line") if isinstance(extra.get("status_line"), str) else None

    if backend_health and isinstance(backend_health, dict) and backend_health.get("status") is None:
        # New shape: dict of backends -> draw colored pills
        from python.tui.components.snapshot_display import BACKEND_SHORT_LABELS
        pill_fill = {"ok": "#22c55e", "disabled": "#eab308", "checking": "#eab308"}
        pills_x = 58
        parts_svg = []
        for name, payload in sorted(backend_health.items()):
            if not isinstance(payload, dict):
                continue
            status = payload.get("status", "error")
            label = BACKEND_SHORT_LABELS.get(name.lower(), name)
            fill = pill_fill.get(status, "#ef4444")
            parts_svg.append(f'<tspan x="{pills_x}" fill="{fill}" font-weight="bold"> {label} </tspan>')
            pills_x += len(f" {label} ") * 7  # approximate width
        pills_markup = "".join(parts_svg)
        vb_match = _re.search(r'viewBox="([^"]+)"', raw)
        if vb_match and parts_svg:
            import re as _re
            parts = vb_match.group(1).strip().split()
            if len(parts) >= 4:
                try:
                    height = float(parts[3])
                    width_svg = float(parts[2]) if len(parts) >= 3 else 994
                    y = max(16, height - 14)
                    badge_fill = {"mock": "#5a9aa8", "paper": "#b8a84a", "live": "#c44"}.get(
                        environment, "#9e9e9e"
                    )
                    visible = (
                        f'<g id="tui-qa-status-line" font-family="monospace" font-size="12">'
                        f'<rect x="0" y="{y - 11}" width="{width_svg:.0f}" height="14" fill="#1a1a1a" opacity="0.95"/>'
                        f'<text x="10" y="{y}" fill="{badge_fill}" font-weight="bold">[{environment.upper()}]</text>'
                        f'<text x="58" y="{y}">{pills_markup}</text></g>\n'
                    )
                    new_content = new_content.replace("</svg>", visible + "</svg>")
                except (ValueError, TypeError):
                    pass
    elif status_line:
        escaped = (
            status_line.replace("&", "&amp;")
            .replace("<", "&lt;")
            .replace(">", "&gt;")
            .replace('"', "&quot;")
        )
        vb_match = _re.search(r'viewBox="([^"]+)"', raw)
        if vb_match:
            parts = vb_match.group(1).strip().split()
            if len(parts) >= 4:
                try:
                    height = float(parts[3])
                    width_svg = float(parts[2]) if len(parts) >= 3 else 994
                    y = max(16, height - 14)
                    badge_fill = {"mock": "#5a9aa8", "paper": "#b8a84a", "live": "#c44"}.get(
                        environment, "#9e9e9e"
                    )
                    visible = (
                        f'<g id="tui-qa-status-line" font-family="monospace" font-size="12">'
                        f'<rect x="0" y="{y - 11}" width="{width_svg:.0f}" height="14" fill="#1a1a1a" opacity="0.95"/>'
                        f'<text x="10" y="{y}" fill="{badge_fill}" font-weight="bold">[{environment.upper()}]</text>'
                        f'<text x="58" y="{y}" fill="#b0b0b0">{escaped[:220]}</text></g>\n'
                    )
                    new_content = new_content.replace("</svg>", visible + "</svg>")
                except (ValueError, TypeError):
                    pass

    path.write_text(new_content, encoding="utf-8")


class QAScreenshotApp:
    """Wrapper that runs TUIApp and triggers a screenshot after a delay then exits."""

    def __init__(
        self,
        output_path: Path,
        delay: float,
        tab_id: str | None = None,
        all_tabs: bool = False,
        output_dir: Path | None = None,
    ):
        self.output_path = output_path
        self.output_dir = output_dir or output_path.parent
        self.delay = delay
        self.tab_id = tab_id
        self.all_tabs = all_tabs
        self._app = None
        self._saved_path: str | None = None
        self._saved_paths: list[str] = []

    async def run(self) -> str | list[str] | None:
        from python.tui.app import TUIApp, TUI_TAB_IDS
        from python.tui.providers import MockProvider
        from python.tui.config import load_config

        config = load_config()
        provider = MockProvider()
        saved_paths: list[str] = []
        saved_path: list[str] = []  # single-element list so closure can set it

        class QAApp(TUIApp):
            def on_mount(self) -> None:
                super().on_mount()

                def _status_bar_debug() -> dict:
                    """Read current status bar state for debug metadata (provider health bar)."""
                    try:
                        from python.tui.components.snapshot_display import (
                            StatusBar,
                            format_status_line,
                            get_environment,
                        )
                        bar = self.query_one("#status-bar", StatusBar)
                        label = getattr(bar, "provider_label", "") or ""
                        health = getattr(bar, "backend_health", None)
                        snapshot = getattr(bar, "snapshot", None)
                        environment = getattr(bar, "environment", "") or get_environment(
                            self.provider, snapshot
                        )
                        line = format_status_line(label, snapshot, health)
                        return {
                            "provider_label": label,
                            "backend_health": health,
                            "status_line": line,
                            "environment": environment,
                        }
                    except Exception as e:
                        return {"status_bar_error": str(e)}

                def capture() -> None:
                    try:
                        # Ensure status bar is populated (backend health, provider label) before
                        # screenshot so the exported SVG shows the health dashbar / pills.
                        self._update_snapshot()
                        try:
                            from python.tui.components.snapshot_display import StatusBar
                            status_bar = self.query_one("#status-bar", StatusBar)
                            # If aggregator hasn't run yet, inject fake backend_health so pills show in screenshot
                            health = getattr(status_bar, "backend_health", None)
                            if not health or (isinstance(health, dict) and len(health) == 0):
                                status_bar.backend_health = {
                                    "ib": {"status": "ok"},
                                    "alpaca": {"status": "disabled", "error": "Missing API key"},
                                    "tastytrade": {"status": "disabled", "error": "Missing credentials"},
                                    "tws": {"status": "ok"},
                                }
                            status_bar._refresh()
                        except Exception:
                            pass

                        def do_screenshot() -> None:
                            try:
                                if self.all_tabs:
                                    for tid in TUI_TAB_IDS:
                                        self.switch_to_tab(tid)
                                        if tid == "logs-tab":
                                            _seed_logs_tab_for_screenshot(self)
                                        out_path = self.output_dir / f"tui-{tid}.svg"
                                        out = self.save_screenshot(
                                            path=str(out_path.parent),
                                            filename=out_path.name,
                                        )
                                        if out:
                                            saved_paths.append(out)
                                            inject_svg_debug_metadata(
                                                Path(out),
                                                tab_id=tid,
                                                **_status_bar_debug(),
                                            )
                                            logger.info("Screenshot saved: %s", out)
                                else:
                                    if self.tab_id:
                                        self.switch_to_tab(self.tab_id)
                                    if self.tab_id == "logs-tab":
                                        _seed_logs_tab_for_screenshot(self)
                                    out = self.save_screenshot(
                                        path=str(self.output_path.parent),
                                        filename=self.output_path.name,
                                    )
                                    if out:
                                        saved_path.append(out)
                                        inject_svg_debug_metadata(
                                            Path(out),
                                            tab_id=getattr(self, "tab_id", None),
                                            **_status_bar_debug(),
                                        )
                                        logger.info("Screenshot saved: %s", out)
                            except Exception as e:
                                logger.error("Failed to save screenshot: %s", e)
                            self.exit()

                        # Defer screenshot so the status bar repaint is included in the export.
                        self.set_timer(0.25, do_screenshot)
                    except Exception as e:
                        logger.error("Failed to update snapshot before screenshot: %s", e)
                        self.exit()

                def delayed_capture() -> None:
                    if self.tab_id and not self.all_tabs:
                        self.switch_to_tab(self.tab_id)
                        self.set_timer(0.35, capture)
                    else:
                        capture()

                self.set_timer(self.delay, delayed_capture)

        app = QAApp(provider, config)
        app.output_path = self.output_path
        app.output_dir = self.output_dir
        app.tab_id = self.tab_id
        app.all_tabs = self.all_tabs
        app.delay = self.delay
        self._app = app

        await app.run_async()
        if self.all_tabs and saved_paths:
            self._saved_paths = saved_paths
            return self._saved_paths
        self._saved_path = saved_path[0] if saved_path else None
        return self._saved_path or (str(self.output_path) if self.output_path.exists() else None)


def main() -> int:
    args = _parse_args()

    if args.list_tabs:
        from python.tui.app import TUI_TAB_IDS
        for tid in TUI_TAB_IDS:
            print(tid)
        return 0

    out_dir = Path(args.output_dir)
    out_dir.mkdir(parents=True, exist_ok=True)

    if args.all_tabs:
        output_path = out_dir / "tui-dashboard-tab.svg"
    elif args.output_name:
        name = args.output_name if args.output_name.endswith(".svg") else f"{args.output_name}.svg"
        output_path = out_dir / name
    else:
        output_path = out_dir / f"tui-screenshot-{datetime.now().strftime('%Y%m%d-%H%M%S')}.svg"

    try:
        result = asyncio.run(
            QAScreenshotApp(
                output_path,
                args.delay,
                tab_id=args.tab,
                all_tabs=args.all_tabs,
                output_dir=out_dir,
            ).run()
        )
        if result is None:
            return 1
        if isinstance(result, list):
            for p in result:
                print(p)
        else:
            print(result)
        return 0
    except Exception as e:
        logger.error("QA screenshot failed: %s", e)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
