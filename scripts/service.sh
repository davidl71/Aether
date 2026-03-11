#!/usr/bin/env bash
# Unified service manager: start, stop, status, logs for all platform services.
# Usage: ./scripts/service.sh {start|stop|status|logs} <service-name>
#        ./scripts/service.sh list
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
SCRIPTS_DIR="${ROOT_DIR}/scripts"
LOGS_DIR="${ROOT_DIR}/logs"

if [ -f "${SCRIPTS_DIR}/include/config.sh" ]; then
  # shellcheck source=scripts/include/config.sh
  source "${SCRIPTS_DIR}/include/config.sh"
fi

_config_port() {
  if type config_get_port &>/dev/null; then
    config_get_port "$1" "$2" 2>/dev/null || echo "$2"
  else
    echo "$2"
  fi
}

_config_enabled() {
  if type config_is_enabled &>/dev/null; then
    config_is_enabled "$1" "${2:-true}"
  else
    return 0
  fi
}

_svc_display() {
  case "$1" in
    nats) echo "NATS" ;;
    memcached) echo "Memcached" ;;
    gateway) echo "IB Gateway" ;;
    discount) echo "Discount Bank" ;;
    rust) echo "Rust Backend" ;;
    questdb_nats) echo "QuestDB NATS" ;;
    *) echo "" ;;
  esac
}

_svc_port() {
  case "$1" in
    nats) echo "4222, 8222, 8081" ;;
    memcached) echo "11211" ;;
    gateway) echo "${IB_GATEWAY_PORT:-5001}" ;;
    rust) _config_port rust_backend 8080 ;;
    questdb_nats) echo "" ;;
    *) echo "" ;;
  esac
}

_svc_cmd() {
  case "$1" in
    nats) echo "nats-server" ;;
    memcached) echo "memcached -l 127.0.0.1" ;;
    gateway) echo "./ib-gateway/run-gateway.sh" ;;
    rust) echo "./agents/start_rust_backend.sh" ;;
    questdb_nats) echo "./scripts/run_questdb_nats_writer.sh" ;;
    *) echo "" ;;
  esac
}

_svc_log() {
  case "$1" in
    nats) echo "nats-server.log" ;;
    memcached) echo "memcached.log" ;;
    gateway) echo "ib-gateway.log" ;;
    rust) echo "rust-backend.log" ;;
    questdb_nats) echo "questdb-nats-writer.log" ;;
    *) echo "" ;;
  esac
}

_svc_health() {
  case "$1" in
    nats) echo "http://localhost:8222/healthz" ;;
    memcached) echo "" ;;
    gateway) echo "https://localhost:\${PORT}" ;;
    rust) echo "http://localhost:\${PORT}/health" ;;
    questdb_nats) echo "" ;;
    *) echo "" ;;
  esac
}

_svc_wait() {
  case "$1" in
    nats|memcached) echo "2" ;;
    gateway) echo "5" ;;
    questdb_nats) echo "3" ;;
    *) echo "4" ;;
  esac
}

_svc_known() {
  case "$1" in
    nats|memcached|gateway|rust|questdb_nats)
      return 0
      ;;
    *) return 1 ;;
  esac
}

_is_nats() { [[ "$1" == "nats" ]]; }
_is_memcached() { [[ "$1" == "memcached" ]]; }
_is_questdb_nats() { [[ "$1" == "questdb_nats" ]]; }

_find_pid() {
  local svc="$1"
  local port
  if _is_nats "$svc"; then
    pgrep -f "nats-server" 2>/dev/null || true
  elif _is_memcached "$svc"; then
    pgrep -f "memcached" 2>/dev/null || true
  elif _is_questdb_nats "$svc"; then
    pgrep -f "collection-daemon|run_questdb_nats_writer" 2>/dev/null || true
  else
    port=$(_svc_port "$svc")
    if [[ -n "$port" ]]; then
      # Only the process listening on the port (server), not clients (e.g. TUI) connected to it
      lsof -i :"$port" 2>/dev/null | awk '/LISTEN/ {print $2}' | sort -u || true
    fi
  fi
}

