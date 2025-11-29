# Quick Enable - Project Management Automation Extension

## One-Command Enable

```bash
cd cursor-extension && npm install && npm run compile && npm run package
```

This will create a `.vsix` file that you can install in Cursor.

## Install in Cursor

After building, install the extension:

1. **Open Cursor**
2. **Press `Cmd+Shift+P`** (Mac) or **`Ctrl+Shift+P`** (Linux/Windows)
3. **Type**: `Extensions: Install from VSIX...`
4. **Select**: The `.vsix` file from `cursor-extension/` directory
5. **Reload**: `Cmd+Shift+P` → `Developer: Reload Window`

## Verify Installation

After reloading:
- Check status bar (right side) for automation status indicator
- Press `Cmd+Shift+P` → Type `Project Automation` → See all commands listed

## Available Commands

Once installed, you'll have access to:
- Check Documentation Health
- Analyze Task Alignment
- Detect Duplicate Tasks
- Scan Dependencies for Security
- Find Automation Opportunities
- Sync Todo Tasks
- Review PWA Config
- Add External Tool Hints
- Run Daily Automation
- Show Server Status

## Troubleshooting

If the build fails:
1. Ensure Node.js 18+ is installed: `node --version`
2. Ensure npm is installed: `npm --version`
3. Try: `npm install --force` if dependencies conflict

If installation fails:
1. Check Cursor version (requires 1.80.0+)
2. Try installing via command line: `cursor --install-extension cursor-extension/*.vsix`
3. Check Cursor logs: Help → Toggle Developer Tools → Console
