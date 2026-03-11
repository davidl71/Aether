#!/usr/bin/env bash
# Launch all PWA services (web frontend + backend services) as daemonized background processes
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
WEB_DIR="$ROOT_DIR/web"
SCRIPTS_DIR="$WEB_DIR/scripts"

# Colors for output (only if terminal supports it)
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

# Load shared configuration if available
SCRIPTS_DIR_ROOT="${ROOT_DIR}/scripts"
if [ -f "${SCRIPTS_DIR_ROOT}/include/config.sh" ]; then
  # shellcheck source=../../scripts/include/config.sh
  source "${SCRIPTS_DIR_ROOT}/include/config.sh"
fi

service_enabled() {
  local service_name="${1:-}"
  if [ -z "${service_name}" ]; then
    return 1
  fi
  if command -v config_is_enabled >/dev/null 2>&1; then
    config_is_enabled "${service_name}" true
    return $?
  fi
  return 0
}

# Detect OS and service management method
detect_service_manager() {
  # Check for systemctl (Linux with systemd)
  if command -v systemctl >/dev/null 2>&1 && systemctl --user --version >/dev/null 2>&1; then
    # Check if we're on Linux
    if [[ "$OSTYPE" == "linux-gnu"* ]] || [[ "$OSTYPE" == "linux"* ]]; then
      echo "systemctl"
      return 0
    fi
  fi

  # Check for brew services (macOS)
  if [[ "$OSTYPE" == "darwin"* ]] && command -v brew >/dev/null 2>&1; then
    if brew services --version >/dev/null 2>&1; then
      echo "brew"
      return 0
    fi
  fi

  # Fallback to manual background processes
  echo "manual"
  return 0
}

SERVICE_MANAGER=$(detect_service_manager)

# Function to check if a service is running via systemctl
check_systemctl_service() {
  local service_name="${1:-}"
  if [ -z "${service_name}" ]; then
    return 1
  fi

  if [ "${SERVICE_MANAGER}" != "systemctl" ]; then
    return 1
  fi

  systemctl --user is-active --quiet "${service_name}.service" 2>/dev/null
}

# Function to start a service via systemctl
start_systemctl_service() {
  local service_name="${1:-}"
  if [ -z "${service_name}" ]; then
    return 1
  fi

  if [ "${SERVICE_MANAGER}" != "systemctl" ]; then
    return 1
  fi

  # Check if service file exists
  if [ ! -f "${HOME}/.config/systemd/user/${service_name}.service" ]; then
    echo "${YELLOW}  ⚠ Service file not found: ${service_name}.service${NC}"
    echo "${YELLOW}    Install systemd services with:${NC}"
    echo "${YELLOW}    ${ROOT_DIR}/web/scripts/install-systemd-services.sh${NC}"
    return 1
  fi

  if systemctl --user start "${service_name}.service" 2>/dev/null; then
    echo "${GREEN}  ✓ Started ${service_name} via systemctl${NC}"
    return 0
  else
    echo "${RED}  ✗ Failed to start ${service_name} via systemctl${NC}"
    systemctl --user status "${service_name}.service" --no-pager -l 2>&1 | head -10 || true
    return 1
  fi
}

# Function to stop a service via systemctl
stop_systemctl_service() {
  local service_name="${1:-}"
  if [ -z "${service_name}" ]; then
    return 1
  fi

  if [ "${SERVICE_MANAGER}" != "systemctl" ]; then
    return 1
  fi

  if systemctl --user stop "${service_name}.service" 2>/dev/null; then
    echo "${GREEN}  ✓ Stopped ${service_name} via systemctl${NC}"
    return 0
  else
    return 1
  fi
}

# Function to check if a port is in use
check_port() {
  local port="${1:-}"
  if command -v lsof >/dev/null 2>&1; then
    lsof -ti ":${port}" >/dev/null 2>&1
  elif command -v netstat >/dev/null 2>&1; then
    netstat -an 2>/dev/null | grep -q ":${port}.*LISTEN"
  else
    return 1
  fi
}

# Function to check service health via HTTP endpoint
# Usage: check_service_health <port> [path] [scheme]
# Returns 0 if healthy, 1 if not
check_service_health() {
  local port="${1:-}"
  local health_path="${2:-/api/health}"
  local scheme="${3:-http}"
  local host="127.0.0.1"

  if [ -z "${port}" ]; then
    return 1
  fi

  # Special handling for IB Gateway (HTTPS, different endpoint)
  if [ "${port}" = "5001" ]; then
    if command -v curl >/dev/null 2>&1; then
      curl -k -s --connect-timeout 2 "https://localhost:${port}/sso/validate" >/dev/null 2>&1
      return $?
    fi
    return 1
  fi

  # Standard HTTP health check
  if command -v curl >/dev/null 2>&1; then
    curl -s --connect-timeout 2 "${scheme}://${host}:${port}${health_path}" >/dev/null 2>&1
    return $?
  elif command -v python3 >/dev/null 2>&1; then
    python3 -c "
import urllib.request
import socket
try:
    with urllib.request.urlopen('${scheme}://${host}:${port}${health_path}', timeout=2) as response:
        exit(0 if response.status == 200 else 1)
except:
    exit(1)
" 2>/dev/null
    return $?
  fi

  # Fallback: just check if port is listening
  check_port "${port}"
}