_health_url() {
  local svc="$1"
  local url port
  url=$(_svc_health "$svc")
  port=$(_svc_port "$svc")
  if [[ -z "$url" ]]; then
    echo ""
    return
  fi
  echo "${url//\$\{PORT\}/$port}"
}

# Returns 0 if the service is really running (PID present and health check passes when available).
_service_really_running() {
  local svc="$1"
  local pid url code timeout
  pid=$(_find_pid "$svc")
  if [[ -z "$pid" ]]; then
    return 1
  fi
  url=$(_health_url "$svc")
  if [[ -z "$url" ]]; then
    # No health URL (e.g. memcached): PID present is enough
    return 0
  fi
  # IB service can be slow (gateway round-trip); use longer timeout
  timeout=2
  [[ "$svc" == "ib" ]] && timeout=5
  if [[ "$url" == https* ]]; then
    code=$(curl -sk --connect-timeout "$timeout" -o /dev/null -w "%{http_code}" "$url" 2>/dev/null || echo "000")
    # Gateway returns 401 when not logged in but is running; any response means process is up
    [[ "$code" != "000" ]]
    return $?
  fi
  curl -sf --connect-timeout "$timeout" "$url" >/dev/null 2>&1
  return $?
}

do_start() {
  local svc="$1"
  local display port log wait_sec cmd
  display=$(_svc_display "$svc")
  if ! _config_enabled "$svc" "true"; then
    echo "[info] ${display} is disabled in config; skipping start"
    return 0
  fi
  port=$(_svc_port "$svc")
  log="${LOGS_DIR}/$(_svc_log "$svc")"
  wait_sec=$(_svc_wait "$svc")
  cmd=$(_svc_cmd "$svc")

  local pid
  pid=$(_find_pid "$svc")
  if [[ -n "$pid" ]]; then
    if _service_really_running "$svc"; then
      echo "[info] ${display} already running (PID: ${pid//$'\n'/, }, healthy)"
      return 0
    fi
    # Port/process present but not responding – assume stale or wrong process; kill and restart
    if [[ -n "$port" ]]; then
      echo "[warn] ${display}: process on port ${port} not responding to health check; killing and restarting..."
    else
      echo "[warn] ${display}: process in use but not responding; killing and restarting..."
    fi
    if [[ -n "$port" ]] && command -v lsof >/dev/null 2>&1; then
      lsof -i :"$port" 2>/dev/null | awk '/LISTEN/ {print $2}' | sort -u | xargs kill -9 2>/dev/null || true
    else
      echo "$pid" | xargs kill -9 2>/dev/null || true
    fi
    sleep 2
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
  elif [[ "$svc" == "gateway" ]]; then
    (cd "${ROOT_DIR}" && ./ib-gateway/run-gateway.sh >> "$log" 2>&1) &
  elif [[ "$svc" == "web" ]]; then
    (cd "${ROOT_DIR}/web" && npm run dev > "$log" 2>&1) &
  else
    (cd "${ROOT_DIR}" && eval "$cmd" > "$log" 2>&1) &
  fi

  local svc_pid=$!
  disown "$svc_pid" 2>/dev/null || true

  # Short initial wait; if the run script exits 0 quickly (e.g. "already running"), check port now
  local early_sec=3
  sleep "$early_sec"
  if ! kill -0 "$svc_pid" 2>/dev/null; then
    wait "$svc_pid" 2>/dev/null
    local exitcode=$?
    if [[ "$exitcode" -eq 0 ]]; then
      pid=$(_find_pid "$svc")
      if [[ -n "$pid" ]] && _service_really_running "$svc"; then
        echo "[info] ${display} started (existing service, PID: ${pid//$'\n'/, }, healthy)"
        [[ -n "$port" ]] && echo "[info] Port: $port"
        echo "[info] Log: $log"
        return 0
      fi
    fi
  fi

  local remainder=$((wait_sec - early_sec))
  [[ "$remainder" -gt 0 ]] && sleep "$remainder"

  pid=$(_find_pid "$svc")
  if [[ -n "$pid" ]]; then
    if _service_really_running "$svc"; then
      echo "[info] ${display} started (PID: ${pid//$'\n'/, }, healthy)"
    else
      echo "[info] ${display} started (PID: ${pid//$'\n'/, }; health check not yet passing)"
    fi
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
  local display port
  display=$(_svc_display "$svc")
  port=$(_svc_port "$svc")

  local pid
  pid=$(_find_pid "$svc")
  if [[ -z "$pid" ]]; then
    echo "[info] ${display} is not running"
    return 0
  fi

  echo "[info] Stopping ${display} (PID: ${pid//$'\n'/, })..."
  # Kill all PIDs (may be newline-separated)
  echo "$pid" | xargs kill 2>/dev/null || true

  local i
  for i in 1 2 3 4 5 6 7 8 9 10; do
    pid=$(_find_pid "$svc")
    [[ -z "$pid" ]] && { echo "[info] ${display} stopped"; return 0; }
    sleep 0.5
  done

  echo "[warn] Force killing ${display}..."
  echo "$pid" | xargs kill -9 2>/dev/null || true
  sleep 1

  # Fallback: kill by port (listener only) so nothing is left listening
  if [[ -n "$port" ]] && command -v lsof >/dev/null 2>&1; then
    lsof -i :"$port" 2>/dev/null | awk '/LISTEN/ {print $2}' | sort -u | xargs kill -9 2>/dev/null || true
    sleep 0.5
  fi

  pid=$(_find_pid "$svc")
  if [[ -n "$pid" ]]; then
    echo "[error] Failed to stop ${display}"
    return 1
  fi
  echo "[info] ${display} stopped"
}

