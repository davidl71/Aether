# Enable Project Management Automation Cursor Extension

## Quick Enable

Run the enable script:

```bash
cd cursor-extension
./enable-extension.sh
```

This will:
1. Install dependencies
2. Compile TypeScript
3. Package the extension
4. Provide installation instructions

## Manual Installation

### Step 1: Build Extension

```bash
cd cursor-extension
npm install
npm run compile
npm run package
```

### Step 2: Install in Cursor

**Option A: Via Command Palette**
1. Open Cursor
2. Press `Cmd+Shift+P` (Mac) or `Ctrl+Shift+P` (Windows/Linux)
3. Type: `Extensions: Install from VSIX...`
4. Select the `.vsix` file from `cursor-extension/` directory

**Option B: Via Command Line**
```bash
cursor --install-extension cursor-extension/project-management-automation-*.vsix
```

### Step 3: Reload Window

After installation:
1. Press `Cmd+Shift+P` (Mac) or `Ctrl+Shift+P` (Windows/Linux)
2. Type: `Developer: Reload Window`
3. Press Enter

### Step 4: Verify Installation

1. **Check Status Bar**: Look for automation status indicator (right side)
2. **Test Commands**:
   - Press `Cmd+Shift+P` (Mac) or `Ctrl+Shift+P` (Windows/Linux)
   - Type: `Project Automation`
   - You should see all automation commands listed

## Available Commands

After installation, these commands are available:

- **Project Automation: Show Server Status**
- **Project Automation: Check Documentation Health**
- **Project Automation: Analyze Todo2 Alignment**
- **Project Automation: Detect Duplicate Tasks**
- **Project Automation: Scan Dependency Security**
- **Project Automation: Find Automation Opportunities**
- **Project Automation: Sync Todo Tasks**
- **Project Automation: Review PWA Config**
- **Project Automation: Add External Tool Hints**
- **Project Automation: Run Daily Automation**

## Status Bar Integration

The extension adds three status bar items:

1. **Automation Status** - Shows current operation status
2. **Server Status** - Shows MCP server availability
3. **Last Operation** - Shows brief result of last operation

Click the Automation status to open Quick Actions menu.

## Troubleshooting

### Extension Not Appearing

1. Reload window: `Cmd+Shift+P` → `Developer: Reload Window`
2. Check activation: Extension activates on startup
3. Check logs: Help → Toggle Developer Tools → Console

### Commands Not Working

1. Check workspace: Extension requires a workspace folder
2. Check server path: Verify `mcp-servers/project-management-automation/` exists
3. Check venv: Verify `venv/bin/python3` exists
4. Check permissions: Ensure `run_server.sh` is executable

### Python Errors

1. Check Python path: Verify venv Python is correct
2. Check dependencies: Run `pip install -r requirements.txt` in venv
