#!/usr/bin/env bash
# shellcheck shell=sh
# Test suite for scripts/include/workspace_paths.sh
# Tests canonical path resolution and workspace cache/temp export setup.
#
# AI Context:
# - workspace_project_root() resolves the repo root via BASH_SOURCE or PROJECT_ROOT override
# - setup_workspace_paths() exports WORKSPACE_CACHE_ROOT, CARGO_HOME, XDG_CACHE_HOME, etc.
# - All exports use ${VAR:-default} so pre-set values are respected (idempotent)

if [ -z "${PROJECT_ROOT:-}" ]; then
  _TEST_DIR="${SHELLSPEC_SPECDIR:-spec}"
  PROJECT_ROOT="$(cd "${_TEST_DIR}/.." 2>/dev/null && pwd || pwd)"
fi

# Store real root for assertions before any overrides
_REAL_PROJECT_ROOT="${PROJECT_ROOT}"

Describe 'workspace_paths.sh - workspace_project_root()'

  before_each() {
    # Unset guard and override so each test loads fresh
    unset __IB_BOX_WORKSPACE_PATHS_INCLUDED
    unset PROJECT_ROOT
    # shellcheck source=scripts/include/workspace_paths.sh
    . "${_REAL_PROJECT_ROOT}/scripts/include/workspace_paths.sh"
  }

  It 'returns an absolute path'
    When call workspace_project_root
    The output should start with "/"
    The status should eq 0
  End

  It 'returned path contains the repo marker CLAUDE.md'
    When run sh -c "ROOT=\$(. '${_REAL_PROJECT_ROOT}/scripts/include/workspace_paths.sh' && workspace_project_root) && test -f \"\${ROOT}/CLAUDE.md\""
    The status should eq 0
  End

  It 'honours PROJECT_ROOT env var when set'
    export PROJECT_ROOT="/tmp/fake_root"
    When call workspace_project_root
    The output should eq "/tmp/fake_root"
  End

End

Describe 'workspace_paths.sh - setup_workspace_paths()'

  before_each() {
    unset __IB_BOX_WORKSPACE_PATHS_INCLUDED
    unset WORKSPACE_CACHE_ROOT
    unset WORKSPACE_TMP_ROOT
    unset BUILD_ARTIFACT_ROOT
    unset XDG_CACHE_HOME
    unset CARGO_HOME
    unset GOCACHE
    unset GOMODCACHE
    export PROJECT_ROOT="${_REAL_PROJECT_ROOT}"
    . "${_REAL_PROJECT_ROOT}/scripts/include/workspace_paths.sh"
  }

  It 'exports WORKSPACE_CACHE_ROOT as a non-empty string'
    When call setup_workspace_paths
    The variable WORKSPACE_CACHE_ROOT should be defined
    The variable WORKSPACE_CACHE_ROOT should not eq ""
  End

  It 'sets XDG_CACHE_HOME equal to WORKSPACE_CACHE_ROOT'
    setup_workspace_paths
    When run sh -c "[ \"\${XDG_CACHE_HOME}\" = \"\${WORKSPACE_CACHE_ROOT}\" ]"
    The status should eq 0
  End

  It 'exports CARGO_HOME under WORKSPACE_CACHE_ROOT'
    When run sh -c "
      export PROJECT_ROOT='${_REAL_PROJECT_ROOT}'
      unset CARGO_HOME WORKSPACE_CACHE_ROOT
      . '${_REAL_PROJECT_ROOT}/scripts/include/workspace_paths.sh'
      setup_workspace_paths
      case \"\${CARGO_HOME}\" in
        \"\${WORKSPACE_CACHE_ROOT}\"*) exit 0 ;;
        *) exit 1 ;;
      esac
    "
    The status should eq 0
  End

  It 'creates the WORKSPACE_CACHE_ROOT directory'
    WORKSPACE_CACHE_ROOT="${SHELLSPEC_TMPBASE}/test_cache"
    export WORKSPACE_CACHE_ROOT
    When call setup_workspace_paths
    The path "${SHELLSPEC_TMPBASE}/test_cache" should be directory
  End

  It 'exports GOCACHE under WORKSPACE_CACHE_ROOT'
    When run sh -c "
      export PROJECT_ROOT='${_REAL_PROJECT_ROOT}'
      unset GOCACHE WORKSPACE_CACHE_ROOT
      . '${_REAL_PROJECT_ROOT}/scripts/include/workspace_paths.sh'
      setup_workspace_paths
      case \"\${GOCACHE}\" in
        \"\${WORKSPACE_CACHE_ROOT}\"*) exit 0 ;;
        *) exit 1 ;;
      esac
    "
    The status should eq 0
  End

  It 'is idempotent — calling twice keeps original WORKSPACE_CACHE_ROOT'
    WORKSPACE_CACHE_ROOT="/tmp/fixed_cache"
    export WORKSPACE_CACHE_ROOT
    setup_workspace_paths
    setup_workspace_paths
    When run sh -c "[ \"\${WORKSPACE_CACHE_ROOT}\" = '/tmp/fixed_cache' ]"
    The status should eq 0
  End

End

Describe 'workspace_paths.sh - double-source guard'

  It 'sourcing twice does not cause errors or redefinition'
    When run sh -c "
      export PROJECT_ROOT='${_REAL_PROJECT_ROOT}'
      . '${_REAL_PROJECT_ROOT}/scripts/include/workspace_paths.sh'
      . '${_REAL_PROJECT_ROOT}/scripts/include/workspace_paths.sh'
      workspace_project_root
    "
    The status should eq 0
    The output should start with "/"
  End

End
