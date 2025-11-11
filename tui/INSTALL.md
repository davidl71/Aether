# TUI Installation Guide

Complete installation instructions for the IBKR Box Spread TUI.

## Prerequisites

- Go 1.21 or later
- Terminal with color support (256 colors recommended)
- Unix-like system (macOS, Linux, BSD)

## Installation Methods

### Method 1: From Source (Recommended)

```bash
# Clone repository (if not already done)
git clone <repository-url>
cd ib-box-spread-full-universal/tui

# Build binary
go build -o ib-box-spread-tui ./cmd/tui

# Install to system path
sudo mv ib-box-spread-tui /usr/local/bin/

# Install man page
sudo mkdir -p /usr/local/share/man/man1
sudo cp man/ib-box-spread-tui.1 /usr/local/share/man/man1/
sudo mandb  # Update man database (Linux)
```

### Method 2: Using Make (if Makefile exists)

```bash
cd tui
make install
```

### Method 3: Local Installation (No sudo)

```bash
# Build binary
go build -o ib-box-spread-tui ./cmd/tui

# Install to local bin directory
mkdir -p ~/bin
mv ib-box-spread-tui ~/bin/

# Add to PATH (add to ~/.bashrc or ~/.zshrc)
export PATH="$HOME/bin:$PATH"

# Install man page locally
mkdir -p ~/.local/share/man/man1
cp man/ib-box-spread-tui.1 ~/.local/share/man/man1/
```

## Verification

### Check Installation

```bash
# Verify binary is in PATH
which ib-box-spread-tui

# Check version (if implemented)
ib-box-spread-tui --version

# Test man page
man ib-box-spread-tui
```

### Test Run

```bash
# Run with mock data (safe test)
ib-box-spread-tui --mock

# Should display TUI interface
# Press 'Q' to quit
```

## Post-Installation

### Terminal Configuration

For best experience, ensure your terminal supports:

1. **256 Colors**:
   ```bash
   export TERM=xterm-256color
   ```

2. **UTF-8 Encoding**:
   ```bash
   export LANG=en_US.UTF-8
   ```

3. **Minimum Size**: 80x24 characters (120x40 recommended)

### Environment Variables (Optional)

Set default configuration:

```bash
# Add to ~/.bashrc or ~/.zshrc
export TUI_BACKEND="rest"              # or "mock", "nautilus"
export TUI_API_URL="http://localhost:8080/api/snapshot"
export TUI_INTERVAL=1                   # Polling interval in seconds
```

## Uninstallation

### Remove Binary

```bash
sudo rm /usr/local/bin/ib-box-spread-tui
```

### Remove Man Page

```bash
sudo rm /usr/local/share/man/man1/ib-box-spread-tui.1
sudo mandb  # Update man database (Linux)
```

### Remove Local Installation

```bash
rm ~/bin/ib-box-spread-tui
rm ~/.local/share/man/man1/ib-box-spread-tui.1
```

## Troubleshooting

### Man Page Not Found

**Linux**:
```bash
sudo mandb  # Update man database
```

**macOS**:
```bash
# Man pages should work automatically
# If not, check MANPATH:
echo $MANPATH
```

**Custom Location**:
```bash
# Add to MANPATH
export MANPATH="$HOME/.local/share/man:$MANPATH"
```

### Binary Not Found

**Check PATH**:
```bash
echo $PATH
which ib-box-spread-tui
```

**Add to PATH**:
```bash
# Add to ~/.bashrc or ~/.zshrc
export PATH="/usr/local/bin:$PATH"
```

### Permission Denied

**Fix Permissions**:
```bash
chmod +x ib-box-spread-tui
```

**Or install with sudo**:
```bash
sudo install -m 755 ib-box-spread-tui /usr/local/bin/
```

## Development Installation

For development, you can run directly without installation:

```bash
cd tui
go run ./cmd/tui
```

Or build and run:

```bash
go build -o ib-box-spread-tui ./cmd/tui
./ib-box-spread-tui
```

## See Also

- **README**: `README.md` - User guide
- **Man Page**: `man/ib-box-spread-tui.1` - Unix manual
- **Usage Guide**: `docs/USAGE_GUIDE.md` - Detailed usage
- **Keyboard Shortcuts**: `docs/KEYBOARD_SHORTCUTS.md` - Shortcut reference
