# Workspace RAM Disk Auto-Startup & Save Setup

This guide explains how to configure your workspace to automatically start and pre-warm RAM disks when opening Cursor/VS Code, and save them before closing.

---

## Overview

The workspace includes:

- ✅ **Auto-startup script**: Creates and pre-warms RAM disks on workspace open
- ✅ **Auto-save script**: Saves important build artifacts before closing
- ✅ **Tasks**: Available in Command Palette for manual use
- ✅ **Keyboard shortcuts**: Quick access to save/status commands

---

## Quick Setup

### Step 1: Enable Auto-Startup (Recommended)

Add to your shell profile (`.zshrc` or `.bashrc`):

```bash

# Auto-start RAM disk when opening workspace in Cursor/VS Code

if [[ -n "$VSCODE_INJECTION" ]] || [[ -n "$CURSOR_INJECTION" ]]; then
  # Only run if we're in the project directory
  if [[ "$PWD" == *"ib_box_spread_full_universal"* ]]; then
    "${PWD}/scripts/workspace_ram_disk_manager.sh" startup >/dev/null 2>&1 &
  fi
fi
```

Or create a simple startup script:

```bash

# ~/bin/cursor-workspace-startup.sh
#!/bin/bash

cd /path/to/ib_box_spread_full_universal
./scripts/workspace_ram_disk_manager.sh startup
```

### Step 2: Enable Auto-Save on Close

**Option A: Manual (Before Closing)**

- Press `Cmd+Shift+R` to run "RAM Disk: Save & Shutdown"
- Or run: `./scripts/workspace_ram_disk_manager.sh shutdown`

**Option B: Shell Hook (Automatic)**

Add to your shell profile:

```bash

# Save RAM disk on shell exit (if in workspace)

cursor_save_ramdisk() {
  if [[ "$PWD" == *"ib_box_spread_full_universal"* ]]; then
    "${PWD}/scripts/workspace_ram_disk_manager.sh" shutdown >/dev/null 2>&1
  fi
}

# Hook into shell exit (zsh)

if [[ "$SHELL" == *"zsh"* ]]; then
  autoload -Uz add-zsh-hook
  add-zsh-hook zshexit cursor_save_ramdisk
fi

# Hook into shell exit (bash)

if [[ "$SHELL" == *"bash"* ]]; then
  trap cursor_save_ramdisk EXIT
fi
```

---

## Tasks Available

### RAM Disk: Startup (Auto)

- **Purpose**: Creates and pre-warms RAM disks
- **Runs**: Automatically on workspace open (if configured)
- **Manual**: `Cmd+Shift+P` → "Tasks: Run Task" → "RAM Disk: Startup (Auto)"

### RAM Disk: Save & Shutdown

- **Purpose**: Saves build artifacts and cache stats before closing
- **Usage**: Run before closing workspace
- **Shortcut**: `Cmd+Shift+R` (after setup)

### RAM Disk: Save Now

- **Purpose**: Save current build artifacts without shutdown
- **Usage**: Periodic saves during development
- **Shortcut**: `Cmd+Shift+S` (after setup)

### RAM Disk: Status

- **Purpose**: Show RAM disk status and usage
- **Usage**: Check current state
- **Shortcut**: `Cmd+Shift+T` (after setup)

---

## Keyboard Shortcuts Setup

### Step 1: Copy Example Keybindings

```bash

# Copy example to your keybindings

cp .vscode/keybindings.json.example ~/Library/Application\ Support/Cursor/User/keybindings.json

# Or merge manually
```

### Step 2: Customize (Optional)

Edit `keybindings.json` to change shortcuts:

```json
[
  {
    "key": "cmd+shift+r",
    "command": "workbench.action.tasks.runTask",
    "args": "RAM Disk: Save & Shutdown"
  },
  {
    "key": "cmd+shift+s",
    "command": "workbench.action.tasks.runTask",
    "args": "RAM Disk: Save Now"
  },
  {
    "key": "cmd+shift+t",
    "command": "workbench.action.tasks.runTask",
    "args": "RAM Disk: Status"
  }
]
```

---

## How It Works

### On Startup

1. **Check for existing RAM disk**
   - If found, use it (from previous session)
   - If not, create new RAM disk

2. **Pre-warm build RAM disk**
   - Restore saved build artifacts (if available)
   - Create directory structure
   - Ready for immediate builds

3. **Pre-warm cache RAM disk**
   - Ensure cache directories exist
   - Load cache optimization environment
   - Ready for immediate cache operations

### On Shutdown

1. **Save build artifacts**
   - `compile_commands.json` (for IDE)
   - `CMakeCache.txt` (for configuration)
   - Binaries and libraries
   - Keeps last 3 saves (auto-cleanup)

