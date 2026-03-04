# Changelog

## Version 1.1.0 - Context-Aware Task Lists & MOTD

### New Features

#### Context-Aware Task Lists
- **`exarp_tasks`** function - Shows tasks filtered by current directory/git repo context
- **`exarp_motd`** function - Displays task summary MOTD (Message of the Day)
- **`exarp_task_summary`** function - Returns task summary as JSON
- Automatic detection of:
  - Git repository name
  - Current folder/module context
  - Project root directory

#### Task Filtering
Tasks are automatically filtered based on:
- Current folder name (e.g., `backend`, `web`, `native`)
- Module/submodule paths (e.g., `agents/backend`, `native/src`)
- Git repository name
- Task tags matching folder/module names
- Task descriptions mentioning folder/module

#### MOTD (Message of the Day)
- Quick task summary on shell startup (optional)
- Shows task counts by status:
  - ⚠️ Pending review
  - 🔄 In progress
  - 📝 Todo
  - 🚫 Blocked
  - ✅ Done
- Displays current repository/folder context

### New Aliases
- `ext` - `exarp_tasks` (context-aware task list)
- `exm` - `exarp_motd` (task summary MOTD)

### New Helper Script
- ~~`exarp_context_tasks.py`~~ **Removed:** Context-aware task querying is now provided by exarp-go (`exarp_tasks`, `exarp_motd`). No Python exarp tools in this repo.

### Usage Examples

```bash
# Show context-aware task list
ext

# Show task summary MOTD
exm

# Get task summary as JSON
exarp_task_summary

# Enable MOTD on shell startup
# Edit exarp.plugin.zsh and uncomment:
# exarp_motd 2>/dev/null || true
```

### Technical Details

- Plugin version updated to 1.1.0
- Added `_exarp_plugin_dir()` helper function for script path detection
- Python script requires Python 3.9+
- Works with Todo2 task format (`.todo2/state.todo2.json`)

---

## Version 1.0.0 - Initial Release

### Features
- Basic exarp command aliases
- Documentation health checking
- Task alignment analysis
- Duplicate detection
- Security scanning
- Daily automation
- Automation opportunities discovery
- Zsh completion support
