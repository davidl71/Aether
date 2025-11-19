#!/usr/bin/env bash
# Launch all PWA services (web frontend + backend services) using tmux
# Falls back to background processes if tmux is not available
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
WEB_DIR="$ROOT_DIR/web"
SCRIPTS_DIR="$WEB_DIR/scripts"
SESSION_NAME="pwa-services"

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

# Check if tmux is available
TMUX_AVAILABLE=false
if command -v tmux >/dev/null 2>&1; then
  TMUX_AVAILABLE=true
fi

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
  if [ "${port}" = "5000" ]; then
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

  # Check if using brew services for gateway
  if [ "${USE_BREW_SERVICES:-false}" = true ] || [ "${IB_GATEWAY_USE_BREW_SERVICES:-}" = "1" ]; then
    PLIST_NAME="com.davidl71.ib-gateway"
    if brew services list 2>/dev/null | grep -q "${PLIST_NAME}.*started"; then
      echo "${BLUE}Stopping IB Gateway (brew services)...${NC}"
      brew services stop "${PLIST_NAME}" 2>/dev/null || true
      echo "${GREEN}✓ IB Gateway stopped${NC}"
    fi
  fi

  if [ "$TMUX_AVAILABLE" = true ]; then
    # Kill tmux session if it exists
    if tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
      echo "${BLUE}Stopping tmux session: ${SESSION_NAME}${NC}"
      tmux kill-session -t "$SESSION_NAME" 2>/dev/null || true
      echo "${GREEN}✓ Tmux session stopped${NC}"
    else
      echo "${YELLOW}No tmux session found${NC}"
    fi
  fi

  # Kill processes on service ports
  for port in 5173 5000 8000 8001 8002 8003 8004; do
    if check_port "$port"; then
      echo "${BLUE}Stopping service on port ${port}...${NC}"
      if command -v lsof >/dev/null 2>&1; then
        lsof -ti ":${port}" | xargs kill -9 2>/dev/null || true
      fi
    fi
  done

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

    if [ "$TMUX_AVAILABLE" = true ]; then
      if tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
        echo "${GREEN}✓ Tmux session '${SESSION_NAME}' is running${NC}"
        echo ""
        echo "Windows:"
        tmux list-windows -t "$SESSION_NAME" -F "  #{window_index}: #{window_name}"
        echo ""
        if [ -n "${ITERM_PROFILE:-}" ] || [ -n "${ITERM_SESSION_ID:-}" ]; then
          echo "Attach with iTerm2: ${BLUE}tmux -CC attach -t ${SESSION_NAME}${NC}"
        else
          echo "Attach with: ${BLUE}tmux attach -t ${SESSION_NAME}${NC}"
        fi
      else
        echo "${YELLOW}⚠ Tmux session '${SESSION_NAME}' is not running${NC}"
      fi
    else
      echo "${YELLOW}⚠ Tmux not available - services may be running as background processes${NC}"
    fi

    echo ""
    echo "Port Status:"
    for port in 5173 5000 8000 8001 8002 8003 8004; do
      if check_port "$port"; then
        echo "  ${GREEN}✓ Port ${port}: In use${NC}"
      else
        echo "  ${RED}✗ Port ${port}: Available${NC}"
      fi
    done
    exit 0
    ;;
  attach)
    if [ "$TMUX_AVAILABLE" = true ]; then
      if tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
        # Use iTerm2 integration if available
        if [ -n "${ITERM_PROFILE:-}" ] || [ -n "${ITERM_SESSION_ID:-}" ] || echo "${TERM_PROGRAM:-}" | grep -qi "iterm"; then
          exec tmux -CC attach -t "$SESSION_NAME"
        else
          exec tmux attach -t "$SESSION_NAME"
        fi
      else
        echo "${RED}Error: Tmux session '${SESSION_NAME}' not found${NC}" >&2
        echo "Start services first with: $0 start" >&2
        exit 1
      fi
    else
      echo "${RED}Error: Tmux not available${NC}" >&2
      exit 1
    fi
    ;;
  start|*)
    # Continue with start logic
    ;;
esac

# Get service ports from config or defaults
WEB_PORT=$(get_service_port "web" 5173)
GATEWAY_PORT=$(get_service_port "ib_gateway" 5000)
ALPACA_PORT=$(get_service_port "alpaca" 8000)
TRADESTATION_PORT=$(get_service_port "tradestation" 8001)
IB_PORT=$(get_service_port "ib" 8002)
DISCOUNT_BANK_PORT=$(get_service_port "discount_bank" 8003)
RISK_FREE_RATE_PORT=$(get_service_port "risk_free_rate" 8004)
JUPYTERLAB_PORT=$(get_service_port "jupyterlab" 8888)

