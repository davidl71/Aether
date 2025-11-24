# Exarp Oh My Zsh Plugin

Oh My Zsh plugin for Exarp project management automation tools.

## Installation

### Option 1: Custom Plugins Directory (Recommended)

1. Clone or copy this plugin to your Oh My Zsh custom plugins directory:

```bash
# Create custom plugins directory if it doesn't exist
mkdir -p ~/.oh-my-zsh/custom/plugins

# Copy the plugin
cp -r scripts/oh-my-zsh-exarp-plugin ~/.oh-my-zsh/custom/plugins/exarp

# Or create a symlink (if plugin is in your project)
ln -s /path/to/ib_box_spread_full_universal/scripts/oh-my-zsh-exarp-plugin ~/.oh-my-zsh/custom/plugins/exarp
```

2. Enable the plugin in your `~/.zshrc`:

```zsh
plugins=(... exarp ...)
```

3. Reload your shell:

```bash
source ~/.zshrc
```

### Option 2: Direct Sourcing

Add to your `~/.zshrc`:

```zsh
source /path/to/ib_box_spread_full_universal/scripts/oh-my-zsh-exarp-plugin/exarp.plugin.zsh
```

## Features

### Functions

- `exarp_check` - Check if exarp is installed
- `exarp_server` - Run Exarp MCP server
- `exarp_docs_health [project_dir]` - Check documentation health
- `exarp_task_align [project_dir]` - Analyze task alignment
- `exarp_duplicates [project_dir]` - Detect duplicate tasks
- `exarp_security [project_dir]` - Scan dependencies for security issues
- `exarp_daily [project_dir]` - Run daily automation tasks
- `exarp_opportunities [project_dir]` - Find automation opportunities
- `exarp_status` - Show exarp status and available commands
- `exarp_tasks` - Show context-aware task list (filtered by current directory/git repo)
- `exarp_motd` - Show task summary MOTD (Message of the Day)
- `exarp_task_summary` - Get task summary as JSON

### Aliases

- `exd` - `exarp_docs_health`
- `exa` - `exarp_task_align`
- `exdup` - `exarp_duplicates`
- `exsec` - `exarp_security`
- `exday` - `exarp_daily`
- `exopp` - `exarp_opportunities`
- `exsrv` - `exarp_server`
- `exstat` - `exarp_status`
- `excheck` - `exarp_check`
- `ext` - `exarp_tasks` (context-aware task list)
- `exm` - `exarp_motd` (task summary MOTD)

## Usage Examples

```bash
# Check if exarp is installed
excheck

# Check documentation health for current directory
exd

# Check documentation health for specific project
exd /path/to/project

# Analyze task alignment
exa

# Detect duplicate tasks
exdup

# Scan dependencies for security issues
exsec

# Run daily automation
exday

# Find automation opportunities
exopp

# Show all available commands
exstat

# Run exarp MCP server
exsrv

# Show context-aware task list (filtered by current directory)
ext

# Show task summary MOTD
exm
```

## Requirements

- Python 3.9+
- Exarp installed: `pip install exarp-automation-mcp`

## Completion Support

The plugin includes basic zsh completion support. After enabling the plugin, you can use tab completion:

```bash
exarp <TAB>
```

## Context-Aware Task Lists

The plugin includes context-aware task filtering that automatically shows tasks relevant to your current directory:

- **Git Repository Detection**: Automatically detects git repo name and filters tasks
- **Directory-Based Filtering**: Matches tasks by folder name, module path, and tags
- **Smart Matching**: Checks task names, descriptions, content, and tags against current context

### How It Works

When you run `exarp_tasks` or `exarp_motd`, the plugin:

1. Detects your current directory and project root
2. Identifies git repository name (if available)
3. Filters Todo2 tasks that match:
   - Current folder name
   - Module/submodule paths
   - Git repository name
   - Task tags
4. Displays relevant tasks or summary

### Example

```bash
# In agents/backend directory
cd agents/backend
ext  # Shows tasks tagged with "backend" or mentioning "backend"

# In native/src directory
cd native/src
ext  # Shows tasks related to native C++ code

# At project root
cd ~/projects/my-repo
ext  # Shows all tasks for the project
```

## MOTD (Message of the Day)

The MOTD feature provides a quick summary of tasks when you open a new terminal:

```bash
# Show MOTD manually
exm

# Enable MOTD on shell startup (edit exarp.plugin.zsh)
# Uncomment: exarp_motd 2>/dev/null || true
```

The MOTD shows:
- Current repository/folder context
- Task counts by status (Review, In Progress, Todo, Blocked, Done)
- Total task count

## Customization

You can customize the plugin by editing `exarp.plugin.zsh`:

- Modify aliases to match your preferences
- Add additional helper functions
- Enable/disable auto-check on plugin load (uncomment line)
- Enable MOTD on shell startup (uncomment line at bottom)

## Troubleshooting

### Plugin not loading

1. Check that the plugin is in the correct location:
   ```bash
   ls ~/.oh-my-zsh/custom/plugins/exarp/exarp.plugin.zsh
   ```

2. Verify it's enabled in `~/.zshrc`:
   ```bash
   grep exarp ~/.zshrc
   ```

3. Reload your shell:
   ```bash
   source ~/.zshrc
   ```

### Commands not found

1. Check if exarp is installed:
   ```bash
   excheck
   ```

2. Install exarp if needed:
   ```bash
   pip install exarp-automation-mcp
   ```

### Completion not working

Completion requires `compdef` to be available. If completion doesn't work, you can still use the functions and aliases directly.

## Contributing

To improve this plugin:

1. Edit `exarp.plugin.zsh`
2. Test your changes
3. Update this README if needed

## License

Same license as the main project.
