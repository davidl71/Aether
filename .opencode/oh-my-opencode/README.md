---
description: Oh My OpenCode - Enhanced Aether workflow
---

# 🚀 Oh My OpenCode for Aether

Enhanced OpenCode configuration with themes, shortcuts, and automation.

## Installation

```bash
# Copy config to your Oh My OpenCode directory
cp .opencode/oh-my-opencode/config.json ~/.config/opencode/projects/aether.json

# Or symlink for live updates
ln -s $(pwd)/.opencode/oh-my-opencode/config.json ~/.config/opencode/projects/aether.json
```

## Features

### 🎨 Custom Theme
- Cyan primary (`#00d4ff`) - matches Aether branding
- Purple secondary (`#7c3aed`)
- Semantic colors for success/warning/error

### ⌨️ Keyboard Shortcuts

| Shortcut | Command | Description |
|----------|---------|-------------|
| `p` | `prime` | Prime session |
| `t` | `tasks` | List tasks |
| `s` | `scorecard` | Project health |
| `h` | `health` | Health checks |
| `b` | `build` | Build project |
| `bt` | `build --test` | Build and test |
| `l` | `lint` | Run linters |
| `f` | `followup` | Follow-up suggestions |
| `w` | `welcome` | Show welcome |

### 🪝 Automated Hooks

**Pre-command:**
- Check PROJECT_ROOT is set
- Refresh task cache

**Post-command:**
- Show in-progress task reminders

**Session start:**
- Auto-prime session
- Show welcome message

### 📊 Enhanced Prompt

```
[3 tasks] [main] 12ms › 
```

Shows:
- Active task count
- Git branch
- Last command duration

## Commands

### `/omo-status`
Show Oh My OpenCode status and configuration.

### `/omo-theme`
Switch between available themes:
- `aether` (default) - Cyan/purple
- `dark` - High contrast
- `minimal` - Clean

### `/omo-shortcuts`
List all keyboard shortcuts.

### `/omo-hooks`
Enable/disable hooks:
- `omo-hooks enable auto-prime`
- `omo-hooks disable task-reminders`

## Customization

Edit `~/.config/opencode/projects/aether.json`:

```json
{
  "theme": {
    "primary": "#your-color"
  },
  "shortcuts": {
    "custom": "your-command"
  }
}
```

## Integration with exarp-go

### Auto-prime on startup
Set in config:
```json
{
  "integrations": {
    "exarp-go": {
      "autoPrime": true
    }
  }
}
```

### Task count in prompt
Shows `[N tasks]` in prompt when enabled.

### Task completion notifications
Get notified when tasks are completed by other agents.

## Workflows

### Quick Start
```
w          # Welcome
p          # Prime
t          # Tasks
```

### Development Loop
```
b          # Build
bt         # Build and test
l          # Lint
```

### Task Management
```
t          # List tasks
<work>
exarp_update_task(task_id="T-...", new_status="Done")
f          # Follow-up suggestions
```

## Tips

1. **Use shortcuts** - They're much faster
2. **Enable auto-prime** - Never forget to prime
3. **Customize theme** - Make it yours
4. **Add your own hooks** - Automate repetitive tasks

## Troubleshooting

**Shortcuts not working**: Check `~/.config/opencode/shortcuts.json`

**Theme not applying**: Verify config path and JSON syntax

**Hooks not firing**: Check `~/.config/opencode/hooks.log`
