# Shell Script Test Suite

This directory contains ShellSpec tests for shared shell script functions.

## Test Files

All test files are in `spec/scripts/include/`:

- ✅ `python_utils_spec.sh` - Tests for Python utility functions
  - `find_python()` - Python interpreter detection
  - `setup_venv()` - Virtual environment setup
  - `install_python_packages()` - Package installation
  - `test_python_import()` - Module import testing
  - `disable_init_py()` - __init__.py workaround

- ✅ `config_spec.sh` - Tests for configuration functions
  - `config_get_port()` - Service port retrieval
  - `config_check_port_available()` - Port availability checking
  - `config_get()` - Generic config value retrieval

- ✅ `service_utils_spec.sh` - Tests for service utility functions
  - `check_service_health()` - Health endpoint checking
  - `check_port_with_service()` - Port conflict detection with service verification

- ✅ `onepassword_spec.sh` - Tests for 1Password functions
  - `read_credential()` - Credential reading with fallback
  - `op_detect_fields()` - Field name auto-detection
  - `op_build_secret_paths()` - Secret path construction

## Running Tests

### Run All Tests

```bash
./scripts/run_tests.sh
```

### Run Specific Test File

```bash
shellspec spec/scripts/include/python_utils_spec.sh
```

### Run with Different Formats

```bash
# Documentation format (default)
./scripts/run_tests.sh --format documentation

# Progress format (faster)
./scripts/run_tests.sh --format progress

# TAP format (for CI)
./scripts/run_tests.sh --format tap

# jUnit XML format (for CI)
./scripts/run_tests.sh --format junit
```

### Run Tests in Parallel

```bash
./scripts/run_tests.sh --parallel 8
```

### Run Only Failed Tests

```bash
./scripts/run_tests.sh --quick
```

## Test Coverage

### Functions Tested

| Function | Test File | Coverage |
|----------|-----------|----------|
| `find_python()` | `python_utils_spec.sh` | ✅ Complete |
| `setup_venv()` | `python_utils_spec.sh` | ✅ Complete |
| `install_python_packages()` | `python_utils_spec.sh` | ✅ Complete |
| `test_python_import()` | `python_utils_spec.sh` | ✅ Complete |
| `disable_init_py()` | `python_utils_spec.sh` | ✅ Complete |
| `config_get_port()` | `config_spec.sh` | ✅ Complete |
| `config_check_port_available()` | `config_spec.sh` | ✅ Complete |
| `config_get()` | `config_spec.sh` | ✅ Complete |
| `check_service_health()` | `service_utils_spec.sh` | ✅ Complete |
| `check_port_with_service()` | `service_utils_spec.sh` | ✅ Complete |
| `read_credential()` | `onepassword_spec.sh` | ✅ Complete |
| `op_detect_fields()` | `onepassword_spec.sh` | ✅ Complete |
| `op_build_secret_paths()` | `onepassword_spec.sh` | ✅ Complete |

### Test Scenarios Covered

Each function is tested for:
- ✅ Success cases
- ✅ Error cases
- ✅ Edge cases
- ✅ Parameter validation
- ✅ Return value verification
- ✅ Global variable setting
- ✅ Error message validation

## Writing New Tests

When adding new functions to shared include files:

1. **Add function to include file** with comprehensive AI-friendly comments
2. **Create test file** in `spec/scripts/include/` (or add to existing)
3. **Write tests** using ShellSpec BDD syntax
4. **Run tests** to verify they pass
5. **Update this README** with new test coverage

### Test Template

```bash
#!/usr/bin/env bash
#shellcheck shell=sh

Describe 'function_name()'
  BeforeAll 'setup_test_environment() {
    SCRIPT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
    source "${SCRIPT_DIR}/scripts/include/your_file.sh"
  }'

  It 'does something specific'
    When call function_name "arg1" "arg2"
    The status should eq 0
    The output should eq "expected"
  End

  It 'handles error case'
    When call function_name ""
    The status should eq 1
    The stderr should include "Error"
  End
End
```

## CI/CD Integration

Tests can be integrated into CI/CD pipelines:

### GitHub Actions

```yaml
- name: Run shell script tests
  run: |
    curl -fsSL https://git.io/shellspec | sh
    ~/.local/bin/shellspec --format tap
```

### GitLab CI

```yaml
test:shell:
  script:
    - curl -fsSL https://git.io/shellspec | sh
    - ~/.local/bin/shellspec --format tap
```

## Resources

- [ShellSpec Documentation](https://shellspec.info/)
- [Testing Guide](../../docs/SHELL_SCRIPT_TESTING.md)
- [AI Agent Guide](../../docs/AI_AGENT_SHELL_SCRIPT_GUIDE.md)
