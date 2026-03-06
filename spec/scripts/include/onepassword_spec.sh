#!/usr/bin/env bash
#shellcheck shell=sh
# Test suite for scripts/include/onepassword.sh
# Tests 1Password credential management functions
#
# AI Context:
# - This file tests shared 1Password credential functions
# - Functions are sourced from scripts/include/onepassword.sh
# - These functions are used by service scripts that require API credentials
# - Tests use ShellSpec BDD syntax for clarity

# Source the functions we're testing
# AI Note: PROJECT_ROOT is set in spec_helper.sh (loaded via --require in .shellspec)
if [ -z "${PROJECT_ROOT:-}" ]; then
  _TEST_DIR="${SHELLSPEC_SPECDIR:-spec}"
  PROJECT_ROOT="$(cd "${_TEST_DIR}/.." 2>/dev/null && pwd || pwd)"
fi
. "${PROJECT_ROOT}/scripts/include/onepassword.sh"

Describe 'onepassword.sh - 1Password credential management functions'

  Describe 'read_credential()'
    # AI Context: This function reads credentials from 1Password or environment variables
    # Priority: 1Password first, then environment variable fallback
    # Used by service scripts to securely retrieve API credentials

    It 'returns credential from 1Password when available'
      Mock op
        echo "secret_value_from_1password"
        return 0
      End

      When call read_credential "op://Vault/Item/Field" ""
      The output should eq "secret_value_from_1password"
      The status should eq 0
    End

    It 'falls back to environment variable when 1Password fails'
      Mock op
        return 1
      End

      TEST_ENV_VAR="fallback_value"
      export TEST_ENV_VAR

      When call read_credential "op://Vault/Item/Field" "${TEST_ENV_VAR}"
      The output should eq "fallback_value"
      The status should eq 0
    End

    It 'returns 1 when neither 1Password nor env var available'
      Mock op
        return 1
      End

      When call read_credential "op://Vault/Item/Field" ""
      The status should eq 1
      The output should be empty
    End

    It 'handles empty 1Password secret path'
      TEST_ENV_VAR="env_value"
      export TEST_ENV_VAR

      When call read_credential "" "${TEST_ENV_VAR}"
      The output should eq "env_value"
      The status should eq 0
    End

    It 'trims whitespace from 1Password output'
      Mock op
        echo "  secret_with_whitespace  "
        return 0
      End

      When call read_credential "op://Vault/Item/Field" ""
      The output should eq "secret_with_whitespace"
      The status should eq 0
    End

    It 'handles 1Password CLI not available'
      # Mock command to simulate op not found
      Mock command
        [ "$1" = "op" ] && return 1 || return 0
      End

      TEST_ENV_VAR="env_fallback"
      export TEST_ENV_VAR

      When call read_credential "op://Vault/Item/Field" "${TEST_ENV_VAR}"
      The output should eq "env_fallback"
      The status should eq 0
    End
  End

  Describe 'op_detect_fields()'
    # AI Context: This function auto-detects field names from 1Password item UUID
    # It searches for common field name patterns for API keys and secrets
    # Used to simplify 1Password integration by auto-detecting field names

    It 'detects key field from 1Password item'
      Mock op
        echo '{"fields": [{"label": "API Key ID", "value": "key123"}]}'
        return 0
      End

      Mock python3
        echo "API Key ID"
        return 0
      End

      When call op_detect_fields "test-uuid" "KEY_FIELD" "SECRET_FIELD"
      The status should eq 0
      The variable KEY_FIELD should eq "API Key ID"
    End

    It 'detects secret field from 1Password item'
      Mock op
        echo '{"fields": [{"label": "API Secret Key", "type": "CONCEALED", "value": "secret123"}]}'
        return 0
      End

      Mock python3
        case "$1" in
          -c)
            # First call detects key field
            if echo "$2" | grep -q "API Key ID"; then
              echo "API Key ID"
            # Second call detects secret field
            elif echo "$2" | grep -q "CONCEALED"; then
              echo "API Secret Key"
            fi
            ;;
        esac
        return 0
      End

      When call op_detect_fields "test-uuid" "KEY_FIELD" "SECRET_FIELD"
      The status should eq 0
    End

    It 'returns 1 when item UUID not provided'
      When call op_detect_fields "" "KEY_FIELD" "SECRET_FIELD"
      The status should eq 1
    End

    It 'returns 1 when 1Password CLI not available'
      Mock command
        [ "$1" = "op" ] && return 1 || return 0
      End

      When call op_detect_fields "test-uuid" "KEY_FIELD" "SECRET_FIELD"
      The status should eq 1
    End

    It 'returns 1 when item not found'
      Mock op
        return 1
      End

      When call op_detect_fields "invalid-uuid" "KEY_FIELD" "SECRET_FIELD"
      The status should eq 1
    End

    It 'uses default variable names when not provided'
      Mock op
        echo '{"fields": [{"label": "API Key ID", "value": "key123"}]}'
        return 0
      End

      Mock python3
        echo "API Key ID"
        return 0
      End

      When call op_detect_fields "test-uuid"
      The status should eq 0
      The variable KEY_FIELD should eq "API Key ID"
    End

    It 'handles multiple field name patterns'
      Mock op
        echo '{"fields": [{"label": "username", "value": "key123"}, {"label": "credential", "type": "CONCEALED", "value": "secret123"}]}'
        return 0
      End

      Mock python3
        case "$1" in
          -c)
            if echo "$2" | grep -q "username"; then
              echo "username"
            elif echo "$2" | grep -q "credential"; then
              echo "credential"
            fi
            ;;
        esac
        return 0
      End

      When call op_detect_fields "test-uuid" "KEY_FIELD" "SECRET_FIELD"
      The status should eq 0
    End
  End

  Describe 'op_build_secret_paths()'
    # AI Context: This function builds full op:// secret paths from item UUID and field names
    # It extracts vault information and constructs proper op:// references
    # Used to convert item UUIDs into usable secret paths

    It 'builds secret paths with vault ID'
      Mock op
        echo '{"vault": {"id": "vault123", "name": "TestVault"}, "id": "item456"}'
        return 0
      End

      Mock python3
        case "$1" in
          -c)
            if echo "$2" | grep -q "vault.*id"; then
              echo "vault123"
            elif echo "$2" | grep -q "vault.*name"; then
              echo "TestVault"
            fi
            ;;
        esac
        return 0
      End

      When call op_build_secret_paths "item456" "API Key ID" "API Secret Key" "OP_KEY_SECRET" "OP_SECRET_SECRET"
      The status should eq 0
      The variable OP_KEY_SECRET should eq "op://vault123/item456/API Key ID"
      The variable OP_SECRET_SECRET should eq "op://vault123/item456/API Secret Key"
    End

    It 'builds secret paths with vault name when ID not available'
      Mock op
        echo '{"vault": {"name": "TestVault"}, "id": "item456"}'
        return 0
      End

      Mock python3
        case "$1" in
          -c)
            if echo "$2" | grep -q "vault.*id"; then
              echo ""  # No vault ID
            elif echo "$2" | grep -q "vault.*name"; then
              echo "TestVault"
            fi
            ;;
        esac
        return 0
      End

      When call op_build_secret_paths "item456" "API Key ID" "API Secret Key" "OP_KEY_SECRET" "OP_SECRET_SECRET"
      The status should eq 0
      The variable OP_KEY_SECRET should eq "op://TestVault/item456/API Key ID"
    End

    It 'falls back to Private vault when vault info not available'
      Mock op
        echo '{"id": "item456"}'
        return 0
      End

      Mock python3
        echo ""  # No vault info
        return 0
      End

      When call op_build_secret_paths "item456" "API Key ID" "API Secret Key" "OP_KEY_SECRET" "OP_SECRET_SECRET"
      The status should eq 0
      The variable OP_KEY_SECRET should eq "op://Private/item456/API Key ID"
    End

    It 'returns 1 when item UUID not provided'
      When call op_build_secret_paths "" "API Key ID" "API Secret Key" "OP_KEY_SECRET" "OP_SECRET_SECRET"
      The status should eq 1
    End

    It 'returns 1 when item not found'
      Mock op
        return 1
      End

      When call op_build_secret_paths "invalid-uuid" "API Key ID" "API Secret Key" "OP_KEY_SECRET" "OP_SECRET_SECRET"
      The status should eq 1
    End

    It 'uses default variable names when not provided'
      Mock op
        echo '{"vault": {"id": "vault123"}, "id": "item456"}'
        return 0
      End

      Mock python3
        echo "vault123"
        return 0
      End

      When call op_build_secret_paths "item456" "API Key ID" "API Secret Key"
      The status should eq 0
      The variable OP_KEY_SECRET should eq "op://vault123/item456/API Key ID"
      The variable OP_SECRET_SECRET should eq "op://vault123/item456/API Secret Key"
    End

    It 'handles special characters in field names'
      Mock op
        echo '{"vault": {"id": "vault123"}, "id": "item456"}'
        return 0
      End

      Mock python3
        echo "vault123"
        return 0
      End

      When call op_build_secret_paths "item456" "API Key (ID)" "API Secret-Key" "OP_KEY_SECRET" "OP_SECRET_SECRET"
      The status should eq 0
      The variable OP_KEY_SECRET should eq "op://vault123/item456/API Key (ID)"
      The variable OP_SECRET_SECRET should eq "op://vault123/item456/API Secret-Key"
    End
  End
End
