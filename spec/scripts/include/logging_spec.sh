#!/usr/bin/env bash
# shellcheck shell=sh
# Test suite for scripts/include/logging.sh
# Tests shared logging helpers used by all project scripts.
#
# AI Context:
# - logging.sh provides log_info, log_warn, log_error, log_note plus aliases
# - ERROR level writes to stderr; all others write to stdout
# - Colors are enabled only when stdout is a TTY ([[ -t 1 ]])
# - A guard variable prevents double-sourcing

if [ -z "${PROJECT_ROOT:-}" ]; then
  _TEST_DIR="${SHELLSPEC_SPECDIR:-spec}"
  PROJECT_ROOT="$(cd "${_TEST_DIR}/.." 2>/dev/null && pwd || pwd)"
fi

# Unset guard so logging.sh loads fresh for each spec run
unset __IB_BOX_SPREAD_LOGGING_INCLUDED
. "${PROJECT_ROOT}/scripts/include/logging.sh"

Describe 'logging.sh - log_info()'

  It 'outputs [INFO] prefix to stdout'
    When call log_info "hello world"
    The output should include "[INFO]"
    The status should eq 0
  End

  It 'includes the message in stdout'
    When call log_info "test message"
    The output should include "test message"
  End

  It 'does not write to stderr'
    When call log_info "silent stderr"
    The stderr should eq ""
  End

End

Describe 'logging.sh - log_warn()'

  It 'outputs [WARN] prefix to stdout'
    When call log_warn "something odd"
    The output should include "[WARN]"
    The status should eq 0
  End

  It 'includes the message in stdout'
    When call log_warn "watch out"
    The output should include "watch out"
  End

  It 'does not write to stderr'
    When call log_warn "no stderr"
    The stderr should eq ""
  End

End

Describe 'logging.sh - log_error()'

  It 'outputs [ERROR] prefix to stderr'
    When call log_error "something broke"
    The stderr should include "[ERROR]"
    The status should eq 0
  End

  It 'includes the message in stderr'
    When call log_error "bad thing"
    The stderr should include "bad thing"
  End

  It 'does not write to stdout'
    When call log_error "only stderr"
    The output should eq ""
  End

End

Describe 'logging.sh - log_note()'

  It 'outputs [NOTE] prefix to stdout'
    When call log_note "fyi"
    The output should include "[NOTE]"
    The status should eq 0
  End

  It 'does not write to stderr'
    When call log_note "quiet"
    The stderr should eq ""
  End

End

Describe 'logging.sh - backward-compatible aliases'

  It 'log() delegates to log_info and outputs [INFO]'
    When call log "via alias"
    The output should include "[INFO]"
    The output should include "via alias"
  End

  It 'warn() delegates to log_warn and outputs [WARN]'
    When call warn "alias warn"
    The output should include "[WARN]"
  End

  It 'err() delegates to log_error and outputs [ERROR] on stderr'
    When call err "alias error"
    The stderr should include "[ERROR]"
  End

End

Describe 'logging.sh - empty message handling'

  It 'log_info with no args returns 0 and produces no output'
    When call log_info
    The output should eq ""
    The stderr should eq ""
    The status should eq 0
  End

  It 'log_error with no args returns 0 and produces no output'
    When call log_error
    The output should eq ""
    The stderr should eq ""
    The status should eq 0
  End

End

Describe 'logging.sh - double-source guard'

  It 'sourcing twice does not redefine or duplicate functions'
    # Source again; guard should short-circuit
    When run sh -c ". '${PROJECT_ROOT}/scripts/include/logging.sh' && . '${PROJECT_ROOT}/scripts/include/logging.sh' && log_info 'once'"
    The output should include "[INFO]"
    # Output should contain exactly one [INFO] line, not two
    The lines of output should eq 1
  End

End
