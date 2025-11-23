#!/usr/bin/env bash
# Create a Debian repository for all projects in the workspace
# Usage: ./scripts/create_deb_repo.sh [--repo-dir REPO_DIR] [--clean]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_DIR="${REPO_DIR:-${PROJECT_ROOT}/deb-repo}"
CLEAN_BUILD="${CLEAN_BUILD:-false}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
  echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
  echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
  echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
  echo -e "${RED}[ERROR]${NC} $1" >&2
}

# Parse arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --repo-dir)
      REPO_DIR="$2"
      shift 2
      ;;
    --clean)
      CLEAN_BUILD=true
      shift
      ;;
    *)
      log_error "Unknown option: $1"
      exit 1
      ;;
  esac
done

# Check dependencies
check_dependencies() {
  local missing=()

  for cmd in dpkg-deb dpkg-buildpackage reprepro dpkg-scanpackages apt-ftparchive; do
    if ! command -v "$cmd" >/dev/null 2>&1; then
      missing+=("$cmd")
    fi
  done

  if [ ${#missing[@]} -gt 0 ]; then
    log_error "Missing required tools: ${missing[*]}"
    log_info "Install with: sudo apt-get install dpkg-dev reprepro apt-utils"
    exit 1
  fi

  log_success "All required tools are available"
}

# Setup GPG key for signing
setup_gpg_key() {
  log_info "Setting up GPG key for repository signing..."

  local gpg_key_dir="$PROJECT_ROOT/.gnupg-repo"
  local gpg_key_id=""

  mkdir -p "$gpg_key_dir"
  chmod 700 "$gpg_key_dir"

  # Check if key already exists
  if [ -f "$gpg_key_dir/pubring.gpg" ] || [ -f "$gpg_key_dir/public.key" ]; then
    log_info "GPG key already exists, using existing key"
    if [ -f "$gpg_key_dir/keyid.txt" ]; then
      gpg_key_id=$(cat "$gpg_key_dir/keyid.txt")
    fi
  else
    log_info "Generating new GPG key for repository signing..."

    # Generate GPG key non-interactively
    cat > "$gpg_key_dir/keygen.conf" <<EOF
%no-protection
Key-Type: RSA
Key-Length: 4096
Subkey-Type: RSA
Subkey-Length: 4096
Name-Real: IB Box Spread Platform
Name-Email: platform@ib-box-spread.local
Expire-Date: 0
%commit
EOF

    # Generate key (with entropy source)
    if [ -c /dev/urandom ]; then
      export GNUPGHOME="$gpg_key_dir"
      gpg --batch --gen-key "$gpg_key_dir/keygen.conf" >/dev/null 2>&1 || {
        # Fallback: try with rng-tools if available
        if command -v rngd >/dev/null 2>&1; then
          sudo rngd -r /dev/urandom
        fi
        gpg --batch --gen-key "$gpg_key_dir/keygen.conf" >/dev/null 2>&1 || {
          log_warn "GPG key generation may need more entropy. Continuing without signing..."
          return 0
        }
      }
    else
      log_warn "Cannot generate GPG key: /dev/urandom not available"
      return 0
    fi

    # Get key ID
    gpg_key_id=$(gpg --homedir "$gpg_key_dir" --list-secret-keys --keyid-format LONG 2>/dev/null | grep -E "^sec" | head -1 | awk '{print $2}' | cut -d'/' -f2)

    if [ -z "$gpg_key_id" ]; then
      log_error "Failed to generate GPG key"
      return 1
    fi

    # Save key ID
    echo "$gpg_key_id" > "$gpg_key_dir/keyid.txt"

    # Export public key
    gpg --homedir "$gpg_key_dir" --armor --export "$gpg_key_id" > "$gpg_key_dir/public.key"

    # Copy public key to repo
    cp "$gpg_key_dir/public.key" "$REPO_DIR/public.key"

    log_success "GPG key generated: $gpg_key_id"
  fi

  # Export GPG key ID for use in other functions
  export GPG_KEY_ID="$gpg_key_id"
  export GPG_HOMEDIR="$gpg_key_dir"

  log_success "GPG key setup complete"
}

# Create repository structure
setup_repo_structure() {
  log_info "Setting up repository structure at $REPO_DIR"

  mkdir -p "$REPO_DIR"/{conf,dists,pool,db}

  # Setup GPG key first
  setup_gpg_key

  # Create reprepro configuration
  cat > "$REPO_DIR/conf/distributions" <<EOF
Origin: IB Box Spread Platform
Label: IB Box Spread Debian Repository
Codename: stable
Architectures: amd64 arm64
Components: main
Description: Debian repository for IB Box Spread Platform packages
SignWith: ${GPG_KEY_ID:-}
EOF

  cat > "$REPO_DIR/conf/options" <<EOF
verbose
basedir $REPO_DIR
gpg-home $GPG_HOMEDIR
EOF

  log_success "Repository structure created"
}

# Create Debian package for native C++ project
create_native_deb() {
  log_info "Creating Debian package for native C++ project"

  local pkg_name="ib-box-spread-native"
  local pkg_version="1.3.3"
  local pkg_dir="$PROJECT_ROOT/deb-packages/$pkg_name"
  local build_dir="$pkg_dir/build"

  mkdir -p "$pkg_dir"/{DEBIAN,usr/{bin,lib,share/doc/$pkg_name,share/man/man1}}

  # Build the native project
  log_info "Building native C++ project..."
  cd "$PROJECT_ROOT"

  if [ "$CLEAN_BUILD" = "true" ]; then
    rm -rf build
  fi

  # Configure and build
  cmake --preset ubuntu-x64-release || cmake --preset linux-x64-release || {
    log_warn "CMake preset not found, using manual configuration"
    mkdir -p build
    cd build
    cmake -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=/usr ..
    cmake --build . --config Release
    cmake --install . --prefix "$pkg_dir/usr"
    cd ..
  } || {
    log_warn "Using manual build process"
    mkdir -p build
    cd build
    cmake -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=/usr "$PROJECT_ROOT/native"
    cmake --build . -j$(nproc)
    DESTDIR="$pkg_dir" cmake --install .
    cd ..
  }

  # Copy binaries if they exist
  if [ -f "$PROJECT_ROOT/build/bin/ib_box_spread" ]; then
    cp "$PROJECT_ROOT/build/bin/ib_box_spread" "$pkg_dir/usr/bin/"
    chmod +x "$pkg_dir/usr/bin/ib_box_spread"
  fi

  if [ -f "$PROJECT_ROOT/build/bin/ib_box_spread_tui" ]; then
    cp "$PROJECT_ROOT/build/bin/ib_box_spread_tui" "$pkg_dir/usr/bin/"
    chmod +x "$pkg_dir/usr/bin/ib_box_spread_tui"
  fi

  # Create control file
  cat > "$pkg_dir/DEBIAN/control" <<EOF
Package: $pkg_name
Version: $pkg_version
Section: finance
Priority: optional
Architecture: amd64
Depends: libc6 (>= 2.34), libstdc++6 (>= 12), libprotobuf32 | libprotobuf35 | libprotobuf-lite32 | libprotobuf-lite35
Maintainer: IB Box Spread Platform Team <platform@example.com>
Description: IB Box Spread Native Trading Engine
 Comprehensive multi-asset financing optimization system.
 Box spreads are one strategy component of this platform.
EOF

  # Create postinst script
  cat > "$pkg_dir/DEBIAN/postinst" <<'EOF'
#!/bin/bash
set -e
# Update system library cache
ldconfig
EOF
  chmod +x "$pkg_dir/DEBIAN/postinst"

  # Build .deb package
  log_info "Building .deb package..."
  dpkg-deb --build "$pkg_dir" "$REPO_DIR/pool/${pkg_name}_${pkg_version}_amd64.deb"

  log_success "Created $pkg_name package"
}

# Create Debian package for Python package
create_python_deb() {
  log_info "Creating Debian package for Python package"

  local pkg_name="synthetic-financing-platform"
  local pkg_version="1.0.0"
  local pkg_dir="$PROJECT_ROOT/deb-packages/$pkg_name"

  mkdir -p "$pkg_dir"/{DEBIAN,usr/{lib/python3/dist-packages,share/doc/$pkg_name}}

  # Build Python package
  log_info "Building Python package..."
  cd "$PROJECT_ROOT/python"

  # Install to staging directory
  python3 -m pip install . --target "$pkg_dir/usr/lib/python3/dist-packages" --no-deps || {
    log_warn "pip install failed, using setup.py"
    python3 setup.py install --prefix="$pkg_dir/usr" --single-version-externally-managed --record="$pkg_dir/install.log"
  }

  # Create control file
  cat > "$pkg_dir/DEBIAN/control" <<EOF
Package: $pkg_name
Version: $pkg_version
Section: python
Priority: optional
Architecture: all
Depends: python3 (>= 3.11), python3-numpy (>= 1.24.0)
Maintainer: IB Box Spread Platform Team <platform@example.com>
Description: Synthetic Financing Platform - Python Integration
 Python bindings and integration layer for the IB Box Spread Platform.
 Provides Cython bindings, Lean integration, and trading utilities.
EOF

  # Build .deb package
  log_info "Building .deb package..."
  dpkg-deb --build "$pkg_dir" "$REPO_DIR/pool/${pkg_name}_${pkg_version}_all.deb"

  log_success "Created $pkg_name package"
}

# Create Debian package for Web PWA
create_web_deb() {
  log_info "Creating Debian package for Web PWA"

  local pkg_name="ib-box-spread-web"
  local pkg_version="0.1.0"
  local pkg_dir="$PROJECT_ROOT/deb-packages/$pkg_name"

  mkdir -p "$pkg_dir"/{DEBIAN,usr/share/ib-box-spread-web,etc/nginx/sites-available}

  # Build web PWA if not already built
  if [ ! -d "$PROJECT_ROOT/web/dist" ]; then
    log_info "Building web PWA..."
    cd "$PROJECT_ROOT/web"
    npm install
    npm run build
  fi

  # Copy built files
  cp -r "$PROJECT_ROOT/web/dist"/* "$pkg_dir/usr/share/ib-box-spread-web/"

  # Create nginx configuration
  cat > "$pkg_dir/etc/nginx/sites-available/ib-box-spread-web" <<'EOF'
server {
    listen 80;
    server_name _;
    root /usr/share/ib-box-spread-web;
    index index.html;

    location / {
        try_files $uri $uri/ /index.html;
    }

    location /api {
        proxy_pass http://127.0.0.1:8000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
EOF

  # Create control file
  cat > "$pkg_dir/DEBIAN/control" <<EOF
Package: $pkg_name
Version: $pkg_version
Section: web
Priority: optional
Architecture: all
Depends: nginx (>= 1.18) | apache2 (>= 2.4)
Maintainer: IB Box Spread Platform Team <platform@example.com>
Description: IB Box Spread Web Dashboard (PWA)
 Progressive Web App dashboard for IB Box Spread trading platform.
 Provides real-time monitoring, box spread explorer, and trading controls.
EOF

  # Create postinst script
  cat > "$pkg_dir/DEBIAN/postinst" <<'EOF'
#!/bin/bash
set -e
if [ -d /etc/nginx/sites-available ] && [ -f /etc/nginx/sites-available/ib-box-spread-web ]; then
    if [ ! -L /etc/nginx/sites-enabled/ib-box-spread-web ]; then
        ln -s /etc/nginx/sites-available/ib-box-spread-web /etc/nginx/sites-enabled/
    fi
    systemctl reload nginx 2>/dev/null || true
fi
EOF
  chmod +x "$pkg_dir/DEBIAN/postinst"

  # Build .deb package
  log_info "Building .deb package..."
  dpkg-deb --build "$pkg_dir" "$REPO_DIR/pool/${pkg_name}_${pkg_version}_all.deb"

  log_success "Created $pkg_name package"
}

# Create Debian package for Rust backend
create_rust_deb() {
  log_info "Creating Debian package for Rust backend services"

  local pkg_name="ib-box-spread-backend"
  local pkg_version="0.1.0"
  local pkg_dir="$PROJECT_ROOT/deb-packages/$pkg_name"

  mkdir -p "$pkg_dir"/{DEBIAN,usr/{bin,lib/systemd/system,share/doc/$pkg_name}}

  # Build Rust services
  log_info "Building Rust backend services..."
  cd "$PROJECT_ROOT/agents/backend"

  if command -v cargo >/dev/null 2>&1; then
    cargo build --release

    # Copy binaries
    find target/release -maxdepth 1 -type f -executable -not -name "*.so" -not -name "*.dylib" | while read -r bin; do
      cp "$bin" "$pkg_dir/usr/bin/$(basename "$bin")"
      chmod +x "$pkg_dir/usr/bin/$(basename "$bin")"
    done
  else
    log_warn "Cargo not found, skipping Rust backend build"
    return 0
  fi

  # Create systemd service file
  cat > "$pkg_dir/usr/lib/systemd/system/ib-box-spread-backend.service" <<'EOF'
[Unit]
Description=IB Box Spread Backend Service
After=network.target

[Service]
Type=simple
ExecStart=/usr/bin/backend_service
Restart=always
User=ib-box-spread

[Install]
WantedBy=multi-user.target
EOF

  # Create control file
  cat > "$pkg_dir/DEBIAN/control" <<EOF
Package: $pkg_name
Version: $pkg_version
Section: finance
Priority: optional
Architecture: amd64
Depends: libc6 (>= 2.34), libssl3 (>= 3.0.0)
Maintainer: IB Box Spread Platform Team <platform@example.com>
Description: IB Box Spread Backend Services
 Rust-based backend services for the IB Box Spread Platform.
 Provides market data, order management, and risk calculation services.
EOF

  # Create postinst script
  cat > "$pkg_dir/DEBIAN/postinst" <<'EOF'
#!/bin/bash
set -e
systemctl daemon-reload
systemctl enable ib-box-spread-backend.service 2>/dev/null || true
EOF
  chmod +x "$pkg_dir/DEBIAN/postinst"

  # Build .deb package
  log_info "Building .deb package..."
  dpkg-deb --build "$pkg_dir" "$REPO_DIR/pool/${pkg_name}_${pkg_version}_amd64.deb"

  log_success "Created $pkg_name package"
}

# Generate repository metadata
generate_repo_metadata() {
  log_info "Generating repository metadata..."

  cd "$REPO_DIR"

  # Use reprepro if available
  if command -v reprepro >/dev/null 2>&1; then
    log_info "Using reprepro to generate repository..."
    reprepro includedeb stable pool/*.deb || {
      log_warn "reprepro failed, using apt-ftparchive"
      generate_repo_metadata_apt
    }
  else
    generate_repo_metadata_apt
  fi

  log_success "Repository metadata generated"
}

# Generate repository metadata using apt-ftparchive
generate_repo_metadata_apt() {
  log_info "Using apt-ftparchive to generate repository..."

  cd "$REPO_DIR"

  # Create Packages file
  mkdir -p "dists/stable/main/binary-amd64"
  mkdir -p "dists/stable/main/binary-all"

  dpkg-scanpackages pool/ > dists/stable/main/binary-amd64/Packages 2>/dev/null || \
    dpkg-scanpackages -m pool/ > dists/stable/main/binary-amd64/Packages

  # Create Packages.gz
  gzip -c dists/stable/main/binary-amd64/Packages > dists/stable/main/binary-amd64/Packages.gz

  # Create Release file
  cat > dists/stable/Release <<EOF
Origin: IB Box Spread Platform
Label: IB Box Spread Debian Repository
Codename: stable
Architectures: amd64 all
Components: main
Description: Debian repository for IB Box Spread Platform packages
Date: $(date -Ru)
EOF

  # Generate checksums
  cd dists/stable
  md5sum main/binary-amd64/Packages* > MD5Sum 2>/dev/null || true
  sha256sum main/binary-amd64/Packages* > SHA256 2>/dev/null || true

  # Sign Release file with GPG
  if [ -n "${GPG_KEY_ID:-}" ] && [ -n "${GPG_HOMEDIR:-}" ]; then
    log_info "Signing Release file with GPG..."
    gpg --homedir "$GPG_HOMEDIR" --armor --detach-sign --output Release.gpg Release 2>/dev/null || log_warn "Failed to sign Release file"
    gpg --homedir "$GPG_HOMEDIR" --clear-sign --output InRelease Release 2>/dev/null || log_warn "Failed to create InRelease"
  fi

  log_success "Repository metadata generated with apt-ftparchive"
}

# Create Debian package for Project Management Automation MCP Server
create_project_management_mcp_deb() {
  log_info "Creating Debian package for Project Management Automation MCP Server"

  local pkg_name="project-management-automation-mcp"
  local pkg_version="0.1.0"
  local pkg_dir="$PROJECT_ROOT/deb-packages/$pkg_name"

  mkdir -p "$pkg_dir"/{DEBIAN,usr/{lib/python3/dist-packages,share/doc/$pkg_name,bin}}

  # Build Python package
  log_info "Building Project Management Automation MCP Server..."
  cd "$PROJECT_ROOT/mcp-servers/project-management-automation"

  # Install to staging directory
  python3 -m pip install . --target "$pkg_dir/usr/lib/python3/dist-packages" --no-deps || {
    log_warn "pip install failed, using setup.py"
    python3 setup.py install --prefix="$pkg_dir/usr" --single-version-externally-managed --record="$pkg_dir/install.log"
  }

  # Create symlink for CLI
  if [ -f "$pkg_dir/usr/lib/python3/dist-packages/bin/project-management-automation" ]; then
    ln -s ../lib/python3/dist-packages/bin/project-management-automation "$pkg_dir/usr/bin/project-management-automation"
  fi

  # Create control file
  cat > "$pkg_dir/DEBIAN/control" <<EOF
Package: $pkg_name
Version: $pkg_version
Section: python
Priority: optional
Architecture: all
Depends: python3 (>= 3.9), python3-mcp (>= 0.1.0), python3-pydantic (>= 2.0.0)
Maintainer: IB Box Spread Platform Team <platform@example.com>
Description: Project Management Automation MCP Server
 MCP server exposing project management automation tools.
 Provides documentation health checks, Todo2 analysis, duplicate detection,
 security scanning, and automation opportunity discovery.
EOF

  # Build .deb package
  log_info "Building .deb package..."
  dpkg-deb --build "$pkg_dir" "$REPO_DIR/pool/${pkg_name}_${pkg_version}_all.deb"

  log_success "Created $pkg_name package"
}

# Create Debian package for Trading MCP Server
create_trading_mcp_deb() {
  log_info "Creating Debian package for Trading MCP Server"

  local pkg_name="trading-mcp-server"
  local pkg_version="0.1.0"
  local pkg_dir="$PROJECT_ROOT/deb-packages/$pkg_name"

  mkdir -p "$pkg_dir"/{DEBIAN,usr/{lib/python3/dist-packages,share/doc/$pkg_name}}

  # Build Python package
  log_info "Building Trading MCP Server..."
  cd "$PROJECT_ROOT/mcp/trading_server"

  # Install to staging directory
  python3 -m pip install . --target "$pkg_dir/usr/lib/python3/dist-packages" --no-deps || {
    log_warn "pip install failed, using setup.py"
    python3 setup.py install --prefix="$pkg_dir/usr" --single-version-externally-managed --record="$pkg_dir/install.log"
  }

  # Create control file
  cat > "$pkg_dir/DEBIAN/control" <<EOF
Package: $pkg_name
Version: $pkg_version
Section: python
Priority: optional
Architecture: all
Depends: python3 (>= 3.9), python3-mcp (>= 0.1.0), python3-requests (>= 2.31.0)
Maintainer: IB Box Spread Platform Team <platform@example.com>
Description: Trading MCP Server
 MCP server for trading operations and broker integration.
 Provides broker-agnostic REST API bridge for trading operations.
EOF

  # Build .deb package
  log_info "Building .deb package..."
  dpkg-deb --build "$pkg_dir" "$REPO_DIR/pool/${pkg_name}_${pkg_version}_all.deb"

  log_success "Created $pkg_name package"
}

# Create Debian package for build tools and scripts
create_build_tools_deb() {
  log_info "Creating Debian package for build tools and scripts"

  local pkg_name="ib-box-spread-build-tools"
  local pkg_version="1.0.0"
  local pkg_dir="$PROJECT_ROOT/deb-packages/$pkg_name"

  mkdir -p "$pkg_dir"/{DEBIAN,usr/{share/ib-box-spread/build-tools,bin,share/doc/$pkg_name}}

  # Copy build scripts
  log_info "Copying build tools..."
  cp -r "$PROJECT_ROOT/scripts"/* "$pkg_dir/usr/share/ib-box-spread/build-tools/" 2>/dev/null || true

  # Copy CMake presets
  if [ -f "$PROJECT_ROOT/CMakePresets.json" ]; then
    mkdir -p "$pkg_dir/usr/share/ib-box-spread/cmake"
    cp "$PROJECT_ROOT/CMakePresets.json" "$pkg_dir/usr/share/ib-box-spread/cmake/"
  fi

  # Create wrapper scripts in /usr/bin
  cat > "$pkg_dir/usr/bin/ib-box-spread-build" <<'EOF'
#!/bin/bash
# Wrapper script for IB Box Spread build tools
exec /usr/share/ib-box-spread/build-tools/build_universal.sh "$@"
EOF
  chmod +x "$pkg_dir/usr/bin/ib-box-spread-build"

  # Create control file
  cat > "$pkg_dir/DEBIAN/control" <<EOF
Package: $pkg_name
Version: $pkg_version
Section: devel
Priority: optional
Architecture: all
Depends: build-essential, cmake (>= 3.21), ninja-build, python3
Recommends: cargo, rustc
Maintainer: IB Box Spread Platform Team <platform@example.com>
Description: IB Box Spread Build Tools and Scripts
 Build automation scripts, CMake presets, and development utilities
 for the IB Box Spread Platform.
 Includes universal binary builds, distributed compilation, and
 platform-specific build configurations.
EOF

  # Build .deb package
  log_info "Building .deb package..."
  dpkg-deb --build "$pkg_dir" "$REPO_DIR/pool/${pkg_name}_${pkg_version}_all.deb"

  log_success "Created $pkg_name package"
}

# Create Debian package for automation scripts
create_automation_tools_deb() {
  log_info "Creating Debian package for automation tools"

  local pkg_name="ib-box-spread-automation-tools"
  local pkg_version="1.0.0"
  local pkg_dir="$PROJECT_ROOT/deb-packages/$pkg_name"

  mkdir -p "$pkg_dir"/{DEBIAN,usr/{share/ib-box-spread/automation,bin,share/doc/$pkg_name}}

  # Copy automation scripts
  log_info "Copying automation tools..."
  local automation_scripts=(
    "automate_docs_health_v2.py"
    "automate_todo2_alignment_v2.py"
    "automate_todo2_duplicate_detection.py"
    "automate_dependency_security.py"
    "automate_todo_sync.py"
    "automate_automation_opportunities.py"
    "automate_pwa_review.py"
  )

  for script in "${automation_scripts[@]}"; do
    if [ -f "$PROJECT_ROOT/scripts/$script" ]; then
      cp "$PROJECT_ROOT/scripts/$script" "$pkg_dir/usr/share/ib-box-spread/automation/"
    fi
  done

  # Copy base automation framework
  if [ -d "$PROJECT_ROOT/scripts/base" ]; then
    cp -r "$PROJECT_ROOT/scripts/base" "$pkg_dir/usr/share/ib-box-spread/automation/"
  fi

  # Copy config files
  for config in "docs_health_config.json" "todo2_alignment_config.json" "todo2_duplicate_config.json" "dependency_security_config.json" "pwa_review_config.json" "todo_sync_config.json"; do
    if [ -f "$PROJECT_ROOT/scripts/$config" ]; then
      cp "$PROJECT_ROOT/scripts/$config" "$pkg_dir/usr/share/ib-box-spread/automation/"
    fi
  done

  # Create wrapper scripts
  cat > "$pkg_dir/usr/bin/ib-box-spread-docs-health" <<'EOF'
#!/bin/bash
exec python3 /usr/share/ib-box-spread/automation/automate_docs_health_v2.py "$@"
EOF
  chmod +x "$pkg_dir/usr/bin/ib-box-spread-docs-health"

  cat > "$pkg_dir/usr/bin/ib-box-spread-todo-align" <<'EOF'
#!/bin/bash
exec python3 /usr/share/ib-box-spread/automation/automate_todo2_alignment_v2.py "$@"
EOF
  chmod +x "$pkg_dir/usr/bin/ib-box-spread-todo-align"

  # Create control file
  cat > "$pkg_dir/DEBIAN/control" <<EOF
Package: $pkg_name
Version: $pkg_version
Section: devel
Priority: optional
Architecture: all
Depends: python3 (>= 3.9), python3-networkx, python3-requests
Recommends: python3-openai | python3-anthropic
Maintainer: IB Box Spread Platform Team <platform@example.com>
Description: IB Box Spread Automation Tools
 Project management automation tools for documentation health,
 task alignment, duplicate detection, security scanning, and more.
 Built on IntelligentAutomationBase framework with Todo2 integration.
EOF

  # Build .deb package
  log_info "Building .deb package..."
  dpkg-deb --build "$pkg_dir" "$REPO_DIR/pool/${pkg_name}_${pkg_version}_all.deb"

  log_success "Created $pkg_name package"
}

# Main execution
main() {
  log_info "Starting Debian repository creation..."
  log_info "Repository directory: $REPO_DIR"

  check_dependencies
  setup_repo_structure

  # Create packages
  create_native_deb || log_warn "Failed to create native package"
  create_python_deb || log_warn "Failed to create Python package"
  create_web_deb || log_warn "Failed to create web package"
  create_rust_deb || log_warn "Failed to create Rust backend package"
  create_project_management_mcp_deb || log_warn "Failed to create Project Management MCP package"
  create_trading_mcp_deb || log_warn "Failed to create Trading MCP package"
  create_build_tools_deb || log_warn "Failed to create build tools package"
  create_automation_tools_deb || log_warn "Failed to create automation tools package"

  # Generate repository metadata
  generate_repo_metadata

  log_success "Debian repository created at $REPO_DIR"
  log_info "To use this repository, add to /etc/apt/sources.list:"
  log_info "  deb file://$REPO_DIR stable main"
  log_info "Then run: sudo apt-get update"
  log_info ""
  log_info "Available packages:"
  log_info "  - ib-box-spread-native (C++ trading engine)"
  log_info "  - synthetic-financing-platform (Python integration)"
  log_info "  - ib-box-spread-web (PWA dashboard)"
  log_info "  - ib-box-spread-backend (Rust services)"
  log_info "  - project-management-automation-mcp (MCP server)"
  log_info "  - trading-mcp-server (Trading MCP server)"
  log_info "  - ib-box-spread-build-tools (Build scripts)"
  log_info "  - ib-box-spread-automation-tools (Automation scripts)"
}

main "$@"
