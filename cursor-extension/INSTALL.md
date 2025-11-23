# Installation Guide - Project Management Automation Extension

## Quick Install

### Option 1: Install from VSIX (Recommended)

1. **Build the extension:**
   ```bash
   cd cursor-extension
   npm install
   npm run compile
   npm run package
   ```

2. **Install in Cursor:**
   - Open Cursor
   - Press `Cmd+Shift+P` (Mac) or `Ctrl+Shift+P` (Windows/Linux)
   - Type: "Extensions: Install from VSIX..."
   - Select `project-management-automation-0.1.0.vsix`

### Option 2: Development Mode

1. **Open extension folder:**
   ```bash
   cursor cursor-extension
   ```

2. **Install dependencies:**
   ```bash
   npm install
   ```

3. **Press F5** to launch extension development host

4. **Test commands** in the new Cursor window

## Prerequisites

- ✅ Node.js 18+ and npm
- ✅ TypeScript 5.0+
- ✅ Project Management Automation MCP Server configured
- ✅ Virtual environment with FastMCP installed

## Verification

After installation:

1. **Check commands are available:**
   - `Cmd+Shift+P` → Type "Project Automation"
   - You should see all 11 commands listed

2. **Test a command:**
   - `Cmd+Shift+P` → "Project Automation: Show Server Status"
   - Should show server status in output channel

3. **Check output:**
   - View → Output
   - Select "Project Automation" channel

## Troubleshooting

### Extension Not Appearing

1. **Reload window:** `Cmd+Shift+P` → "Developer: Reload Window"
2. **Check activation:** Extension activates on startup
3. **Check logs:** Help → Toggle Developer Tools → Console

### Commands Not Working

1. **Check workspace:** Extension requires a workspace folder
2. **Check server path:** Verify `mcp-servers/project-management-automation/` exists
3. **Check venv:** Verify `venv/bin/python3` exists
4. **Check permissions:** Ensure `run_server.sh` is executable

### Python Errors

1. **Check Python path:** Verify venv Python is correct
2. **Check dependencies:** Run `pip install -r requirements.txt` in venv
3. **Check logs:** Review output channel for detailed errors

## Next Steps

- See `README.md` for usage instructions
- See `../mcp-servers/project-management-automation/README.md` for MCP server details
