# macOS M4 Agent Quick Setup

**Agent:** `davidl@192.168.192.141`
**Project Path:** `/Users/davidl/Projects/Trading/ib_box_spread_full_universal`
**Current System:** `Davids-iMac.local`

---

## One-Command Setup (From Local Machine)

### Setup GitHub Actions Runner

```bash
# Get registration token from GitHub first, then:
ssh davidl@192.168.192.141 "cd /Users/davidl/Projects/Trading/ib_box_spread_full_universal && bash scripts/setup_github_runner_macos.sh https://github.com/YOUR_USERNAME/YOUR_REPO YOUR_TOKEN macos-m4-agent"
```

### Collect System Information

```bash
ssh davidl@192.168.192.141 "cd /Users/davidl/Projects/Trading/ib_box_spread_full_universal && python3 scripts/collect_system_info_python.py" > system_info_macos.json
```

---

## Step-by-Step Setup

### 1. Connect to macOS M4 Agent

```bash
ssh davidl@192.168.192.141
```

### 2. Navigate to Project

```bash
cd /Users/davidl/Projects/Trading/ib_box_spread_full_universal
```

### 3. Setup GitHub Actions Runner

```bash
# Get registration token from GitHub first:
# Repository → Settings → Actions → Runners → New self-hosted runner → macOS

bash scripts/setup_github_runner_macos.sh \
    https://github.com/YOUR_USERNAME/YOUR_REPO \
    YOUR_REGISTRATION_TOKEN \
    macos-m4-agent
```

### 4. Verify Setup

```bash
# Check runner status
cd ~/actions-runner
./svc.sh status

# View logs
log show --predicate 'process == "Runner.Listener"' --last 5m
```

---

## Quick Verification

```bash
# From local machine - test connection
ssh davidl@192.168.192.141 "hostname && pwd"

# Test project access
ssh davidl@192.168.192.141 "cd /Users/davidl/Projects/Trading/ib_box_spread_full_universal && pwd"

# Check runner (if already installed)
ssh davidl@192.168.192.141 "cd ~/actions-runner && ./svc.sh status"

# Verify Apple Intelligence (M4 chip)
ssh davidl@192.168.192.141 "sysctl machdep.cpu.brand_string"
```

---

## References

- [macOS M4 Agent Setup Commands](./MACOS_M4_AGENT_SETUP_COMMANDS.md) - Complete command reference
- [Agent Hostnames](./AGENT_HOSTNAMES.md) - All agent connection details
- [Apple Intelligence Quick Reference](./APPLE_INTELLIGENCE_QUICK_REFERENCE.md) - AI features guide
- [CI/CD Quick Start](./CI_CD_QUICK_START.md) - Quick setup guide

---

**Quick Command:** `ssh davidl@192.168.192.141`