# Check service health and track which are running
declare -A SERVICE_STATUS
SERVICES_TO_START=()

echo "${BLUE}Checking PWA Services Status...${NC}"
echo ""

# Check each service
if check_service_health "${WEB_PORT}"; then
  SERVICE_STATUS["web"]="running"
  echo "${GREEN}✓ Web service (port ${WEB_PORT}) is running${NC}"
else
  SERVICE_STATUS["web"]="stopped"
  SERVICES_TO_START+=("web")
  echo "${YELLOW}⚠ Web service (port ${WEB_PORT}) is not running${NC}"
fi

# Check if using brew services
if [ "$USE_BREW_SERVICES" = true ]; then
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

if check_service_health "${ALPACA_PORT}"; then
  SERVICE_STATUS["alpaca"]="running"
  echo "${GREEN}✓ Alpaca service (port ${ALPACA_PORT}) is running${NC}"
else
  SERVICE_STATUS["alpaca"]="stopped"
  SERVICES_TO_START+=("alpaca")
  echo "${YELLOW}⚠ Alpaca service (port ${ALPACA_PORT}) is not running${NC}"
fi

if check_service_health "${TRADESTATION_PORT}"; then
  SERVICE_STATUS["tradestation"]="running"
  echo "${GREEN}✓ TradeStation service (port ${TRADESTATION_PORT}) is running${NC}"
else
  SERVICE_STATUS["tradestation"]="stopped"
  SERVICES_TO_START+=("tradestation")
  echo "${YELLOW}⚠ TradeStation service (port ${TRADESTATION_PORT}) is not running${NC}"
fi

if check_service_health "${IB_PORT}"; then
  SERVICE_STATUS["ib"]="running"
  echo "${GREEN}✓ IB service (port ${IB_PORT}) is running${NC}"
else
  SERVICE_STATUS["ib"]="stopped"
  SERVICES_TO_START+=("ib")
  echo "${YELLOW}⚠ IB service (port ${IB_PORT}) is not running${NC}"
fi

if check_service_health "${DISCOUNT_BANK_PORT}"; then
  SERVICE_STATUS["discount_bank"]="running"
  echo "${GREEN}✓ Discount Bank service (port ${DISCOUNT_BANK_PORT}) is running${NC}"
else
  SERVICE_STATUS["discount_bank"]="stopped"
  SERVICES_TO_START+=("discount_bank")
  echo "${YELLOW}⚠ Discount Bank service (port ${DISCOUNT_BANK_PORT}) is not running${NC}"
fi

if check_service_health "${JUPYTERLAB_PORT}"; then
  SERVICE_STATUS["jupyterlab"]="running"
  echo "${GREEN}✓ JupyterLab service (port ${JUPYTERLAB_PORT}) is running${NC}"
else
  SERVICE_STATUS["jupyterlab"]="stopped"
  SERVICES_TO_START+=("jupyterlab")
  echo "${YELLOW}⚠ JupyterLab service (port ${JUPYTERLAB_PORT}) is not running${NC}"
fi

if check_service_health "${RISK_FREE_RATE_PORT}"; then
  SERVICE_STATUS["risk_free_rate"]="running"
  echo "${GREEN}✓ Risk-Free Rate service (port ${RISK_FREE_RATE_PORT}) is running${NC}"
else
  SERVICE_STATUS["risk_free_rate"]="stopped"
  SERVICES_TO_START+=("risk_free_rate")
  echo "${YELLOW}⚠ Risk-Free Rate service (port ${RISK_FREE_RATE_PORT}) is not running${NC}"
fi

echo ""

