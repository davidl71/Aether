#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

info() {
  printf '\n\033[1m==> %s\033[0m\n' "$1"
}

warn() {
  printf '\033[33m[warn]\033[0m %s\n' "$1"
}

err() {
  printf '\033[31m[error]\033[0m %s\n' "$1" >&2
}

run_cppcheck() {
  if ! command -v cppcheck >/dev/null 2>&1; then
    warn "Skipping cppcheck (executable not found)"
    return 0
  fi

  info "Running cppcheck (C++ core)"
  cppcheck \
    --enable=warning,performance,style,portability \
    --std=c++17 \
    --suppress=missingIncludeSystem \
    --inline-suppr \
    --force \
    "${ROOT_DIR}/src" \
    "${ROOT_DIR}/include" \
    "${ROOT_DIR}/tests"
}

run_clang_analyze() {
  if ! command -v clang >/dev/null 2>&1; then
    warn "Skipping clang --analyze (clang not found)"
    return 0
  fi

  info "Running clang --analyze (C++ core)"

  local build_dir="${ROOT_DIR}/build"
  local compile_db="${build_dir}/compile_commands.json"

  if [ ! -f "${compile_db}" ]; then
    err "compile_commands.json not found. Configure the project with CMake (e.g. ninja generator) and enable CMAKE_EXPORT_COMPILE_COMMANDS."
    return 1
  fi

  local status=0

  while IFS= read -r entry; do
    local file dir
    file=$(printf '%s' "${entry}" | jq -r '.file')
    dir=$(printf '%s' "${entry}" | jq -r '.directory')

    case "${file}" in
      "${ROOT_DIR}/src/"*| "${ROOT_DIR}/tests/"*| "${ROOT_DIR}/include/"*)
        ;;
      *)
        continue
        ;;
    esac

    if [ ! -f "${file}" ]; then
      continue
    fi

    if jq -e '.arguments' >/dev/null 2>&1 <<<"${entry}"; then
      mapfile -t args < <(printf '%s' "${entry}" | jq -r '.arguments[]')
    else
      cmd=$(printf '%s' "${entry}" | jq -r '.command // empty')
      if [ -z "${cmd}" ]; then
        warn "Skipping ${file} (missing command information in compile_commands.json)"
        status=1
        continue
      fi
      mapfile -t args < <(CLANG_ANALYZE_CMD="${cmd}" python3 - <<'PY'
import os, shlex
cmd = os.environ.get("CLANG_ANALYZE_CMD", "")
for token in shlex.split(cmd):
    print(token)
PY
)
    fi

    if [ "${#args[@]}" -eq 0 ]; then
      continue
    fi

    local compiler="${args[0]}"
    local analyzer="clang"
    if [[ "${compiler}" == *++* ]]; then
      analyzer="clang++"
    fi
    if ! command -v "${analyzer}" >/dev/null 2>&1; then
      analyzer="clang++"
    fi

    local filtered=()
    local skip_next=0
    for ((i=1; i<${#args[@]}; i++)); do
      if [ "${skip_next}" -eq 1 ]; then
        skip_next=0
        continue
      fi
      case "${args[i]}" in
        -o|-MF|-MT|-MQ)
          skip_next=1
          continue
          ;;
        -MD|-MMD)
          continue
          ;;
      esac
      filtered+=("${args[i]}")
    done

    (
      cd "${dir}" || exit 1
      "${analyzer}" --analyze "${filtered[@]}"
    ) || status=1
  done < <(jq -c '.[]' "${compile_db}")

  return "${status}"
}

run_swiftlint() {
  if ! command -v swiftlint >/dev/null 2>&1; then
    warn "Skipping swiftlint (executable not found)"
    return 0
  fi

  info "Running swiftlint (macOS app)"
  (cd "${ROOT_DIR}/desktop" && swiftlint)
}

run_golangci_lint() {
  if ! command -v golangci-lint >/dev/null 2>&1; then
    warn "Skipping golangci-lint (executable not found)"
    return 0
  fi

  info "Running golangci-lint (TUI)"
  (cd "${ROOT_DIR}/tui" && golangci-lint run)
}

run_bandit() {
  info "Running bandit (Python)"

  local bandit_cmd=()

  if [ -x "${ROOT_DIR}/.bandit-env/bin/bandit" ]; then
    bandit_cmd=("${ROOT_DIR}/.bandit-env/bin/bandit")
  elif command -v bandit >/dev/null 2>&1; then
    bandit_cmd=(bandit)
  else
    err "Bandit not available. Create .bandit-env (Python 3.10) or install bandit globally."
    return 1
  fi

  "${bandit_cmd[@]}" -r "${ROOT_DIR}/python" "${ROOT_DIR}/agents/backend/python"
}

main() {
  run_cppcheck
  run_clang_analyze
  run_swiftlint
  run_golangci_lint
  run_bandit

  info "Lint checks completed"
}

main "$@"

