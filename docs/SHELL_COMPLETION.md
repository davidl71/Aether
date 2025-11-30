# Shell Completion Guide

Shell completion provides tab-completion for command-line options and arguments, making the CLI tools easier to use.

## Supported Shells

- **Bash** (macOS default)
- **Zsh** (macOS default since Catalina)
- **Fish** (modern shell)

## Quick Start

### Generate Completions

```bash

# Generate all completion scripts

./scripts/generate_completions.sh

# Generate for specific shell

./scripts/generate_completions.sh bash
./scripts/generate_completions.sh zsh
./scripts/generate_completions.sh fish
```

### Install Completions

```bash

# Install for all shells

./scripts/install_completions.sh

# Install for specific shell

./scripts/install_completions.sh bash
./scripts/install_completions.sh zsh
./scripts/install_completions.sh fish

# Auto-detect shell

./scripts/install_completions.sh auto
```

## Manual Installation

### Bash

```bash

# Generate completion

./scripts/generate_completions.sh bash

# Source in current session

source completions/ib_box_spread.bash

# Add to ~/.bashrc for persistence

echo "source $(pwd)/completions/ib_box_spread.bash" >> ~/.bashrc
```

### Zsh

```bash

# Generate completion

./scripts/generate_completions.sh zsh

# Add to fpath

fpath=($(pwd)/completions $fpath)

# Initialize completions

compinit

# Add to ~/.zshrc for persistence

echo "fpath=($(pwd)/completions \$fpath)" >> ~/.zshrc
echo "autoload -U compinit && compinit" >> ~/.zshrc
```

### Fish

```bash

# Generate completion

./scripts/generate_completions.sh fish

# Copy to fish completions directory

mkdir -p ~/.config/fish/completions
cp completions/*.fish ~/.config/fish/completions/

# Completions load automatically in new sessions
```

## Using Completions

After installation, you can use tab completion:

```bash

# Tab-complete options

ib_box_spread --<TAB>

# Shows: --config --dry-run --validate --use-nautilus --log-level --version --help

# Tab-complete log levels

ib_box_spread --log-level <TAB>

# Shows: trace debug info warn error

# Tab-complete config files

ib_box_spread --config <TAB>

# Shows: config/config.json config/config.example.json

# TUI completions

ib-box-spread-tui --<TAB>

# Shows: --mock --endpoint --help
```

## Completion Features

### ib_box_spread

- **Options**: All command-line flags
- **Config files**: JSON file completion
- **Log levels**: Predefined log level values
- **File paths**: Smart file path completion

**Available Options:**

- `-c, --config` - Configuration file path
- `--dry-run` - Simulate trading without executing orders
- `--validate` - Validate configuration and exit
- `--use-nautilus` - Use nautilus_trader integration
- `--log-level` - Override log level (trace|debug|info|warn|error)
- `-v, --version` - Show version information
- `-h, --help` - Show help message

### ib-box-spread-tui

- **Options**: Command-line flags
- **Endpoints**: API endpoint URL completion

**Available Options:**

- `--mock` - Use mock data provider
- `--endpoint` - API endpoint URL
- `-h, --help` - Show help message

**Environment Variables** (for TUI):

- `TUI_BACKEND` - Backend type (mock|rest|nautilus)
- `TUI_API_URL` - API endpoint URL (when using rest backend)

## Troubleshooting

### Completions Not Working

1. **Check installation**:

   ```bash
   # Bash
   type _ib_box_spread

   # Zsh
   which _ib_box_spread

   # Fish
   complete -c ib_box_spread
   ```

2. **Reload shell configuration**:

   ```bash
   # Bash
   source ~/.bashrc

   # Zsh
   source ~/.zshrc

   # Fish
   # Restart terminal (completions load automatically)
   ```

3. **Regenerate completions**:

   ```bash
   ./scripts/generate_completions.sh
   ./scripts/install_completions.sh
   ```

### Binary Not Found

If the binary isn't found for completion generation:

```bash

# Build first

./scripts/build_universal.sh

# Then generate completions

./scripts/generate_completions.sh
```

### Permission Errors

If you get permission errors during installation:

```bash

# Use sudo for system-wide installation

sudo ./scripts/install_completions.sh

# Or install to user directory (no sudo needed)

./scripts/install_completions.sh
```

## How It Works

### CLI11 (C++ Binary)

The C++ binary uses [CLI11](https://github.com/CLIUtils/CLI11) which provides built-in completion generation:

```bash

# CLI11 automatically adds --generate-completion flag

ib_box_spread --generate-completion bash
ib_box_spread --generate-completion zsh
ib_box_spread --generate-completion fish
```

Our scripts use this feature when available, falling back to manual completions if needed.

### Manual Completions

For cases where CLI11 completion generation isn't available, we provide manually crafted completion scripts that:

- Complete all known options
- Provide file path completion for config files
- Suggest valid values for options (e.g., log levels)
- Handle both short and long option forms

## Integration with Homebrew

When installed via Homebrew, completions can be installed system-wide:

```bash

# Install via Homebrew

brew install davidl71/ib-box-spread/ib-box-spread

# Completions are available in the formula
# They can be sourced from the installation directory
```

## Updating Completions

When adding new CLI options, update completions:

```bash

# Regenerate completions

./scripts/generate_completions.sh

# Reinstall

./scripts/install_completions.sh
```

## Files

- `scripts/generate_completions.sh` - Generate completion scripts
- `scripts/install_completions.sh` - Install completion scripts
- `completions/` - Generated completion scripts directory
  - `ib_box_spread.bash` - Bash completion
  - `_ib_box_spread` - Zsh completion
  - `ib_box_spread.fish` - Fish completion
  - `ib-box-spread-tui.bash` - TUI Bash completion
  - `_ib-box-spread-tui` - TUI Zsh completion
  - `ib-box-spread-tui.fish` - TUI Fish completion

## See Also

- [CLI11 Documentation](https://cliutils.github.io/CLI11/)
- [Bash Completion Guide](https://www.gnu.org/software/bash/manual/html_node/Programmable-Completion.html)
- [Zsh Completion System](http://zsh.sourceforge.net/Doc/Release/Completion-System.html)
- [Fish Completion Guide](https://fishshell.com/docs/current/completions.html)