do_status() {
  local svc="$1"
  local display pid health_url health code timeout
  display=$(_svc_display "$svc")
  pid=$(_find_pid "$svc")
  if [[ -n "$pid" ]]; then
    health_url=$(_health_url "$svc")
    health="unknown"
    timeout=2
    [[ "$svc" == "ib" ]] && timeout=5
    if [[ -n "$health_url" ]]; then
      if [[ "$health_url" == https* ]]; then
        code=$(curl -sk --connect-timeout "$timeout" -o /dev/null -w "%{http_code}" "$health_url" 2>/dev/null || echo "000")
        [[ "$code" != "000" ]] && health="healthy" || health="unhealthy"
      else
        curl -sf --connect-timeout "$timeout" "$health_url" >/dev/null 2>&1 && health="healthy" || health="unhealthy"
      fi
    else
      health="no-check"
    fi
    echo "[info] ${display}: running (PID: ${pid//$'\n'/, }, health: $health)"
  else
    echo "[info] ${display}: stopped"
  fi
}

do_logs() {
  local svc="$1"
  local log
  log="${LOGS_DIR}/$(_svc_log "$svc")"
  if [[ -f "$log" ]]; then
    tail -50 "$log"
  else
    echo "[info] No log file found: $log"
  fi
}

do_restart() {
  local svc="$1"
  do_stop "$svc"
  do_start "$svc"
}

do_list() {
  echo "Available services:"
  local svc port enabled
  for svc in nats memcached gateway web rust; do
    port=$(_svc_port "$svc")
    enabled="enabled"
    _config_enabled "$svc" "true" || enabled="disabled"
    printf "  %-14s  %-20s  %s  [%s]\n" "$svc" "$(_svc_display "$svc")" "${port:+port $port}" "$enabled"
  done
}

# Service order for start-all (dependency order); stop-all uses reverse
# healthdashboard after backends so NATS has time to be ready
ALL_SERVICES_START=(nats memcached gateway web)

