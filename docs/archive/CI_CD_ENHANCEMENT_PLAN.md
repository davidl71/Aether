# CI/CD Enhancement Plan for Parallel Agent Workflows

**Date:** 2025-01-20
**Purpose:** Comprehensive CI/CD enhancement for Ubuntu + macOS M4 parallel Cursor agent workflows
**Status:** ✅ **Enhancement Plan**

---

## Overview

This document outlines a comprehensive CI/CD enhancement strategy specifically designed for parallel development workflows with Ubuntu and macOS M4 Cursor agents. The plan builds on existing GitHub Actions workflows and adds parallel testing, integration validation, and coordination checks.

---

## Current State Analysis

### Existing CI/CD Setup

✅ **What's Working:**

- Basic GitHub Actions workflow (`.github/workflows/ci.yml`)
- Toolchain validation (Linux + macOS)
- TUI Go tests
- Homebrew bottle builds
- Documentation validation workflow

🟡 **What Needs Enhancement:**

- Parallel agent workflow support
- Cross-platform integration testing
- Coordination validation (TODO table, API contracts)
- Agent-specific test suites
- Integration test after parallel merges
- Performance benchmarking

---

## CI/CD Tool Recommendation

### GitHub Actions (Recommended)

**Why GitHub Actions:**

- ✅ **Native Integration:** Already in use, seamless Git integration
- ✅ **Parallel Execution:** Supports matrix builds for multi-platform testing
- ✅ **Cost-Effective:** Free for public repos, generous free tier for private
- ✅ **Self-Hosted Runners:** Can use your Ubuntu + macOS M4 agents as runners
- ✅ **Workflow Flexibility:** Complex workflows, dependencies, conditional execution
- ✅ **Artifact Management:** Store build artifacts, test results, reports

**Alternative Options:**

