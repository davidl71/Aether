# macOS M4 Agent Setup Commands

**Date:** 2025-01-20
**Purpose:** Quick reference commands for macOS M4 agent setup
**Agent:** `davidl@192.168.192.141`
**Project Path:** `/Users/davidl/Projects/Trading/ib_box_spread_full_universal`

---

## Quick Connection

```bash

# Connect to macOS M4 agent

ssh davidl@192.168.192.141

# Navigate to project

cd /Users/davidl/Projects/Trading/ib_box_spread_full_universal
```

---

## GitHub Actions Runner Setup

### Step 1: Get Registration Token

1. Go to: GitHub → Repository → Settings → Actions → Runners
2. Click "New self-hosted runner"
3. Select "macOS"
4. Copy the registration token

### Step 2: Setup Runner

```bash

# Connect to macOS M4 agent

ssh davidl@192.168.192.141

# Navigate to project

cd /Users/davidl/Projects/Trading/ib_box_spread_full_universal

# Run setup script

bash scripts/setup_github_runner_macos.sh \
    https://github.com/YOUR_USERNAME/YOUR_REPO \
    YOUR_REGISTRATION_TOKEN \
    macos-m4-agent
```

**One-liner (from local machine):**

```bash
ssh davidl@192.168.192.141 "cd /Users/davidl/Projects/Trading/ib_box_spread_full_universal && bash scripts/setup_github_runner_macos.sh https://github.com/YOUR_USERNAME/YOUR_REPO YOUR_TOKEN macos-m4-agent"
```

---

## System Information Collection

```bash

# From local machine

ssh davidl@192.168.192.141 "cd /Users/davidl/Projects/Trading/ib_box_spread_full_universal && python3 scripts/collect_system_info_python.py" > system_info_macos.json

# View results

cat system_info_macos.json | jq .
```

---

## Verification Commands

### Test Connection

```bash

# Test SSH access

ssh davidl@192.168.192.141 "hostname && pwd"

# Test project access

ssh davidl@192.168.192.141 "cd /Users/davidl/Projects/Trading/ib_box_spread_full_universal && pwd && ls -la"

# Test runner status

ssh davidl@192.168.192.141 "cd ~/actions-runner && ~/actions-runner/svc.sh status"
```

### Check Runner Logs

```bash

# View runner logs

ssh davidl@192.168.192.141 "log show --predicate 'process == \"Runner.Listener\"' --last 50"

# Follow logs in real-time

ssh davidl@192.168.192.141 "log show --predicate 'process == \"Runner.Listener\"' --last 5m --style syslog"
```

---

## Useful Commands

### Check Runner Status

```bash
ssh davidl@192.168.192.141 "cd ~/actions-runner && ./svc.sh status"
```

### Restart Runner

```bash
ssh davidl@192.168.192.141 "cd ~/actions-runner && ./svc.sh restart"
```

### Stop Runner

```bash
ssh davidl@192.168.192.141 "cd ~/actions-runner && ./svc.sh stop"
```

### Start Runner

```bash
ssh davidl@192.168.192.141 "cd ~/actions-runner && ./svc.sh start"
```

---

## Project-Specific Commands

### Run Tests

```bash
ssh davidl@192.168.192.141 "cd /Users/davidl/Projects/Trading/ib_box_spread_full_universal && bash scripts/run_tests.sh"
```

### Build Project

```bash
ssh davidl@192.168.192.141 "cd /Users/davidl/Projects/Trading/ib_box_spread_full_universal && cmake --build build"
```

### Pull Latest Changes

```bash
ssh davidl@192.168.192.141 "cd /Users/davidl/Projects/Trading/ib_box_spread_full_universal && git pull origin main"
```

---

## Troubleshooting

### Can't Connect

```bash

# Test connectivity

ping 192.168.192.141

# Test SSH port

nc -zv 192.168.192.141 22

# Verbose SSH

ssh -v davidl@192.168.192.141
```

### Runner Not Working

```bash

# Check runner process

ssh davidl@192.168.192.141 "ps aux | grep Runner.Listener"

# Check service status

ssh davidl@192.168.192.141 "cd ~/actions-runner && ./svc.sh status"

# View recent logs

ssh davidl@192.168.192.141 "log show --predicate 'process == \"Runner.Listener\"' --since '10 minutes ago'"
```

---

## Apple Intelligence Features

### Verify Apple Intelligence Available

```bash
ssh davidl@192.168.192.141 "sysctl machdep.cpu.brand_string"

# Should show "Apple M4" or similar

# Check if Neural Engine available

ssh davidl@192.168.192.141 "sysctl machdep.cpu.brand_string | grep -E 'M[1-4]'"
```

### System Information

```bash

# macOS version

ssh davidl@192.168.192.141 "sw_vers"

# CPU info

ssh davidl@192.168.192.141 "sysctl machdep.cpu.brand_string hw.ncpu"

# Memory

ssh davidl@192.168.192.141 "sysctl hw.memsize"
```

---

## References

- [Agent Hostnames](./AGENT_HOSTNAMES.md) - Complete hostname reference
- [Self-Hosted Runner Setup](./SELF_HOSTED_RUNNER_SETUP.md) - Detailed setup guide
- [Apple Intelligence Quick Reference](./APPLE_INTELLIGENCE_QUICK_REFERENCE.md) - AI features guide

---

**Agent:** `davidl@192.168.192.141`
**Project:** `/Users/davidl/Projects/Trading/ib_box_spread_full_universal`
