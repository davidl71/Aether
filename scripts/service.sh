#!/usr/bin/env bash
# Unified service manager: start, stop, status, logs for all platform services.
# Usage: ./scripts/service.sh {start|stop|status|logs} <service-name>
#        ./scripts/service.sh list
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
SCRIPTS_DIR="${ROOT_DIR}/scripts"
LOGS_DIR="${ROOT_DIR}/logs"

if [ -f "${SCRIPTS_DIR}/include/config.sh" ]; then
  source "${SCRIPTS_DIR}/include/config.sh"
fi

declare -A SVC_PORT SVC_CMD SVC_LOG SVC_HEALTH SVC_DISPLAY SVC_WAIT

_register() {
  local name="$1" display="$2" port="$3" cmd="$4" log="$5" health="${6:-}" wait="${7:-4}"
  SVC_DISPLAY[$name]="$display"
  SVC_PORT[$name]="$port"
  SVC_CMD[$name]="$cmd"
  SVC_LOG[$name]="$log"
  SVC_HEALTH[$name]="$health"
  SVC_WAIT[$name]="$wait"
}

_register nats       "NATS"              "" "nats-server"                                        "nats-server.log"            "http://localhost:8222/healthz" 2
_register ib         "IB"                "$(config_get_port ib 8002 2>/dev/null || echo 8002)"         "./web/scripts/run-ib-service.sh"        "ib-service.log"             "http://localhost:\${PORT}/api/health"
_register alpaca     "Alpaca"            "$(config_get_port alpaca 8000 2>/dev/null || echo 8000)"     "./web/scripts/run-alpaca-service.sh"    "alpaca-service.log"         "http://localhost:\${PORT}/api/health"
_register tastytrade "Tastytrade"        "$(config_get_port tastytrade 8005 2>/dev/null || echo 8005)" "./web/scripts/run-tastytrade-service.sh" "tastytrade-service.log"    "http://localhost:\${PORT}/api/health"
_register tradestation "TradeStation"    "$(config_get_port tradestation 8006 2>/dev/null || echo 8006)" "./web/scripts/run-tradestation-service.sh" "tradestation-service.log" "http://localhost:\${PORT}/api/health"
_register riskfree   "Risk-Free Rate"    "$(config_get_port risk_free_rate 8004 2>/dev/null || echo 8004)" "./web/scripts/run-risk-free-rate-service.sh" "risk-free-rate-service.log" "http://localhost:\${PORT}/api/health"
_register discount   "Discount Bank"     "$(config_get_port discount_bank 8003 2>/dev/null || echo 8003)" "./web/scripts/run-discount-bank-service.sh" "discount-bank-service.log" "http://localhost:\${PORT}/api/health"
_register web        "Web Dev"           "5173"                                                  "npm run dev"                            "web-dev-server.log"         "http://localhost:5173" 3
_register rust       "Rust Backend"      "$(config_get_port rust_backend 8010 2>/dev/null || echo 8010)" "./agents/start_rust_backend.sh"  "rust-backend.log"           "http://localhost:\${PORT}/health"
_register memcached  "Memcached"         "11211" "memcached -l 127.0.0.1"                             "memcached.log"              "" 2

_is_nats() { [[ "$1" == "nats" ]]; }
_is_memcached() { [[ "$1" == "memcached" ]]; }

_find_pid() {
  local svc="$1"
  if _is_nats "$svc"; then
    pgrep -f "nats-server" 2>/dev/null || true
  elif _is_memcached "$svc"; then
    pgrep -f "memcached" 2>/dev/null || true
  else
    lsof -ti :"${SVC_PORT[$svc]}" 2>/dev/null || true
  fi
}

_health_url() {
  local svc="$1"
  local url="${SVC_HEALTH[$svc]}"
  local port="${SVC_PORT[$svc]}"
  echo "${url//\$\{PORT\}/$port}"
}