2. **Save cache statistics**
   - ccache stats
   - sccache stats
   - For analysis and debugging

3. **Keep RAM disk mounted** (by default)
   - Preserves cache data
   - Faster next startup
   - Can unmount manually if needed

---

## Saved Builds

Build artifacts are saved to `.saved-builds/` directory:

```
.saved-builds/
├── 20250127-143022/  # Latest save
│   ├── compile_commands.json
│   ├── CMakeCache.txt
│   ├── bin/
│   └── lib/
├── 20250127-120000/  # Previous save
└── 20250127-090000/  # Oldest save (kept)
```

**Auto-cleanup**: Only last 3 saves are kept to save disk space.

**Restore**: On next startup, latest save is automatically restored to RAM disk.

---

## Manual Usage

### Startup

```bash

# Run startup manually

./scripts/workspace_ram_disk_manager.sh startup
```

### Save Current Build

```bash

# Save without shutdown

./scripts/workspace_ram_disk_manager.sh save
```

### Shutdown & Save

```bash

# Save and optionally unmount

./scripts/workspace_ram_disk_manager.sh shutdown
```

### Check Status

```bash

# Show RAM disk status

./scripts/workspace_ram_disk_manager.sh status
```

---

## Troubleshooting

### RAM Disk Not Auto-Starting

**Check 1**: Verify task exists

```bash

# In Cursor: Cmd+Shift+P → "Tasks: Run Task" → "RAM Disk: Startup (Auto)"
```

**Check 2**: Check shell profile

```bash

# Verify startup command in ~/.zshrc or ~/.bashrc

grep "workspace_ram_disk_manager" ~/.zshrc ~/.bashrc
```

**Check 3**: Run manually

```bash
./scripts/workspace_ram_disk_manager.sh startup
```

### Save Not Working

**Check 1**: Verify RAM disk exists

```bash
./scripts/workspace_ram_disk_manager.sh status
```

**Check 2**: Check disk space

```bash
df -h /Volumes/IBBoxSpreadBuild
```

**Check 3**: Manual save

```bash
./scripts/workspace_ram_disk_manager.sh save
```

### Saved Build Not Restoring

**Check saved builds**:

```bash
ls -lt .saved-builds/
```

**Manual restore**:

```bash

# Copy from latest save

cp -r .saved-builds/$(ls -t .saved-builds/ | head -1)/* build-ramdisk/
```

---

## Advanced Configuration

### Custom RAM Disk Size

```bash

# Set custom size (before startup)

export RAMDISK_SIZE_GB=16
./scripts/workspace_ram_disk_manager.sh startup
```

### Auto-Unmount on Shutdown

Edit `scripts/workspace_ram_disk_manager.sh`:

```bash

# In handle_shutdown() function, uncomment:
# "${SCRIPT_DIR}/setup_ramdisk.sh" unmount || true
```

### Periodic Auto-Save

Add to your shell profile:

```bash

# Auto-save every 30 minutes (background)

cursor_auto_save() {
  while true; do
    sleep 1800  # 30 minutes
    if [[ "$PWD" == *"ib_box_spread_full_universal"* ]]; then
      "${PWD}/scripts/workspace_ram_disk_manager.sh" save >/dev/null 2>&1
    fi
  done
}

# Start in background

cursor_auto_save &
```

---

## Integration with Other Tools

### Git Hooks

Add to `.git/hooks/pre-commit`:

```bash

#!/bin/bash
# Save RAM disk before commit

"${PWD}/scripts/workspace_ram_disk_manager.sh" save >/dev/null 2>&1
```

### Build Scripts

Modify build scripts to use RAM disk:

```bash

# In build scripts, use build-ramdisk if available

if [ -d "build-ramdisk" ]; then
  BUILD_DIR="build-ramdisk"
else
  BUILD_DIR="build"
fi
```

---

## Summary

✅ **Auto-startup**: Runs on workspace open (via shell hook or manual)

✅ **Auto-save**: Run `Cmd+Shift+R` before closing, or set up shell hook

✅ **Tasks**: Available in Command Palette

✅ **Shortcuts**: `Cmd+Shift+R` (save), `Cmd+Shift+S` (save now), `Cmd+Shift+T` (status)

✅ **Saved builds**: Automatic restore on next startup

**Next steps**:

1. Add startup command to shell profile
2. Copy keyboard shortcuts to keybindings.json
3. Test with: `./scripts/workspace_ram_disk_manager.sh startup`

For questions, see `docs/RAM_OPTIMIZATION_GUIDE.md` or open an issue.
