#!/usr/bin/env bash
# Run all project linters (C++, Python, JS/TS, shell, Ansible, exarp-go, etc.).
#
# Usage:
#   ./scripts/run_linters.sh                    # normal (verbose)
#   ./scripts/run_linters.sh --ai-friendly      # quiet, log to logs/lint_ai_friendly.log, emit JSON
#   ./scripts/run_linters.sh --json-only        # same but print only JSON to stdout (for tools/AI)
#   LINT_MAX_LINES=80 ./scripts/run_linters.sh # truncate each linter output to 80 lines
#   LINT_QUIET=1 ./scripts/run_linters.sh      # same as LINT_MAX_LINES=80
#   ./scripts/run_linters.sh --parallel        # run independent linters in parallel (faster)
#   LINT_PARALLEL=1 ./scripts/run_linters.sh   # same as --parallel
#
# ansible-lint: ANSIBLE_LINT_TIMEOUT (default 300s); use timeout(1)/gtimeout(1) when available.
#
# Caches (faster re-runs): cppcheck (.cppcheck-cache/), ESLint (web/.eslintcache),
# Stylelint (web/.stylelintcache), SwiftLint (desktop/.swiftlint-cache), Infer (build/*/infer-out).
# Set LINT_INFER_FULL=1 to force a full Infer run (delete infer-out first).
#
# When run from a non-interactive context (no TTY, CI, or CURSOR), AI-friendly
# mode is enabled automatically (quiet + JSON only to stdout).
#
# AI-friendly output (single JSON object):
#   {"success": true|false, "exit_code": N, "duration_sec": F, "log_path": "...", "errors": ["..."]}
#
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/with_nix.sh
. "${SCRIPT_DIR}/with_nix.sh"
PROJECT_ROOT="${ROOT_DIR}"

# Parse --ai-friendly and --json-only before Nix re-exec so they work with USE_NIX=1
LINT_AI_FRIENDLY=0
LINT_JSON_ONLY=0
LINT_FILTERED_ARGS=()
for a in "$@"; do
  case "${a}" in
    --ai-friendly) LINT_AI_FRIENDLY=1 ;;
    --json-only)  LINT_JSON_ONLY=1; LINT_AI_FRIENDLY=1 ;;
    --parallel)   LINT_PARALLEL=1 ;;
    *)            LINT_FILTERED_ARGS+=("${a}") ;;
  esac
done
# LINT_PARALLEL can also be set by env (e.g. LINT_PARALLEL=1)
[[ -n "${LINT_PARALLEL:-}" ]] && [[ "${LINT_PARALLEL}" != "0" ]] && LINT_PARALLEL=1 || LINT_PARALLEL="${LINT_PARALLEL:-0}"
export LINT_AI_FRIENDLY LINT_JSON_ONLY LINT_PARALLEL