# Function to start NATS server
# Returns 0 if started successfully, 1 if failed
start_nats_server() {
  local mode="${1:-background}"  # "background" only (tmux removed)

  # Check if NATS is installed
  if ! command -v nats-server >/dev/null 2>&1; then
    echo "${YELLOW}  ⚠ NATS server not found. Install with: ./scripts/install_nats.sh${NC}"
    return 1
  fi

  # Check if already running
  if pgrep -f "nats-server" >/dev/null; then
    echo "${GREEN}  ✓ NATS server already running (PID: $(pgrep -f 'nats-server'))${NC}"
    return 0
  fi

  # Create logs directory if needed
  mkdir -p "${ROOT_DIR}/logs"

  # Configuration file
  CONFIG_FILE="${ROOT_DIR}/config/nats-server.conf"
  LOG_FILE="${ROOT_DIR}/logs/nats-server.log"

  # Start in background
  echo "${BLUE}  Starting NATS server in background...${NC}"
  if [ -f "${CONFIG_FILE}" ]; then
    nats-server -c "${CONFIG_FILE}" > "${LOG_FILE}" 2>&1 &
  else
    nats-server > "${LOG_FILE}" 2>&1 &
  fi
  NATS_PID=$!
  disown $NATS_PID 2>/dev/null || true
  echo $NATS_PID > /tmp/pwa-nats.pid

  # Wait a moment for server to start
  sleep 2

  # Verify it started
  if kill -0 "$NATS_PID" 2>/dev/null && pgrep -f "nats-server" >/dev/null; then
    echo "${GREEN}  ✓ NATS server started (PID: $NATS_PID)${NC}"
    echo "${BLUE}    Server URL: nats://localhost:${NATS_PORT}${NC}"
    echo "${BLUE}    Monitoring: http://localhost:${NATS_HTTP_PORT}${NC}"
    echo "${BLUE}    Log: ${LOG_FILE}${NC}"
    return 0
  else
    echo "${RED}  ✗ Failed to start NATS server${NC}"
    echo "${YELLOW}    Check log: ${LOG_FILE}${NC}"
    tail -20 "${LOG_FILE}" 2>/dev/null || true
    return 1
  fi
}

# Function to open URL in browser (cross-platform)
open_browser() {
  local url="${1:-}"
  if [ -z "${url}" ]; then
    return 1
  fi

  # Only open if running in interactive terminal
  if [ ! -t 1 ]; then
    return 0
  fi

  # macOS
  if [[ "$OSTYPE" == "darwin"* ]]; then
    open "${url}" 2>/dev/null || true
    return 0
  fi

  # Linux
  if command -v xdg-open >/dev/null 2>&1; then
    xdg-open "${url}" 2>/dev/null || true
    return 0
  fi

  # Fallback: try common browsers
  if command -v firefox >/dev/null 2>&1; then
    firefox "${url}" 2>/dev/null || true
    return 0
  fi

  if command -v google-chrome >/dev/null 2>&1; then
    google-chrome "${url}" 2>/dev/null || true
    return 0
  fi

  return 1
}

# Function to get service port from config or default
get_service_port() {
  local service_name="${1:-}"
  local default_port="${2:-}"

  if [ -n "${service_name}" ] && command -v config_get_port >/dev/null 2>&1; then
    config_get_port "${service_name}" "${default_port}"
  else
    echo "${default_port}"
  fi
}

# Function to stop all services
stop_services() {
  echo "${YELLOW}Stopping PWA services...${NC}"

  # Stop via systemctl if available
  if [ "${SERVICE_MANAGER}" = "systemctl" ]; then
    echo "${BLUE}Stopping services via systemctl...${NC}"
    SERVICES_TO_STOP=(
      "pwa-web"
      "pwa-alpaca"
      "pwa-ib"
      "pwa-discount-bank"
      "pwa-risk-free-rate"
      "pwa-nats"
      "pwa-rust-backend"
      "pwa-ib-gateway"
    )

    for service in "${SERVICES_TO_STOP[@]}"; do
      if check_systemctl_service "${service}"; then
        stop_systemctl_service "${service}" || true
      fi
    done

    echo "${GREEN}✓ All systemctl services stopped${NC}"
    exit 0
  fi

  # Check if using brew services for gateway
  if [ "${USE_BREW_SERVICES:-false}" = true ] || [ "${IB_GATEWAY_USE_BREW_SERVICES:-}" = "1" ]; then
    PLIST_NAME="com.davidl71.ib-gateway"
    if brew services list 2>/dev/null | grep -q "${PLIST_NAME}.*started"; then
      echo "${BLUE}Stopping IB Gateway (brew services)...${NC}"
      brew services stop "${PLIST_NAME}" 2>/dev/null || true
      echo "${GREEN}✓ IB Gateway stopped${NC}"
    fi
  fi

  # Kill processes on service ports (including NATS and Rust backend)
  for port in 5173 5001 8000 8001 8002 8003 8004 8080 50051 4222 8222; do
    if check_port "$port"; then
      echo "${BLUE}Stopping service on port ${port}...${NC}"
      if command -v lsof >/dev/null 2>&1; then
        lsof -ti ":${port}" | xargs kill -9 2>/dev/null || true
      fi
    fi
  done

  # Also stop Rust backend via script if available
  if [ -f "${ROOT_DIR}/scripts/stop_rust_backend.sh" ]; then
    echo "${BLUE}Stopping Rust backend...${NC}"
    bash "${ROOT_DIR}/scripts/stop_rust_backend.sh" >/dev/null 2>&1 || true
  fi

  echo "${GREEN}✓ All services stopped${NC}"
  exit 0
}

