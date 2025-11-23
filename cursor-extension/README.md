# Project Management Automation - Cursor Extension

Cursor/VS Code extension for project management automation tools.

## Features

### Status Bar Integration ✨

**Three status bar items:**
1. **Automation Status** (Right side) - Shows current operation status
   - Click to open Quick Actions menu
   - Shows: `$(tools) Automation` (idle), `$(sync~spin) Running...` (active), `$(check) Automation` (success), `$(error) Automation` (error)

2. **Server Status** (Right side) - Shows MCP server availability
   - Click to check server status
   - Shows: `$(check) Server Ready` (operational), `$(warning) Server Not Found` (error)

3. **Last Operation** (Right side) - Shows brief result of last operation
   - Auto-hides after 3-5 seconds
   - Shows success/error messages

### Individual Tools
- **Documentation Health Check** - Analyze documentation structure and health
- **Task Alignment Analysis** - Check Todo2 task alignment with project goals
- **Duplicate Task Detection** - Find and consolidate duplicate tasks
- **Security Scanning** - Scan dependencies for vulnerabilities
- **Automation Discovery** - Find automation opportunities
- **Task Synchronization** - Sync tasks across systems
- **PWA Review** - Review PWA configuration

### Workflow Commands
- **Pre-Sprint Cleanup** - Complete workflow before starting new work
- **Post-Implementation Review** - Review workflow after completing features
- **Weekly Maintenance** - Regular maintenance routine

### Quick Actions Menu
- **Click Automation status bar** → Quick pick menu with all tools
- Fast access to most-used commands
- Organized by category

## Installation

### From Source

1. **Install dependencies:**
   ```bash
   cd cursor-extension
   npm install
   ```

2. **Compile:**
   ```bash
   npm run compile
   ```

3. **Package extension:**
   ```bash
   npm run package
   ```

4. **Install in Cursor:**
   - Open Cursor
   - `Cmd+Shift+P` → "Extensions: Install from VSIX..."
   - Select the generated `.vsix` file

### Development

1. **Open in Cursor:**
   ```bash
   cursor cursor-extension
   ```

2. **Press F5** to launch extension development host

3. **Test commands:**
   - `Cmd+Shift+P` → "Project Automation: ..."

## Usage

### Status Bar (Quick Access)

**Click the status bar items for quick access:**

1. **Click "Automation" status** → Opens Quick Actions menu
   - Select any tool or workflow
   - Fastest way to access tools

2. **Click "Server Ready" status** → Shows server status
   - Check if MCP server is operational
   - View available tools

3. **Last Operation status** → Shows briefly after operations
   - Success/error indicators
   - Auto-hides after a few seconds

### Command Palette

All commands are available via Command Palette (`Cmd+Shift+P`):

- `Project Automation: Check Documentation Health`
- `Project Automation: Analyze Task Alignment`
- `Project Automation: Detect Duplicate Tasks`
- `Project Automation: Scan Dependencies for Security`
- `Project Automation: Discover Automation Opportunities`
- `Project Automation: Sync Tasks Across Systems`
- `Project Automation: Review PWA Configuration`
- `Project Automation: Pre-Sprint Cleanup Workflow`
- `Project Automation: Post-Implementation Review Workflow`
- `Project Automation: Weekly Maintenance Workflow`
- `Project Automation: Show Server Status`
- `Project Automation: Show Quick Actions`

### Workflows

**Pre-Sprint Cleanup:**
1. Detects duplicate tasks
2. Analyzes task alignment
3. Checks documentation health

**Post-Implementation Review:**
1. Updates documentation
2. Scans for security vulnerabilities
3. Discovers automation opportunities

**Weekly Maintenance:**
1. Checks documentation health
2. Cleans up duplicate tasks
3. Scans dependencies for security
4. Synchronizes tasks across systems

## Requirements

- Cursor IDE or VS Code 1.80+
- Project Management Automation MCP Server configured
- Python 3.9+ with virtual environment
- FastMCP installed in venv

## Configuration

The extension automatically detects:
- Project root from workspace
- MCP server location (`mcp-servers/project-management-automation/`)
- Virtual environment (`venv/bin/python3`)

## Output

All commands output results to:
- Output channels (View → Output → Select channel)
- Report files in `docs/` directory
- Notification messages

## Troubleshooting

### Commands Not Appearing

1. **Reload window:** `Cmd+Shift+P` → "Developer: Reload Window"
2. **Check activation:** Extension activates on startup
3. **Check logs:** View → Output → "Project Automation"

### Server Not Found

1. **Verify server path:** Check `mcp-servers/project-management-automation/run_server.sh` exists
2. **Check venv:** Verify `venv/bin/python3` exists
3. **Check permissions:** Ensure `run_server.sh` is executable

### Tool Execution Errors

1. **Check Python path:** Verify venv Python is correct
2. **Check dependencies:** Ensure all tools are installed
3. **Check logs:** Review output channel for detailed errors

## Development

### Project Structure

```
cursor-extension/
├── src/
│   └── extension.ts      # Main extension code
├── package.json          # Extension manifest
├── tsconfig.json         # TypeScript config
└── README.md            # This file
```

### Building

```bash
npm run compile      # Compile TypeScript
npm run watch        # Watch mode
npm run package      # Create .vsix package
```

## License

Same as main project.