if [[ ${#LINT_FILTERED_ARGS[@]} -gt 0 ]]; then
  run_with_nix_if_requested "${LINT_FILTERED_ARGS[@]}"
else
  run_with_nix_if_requested
fi

# Detect non-interactive / AI tool context: enable AI-friendly + JSON-only if not already set.
if [[ "${LINT_AI_FRIENDLY}" -eq 0 ]]; then
  if [[ ! -t 1 ]]; then
    LINT_AI_FRIENDLY=1
    LINT_JSON_ONLY=1
    export LINT_AI_FRIENDLY LINT_JSON_ONLY
  elif [[ -n "${CI:-}" ]] && [[ "${CI}" != "0" ]]; then
    LINT_AI_FRIENDLY=1
    LINT_JSON_ONLY=1
    export LINT_AI_FRIENDLY LINT_JSON_ONLY
  elif [[ -n "${CURSOR:-}" ]] && [[ "${CURSOR}" != "0" ]]; then
    LINT_AI_FRIENDLY=1
    LINT_JSON_ONLY=1
    export LINT_AI_FRIENDLY LINT_JSON_ONLY
  fi
fi

# Limit output: set LINT_MAX_LINES to cap each linter's stdout (e.g. 80). 0 = no limit.
# LINT_QUIET=1 sets LINT_MAX_LINES=80 if unset.
# Usage: LINT_MAX_LINES=80 ./scripts/run_linters.sh   or   LINT_QUIET=1 ./scripts/run_linters.sh
[[ -n "${LINT_QUIET:-}" ]] && [[ -z "${LINT_MAX_LINES:-}" ]] && LINT_MAX_LINES=80
LINT_MAX_LINES="${LINT_MAX_LINES:-0}"

# ansible-lint can be slow (Galaxy/collections). Allow up to this many seconds; 0 = no limit.
ANSIBLE_LINT_TIMEOUT="${ANSIBLE_LINT_TIMEOUT:-300}"

info() {
  printf '\n\033[1m==> %s\033[0m\n' "$1"
}

warn() {
  printf '\033[33m[warn]\033[0m %s\n' "$1"
}

err() {
  printf '\033[31m[error]\033[0m %s\n' "$1" >&2
}

# Run command, optionally truncating stdout+stderr to first LINT_MAX_LINES lines.
run_limited() {
  local tmp
  tmp=$(mktemp)
  if "${@}" > "${tmp}" 2>&1; then
    local ret=0
  else
    local ret=$?
  fi
  if [[ "${LINT_MAX_LINES}" -gt 0 ]]; then
    head -n "${LINT_MAX_LINES}" "${tmp}"
    local lines
    lines=$(wc -l < "${tmp}" | tr -d ' ')
    if [[ "${lines}" -gt "${LINT_MAX_LINES}" ]]; then
      echo "[truncated; ${lines} lines total. Unset LINT_MAX_LINES for full output.]"
    fi
  else
    cat "${tmp}"
  fi
  rm -f "${tmp}"
  return "${ret}"
}

run_cppcheck() {
  if ! command -v cppcheck >/dev/null 2>&1; then
    warn "Skipping cppcheck (executable not found)"
    return 0
  fi

  info "Running cppcheck (C++ core)"
  local cpp_src="${ROOT_DIR}/native/src"
  local cpp_include="${ROOT_DIR}/native/include"
  local cpp_tests="${ROOT_DIR}/native/tests"
  if [ ! -d "${cpp_src}" ]; then
    cpp_src="${ROOT_DIR}/src"
    cpp_include="${ROOT_DIR}/include"
    cpp_tests="${ROOT_DIR}/tests"
  fi
  local cppcheck_cache="${ROOT_DIR}/.cppcheck-cache"
  mkdir -p "${cppcheck_cache}"
  if [[ "${LINT_MAX_LINES}" -gt 0 ]]; then
    run_limited cppcheck \
      --cppcheck-build-dir="${cppcheck_cache}" \
      --enable=warning,performance,style,portability \
      --std=c++17 \
      --suppress=missingIncludeSystem \
      --inline-suppr \
      --force \
      "${cpp_src}" \
      "${cpp_include}" \
      "${cpp_tests}"
  else
    cppcheck \
      --cppcheck-build-dir="${cppcheck_cache}" \
      --enable=warning,performance,style,portability \
      --std=c++17 \
      --suppress=missingIncludeSystem \
      --inline-suppr \
      --force \
      "${cpp_src}" \
      "${cpp_include}" \
      "${cpp_tests}"
  fi
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
    warn "Skipping clang --analyze (compile_commands.json not found). Configure with: cmake -S . -B build -G Ninja -DCMAKE_EXPORT_COMPILE_COMMANDS=ON"
    return 0
  fi

  local status=0

  while IFS= read -r entry; do
    local file dir
    file=$(printf '%s' "${entry}" | jq -r '.file')
    dir=$(printf '%s' "${entry}" | jq -r '.directory')

    case "${file}" in
    "${ROOT_DIR}/native/src/"* | "${ROOT_DIR}/native/tests/"* | "${ROOT_DIR}/native/include/"* | "${ROOT_DIR}/src/"* | "${ROOT_DIR}/tests/"* | "${ROOT_DIR}/include/"*) ;;
    *)
      continue
      ;;
    esac

    if [ ! -f "${file}" ]; then
      continue
    fi

    if jq -e '.arguments' >/dev/null 2>&1 <<<"${entry}"; then
      args=()
      while IFS= read -r line; do args+=("$line"); done < <(printf '%s' "${entry}" | jq -r '.arguments[]')
    else
      cmd=$(printf '%s' "${entry}" | jq -r '.command // empty')
      if [ -z "${cmd}" ]; then
        warn "Skipping ${file} (missing command information in compile_commands.json)"
        status=1
        continue
      fi
      args=()
      while IFS= read -r line; do args+=("$line"); done < <(
        CLANG_ANALYZE_CMD="${cmd}" python3 - <<'PY'
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
    if [[ ${compiler} == *++* ]]; then
      analyzer="clang++"
    fi
    if ! command -v "${analyzer}" >/dev/null 2>&1; then
      analyzer="clang++"
    fi

    local filtered=()
    local skip_next=0
    for ((i = 1; i < ${#args[@]}; i++)); do
      if [ "${skip_next}" -eq 1 ]; then
        skip_next=0
        continue
      fi
      case "${args[i]}" in
      -o | -MF | -MT | -MQ)
        skip_next=1
        continue
        ;;
      -MD | -MMD)
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

  if [[ ! -d "${ROOT_DIR}/desktop" ]]; then
    warn "Skipping swiftlint (desktop/ not found)"
    return 0
  fi

  info "Running swiftlint (macOS app)"
  local cache_path="${ROOT_DIR}/desktop/.swiftlint-cache"
  (cd "${ROOT_DIR}/desktop" && swiftlint --cache-path "${cache_path}")
}

# Go TUI removed - using C++ TUI instead
# run_golangci_lint() {
#   if ! command -v golangci-lint >/dev/null 2>&1; then
#     warn "Skipping golangci-lint (executable not found)"
#     return 0
#   fi
#
#   info "Running golangci-lint (TUI)"
#   (cd "${ROOT_DIR}/tui" && golangci-lint run)
# }

run_eslint() {
  info "Running ESLint (React/TypeScript/JSON web frontend)"
  if ! command -v npm >/dev/null 2>&1; then
    warn "Skipping ESLint (npm not found)"
    return 0
  fi

  if [ ! -f "${ROOT_DIR}/web/package.json" ]; then
    warn "Skipping ESLint (web/package.json not found)"
    return 0
  fi

  if [[ "${LINT_MAX_LINES}" -gt 0 ]]; then
    run_limited bash -c "cd '${ROOT_DIR}/web' && npm run lint" || {
      warn "ESLint found issues. Run 'cd web && npm run lint:fix' to auto-fix some issues."
      return 1
    }
  else
    (cd "${ROOT_DIR}/web" && npm run lint) || {
      warn "ESLint found issues. Run 'cd web && npm run lint:fix' to auto-fix some issues."
      return 1
    }
  fi
}

run_stylelint() {
  if ! command -v npm >/dev/null 2>&1; then
    warn "Skipping stylelint (npm not found)"
    return 0
  fi

  if [ ! -f "${ROOT_DIR}/web/package.json" ]; then
    warn "Skipping stylelint (web/package.json not found)"
    return 0
  fi

  info "Running stylelint (CSS web frontend)"
  if [[ "${LINT_MAX_LINES}" -gt 0 ]]; then
    run_limited bash -c "cd '${ROOT_DIR}/web' && npm run lint:css" || {
      warn "stylelint found issues. Run 'cd web && npm run lint:css:fix' to auto-fix some issues."
      return 1
    }
  else
    (cd "${ROOT_DIR}/web" && npm run lint:css) || {
      warn "stylelint found issues. Run 'cd web && npm run lint:css:fix' to auto-fix some issues."
      return 1
    }
  fi
}

run_type_check() {
  if ! command -v npm >/dev/null 2>&1; then
    warn "Skipping TypeScript type check (npm not found)"
    return 0
  fi

  if [ ! -f "${ROOT_DIR}/web/package.json" ]; then
    warn "Skipping TypeScript type check (web/package.json not found)"
    return 0
  fi

  info "Running TypeScript type check (tsc --noEmit)"
  (cd "${ROOT_DIR}/web" && npm run type-check) || {
    warn "TypeScript type check found errors. Fix type errors before committing."
    return 1
  }
}

run_js_syntax_check() {
  if [ -f "${ROOT_DIR}/scripts/check_javascript.sh" ]; then
    "${ROOT_DIR}/scripts/check_javascript.sh" || return 1
  else
    warn "Skipping JavaScript syntax check (check_javascript.sh not found)"
    return 0
  fi
}

run_shellcheck() {
  if [[ -n "${EXARP_RAN_SHELLCHECK:-}" ]]; then
    if [[ -f "${ROOT_DIR}/ansible/run-dev-setup.sh" ]] && command -v shellcheck >/dev/null 2>&1; then
      info "Running shellcheck (ansible/run-dev-setup.sh; scripts/ already linted by exarp-go)"
      shellcheck "${ROOT_DIR}/ansible/run-dev-setup.sh" || return 1
    fi
    return 0
  fi
  if ! command -v shellcheck >/dev/null 2>&1; then
    warn "Skipping shellcheck (executable not found)"
    return 0
  fi
  info "Running shellcheck (shell scripts)"
  local script_dir="${ROOT_DIR}/scripts"
  local failed=0
  while IFS= read -r -d '' f; do
    if ! shellcheck "$f"; then
      failed=1
    fi
  done < <(find "${script_dir}" -maxdepth 1 -name '*.sh' -print0 2>/dev/null)
  if [[ -f "${ROOT_DIR}/ansible/run-dev-setup.sh" ]]; then
    if ! shellcheck "${ROOT_DIR}/ansible/run-dev-setup.sh"; then
      failed=1
    fi
  fi
  [ "${failed}" -eq 0 ]
}

run_ansible_lint() {
  if ! command -v ansible-lint >/dev/null 2>&1; then
    warn "Skipping ansible-lint (executable not found; install with: pip install ansible-lint or uv tool install ansible-lint)"
    return 0
  fi
  info "Running ansible-lint (ansible/ playbooks and roles)"
  local timeout_cmd=()
  if [[ "${ANSIBLE_LINT_TIMEOUT}" -gt 0 ]]; then
    if command -v timeout >/dev/null 2>&1; then
      timeout_cmd=(timeout "${ANSIBLE_LINT_TIMEOUT}")
    elif command -v gtimeout >/dev/null 2>&1; then
      timeout_cmd=(gtimeout "${ANSIBLE_LINT_TIMEOUT}")
    fi
  fi
  if [[ "${LINT_MAX_LINES}" -gt 0 ]]; then
    run_limited "${timeout_cmd[@]}" ansible-lint ansible/ || return 1
  else
    "${timeout_cmd[@]}" ansible-lint ansible/ || return 1
  fi
}

run_cmake_lint() {
  if ! command -v cmake-lint >/dev/null 2>&1; then
    warn "Skipping cmake-lint (executable not found; install with: pip install cmakelang or uv tool install cmakelang)"
    return 0
  fi
  info "Running cmake-lint (CMakeLists.txt and cmake files)"
  local files=(CMakeLists.txt native/CMakeLists.txt native/tests/CMakeLists.txt native/ibapi_cmake/CMakeLists.txt)
  if [[ "${LINT_MAX_LINES}" -gt 0 ]]; then
    run_limited cmake-lint "${files[@]}" || return 1
  else
    cmake-lint "${files[@]}" || return 1
  fi
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

  if [[ "${LINT_MAX_LINES}" -gt 0 ]]; then
    run_limited "${bandit_cmd[@]}" -r "${ROOT_DIR}/python" "${ROOT_DIR}/agents/backend/python"
  else
    "${bandit_cmd[@]}" -r "${ROOT_DIR}/python" "${ROOT_DIR}/agents/backend/python"
  fi
}

run_ruff() {
  info "Running ruff (Python)"

  local ruff_cmd=()
  if command -v uv >/dev/null 2>&1 && [[ -f "${ROOT_DIR}/python/pyproject.toml" || -f "${ROOT_DIR}/pyproject.toml" ]]; then
    ruff_cmd=(uv run ruff)
  elif command -v ruff >/dev/null 2>&1; then
    ruff_cmd=(ruff)
  else
    warn "Skipping ruff (uv/ruff not found; install with: uv pip install ruff or pip install ruff)"
    return 0
  fi

  if [[ "${LINT_MAX_LINES}" -gt 0 ]]; then
    run_limited "${ruff_cmd[@]}" check "${ROOT_DIR}/python"
  else
    "${ruff_cmd[@]}" check "${ROOT_DIR}/python"
  fi
}

run_infer() {
  if ! command -v infer >/dev/null 2>&1; then
    warn "Skipping Infer (executable not found)"
    warn "Install Infer: brew install infer or see https://fbinfer.com/docs/getting-started"
    return 0
  fi

  info "Running Infer (C++ static analysis)"

  # Search for compile_commands.json in common build directories
  local compile_db=""
  local build_dir=""

  # Check common build directory locations
  for possible_dir in \
    "${ROOT_DIR}/build/macos-arm64-debug" \
    "${ROOT_DIR}/build/macos-arm64-release" \
    "${ROOT_DIR}/build/macos-x86_64-debug" \
    "${ROOT_DIR}/build/macos-x86_64-release" \
    "${ROOT_DIR}/build/macos-universal-debug" \
    "${ROOT_DIR}/build/macos-universal-release" \
    "${ROOT_DIR}/build/linux-x64-debug" \
    "${ROOT_DIR}/build/linux-x64-release" \
    "${ROOT_DIR}/build"; do
    if [ -f "${possible_dir}/compile_commands.json" ]; then
      compile_db="${possible_dir}/compile_commands.json"
      build_dir="${possible_dir}"
      break
    fi
  done

  if [ -z "${compile_db}" ] || [ ! -f "${compile_db}" ]; then
    warn "Skipping Infer (compile_commands.json not found). Configure with: cmake -S . -B build -G Ninja -DCMAKE_EXPORT_COMPILE_COMMANDS=ON"
    return 0
  fi

  local infer_out_dir="${build_dir}/infer-out"

  # Keep infer-out for incremental runs unless LINT_INFER_FULL=1
  if [[ -n "${LINT_INFER_FULL:-}" ]] && [[ "${LINT_INFER_FULL}" != "0" ]]; then
    if [ -d "${infer_out_dir}" ]; then
      rm -rf "${infer_out_dir}"
    fi
  fi

  # Infer needs to run from the build directory with compile_commands.json
  # Use infer run with --compilation-database to analyze all files
  # RacerD is enabled by default for C++ code (thread safety analysis)
  (
    cd "${build_dir}" || exit 1
    infer run \
      --compilation-database compile_commands.json \
      --compilation-database-escaped \
      --racerd \
      --no-progress-bar \
      --quiet
  ) || {
    warn "Infer found issues. Check ${infer_out_dir}/report.json for details."
    warn "View report: infer-explore --html ${infer_out_dir}"
    warn "For thread safety issues, see: https://fbinfer.com/docs/checker-racerd"
    return 1
  }

  info "Infer analysis completed. Results in ${infer_out_dir}/"
  info "RacerD (thread safety) analysis included. For details: https://fbinfer.com/docs/checker-racerd"
}

run_exarp_go_lint() {
  local exarp_script="${ROOT_DIR}/scripts/run_exarp_go_tool.sh"
  if [[ ! -x "${exarp_script}" ]]; then
    warn "Skipping exarp-go lint (run_exarp_go_tool.sh not executable)"
    return 0
  fi
  if ! "${exarp_script}" --list &>/dev/null; then
    warn "Skipping exarp-go lint (exarp-go not found or not in PATH)"
    return 0
  fi
  info "Running exarp-go lint (Go + shell)"
  local status=0
  "${exarp_script}" lint || status=0
  "${exarp_script}" lint '{"linter":"shellcheck","path":"scripts"}' || status=0
  if [[ ${status} -ne 0 ]]; then
    warn "exarp-go lint reported issues (optional; install exarp-go for full coverage)"
  fi
  export EXARP_RAN_SHELLCHECK=1
  return 0
}

main() {
  if [[ "${LINT_PARALLEL}" -eq 1 ]]; then
    main_parallel
    return $?
  fi
  run_cppcheck
  run_clang_analyze
  run_infer
  run_swiftlint
  # run_golangci_lint  # Go TUI removed - using C++ TUI instead
  run_exarp_go_lint
  run_shellcheck
  run_ansible_lint
  run_cmake_lint
  run_bandit
  run_ruff
  run_eslint
  run_stylelint
  run_type_check
  run_js_syntax_check

  info "Lint checks completed"
}

# Run independent linters in parallel; then exarp + shellcheck (order preserved for EXARP_RAN_SHELLCHECK).
main_parallel() {
  local tmpdir
  tmpdir=$(mktemp -d)
  trap 'rm -rf "${tmpdir}"' EXIT
  local status=0

  ( run_cppcheck; echo $? > "${tmpdir}/cppcheck" ) &
  ( run_clang_analyze; echo $? > "${tmpdir}/clang_analyze" ) &
  ( run_infer; echo $? > "${tmpdir}/infer" ) &
  ( run_swiftlint; echo $? > "${tmpdir}/swiftlint" ) &
  ( run_bandit; echo $? > "${tmpdir}/bandit" ) &
  ( run_ruff; echo $? > "${tmpdir}/ruff" ) &
  ( run_ansible_lint; echo $? > "${tmpdir}/ansible_lint" ) &
  ( run_cmake_lint; echo $? > "${tmpdir}/cmake_lint" ) &
  ( run_eslint; echo $? > "${tmpdir}/eslint" ) &
  ( run_stylelint; echo $? > "${tmpdir}/stylelint" ) &
  ( run_type_check; echo $? > "${tmpdir}/type_check" ) &
  ( run_js_syntax_check; echo $? > "${tmpdir}/js_syntax_check" ) &

  wait

  for f in "${tmpdir}"/*; do
    [[ -f "${f}" ]] && [[ "$(cat "${f}")" -ne 0 ]] && status=1
  done

  run_exarp_go_lint || status=1
  run_shellcheck || status=1

  info "Lint checks completed"
  return "${status}"
}

# Extract error/warning lines from lint log for JSON (first 50).
extract_lint_errors() {
  local log="$1"
  if [[ ! -f "${log}" ]]; then
    echo "[]"
    return
  fi
  local errs
  errs="$(grep -E "error:|warning:|\[error\]|\[warn\].*found|FAILED|Error:|fatal" "${log}" 2>/dev/null | head -50)"
  if [[ -z "${errs}" ]]; then
    errs="$(grep -E "^\s*[0-9]+ (error|warning)" "${log}" 2>/dev/null | head -50)"
  fi
  if [[ -z "${errs}" ]]; then
    errs="$(tail -30 "${log}" 2>/dev/null)"
  fi
  if [[ -z "${errs}" ]]; then
    echo "[]"
    return
  fi
  if command -v jq >/dev/null 2>&1; then
    echo "${errs}" | jq -R -s -c 'split("\n") | map(select(length > 0))'
  else
    echo "${errs}" | awk 'BEGIN { first=1; printf "[" }
      { gsub(/\\/,"\\\\"); gsub(/"/,"\\\""); gsub(/\t/," "); if (!first) printf ","; first=0; printf "\""; for(i=1;i<=NF;i++){if(i>1)printf " "; printf "%s",$i}; printf "\"" }
      END { printf "]" }'
  fi
}

if [[ "${LINT_AI_FRIENDLY}" -eq 1 ]]; then
  LOG_DIR="${ROOT_DIR}/logs"
  mkdir -p "${LOG_DIR}"
  LINT_LOG="${LOG_DIR}/lint_ai_friendly.log"
  START="$(date +%s.%N 2>/dev/null || echo 0)"
  ( main ) >> "${LINT_LOG}" 2>&1
  lint_exit=$?
  END="$(date +%s.%N 2>/dev/null || echo 0)"
  DURATION="$(awk "BEGIN { printf \"%.2f\", ${END} - ${START} }" 2>/dev/null || echo "0")"
  ERRORS_JSON="$(extract_lint_errors "${LINT_LOG}")"
  SUCCESS="false"
  [[ ${lint_exit} -eq 0 ]] && SUCCESS="true"
  if [[ "${LINT_JSON_ONLY}" -eq 1 ]]; then
    echo "{\"success\":${SUCCESS},\"exit_code\":${lint_exit},\"duration_sec\":${DURATION},\"log_path\":\"${LINT_LOG}\",\"errors\":${ERRORS_JSON}}"
  else
    echo "{\"success\":${SUCCESS},\"exit_code\":${lint_exit},\"duration_sec\":${DURATION},\"log_path\":\"${LINT_LOG}\",\"errors\":${ERRORS_JSON}}"
    if [[ ${lint_exit} -ne 0 ]]; then
      echo "" 1>&2
      echo "Lint failed. Log: ${LINT_LOG}" 1>&2
      echo "Last 20 lines:" 1>&2
      tail -20 "${LINT_LOG}" 1>&2
    fi
  fi
  exit "${lint_exit}"
fi

main