# Check for brew services mode
USE_BREW_SERVICES=false
if [ "${IB_GATEWAY_USE_BREW_SERVICES:-}" = "1" ] || [ "${1:-}" = "--brew-services" ]; then
  USE_BREW_SERVICES=true
  # Remove --brew-services from args if present
  if [ "${1:-}" = "--brew-services" ]; then
    shift
  fi
fi

# Parse command line arguments
case "${1:-start}" in
  stop)
    stop_services
    ;;
  restart)
    stop_services
    sleep 2
    # Fall through to start
    ;;
  status)
    echo "${BLUE}PWA Services Status:${NC}"
    echo ""

    # Show systemctl status if available
    if [ "${SERVICE_MANAGER}" = "systemctl" ]; then
      echo "Systemd Service Status:"
      SYSTEMCTL_SERVICES=(
        "pwa-web:Web"
        "pwa-alpaca:Alpaca"
        "pwa-ib-gateway:IB Gateway"
        "pwa-ib:IB"
        "pwa-discount-bank:Discount Bank"
        "pwa-risk-free-rate:Risk-Free Rate"
        "pwa-nats:NATS"
        "pwa-rust-backend:Rust Backend"
      )

      for service_entry in "${SYSTEMCTL_SERVICES[@]}"; do
        IFS=':' read -r service_name display_name <<< "$service_entry"
        if check_systemctl_service "${service_name}"; then
          STATUS=$(systemctl --user is-active "${service_name}.service" 2>/dev/null || echo "unknown")
          echo "  ${GREEN}✓${NC} ${display_name} (${STATUS})"
        else
          echo "  ${RED}✗${NC} ${display_name} (not running)"
        fi
      done
      echo ""
      echo "For detailed status: ${BLUE}systemctl --user status <service-name>${NC}"
      echo "For logs: ${BLUE}journalctl --user -u <service-name> -f${NC}"
      echo ""
    fi

    echo "Port Status:"
    for port in 5173 5001 8000 8001 8002 8003 8004 8080 50051 4222 8222; do
      if check_port "$port"; then
        echo "  ${GREEN}✓ Port ${port}: In use${NC}"
      else
        echo "  ${RED}✗ Port ${port}: Available${NC}"
      fi
    done
    echo ""
    echo "Background Process Status:"
    for pidfile in /tmp/pwa-*.pid; do
      if [ -f "$pidfile" ]; then
        PID=$(cat "$pidfile" 2>/dev/null || echo "")
        SERVICE_NAME=$(basename "$pidfile" .pid | sed 's/pwa-//')
        if [ -n "$PID" ] && kill -0 "$PID" 2>/dev/null; then
          echo "  ${GREEN}✓${NC} ${SERVICE_NAME} (PID: ${PID})"
        else
          echo "  ${RED}✗${NC} ${SERVICE_NAME} (not running)"
        fi
      fi
    done

    # Check NATS
    if pgrep -f "nats-server" >/dev/null; then
      echo "  ${GREEN}✓${NC} NATS server (PID: $(pgrep -f 'nats-server'))"
    else
      echo "  ${RED}✗${NC} NATS server (not running)"
    fi

    exit 0
    ;;
  start|*)
    # Continue with start logic
    ;;
esac

# Get service ports from config or defaults
WEB_PORT=$(get_service_port "web" 5173)
GATEWAY_PORT=$(get_service_port "ib_gateway" 5001)
ALPACA_PORT=$(get_service_port "alpaca" 8000)
IB_PORT=$(get_service_port "ib" 8002)
DISCOUNT_BANK_PORT=$(get_service_port "discount_bank" 8003)
RISK_FREE_RATE_PORT=$(get_service_port "risk_free_rate" 8004)
# NATS ports - use config_get_port for main port, config_get for nested ports
NATS_PORT=$(get_service_port "nats" 4222)
if command -v config_get >/dev/null 2>&1; then
  NATS_HTTP_PORT=$(config_get ".services.nats.http_port" 8222)
  NATS_WEBSOCKET_PORT=$(config_get ".services.nats.websocket_port" 8081)
else
  NATS_HTTP_PORT=8222
  NATS_WEBSOCKET_PORT=8081
fi

# Rust backend ports
if command -v config_get >/dev/null 2>&1; then
  RUST_BACKEND_REST_PORT=$(config_get ".services.rust_backend.rest_port" 8080)
  RUST_BACKEND_GRPC_PORT=$(config_get ".services.rust_backend.grpc_port" 50051)
else
  RUST_BACKEND_REST_PORT=8080
  RUST_BACKEND_GRPC_PORT=50051
fi

# Check service health and track which are running
declare -A SERVICE_STATUS
SERVICES_TO_START=()

