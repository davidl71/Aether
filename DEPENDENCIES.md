# Platform Dependencies

Complete list of dependencies required to build and run the platform.

## Quick Start

```bash
# Check all dependencies
./scripts/check_dependencies.sh

# Auto-install missing required dependencies (macOS)
./scripts/check_dependencies.sh --install
```

---

## Core Requirements

### System Tools

| Tool | Required | Purpose | Install |
|------|----------|---------|---------|
| **bash** | ✅ Required | Shell scripting | Built-in |
| **git** | ✅ Required | Version control | Built-in / `brew install git` |
| **cmake** | ✅ Required | Build system | `brew install cmake` |
| **ninja** | ✅ Required | Fast build | `brew install ninja` |
| **make** | ✅ Required | Build automation | Built-in (Xcode tools) |
| **lsof** | ✅ Required | Port/process detection | Built-in (macOS) |

### Compilers

| Tool | Required | Purpose | Install |
|------|----------|---------|---------|
| **clang** | ✅ Required | C++ compiler | Built-in (Xcode tools) |
| **g++** | ⚠️ Optional | Alternative C++ compiler | `brew install gcc` |

### Build Optimization (Optional)

| Tool | Required | Purpose | Install |
|------|----------|---------|---------|
| **sccache** | ⚠️ Optional | Compilation caching | `brew install sccache` |
| **ccache** | ⚠️ Optional | Compilation caching | `brew install ccache` |
| **distcc** | ⚠️ Optional | Distributed compilation | `brew install distcc` |

---

## Language Runtimes

### Python (Required)

```bash
# Install Python 3
brew install python3

# Install core packages
pip3 install uvicorn fastapi textual numpy pandas
```

**Required packages**:
- `uvicorn` - ASGI server for services
- `fastapi` - API framework
- `textual` - TUI framework
- `numpy` - Numerical computing
- `pandas` - Data analysis

**Optional**:
- `uv` - Fast Python package manager (`brew install uv`)

### Node.js (Required for Web)

```bash
# Install Node.js and npm
brew install node
```

**Required for**:
- Web frontend (React/Vite)
- Development server

### Rust (Optional)

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Required for**:
- Rust backend agents (optional component)

---

## Ansible (Required for Setup)

Ansible is used for:
1. **Environment setup** - Install system dependencies
2. **Third-party fetch** - Download IB API, protobuf, etc.
3. **Project configuration** - Validate paths and settings

### Installation

```bash
# macOS
brew install ansible

# Or via uv (Python package manager)
uv tool install ansible

# Install required Ansible collections
ansible-galaxy collection install community.general
```

### Locale Fix (IMPORTANT)

Ansible requires UTF-8 locale. Add to `~/.zshrc`:

```bash
export LC_ALL=en_US.UTF-8
export LANG=en_US.UTF-8
```

Then reload: `source ~/.zshrc`

### What Ansible Does

**Playbooks**:
- `playbooks/global.yml` - Install system tools (cmake, ninja, jq, python, shellcheck, etc.)
- `playbooks/project.yml` - Install C++ dependencies (protobuf, boost, abseil, curl)
- `ansible/playbooks/fetch_third_party.yml` - Download third-party libraries

**Roles**:
- `roles/common` - Common system setup
- `roles/editor` - Editor/IDE configuration
- `roles/langs` - Language runtime setup
- `roles/ib_box_spread` - Project-specific dependencies

---

## C++ Dependencies

Managed by Ansible or Homebrew:

| Library | Purpose | Install |
|---------|---------|---------|
| **protobuf** | Serialization | `brew install protobuf` |
| **abseil** | Google utilities | `brew install abseil` |
| **boost** | C++ libraries | `brew install boost` |
| **curl** | HTTP client | `brew install curl` |

**Note**: These are installed automatically via:
```bash
ansible-playbook playbooks/project.yml
```

---

## Third-Party Libraries (Vendored)

Downloaded via `./scripts/fetch_third_party.sh`:

| Library | Purpose | Location |
|---------|---------|----------|
| **TWS API** | Interactive Brokers API | `native/third_party/tws-api/` |
| **Intel Decimal** | Exact decimal math | `native/third_party/IntelRDFPMathLib20U4/` |
| **Protobuf** | (if not via Homebrew) | `native/third_party/protobuf-3.20.3/` |
| **Nautilus Trader** | Trading framework | `native/third_party/nautilus/` |

