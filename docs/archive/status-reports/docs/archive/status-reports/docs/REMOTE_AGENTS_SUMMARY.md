# Remote Agents Summary

**Date:** 2025-01-20
**Purpose:** Quick reference for all remote agent connection details
**Status:** ✅ **Complete Reference**

---

## Current System

**Hostname:** `Davids-iMac.local`
**OS:** macOS
**Role:** Local development machine

---

## Remote Agents Overview

| Agent | Host | User | Project Path | SSH Command |
|-------|------|------|--------------|-------------|
| **Ubuntu Agent** | `192.168.192.57` | `david` | `~/ib_box_spread_full_universal` | `ssh david@192.168.192.57` |
| **macOS M4 Agent** | `192.168.192.141` | `davidl` | `/Users/davidl/Projects/Trading/ib_box_spread_full_universal` | `ssh davidl@192.168.192.141` |

---

## Ubuntu Agent

**Connection:**

```bash
ssh david@192.168.192.57
cd ~/ib_box_spread_full_universal
```

**Setup GitHub Actions Runner:**

```bash
ssh david@192.168.192.57 "cd ~/ib_box_spread_full_universal && bash scripts/setup_github_runner_ubuntu.sh https://github.com/YOUR_USERNAME/YOUR_REPO YOUR_TOKEN ubuntu-agent"
```

**Collect System Info:**

```bash
ssh david@192.168.192.57 "cd ~/ib_box_spread_full_universal && python3 scripts/collect_system_info_python.py" > system_info_ubuntu.json
```

---

## macOS M4 Agent

**Connection:**

```bash
ssh davidl@192.168.192.141
cd /Users/davidl/Projects/Trading/ib_box_spread_full_universal
```

**Setup GitHub Actions Runner:**

```bash
ssh davidl@192.168.192.141 "cd /Users/davidl/Projects/Trading/ib_box_spread_full_universal && bash scripts/setup_github_runner_macos.sh https://github.com/YOUR_USERNAME/YOUR_REPO YOUR_TOKEN macos-m4-agent"
```

**Collect System Info:**

```bash
ssh davidl@192.168.192.141 "cd /Users/davidl/Projects/Trading/ib_box_spread_full_universal && python3 scripts/collect_system_info_python.py" > system_info_macos.json
```

**Verify Apple Intelligence (M4):**

```bash
ssh davidl@192.168.192.141 "sysctl machdep.cpu.brand_string"

# Should show: Apple M4
```

---

## Quick Setup (Both Agents)

### Ubuntu Agent

```bash

# One-command setup

ssh david@192.168.192.57 "cd ~/ib_box_spread_full_universal && bash scripts/setup_github_runner_ubuntu.sh https://github.com/YOUR_USERNAME/YOUR_REPO YOUR_LINUX_TOKEN ubuntu-agent"
```

### macOS M4 Agent

```bash

# One-command setup

ssh davidl@192.168.192.141 "cd /Users/davidl/Projects/Trading/ib_box_spread_full_universal && bash scripts/setup_github_runner_macos.sh https://github.com/YOUR_USERNAME/YOUR_REPO YOUR_MACOS_TOKEN macos-m4-agent"
```

---

## System Information Collection

### Both Agents

```bash

# Ubuntu

ssh david@192.168.192.57 "cd ~/ib_box_spread_full_universal && python3 scripts/collect_system_info_python.py" > system_info_ubuntu.json

# macOS M4

ssh davidl@192.168.192.141 "cd /Users/davidl/Projects/Trading/ib_box_spread_full_universal && python3 scripts/collect_system_info_python.py" > system_info_macos.json

# View results

cat system_info_ubuntu.json | jq .
cat system_info_macos.json | jq .
```

---

## Verification

### Test Connections

```bash

# Ubuntu Agent

ssh david@192.168.192.57 "hostname && pwd && ls -la ~/ib_box_spread_full_universal | head -5"

# macOS M4 Agent

ssh davidl@192.168.192.141 "hostname && pwd && ls -la /Users/davidl/Projects/Trading/ib_box_spread_full_universal | head -5"
```

### Check Runners (After Setup)

```bash

# Ubuntu Agent

ssh david@192.168.192.57 "cd ~/actions-runner && sudo ./svc.sh status"

# macOS M4 Agent

ssh davidl@192.168.192.141 "cd ~/actions-runner && ./svc.sh status"
```

---

## SSH Config Reference

Add to `~/.ssh/config` on local machine:

```ssh-config

# Ubuntu Agent

Host cursor-ubuntu
  HostName 192.168.192.57
  User david
  IdentityFile ~/.ssh/cursor_ubuntu_id_ed25519
  StrictHostKeyChecking accept-new

# macOS M4 Agent

Host cursor-m4-mac
  HostName 192.168.192.141
  User davidl
  IdentityFile ~/.ssh/cursor_m4_id_ed25519
  StrictHostKeyChecking accept-new
```

Then use:

```bash
ssh cursor-ubuntu
ssh cursor-m4-mac
```

---

## Documentation Reference

| Document | Purpose |
|----------|---------|
| [Agent Hostnames](./AGENT_HOSTNAMES.md) | Complete hostname reference |
| [Ubuntu Agent Setup Commands](./UBUNTU_AGENT_SETUP_COMMANDS.md) | Ubuntu-specific commands |
| [macOS M4 Agent Setup Commands](./MACOS_M4_AGENT_SETUP_COMMANDS.md) | macOS-specific commands |
| [Self-Hosted Runner Setup](./SELF_HOSTED_RUNNER_SETUP.md) | Detailed runner installation |
| [CI/CD Quick Start](./CI_CD_QUICK_START.md) | Quick CI/CD setup reference |
| [Development Environment](./DEVELOPMENT_ENVIRONMENT.md) | System specifications |

---

## Quick Command Cheat Sheet

```bash

# Connect

ssh david@192.168.192.57        # Ubuntu
ssh davidl@192.168.192.141      # macOS M4

# Setup Runner (one-command)

ssh david@192.168.192.57 "cd ~/ib_box_spread_full_universal && bash scripts/setup_github_runner_ubuntu.sh <REPO> <TOKEN> ubuntu-agent"
ssh davidl@192.168.192.141 "cd /Users/davidl/Projects/Trading/ib_box_spread_full_universal && bash scripts/setup_github_runner_macos.sh <REPO> <TOKEN> macos-m4-agent"

# Collect System Info

ssh david@192.168.192.57 "cd ~/ib_box_spread_full_universal && python3 scripts/collect_system_info_python.py" > system_info_ubuntu.json
ssh davidl@192.168.192.141 "cd /Users/davidl/Projects/Trading/ib_box_spread_full_universal && python3 scripts/collect_system_info_python.py" > system_info_macos.json

# Verify Runner Status

ssh david@192.168.192.57 "cd ~/actions-runner && sudo ./svc.sh status"
ssh davidl@192.168.192.141 "cd ~/actions-runner && ./svc.sh status"
```

---

**Last Updated:** 2025-01-20
**Status:** Complete - All agent details documented
