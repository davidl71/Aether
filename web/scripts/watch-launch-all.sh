#!/usr/bin/env bash
# File watcher for launch-all-pwa-services.sh
# Automatically re-runs the launch script when it changes
# Useful for development and testing script changes
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"
LAUNCH_SCRIPT="${SCRIPT_DIR}/launch-all-pwa-services.sh"
WATCHER_PID=""
RELOAD_DELAY=1  # Wait 1 second after file change before re-running

# Colors for output
if [ -t 1 ] && command -v tput >/dev/null 2>&1; then
  RED=$(tput setaf 1)
  GREEN=$(tput setaf 2)
  YELLOW=$(tput setaf 3)
  BLUE=$(tput setaf 4)
  NC=$(tput sgr0)
else
  RED=''
  GREEN=''
  YELLOW=''
  BLUE=''
  NC=''
fi

# Cleanup function
cleanup() {
  echo "${YELLOW}[INFO] Shutting down file watcher...${NC}" >&2
  if [ -n "${WATCHER_PID:-}" ] && kill -0 "${WATCHER_PID}" 2>/dev/null; then
    kill "${WATCHER_PID}" 2>/dev/null || true
    wait "${WATCHER_PID}" 2>/dev/null || true
  fi
  exit 0
}

trap cleanup SIGINT SIGTERM EXIT

# Verify launch script exists
if [ ! -f "${LAUNCH_SCRIPT}" ]; then
  echo "${RED}[ERROR] Launch script not found${NC}" >&2
  echo "${RED}[PATH] Expected: ${LAUNCH_SCRIPT}${NC}" >&2
  exit 1
fi

# Make sure launch script is executable
chmod +x "${LAUNCH_SCRIPT}"

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

# Function to run launch script
run_launch_script() {
  echo "${BLUE}[INFO] Launch script changed, re-running in ${RELOAD_DELAY} second(s)...${NC}" >&2
  sleep "${RELOAD_DELAY}"
  echo "${GREEN}[INFO] Executing: ${LAUNCH_SCRIPT}${NC}" >&2
  echo "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━NC}" >&2
  # Run the launch script in foreground so output is visible
  bash "${LAUNCH_SCRIPT}" "$@" || {
    echo "${RED}[ERROR] Launch script exited with error code: $?${NC}" >&2
  }
  echo "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━2>&1
}

# Function to watch files with fswatch (macOS)
watch_with_fswatch() {
  echo "${GREEN}[INFO] Watching ${LAUNCH_SCRIPT} for changes...${NC}" >&2
  echo "${BLUE}[INFO] Using fswatch (macOS)${NC}" >&2
  echo "${YELLOW}[INFO] Press Ctrl+C to stop watching${NC}" >&2
  echo "" >&2

  # Run once initially
  run_launch_script "$@"

  # Watch for changes
  fswatch -o "${LAUNCH_SCRIPT}" | while read -r; do
    run_launch_script "$@"
  done
}

# Function to watch files with inotifywait (Linux)
watch_with_inotifywait() {
  echo "${GREEN}[INFO] Watching ${LAUNCH_SCRIPT} for changes...${NC}" >&2
  echo "${BLUE}[INFO] Using inotifywait (Linux)${NC}" >&2
  echo "${YELLOW}[INFO] Press Ctrl+C to stop watching${NC}" >&2
  echo "" >&2

  # Run once initially
  run_launch_script "$@"

  # Watch for changes
  while true; do
    inotifywait -e modify,create "${LAUNCH_SCRIPT}" >/dev/null 2>&1
    run_launch_script "$@"
  done
}

# Function to watch files with Python watchdog
watch_with_python() {
  echo "${GREEN}[INFO] Watching ${LAUNCH_SCRIPT} for changes...${NC}" >&2
  echo "${BLUE}[INFO] Using Python watchdog${NC}" >&2
  echo "${YELLOW}[INFO] Press Ctrl+C to stop watching${NC}" >&2
  echo "" >&2

  python3 << PYTHON_SCRIPT
import sys
import time
import subprocess
from pathlib import Path
from watchdog.observers import Observer
from watchdog.events import FileSystemEventHandler

class LaunchScriptHandler(FileSystemEventHandler):
    def __init__(self, launch_script, reload_delay=1, script_args=None):
        self.launch_script = Path(launch_script)
        self.reload_delay = reload_delay
        self.script_args = script_args or []
        self.last_run = 0

    def on_modified(self, event):
        if event.is_directory:
            return
        path = Path(event.src_path)
        if path != self.launch_script:
            return

        # Debounce rapid changes
        now = time.time()
        if now - self.last_run < self.reload_delay:
            return
        self.last_run = now

        print(f"[INFO] Launch script changed, re-running in {self.reload_delay} second(s)...", file=sys.stderr)
        time.sleep(self.reload_delay)
        print(f"[INFO] Executing: {self.launch_script}", file=sys.stderr)

        try:
            subprocess.run(
                ["bash", str(self.launch_script)] + self.script_args,
                check=False
            )
        except Exception as e:
            print(f"[ERROR] Failed to run launch script: {e}", file=sys.stderr)

if __name__ == "__main__":
    import json
    launch_script = "${LAUNCH_SCRIPT}"
    script_args = json.loads('${script_args_json:-[]}')

    handler = LaunchScriptHandler(launch_script, ${RELOAD_DELAY}, script_args)
    observer = Observer()
    observer.schedule(handler, str(Path(launch_script).parent), recursive=False)
    observer.start()

    # Run once initially
    handler.on_modified(type('Event', (), {'is_directory': False, 'src_path': launch_script})())

    try:
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        observer.stop()
    observer.join()
PYTHON_SCRIPT
}

# Main execution
WATCHER=$(check_file_watcher)

if [ -z "${WATCHER}" ]; then
  echo "${RED}[ERROR] No file watching tool found${NC}" >&2
  echo "${YELLOW}[INFO] Install one of the following:${NC}" >&2
  echo "  ${BLUE}brew install fswatch${NC}  # macOS" >&2
  echo "  ${BLUE}sudo apt-get install inotify-tools${NC}  # Linux" >&2
  echo "  ${BLUE}pip install watchdog${NC}  # Cross-platform (Python)" >&2
  exit 1
fi

# Store script arguments to pass to launch script
script_args=("$@")

# Convert script args to JSON for Python (if needed)
if [ "${WATCHER}" = "python-watchdog" ]; then
  script_args_json=$(python3 -c "import json, sys; json.dump(sys.argv[1:], sys.stdout)" "${script_args[@]}")
else
  script_args_json="[]"
fi

# Start watching based on available tool
case "${WATCHER}" in
  fswatch)
    watch_with_fswatch "$@"
    ;;
  inotifywait)
    watch_with_inotifywait "$@"
    ;;
  python-watchdog)
    watch_with_python "$@"
    ;;
  *)
    echo "${RED}[ERROR] Unknown watcher: ${WATCHER}${NC}" >&2
    exit 1
    ;;
esac
