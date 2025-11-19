#!/usr/bin/env bash
# Auto-reload wrapper for IB Gateway
# Monitors config files and automatically restarts gateway on changes
# Similar to how Vite and uvicorn auto-reload
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
GATEWAY_PID=""
WATCHER_PID=""
RELOAD_DELAY=2  # Wait 2 seconds after file change before restarting

# Cleanup function
cleanup() {
  echo "[INFO] Shutting down..." >&2
  if [ -n "${GATEWAY_PID:-}" ] && kill -0 "${GATEWAY_PID}" 2>/dev/null; then
    echo "[INFO] Stopping IB Gateway (PID: ${GATEWAY_PID})..." >&2
    kill "${GATEWAY_PID}" 2>/dev/null || true
    wait "${GATEWAY_PID}" 2>/dev/null || true
  fi
  if [ -n "${WATCHER_PID:-}" ] && kill -0 "${WATCHER_PID}" 2>/dev/null; then
    echo "[INFO] Stopping file watcher (PID: ${WATCHER_PID})..." >&2
    kill "${WATCHER_PID}" 2>/dev/null || true
    wait "${WATCHER_PID}" 2>/dev/null || true
  fi
  exit 0
}

trap cleanup SIGINT SIGTERM EXIT

# Find config file
CONFIG_FILE="${SCRIPT_DIR}/root/conf.yaml"
if [ ! -f "${CONFIG_FILE}" ]; then
  CONFIG_FILE="${SCRIPT_DIR}/root/conf.tws.yaml"
fi

# Normalize config file path
if [ -f "${CONFIG_FILE}" ]; then
  if command -v realpath >/dev/null 2>&1; then
    CONFIG_FILE="$(realpath "${CONFIG_FILE}")"
  else
    CONFIG_FILE="$(cd "$(dirname "${CONFIG_FILE}")" && pwd)/$(basename "${CONFIG_FILE}")"
  fi
fi

if [ ! -f "${CONFIG_FILE}" ]; then
  echo "[ERROR] IB Gateway configuration file not found" >&2
  echo "[PATH] Searched: ${SCRIPT_DIR}/root/conf.yaml" >&2
  echo "[PATH] Searched: ${SCRIPT_DIR}/root/conf.tws.yaml" >&2
  exit 1
fi

# Files to watch for changes
WATCH_PATHS=(
  "${CONFIG_FILE}"
  "${SCRIPT_DIR}/root/conf.yaml"
  "${SCRIPT_DIR}/root/conf.tws.yaml"
  "${SCRIPT_DIR}/root/logback.xml"
)

# Function to start gateway
start_gateway() {
  # Stop existing gateway if running
  if [ -n "${GATEWAY_PID:-}" ] && kill -0 "${GATEWAY_PID}" 2>/dev/null; then
    echo "[INFO] Stopping existing gateway (PID: ${GATEWAY_PID})..." >&2
    kill "${GATEWAY_PID}" 2>/dev/null || true
    wait "${GATEWAY_PID}" 2>/dev/null || true
    sleep 1
  fi

  echo "[INFO] Starting IB Gateway..." >&2
  echo "[PATH] Config file: ${CONFIG_FILE}" >&2

  # Run gateway in background
  "${SCRIPT_DIR}/run-gateway.sh" > "${SCRIPT_DIR}/gateway.log" 2>&1 &
  GATEWAY_PID=$!

  echo "[INFO] IB Gateway started (PID: ${GATEWAY_PID})" >&2
  echo "[INFO] Logs: [PATH] ${SCRIPT_DIR}/gateway.log" >&2
}

# Function to check for file watching tools
check_file_watcher() {
  # Check for fswatch (macOS, install via: brew install fswatch)
  if command -v fswatch >/dev/null 2>&1; then
    echo "fswatch"
    return 0
  fi

  # Check for inotifywait (Linux, usually in inotify-tools package)
  if command -v inotifywait >/dev/null 2>&1; then
    echo "inotifywait"
    return 0
  fi

  # Check for Python watchdog (cross-platform)
  if command -v python3 >/dev/null 2>&1; then
    if python3 -c "import watchdog" 2>/dev/null; then
      echo "python-watchdog"
      return 0
    fi
  fi

  return 1
}

# Function to watch files with fswatch (macOS)
watch_with_fswatch() {
  fswatch -o "${WATCH_PATHS[@]}" | while read -r; do
    echo "[INFO] Configuration file changed, reloading in ${RELOAD_DELAY} seconds..." >&2
    sleep "${RELOAD_DELAY}"
    start_gateway
  done
}

