# Self-Hosted GitHub Actions Runner Setup

**Date:** 2025-01-20
**Purpose:** Setup instructions for using Ubuntu and macOS M4 remote agents as GitHub Actions runners
**Status:** ✅ **Complete Setup Guide**

---

## Overview

This guide shows how to set up your Ubuntu and macOS M4 remote Cursor agents as self-hosted GitHub Actions runners. This allows you to:

- ✅ Run tests on your actual development hardware
- ✅ Faster builds (no cloud queue times)
- ✅ Access to local dependencies and tools
- ✅ No GitHub Actions minute usage (unlimited minutes)
- ✅ Test on real hardware configurations

---

## System Information

### Current System

**Hostname:** `Davids-iMac.local`
**OS:** macOS
**Role:** Local development machine

### Remote Agents

**Ubuntu Agent:**

- **SSH Alias:** `cursor-ubuntu`
- **Host:** `192.168.192.57`
- **User:** `david`
- **Project Path:** `/home/david/ib_box_spread_full_universal`
- **SSH Command:** `ssh david@192.168.192.57`
- **Role:** GitHub Actions runner for Ubuntu/Linux tests

**macOS M4 Agent:**

- **SSH Alias:** `cursor-m4-mac`
- **Host:** `192.168.192.141`
- **User:** `davidl`
- **Project Path:** `/Users/davidl/Projects/Trading/ib_box_spread_full_universal`
- **SSH Command:** `ssh davidl@192.168.192.141`
- **Role:** GitHub Actions runner for macOS/Apple Silicon tests

---

## Prerequisites

### On Each Remote Agent

1. **SSH Access:**
   - SSH enabled and accessible
   - User account with appropriate permissions
   - SSH key authentication configured

2. **Development Tools:**
   - Git installed
   - Required build tools (CMake, Rust, Node.js, etc.)
   - Project dependencies installed

3. **Network Access:**
   - Internet connection for GitHub communication
   - Outbound HTTPS access (port 443)
   - GitHub.com accessible

---

## Setup Steps

### Step 1: Get Runner Registration Token

**From GitHub Repository:**

1. Go to your repository on GitHub
2. Navigate to: **Settings → Actions → Runners**
3. Click **"New self-hosted runner"**
4. Select OS: **Linux** (for Ubuntu) or **macOS** (for macOS M4)
5. Copy the registration token (keep it secure!)

**Or via GitHub CLI:**

```bash

# Install GitHub CLI if not installed
# macOS: brew install gh
# Ubuntu: sudo apt install gh

# Authenticate

gh auth login

# Get registration token

gh api repos/:owner/:repo/actions/runners/registration-token --method POST
```

---

### Step 2: Install Runner on Ubuntu Agent

**Connect to Ubuntu Agent:**

```bash

# Option 1: Using SSH alias (if configured)

ssh cursor-ubuntu

# Option 2: Direct connection

ssh david@192.168.192.57
```

**Download and Install Runner:**

```bash

# Create actions-runner directory

mkdir -p ~/actions-runner
cd ~/actions-runner

# Download runner (replace X.Y.Z with latest version)

curl -o actions-runner-linux-x64-2.311.0.tar.gz -L https://github.com/actions/runner/releases/download/v2.311.0/actions-runner-linux-x64-2.311.0.tar.gz

# Extract

tar xzf ./actions-runner-linux-x64-2.311.0.tar.gz

# Configure runner (replace TOKEN with your registration token)

./config.sh --url https://github.com/YOUR_USERNAME/YOUR_REPO --token YOUR_REGISTRATION_TOKEN --name ubuntu-agent --labels ubuntu,linux --work _work

# Install as systemd service (runs automatically)

sudo ./svc.sh install
sudo ./svc.sh start

# Check status

sudo ./svc.sh status
```

**Verify Runner:**

```bash

# Check runner is running

ps aux | grep Runner.Listener

# View logs

sudo journalctl -u actions.runner.* -f
```

---

### Step 3: Install Runner on macOS M4 Agent

**Connect to macOS M4 Agent:**

```bash

# Option 1: Using SSH alias (if configured)

ssh cursor-m4-mac

# Option 2: Direct connection

ssh davidl@192.168.192.141
```

**Download and Install Runner:**

```bash

# Create actions-runner directory

mkdir -p ~/actions-runner
cd ~/actions-runner

# Download runner (replace X.Y.Z with latest version)

curl -o actions-runner-osx-arm64-2.311.0.tar.gz -L https://github.com/actions/runner/releases/download/v2.311.0/actions-runner-osx-arm64-2.311.0.tar.gz

# Extract

tar xzf ./actions-runner-osx-arm64-2.311.0.tar.gz

# Configure runner (replace TOKEN with your registration token)

./config.sh --url https://github.com/YOUR_USERNAME/YOUR_REPO --token YOUR_REGISTRATION_TOKEN --name macos-m4-agent --labels macos,apple-silicon,m4 --work _work

# Install as launchd service (runs automatically)

./svc.sh install
./svc.sh start

# Check status

./svc.sh status
```

**Verify Runner:**

