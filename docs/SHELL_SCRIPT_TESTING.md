# Shell Script Testing with ShellSpec

This document describes the testing infrastructure for shell scripts in this project, using [ShellSpec](https://shellspec.info/), a BDD testing framework for shell scripts.

## Overview

We use ShellSpec to test shared utility functions in `scripts/include/`. These functions are used by all service startup scripts and are critical for reliable service deployment.

## Installation

ShellSpec is installed via the official installer:

```bash
curl -fsSL https://git.io/shellspec | sh
```

This installs ShellSpec to `~/.local/bin/shellspec` and makes it available system-wide.

## Project Structure

```
spec/
├── scripts/
│   └── include/
│       ├── python_utils_spec.sh    # Tests for Python utility functions
│       ├── config_spec.sh          # Tests for configuration functions
│       ├── service_utils_spec.sh   # Tests for service utility functions
│       └── onepassword_spec.sh     # Tests for 1Password functions
.shellspec                          # ShellSpec configuration file
```

## Running Tests

### Run All Tests

```bash
shellspec
```

### Run Specific Test File

```bash
shellspec spec/scripts/include/python_utils_spec.sh
```

### Run with Documentation Formatter

```bash
shellspec --format documentation
```

### Run with Progress Formatter (faster)

```bash
shellspec --format progress
```

### Run Tests in Parallel

```bash
shellspec --jobs 4
```

### Run Only Failed Tests (Quick Mode)

```bash
shellspec --quick
```

## Test Structure

Tests use ShellSpec's BDD syntax:

```bash
#!/usr/bin/env bash
#shellcheck shell=sh

Describe 'function_name()'
  It 'does something specific'
    When call function_name "arg1" "arg2"
    The status should eq 0
    The output should eq "expected output"
  End
End
```

## Writing Tests

### Basic Test Example

```bash
Describe 'find_python()'
  It 'finds python3 when available'
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
End
```

### Mocking External Commands

ShellSpec supports mocking external commands:

```bash
Mock lsof
  return 1  # Port not in use
End

When call config_check_port_available 8000
The status should eq 0
```

### Testing Functions That Modify Global Variables

Some functions set global variables. Test them like this:

```bash
Describe 'setup_venv()'
  It 'sets VENV_DIR variable'
    PYTHON_CMD="python3"
    TEST_DIR="${SHELLSPEC_TMPBASE}/test_venv"

    When call setup_venv "${TEST_DIR}"
    The variable VENV_DIR should eq "${TEST_DIR}/.venv"
  End
End
```

## Test Coverage

Current test coverage:

- ✅ `python_utils.sh` - Python utility functions
- ✅ `config.sh` - Configuration loading functions
- 🔄 `service_utils.sh` - Service utility functions (in progress)
- 🔄 `onepassword.sh` - 1Password functions (in progress)

## CI/CD Integration

### GitHub Actions

Add to `.github/workflows/test.yml`:

```yaml
name: Shell Script Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install ShellSpec
        run: curl -fsSL https://git.io/shellspec | sh
      - name: Run tests
        run: ~/.local/bin/shellspec --format tap
```

### TAP Output for CI

ShellSpec can output TAP format for CI integration:

```bash
shellspec --format tap > test-results.tap
```

### jUnit XML Output

For CI systems that support jUnit XML:

```bash
shellspec --format junit > test-results.xml
```

## Best Practices

1. **Test One Thing**: Each `It` block should test one specific behavior
2. **Use Descriptive Names**: Test names should clearly describe what's being tested
3. **Mock External Dependencies**: Mock commands like `python3`, `lsof`, `jq` to avoid system dependencies
4. **Test Error Cases**: Include tests for error conditions and edge cases
5. **Use Temporary Directories**: Use `SHELLSPEC_TMPBASE` for temporary files
6. **Clean Up**: ShellSpec automatically cleans up, but be mindful of side effects

## Debugging Tests

### Verbose Output

```bash
shellspec --verbose
```

### Trace Execution

```bash
shellspec --trace
```

### Debug Mode

```bash
shellspec --debug
```

### Show Examples Without Running

```bash
shellspec --list
```

## Resources

- [ShellSpec Documentation](https://github.com/shellspec/shellspec)
- [ShellSpec Examples](https://github.com/shellspec/shellspec/tree/master/examples)
- [BDD Syntax Reference](https://github.com/shellspec/shellspec/blob/master/docs/references.md)

## Contributing

When adding new functions to shared include files:

1. Add comprehensive AI-friendly comments (see existing files for examples)
2. Write tests in `spec/scripts/include/`
3. Run tests before committing: `shellspec`
4. Ensure all tests pass: `shellspec --fail-fast`

## Troubleshooting

### Tests Fail with "command not found"

Make sure you've sourced the functions being tested:

```bash
BeforeAll 'setup_test_environment() {
  SCRIPT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
  source "${SCRIPT_DIR}/scripts/include/python_utils.sh"
}'
```

### Mock Not Working

Ensure mocks are defined before the `When call` statement:

```bash
Mock command
  return 0
End

When call function_that_uses_command
```

### Global Variables Not Set

Some functions set global variables. Make sure you're testing the right scope:

```bash
When call setup_venv "${TEST_DIR}"
The variable VENV_DIR should eq "${TEST_DIR}/.venv"
```
