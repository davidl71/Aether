# Status Bar Integration Guide

## Overview

The extension provides **three status bar items** for quick access and status monitoring:

1. **Automation Status** - Main tool access and operation status
2. **Server Status** - MCP server availability
3. **Last Operation** - Brief result display

---

## Status Bar Items

### 1. Automation Status (Main)

**Location:** Right side of status bar
**Icon:** `$(tools) Automation`
**Click Action:** Opens Quick Actions menu

**States:**
- **Idle:** `$(tools) Automation` (gray) - Ready for commands
- **Running:** `$(sync~spin) Running...` (blue) - Operation in progress
- **Success:** `$(check) Automation` (green) - Last operation succeeded
- **Error:** `$(error) Automation` (red) - Last operation failed

**Usage:**
- Click to open Quick Actions menu
- Shows current operation status
- Updates automatically during operations

---

### 2. Server Status

**Location:** Right side of status bar (next to Automation)
**Icon:** `$(check) Server Ready` or `$(warning) Server Not Found`
**Click Action:** Shows detailed server status

**States:**
- **Operational:** `$(check) Server Ready` (green) - Server available
- **Not Found:** `$(warning) Server Not Found` (yellow) - Server path missing
- **Error:** `$(error) Server Error` (red) - Server check failed
- **Checking:** `$(sync~spin) Checking...` (blue) - Status check in progress

**Usage:**
- Click to view server details
- Auto-checks on extension activation
- Shows server availability at a glance

---

### 3. Last Operation Status

**Location:** Right side of status bar (temporary)
**Icon:** `$(check)` or `$(error)`
**Click Action:** None (informational only)

**States:**
- **Success:** `$(check) [Message]` (green) - Shows for 3 seconds
- **Error:** `$(error) [Message]` (red) - Shows for 5 seconds
- **Hidden:** Not shown when idle

**Messages:**
- `Docs: 85` - Documentation health score
- `Tasks: 3 misaligned` - Task alignment results
- `Sync: 5 matches, 2 new` - Task sync results
- `Pre-sprint complete` - Workflow completion
- `Maintenance complete` - Weekly maintenance done

**Usage:**
- Automatically appears after operations
- Shows brief result summary
- Auto-hides after timeout

---

## Quick Actions Menu

**Access:** Click "Automation" status bar item

**Menu Items:**
- 📄 Documentation Health
- 📋 Task Alignment
- 🔄 Duplicate Tasks
- 🛡️ Security Scan
- 🔄 Sync Tasks
- 🚀 Pre-Sprint Cleanup
- ✅ Weekly Maintenance
- ℹ️ Server Status

**Usage:**
1. Click Automation status bar
2. Select tool or workflow
3. Follow prompts
4. View results in Output channel

---

## Status Indicators

### Color Coding

- **Green** - Success, operational, ready
- **Blue** - In progress, checking
- **Yellow** - Warning, needs attention
- **Red** - Error, failed operation

### Icons

- `$(tools)` - Automation tools available
- `$(sync~spin)` - Operation in progress
- `$(check)` - Success, operational
- `$(error)` - Error occurred
- `$(warning)` - Warning condition

---

## Examples

### Example 1: Running Documentation Check

1. **Before:** Status shows `$(tools) Automation` (idle)
2. **During:** Status shows `$(sync~spin) Running...` (blue, spinning)
3. **After:** Status shows `$(check) Automation` (green)
4. **Brief:** Last Operation shows `$(check) Docs: 85` for 3 seconds

### Example 2: Server Check

1. **On activation:** Server status shows `$(sync~spin) Checking...`
2. **If found:** Changes to `$(check) Server Ready` (green)
3. **If missing:** Changes to `$(warning) Server Not Found` (yellow)

### Example 3: Quick Access

1. **Click Automation status** → Quick Actions menu opens
2. **Select "Pre-Sprint Cleanup"** → Workflow starts
3. **Status updates** → Shows progress through each step
4. **Completion** → Shows success with brief message

---

## Customization

Status bar items are automatically managed by the extension:
- **Position:** Right side (StatusBarAlignment.Right)
- **Priority:** 100 (Automation), 99 (Server), 98 (Last Operation)
- **Visibility:** Auto-shown/hidden based on state
- **Colors:** Use VS Code theme colors

---

## Troubleshooting

### Status Bar Not Showing

1. **Reload window:** `Cmd+Shift+P` → "Developer: Reload Window"
2. **Check activation:** Extension activates on startup
3. **Check workspace:** Requires workspace folder

### Status Not Updating

1. **Check operations:** Status only updates during operations
2. **Check logs:** View Output channel for errors
3. **Restart extension:** Reload window

### Quick Actions Not Working

1. **Click status bar:** Must click the Automation status item
2. **Check commands:** Verify commands are registered
3. **Check logs:** View Developer Tools console

---

**Status bar provides at-a-glance status and quick access to all automation tools!**