```bash

# Check runner is running

ps aux | grep Runner.Listener

# View logs

log show --predicate 'process == "Runner.Listener"' --last 5m
```

---

## Configuration Details

### Runner Labels

**Ubuntu Agent:**

- `ubuntu` - Platform identifier
- `linux` - OS type
- `self-hosted` - Auto-added by GitHub

**macOS M4 Agent:**

- `macos` - Platform identifier
- `apple-silicon` - Chip type
- `m4` - Specific chip model
- `self-hosted` - Auto-added by GitHub

**Usage in Workflows:**

```yaml
runs-on: self-hosted
labels: [ubuntu, linux]  # For Ubuntu agent

runs-on: self-hosted
labels: [macos, apple-silicon, m4]  # For macOS M4 agent
```

### Work Directory

**Default:** `~/actions-runner/_work`

**Custom Location:**

```bash

# During configuration

./config.sh ... --work /path/to/work

# Or edit config after setup

nano ~/actions-runner/.runner
```

---

## Workflow Configuration

### Update Workflow Files

The enhanced workflow (`.github/workflows/parallel-agents-ci.yml`) already uses self-hosted runners:

```yaml
jobs:
  ubuntu-agent-tests:
    runs-on: self-hosted
    labels: [ubuntu, linux]

  macos-agent-tests:
    runs-on: self-hosted
    labels: [macos, apple-silicon, m4]
```

**Note:** If runners aren't available, GitHub Actions will fall back to cloud runners. To force self-hosted only, use:

```yaml
runs-on: [self-hosted, ubuntu, linux]  # Must match all labels
```

---

## Maintenance

### Updating Runners

**Check for Updates:**

```bash
cd ~/actions-runner
./run.sh --check-update
```

**Update Runner:**

```bash

# Stop service

sudo ./svc.sh stop  # Linux
./svc.sh stop       # macOS

# Download latest version

curl -o actions-runner-linux-x64-2.311.0.tar.gz -L https://github.com/actions/runner/releases/download/v2.311.0/actions-runner-linux-x64-2.311.0.tar.gz

# Extract (overwrites old files)

tar xzf ./actions-runner-linux-x64-2.311.0.tar.gz

# Restart service

sudo ./svc.sh start  # Linux
./svc.sh start       # macOS
```

### Monitoring

**View Runner Status:**

- GitHub: Repository → Settings → Actions → Runners
- See runner status, last job, labels

**Check Logs:**

```bash

# Ubuntu (systemd)

sudo journalctl -u actions.runner.* -f

# macOS (launchd)

log show --predicate 'process == "Runner.Listener"' --last 1h
```

**Runner Health:**

```bash

# Check runner process

ps aux | grep Runner.Listener

# Check disk space

df -h ~/actions-runner/_work

# Check network connectivity

curl -I https://github.com
```

---

## Security Considerations

### 1. Runner Isolation

**Best Practices:**

- Use dedicated user accounts for runners
- Restrict file system access
- Limit network access if possible
- Use separate runners for different projects

**User Account:**

```bash

# Create dedicated user

sudo useradd -m -s /bin/bash actions-runner

# Run as that user

sudo -u actions-runner ./config.sh ...
```

### 2. Secret Management

**GitHub Secrets:**

- Secrets are encrypted in transit
- Secrets are not logged in runner logs
- Secrets are cleared after job completion

**Best Practices:**

- Never commit secrets to repository
- Use GitHub Secrets for sensitive data
- Rotate secrets regularly
- Use least-privilege access

### 3. Network Security

**Firewall Rules:**

```bash

# Allow GitHub Actions communication
# Outbound HTTPS (port 443) to github.com
# Outbound HTTPS (port 443) to api.github.com
```

**VPN Considerations:**

- Runners need direct access to GitHub
- VPN may interfere with runner communication
- Test runner connectivity after VPN setup

---

## Troubleshooting

### Runner Not Appearing in GitHub

**Issue:** Runner configured but not visible in GitHub

**Solutions:**

1. Check registration token is valid
2. Verify network connectivity to GitHub
3. Check runner logs for errors
4. Restart runner service

```bash

# Check logs

sudo journalctl -u actions.runner.* -n 50

# Restart service

sudo ./svc.sh restart  # Linux
./svc.sh restart       # macOS
```

### Jobs Not Running on Self-Hosted Runner

**Issue:** Jobs queue but don't run on self-hosted runner

**Solutions:**

1. Verify runner is online (green status in GitHub)
2. Check runner labels match workflow requirements
3. Verify runner has required tools installed
4. Check runner disk space

```bash

# Check runner status

cd ~/actions-runner
./run.sh --version

# Check labels

cat .runner | grep -i label

# Check disk space

df -h ~/actions-runner/_work
```

### Runner Service Not Starting

**Issue:** Service fails to start

**Solutions:**

1. Check service logs
2. Verify configuration file
3. Check file permissions
4. Reinstall service

```bash

# Check service status

sudo ./svc.sh status

# Check configuration

cat .runner

# Reinstall service

sudo ./svc.sh uninstall
sudo ./svc.sh install
sudo ./svc.sh start
```