# Display service manager being used
if [ "${SERVICE_MANAGER}" = "systemctl" ]; then
  echo "${BLUE}Using systemctl for service management (Linux)${NC}"
  # Check if services are installed
  if [ ! -f "${HOME}/.config/systemd/user/pwa-web.service" ]; then
    echo "${YELLOW}⚠ Systemd services not installed.${NC}"
    echo "${YELLOW}  Install with: ${ROOT_DIR}/web/scripts/install-systemd-services.sh${NC}"
    echo "${YELLOW}  Or services will fall back to manual background processes.${NC}"
    echo ""
  fi
elif [ "${SERVICE_MANAGER}" = "brew" ]; then
  echo "${BLUE}Using brew services for service management (macOS)${NC}"
else
  echo "${BLUE}Using manual background processes for service management${NC}"
fi
echo ""

echo "${BLUE}Checking PWA Services Status...${NC}"
echo ""

# Check each service
# Use systemctl check first if available, then fallback to port check
if [ "${SERVICE_MANAGER}" = "systemctl" ] && check_systemctl_service "pwa-web"; then
  SERVICE_STATUS["web"]="running"
  echo "${GREEN}✓ Web service (port ${WEB_PORT}) is running via systemctl${NC}"
elif check_service_health "${WEB_PORT}"; then
  SERVICE_STATUS["web"]="running"
  echo "${GREEN}✓ Web service (port ${WEB_PORT}) is running${NC}"
else
  SERVICE_STATUS["web"]="stopped"
  SERVICES_TO_START+=("web")
  echo "${YELLOW}⚠ Web service (port ${WEB_PORT}) is not running${NC}"
fi

# Check IB Gateway
if [ "${SERVICE_MANAGER}" = "systemctl" ] && check_systemctl_service "pwa-ib-gateway"; then
  SERVICE_STATUS["gateway"]="running"
  echo "${GREEN}✓ IB Gateway (port ${GATEWAY_PORT}) is running via systemctl${NC}"
elif [ "$USE_BREW_SERVICES" = true ]; then
  PLIST_NAME="com.davidl71.ib-gateway"
  if brew services list 2>/dev/null | grep -q "${PLIST_NAME}.*started"; then
    SERVICE_STATUS["gateway"]="running"
    echo "${GREEN}✓ IB Gateway (port ${GATEWAY_PORT}) is running via brew services${NC}"
  else
    SERVICE_STATUS["gateway"]="stopped"
    SERVICES_TO_START+=("gateway")
    echo "${YELLOW}⚠ IB Gateway (port ${GATEWAY_PORT}) is not running (brew services)${NC}"
  fi
elif check_service_health "${GATEWAY_PORT}"; then
  SERVICE_STATUS["gateway"]="running"
  echo "${GREEN}✓ IB Gateway (port ${GATEWAY_PORT}) is running${NC}"
else
  SERVICE_STATUS["gateway"]="stopped"
  SERVICES_TO_START+=("gateway")
  echo "${YELLOW}⚠ IB Gateway (port ${GATEWAY_PORT}) is not running${NC}"
fi

if ! service_enabled "alpaca"; then
  SERVICE_STATUS["alpaca"]="disabled"
  echo "${BLUE}○ Alpaca service is disabled in config${NC}"
elif [ "${SERVICE_MANAGER}" = "systemctl" ] && check_systemctl_service "pwa-alpaca"; then
  SERVICE_STATUS["alpaca"]="running"
  echo "${GREEN}✓ Alpaca service (port ${ALPACA_PORT}) is running via systemctl${NC}"
elif check_service_health "${ALPACA_PORT}"; then
  SERVICE_STATUS["alpaca"]="running"
  echo "${GREEN}✓ Alpaca service (port ${ALPACA_PORT}) is running${NC}"
else
  SERVICE_STATUS["alpaca"]="stopped"
  SERVICES_TO_START+=("alpaca")
  echo "${YELLOW}⚠ Alpaca service (port ${ALPACA_PORT}) is not running${NC}"
fi

if [ "${SERVICE_MANAGER}" = "systemctl" ] && check_systemctl_service "pwa-ib"; then
  SERVICE_STATUS["ib"]="running"
  echo "${GREEN}✓ IB service (port ${IB_PORT}) is running via systemctl${NC}"
elif check_service_health "${IB_PORT}"; then
  SERVICE_STATUS["ib"]="running"
  echo "${GREEN}✓ IB service (port ${IB_PORT}) is running${NC}"
else
  SERVICE_STATUS["ib"]="stopped"
  SERVICES_TO_START+=("ib")
  echo "${YELLOW}⚠ IB service (port ${IB_PORT}) is not running${NC}"
fi

if [ "${SERVICE_MANAGER}" = "systemctl" ] && check_systemctl_service "pwa-discount-bank"; then
  SERVICE_STATUS["discount_bank"]="running"
  echo "${GREEN}✓ Discount Bank service (port ${DISCOUNT_BANK_PORT}) is running via systemctl${NC}"
elif check_service_health "${DISCOUNT_BANK_PORT}"; then
  SERVICE_STATUS["discount_bank"]="running"
  echo "${GREEN}✓ Discount Bank service (port ${DISCOUNT_BANK_PORT}) is running${NC}"
else
  SERVICE_STATUS["discount_bank"]="stopped"
  SERVICES_TO_START+=("discount_bank")
  echo "${YELLOW}⚠ Discount Bank service (port ${DISCOUNT_BANK_PORT}) is not running${NC}"
fi