# If all services are running and tmux session exists, just attach
if [ "$TMUX_AVAILABLE" = true ] && [ ${#SERVICES_TO_START[@]} -eq 0 ]; then
  if tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
    echo "${GREEN}✓ All services are running and tmux session exists${NC}"
    echo "${BLUE}Attaching to existing tmux session...${NC}"
    echo ""
    if [ -n "${ITERM_PROFILE:-}" ] || [ -n "${ITERM_SESSION_ID:-}" ] || echo "${TERM_PROGRAM:-}" | grep -qi "iterm"; then
      exec tmux -CC attach -t "$SESSION_NAME"
    else
      exec tmux attach -t "$SESSION_NAME"
    fi
  else
    echo "${YELLOW}⚠ All services are running but no tmux session found${NC}"
    echo "${BLUE}Creating tmux session to manage services...${NC}"
    echo ""
    # Continue to create tmux session below
  fi
elif [ ${#SERVICES_TO_START[@]} -eq 0 ]; then
  echo "${GREEN}✓ All services are running${NC}"
  if [ "$TMUX_AVAILABLE" = false ]; then
    echo "${YELLOW}⚠ Tmux not available - services are running in background${NC}"
    echo "Use ${BLUE}$0 status${NC} to check service status"
  fi
  exit 0
fi

# If we get here, we need to start some services
if [ ${#SERVICES_TO_START[@]} -gt 0 ]; then
  echo "${BLUE}Starting ${#SERVICES_TO_START[@]} service(s): ${SERVICES_TO_START[*]}${NC}"
  echo ""
fi

# Launch services
if [ "$TMUX_AVAILABLE" = true ]; then
  echo "${GREEN}Using tmux for service management${NC}"
  echo ""

  # Check if session already exists - if so, we'll attach to it after starting missing services
  SESSION_EXISTS=false
  if tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
    SESSION_EXISTS=true
    echo "${BLUE}Tmux session '${SESSION_NAME}' already exists${NC}"
  fi

  # Create new session if it doesn't exist (start with first service that needs starting)
  if [ "$SESSION_EXISTS" = false ]; then
    # Find first service to start for initial window
    FIRST_SERVICE=""
    for service in web gateway alpaca tradestation ib discount_bank risk_free_rate; do
      if [[ " ${SERVICES_TO_START[*]} " =~ ${service} ]]; then
        FIRST_SERVICE="${service}"
        break
      fi
    done

    # If no services need starting, create empty session
    if [ -z "${FIRST_SERVICE}" ]; then
      FIRST_SERVICE="web"
    fi

    # Create session with appropriate first window
    case "${FIRST_SERVICE}" in
      web)
        if ! tmux new-session -d -s "$SESSION_NAME" -n "web" -c "$WEB_DIR" \
          "bash ${SCRIPTS_DIR}/run-web-service.sh" 2>/dev/null; then
          echo "${RED}Error: Failed to create tmux session${NC}" >&2
          exit 1
        fi
        ;;
      *)
        # Create empty session, we'll add windows below
        if ! tmux new-session -d -s "$SESSION_NAME" -n "${FIRST_SERVICE}" 2>/dev/null; then
          echo "${RED}Error: Failed to create tmux session${NC}" >&2
          exit 1
        fi
        ;;
    esac
  fi

  # Verify session exists
  if ! tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
    echo "${RED}Error: Failed to verify tmux session${NC}" >&2
    exit 1
  fi

  # Small delay to ensure session is ready
  sleep 0.5

  # Configure tmux to show all windows in status bar if terminal size allows
  # Use session-specific options (not global) for this session only
  tmux set-option -t "$SESSION_NAME" status on
  tmux set-option -t "$SESSION_NAME" status-position bottom
  # Minimize left/right status to maximize space for window list
  tmux set-option -t "$SESSION_NAME" status-left-length 0
  tmux set-option -t "$SESSION_NAME" status-right-length 0
  tmux set-option -t "$SESSION_NAME" status-justify left
  # Compact window format: show index and name (inactive windows)
  tmux set-option -t "$SESSION_NAME" window-status-format " #I:#W "
  # Highlight current window (bold and brighter)
  tmux set-option -t "$SESSION_NAME" window-status-current-format "#[bold]#[fg=colour255] #I:#W #[fg=colour245]"
  # Clear status left/right to maximize window list space
  tmux set-option -t "$SESSION_NAME" status-left ""
  tmux set-option -t "$SESSION_NAME" status-right ""
  # Window list separator (space between windows)
  tmux set-option -t "$SESSION_NAME" window-status-separator ""
  # Enable automatic window list (shows all windows that fit in terminal width)
  tmux set-option -t "$SESSION_NAME" automatic-rename on

  # Start services that need starting
  GATEWAY_DIR="${ROOT_DIR}/ib-gateway"
  GATEWAY_RUNNING=false

  # Check if gateway is already running
  if [ "${SERVICE_STATUS[gateway]}" = "running" ]; then
    GATEWAY_RUNNING=true
    echo "${GREEN}✓ IB Gateway already running${NC}"
  fi

  # Start Web service if needed
  if [[ " ${SERVICES_TO_START[*]} " =~ " web " ]]; then
    # Check if web window already exists
    if ! tmux list-windows -t "$SESSION_NAME" -F "#{window_name}" 2>/dev/null | grep -q "^web$"; then
      echo "${BLUE}  Starting web service...${NC}"
      tmux new-window -t "$SESSION_NAME" -n "web" -c "$WEB_DIR" \
        "bash ${SCRIPTS_DIR}/run-web-service.sh" 2>/dev/null || true
    fi
  fi

  # Start IB Gateway if needed
  if [[ " ${SERVICES_TO_START[*]} " =~ " gateway " ]]; then
    # Check if using brew services
    if [ "$USE_BREW_SERVICES" = true ]; then
      PLIST_NAME="com.davidl71.ib-gateway"
      if [ ! -f "${HOME}/Library/LaunchAgents/${PLIST_NAME}.plist" ]; then
        echo "${YELLOW}  Brew service not installed. Installing...${NC}"
        if [ -f "${GATEWAY_DIR}/install-brew-service.sh" ]; then
          bash "${GATEWAY_DIR}/install-brew-service.sh"
        else
          echo "${RED}  [ERROR] install-brew-service.sh not found${NC}" >&2
          echo "${YELLOW}  [PATH] Expected: ${GATEWAY_DIR}/install-brew-service.sh${NC}" >&2
        fi
      fi
      echo "${BLUE}  Starting IB Gateway via brew services...${NC}"
      brew services start "${PLIST_NAME}" 2>&1 || {
        echo "${RED}  [ERROR] Failed to start brew service${NC}" >&2
        echo "${YELLOW}  [INFO] Try: brew services start ${PLIST_NAME}${NC}" >&2
      }
    elif [ -f "${GATEWAY_DIR}/run-gateway-with-reload.sh" ] || [ -f "${GATEWAY_DIR}/run-gateway.sh" ] || [ -f "${GATEWAY_DIR}/bin/run.sh" ]; then
      # Check if gateway window already exists
      if ! tmux list-windows -t "$SESSION_NAME" -F "#{window_name}" 2>/dev/null | grep -q "^ib-gateway$"; then
        echo "${BLUE}  Starting IB Gateway...${NC}"
        # Prefer auto-reload wrapper if available, fallback to regular script
        if [ -f "${GATEWAY_DIR}/run-gateway-with-reload.sh" ]; then
          tmux new-window -t "$SESSION_NAME" -n "ib-gateway" -c "$GATEWAY_DIR" \
            "bash ${GATEWAY_DIR}/run-gateway-with-reload.sh 2>&1 | tee ${GATEWAY_DIR}/gateway.log || (echo ''; echo '[ERROR] Gateway failed'; echo '[PATH] Log: ${GATEWAY_DIR}/gateway.log'; echo '[PATH] Gateway dir: ${GATEWAY_DIR}'; echo 'Press any key to continue...'; read -n 1 -s || true; exit 1)" 2>/dev/null || true
        elif [ -f "${GATEWAY_DIR}/run-gateway.sh" ]; then
          tmux new-window -t "$SESSION_NAME" -n "ib-gateway" -c "$GATEWAY_DIR" \
            "bash ${GATEWAY_DIR}/run-gateway.sh 2>&1 | tee ${GATEWAY_DIR}/gateway.log || (echo ''; echo '[ERROR] Gateway failed'; echo '[PATH] Log: ${GATEWAY_DIR}/gateway.log'; echo '[PATH] Gateway dir: ${GATEWAY_DIR}'; echo 'Press any key to continue...'; read -n 1 -s || true; exit 1)" 2>/dev/null || true
        elif [ -f "${GATEWAY_DIR}/bin/run.sh" ]; then
          CONFIG_FILE="${GATEWAY_DIR}/root/conf.yaml"
          if [ ! -f "${CONFIG_FILE}" ]; then
            CONFIG_FILE="${GATEWAY_DIR}/root/conf.tws.yaml"
          fi
          # Normalize config file path to absolute path (fixes ..// issues)
          if [ -f "${CONFIG_FILE}" ]; then
            if command -v realpath >/dev/null 2>&1; then
              CONFIG_FILE="$(realpath "${CONFIG_FILE}")"
            else
              CONFIG_FILE="$(cd "$(dirname "${CONFIG_FILE}")" && pwd)/$(basename "${CONFIG_FILE}")"
            fi
          fi
          if [ -f "${CONFIG_FILE}" ]; then
            tmux new-window -t "$SESSION_NAME" -n "ib-gateway" -c "$GATEWAY_DIR" \
              "bash ${GATEWAY_DIR}/bin/run.sh ${CONFIG_FILE} 2>&1 | tee ${GATEWAY_DIR}/gateway.log || (echo ''; echo '[ERROR] Gateway failed'; echo '[PATH] Config: ${CONFIG_FILE}'; echo '[PATH] Log: ${GATEWAY_DIR}/gateway.log'; echo '[PATH] Gateway dir: ${GATEWAY_DIR}'; echo 'Press any key to continue...'; read -n 1 -s || true; exit 1)" 2>/dev/null || true
          else
            echo "${YELLOW}  [ERROR] No config file found${NC}"
            echo "${YELLOW}  [PATH] Searched: ${GATEWAY_DIR}/root/conf.yaml${NC}"
            echo "${YELLOW}  [PATH] Searched: ${GATEWAY_DIR}/root/conf.tws.yaml${NC}"
          fi
        fi
      fi
    else
      echo "${YELLOW}  Gateway not installed (IB service will wait)${NC}"
    fi
  fi

  # Start Alpaca service if needed
  if [[ " ${SERVICES_TO_START[*]} " =~ " alpaca " ]]; then
    if ! tmux list-windows -t "$SESSION_NAME" -F "#{window_name}" 2>/dev/null | grep -q "^alpaca$"; then
      echo "${BLUE}  Starting Alpaca service...${NC}"
      tmux new-window -t "$SESSION_NAME" -n "alpaca" -c "$ROOT_DIR" \
        "bash ${SCRIPTS_DIR}/run-alpaca-service.sh" 2>/dev/null || true
    fi
  fi

  # Start TradeStation service if needed
  if [[ " ${SERVICES_TO_START[*]} " =~ " tradestation " ]]; then
    if ! tmux list-windows -t "$SESSION_NAME" -F "#{window_name}" 2>/dev/null | grep -q "^tradestation$"; then
      echo "${BLUE}  Starting TradeStation service...${NC}"
      tmux new-window -t "$SESSION_NAME" -n "tradestation" -c "$ROOT_DIR" \
        "bash ${SCRIPTS_DIR}/run-tradestation-service.sh" 2>/dev/null || true
    fi
  fi

  # Start Discount Bank service if needed
  if [[ " ${SERVICES_TO_START[*]} " =~ " discount_bank " ]]; then
    if ! tmux list-windows -t "$SESSION_NAME" -F "#{window_name}" 2>/dev/null | grep -q "^discount-bank$"; then
      echo "${BLUE}  Starting Discount Bank service...${NC}"
      tmux new-window -t "$SESSION_NAME" -n "discount-bank" -c "$ROOT_DIR" \
        "bash ${SCRIPTS_DIR}/run-discount-bank-service.sh" 2>/dev/null || true
    fi
  fi

  # Start Risk-Free Rate service if needed
  if [[ " ${SERVICES_TO_START[*]} " =~ " risk_free_rate " ]]; then
    if ! tmux list-windows -t "$SESSION_NAME" -F "#{window_name}" 2>/dev/null | grep -q "^risk-free-rate$"; then
      echo "${BLUE}  Starting Risk-Free Rate service...${NC}"
      tmux new-window -t "$SESSION_NAME" -n "risk-free-rate" -c "$ROOT_DIR" \
        "bash ${SCRIPTS_DIR}/run-risk-free-rate-service.sh" 2>/dev/null || true
    fi
  fi

  # Start JupyterLab service if needed
  if [[ " ${SERVICES_TO_START[*]} " =~ " jupyterlab " ]]; then
    if ! tmux list-windows -t "$SESSION_NAME" -F "#{window_name}" 2>/dev/null | grep -q "^jupyterlab$"; then
      echo "${BLUE}  Starting JupyterLab service...${NC}"
      tmux new-window -t "$SESSION_NAME" -n "jupyterlab" -c "$ROOT_DIR" \
        "bash ${SCRIPTS_DIR}/run-jupyterlab-service.sh" 2>/dev/null || true
    fi
  fi

  # Wait for Gateway if IB service needs starting
  if [[ " ${SERVICES_TO_START[*]} " =~ " ib " ]]; then
    if [ "$GATEWAY_RUNNING" = false ]; then
      echo "${BLUE}Waiting for Gateway to be ready before starting IB service...${NC}"
      for i in {1..20}; do
        sleep 1
        if check_service_health "${GATEWAY_PORT}"; then
          echo "${GREEN}✓ Gateway is now running and ready${NC}"
          GATEWAY_RUNNING=true
          break
        fi
        # Show progress every 3 seconds
        if [ $((i % 3)) -eq 0 ]; then
          echo "${BLUE}  Still waiting for gateway... (${i}/20 seconds)${NC}"
        fi
      done

      if [ "$GATEWAY_RUNNING" = false ]; then
        echo "${YELLOW}  Gateway may still be starting. IB service will start but API calls may fail.${NC}"
        echo "${YELLOW}  Check the 'ib-gateway' tmux window for status.${NC}"
      fi
    fi

    # Start IB service if needed
    if ! tmux list-windows -t "$SESSION_NAME" -F "#{window_name}" 2>/dev/null | grep -q "^ib$"; then
      echo "${BLUE}  Starting IB service...${NC}"
      tmux new-window -t "$SESSION_NAME" -n "ib" -c "$ROOT_DIR" \
        "bash ${SCRIPTS_DIR}/run-ib-service.sh || (echo ''; echo 'IB service failed - check logs above'; echo 'Press any key to continue...'; read -n 1 -s || true; exit 1)" 2>/dev/null || true
    fi
  fi

  # Select the first window (web) - use index 0 as fallback if name doesn't work
  # List windows to verify they exist
  if tmux list-windows -t "$SESSION_NAME" >/dev/null 2>&1; then
    if tmux list-windows -t "$SESSION_NAME" -F "#{window_name}" | grep -q "^web$"; then
      tmux select-window -t "$SESSION_NAME:web" 2>/dev/null || tmux select-window -t "$SESSION_NAME:0" 2>/dev/null || true
    else
      # Fallback to first window by index
      tmux select-window -t "$SESSION_NAME:0" 2>/dev/null || true
    fi
  fi

  echo "${GREEN}✓ Services ready in tmux session '${SESSION_NAME}'${NC}"
  echo ""
  echo "Services:"
  if [ "${SERVICE_STATUS[web]}" = "running" ]; then
    echo "  ${GREEN}✓${NC} Web service (port ${WEB_PORT}) - Window: web [running]"
  else
    echo "  ${YELLOW}⚠${NC} Web service (port ${WEB_PORT}) - Window: web [starting]"
  fi
  echo "    ${BLUE}URL: http://127.0.0.1:${WEB_PORT}${NC}"

  if [ -f "${GATEWAY_DIR}/run-gateway-with-reload.sh" ] || [ -f "${GATEWAY_DIR}/run-gateway.sh" ] || [ -f "${GATEWAY_DIR}/bin/run.sh" ]; then
    if [ "${SERVICE_STATUS[gateway]}" = "running" ]; then
      echo "  ${GREEN}✓${NC} IB Gateway (port ${GATEWAY_PORT}) - Window: ib-gateway [running]"
    else
      echo "  ${YELLOW}⚠${NC} IB Gateway (port ${GATEWAY_PORT}) - Window: ib-gateway [starting]"
    fi
    echo "    ${BLUE}URL: https://localhost:${GATEWAY_PORT}${NC}"
  fi

  if [ "${SERVICE_STATUS[alpaca]}" = "running" ]; then
    echo "  ${GREEN}✓${NC} Alpaca service (port ${ALPACA_PORT}) - Window: alpaca [running]"
  else
    echo "  ${YELLOW}⚠${NC} Alpaca service (port ${ALPACA_PORT}) - Window: alpaca [starting]"
  fi
  echo "    ${BLUE}URL: http://127.0.0.1:${ALPACA_PORT}${NC}"

  if [ "${SERVICE_STATUS[ib]}" = "running" ]; then
    echo "  ${GREEN}✓${NC} IB service (port ${IB_PORT}) - Window: ib [running]"
  else
    echo "  ${YELLOW}⚠${NC} IB service (port ${IB_PORT}) - Window: ib [starting]"
  fi
  echo "    ${BLUE}URL: http://127.0.0.1:${IB_PORT}${NC}"

  if [ "${SERVICE_STATUS[tradestation]}" = "running" ]; then
    echo "  ${GREEN}✓${NC} TradeStation service (port ${TRADESTATION_PORT}) - Window: tradestation [running]"
  else
    echo "  ${YELLOW}⚠${NC} TradeStation service (port ${TRADESTATION_PORT}) - Window: tradestation [starting]"
  fi
  echo "    ${BLUE}URL: http://127.0.0.1:${TRADESTATION_PORT}${NC}"

  if [ "${SERVICE_STATUS[discount_bank]}" = "running" ]; then
    echo "  ${GREEN}✓${NC} Discount Bank service (port ${DISCOUNT_BANK_PORT}) - Window: discount-bank [running]"
  else
    echo "  ${YELLOW}⚠${NC} Discount Bank service (port ${DISCOUNT_BANK_PORT}) - Window: discount-bank [starting]"
  fi
  echo "    ${BLUE}URL: http://127.0.0.1:${DISCOUNT_BANK_PORT}${NC}"

  if [ "${SERVICE_STATUS[risk_free_rate]}" = "running" ]; then
    echo "  ${GREEN}✓${NC} Risk-Free Rate service (port ${RISK_FREE_RATE_PORT}) - Window: risk-free-rate [running]"
  else
    echo "  ${YELLOW}⚠${NC} Risk-Free Rate service (port ${RISK_FREE_RATE_PORT}) - Window: risk-free-rate [starting]"
  fi
  echo "    ${BLUE}URL: http://127.0.0.1:${RISK_FREE_RATE_PORT}${NC}"
  echo ""
  echo "Commands:"
  if [ -n "${ITERM_PROFILE:-}" ] || [ -n "${ITERM_SESSION_ID:-}" ]; then
    echo "  ${BLUE}tmux -CC attach -t ${SESSION_NAME}${NC}  # Attach with iTerm2 integration"
  else
    echo "  ${BLUE}tmux attach -t ${SESSION_NAME}${NC}  # Attach to session"
  fi
  echo "  ${BLUE}$0 attach${NC}                       # Attach to session (alias)"
  echo "  ${BLUE}$0 status${NC}                       # Check service status"
  echo "  ${BLUE}$0 stop${NC}                         # Stop all services"
  echo "  ${BLUE}$0 --brew-services${NC}              # Use brew services for IB Gateway"
  echo ""
  echo "Tmux shortcuts (when attached):"
  echo "  ${BLUE}Ctrl-b w${NC}  # List windows"
  echo "  ${BLUE}Ctrl-b n${NC}  # Next window"
  echo "  ${BLUE}Ctrl-b p${NC}  # Previous window"
  echo "  ${BLUE}Ctrl-b d${NC}  # Detach from session"
  echo ""
  # iTerm2 Captured Output markers
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━NC}"
  echo ""
  echo "[CAPTURE:URL] Service URLs (clickable in iTerm2):"
  echo "  http://127.0.0.1:${WEB_PORT}  # Web frontend"
  if [ -f "${GATEWAY_DIR}/run-gateway-with-reload.sh" ] || [ -f "${GATEWAY_DIR}/run-gateway.sh" ] || [ -f "${GATEWAY_DIR}/bin/run.sh" ]; then
    echo "  https://localhost:${GATEWAY_PORT}  # IB Gateway"
  fi
  echo "  http://127.0.0.1:${ALPACA_PORT}  # Alpaca API"
  echo "  http://127.0.0.1:${TRADESTATION_PORT}  # TradeStation API"
  echo "  http://127.0.0.1:${IB_PORT}  # IB API"
  echo "  http://127.0.0.1:${DISCOUNT_BANK_PORT}  # Discount Bank API"
  echo "  http://127.0.0.1:${RISK_FREE_RATE_PORT}  # Risk-Free Rate API"
  echo ""
  echo "[CAPTURE:PORT] Service ports: ${WEB_PORT}, ${GATEWAY_PORT}, ${ALPACA_PORT}, ${TRADESTATION_PORT}, ${IB_PORT}, ${DISCOUNT_BANK_PORT}, ${RISK_FREE_RATE_PORT}"
  echo "[CAPTURE:PATH] Config file: ${ROOT_DIR}/config/config.json"
  echo "[CAPTURE:PATH] Scripts: ${SCRIPTS_DIR}"
  echo ""
  echo "[AI:ANALYZE] Service Status Summary (for iTerm2 AI Chat):"
  echo "  Services: Web(${WEB_PORT}), Gateway(${GATEWAY_PORT}), Alpaca(${ALPACA_PORT}), IB(${IB_PORT}), TradeStation(${TRADESTATION_PORT}), DiscountBank(${DISCOUNT_BANK_PORT}), RiskFreeRate(${RISK_FREE_RATE_PORT})"
  echo "  Session: ${SESSION_NAME}"
  echo "  Status: Services managed in tmux"
  echo "  Running: $([ ${#SERVICES_TO_START[@]} -eq 0 ] && echo "All" || echo "$((7 - ${#SERVICES_TO_START[@]})) of 7")"
  echo ""
  echo "iTerm2 AI Chat Integration:"
  echo "  • Use 'Edit > Explain Output with AI' to analyze this output"
  echo "  • Link this session to AI Chat for real-time analysis"
  echo "  • AI can check service health, analyze logs, and suggest fixes"
  echo ""

  # Automatically attach if running in interactive terminal
  if [ -t 1 ]; then
    echo "${BLUE}Attaching to tmux session...${NC}"
    # Check if running in iTerm2 and use iTerm2's tmux integration if available
    if [ -n "${ITERM_PROFILE:-}" ] || [ -n "${ITERM_SESSION_ID:-}" ] || echo "$TERM_PROGRAM" | grep -qi "iterm"; then
      echo "${BLUE}Detected iTerm2 - using native tmux integration (Control Mode)${NC}"
      exec tmux -CC attach -t "$SESSION_NAME"
    else
      exec tmux attach -t "$SESSION_NAME"
    fi
  else
    echo "${YELLOW}Not running in interactive terminal - skipping auto-attach${NC}"
    if [ -n "${ITERM_PROFILE:-}" ] || [ -n "${ITERM_SESSION_ID:-}" ]; then
      echo "Attach with iTerm2 integration: ${BLUE}tmux -CC attach -t ${SESSION_NAME}${NC}"
    else
      echo "Attach manually with: ${BLUE}tmux attach -t ${SESSION_NAME}${NC}"
    fi
  fi

else
  echo "${YELLOW}⚠ Tmux not available - launching services in background${NC}"
  echo ""

  # Launch services in background (parallel groups for faster startup)
  # Group 1 (parallel): Web, Gateway, Alpaca, TradeStation, Discount Bank
  # Group 2 (after Gateway ready): IB service

  echo "${BLUE}Starting independent services in parallel...${NC}"

  # Group 1: Start all independent services simultaneously
  # Web service
  (
    cd "$WEB_DIR"
    bash "${SCRIPTS_DIR}/run-web-service.sh" > /tmp/pwa-web.log 2>&1 &
    echo $! > /tmp/pwa-web.pid
  ) &

  # IB Gateway (IB service will wait for it)
  GATEWAY_DIR="${ROOT_DIR}/ib-gateway"
  GATEWAY_RUNNING=false
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
    if ! curl -k -s --connect-timeout 1 "https://localhost:5000/sso/validate" >/dev/null 2>&1; then
      echo "${BLUE}Starting IB Gateway in background...${NC}"
      (
        cd "$GATEWAY_DIR"
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
      ) &
    else
      echo "${GREEN}✓ IB Gateway already running${NC}"
      GATEWAY_RUNNING=true
    fi
  fi

  # Alpaca service (independent, port 8000)
  (
    cd "$ROOT_DIR"
    bash "${SCRIPTS_DIR}/run-alpaca-service.sh" > /tmp/pwa-alpaca.log 2>&1 &
    echo $! > /tmp/pwa-alpaca.pid
  ) &

  # TradeStation service (independent, port 8001)
  (
    cd "$ROOT_DIR"
    bash "${SCRIPTS_DIR}/run-tradestation-service.sh" > /tmp/pwa-tradestation.log 2>&1 &
    echo $! > /tmp/pwa-tradestation.pid
  ) &

  # Discount Bank service (independent, port 8003)
  (
    cd "$ROOT_DIR"
    bash "${SCRIPTS_DIR}/run-discount-bank-service.sh" > /tmp/pwa-discount-bank.log 2>&1 &
    echo $! > /tmp/pwa-discount-bank.pid
  ) &

  # Risk-Free Rate service (port 8004)
  (
    cd "$ROOT_DIR"
    bash "${SCRIPTS_DIR}/run-risk-free-rate-service.sh" > /tmp/pwa-risk-free-rate.log 2>&1 &
    echo $! > /tmp/pwa-risk-free-rate.pid
  ) &

  # JupyterLab service (port 8888)
  (
    cd "$ROOT_DIR"
    bash "${SCRIPTS_DIR}/run-jupyterlab-service.sh" > /tmp/pwa-jupyterlab.log 2>&1 &
    echo $! > /tmp/pwa-jupyterlab.pid
  ) &

  # Group 2: Wait for Gateway, then start IB service
  if [ "$GATEWAY_RUNNING" = false ]; then
    echo "${BLUE}Waiting for Gateway to be ready before starting IB service...${NC}"
    for i in {1..20}; do
      sleep 1
      if curl -k -s --connect-timeout 1 "https://localhost:5000/sso/validate" >/dev/null 2>&1; then
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

  wait

  echo "${GREEN}✓ All services launched in background${NC}"
  echo ""
  echo "Services:"
  echo "  ${GREEN}✓${NC} Web service (Vite) - Log: /tmp/pwa-web.log"
  if [ -f "${GATEWAY_DIR}/run-gateway-with-reload.sh" ] || [ -f "${GATEWAY_DIR}/run-gateway.sh" ] || [ -f "${GATEWAY_DIR}/bin/run.sh" ]; then
    echo "  ${GREEN}✓${NC} IB Gateway (port 5000) - Log: /tmp/pwa-ib-gateway.log"
  fi
  echo "  ${GREEN}✓${NC} Alpaca service (port 8000) - Log: /tmp/pwa-alpaca.log"
  echo "  ${GREEN}✓${NC} IB service (port 8002) - Log: /tmp/pwa-ib.log"
  echo "  ${GREEN}✓${NC} TradeStation service (port 8001) - Log: /tmp/pwa-tradestation.log"
  echo "  ${GREEN}✓${NC} Discount Bank service (port 8003) - Log: /tmp/pwa-discount-bank.log"
  echo "  ${GREEN}✓${NC} Risk-Free Rate service (port 8004) - Log: /tmp/pwa-risk-free-rate.log"
  echo ""
  echo "Commands:"
  echo "  ${BLUE}tail -f /tmp/pwa-*.log${NC}  # View all logs"
  echo "  ${BLUE}$0 status${NC}                # Check service status"
  echo "  ${BLUE}$0 stop${NC}                  # Stop all services"
  echo ""
fi