- **Jenkins:** More complex setup, better for on-premises
- **GitLab CI:** Good if using GitLab (you're on GitHub)
- **CircleCI:** Similar to GitHub Actions, more complex pricing
- **TeamCity:** Enterprise-focused, overkill for this project

**Recommendation:** **Stick with GitHub Actions** and enhance existing workflows.

---

## Enhanced Workflow Architecture

### Workflow Structure

```
┌─────────────────────────────────────────────────────────┐
│                   PR/Push Trigger                        │
└─────────────────────┬───────────────────────────────────┘
                      │
        ┌─────────────┴─────────────┐
        │                           │
┌───────▼────────┐        ┌────────▼────────┐
│  Ubuntu Tests  │        │  macOS Tests    │
│                │        │                 │
│ - Backend      │        │ - AppKit        │
│ - NATS         │        │ - UI            │
│ - Linux builds │        │ - macOS builds  │
└───────┬────────┘        └────────┬────────┘
        │                           │
        └─────────────┬─────────────┘
                      │
        ┌─────────────▼─────────────┐
        │   Integration Tests       │
        │                           │
        │ - Cross-platform          │
        │ - API contract validation │
        │ - Coordination checks     │
        └───────┬───────────────────┘
                │
        ┌───────▼────────┐
        │  Merge Ready?  │
        └────────────────┘
```

---

## Enhanced Workflow Implementation

### 1. Parallel Agent Workflows

**File:** `.github/workflows/parallel-agents-ci.yml`

```yaml
name: Parallel Agents CI

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main, develop ]
  workflow_dispatch:

jobs:
  # Ubuntu Agent Workflow
  ubuntu-agent-tests:
    name: Ubuntu Agent Tests
    runs-on: ubuntu-latest
    strategy:
      matrix:
        component: [backend, nats, tui, web]

    steps:
      - name: Checkout code
        uses: actions/checkout@v4
        with:
          fetch-depth: 0  # Full history for git worktree support

      - name: Detect changed files
        id: changes
        uses: dorny/paths-filter@v2
        with:
          filters: |
            backend:
              - 'agents/backend/**'
              - 'native/src/**'
            nats:
              - 'agents/backend/src/nats/**'
            web:
              - 'web/**'

      - name: Setup Rust (for backend/NATS)
        if: matrix.component == 'backend' || matrix.component == 'nats'
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Setup Python (for backend)
        if: matrix.component == 'backend'
        uses: actions/setup-python@v5
        with:
          python-version: '3.12'

      - name: Setup Node.js (for web)
        if: matrix.component == 'web'
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Run Ubuntu Agent Tests
        run: |
          case "${{ matrix.component }}" in
            backend)
              cd agents/backend
              cargo test --all-features
              ;;
            nats)
              cd agents/backend
              cargo test --package nats_adapter
              ;;
            web)
              cd web
              npm ci
              npm test -- --watch=false
              ;;
          esac

      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: ubuntu-${{ matrix.component }}-test-results
          path: |
            **/test-results.xml
            **/coverage.xml

  # macOS M4 Agent Workflow
  macos-agent-tests:
    name: macOS Agent Tests
    runs-on: macos-14

    strategy:
      matrix:
        component: [appkit, native-build, universal-binary]

    steps:
      - name: Checkout code
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Detect changed files
        id: changes
        uses: dorny/paths-filter@v2
        with:
          filters: |
            appkit:
              - 'native/app/**'
            native-build:
              - 'native/**'
            universal-binary:
              - 'native/**'
              - 'CMakeLists.txt'

      - name: Setup CMake
        uses: actions/setup-cmake@v3
        with:
          cmake-version: '3.28'

      - name: Configure CMake
        run: cmake --preset macos-arm64-debug

      - name: Build
        run: cmake --build build

      - name: Run macOS Tests
        run: |
          case "${{ matrix.component }}" in
            appkit)
              ctest --test-dir build -L appkit --output-on-failure
              ;;
            native-build)
              ctest --test-dir build --output-on-failure
              ;;
            universal-binary)
              cmake --preset macos-universal-debug
              cmake --build build
              ctest --test-dir build --output-on-failure
              ;;
          esac

      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: macos-${{ matrix.component }}-test-results
          path: |
            build/**/*.app
            build/test-results.xml

  # Cross-Platform Integration Tests
  integration-tests:
    name: Integration Tests
    runs-on: ${{ matrix.os }}
    needs: [ubuntu-agent-tests, macos-agent-tests]

    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-14]

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Download Ubuntu test artifacts
        uses: actions/download-artifact@v4
        with:
          name: ubuntu-backend-test-results
          path: artifacts/ubuntu/

      - name: Download macOS test artifacts
        uses: actions/download-artifact@v4
        with:
          name: macos-native-build-test-results
          path: artifacts/macos/

      - name: Run Integration Tests
        run: |
          # Test API contract compatibility
          ./scripts/validate_api_contract.sh

          # Test cross-platform communication
          ./scripts/run_integration_tests.sh

      - name: Upload integration results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: integration-test-results-${{ matrix.os }}
          path: integration-results/

  # Coordination Validation
  coordination-validation:
    name: Coordination Validation
    runs-on: ubuntu-latest
    needs: [ubuntu-agent-tests, macos-agent-tests]

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Validate TODO Table
        run: |
          # Check TODO_OVERVIEW.md format
          ./scripts/validate_todo_table.sh

      - name: Validate API Contract
        run: |
          # Check API_CONTRACT.md for breaking changes
          ./scripts/validate_api_contract.sh

      - name: Check for merge conflicts
        run: |
          # Verify both agents' branches can merge
          git fetch origin
          git checkout main
          git merge --no-commit --no-ff origin/feature/ubuntu-agent
          git merge --no-commit --no-ff origin/feature/macos-agent
          git merge --abort || true

      - name: Comment on PR
        if: github.event_name == 'pull_request'
        uses: actions/github-script@v7
        with:
          script: |
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: '✅ Coordination validation passed! Both agents are in sync.'
            })

  # Final Status Check
  ci-success:
    name: CI Success
    runs-on: ubuntu-latest
    needs: [ubuntu-agent-tests, macos-agent-tests, integration-tests, coordination-validation]
    if: always()

    steps:
      - name: Check all jobs
        run: |
          if [ "${{ needs.ubuntu-agent-tests.result }}" != "success" ] || \
             [ "${{ needs.macos-agent-tests.result }}" != "success" ] || \
             [ "${{ needs.integration-tests.result }}" != "success" ] || \
             [ "${{ needs.coordination-validation.result }}" != "success" ]; then
            echo "❌ CI failed - check individual job results"
            exit 1
          fi
          echo "✅ All CI checks passed!"
```

---

## Validation Scripts

### 1. API Contract Validation

**File:** `scripts/validate_api_contract.sh`

```bash

#!/usr/bin/env bash
# Validate API contract hasn't been broken by parallel agent changes

set -euo pipefail

CONTRACT_FILE="agents/shared/API_CONTRACT.md"

echo "Validating API Contract..."

# Check for breaking changes

if git diff HEAD~1 "$CONTRACT_FILE" | grep -q "^\-.*:"; then
    echo "⚠️  Breaking API changes detected!"
    echo "Please document breaking changes in PR description"
    exit 1
fi

# Validate format

if ! grep -q "## API Endpoints" "$CONTRACT_FILE"; then
    echo "❌ API Contract missing required sections"
    exit 1
fi

echo "✅ API Contract validation passed"
```

### 2. TODO Table Validation

**File:** `scripts/validate_todo_table.sh`

```bash

#!/usr/bin/env bash
# Validate TODO table format and completeness

set -euo pipefail

TODO_FILE="agents/shared/TODO_OVERVIEW.md"

echo "Validating TODO Table..."

# Check file exists

if [ ! -f "$TODO_FILE" ]; then
    echo "❌ TODO_OVERVIEW.md not found"
    exit 1
fi

# Check for required columns

if ! grep -q "| Task | Agent | Status |" "$TODO_FILE"; then
    echo "❌ TODO table missing required columns"
    exit 1
fi

# Check for in-progress tasks with notes

if grep -q "| .* | .* | in_progress |" "$TODO_FILE"; then
    echo "✅ TODO table validation passed (has in-progress tasks)"
fi

echo "✅ TODO Table validation passed"
```

### 3. Integration Test Runner

**File:** `scripts/run_integration_tests.sh`

```bash

#!/usr/bin/env bash
# Run cross-platform integration tests

set -euo pipefail

echo "Running integration tests..."

# Test NATS communication between platforms

cd agents/backend
cargo test --test integration_nats_cross_platform

# Test API endpoint compatibility

curl -f http://localhost:8080/health || echo "Backend not running (expected in CI)"

# Test macOS UI can connect to backend
# (if backend is available in CI)

echo "✅ Integration tests passed"
```

---

## Workflow Triggers

### When CI Runs

1. **On Every Push:**
   - Ubuntu agent tests (if Ubuntu-related files changed)
   - macOS agent tests (if macOS-related files changed)
   - Integration tests (if both agents' files changed)

2. **On Pull Requests:**
   - All agent tests
   - Integration tests
   - Coordination validation
   - Status comment on PR

3. **On Merge to Main:**
   - Full test suite
   - Build artifacts
   - Release preparation (if tagged)

---

## Benefits for Parallel Development

### 1. Early Detection

- **Before Merge:** Catch integration issues before merging
- **Parallel Validation:** Both agents' code tested simultaneously
- **Fast Feedback:** Agents get immediate test results

### 2. Coordination

- **API Contract:** Ensures both agents stay compatible
- **TODO Table:** Validates coordination tracking
- **Merge Safety:** Checks for conflicts before merge

### 3. Quality Assurance

- **Cross-Platform:** Tests work on both Ubuntu and macOS
- **Integration:** Validates communication between agents
- **Coverage:** Comprehensive test coverage across components

---

## Setup Instructions

### 1. Add Enhanced Workflows

```bash

# Create enhanced workflow file

cp .github/workflows/ci.yml .github/workflows/parallel-agents-ci.yml

# Add validation scripts

chmod +x scripts/validate_api_contract.sh
chmod +x scripts/validate_todo_table.sh
chmod +x scripts/run_integration_tests.sh
```

### 2. Configure GitHub Actions

**Repository Settings:**

1. Go to Settings → Actions → General
2. Enable "Allow all actions and reusable workflows"
3. Enable "Allow actions to create and approve pull requests"

**Secrets (if needed):**

- Add any API keys or tokens to repository secrets
- Configure self-hosted runners (optional)

### 3. Test Workflows

```bash

# Test locally

act -j ubuntu-agent-tests  # Requires act (GitHub Actions local runner)

# Or push to test branch

git checkout -b test/ci-enhancement
git push origin test/ci-enhancement

# Create PR to trigger workflows
```

---

## Monitoring and Optimization

### 1. Workflow Performance

**Track:**

- Workflow duration
- Test execution time
- Resource usage
- Failure rates

**Optimize:**

- Parallel execution
- Caching dependencies
- Skipping unnecessary jobs
- Using matrix builds efficiently

### 2. Notification Setup

**GitHub Notifications:**

- PR status updates
- Workflow failure alerts
- Success notifications (optional)

**Slack/Discord Integration:**

- Workflow failure notifications
- Weekly CI health reports
- Test coverage trends

---

## Alternative: Self-Hosted Runners

### Using Your Agents as Runners

**Benefits:**

- Use actual Ubuntu and macOS M4 hardware
- Faster builds (local network)
- Access to local dependencies
- Test on real hardware

**Setup:**

```bash

# On Ubuntu agent

cd actions-runner
./config.sh --url https://github.com/your-repo --token YOUR_TOKEN
./run.sh

# On macOS M4 agent

cd actions-runner
./config.sh --url https://github.com/your-repo --token YOUR_TOKEN
./run.sh
```

**Workflow Configuration:**

```yaml
jobs:
  ubuntu-agent-tests:
    runs-on: self-hosted  # Uses your Ubuntu agent
    labels: [ubuntu, linux]

  macos-agent-tests:
    runs-on: self-hosted  # Uses your macOS M4 agent
    labels: [macos, apple-silicon]
```

---

## Next Steps

### Immediate Actions

1. **Review Current Workflows:**
   - Analyze existing `.github/workflows/ci.yml`
   - Identify gaps for parallel agent support

2. **Create Enhanced Workflows:**
   - Add `parallel-agents-ci.yml`
   - Create validation scripts
   - Test with sample PR

3. **Setup Validation Scripts:**
   - Implement API contract validation
   - Implement TODO table validation
   - Create integration test suite

4. **Configure Notifications:**
   - Set up PR status updates
   - Configure failure alerts
   - Test notification flow

### Future Enhancements

1. **Performance Benchmarking:**
   - Track build times
   - Monitor test execution
   - Optimize slow tests

2. **Test Coverage:**
   - Integration with codecov or similar
   - Coverage reports per agent
   - Combined coverage reports

3. **Security Scanning:**
   - Dependency vulnerability scanning
   - Code security analysis
   - Secret detection

---

## References

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Parallel Agent Workflow](./PARALLEL_CURSOR_AGENTS_WORKFLOW.md)
- [Development Environment](./DEVELOPMENT_ENVIRONMENT.md)
- [Coordination Guidelines](../agents/shared/COORDINATION.md)
- [CI Strategy](../agents/shared/CI.md)

---

**Status:** ✅ Enhancement plan complete, ready for implementation

**Recommendation:** Start with GitHub Actions enhancements, add validation scripts, then test with a sample parallel development session.
