#!/usr/bin/env bash
# Unified Service Manager - Start/stop/restart/status for all services
# Consolidates all individual start_*.sh and stop_*.sh scripts
#
# Usage:
#   ./scripts/service_manager.sh start <service>
#   ./scripts/service_manager.sh stop <service>
#   ./scripts/service_manager.sh restart <service>
#   ./scripts/service_manager.sh status [service]
#   ./scripts/service_manager.sh start-all
#   ./scripts/service_manager.sh stop-all
#
# Services: ib, alpaca, tastytrade, tradestation, discount_bank, risk_free_rate, rust_backend, nats, web

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Logging functions
log_info() { echo -e "${GREEN}[info]${NC} $*"; }
log_warn() { echo -e "${YELLOW}[warn]${NC} $*"; }
log_error() { echo -e "${RED}[error]${NC} $*"; }

# Load config
CONFIG_FILE="${PROJECT_ROOT}/config/config.json"
if [ ! -f "$CONFIG_FILE" ]; then
  CONFIG_FILE="${PROJECT_ROOT}/config/config.example.json"
fi

# Get port from config (with default fallback)
get_port() {
  local service=$1
  local default_port=$2

  if command -v jq &>/dev/null && [ -f "$CONFIG_FILE" ]; then
    # Try to extract port from config
    local port=$(jq -r ".services.${service}.port // ${default_port}" "$CONFIG_FILE" 2>/dev/null || echo "$default_port")
    echo "$port"
  else
    echo "$default_port"
  fi
}

# Service definitions: name, port, start_command
declare -A SERVICES=(
  ["ib"]="8002|cd ${PROJECT_ROOT}/python/services && python -m uvicorn ib_service:app --host 0.0.0.0 --port"
  ["alpaca"]="8000|cd ${PROJECT_ROOT}/python/services && python -m uvicorn alpaca_service:app --host 0.0.0.0 --port"
  ["tastytrade"]="8005|cd ${PROJECT_ROOT}/python/services && python -m uvicorn tastytrade_service:app --host 0.0.0.0 --port"
  ["tradestation"]="8001|cd ${PROJECT_ROOT}/python/services && python -m uvicorn tradestation_service:app --host 0.0.0.0 --port"
  ["discount_bank"]="8003|cd ${PROJECT_ROOT}/python/services && python -m uvicorn discount_bank_service:app --host 0.0.0.0 --port"
  ["risk_free_rate"]="8004|cd ${PROJECT_ROOT}/python/services && python -m uvicorn risk_free_rate_service:app --host 0.0.0.0 --port"
  ["rust_backend"]="8080|cd ${PROJECT_ROOT}/agents && cargo run --release -- --rest-port 8080 --grpc-port 50051"
  ["nats"]="4222|nats-server -js -DV"
  ["web"]="5173|cd ${PROJECT_ROOT}/web && npm run dev -- --host 0.0.0.0 --port"
)

# Check if service is running
is_running() {
  local port=$1
  lsof -ti ":${port}" >/dev/null 2>&1
}

# Get PID for service
get_pid() {
  local port=$1
  lsof -ti ":${port}" 2>/dev/null || echo ""
}

# Start a service
start_service() {
  local service=$1

  if [ -z "${SERVICES[$service]:-}" ]; then
    log_error "Unknown service: $service"
    log_info "Available services: ${!SERVICES[*]}"
    return 1
  fi

  local config="${SERVICES[$service]}"
  local default_port=$(echo "$config" | cut -d'|' -f1)
  local start_cmd=$(echo "$config" | cut -d'|' -f2-)

  # Get actual port (from config or default)
  local port=$(get_port "$service" "$default_port")

  # Check if already running
  if is_running "$port"; then
    local pid=$(get_pid "$port")
    log_info "$service already running on port $port (PID: $pid)"
    return 0
  fi

  # Create logs directory
  mkdir -p "${PROJECT_ROOT}/logs"
  local log_file="${PROJECT_ROOT}/logs/${service}_service.log"

  # Start service
  log_info "Starting $service on port $port..."

  # Append port to command if needed
  if echo "$start_cmd" | grep -q "-- --port$\|--port$"; then
    start_cmd="$start_cmd $port"
  fi

  # Run in background, redirect output to log
  nohup bash -c "$start_cmd" >"$log_file" 2>&1 &
  local bg_pid=$!

  # Wait a moment and check if it started
  sleep 2

  if is_running "$port"; then
    local pid=$(get_pid "$port")
    log_info "✓ $service started successfully on port $port (PID: $pid)"
    log_info "  Logs: $log_file"
    return 0
  else
    log_error "✗ $service failed to start. Check logs: $log_file"
    return 1
  fi
}