# Function to watch files with inotifywait (Linux)
watch_with_inotifywait() {
  while true; do
    inotifywait -e modify,create,delete "${WATCH_PATHS[@]}" >/dev/null 2>&1
    echo "[INFO] Configuration file changed, reloading in ${RELOAD_DELAY} seconds..." >&2
    sleep "${RELOAD_DELAY}"
    start_gateway
  done
}

# Function to watch files with Python watchdog
watch_with_python() {
  python3 << PYTHON_SCRIPT
import sys
import time
import subprocess
from pathlib import Path
from watchdog.observers import Observer
from watchdog.events import FileSystemEventHandler

class GatewayReloadHandler(FileSystemEventHandler):
    def __init__(self, script_dir, config_file, reload_delay=2):
        self.script_dir = Path(script_dir)
        self.config_file = Path(config_file)
        self.reload_delay = reload_delay
        self.gateway_pid = None
        self.last_reload = 0

    def on_modified(self, event):
        if event.is_directory:
            return
        path = Path(event.src_path)
        # Only watch specific files
        watch_files = [
            self.config_file,
            self.script_dir / "root" / "conf.yaml",
            self.script_dir / "root" / "conf.tws.yaml",
            self.script_dir / "root" / "logback.xml",
        ]
        if path in watch_files:
            # Debounce: don't reload more than once per second
            now = time.time()
            if now - self.last_reload < self.reload_delay:
                return
            self.last_reload = now
            print(f"[INFO] Configuration file changed: {path}", file=sys.stderr)
            print(f"[INFO] Reloading in {self.reload_delay} seconds...", file=sys.stderr)
            time.sleep(self.reload_delay)
            self.reload_gateway()

    def reload_gateway(self):
        # Stop existing gateway
        if self.gateway_pid and self.gateway_pid.poll() is None:
            print(f"[INFO] Stopping gateway (PID: {self.gateway_pid.pid})...", file=sys.stderr)
            self.gateway_pid.terminate()
            try:
                self.gateway_pid.wait(timeout=5)
            except subprocess.TimeoutExpired:
                self.gateway_pid.kill()

        # Start new gateway
        print("[INFO] Starting IB Gateway...", file=sys.stderr)
        run_script = self.script_dir / "run-gateway.sh"
        self.gateway_pid = subprocess.Popen(
            [str(run_script)],
            cwd=str(self.script_dir),
            stdout=open(self.script_dir / "gateway.log", "a"),
            stderr=subprocess.STDOUT
        )
        print(f"[INFO] IB Gateway started (PID: {self.gateway_pid.pid})", file=sys.stderr)

if __name__ == "__main__":
    script_dir = "${SCRIPT_DIR}"
    config_file = "${CONFIG_FILE}"
    reload_delay = ${RELOAD_DELAY}

    handler = GatewayReloadHandler(script_dir, config_file, reload_delay)
    observer = Observer()
    observer.schedule(handler, str(Path(script_dir) / "root"), recursive=False)
    observer.start()

    try:
        # Start initial gateway
        handler.reload_gateway()

        # Keep watching
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        observer.stop()
    observer.join()
PYTHON_SCRIPT
}

# Main execution
echo "[INFO] IB Gateway Auto-Reload Wrapper" >&2
echo "[INFO] Monitoring files for changes..." >&2
for path in "${WATCH_PATHS[@]}"; do
  if [ -f "${path}" ]; then
    echo "[PATH] Watching: ${path}" >&2
  fi
done
echo ""

# Check for file watcher
WATCHER_TOOL=$(check_file_watcher || echo "")

if [ -z "${WATCHER_TOOL}" ]; then
  echo "[ERROR] No file watching tool found" >&2
  echo "[INFO] Install one of the following:" >&2
  echo "  macOS:   brew install fswatch" >&2
  echo "  Linux:   sudo apt-get install inotify-tools" >&2
  echo "  Python:  pip install watchdog" >&2
  echo "" >&2
  echo "[INFO] Falling back to single-run mode (no auto-reload)" >&2
  "${SCRIPT_DIR}/run-gateway.sh"
  exit $?
fi

echo "[INFO] Using file watcher: ${WATCHER_TOOL}" >&2
echo ""

# Start initial gateway
start_gateway

# Start file watcher in background
case "${WATCHER_TOOL}" in
  fswatch)
    watch_with_fswatch &
    WATCHER_PID=$!
    ;;
  inotifywait)
    watch_with_inotifywait &
    WATCHER_PID=$!
    ;;
  python-watchdog)
    watch_with_python &
    WATCHER_PID=$!
    ;;
esac

echo "[INFO] Auto-reload enabled. Press Ctrl+C to stop." >&2
echo ""

# Wait for gateway process
wait "${GATEWAY_PID}" 2>/dev/null || true
