#!/usr/bin/env bash
#shellcheck shell=sh
# Test suite for scripts/include/config.sh
# Tests configuration loading functions used across service scripts
#
# AI Context:
# - This file tests shared configuration management functions
# - Functions read from config.json with environment variable overrides
# - Used by all service scripts for port configuration
# - Tests use ShellSpec BDD syntax for clarity

# Source the functions we're testing
# AI Note: PROJECT_ROOT is set in spec_helper.sh (loaded via --require in .shellspec)
if [ -z "${PROJECT_ROOT:-}" ]; then
  _TEST_DIR="${SHELLSPEC_SPECDIR:-spec}"
  PROJECT_ROOT="$(cd "${_TEST_DIR}/.." 2>/dev/null && pwd || pwd)"
fi
. "${PROJECT_ROOT}/scripts/include/config.sh"

Describe 'config.sh - Configuration loader functions'

  Describe 'config_get_port()'
    # AI Context: Gets service port from config.json or environment variable
    # Priority: ENV_VAR > config.json > default_port
    # Used by all service scripts to determine which port to use

    It 'returns environment variable when set'
      # Set environment variable
      ALPACA_PORT=9000
      export ALPACA_PORT

      When call config_get_port "alpaca" 8000
      The output should eq "9000"
      The status should eq 0
    End

    It 'returns config file value when env var not set'
      # Mock config file and jq
      TEST_CONFIG="${SHELLSPEC_TMPBASE}/config.json"
      echo '{"services":{"alpaca":{"port":7500}}}' > "${TEST_CONFIG}"

      Mock _find_config_file
        echo "${TEST_CONFIG}"
      End

      Mock jq
        echo "7500"
      End

      When call config_get_port "alpaca" 8000
      The output should eq "7500"
    End

    It 'returns default when no config found'
      Mock _find_config_file
        return 1
      End

      When call config_get_port "alpaca" 8000
      The output should eq "8000"
      The status should eq 0
    End

    It 'returns error when service_name not provided'
      When call config_get_port
      The status should eq 1
      The stderr should include "Error: service_name required"
    End
  End

  Describe 'config_check_port_available()'
    # AI Context: Checks if a port is available (not in use)
    # Uses lsof, netstat, or Python socket as fallback
    # Used by service scripts before starting to avoid conflicts

    It 'returns 0 when port is available (lsof)'
      Mock lsof
        return 1 # Port not in use
      End

      When call config_check_port_available 8000
      The status should eq 0
    End

    It 'returns 1 when port is in use (lsof)'
      Mock lsof
        return 0 # Port in use
      End

      When call config_check_port_available 8000
      The status should eq 1
    End

    It 'falls back to netstat when lsof not available'
      Mock command
        case "$1" in
          lsof) return 1 ;;
          netstat) return 0 ;;
        esac
      End

      Mock netstat
        echo "tcp4  0  0  *.8000  *.*  LISTEN"
      End

      When call config_check_port_available 8000
      The status should eq 1
    End

    It 'returns error when port not provided'
      When call config_check_port_available
      The status should eq 1
      The stderr should include "Error: port required"
    End
  End

  Describe 'config_get()'
    # AI Context: Generic config value getter using jq JSON path
    # Used for reading any value from config.json
    # Supports default values

    It 'returns config value when found'
      TEST_CONFIG="${SHELLSPEC_TMPBASE}/config.json"
      echo '{"tws":{"port":7497}}' > "${TEST_CONFIG}"

      Mock _find_config_file
        echo "${TEST_CONFIG}"
      End

      Mock jq
        echo "7497"
      End

      When call config_get ".tws.port" 7496
      The output should eq "7497"
    End

    It 'returns default when config value not found'
      Mock _find_config_file
        return 1
      End

      When call config_get ".tws.port" 7497
      The output should eq "7497"
    End
  End
End