if [ "${SERVICE_MANAGER}" = "systemctl" ] && check_systemctl_service "pwa-risk-free-rate"; then
  SERVICE_STATUS["risk_free_rate"]="running"
  echo "${GREEN}✓ Risk-Free Rate service (port ${RISK_FREE_RATE_PORT}) is running via systemctl${NC}"
elif check_service_health "${RISK_FREE_RATE_PORT}"; then
  SERVICE_STATUS["risk_free_rate"]="running"
  echo "${GREEN}✓ Risk-Free Rate service (port ${RISK_FREE_RATE_PORT}) is running${NC}"
else
  SERVICE_STATUS["risk_free_rate"]="stopped"
  SERVICES_TO_START+=("risk_free_rate")
  echo "${YELLOW}⚠ Risk-Free Rate service (port ${RISK_FREE_RATE_PORT}) is not running${NC}"
fi

# Check NATS server (check monitoring port 8222)
if [ "${SERVICE_MANAGER}" = "systemctl" ] && check_systemctl_service "pwa-nats"; then
  SERVICE_STATUS["nats"]="running"
  echo "${GREEN}✓ NATS server (port ${NATS_PORT}) is running via systemctl${NC}"
elif check_port "${NATS_PORT}" || check_service_health "${NATS_HTTP_PORT}" "/healthz" "http"; then
  SERVICE_STATUS["nats"]="running"
  echo "${GREEN}✓ NATS server (port ${NATS_PORT}) is running${NC}"
else
  SERVICE_STATUS["nats"]="stopped"
  SERVICES_TO_START+=("nats")
  echo "${YELLOW}⚠ NATS server (port ${NATS_PORT}) is not running${NC}"
fi

# Check Rust backend (check REST port 8080)
if [ "${SERVICE_MANAGER}" = "systemctl" ] && check_systemctl_service "pwa-rust-backend"; then
  SERVICE_STATUS["rust_backend"]="running"
  echo "${GREEN}✓ Rust backend (port ${RUST_BACKEND_REST_PORT}) is running via systemctl${NC}"
elif check_service_health "${RUST_BACKEND_REST_PORT}" "/health" "http"; then
  SERVICE_STATUS["rust_backend"]="running"
  echo "${GREEN}✓ Rust backend (port ${RUST_BACKEND_REST_PORT}) is running${NC}"
else
  SERVICE_STATUS["rust_backend"]="stopped"
  SERVICES_TO_START+=("rust_backend")
  echo "${YELLOW}⚠ Rust backend (port ${RUST_BACKEND_REST_PORT}) is not running${NC}"
fi

echo ""

