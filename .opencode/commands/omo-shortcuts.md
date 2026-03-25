---
description: List Oh My OpenCode keyboard shortcuts
---

# Keyboard Shortcuts

Oh My OpenCode shortcuts for Aether.

## Navigation

| Shortcut | Command | When to Use |
|----------|---------|-------------|
| `p` | `/prime` | Start of session |
| `w` | `/welcome` | Get oriented |
| `t` | `/tasks` | List tasks |

## Development

| Shortcut | Command | When to Use |
|----------|---------|-------------|
| `b` | `/build` | Build project |
| `bt` | `/build --test` | Build and test |
| `l` | `/lint` | Run linters |

## Project Health

| Shortcut | Command | When to Use |
|----------|---------|-------------|
| `s` | `/scorecard` | Project overview |
| `h` | `/health` | Health checks |
| `f` | `/followup` | After completing work |

## Adding Custom Shortcuts

Edit `~/.config/opencode/projects/aether.json`:

```json
{
  "shortcuts": {
    "custom": "your-command",
    "r": "cargo run -p backend_service"
  }
}
```

## Tips

- Shortcuts work anywhere in the prompt
- Type `?` to see available shortcuts
- Shortcuts can have arguments: `b --release`
