# IB Box Spread Debian Repository

This directory contains a local Debian repository for all IB Box Spread Platform packages.

## Quick Start

### 1. Create the Repository

```bash
./scripts/create_deb_repo.sh
```

This will:
- Build all projects (native C++, Python, Web PWA, Rust backend)
- Create Debian packages (.deb files)
- Generate repository metadata
- Set up the repository structure

### 2. Add Repository to apt

```bash
./scripts/add_deb_repo.sh
```

Or manually:
```bash
echo "deb [trusted=yes] file://$(pwd)/deb-repo stable main" | sudo tee /etc/apt/sources.list.d/ib-box-spread.list
sudo apt-get update
```

### 3. Install Packages

```bash
# Core trading platform
sudo apt-get install ib-box-spread-native
sudo apt-get install synthetic-financing-platform
sudo apt-get install ib-box-spread-web
sudo apt-get install ib-box-spread-backend

# MCP servers
sudo apt-get install project-management-automation-mcp

# Development tools
sudo apt-get install ib-box-spread-build-tools
sudo apt-get install ib-box-spread-automation-tools

# Or install everything
sudo apt-get install ib-box-spread-*
```

## Repository Structure

```
deb-repo/
├── conf/              # Repository configuration
│   ├── distributions  # Distribution definitions
│   └── options        # Repository options
├── dists/             # Distribution metadata
│   └── stable/
│       └── main/
│           └── binary-amd64/
│               ├── Packages
│               └── Packages.gz
├── pool/              # Debian packages (.deb files)
│   ├── ib-box-spread-native_1.3.3_amd64.deb
│   ├── synthetic-financing-platform_1.0.0_all.deb
│   ├── ib-box-spread-web_0.1.0_all.deb
│   └── ib-box-spread-backend_0.1.0_amd64.deb
└── db/                # Repository database (if using reprepro)
```

## Available Packages

### Core Trading Platform

#### ib-box-spread-native
- **Version**: 1.3.3
- **Architecture**: amd64
- **Description**: Native C++ trading engine with CLI and TUI
- **Dependencies**: libc6, libstdc++6, libprotobuf
- **Binaries**: `ib_box_spread`, `ib_box_spread_tui`

#### synthetic-financing-platform
- **Version**: 1.0.0
- **Architecture**: all
- **Description**: Python integration layer with Cython bindings
- **Dependencies**: python3 (>= 3.11), python3-numpy
- **Location**: `/usr/lib/python3/dist-packages/`

#### ib-box-spread-web
- **Version**: 0.1.0
- **Architecture**: all
- **Description**: Progressive Web App dashboard
- **Dependencies**: nginx or apache2
- **Location**: `/usr/share/ib-box-spread-web/`
- **Nginx Config**: `/etc/nginx/sites-available/ib-box-spread-web`

#### ib-box-spread-backend
- **Version**: 0.1.0
- **Architecture**: amd64
- **Description**: Rust backend services
- **Dependencies**: libc6, libssl3
- **Systemd Service**: `ib-box-spread-backend.service`

### MCP Servers

#### project-management-automation-mcp
- **Version**: 0.1.0
- **Architecture**: all
- **Description**: MCP server for project management automation
- **Dependencies**: python3 (>= 3.9), python3-mcp, python3-pydantic
- **Features**: Documentation health checks, Todo2 analysis, duplicate detection, security scanning
- **CLI**: `project-management-automation`

### Development Tools

#### ib-box-spread-build-tools
- **Version**: 1.0.0
- **Architecture**: all
- **Description**: Build automation scripts and CMake presets
- **Dependencies**: build-essential, cmake (>= 3.21), ninja-build, python3
- **Recommends**: cargo, rustc
- **Location**: `/usr/share/ib-box-spread/build-tools/`
- **CLI**: `ib-box-spread-build`
- **Features**: Universal binary builds, distributed compilation, platform-specific configurations

#### ib-box-spread-automation-tools
- **Version**: 1.0.0
- **Architecture**: all
- **Description**: Project management automation tools
- **Dependencies**: python3 (>= 3.9), python3-networkx, python3-requests
- **Recommends**: python3-openai | python3-anthropic
- **Location**: `/usr/share/ib-box-spread/automation/`
- **CLI Tools**:
  - `ib-box-spread-docs-health` - Documentation health analysis
  - `ib-box-spread-todo-align` - Todo2 alignment analysis
- **Features**:
  - Documentation health checks
  - Task alignment analysis
  - Duplicate task detection
  - Dependency security scanning
  - Todo synchronization
  - Automation opportunity discovery
  - PWA review automation

## Rebuilding the Repository

To rebuild all packages and regenerate the repository:

```bash
./scripts/create_deb_repo.sh --clean
```

## Manual Package Installation

If you prefer to install packages manually without adding the repository:

```bash
# Install a specific package
sudo dpkg -i deb-repo/pool/ib-box-spread-native_1.3.3_amd64.deb

# Fix dependencies if needed
sudo apt-get install -f
```

## Troubleshooting

### Missing Dependencies

If package installation fails due to missing dependencies:

```bash
sudo apt-get install -f
```

### Repository Not Found

If apt can't find the repository:

1. Check that the repository was created:
   ```bash
   ls -la deb-repo/pool/
   ```

2. Verify the sources.list entry:
   ```bash
   cat /etc/apt/sources.list.d/ib-box-spread.list
   ```

3. Update apt cache:
   ```bash
   sudo apt-get update
   ```

### Build Failures

If a package fails to build:

1. Check build logs in the respective project directories
2. Ensure all build dependencies are installed:
   ```bash
   sudo apt-get install build-essential cmake ninja-build
   sudo apt-get install python3-dev python3-pip
   sudo apt-get install cargo rustc  # For Rust packages
   ```

## Sharing the Repository

To share this repository with other systems:

1. **Copy the entire `deb-repo/` directory** to the target system
2. **Add the repository** on the target system:
   ```bash
   echo "deb [trusted=yes] file:///path/to/deb-repo stable main" | sudo tee /etc/apt/sources.list.d/ib-box-spread.list
   sudo apt-get update
   ```

3. **Or serve via HTTP**:
   ```bash
   # On source system
   cd deb-repo
   python3 -m http.server 8000

   # On target system
   echo "deb [trusted=yes] http://source-ip:8000 stable main" | sudo tee /etc/apt/sources.list.d/ib-box-spread.list
   sudo apt-get update
   ```

## Repository Maintenance

### Adding New Packages

1. Add package creation function to `scripts/create_deb_repo.sh`
2. Call the function in the `main()` function
3. Rebuild the repository

### Updating Package Versions

1. Update version numbers in:
   - `native/CMakeLists.txt` (for native package)
   - `python/pyproject.toml` (for Python package)
   - `web/package.json` (for web package)
   - `agents/backend/Cargo.toml` (for Rust package)

2. Rebuild the repository:
   ```bash
   ./scripts/create_deb_repo.sh --clean
   ```

### Cleaning Up

To remove all built packages and start fresh:

```bash
rm -rf deb-repo deb-packages
./scripts/create_deb_repo.sh
```