# Stop a service
stop_service() {
  local service=$1

  if [ -z "${SERVICES[$service]:-}" ]; then
    log_error "Unknown service: $service"
    return 1
  fi

  local config="${SERVICES[$service]}"
  local default_port=$(echo "$config" | cut -d'|' -f1)
  local port=$(get_port "$service" "$default_port")

  if ! is_running "$port"; then
    log_warn "$service not running on port $port"
    return 0
  fi

  local pid=$(get_pid "$port")
  log_info "Stopping $service (PID: $pid, port: $port)..."

  # Try graceful shutdown first
  kill "$pid" 2>/dev/null || true

  # Wait up to 5 seconds for graceful shutdown
  for i in {1..10}; do
    if ! is_running "$port"; then
      log_info "✓ $service stopped successfully"
      return 0
    fi
    sleep 0.5
  done

  # Force kill if still running
  log_warn "Service did not stop gracefully, forcing..."
  kill -9 "$pid" 2>/dev/null || true
  sleep 1

  if ! is_running "$port"; then
    log_info "✓ $service stopped (forced)"
    return 0
  else
    log_error "✗ Failed to stop $service"
    return 1
  fi
}

# Restart a service
restart_service() {
  local service=$1
  log_info "Restarting $service..."
  stop_service "$service"
  sleep 1
  start_service "$service"
}

# Show status of service(s)
show_status() {
  local service=${1:-}

  echo ""
  echo "Service Status:"
  echo "==============="

  local services_to_check
  if [ -n "$service" ]; then
    services_to_check=("$service")
  else
    services_to_check=("${!SERVICES[@]}")
  fi

  for svc in "${services_to_check[@]}"; do
    if [ -z "${SERVICES[$svc]:-}" ]; then
      log_error "Unknown service: $svc"
      continue
    fi

    local config="${SERVICES[$svc]}"
    local default_port=$(echo "$config" | cut -d'|' -f1)
    local port=$(get_port "$svc" "$default_port")

    printf "  %-20s " "$svc:"
    if is_running "$port"; then
      local pid=$(get_pid "$port")
      echo -e "${GREEN}RUNNING${NC} (PID: $pid, port: $port)"
    else
      echo -e "${RED}STOPPED${NC} (port: $port)"
    fi
  done
  echo ""
}

# Start all services
start_all() {
  log_info "Starting all services..."
  for service in "${!SERVICES[@]}"; do
    start_service "$service" || true
  done
  show_status
}

# Stop all services
stop_all() {
  log_info "Stopping all services..."
  for service in "${!SERVICES[@]}"; do
    stop_service "$service" || true
  done
  show_status
}

# Main
main() {
  local cmd=${1:-}
  local service=${2:-}

  case "$cmd" in
  start)
    if [ -z "$service" ]; then
      log_error "Usage: $0 start <service>"
      exit 1
    fi
    start_service "$service"
    ;;
  stop)
    if [ -z "$service" ]; then
      log_error "Usage: $0 stop <service>"
      exit 1
    fi
    stop_service "$service"
    ;;
  restart)
    if [ -z "$service" ]; then
      log_error "Usage: $0 restart <service>"
      exit 1
    fi
    restart_service "$service"
    ;;
  status)
    show_status "$service"
    ;;
  start-all)
    start_all
    ;;
  stop-all)
    stop_all
    ;;
  list)
    echo "Available services:"
    for svc in "${!SERVICES[@]}"; do
      echo "  - $svc"
    done
    ;;
  *)
    echo "Usage: $0 {start|stop|restart|status|start-all|stop-all|list} [service]"
    echo ""
    echo "Commands:"
    echo "  start <service>    Start a service"
    echo "  stop <service>     Stop a service"
    echo "  restart <service>  Restart a service"
    echo "  status [service]   Show status (all or specific service)"
    echo "  start-all          Start all services"
    echo "  stop-all           Stop all services"
    echo "  list               List available services"
    echo ""
    echo "Services: ${!SERVICES[*]}"
    exit 1
    ;;
  esac
}

main "$@"
