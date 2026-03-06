#!/usr/bin/env bash
#shellcheck shell=sh
# Test suite for scripts/include/service_utils.sh
# Tests service utility functions for health checks and port validation
#
# AI Context:
# - This file tests shared service utility functions
# - Functions are sourced from scripts/include/service_utils.sh
# - These functions are used by all service startup scripts
# - Tests use ShellSpec BDD syntax for clarity

# Source the functions we're testing
# AI Note: PROJECT_ROOT is set in spec_helper.sh (loaded via --require in .shellspec)
# service_utils.sh depends on config.sh, so we source both
if [ -z "${PROJECT_ROOT:-}" ]; then
  _TEST_DIR="${SHELLSPEC_SPECDIR:-spec}"
  PROJECT_ROOT="$(cd "${_TEST_DIR}/.." 2>/dev/null && pwd || pwd)"
fi
. "${PROJECT_ROOT}/scripts/include/config.sh"
. "${PROJECT_ROOT}/scripts/include/service_utils.sh"

Describe 'service_utils.sh - Service utility functions'

  Describe 'check_service_health()'
    # AI Context: This function checks service health endpoints
    # It uses Python to make HTTP requests and verify service identity
    # Used by check_port_with_service to verify if a port is used by the expected service

    It 'returns 0 when service is healthy'
      PYTHON_CMD="python3"

      Mock python3
        echo '{"status": "ok"}'
        return 0
      End

      When call check_service_health "python3" "127.0.0.1" "8000" "ALPACA_SERVICE" "status"
      The status should eq 0
      The variable SERVICE_HEALTH_CHECK should eq "ALPACA_SERVICE"
    End

    It 'returns 1 when service is unhealthy'
      PYTHON_CMD="python3"

      Mock python3
        echo '{"status": "error"}'
        return 0
      End

      When call check_service_health "python3" "127.0.0.1" "8000" "ALPACA_SERVICE" "status"
      The status should eq 1
      The variable SERVICE_HEALTH_CHECK should eq "OTHER_SERVICE"
    End

    It 'returns 1 when health endpoint is unreachable'
      PYTHON_CMD="python3"

      Mock python3
        return 1
      End

      When call check_service_health "python3" "127.0.0.1" "8000" "ALPACA_SERVICE" "status"
      The status should eq 1
      The variable SERVICE_HEALTH_CHECK should eq "OTHER_SERVICE"
    End

    It 'returns error when python_cmd not provided'
      When call check_service_health "" "127.0.0.1" "8000" "ALPACA_SERVICE"
      The status should eq 1
      The stderr should include "Error: python_cmd, port, and service_name required"
    End

    It 'returns error when port not provided'
      When call check_service_health "python3" "127.0.0.1" "" "ALPACA_SERVICE"
      The status should eq 1
      The stderr should include "Error: python_cmd, port, and service_name required"
    End

    It 'returns error when service_name not provided'
      When call check_service_health "python3" "127.0.0.1" "8000" ""
      The status should eq 1
      The stderr should include "Error: python_cmd, port, and service_name required"
    End

    It 'checks for expected_key in response'
      PYTHON_CMD="python3"

      Mock python3
        echo '{"ib_connected": true, "status": "ok"}'
        return 0
      End

      When call check_service_health "python3" "127.0.0.1" "8002" "IB_SERVICE" "ib_connected"
      The status should eq 0
      The variable SERVICE_HEALTH_CHECK should eq "IB_SERVICE"
    End
  End

  Describe 'check_port_with_service()'
    # AI Context: This function checks if a port is available and verifies service identity
    # It combines port checking with health endpoint verification
    # Used by service scripts to avoid conflicts and detect existing service instances

    It 'returns 0 when port is available'
      PYTHON_CMD="python3"

      Mock config_check_port_available
        return 0  # Port is available
      End

      When call check_port_with_service "python3" "127.0.0.1" "8001" "TRADESTATION_SERVICE" "TradeStation"
      The status should eq 0
    End

    It 'returns 0 when port is in use by same service'
      PYTHON_CMD="python3"

      Mock config_check_port_available
        return 1  # Port is in use
      End

      Mock check_service_health
        SERVICE_HEALTH_CHECK="ALPACA_SERVICE"
        return 0  # Same service is running
      End

      When call check_port_with_service "python3" "127.0.0.1" "8000" "ALPACA_SERVICE" "Alpaca"
      The status should eq 0
      The output should include "Alpaca service is already running"
    End

    It 'returns 1 when port is in use by different service'
      PYTHON_CMD="python3"

      Mock config_check_port_available
        return 1  # Port is in use
      End

      Mock check_service_health
        SERVICE_HEALTH_CHECK="OTHER_SERVICE"
        return 1  # Different service
      End

      When call check_port_with_service "python3" "127.0.0.1" "8000" "ALPACA_SERVICE" "Alpaca"
      The status should eq 1
      The output should include "Port 8000 is in use by another service"
    End

    It 'returns error when python_cmd not provided'
      When call check_port_with_service "" "127.0.0.1" "8000" "ALPACA_SERVICE" "Alpaca"
      The status should eq 1
      The stderr should include "Error: python_cmd, port, and service_name required"
    End

    It 'returns error when port not provided'
      When call check_port_with_service "python3" "127.0.0.1" "" "ALPACA_SERVICE" "Alpaca"
      The status should eq 1
      The stderr should include "Error: python_cmd, port, and service_name required"
    End

    It 'returns error when service_name not provided'
      When call check_port_with_service "python3" "127.0.0.1" "8000" "" "Alpaca"
      The status should eq 1
      The stderr should include "Error: python_cmd, port, and service_name required"
    End

    It 'uses default service_display_name when not provided'
      PYTHON_CMD="python3"

      Mock config_check_port_available
        return 0
      End

      When call check_port_with_service "python3" "127.0.0.1" "8000" "ALPACA_SERVICE"
      The status should eq 0
    End

    It 'provides helpful error message with resolution steps'
      PYTHON_CMD="python3"

      Mock config_check_port_available
        return 1
      End

      Mock check_service_health
        SERVICE_HEALTH_CHECK="OTHER_SERVICE"
        return 1
      End

      When call check_port_with_service "python3" "127.0.0.1" "8000" "ALPACA_SERVICE" "Alpaca"
      The status should eq 1
      The output should include "export ALPACA_SERVICE_PORT"
      The output should include "config/config.json"
    End

    It 'handles missing config_check_port_available gracefully'
      PYTHON_CMD="python3"

      # Remove function if it exists
      unset -f config_check_port_available 2>/dev/null || true

      When call check_port_with_service "python3" "127.0.0.1" "8000" "ALPACA_SERVICE" "Alpaca"
      The status should eq 1
      The stderr should include "Warning: config_check_port_available not available"
    End
  End
End