---

## Optional Tools

### Development Tools

| Tool | Purpose | Install |
|------|---------|---------|
| **jq** | JSON processing | `brew install jq` |
| **clang-tidy** | C++ linter | `brew install llvm` |
| **clang-format** | C++ formatter | `brew install llvm` |
| **shellcheck** | Shell script linter | `brew install shellcheck` |
| **shfmt** | Shell formatter | `brew install shfmt` |
| **markdownlint** | Markdown linter | `npm install -g markdownlint-cli` |

### Services

| Service | Purpose | Install |
|---------|---------|---------|
| **nats-server** | Messaging broker | `brew tap nats-io/nats-tools && brew install nats-server` |
| **docker** | Containerization | `brew install --cask docker` |

---

## Trading Platforms

### Interactive Brokers

**Required for**: Real market data and trading

**Download**: https://www.interactivebrokers.com/en/trading/tws.php

**Options**:
- **TWS (Trader Workstation)** - Full-featured GUI
- **IB Gateway** - Headless API gateway

**Configuration**:
1. Install TWS or IB Gateway
2. Enable API: `Global Configuration → API → Settings`
3. Enable "ActiveX and Socket Clients"
4. Set port: `7497` (paper) or `7496` (live)

**Verification**:
```bash
# Check if installed
ls /Applications | grep -E "Trader Workstation|IB Gateway"
```

---

## Installation Script

The platform provides an automated dependency checker:

```bash
# Check what's missing
./scripts/check_dependencies.sh

# Install missing required dependencies (macOS)
./scripts/check_dependencies.sh --install

# Optional dependencies are not auto-installed
# Install them manually if needed
```

---

## Platform-Specific Notes

### macOS

**Xcode Command Line Tools** (required):
```bash
xcode-select --install
```

**Homebrew** (required for package management):
```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

### Linux (Ubuntu/Debian)

Most dependencies available via `apt`:
```bash
sudo apt update
sudo apt install build-essential cmake ninja-build git python3 python3-pip \
                 libprotobuf-dev protobuf-compiler libboost-all-dev libcurl4-openssl-dev
```

---

## Verification

After installing dependencies, verify:

```bash
# 1. Check all dependencies
./scripts/check_dependencies.sh

# 2. Fetch third-party libraries
./scripts/fetch_third_party.sh

# 3. Build the project
./scripts/build_fast.sh

# 4. Run tests
./scripts/run_tests.sh
```

---

## Troubleshooting

### "ansible-playbook: command not found"

```bash
# Install Ansible
brew install ansible
# or
uv tool install ansible
```

### "Ansible requires the locale encoding to be UTF-8"

```bash
# Add to ~/.zshrc or ~/.bash_profile
export LC_ALL=en_US.UTF-8
export LANG=en_US.UTF-8

# Reload shell
source ~/.zshrc
```

### "cmake: command not found"

```bash
# Install cmake
brew install cmake ninja
```

### "python3: command not found"

```bash
# Install Python
brew install python3
```

### "node: command not found"

```bash
# Install Node.js
brew install node
```

---

## Summary

**Required**:
- bash, git, cmake, ninja, make, lsof (system tools)
- clang (C++ compiler)
- Python 3 + pip (for services and TUI)
- Node.js + npm (for web frontend)
- Ansible (for setup automation)

**Optional but Recommended**:
- sccache/ccache (faster builds)
- jq (config parsing)
- TWS/IB Gateway (for real trading)

**Auto-installed by Ansible/scripts**:
- C++ libraries (protobuf, boost, abseil, curl)
- Third-party vendored code (TWS API, Intel Decimal, Nautilus)

**Quick install on macOS**:
```bash
# 1. Install Homebrew (if not installed)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# 2. Install core dependencies
brew install cmake ninja python3 node ansible jq sccache

# 3. Run project setup
ansible-playbook playbooks/site.yml

# 4. Fetch third-party libraries
./scripts/fetch_third_party.sh

# 5. Build
./scripts/build_fast.sh
```
