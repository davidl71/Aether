#!/usr/bin/env bash
# Watchdog script for backend services
# Monitors services and restarts them if they fail
# Uses NATS for secure service control (if available)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
LOG_DIR="${ROOT_DIR}/logs"
CONFIG_FILE="${ROOT_DIR}/config/services.toml"
PID_FILE="${LOG_DIR}/watchdog.pid"
CHECK_INTERVAL=${CHECK_INTERVAL:-30}  # Check every 30 seconds
MAX_RESTART_ATTEMPTS=${MAX_RESTART_ATTEMPTS:-5}
RESTART_DELAY=${RESTART_DELAY:-10}  # Wait 10 seconds before restart

# Colors for output
if [ -t 1 ] && command -v tput >/dev/null 2>&1; then
  RED=$(tput setaf 1)
  GREEN=$(tput setaf 2)
  YELLOW=$(tput setaf 3)
  NC=$(tput sgr0)
else
  RED=''
  GREEN=''
  YELLOW=''
  NC=''
fi

# Service configuration
declare -A SERVICE_PORTS=(
  ["alpaca"]=8000
  ["ib"]=8002
  ["discount_bank"]=8003
  ["risk_free_rate"]=8004
  ["tastytrade"]=8005
)

declare -A SERVICE_SCRIPTS=(
  ["alpaca"]="start_alpaca_service.sh"
  ["ib"]="start_ib_service.sh"
  ["discount_bank"]="start_discount_bank_service.sh"
  ["risk_free_rate"]="start_risk_free_rate_service.sh"
  ["tastytrade"]="start_tastytrade_service.sh"
)

declare -A RESTART_COUNTS=()

# Create log directory
mkdir -p "$LOG_DIR"

# Logging function
log() {
  echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*" | tee -a "${LOG_DIR}/watchdog.log"
}

# Check if service is running
is_service_running() {
  local service_name="$1"
  local port="${SERVICE_PORTS[$service_name]}"

  if [ -z "$port" ]; then
    return 1
  fi

  if command -v lsof >/dev/null 2>&1; then
    lsof -ti ":${port}" >/dev/null 2>&1
  elif command -v netstat >/dev/null 2>&1; then
    netstat -an 2>/dev/null | grep -q ":${port}.*LISTEN"
  else
    return 1
  fi
}

# Check if service is enabled
is_service_enabled() {
  local service_name="$1"

  if [ ! -f "$CONFIG_FILE" ]; then
    # Default to enabled if config doesn't exist
    return 0
  fi

  # Parse TOML config (simple grep-based approach)
  if grep -q "\[services\.${service_name}\]" "$CONFIG_FILE" 2>/dev/null; then
    if grep -A 5 "\[services\.${service_name}\]" "$CONFIG_FILE" | grep -q "enabled = false"; then
      return 1
    fi
  fi

  return 0
}

# Restart service
restart_service() {
  local service_name="$1"
  local script_name="${SERVICE_SCRIPTS[$service_name]}"

  if [ -z "$script_name" ]; then
    log "${RED}ERROR:${NC} No start script found for service: $service_name"
    return 1
  fi

  local script_path="${ROOT_DIR}/scripts/${script_name}"

  if [ ! -f "$script_path" ]; then
    log "${RED}ERROR:${NC} Start script not found: $script_path"
    return 1
  fi

  log "${YELLOW}Restarting service:${NC} $service_name"

  # Stop first (if running)
  local stop_script="${script_name/start_/stop_}"
  local stop_path="${ROOT_DIR}/scripts/${stop_script}"
  if [ -f "$stop_path" ]; then
    bash "$stop_path" >/dev/null 2>&1 || true
    sleep 2
  fi

  # Start service
  if bash "$script_path" >/dev/null 2>&1; then
    log "${GREEN}✓${NC} Service $service_name restarted successfully"

    # Reset restart count on success
    RESTART_COUNTS[$service_name]=0
    return 0
  else
    log "${RED}✗${NC} Failed to restart service: $service_name"
    return 1
  fi
}

# Check service health via API
check_service_health() {
  local service_name="$1"
  local port="${SERVICE_PORTS[$service_name]}"
  local health_url="http://localhost:${port}/api/health"

  # Try to check health endpoint (with timeout)
  if command -v curl >/dev/null 2>&1; then
    curl -s --max-time 2 "$health_url" >/dev/null 2>&1
  elif command -v wget >/dev/null 2>&1; then
    wget -q --timeout=2 -O /dev/null "$health_url" >/dev/null 2>&1
  else
    # Fallback to port check
    is_service_running "$service_name"
  fi
}

# Main monitoring loop
monitor_services() {
  log "${GREEN}Watchdog started${NC} - Checking services every ${CHECK_INTERVAL}s"

  while true; do
    for service_name in "${!SERVICE_PORTS[@]}"; do
      # Skip if service is disabled
      if ! is_service_enabled "$service_name"; then
        continue
      fi

      # Check if service is running
      if ! is_service_running "$service_name"; then
        local count="${RESTART_COUNTS[$service_name]:-0}"

        if [ "$count" -lt "$MAX_RESTART_ATTEMPTS" ]; then
          log "${YELLOW}Service $service_name is not running${NC} (attempt $((count + 1))/$MAX_RESTART_ATTEMPTS)"

          # Wait before restarting
          sleep "$RESTART_DELAY"

          if restart_service "$service_name"; then
            RESTART_COUNTS[$service_name]=0
          else
            RESTART_COUNTS[$service_name]=$((count + 1))
          fi
        else
          log "${RED}Service $service_name failed $MAX_RESTART_ATTEMPTS times${NC} - Giving up"
          # Reset count after a longer delay
          sleep $((RESTART_DELAY * 2))
          RESTART_COUNTS[$service_name]=0
        fi
      else
        # Service is running - check health
        if ! check_service_health "$service_name"; then
          log "${YELLOW}Service $service_name is running but unhealthy${NC}"
          # Could implement health-based restart here if needed
        else
          # Service is healthy - reset restart count
          RESTART_COUNTS[$service_name]=0
        fi
      fi
    done

    sleep "$CHECK_INTERVAL"
  done
}

# Signal handlers
cleanup() {
  log "${YELLOW}Watchdog shutting down...${NC}"
  rm -f "$PID_FILE"
  exit 0
}

trap cleanup SIGTERM SIGINT

# Check if already running
if [ -f "$PID_FILE" ]; then
  old_pid=$(cat "$PID_FILE")
  if ps -p "$old_pid" >/dev/null 2>&1; then
    log "${YELLOW}Watchdog already running (PID: $old_pid)${NC}"
    exit 1
  else
    rm -f "$PID_FILE"
  fi
fi

# Write PID file
echo $$ > "$PID_FILE"

# Start monitoring
monitor_services
