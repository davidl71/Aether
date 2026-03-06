#!/usr/bin/env python3
"""
Run the TUI in a subprocess and restart it when Python source or config changes.
Used by dev_watch_tui.sh when watchfiles is available (e.g. uv sync / pip install).
"""
from __future__ import annotations

import os
import subprocess
import sys
import time
from pathlib import Path


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    watch_dirs = [
        root / "python" / "tui",
        root / "python" / "integration",
        root / "config",
    ]
    watch_dirs = [d for d in watch_dirs if d.exists()]
    if not watch_dirs:
        print("No directories to watch.", file=sys.stderr)
        return 1

    run_cmd = sys.argv[1:]
    if not run_cmd:
        script = root / "scripts" / "run_python_tui.sh"
        run_cmd = ["bash", str(script)] if script.exists() else [sys.executable, "-m", "python.tui"]
    elif run_cmd[0].endswith(".sh"):
        run_cmd = ["bash", run_cmd[0]] + list(run_cmd[1:])
    proc: subprocess.Popen | None = None
    last_restart = 0.0
    debounce_sec = 0.5

    def start() -> None:
        nonlocal proc
        if proc is not None and proc.poll() is None:
            proc.terminate()
            try:
                proc.wait(timeout=3)
            except subprocess.TimeoutExpired:
                proc.kill()
        proc = subprocess.Popen(
            run_cmd,
            cwd=root,
            env={
                **os.environ,
                "PYTHONPATH": os.pathsep.join(
                    [str(root / "python"), os.environ.get("PYTHONPATH", "")]
                ),
            },
        )
        print(f"[{time.strftime('%H:%M:%S')}] TUI started (PID: {proc.pid})")

    try:
        import watchfiles
    except ImportError:
        print(
            "watchfiles not found. Install with: uv sync  # or pip install watchfiles",
            file=sys.stderr,
        )
        return 1

    start()
    for changes in watchfiles.watch(*watch_dirs, debounce=400):
        # changes: set of (Change, path) or path-like; only restart on .py / .json
        paths = []
        for item in changes:
            p = item[1] if isinstance(item, (tuple, list)) and len(item) >= 2 else item
            path_str = str(p)
            if path_str.endswith((".py", ".json")):
                paths.append(path_str)
        if not paths:
            continue
        if time.monotonic() - last_restart < debounce_sec:
            continue
        last_restart = time.monotonic()
        print(f"[{time.strftime('%H:%M:%S')}] Change detected; restarting TUI...")
        start()

    return 0


if __name__ == "__main__":
    sys.exit(main())