---

## Automation Scripts

### Setup Script for Ubuntu Agent

**File:** `scripts/setup_github_runner_ubuntu.sh`

```bash

#!/usr/bin/env bash
# Setup GitHub Actions runner on Ubuntu agent

set -euo pipefail

REPO_URL="${1:-}"
REGISTRATION_TOKEN="${2:-}"
RUNNER_NAME="${3:-ubuntu-agent}"

if [ -z "$REPO_URL" ] || [ -z "$REGISTRATION_TOKEN" ]; then
    echo "Usage: $0 <REPO_URL> <REGISTRATION_TOKEN> [RUNNER_NAME]"
    echo "Example: $0 https://github.com/user/repo ghs_TOKEN ubuntu-agent"
    exit 1
fi

mkdir -p ~/actions-runner
cd ~/actions-runner

# Download latest runner

RUNNER_VERSION="2.311.0"
curl -o actions-runner-linux-x64-${RUNNER_VERSION}.tar.gz -L \
    https://github.com/actions/runner/releases/download/v${RUNNER_VERSION}/actions-runner-linux-x64-${RUNNER_VERSION}.tar.gz

tar xzf actions-runner-linux-x64-${RUNNER_VERSION}.tar.gz

# Configure

./config.sh --url "$REPO_URL" --token "$REGISTRATION_TOKEN" \
    --name "$RUNNER_NAME" --labels ubuntu,linux --work _work

# Install service

sudo ./svc.sh install
sudo ./svc.sh start

echo "✅ GitHub Actions runner installed and started"
```

### Setup Script for macOS M4 Agent

**File:** `scripts/setup_github_runner_macos.sh`

```bash

#!/usr/bin/env bash
# Setup GitHub Actions runner on macOS M4 agent

set -euo pipefail

REPO_URL="${1:-}"
REGISTRATION_TOKEN="${2:-}"
RUNNER_NAME="${3:-macos-m4-agent}"

if [ -z "$REPO_URL" ] || [ -z "$REGISTRATION_TOKEN" ]; then
    echo "Usage: $0 <REPO_URL> <REGISTRATION_TOKEN> [RUNNER_NAME]"
    echo "Example: $0 https://github.com/user/repo ghs_TOKEN macos-m4-agent"
    exit 1
fi

mkdir -p ~/actions-runner
cd ~/actions-runner

# Download latest runner

RUNNER_VERSION="2.311.0"
curl -o actions-runner-osx-arm64-${RUNNER_VERSION}.tar.gz -L \
    https://github.com/actions/runner/releases/download/v${RUNNER_VERSION}/actions-runner-osx-arm64-${RUNNER_VERSION}.tar.gz

tar xzf actions-runner-osx-arm64-${RUNNER_VERSION}.tar.gz

# Configure

./config.sh --url "$REPO_URL" --token "$REGISTRATION_TOKEN" \
    --name "$RUNNER_NAME" --labels macos,apple-silicon,m4 --work _work

# Install service

./svc.sh install
./svc.sh start

echo "✅ GitHub Actions runner installed and started"
```

---

## Verification

### Test Workflow

Create a test workflow to verify runners are working:

```yaml

# .github/workflows/test-runners.yml

name: Test Runners

on:
  workflow_dispatch:

jobs:
  test-ubuntu:
    runs-on: self-hosted
    labels: [ubuntu, linux]
    steps:
      - name: Check runner
        run: |
          echo "Running on: $(uname -a)"
          echo "Hostname: $(hostname)"
          echo "User: $(whoami)"

  test-macos:
    runs-on: self-hosted
    labels: [macos, apple-silicon, m4]
    steps:
      - name: Check runner
        run: |
          echo "Running on: $(uname -a)"
          echo "Hostname: $(hostname)"
          echo "User: $(whoami)"
          sw_vers
```

Run the test workflow from GitHub Actions UI and verify both runners execute.

---

## Next Steps

1. **Setup Runners:**
   - Get registration tokens from GitHub
   - Run setup scripts on both agents
   - Verify runners appear in GitHub

2. **Test Workflows:**
   - Create test workflow
   - Verify jobs run on correct runners
   - Check logs and artifacts

3. **Monitor Usage:**
   - Track runner utilization
   - Monitor performance
   - Optimize as needed

---

## References

- [GitHub Actions Self-Hosted Runners](https://docs.github.com/en/actions/hosting-your-own-runners)
- [Runner Configuration](https://docs.github.com/en/actions/hosting-your-own-runners/managing-self-hosted-runners)
- [Runner Security](https://docs.github.com/en/actions/security-guides/security-hardening-for-github-actions)
- [Parallel Agents Workflow](./PARALLEL_CURSOR_AGENTS_WORKFLOW.md)
- [CI/CD Enhancement Plan](./CI_CD_ENHANCEMENT_PLAN.md)

---

**Status:** ✅ Setup guide complete, ready for runner installation

**Current System:** `Davids-iMac.local` (macOS)

**Next Action:** Get registration tokens from GitHub and run setup scripts on both remote agents.
