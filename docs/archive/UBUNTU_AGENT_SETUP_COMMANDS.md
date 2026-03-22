# Ubuntu Agent Setup Commands

**Date:** 2025-01-20
**Purpose:** Quick reference commands for Ubuntu agent setup
**Agent:** `david@192.168.192.57`
**Project Path:** `~/ib_box_spread_full_universal`

---

## Quick Connection

```bash

# Connect to Ubuntu agent

ssh david@192.168.192.57

# Navigate to project

cd ~/ib_box_spread_full_universal
```

---

## GitHub Actions Runner Setup

### Step 1: Get Registration Token

1. Go to: GitHub → Repository → Settings → Actions → Runners
2. Click "New self-hosted runner"
3. Select "Linux"
4. Copy the registration token

### Step 2: Setup Runner

```bash

# Connect to Ubuntu agent

ssh david@192.168.192.57

# Navigate to project

cd ~/ib_box_spread_full_universal

# Run setup script

bash scripts/setup_github_runner_ubuntu.sh \
    https://github.com/YOUR_USERNAME/YOUR_REPO \
    YOUR_REGISTRATION_TOKEN \
    ubuntu-agent
```

**One-liner (from local machine):**

```bash
ssh david@192.168.192.57 "cd ~/ib_box_spread_full_universal && bash scripts/setup_github_runner_ubuntu.sh https://github.com/YOUR_USERNAME/YOUR_REPO YOUR_TOKEN ubuntu-agent"
```

---

## System Information Collection

```bash

# From local machine

ssh david@192.168.192.57 "cd ~/ib_box_spread_full_universal && python3 scripts/collect_system_info_python.py" > system_info_ubuntu.json

# View results

cat system_info_ubuntu.json | jq .
```

---

## Verification Commands

### Test Connection

```bash

# Test SSH access

ssh david@192.168.192.57 "hostname && pwd"

# Test project access

ssh david@192.168.192.57 "cd ~/ib_box_spread_full_universal && pwd && ls -la"

# Test runner status

ssh david@192.168.192.57 "cd ~/actions-runner && sudo ~/actions-runner/svc.sh status"
```

### Check Runner Logs

```bash

# View runner logs

ssh david@192.168.192.57 "sudo journalctl -u actions.runner.* -n 50"

# Follow logs in real-time

ssh david@192.168.192.57 "sudo journalctl -u actions.runner.* -f"
```

---

## Useful Commands

### Check Runner Status

```bash
ssh david@192.168.192.57 "cd ~/actions-runner && sudo ./svc.sh status"
```

### Restart Runner

```bash
ssh david@192.168.192.57 "cd ~/actions-runner && sudo ./svc.sh restart"
```

### Stop Runner

```bash
ssh david@192.168.192.57 "cd ~/actions-runner && sudo ./svc.sh stop"
```

### Start Runner

```bash
ssh david@192.168.192.57 "cd ~/actions-runner && sudo ./svc.sh start"
```

---

## Project-Specific Commands

### Run Tests

```bash
ssh david@192.168.192.57 "cd ~/ib_box_spread_full_universal && bash scripts/run_tests.sh"
```

### Build Project

```bash
ssh david@192.168.192.57 "cd ~/ib_box_spread_full_universal && cmake --build build"
```

### Pull Latest Changes

```bash
ssh david@192.168.192.57 "cd ~/ib_box_spread_full_universal && git pull origin main"
```

---

## Troubleshooting

### Can't Connect

```bash

# Test connectivity

ping 192.168.192.57

# Test SSH port

nc -zv 192.168.192.57 22

# Verbose SSH

ssh -v david@192.168.192.57
```

### Runner Not Working

```bash

# Check runner process

ssh david@192.168.192.57 "ps aux | grep Runner.Listener"

# Check service status

ssh david@192.168.192.57 "sudo systemctl status actions.runner.*"

# View recent logs

ssh david@192.168.192.57 "sudo journalctl -u actions.runner.* --since '10 minutes ago'"
```

---

## References

- [Agent Hostnames](./AGENT_HOSTNAMES.md) - Complete hostname reference
- [Self-Hosted Runner Setup](./SELF_HOSTED_RUNNER_SETUP.md) - Detailed setup guide
- [CI/CD Quick Start](./CI_CD_QUICK_START.md) - Quick setup reference

---

**Agent:** `david@192.168.192.57`
**Project:** `~/ib_box_spread_full_universal`
