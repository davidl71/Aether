# CI/CD Quick Start Guide

**Date:** 2025-01-20
**Purpose:** Quick reference for setting up CI/CD with self-hosted runners
**Status:** ✅ **Quick Start Guide**

---

## Current System

**Hostname:** `Davids-iMac.local`
**OS:** macOS
**Role:** Local development machine

## Remote Agents

**Ubuntu Agent:**

- **Host:** `david@192.168.192.57`
- **Project Path:** `~/ib_box_spread_full_universal`

**macOS M4 Agent:**

- **Host:** `davidl@192.168.192.141`
- **Project Path:** `/Users/davidl/Projects/Trading/Aether`

---

## Quick Setup (5 Minutes)

### 1. Get Registration Tokens

**From GitHub:**

1. Go to: Repository → **Settings** → **Actions** → **Runners**
2. Click **"New self-hosted runner"**
3. Select **Linux** (Ubuntu) or **macOS** (M4)
4. Copy the registration token

### 2. Setup Ubuntu Agent Runner

**SSH to Ubuntu agent:**

```bash
ssh david@192.168.192.57

# or: ssh cursor-ubuntu  (if SSH alias configured)
```

**Run setup script:**

```bash
cd ~/ib_box_spread_full_universal
bash scripts/setup_github_runner_ubuntu.sh \
    https://github.com/YOUR_USERNAME/YOUR_REPO \
    YOUR_REGISTRATION_TOKEN \
    ubuntu-agent
```

### 3. Setup macOS M4 Agent Runner

**SSH to macOS M4 agent:**

```bash
ssh davidl@192.168.192.141

# or: ssh cursor-m4-mac  (if SSH alias configured)
```

**Run setup script:**

```bash
cd /Users/davidl/Projects/Trading/Aether
bash scripts/setup_github_runner_macos.sh \
    https://github.com/YOUR_USERNAME/YOUR_REPO \
    YOUR_REGISTRATION_TOKEN \
    macos-m4-agent
```

### 4. Verify Runners

**In GitHub:**

- Go to: Repository → **Settings** → **Actions** → **Runners**
- Verify both runners show as **"Online"** (green status)

### 5. Test Workflow

**Create test PR:**

- Make a small change
- Push to branch
- Create PR
- Verify workflows run on self-hosted runners

---

## What's Created

### Workflows

✅ **`.github/workflows/parallel-agents-ci.yml`**

- Parallel agent testing (Ubuntu + macOS)
- Integration tests
- Coordination validation

### Scripts

✅ **`scripts/setup_github_runner_ubuntu.sh`**

- Automated Ubuntu runner setup

✅ **`scripts/setup_github_runner_macos.sh`**

- Automated macOS runner setup

### Validation Scripts

✅ **`scripts/validate_api_contract.sh`**

- API contract validation

✅ **`scripts/validate_todo_table.sh`**

- TODO table validation

✅ **`scripts/run_integration_tests.sh`**

- Integration test runner

---

## References

- [Self-Hosted Runner Setup](./SELF_HOSTED_RUNNER_SETUP.md) - Detailed setup guide
- [CI/CD Enhancement Plan](./CI_CD_ENHANCEMENT_PLAN.md) - Complete enhancement plan
- [Parallel Agents Workflow](./PARALLEL_CURSOR_AGENTS_WORKFLOW.md) - Parallel development guide

---

**Ready to go!** Run the setup scripts on both agents and you're done! 🚀