# If all services are running, exit
if [ ${#SERVICES_TO_START[@]} -eq 0 ]; then
  echo "${GREEN}✓ All services are running${NC}"
  echo "Use ${BLUE}$0 status${NC} to check service status"
  exit 0
fi

# If we get here, we need to start some services
if [ ${#SERVICES_TO_START[@]} -gt 0 ]; then
  echo "${BLUE}Starting ${#SERVICES_TO_START[@]} service(s): ${SERVICES_TO_START[*]}${NC}"
  echo ""
fi

# Launch services in background (daemonized) or via systemctl
if [ "${SERVICE_MANAGER}" = "systemctl" ]; then
  echo "${BLUE}Starting services via systemctl...${NC}"
  echo ""

  # Map service names to systemctl service names
  declare -A SYSTEMCTL_SERVICE_MAP
  SYSTEMCTL_SERVICE_MAP["web"]="pwa-web"
  SYSTEMCTL_SERVICE_MAP["alpaca"]="pwa-alpaca"
  SYSTEMCTL_SERVICE_MAP["gateway"]="pwa-ib-gateway"
  SYSTEMCTL_SERVICE_MAP["ib"]="pwa-ib"
  SYSTEMCTL_SERVICE_MAP["discount_bank"]="pwa-discount-bank"
  SYSTEMCTL_SERVICE_MAP["risk_free_rate"]="pwa-risk-free-rate"
  SYSTEMCTL_SERVICE_MAP["nats"]="pwa-nats"
  SYSTEMCTL_SERVICE_MAP["rust_backend"]="pwa-rust-backend"

  # Start services in dependency order
  # 1. NATS first (other services may depend on it)
  if [[ " ${SERVICES_TO_START[*]} " =~ " nats " ]]; then
    start_systemctl_service "${SYSTEMCTL_SERVICE_MAP[nats]}" || start_nats_server "background"
  fi

  # 2. Gateway (IB service depends on it)
  if [[ " ${SERVICES_TO_START[*]} " =~ " gateway " ]]; then
    start_systemctl_service "${SYSTEMCTL_SERVICE_MAP[gateway]}" || {
      # Fallback to manual start if systemctl fails
      echo "${YELLOW}  Falling back to manual gateway start...${NC}"
      GATEWAY_STARTED=true
    }
  fi

  # 3. Independent services (can start in parallel)
  for service in web alpaca discount_bank risk_free_rate rust_backend; do
    if [ "${service}" = "alpaca" ] && ! service_enabled "alpaca"; then
      continue
    fi
    if [[ " ${SERVICES_TO_START[*]} " =~ " ${service} " ]]; then
      start_systemctl_service "${SYSTEMCTL_SERVICE_MAP[$service]}" || {
        echo "${YELLOW}  Falling back to manual start for ${service}...${NC}"
        # Will be handled by manual start section below
      }
    fi
  done

  # 4. IB service (depends on gateway)
  if [[ " ${SERVICES_TO_START[*]} " =~ " ib " ]]; then
    # Wait for gateway if we just started it
    if [ "${GATEWAY_STARTED:-false}" = true ]; then
      echo "${BLUE}Waiting for Gateway to be ready before starting IB service...${NC}"
      for i in {1..20}; do
        sleep 1
        if curl -k -s --connect-timeout 1 "https://localhost:${GATEWAY_PORT}/sso/validate" >/dev/null 2>&1; then
          echo "${GREEN}✓ Gateway is ready${NC}"
          GATEWAY_RUNNING=true
          break
        fi
      done
    fi
    start_systemctl_service "${SYSTEMCTL_SERVICE_MAP[ib]}" || {
      echo "${YELLOW}  Falling back to manual start for IB service...${NC}"
    }
  fi

  echo "${GREEN}✓ Services started via systemctl${NC}"
  echo ""
  echo "To view logs: ${BLUE}journalctl --user -u <service-name> -f${NC}"
  echo "To check status: ${BLUE}systemctl --user status <service-name>${NC}"
  echo ""
  exit 0
fi

# Fallback to manual background processes
echo "${BLUE}Launching services as background processes...${NC}"
echo ""

# Start NATS server first (other services may depend on it)
if [[ " ${SERVICES_TO_START[*]} " =~ " nats " ]]; then
  start_nats_server "background"
fi

# Launch services in background (parallel groups for faster startup)
# Group 1 (parallel): Web, Gateway, optional Alpaca, Discount Bank, Risk-Free Rate
# Group 2 (after Gateway ready): IB service

echo "${BLUE}Starting independent services in parallel...${NC}"

# Initialize gateway tracking
GATEWAY_DIR="${ROOT_DIR}/ib-gateway"
GATEWAY_RUNNING=false
GATEWAY_STARTED=false

# Check if gateway is already running
if [ "${SERVICE_STATUS[gateway]}" = "running" ]; then
  GATEWAY_RUNNING=true
  echo "${GREEN}✓ IB Gateway already running${NC}"
fi

# Group 1: Start all independent services simultaneously
# Web service (check if port is already in use to prevent duplicates)
if [[ " ${SERVICES_TO_START[*]} " =~ " web " ]]; then
  if check_port "${WEB_PORT}"; then
    echo "${YELLOW}  Web service port ${WEB_PORT} is already in use. Skipping start.${NC}"
    echo "${YELLOW}  If you want to restart, stop the existing service first.${NC}"
  else
    (
      cd "$WEB_DIR"
      bash "${SCRIPTS_DIR}/run-web-service.sh" > /tmp/pwa-web.log 2>&1 &
      echo $! > /tmp/pwa-web.pid
    ) &
  fi
fi

# IB Gateway (IB service will wait for it)
if [[ " ${SERVICES_TO_START[*]} " =~ " gateway " ]]; then
  GATEWAY_DIR="${ROOT_DIR}/ib-gateway"
  GATEWAY_RUNNING=false
  GATEWAY_STARTED=false

  if [ "${USE_BREW_SERVICES:-false}" = true ]; then
    PLIST_NAME="com.davidl71.ib-gateway"
    if ! brew services list 2>/dev/null | grep -q "${PLIST_NAME}.*started"; then
      if [ ! -f "${HOME}/Library/LaunchAgents/${PLIST_NAME}.plist" ]; then
        echo "${BLUE}Installing IB Gateway brew service...${NC}"
        if [ -f "${GATEWAY_DIR}/install-brew-service.sh" ]; then
          bash "${GATEWAY_DIR}/install-brew-service.sh"
        fi
      fi
      echo "${BLUE}Starting IB Gateway via brew services...${NC}"
      brew services start "${PLIST_NAME}" 2>&1 || {
        echo "${RED}[ERROR] Failed to start brew service${NC}" >&2
      }
    else
      echo "${GREEN}✓ IB Gateway already running via brew services${NC}"
      GATEWAY_RUNNING=true
    fi
  elif [ -f "${GATEWAY_DIR}/run-gateway-with-reload.sh" ] || [ -f "${GATEWAY_DIR}/run-gateway.sh" ] || [ -f "${GATEWAY_DIR}/bin/run.sh" ]; then
    if ! curl -k -s --connect-timeout 1 "https://localhost:${GATEWAY_PORT}/sso/validate" >/dev/null 2>&1; then
      echo "${BLUE}Starting IB Gateway in background...${NC}"
      (
        cd "$GATEWAY_DIR"
        # Force port so gateway starts on the port we expect (avoids env IB_GATEWAY_PORT=5000 from shell)
        export IB_GATEWAY_PORT="${GATEWAY_PORT}"
        # Prefer auto-reload wrapper if available
        if [ -f "run-gateway-with-reload.sh" ]; then
          bash run-gateway-with-reload.sh > /tmp/pwa-ib-gateway.log 2>&1 &
        elif [ -f "run-gateway.sh" ]; then
          bash run-gateway.sh > /tmp/pwa-ib-gateway.log 2>&1 &
        else
          # Normalize config file path before passing to run.sh
          CONFIG_FILE="${GATEWAY_DIR}/root/conf.yaml"
          if [ ! -f "${CONFIG_FILE}" ]; then
            CONFIG_FILE="${GATEWAY_DIR}/root/conf.tws.yaml"
          fi
          if [ -f "${CONFIG_FILE}" ]; then
            if command -v realpath >/dev/null 2>&1; then
              CONFIG_FILE="$(realpath "${CONFIG_FILE}")"
            else
              CONFIG_FILE="$(cd "$(dirname "${CONFIG_FILE}")" && pwd)/$(basename "${CONFIG_FILE}")"
            fi
            bash bin/run.sh "${CONFIG_FILE}" > /tmp/pwa-ib-gateway.log 2>&1 &
          else
            echo "[ERROR] No config file found" > /tmp/pwa-ib-gateway.log 2>&1
            echo "[PATH] Searched: ${GATEWAY_DIR}/root/conf.yaml" >> /tmp/pwa-ib-gateway.log 2>&1
            echo "[PATH] Searched: ${GATEWAY_DIR}/root/conf.tws.yaml" >> /tmp/pwa-ib-gateway.log 2>&1
            exit 1
          fi
        fi
        echo $! > /tmp/pwa-ib-gateway.pid
        GATEWAY_STARTED=true
      ) &
    else
      echo "${GREEN}✓ IB Gateway already running${NC}"
      GATEWAY_RUNNING=true
    fi
  fi

  # Verify gateway is listening on configured port after starting (background mode)
  # Only verify if we actually started the gateway in this run
  if [ "${GATEWAY_STARTED:-false}" = true ]; then
    if [ -f "${GATEWAY_DIR}/run-gateway-with-reload.sh" ] || [ -f "${GATEWAY_DIR}/run-gateway.sh" ] || [ -f "${GATEWAY_DIR}/bin/run.sh" ]; then
      echo "${BLUE}Verifying Gateway is listening on port ${GATEWAY_PORT}...${NC}"
      GATEWAY_VERIFIED=false
      for i in {1..30}; do
        sleep 1
        if check_port "${GATEWAY_PORT}"; then
          # Port is listening, verify it's actually the gateway
          if check_service_health "${GATEWAY_PORT}" "/sso/validate" "https"; then
            echo "${GREEN}✓ Gateway verified and listening on port ${GATEWAY_PORT}${NC}"
            GATEWAY_VERIFIED=true
            GATEWAY_RUNNING=true
            # Open gateway URL in browser
            echo "${BLUE}Opening gateway in browser...${NC}"
            open_browser "https://localhost:${GATEWAY_PORT}"
            break
          fi
        fi
        # Show progress every 5 seconds
        if [ $((i % 5)) -eq 0 ]; then
          echo "${BLUE}  Still waiting for gateway to start... (${i}/30 seconds)${NC}"
        fi
      done

      if [ "$GATEWAY_VERIFIED" = false ]; then
        echo "${YELLOW}  ⚠ Gateway may not be fully ready on port ${GATEWAY_PORT}${NC}"
        echo "${YELLOW}  Check log: /tmp/pwa-ib-gateway.log${NC}"
      fi
    fi
  fi
fi

# Alpaca service (independent, port 8000)
if service_enabled "alpaca" && [[ " ${SERVICES_TO_START[*]} " =~ " alpaca " ]]; then
  (
    cd "$ROOT_DIR"
    bash "${SCRIPTS_DIR}/run-alpaca-service.sh" > /tmp/pwa-alpaca.log 2>&1 &
    echo $! > /tmp/pwa-alpaca.pid
  ) &
fi

# Discount Bank service (independent, port 8003)
if [[ " ${SERVICES_TO_START[*]} " =~ " discount_bank " ]]; then
  (
    cd "$ROOT_DIR"
    bash "${SCRIPTS_DIR}/run-discount-bank-service.sh" > /tmp/pwa-discount-bank.log 2>&1 &
    echo $! > /tmp/pwa-discount-bank.pid
  ) &
fi

# Risk-Free Rate service (port 8004)
if [[ " ${SERVICES_TO_START[*]} " =~ " risk_free_rate " ]]; then
  (
    cd "$ROOT_DIR"
    bash "${SCRIPTS_DIR}/run-risk-free-rate-service.sh" > /tmp/pwa-risk-free-rate.log 2>&1 &
    echo $! > /tmp/pwa-risk-free-rate.pid
  ) &
fi

# Rust backend service (port 8080 for REST, 50051 for gRPC)
if [[ " ${SERVICES_TO_START[*]} " =~ " rust_backend " ]]; then
  RUST_BACKEND_SCRIPT="${ROOT_DIR}/scripts/start_rust_backend.sh"
  if [ -f "${RUST_BACKEND_SCRIPT}" ]; then
    (
      cd "$ROOT_DIR"
      bash "${RUST_BACKEND_SCRIPT}" > /tmp/pwa-rust-backend.log 2>&1 &
      echo $! > /tmp/pwa-rust-backend.pid
    ) &
  else
    echo "${YELLOW}⚠ Rust backend startup script not found at ${RUST_BACKEND_SCRIPT}${NC}"
  fi
fi

# Group 2: Wait for Gateway, then start IB service
if [[ " ${SERVICES_TO_START[*]} " =~ " ib " ]]; then
  if [ "$GATEWAY_RUNNING" = false ]; then
    echo "${BLUE}Waiting for Gateway to be ready before starting IB service...${NC}"
    for i in {1..20}; do
      sleep 1
      if curl -k -s --connect-timeout 1 "https://localhost:${GATEWAY_PORT}/sso/validate" >/dev/null 2>&1; then
        echo "${GREEN}✓ Gateway is ready${NC}"
        GATEWAY_RUNNING=true
        break
      fi
    done
  fi

  # IB service (depends on Gateway, uses port 8002)
  (
    cd "$ROOT_DIR"
    bash "${SCRIPTS_DIR}/run-ib-service.sh" > /tmp/pwa-ib.log 2>&1 &
    echo $! > /tmp/pwa-ib.pid
  ) &
fi

wait

echo "${GREEN}✓ All services launched in background${NC}"
echo ""
echo "Services:"
if [[ " ${SERVICES_TO_START[*]} " =~ " web " ]] || [ "${SERVICE_STATUS[web]}" = "running" ]; then
  echo "  ${GREEN}✓${NC} Web service (Vite) - Log: /tmp/pwa-web.log"
  echo "    ${BLUE}URL: http://127.0.0.1:${WEB_PORT}${NC}"
fi
if [[ " ${SERVICES_TO_START[*]} " =~ " gateway " ]] || [ "${SERVICE_STATUS[gateway]}" = "running" ]; then
  if [ -f "${GATEWAY_DIR}/run-gateway-with-reload.sh" ] || [ -f "${GATEWAY_DIR}/run-gateway.sh" ] || [ -f "${GATEWAY_DIR}/bin/run.sh" ]; then
    echo "  ${GREEN}✓${NC} IB Gateway (port ${GATEWAY_PORT}) - Log: /tmp/pwa-ib-gateway.log"
    echo "    ${BLUE}URL: https://localhost:${GATEWAY_PORT}${NC}"
  fi
fi
if service_enabled "alpaca" && { [[ " ${SERVICES_TO_START[*]} " =~ " alpaca " ]] || [ "${SERVICE_STATUS[alpaca]}" = "running" ]; }; then
  echo "  ${GREEN}✓${NC} Alpaca service (port ${ALPACA_PORT}) - Log: /tmp/pwa-alpaca.log"
  echo "    ${BLUE}URL: http://127.0.0.1:${ALPACA_PORT}${NC}"
fi
if [[ " ${SERVICES_TO_START[*]} " =~ " ib " ]] || [ "${SERVICE_STATUS[ib]}" = "running" ]; then
  echo "  ${GREEN}✓${NC} IB service (port ${IB_PORT}) - Log: /tmp/pwa-ib.log"
  echo "    ${BLUE}URL: http://127.0.0.1:${IB_PORT}${NC}"
fi
if [[ " ${SERVICES_TO_START[*]} " =~ " discount_bank " ]] || [ "${SERVICE_STATUS[discount_bank]}" = "running" ]; then
  echo "  ${GREEN}✓${NC} Discount Bank service (port ${DISCOUNT_BANK_PORT}) - Log: /tmp/pwa-discount-bank.log"
  echo "    ${BLUE}URL: http://127.0.0.1:${DISCOUNT_BANK_PORT}${NC}"
fi
if [[ " ${SERVICES_TO_START[*]} " =~ " risk_free_rate " ]] || [ "${SERVICE_STATUS[risk_free_rate]}" = "running" ]; then
  echo "  ${GREEN}✓${NC} Risk-Free Rate service (port ${RISK_FREE_RATE_PORT}) - Log: /tmp/pwa-risk-free-rate.log"
  echo "    ${BLUE}URL: http://127.0.0.1:${RISK_FREE_RATE_PORT}${NC}"
fi
if [[ " ${SERVICES_TO_START[*]} " =~ " nats " ]] || [ "${SERVICE_STATUS[nats]}" = "running" ]; then
  echo "  ${GREEN}✓${NC} NATS server (port ${NATS_PORT}) - Log: ${ROOT_DIR}/logs/nats-server.log"
  echo "    ${BLUE}URL: nats://localhost:${NATS_PORT}${NC}"
  echo "    ${BLUE}Monitoring: http://localhost:${NATS_HTTP_PORT}${NC}"
fi
if [[ " ${SERVICES_TO_START[*]} " =~ " rust_backend " ]] || [ "${SERVICE_STATUS[rust_backend]}" = "running" ]; then
  echo "  ${GREEN}✓${NC} Rust backend (REST: ${RUST_BACKEND_REST_PORT}, gRPC: ${RUST_BACKEND_GRPC_PORT}) - Log: /tmp/pwa-rust-backend.log"
  echo "    ${BLUE}REST API: http://localhost:${RUST_BACKEND_REST_PORT}/api/v1/snapshot${NC}"
  echo "    ${BLUE}gRPC API: localhost:${RUST_BACKEND_GRPC_PORT}${NC}"
fi
echo ""
echo "Commands:"
echo "  ${BLUE}tail -f /tmp/pwa-*.log${NC}  # View all logs"
echo "  ${BLUE}$0 status${NC}                # Check service status"
echo "  ${BLUE}$0 stop${NC}                  # Stop all services"
echo ""
