# Development Environment Documentation

**Date:** 2025-01-20
**Purpose:** Comprehensive documentation of development environment for parallel Cursor agent workflows
**Status:** ✅ **Active Documentation**

---

## Overview

This document provides comprehensive information about the development environment, including system specifications, software versions, and configuration details for both remote Cursor agents (Ubuntu and macOS M4).

**Purpose:**

- Enable informed task delegation based on hardware capabilities
- Track system specifications for optimization
- Document software versions for compatibility
- Support parallel development workflow planning

---

## Remote Agents Overview

| Agent | OS | Processor | RAM | Apple Intelligence | Primary Role |
|-------|----|-----------|-----|--------------------|--------------|
| **Ubuntu Agent** | Ubuntu Linux | [See Ubuntu Section](#ubuntu-agent) | [See Ubuntu Section](#ubuntu-agent) | ❌ No | Backend services, Linux builds, testing |
| **macOS M4 Agent** | macOS | M4 | [See macOS Section](#macos-m4-agent) | ✅ Yes | macOS development, documentation, Apple Intelligence |

---

## System Information Collection

### Automated Collection Scripts

**Script 1: Bash Script (Cross-platform)**

```bash

# Collect system information

./scripts/collect_system_info.sh > system_info_$(hostname).json

# View formatted output

cat system_info_$(hostname).json | jq .
```

**Script 2: Python Script (More reliable)**

```bash

# Collect system information

python3 scripts/collect_system_info_python.py > system_info_$(hostname).json

# View formatted output

cat system_info_$(hostname).json | jq .
```

### Manual Collection

For detailed information, use system-specific commands:

**Ubuntu:**

```bash

# OS Version

cat /etc/os-release

# CPU Information

lscpu

# Memory

free -h

# Disk Usage

df -h

# Network Interfaces

ip addr show
```

**macOS:**

```bash

# OS Version

sw_vers

# CPU Information

sysctl machdep.cpu.brand_string
sysctl hw.ncpu
sysctl hw.physicalcpu

# Memory

sysctl hw.memsize
vm_stat

# Disk Usage

df -h

# Network Interfaces

networksetup -listallhardwareports
ifconfig
```

---

## Ubuntu Agent

### System Specifications

*[To be populated after running collection script]*

```bash

# Collect information

ssh cursor-ubuntu "python3 /path/to/collect_system_info_python.py" > system_info_ubuntu.json
```

**Template:**

```json
{
  "hostname": "ubuntu-agent",
  "operating_system": "Linux",
  "os_version": "Ubuntu 22.04 LTS",
  "kernel_version": "5.15.0-xx-generic",
  "cpu_model": "Intel Core i7-xxxx / AMD Ryzen xxxx",
  "cpu_cores": 8,
  "cpu_physical_cores": 4,
  "memory_total_gb": 32,
  "disk_info": [
    {
      "filesystem": "/dev/sda1",
      "size": "500G",
      "used": "200G",
      "available": "300G",
      "mount": "/"
    }
  ]
}
```

### Development Tools

*[To be populated]*

```json
{
  "development_tools": {
    "git_version": "2.34.0",
    "cmake_version": "3.24.0",
    "rust_version": "1.75.0",
    "go_version": "1.21.0",
    "node_version": "20.10.0",
    "python3_version": "3.11.0",
    "cursor_agent_version": "0.xx.x"
  }
}
```

### Network Configuration

*[To be populated]*

```json
{
  "network_interfaces": [
    {
      "name": "eth0",
      "ip_address": "192.168.1.xxx",
      "mac_address": "xx:xx:xx:xx:xx:xx"
    }
  ]
}
```

### Capabilities

- ✅ **Linux builds and testing**
- ✅ **Docker container development**
- ✅ **CI/CD pipeline testing**
- ✅ **Backend services (Rust, Go)**
- ✅ **Cross-platform compatibility testing**
- ❌ **Apple Intelligence** (Linux only)

---

## macOS M4 Agent

### System Specifications

*[To be populated after running collection script]*

```bash

# Collect information

ssh cursor-m4-mac "python3 /path/to/collect_system_info_python.py" > system_info_macos.json
```

**Template:**

```json
{
  "hostname": "macos-m4-agent",
  "operating_system": "Darwin",
  "os_version": "14.2",
  "os_build": "23C64",
  "os_name": "macOS",
  "cpu_brand": "Apple M4",
  "cpu_cores": 10,
  "cpu_physical_cores": 10,
  "cpu_logical_cores": 10,
  "memory_total_gb": 32,
  "apple_intelligence_available": true,
  "neural_engine_available": true,
  "disk_info": [
    {
      "filesystem": "/dev/disk3s1",
      "size": "1TB",
      "used": "500GB",
      "available": "500GB",
      "mount": "/"
    }
  ]
}
```

### Development Tools

*[To be populated]*

```json
{
  "development_tools": {
    "git_version": "2.42.0",
    "cmake_version": "3.28.0",
    "rust_version": "1.75.0",
    "go_version": "1.21.0",
    "node_version": "20.10.0",
    "python3_version": "3.12.0",
    "cursor_agent_version": "0.xx.x"
  }
}
```

### Network Configuration

*[To be populated]*

```json
{
  "network_interfaces": [
    {
      "name": "en0",
      "description": "Wi-Fi",
      "ip_address": "192.168.1.xxx",
      "mac_address": "xx:xx:xx:xx:xx:xx"
    },
    {
      "name": "en1",
      "description": "Thunderbolt Bridge",
      "ip_address": "",
      "mac_address": "xx:xx:xx:xx:xx:xx"
    }
  ]
}
```

### Capabilities

- ✅ **macOS builds (Universal binaries)**
- ✅ **AppKit bundle development**
- ✅ **Xcode toolchain testing**
- ✅ **Apple Intelligence features:**
  - Writing Tools (documentation improvement)
  - Image Playground (diagram generation)
  - Summarization (research papers, logs)
  - Error analysis (plain-language explanations)

- ✅ **Neural Engine** (38 TOPS for AI tasks)
- ✅ **Fast builds** (M4 performance)

---

## Agent Access to System Information

### Can Agents Access System Info?

**Yes!** Cursor remote agents can access system information through:

1. **Standard System Commands:**
   - `uname`, `sysctl`, `lscpu`, `free`, `df`, etc.
   - Available in terminal within Cursor remote session

2. **Environment Variables:**
   - `$HOSTNAME`, `$USER`, `$HOME`
   - Platform-specific variables

3. **Script Execution:**
   - Run collection scripts directly from Cursor
   - Output can be captured and documented

### How to Query System Info from Cursor

**In Cursor Remote Session:**

```bash

# Quick system check

uname -a

# CPU info
# macOS: sysctl machdep.cpu.brand_string
# Linux: lscpu | grep "Model name"

# Memory
# macOS: sysctl hw.memsize
# Linux: free -h

# Disk space

df -h

# Run collection script

python3 scripts/collect_system_info_python.py
```

**Via Background Agent:**

- Agents can execute commands on remote hosts
- Output can be captured and stored
- System info helps with task delegation decisions

---

## Task Delegation Based on System Specs

### High CPU/Memory Tasks → macOS M4 Agent

- Large C++ compilation jobs
- Parallel test execution
- AI-powered documentation generation
- Image generation (Image Playground)

### Linux-Specific Tasks → Ubuntu Agent

- Linux-only builds
- Docker container development
- CI/CD pipeline testing
- Linux compatibility testing

### Network-Intensive Tasks

- Choose agent based on network latency
- Consider bandwidth for large file transfers
- Use agent closest to data source

### Documentation Tasks → macOS M4 Agent

- Leverage Apple Intelligence Writing Tools
- Generate visual diagrams
- Summarize research
- Improve code comments

---

## Environment Variables

### Common Environment Variables

```bash

# Platform Detection

export OS_TYPE="$(uname -s)"
export HOSTNAME="$(hostname)"

# Build Configuration

export CMAKE_BUILD_TYPE="Debug"
export DISTCC_HOSTS="localhost/8 ubuntu-agent.local/8"

# Development Tools

export PYTHONPATH="/path/to/project:$PYTHONPATH"
export RUST_LOG="debug"
```

### Agent-Specific Variables

**Ubuntu Agent:**

```bash
export LINUX_BUILD=true
export DOCKER_AVAILABLE=true
```

**macOS M4 Agent:**

```bash
export MACOS_BUILD=true
export APPLE_INTELLIGENCE_AVAILABLE=true
export NEURAL_ENGINE_AVAILABLE=true
```

---

## Software Versions

### Critical Tool Versions

| Tool | Ubuntu Agent | macOS M4 Agent | Notes |
|------|--------------|----------------|-------|
| **Git** | 2.34+ | 2.42+ | Version control |
| **CMake** | 3.24+ | 3.28+ | Build system |
| **C++ Compiler** | GCC 11+ / Clang 14+ | Clang 15+ | C++20 support |
| **Rust** | 1.75+ | 1.75+ | Backend services |
| **Go** | 1.21+ | 1.21+ | Backend services |
| **Node.js** | 20+ | 20+ | Frontend development |
| **Python** | 3.11+ | 3.12+ | Python bindings |
| **Cursor** | Latest | Latest | IDE agent |

### Dependency Versions

| Dependency | Ubuntu | macOS | Notes |
|------------|--------|-------|-------|
| **TWS API** | Latest | Latest | Interactive Brokers API |
| **Protocol Buffers** | 3.21+ | 3.21+ | Message serialization |
| **Abseil** | Latest | Latest | C++ utilities |
| **NATS** | 2.10+ | 2.10+ | Message queue |

---

## Storage Configuration

### Recommended Directory Structure

**Both Agents:**

```
~/ib_box_spread_full_universal/
├── native/          # C++ core
├── python/          # Python bindings
├── agents/          # Multi-language agents
├── web/             # Frontend
├── build/           # Build artifacts (gitignored)
├── docs/            # Documentation
└── scripts/         # Utility scripts
```

### Disk Space Requirements

**Minimum:**

- Source code: ~500 MB
- Build artifacts: ~2 GB
- Dependencies: ~1 GB
- **Total: ~4 GB**

**Recommended:**

- Source code: ~500 MB
- Build artifacts: ~10 GB (with debug symbols)
- Dependencies: ~2 GB
- Test data: ~5 GB
- **Total: ~20 GB**

---

## Network Configuration

### SSH Access

**Ubuntu Agent:**

```ssh-config
Host cursor-ubuntu
  HostName <ubuntu_ip>
  User <username>
  IdentityFile ~/.ssh/cursor_ubuntu_id_ed25519
  Port 22
```

**macOS M4 Agent:**

```ssh-config
Host cursor-m4-mac
  HostName <mac_ip>
  User <username>
  IdentityFile ~/.ssh/cursor_m4_id_ed25519
  Port 22
```

### Port Forwarding

**Common Ports:**

- `22` - SSH
- `8080` - Backend REST API
- `50051` - Backend gRPC API
- `5173` - Web Frontend
- `4222` - NATS Server

---

## Performance Benchmarks

### Build Performance

*[To be populated with actual benchmarks]*

**Expected Performance:**

| Operation | Ubuntu Agent | macOS M4 Agent | Notes |
|-----------|--------------|----------------|-------|
| **Clean Build** | ~5 min | ~3 min | Full C++ compilation |
| **Incremental Build** | ~30 sec | ~20 sec | Changed files only |
| **Test Execution** | ~2 min | ~1.5 min | All test suites |
| **Documentation Gen** | ~1 min | ~30 sec | With Apple Intelligence |

### Network Performance

*[To be populated]*

- Latency between agents: [TBD] ms
- Bandwidth: [TBD] Mbps
- Git clone time: [TBD] seconds

---

## Maintenance

### Regular Updates

**Weekly:**

- Update system information if hardware changes
- Review software version compatibility
- Update this document with changes

**Monthly:**

- Review performance benchmarks
- Update task delegation strategy
- Optimize based on system specs

### Collection Script Updates

Run collection scripts regularly:

```bash

# Monthly collection

./scripts/collect_system_info_python.py > system_info_$(hostname)_$(date +%Y%m).json
```

---

## Troubleshooting

### Missing System Information

**Problem:** Collection script fails

**Solution:**

1. Check script permissions: `chmod +x scripts/collect_system_info_python.py`
2. Verify Python 3 is installed: `python3 --version`
3. Run manually with verbose output
4. Use bash fallback script

### Inconsistent Information

**Problem:** Different output formats between agents

**Solution:**

- Use Python script (more consistent)
- Validate JSON output with `jq`
- Check for OS-specific differences

### Access Denied

**Problem:** Cannot access system information

**Solution:**

- Verify SSH access works
- Check user permissions
- Review sudo requirements

---

## Next Steps

1. **Run Collection Scripts:**

   ```bash
   # On Ubuntu agent
   ssh cursor-ubuntu "python3 /path/to/collect_system_info_python.py" > system_info_ubuntu.json

   # On macOS M4 agent
   ssh cursor-m4-mac "python3 /path/to/collect_system_info_python.py" > system_info_macos.json
   ```

2. **Populate Documentation:**
   - Update Ubuntu Agent section with collected data
   - Update macOS M4 Agent section with collected data
   - Add performance benchmarks

3. **Verify Agent Access:**
   - Test system info collection from Cursor
   - Document any access issues
   - Update troubleshooting section

---

## References

- [Parallel Cursor Agents Workflow](./PARALLEL_CURSOR_AGENTS_WORKFLOW.md) - Parallel development guide
- [Apple Intelligence Quick Reference](./APPLE_INTELLIGENCE_QUICK_REFERENCE.md) - AI features guide
- [Device Task Delegation](./DEVICE_TASK_DELEGATION.md) - Multi-device workflow
- [Remote Development Workflow](./REMOTE_DEVELOPMENT_WORKFLOW.md) - Remote setup guide

---

**Last Updated:** 2025-01-20
**Maintainer:** Development Team
**Update Frequency:** Monthly or when hardware/software changes
