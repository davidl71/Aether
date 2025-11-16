# VS Code Extension Management Guide

This document outlines which extensions should be **workspace-only** vs **globally enabled** for optimal development experience and to prevent conflicts across projects.

## Extension Categories

### 🔴 **Workspace-Only Extensions** (Recommended)

These extensions should be **disabled globally** and **enabled only in this workspace** to prevent conflicts:

#### Language-Specific Extensions
- **C++**: `ms-vscode.cpptools`, `ms-vscode.cmake-tools`
  - Different projects may need different C++ standards or compiler settings
  - CMake configuration is project-specific

- **Python**: `ms-python.python`, `ms-python.vscode-pylance`, `ms-python.black-formatter`
  - Python versions and formatters vary by project
  - Virtual environments are project-specific

- **Rust**: `rust-lang.rust-analyzer`
  - Rust toolchain versions can differ between projects
  - Cargo workspace configurations are project-specific

- **TypeScript/JavaScript**: `dbaeumer.vscode-eslint`
  - ESLint configurations are project-specific
  - Different projects may use different TypeScript versions

- **Swift**: `sswg.swift-lang`
  - Swift toolchain versions vary
  - Package.swift configurations are project-specific

#### Project-Specific Tools
- **MCP Integration**: `yutengjing.vscode-mcp`
  - MCP server configurations are workspace-specific

- **AI Tools**: `prompttower.prompttower`
  - Project-specific AI configurations

#### AI/Assistant Extensions (Consider Workspace-Only)
- **GitHub Copilot**: `github.copilot`, `github.copilot-chat`
  - Can be global but may benefit from workspace-specific settings
  - Consider workspace-only if you want project-specific suggestions

- **AWS CodeWhisperer**: `amazonwebservices.codewhisperer-for-command-line-companion`
  - Similar to Copilot - can be global but workspace configs may help

#### Build & Linting Tools
- **ShellCheck**: `timonwong.shellcheck`
  - Different projects may have different shell script standards

### 🟢 **Global Extensions** (Safe to Enable Globally)

These extensions can be safely enabled globally as they don't conflict:

#### General Utilities
- **EditorConfig**: `editorconfig.editorconfig`
  - Respects `.editorconfig` files per-project
  - No conflicts across projects

- **YAML**: `redhat.vscode-yaml`
  - Generic YAML support, respects project schemas

#### Git Tools
- **GitLens**: `eamodio.gitlens`
  - Works with any Git repository
  - No project-specific conflicts

#### Markdown Tools
- **Markdown All in One**: `yzhang.markdown-all-in-one`
- **Markdown Lint**: `davidanson.vscode-markdownlint`
  - Markdown is universal, respects project-specific rules

#### Code Quality (Universal)
- **Code Spell Checker**: `streetsidesoftware.code-spell-checker`
  - Respects project-specific dictionaries
  - Can be configured per-workspace

- **Error Lens**: `usernamehw.errorlens`
  - Universal error highlighting
  - No conflicts

## How to Manage Extensions

### Disable Extensions Globally

1. Open VS Code Settings: `Cmd+,` (Mac) or `Ctrl+,` (Windows/Linux)
2. Go to Extensions
3. For each workspace-only extension:
   - Find the extension
   - Click the gear icon
   - Select "Disable (Workspace)" or "Disable" globally
   - Re-enable only in this workspace

### Enable Extensions in Workspace Only

1. Open Command Palette: `Cmd+Shift+P` (Mac) or `Ctrl+Shift+P` (Windows/Linux)
2. Type "Extensions: Show Recommended Extensions"
3. Install extensions marked as "Workspace Recommended"

### Verify Extension Scope

- **Workspace-only**: Shows "This extension is enabled for this workspace only"
- **Global**: Shows "This extension is enabled globally"

## Benefits of Workspace-Only Extensions

1. **Prevents Conflicts**: Different projects may need different extension versions or configurations
2. **Better Performance**: Only loads extensions needed for current project
3. **Team Consistency**: `.vscode/extensions.json` ensures all team members use same extensions
4. **Isolated Configuration**: Project-specific settings don't affect other projects

## Recommended Global Extensions

If you want to enable these globally (they're safe):

```json
{
  "recommendations": [
    "editorconfig.editorconfig",
    "eamodio.gitlens",
    "yzhang.markdown-all-in-one",
    "davidanson.vscode-markdownlint",
    "streetsidesoftware.code-spell-checker",
    "usernamehw.errorlens"
  ]
}
```

## Enterprise/Mainframe Extensions

This project explicitly excludes enterprise/mainframe extensions as they are not used in this trading application:

### Blocked Extensions

- **IBM i / AS/400**: Code for IBM i, IBM i Languages
- **IBM Z / Mainframe**: Z Open Editor, Code4z, Explorer for Endevor
- **COBOL**: Various COBOL language support extensions
- **Mainframe Languages**: JCL, PL/I, HLASM, REXX, Assembler
- **RPG**: RPGLE language support (IBM i)

These are listed in `unwantedRecommendations` to prevent installation prompts and improve performance.

## Troubleshooting

### Extension Conflicts

If you experience conflicts:
1. Check if extension is enabled globally and workspace
2. Disable globally, enable workspace-only
3. Reload VS Code window

### Performance Issues

If VS Code is slow:
1. Check which extensions are globally enabled
2. Disable unused language extensions globally
3. Enable only in workspaces that need them
4. Run `./scripts/check_extensions.sh` to identify problematic extensions

### Team Consistency

If team members have different extensions:
1. Ensure `.vscode/extensions.json` is committed
2. Team members should install workspace recommendations
3. Consider adding extension version pinning if needed

## Extension Version Pinning

To pin specific extension versions (advanced), you can use:

```json
{
  "recommendations": [
    {
      "id": "ms-vscode.cpptools",
      "version": "1.18.5"
    }
  ]
}
```

However, this is generally not recommended as it can cause compatibility issues.
