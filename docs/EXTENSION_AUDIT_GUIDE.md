# VS Code Extension Audit Guide

If you see 99 installed extensions but the CLI only shows a few, you need to check VS Code's Extensions view directly.

## How to Check All Installed Extensions

### Method 1: VS Code Extensions View

1. **Open Extensions View**
   - Press `Cmd+Shift+X` (Mac) or `Ctrl+Shift+X` (Windows/Linux)
   - Or click the Extensions icon in the Activity Bar

2. **View All Installed Extensions**
   - Click the filter dropdown (top right)
   - Select "Installed" to see all installed extensions
   - You can also filter by:
     - "Enabled" - Currently active extensions
     - "Disabled" - Installed but not active
     - "Workspace" - Enabled only in this workspace

3. **Export Extension List**
   - Use Command Palette: `Cmd+Shift+P` → "Extensions: Show Installed Extensions"
   - Or use the script: `./scripts/analyze_all_extensions.sh`

### Method 2: Check Extension Settings File

VS Code stores extension information in:
- **macOS**: `~/Library/Application Support/Code/User/globalStorage/storage.json`
- **Windows**: `%APPDATA%\Code\User\globalStorage\storage.json`
- **Linux**: `~/.config/Code/User/globalStorage/storage.json`

## Categorizing Your 99 Extensions

### 🔴 **Should Be Workspace-Only**

These extensions should be **disabled globally** and **enabled only in this workspace**:

#### Language Extensions
- C++: `ms-vscode.cpptools`, `ms-vscode.cmake-tools`
- Python: `ms-python.python`, `ms-python.vscode-pylance`, `ms-python.black-formatter`
- Rust: `rust-lang.rust-analyzer`
- TypeScript/JavaScript: `dbaeumer.vscode-eslint`
- Swift: `sswg.swift-lang`
- Any other language-specific extensions (Go, Java, C#, PHP, Ruby, etc.)

#### Build Tools
- `timonwong.shellcheck`
- Any project-specific linters or formatters

#### Project-Specific Tools
- `yutengjing.vscode-mcp` (MCP Integration)
- `prompttower.prompttower` (AI tools)

### 🤖 **AI/Assistant Extensions** (Consider Workspace-Only)

These can be global but may benefit from workspace-specific configs:
- `github.copilot`
- `github.copilot-chat`
- `amazonwebservices.codewhisperer-for-command-line-companion`

**Recommendation**: Keep global if you use them across projects, but ensure they respect workspace settings.

### 🟢 **Safe to Keep Global**

These extensions don't conflict across projects:
- `editorconfig.editorconfig`
- `redhat.vscode-yaml`
- `eamodio.gitlens`
- `yzhang.markdown-all-in-one`
- `davidanson.vscode-markdownlint`
- `streetsidesoftware.code-spell-checker`
- `usernamehw.errorlens`

### 🔴 **Should Be Disabled/Uninstalled**

Check for these unwanted extensions (see `.vscode/extensions.json` for full list):
- Unused language extensions (Go, Java, C#, PHP, Ruby, etc. if not used)
- Enterprise/Mainframe extensions (IBM i, COBOL, etc.)
- Docker/Kubernetes (if not used)
- Prettier (if using ESLint for formatting)

## Step-by-Step Audit Process

### Step 1: List All Extensions

1. Open Extensions view (`Cmd+Shift+X`)
2. Filter by "Installed"
3. Count total: Should show ~99 extensions

### Step 2: Categorize Each Extension

For each extension, ask:
1. **Is it a language extension?** → Should be workspace-only
2. **Is it project-specific?** → Should be workspace-only
3. **Is it in the unwanted list?** → Should be disabled
4. **Is it a universal tool?** → Can be global

### Step 3: Take Action

#### For Workspace-Only Extensions:
1. Find the extension
2. Click the gear icon ⚙️
3. Select "Disable" (globally)
4. Re-enable only in this workspace

#### For Unwanted Extensions:
1. Find the extension
2. Click the gear icon ⚙️
3. Select "Uninstall" or "Disable"

### Step 4: Verify

Run the analysis script:
```bash
./scripts/analyze_all_extensions.sh
```

Or check manually:
- Extensions view → Filter by "Enabled in Workspace"
- Should only show workspace-recommended extensions

## Quick Reference: Extension Categories

### Language Extensions (Workspace-Only)
```
C++: ms-vscode.cpptools, ms-vscode.cmake-tools
Python: ms-python.*
Rust: rust-lang.rust-analyzer
TypeScript: dbaeumer.vscode-eslint
Swift: sswg.swift-lang
Go: golang.go (if not used)
Java: vscjava.* (if not used)
C#: ms-dotnettools.* (if not used)
```

### Universal Tools (Global OK)
```
EditorConfig: editorconfig.editorconfig
YAML: redhat.vscode-yaml
Git: eamodio.gitlens
Markdown: yzhang.markdown-all-in-one, davidanson.vscode-markdownlint
Spell Check: streetsidesoftware.code-spell-checker
Error Lens: usernamehw.errorlens
```

### AI Tools (Global OK, but consider workspace configs)
```
GitHub Copilot: github.copilot, github.copilot-chat
AWS CodeWhisperer: amazonwebservices.codewhisperer-for-command-line-companion
```

## Common Issues

### "Too Many Extensions"
If you have 99 extensions, many are likely:
- Language extensions for unused languages → Disable globally
- Theme/UI extensions → Can stay global
- Utility extensions → Review individually

### "VS Code is Slow"
- Disable unused language extensions globally
- Keep only workspace-needed extensions active
- Use `./scripts/analyze_all_extensions.sh` to identify issues

### "Extensions Conflict"
- Ensure language extensions are workspace-only
- Check for duplicate functionality (e.g., Prettier + ESLint)
- Review `.vscode/extensions.json` unwanted list

## Next Steps

1. **Run the analysis script**: `./scripts/analyze_all_extensions.sh`
2. **Review the output** for unwanted and workspace-only extensions
3. **Manually check VS Code Extensions view** for the full list
4. **Disable/uninstall** unwanted extensions
5. **Make language extensions workspace-only**
6. **Verify** with the script again

For detailed extension management, see [EXTENSION_MANAGEMENT.md](EXTENSION_MANAGEMENT.md).
