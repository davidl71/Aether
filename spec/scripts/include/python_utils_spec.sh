#!/usr/bin/env bash
#shellcheck shell=sh
# Test suite for scripts/include/python_utils.sh
# Tests Python utility functions used across service scripts
#
# AI Context:
# - This file tests shared Python management functions
# - Functions are sourced from scripts/include/python_utils.sh
# - These functions are used by all service startup scripts
# - Tests use ShellSpec BDD syntax for clarity

# Source the functions we're testing
# AI Note: PROJECT_ROOT is set in spec_helper.sh (loaded via --require in .shellspec)
# We source at the top level so functions are available to all test cases
if [ -z "${PROJECT_ROOT:-}" ]; then
  _TEST_DIR="${SHELLSPEC_SPECDIR:-spec}"
  PROJECT_ROOT="$(cd "${_TEST_DIR}/.." 2>/dev/null && pwd || pwd)"
fi
. "${PROJECT_ROOT}/scripts/include/python_utils.sh"

Describe 'python_utils.sh - Python utility functions'

  Describe 'find_python()'
    # AI Context: This function detects available Python interpreter
    # It sets PYTHON_CMD variable and returns 0 if found, 1 if not found
    # Used by all service scripts before setting up virtual environments

    It 'finds python3 when available'
      # Mock command to simulate python3 availability
      Mock command
        case "$1" in
          python3) return 0 ;;
          python) return 1 ;;
        esac
      End

      When call find_python
      The status should eq 0
      The variable PYTHON_CMD should eq "python3"
    End

    It 'falls back to python when python3 not available'
      Mock command
        case "$1" in
          python3) return 1 ;;
          python) return 0 ;;
        esac
      End

      When call find_python
      The status should eq 0
      The variable PYTHON_CMD should eq "python"
    End

    It 'returns error when no Python found'
      Mock command
        return 1
      End

      When call find_python
      The status should eq 1
      The variable PYTHON_CMD should be undefined
      The stderr should include "Error: Python not found"
    End
  End

  Describe 'setup_venv()'
    # AI Context: Creates/activates Python virtual environment
    # Sets VENV_DIR, ACTIVATE_PATH, and VENV_PYTHON variables
    # Used by all service scripts to isolate dependencies

    It 'creates venv when it does not exist'
      # Setup: Mock Python and file system
      PYTHON_CMD="python3"
      TEST_DIR="${SHELLSPEC_TMPBASE}/test_venv"
      mkdir -p "${TEST_DIR}"

      Mock python3
        case "$1" in
          -m) [ "$2" = "venv" ] && mkdir -p "$4/bin" && touch "$4/bin/activate" ;;
        esac
      End

      When call setup_venv "${TEST_DIR}"
      The status should eq 0
      The variable VENV_DIR should eq "${TEST_DIR}/.venv"
      The variable ACTIVATE_PATH should eq "${TEST_DIR}/.venv/bin/activate"
      The variable VENV_PYTHON should eq "${TEST_DIR}/.venv/bin/python"
    End

    It 'uses existing venv when present'
      PYTHON_CMD="python3"
      TEST_DIR="${SHELLSPEC_TMPBASE}/test_venv_existing"
      mkdir -p "${TEST_DIR}/.venv/bin"
      touch "${TEST_DIR}/.venv/bin/activate"

      When call setup_venv "${TEST_DIR}"
      The status should eq 0
      The variable VENV_DIR should eq "${TEST_DIR}/.venv"
    End

    It 'returns error when python_dir not provided'
      When call setup_venv
      The status should eq 1
      The stderr should include "Error: python_dir required"
    End
  End

  Describe 'install_python_packages()'
    # AI Context: Installs missing Python packages in virtual environment
    # Handles packages with extras like "uvicorn[standard]"
    # Special handling for uvicorn WebSocket support

    It 'installs missing packages'
      VENV_PYTHON="${SHELLSPEC_TMPBASE}/venv_python"
      Mock "${VENV_PYTHON}"
        case "$1" in
          -c) return 1 ;; # Package not installed
          -m) [ "$2" = "pip" ] && [ "$3" = "install" ] && return 0 ;;
        esac
      End

      When call install_python_packages "${VENV_PYTHON}" "fastapi" "uvicorn"
      The status should eq 0
    End

    It 'skips already installed packages'
      VENV_PYTHON="${SHELLSPEC_TMPBASE}/venv_python"
      Mock "${VENV_PYTHON}"
        case "$1" in
          -c) return 0 ;; # Package already installed
        esac
      End

      When call install_python_packages "${VENV_PYTHON}" "fastapi"
      The status should eq 0
    End

    It 'returns error when venv_python not provided'
      When call install_python_packages
      The status should eq 1
      The stderr should include "Error: venv_python and at least one package required"
    End
  End

  Describe 'test_python_import()'
    # AI Context: Tests if Python module can be imported
    # Temporarily disables __init__.py to avoid dependency issues
    # Used before starting services to verify dependencies

    It 'succeeds when module imports correctly'
      VENV_PYTHON="${SHELLSPEC_TMPBASE}/venv_python"
      Mock "${VENV_PYTHON}"
        echo "OK"
        return 0
      End

      When call test_python_import "${VENV_PYTHON}" "integration.test_module" "app"
      The status should eq 0
    End

    It 'fails when module cannot be imported'
      VENV_PYTHON="${SHELLSPEC_TMPBASE}/venv_python"
      Mock "${VENV_PYTHON}"
        echo "ERROR: No module named test_module"
        return 1
      End

      When call test_python_import "${VENV_PYTHON}" "integration.test_module" "app"
      The status should eq 1
    End
  End

  Describe 'disable_init_py()'
    # AI Context: Temporarily disables __init__.py to avoid dependency issues
    # Sets INIT_PY and INIT_PY_BAK variables and creates EXIT trap
    # Used by service scripts before running uvicorn

    It 'renames __init__.py when it exists'
      TEST_DIR="${SHELLSPEC_TMPBASE}/test_disable_init"
      mkdir -p "${TEST_DIR}/integration"
      touch "${TEST_DIR}/integration/__init__.py"

      When call disable_init_py "${TEST_DIR}"
      The status should eq 0
      The file "${TEST_DIR}/integration/__init__.py" should not be exist
      The file "${TEST_DIR}/integration/__init__.py.bak" should be exist
    End

    It 'does nothing when __init__.py does not exist'
      TEST_DIR="${SHELLSPEC_TMPBASE}/test_no_init"
      mkdir -p "${TEST_DIR}/integration"

      When call disable_init_py "${TEST_DIR}"
      The status should eq 0
    End
  End
End