do_start() {
  local svc="$1"
  local display="${SVC_DISPLAY[$svc]}"
  local port="${SVC_PORT[$svc]}"
  local log="${LOGS_DIR}/${SVC_LOG[$svc]}"
  local wait="${SVC_WAIT[$svc]}"

  local pid; pid=$(_find_pid "$svc")
  if [[ -n "$pid" ]]; then
    echo "[info] ${display} already running (PID: $pid)"
    return 0
  fi

  mkdir -p "$LOGS_DIR"
  echo "[info] Starting ${display}..."

  if _is_nats "$svc"; then
    local cfg="${ROOT_DIR}/config/nats-server.conf"
    if [[ -f "$cfg" ]]; then
      nats-server -c "$cfg" > "$log" 2>&1 &
    else
      nats-server > "$log" 2>&1 &
    fi
  elif _is_memcached "$svc"; then
    memcached -l 127.0.0.1 > "$log" 2>&1 &
  elif [[ "$svc" == "web" ]]; then
    (cd "${ROOT_DIR}/web" && npm run dev > "$log" 2>&1) &
  else
    ${SVC_CMD[$svc]} > "$log" 2>&1 &
  fi

  local svc_pid=$!
  disown "$svc_pid" 2>/dev/null || true
  sleep "$wait"

  pid=$(_find_pid "$svc")
  if [[ -n "$pid" ]]; then
    echo "[info] ${display} started (PID: $pid)"
    [[ -n "$port" ]] && echo "[info] Port: $port"
    echo "[info] Log: $log"
  else
    echo "[error] Failed to start ${display}"
    tail -10 "$log" 2>/dev/null || true
    return 1
  fi
}

do_stop() {
  local svc="$1"
  local display="${SVC_DISPLAY[$svc]}"

  local pid; pid=$(_find_pid "$svc")
  if [[ -z "$pid" ]]; then
    echo "[info] ${display} is not running"
    return 0
  fi

  echo "[info] Stopping ${display} (PID: $pid)..."
  kill $pid 2>/dev/null || true

  for _ in {1..10}; do
    pid=$(_find_pid "$svc")
    [[ -z "$pid" ]] && { echo "[info] ${display} stopped"; return 0; }
    sleep 0.5
  done

  pid=$(_find_pid "$svc")
  if [[ -n "$pid" ]]; then
    echo "[warn] Force killing ${display}..."
    kill -9 $pid 2>/dev/null || true
    sleep 1
    pid=$(_find_pid "$svc")
    if [[ -n "$pid" ]]; then
      echo "[error] Failed to stop ${display}"
      return 1
    fi
  fi
  echo "[info] ${display} stopped"
}

do_status() {
  local svc="$1"
  local display="${SVC_DISPLAY[$svc]}"
  local pid; pid=$(_find_pid "$svc")
  if [[ -n "$pid" ]]; then
    local health_url; health_url=$(_health_url "$svc")
    local health="unknown"
    if [[ -n "$health_url" ]]; then
      curl -sf "$health_url" >/dev/null 2>&1 && health="healthy" || health="unhealthy"
    fi
    echo "[info] ${display}: running (PID: $pid, health: $health)"
  else
    echo "[info] ${display}: stopped"
  fi
}

do_logs() {
  local svc="$1"
  local log="${LOGS_DIR}/${SVC_LOG[$svc]}"
  if [[ -f "$log" ]]; then
    tail -50 "$log"
  else
    echo "[info] No log file found: $log"
  fi
}

do_list() {
  echo "Available services:"
  for svc in nats memcached ib alpaca tastytrade tradestation riskfree discount web rust; do
    local port="${SVC_PORT[$svc]}"
    printf "  %-14s  %-20s  %s\n" "$svc" "${SVC_DISPLAY[$svc]}" "${port:+port $port}"
  done
}

ACTION="${1:-}"
SERVICE="${2:-}"

case "$ACTION" in
  list)
    do_list
    ;;
  start|stop|status|logs)
    if [[ -z "$SERVICE" ]]; then
      echo "Usage: $0 $ACTION <service-name>"
      echo "Run '$0 list' to see available services."
      exit 1
    fi
    if [[ -z "${SVC_DISPLAY[$SERVICE]+x}" ]]; then
      echo "[error] Unknown service: $SERVICE"
      do_list
      exit 1
    fi
    "do_${ACTION}" "$SERVICE"
    ;;
  *)
    echo "Usage: $0 {start|stop|status|logs|list} [service-name]"
    exit 1
    ;;
esac
