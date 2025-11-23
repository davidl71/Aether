# Installing the IB Box Spread Debian Repository

This guide explains how to install the IB Box Spread Debian repository on your Ubuntu system (25.04+).

## Quick Install

```bash
# From the project root
sudo ./scripts/install_deb_repo.sh
```

This script will:
1. Detect your Ubuntu version
2. Import the GPG signing key
3. Create the `.sources` file with `Signed-By` field (required for Ubuntu 25.04+)
4. Update apt cache
5. Verify the repository is working

## Manual Installation

If you prefer to install manually:

### 1. Import GPG Key

```bash
# Download the public key
curl -fsSL file:///path/to/deb-repo/public.key | sudo gpg --dearmor -o /usr/share/keyrings/ib-box-spread-archive-keyring.gpg

# Or if using HTTP repository
curl -fsSL https://your-server/deb-repo/public.key | sudo gpg --dearmor -o /usr/share/keyrings/ib-box-spread-archive-keyring.gpg
```

### 2. Create .sources File

For Ubuntu 25.04+, create `/etc/apt/sources.list.d/ib-box-spread.sources`:

```bash
sudo tee /etc/apt/sources.list.d/ib-box-spread.sources > /dev/null <<EOF
Types: deb
URIs: file:///path/to/deb-repo
Suites: stable
Components: main
Signed-By: /usr/share/keyrings/ib-box-spread-archive-keyring.gpg
EOF
```

For HTTP repository:
```bash
sudo tee /etc/apt/sources.list.d/ib-box-spread.sources > /dev/null <<EOF
Types: deb
URIs: https://your-server/deb-repo
Suites: stable
Components: main
Signed-By: /usr/share/keyrings/ib-box-spread-archive-keyring.gpg
EOF
```

### 3. Update Apt Cache

```bash
sudo apt-get update
```

### 4. Install Packages

```bash
# Install all packages
sudo apt-get install ib-box-spread-*

# Or install individually
sudo apt-get install ib-box-spread-native
sudo apt-get install synthetic-financing-platform
sudo apt-get install ib-box-spread-web
sudo apt-get install ib-box-spread-backend
sudo apt-get install project-management-automation-mcp
sudo apt-get install trading-mcp-server
sudo apt-get install ib-box-spread-build-tools
sudo apt-get install ib-box-spread-automation-tools
```

## Testing the Installation

Before installing, you can test the setup:

```bash
./scripts/test_repo_install.sh
```

This will show:
- Repository status
- GPG key status
- Available packages
- System compatibility

## Troubleshooting

### GPG Key Issues

If you get GPG errors:

```bash
# Re-import the key
sudo rm /usr/share/keyrings/ib-box-spread-archive-keyring.gpg
sudo ./scripts/install_deb_repo.sh
```

### Repository Not Found

If apt can't find the repository:

```bash
# Check the .sources file
cat /etc/apt/sources.list.d/ib-box-spread.sources

# Verify the repository path is correct
ls -la /path/to/deb-repo/pool/

# Update apt cache
sudo apt-get update
```

### Permission Errors

Make sure you're running with sudo:

```bash
sudo ./scripts/install_deb_repo.sh
```

## Remote Repository Setup

To serve the repository over HTTP:

```bash
# On the server
cd /path/to/deb-repo
python3 -m http.server 8000

# On client machines
sudo ./scripts/install_deb_repo.sh --repo-url http://server-ip:8000
```

## Verification

After installation, verify packages are available:

```bash
apt-cache search ib-box-spread
apt-cache policy ib-box-spread-native
```
