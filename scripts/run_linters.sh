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
  local cpp_src="${ROOT_DIR}/native/src"
  local cpp_include="${ROOT_DIR}/native/include"
  local cpp_tests="${ROOT_DIR}/native/tests"
  if [ ! -d "${cpp_src}" ]; then
    cpp_src="${ROOT_DIR}/src"
    cpp_include="${ROOT_DIR}/include"
    cpp_tests="${ROOT_DIR}/tests"
  fi
  cppcheck \
    --enable=warning,performance,style,portability \
    --std=c++17 \
    --suppress=missingIncludeSystem \
    --inline-suppr \
    --force \
    "${cpp_src}" \
    "${cpp_include}" \
    "${cpp_tests}"
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
      mapfile -t args < <(printf '%s' "${entry}" | jq -r '.arguments[]')
    else
      cmd=$(printf '%s' "${entry}" | jq -r '.command // empty')
      if [ -z "${cmd}" ]; then
        warn "Skipping ${file} (missing command information in compile_commands.json)"
        status=1
        continue
      fi
      mapfile -t args < <(
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

  info "Running swiftlint (macOS app)"
  (cd "${ROOT_DIR}/desktop" && swiftlint)
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

  (cd "${ROOT_DIR}/web" && npm run lint) || {
    warn "ESLint found issues. Run 'cd web && npm run lint:fix' to auto-fix some issues."
    return 1
  }
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
  (cd "${ROOT_DIR}/web" && npm run lint:css) || {
    warn "stylelint found issues. Run 'cd web && npm run lint:css:fix' to auto-fix some issues."
    return 1
  }
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
  [ "${failed}" -eq 0 ]
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

  # Clean previous Infer results
  if [ -d "${infer_out_dir}" ]; then
    rm -rf "${infer_out_dir}"
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
  info "Running exarp-go lint"
  "${exarp_script}" lint || {
    warn "exarp-go lint reported issues (optional; install exarp-go for full coverage)"
    return 0
  }
}

main() {
  run_cppcheck
  run_clang_analyze
  run_infer
  run_swiftlint
  # run_golangci_lint  # Go TUI removed - using C++ TUI instead
  run_exarp_go_lint
  run_shellcheck
  run_bandit
  run_eslint
  run_stylelint
  run_type_check
  run_js_syntax_check

  info "Lint checks completed"
}

main "$@"