do_start_all() {
  local svc
  for svc in "${ALL_SERVICES_START[@]}"; do
    if ! _config_enabled "$svc" "true"; then
      echo "==> Skipping disabled $svc..."
      continue
    fi
    echo "==> Starting $svc..."
    do_start "$svc" || true
  done
  echo ""
  echo "==> All services started. Logs: ${LOGS_DIR}"
  echo "    Stop all: $0 stop-all"
  echo "    Status:   $0 status-all"
}

do_stop_all() {
  local i svc
  for ((i = ${#ALL_SERVICES_START[@]} - 1; i >= 0; i--)); do
    svc="${ALL_SERVICES_START[$i]}"
    echo "==> Stopping $svc..."
    do_stop "$svc" || true
  done
  echo ""
  echo "==> All services stopped."
}

do_restart_all() {
  do_stop_all
  sleep 2
  do_start_all
}

do_status_all() {
  local svc port pid health_url health code timeout
  echo ""
  echo "Service status (all):"
  echo "====================="
  for svc in "${ALL_SERVICES_START[@]}"; do
    port=$(_svc_port "$svc")
    pid=$(_find_pid "$svc")
    if ! _config_enabled "$svc" "true"; then
      printf "  %-14s  %-20s  disabled (port: %s)\n" "$svc" "$(_svc_display "$svc")" "${port:-—}"
    elif [[ -n "$pid" ]]; then
      health_url=$(_health_url "$svc")
      health="unknown"
      timeout=2
      [[ "$svc" == "ib" ]] && timeout=5
      if [[ -n "$health_url" ]]; then
        if [[ "$health_url" == https* ]]; then
          code=$(curl -sk --connect-timeout "$timeout" -o /dev/null -w "%{http_code}" "$health_url" 2>/dev/null || echo "000")
          [[ "$code" != "000" ]] && health="healthy" || health="unhealthy"
        else
          curl -sf --connect-timeout "$timeout" "$health_url" >/dev/null 2>&1 && health="healthy" || health="unhealthy"
        fi
      else
        health="no-check"
      fi
      printf "  %-14s  %-20s  running  (PID: %s, port: %s, health: %s)\n" "$svc" "$(_svc_display "$svc")" "${pid//$'\n'/, }" "${port:-—}" "$health"
    else
      printf "  %-14s  %-20s  stopped  (port: %s)\n" "$svc" "$(_svc_display "$svc")" "${port:-—}"
    fi
  done
  echo ""
}

ACTION="${1:-}"
SERVICE="${2:-}"

# Normalize action: start_all -> start-all
case "$ACTION" in
  start_all) ACTION="start-all" ;;
  stop_all)   ACTION="stop-all" ;;
  restart_all) ACTION="restart-all" ;;
  status_all) ACTION="status-all" ;;
esac

case "$ACTION" in
  list)
    do_list
    ;;
  start-all)
    do_start_all
    ;;
  stop-all)
    do_stop_all
    ;;
  restart-all)
    do_restart_all
    ;;
  status-all)
    do_status_all
    ;;
  start|stop|restart|status|logs)
    if [[ -z "$SERVICE" ]]; then
      echo "Usage: $0 $ACTION <service-name>"
      echo "Run '$0 list' to see available services."
      exit 1
    fi
    if ! _svc_known "$SERVICE"; then
      echo "[error] Unknown service: $SERVICE"
      do_list
      exit 1
    fi
    "do_${ACTION}" "$SERVICE"
    ;;
  *)
    echo "Usage: $0 {start|stop|restart|status|logs|list|start-all|stop-all|restart-all|status-all} [service-name]"
    echo ""
    echo "Single service: $0 {start|stop|restart|status|logs} <service>"
    echo "All services:   $0 {start-all|stop-all|restart-all|status-all}"
    echo "                (also accepted: start_all, stop_all, restart_all, status_all)"
    echo "List:           $0 list"
    exit 1
    ;;
esac
