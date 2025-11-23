# Ubuntu Agent Quick Setup

**Agent:** `david@192.168.192.57`
**Project Path:** `~/ib_box_spread_full_universal`
**Current System:** `Davids-iMac.local`

---

## One-Command Setup (From Local Machine)

### Setup GitHub Actions Runner

```bash
# Get registration token from GitHub first, then:
ssh david@192.168.192.57 "cd ~/ib_box_spread_full_universal && bash scripts/setup_github_runner_ubuntu.sh https://github.com/YOUR_USERNAME/YOUR_REPO YOUR_TOKEN ubuntu-agent"
```

### Collect System Information

```bash
ssh david@192.168.192.57 "cd ~/ib_box_spread_full_universal && python3 scripts/collect_system_info_python.py" > system_info_ubuntu.json
```

---

## Step-by-Step Setup

### 1. Connect to Ubuntu Agent

```bash
ssh david@192.168.192.57
```

### 2. Navigate to Project

```bash
cd ~/ib_box_spread_full_universal
```

### 3. Setup GitHub Actions Runner

```bash
# Get registration token from GitHub first:
# Repository → Settings → Actions → Runners → New self-hosted runner → Linux

bash scripts/setup_github_runner_ubuntu.sh \
    https://github.com/YOUR_USERNAME/YOUR_REPO \
    YOUR_REGISTRATION_TOKEN \
    ubuntu-agent
```

### 4. Verify Setup

```bash
# Check runner status
cd ~/actions-runner
sudo ./svc.sh status

# View logs
sudo journalctl -u actions.runner.* -f
```

---

## Quick Verification

```bash
# From local machine - test connection
ssh david@192.168.192.57 "hostname && pwd"

# Test project access
ssh david@192.168.192.57 "cd ~/ib_box_spread_full_universal && pwd"

# Check runner (if already installed)
ssh david@192.168.192.57 "cd ~/actions-runner && sudo ./svc.sh status"
```

---

## References

- [Ubuntu Agent Setup Commands](./UBUNTU_AGENT_SETUP_COMMANDS.md) - Complete command reference
- [Agent Hostnames](./AGENT_HOSTNAMES.md) - All agent connection details
- [CI/CD Quick Start](./CI_CD_QUICK_START.md) - Quick setup guide

---

**Quick Command:** `ssh david@192.168.192.57`
