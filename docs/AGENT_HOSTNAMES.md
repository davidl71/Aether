# Remote Agent Hostnames and Paths

**Date:** 2025-01-20
**Purpose:** Reference for remote agent connection details
**Status:** ✅ **Active Reference**

---

## Current System

**Hostname:** `Davids-iMac.local`
**OS:** macOS
**Role:** Local development machine

---

## Ubuntu Agent

**Connection Details:**

- **IP Address:** `192.168.192.57`
- **User:** `david`
- **SSH Command:** `ssh david@192.168.192.57`
- **SSH Alias:** `cursor-ubuntu` (if configured in `~/.ssh/config`)
- **Project Path:** `/home/david/Projects/trading/ib_box_spread_full_universal`
- **Project Path (Short):** `~/ib_box_spread_full_universal`

**Full SSH Connection:**

```bash
ssh david@192.168.192.57
cd ~/ib_box_spread_full_universal
```

**With SSH Alias (if configured):**

```bash
ssh cursor-ubuntu
cd ~/ib_box_spread_full_universal
```

**Setup GitHub Actions Runner:**

```bash
ssh david@192.168.192.57
cd ~/ib_box_spread_full_universal
bash scripts/setup_github_runner_ubuntu.sh \
    https://github.com/YOUR_USERNAME/YOUR_REPO \
    YOUR_REGISTRATION_TOKEN \
    ubuntu-agent
```

---

## macOS M4 Agent

**Connection Details:**

- **IP Address:** `192.168.192.141`
- **User:** `davidl`
- **SSH Command:** `ssh davidl@192.168.192.141`
- **SSH Alias:** `cursor-m4-mac` (if configured in `~/.ssh/config`)
- **Project Path:** `/Users/davidl/Projects/Trading/ib_box_spread_full_universal`

**Full SSH Connection:**

```bash
ssh davidl@192.168.192.141
cd /Users/davidl/Projects/Trading/ib_box_spread_full_universal
```

**With SSH Alias (if configured):**

```bash
ssh cursor-m4-mac
cd /Users/davidl/Projects/Trading/ib_box_spread_full_universal
```

**Setup GitHub Actions Runner:**

```bash
ssh davidl@192.168.192.141
cd /Users/davidl/Projects/Trading/ib_box_spread_full_universal
bash scripts/setup_github_runner_macos.sh \
    https://github.com/YOUR_USERNAME/YOUR_REPO \
    YOUR_REGISTRATION_TOKEN \
    macos-m4-agent
```

---

## Quick Reference

### SSH Connection Commands

```bash

# Ubuntu Agent

ssh david@192.168.192.57

# macOS M4 Agent

ssh davidl@192.168.192.141

# or: ssh cursor-m4-mac  (if SSH alias configured)
```

### Project Paths

```bash

# Ubuntu Agent

cd ~/ib_box_spread_full_universal

# macOS M4 Agent

cd /Users/davidl/Projects/Trading/ib_box_spread_full_universal
```

### Runner Setup Commands

```bash

# Ubuntu Agent

cd ~/ib_box_spread_full_universal
bash scripts/setup_github_runner_ubuntu.sh \
    <REPO_URL> <TOKEN> ubuntu-agent

# macOS M4 Agent

cd /Users/davidl/Projects/Trading/ib_box_spread_full_universal
bash scripts/setup_github_runner_macos.sh \
    <REPO_URL> <TOKEN> macos-m4-agent
```

---

## Configuration Files

### SSH Config (Local Machine)

Location: `~/.ssh/config`

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

---

## Verification

### Test Ubuntu Connection

```bash

# Test SSH connection

ssh david@192.168.192.57 "hostname && pwd"

# Test project access

ssh david@192.168.192.57 "cd ~/ib_box_spread_full_universal && pwd && ls -la"
```

### Test macOS Connection

```bash

# Test SSH connection

ssh davidl@192.168.192.141 "hostname && pwd"

# Test project access

ssh davidl@192.168.192.141 "cd /Users/davidl/Projects/Trading/ib_box_spread_full_universal && pwd && ls -la"
```

---

## System Information Collection

### Collect System Info from Ubuntu Agent

```bash
ssh david@192.168.192.57 "cd ~/ib_box_spread_full_universal && python3 scripts/collect_system_info_python.py" > system_info_ubuntu.json
```

### Collect System Info from macOS M4 Agent

```bash
ssh davidl@192.168.192.141 "cd /Users/davidl/Projects/Trading/ib_box_spread_full_universal && python3 scripts/collect_system_info_python.py" > system_info_macos.json
```

---

## References

- [Development Environment](./DEVELOPMENT_ENVIRONMENT.md) - Complete environment documentation
- [Self-Hosted Runner Setup](./SELF_HOSTED_RUNNER_SETUP.md) - Runner installation guide
- [CI/CD Quick Start](./CI_CD_QUICK_START.md) - Quick setup reference
- [Remote Development Workflow](./REMOTE_DEVELOPMENT_WORKFLOW.md) - Remote development guide

---

**Last Updated:** 2025-01-20
**Status:** Active reference - update when agent details change
