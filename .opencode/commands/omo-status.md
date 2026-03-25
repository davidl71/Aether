---
description: Oh My OpenCode status and configuration
---

# Oh My OpenCode Status

Check Oh My OpenCode configuration and features.

## Configuration

### Project Config
```json
{
  "name": "Aether",
  "theme": "aether",
  "shortcuts": 9,
  "hooks": 3
}
```

### Active Features

✅ **Theme**: Aether (cyan/purple)
✅ **Shortcuts**: 9 custom shortcuts
✅ **Hooks**: Pre/post command hooks
✅ **exarp-go**: Auto-prime enabled
✅ **Git**: Branch in prompt

### Shortcuts

| Key | Command |
|-----|---------|
| p | prime |
| t | tasks |
| s | scorecard |
| h | health |
| b | build |
| bt | build --test |
| l | lint |
| f | followup |
| w | welcome |

### Hooks

**Pre-command:**
- ✅ check_project_root
- ✅ refresh_task_cache

**Post-command:**
- ✅ show_task_reminders

**Session start:**
- ✅ prime_session
- ✅ show_welcome

## Commands

Use these to customize:

```bash
/omo-theme <name>        # Switch theme
/omo-shortcuts           # List shortcuts
/omo-hooks enable <name> # Enable hook
/omo-hooks disable <name> # Disable hook
```

## Customization

Edit `~/.config/opencode/projects/aether.json` to customize.
