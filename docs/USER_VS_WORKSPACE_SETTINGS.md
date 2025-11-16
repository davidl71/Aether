# User vs Workspace Settings Guide

**Date**: 2025-01-27

This guide explains which settings should be in **User Settings** (global, personal preferences) vs **Workspace Settings** (project-specific, shared with team).

---

## Current Setup

- **Workspace Settings**: `.vscode/settings.json` (committed to git)
- **User Settings**: Global Cursor/VS Code settings (not in repo)
- **User Override**: `.vscode/settings.json.user` (gitignored, optional)

---

## Settings That Should Move to User Settings

### 1. Editor Appearance (Personal Preferences)

These are personal preferences and should be in **User Settings**:

```json
{
  // Font preferences (personal)
  "editor.fontSize": 14,
  "editor.fontFamily": "Fira Code, 'Courier New', monospace",
  "editor.fontLigatures": true,
  "editor.lineHeight": 1.5,

  // Theme/colors (personal)
  "workbench.colorTheme": "Default Dark+",
  "workbench.iconTheme": "vs-seti",

  // UI preferences (personal)
  "workbench.settings.enableNaturalLanguageSearch": false,
  "workbench.colorCustomizations": {
    "[Default Dark+]": {
      "editorRuler.foreground": "#444444"
    }
  }
}
```

**Why User**: Font size, font family, and themes are personal preferences that vary by developer.

---

### 2. Accessibility & Audio (Personal Preferences)

```json
{
  "editor.accessibilitySupport": "off",
  "audioCues.enabled": "off",
  "accessibility.signals.terminalBell": {
    "sound": "off"
  },
  "terminal.integrated.enableVisualBell": false
}
```

**Why User**: Accessibility needs and audio preferences are personal.

---

### 3. Terminal Preferences (Personal)

```json
{
  // Terminal shell (personal preference)
  "terminal.integrated.defaultProfile.osx": "zsh",
  "terminal.integrated.fontSize": 12,
  "terminal.integrated.fontFamily": "Menlo, Monaco, 'Courier New', monospace"
}
```

**Why User**: Shell choice and terminal appearance are personal preferences.

**Note**: Keep `terminal.integrated.cwd` and `terminal.integrated.env.osx` in workspace (project-specific).

---

### 4. Git Preferences (Can Be User)

```json
{
  "git.ignoreLimitWarning": true,
  "git.autofetch": true,
  "git.confirmSync": false,
  "git.enableSmartCommit": true
}
```

**Why User**: Git workflow preferences can be personal, though some teams standardize.

---

### 5. Editor Behavior (Borderline - Usually User)

```json
{
  // These could be team standards, but often personal
  "editor.minimap.enabled": true,
  "editor.minimap.maxColumn": 120,
  "editor.smoothScrolling": true,
  "editor.cursorBlinking": "smooth",
  "editor.cursorSmoothCaretAnimation": true,
  "editor.wordWrap": "off",
  "editor.wordWrapColumn": 100
}
```

**Why User**: UI behavior preferences are usually personal, though some teams standardize.

---

## Settings That Should Stay in Workspace

### 1. Project-Specific Paths & Tools

```json
{
  // C++ compiler and paths (project-specific)
  "C_Cpp.default.compilerPath": "/usr/bin/clang++",
  "C_Cpp.default.intelliSenseMode": "macos-clang-arm64",
  "C_Cpp.default.includePath": [
    "${workspaceFolder}/native/include",
    "${workspaceFolder}/native/third_party/tws-api/..."
  ],

  // Python interpreter (project might need specific version)
  "python.defaultInterpreterPath": "/usr/local/bin/python3",

  // CMake settings (project-specific)
  "cmake.buildDirectory": "${workspaceFolder}/build",
  "cmake.sourceDirectory": "${workspaceFolder}/native"
}
```

**Why Workspace**: These are project-specific and must be consistent across team.

---

### 2. Code Style & Formatting (Team Standards)

```json
{
  // Code style (team standard)
  "editor.tabSize": 2,
  "editor.insertSpaces": true,
  "editor.rulers": [100],
  "editor.formatOnSave": true,

  // Formatters (team standard)
  "[cpp]": {
    "editor.defaultFormatter": "ms-vscode.cpptools"
  },
  "[python]": {
    "editor.defaultFormatter": "ms-python.black-formatter"
  }
}
```

**Why Workspace**: Code style should be consistent across team (though some teams use EditorConfig).

---

### 3. File Exclusions (Project-Specific)

```json
{
  "files.exclude": {
    "**/build": true,
    "native/third_party/tws-api/IBJts/samples": true,
    // ... project-specific exclusions
  },
  "search.exclude": {
    // ... project-specific search exclusions
  }
}
```

**Why Workspace**: These are project-specific build artifacts and vendor code.

---

### 4. Language-Specific Settings (Project Standards)

```json
{
  "C_Cpp.default.cppStandard": "c++20",
  "python.analysis.typeCheckingMode": "basic",
  "rust-analyzer.checkOnSave.command": "clippy"
}
```

**Why Workspace**: Language standards and tooling should be consistent.

---

### 5. Terminal Environment (Project-Specific)

```json
{
  "terminal.integrated.cwd": "${workspaceFolder}",
  "terminal.integrated.env.osx": {
    "PYTHONPATH": "${workspaceFolder}/python"
  }
}
```

**Why Workspace**: Project-specific environment variables.

---

## Recommended User Settings Template

Create these in **User Settings** (`Cmd+,` → click "Open User Settings (JSON)"):

```json
{
  // ==========================================
  // Editor Appearance (Personal)
  // ==========================================
  "editor.fontSize": 14,
  "editor.fontFamily": "Fira Code, 'Courier New', monospace",
  "editor.fontLigatures": true,
  "editor.lineHeight": 1.5,

  // ==========================================
  // Theme & Colors (Personal)
  // ==========================================
  "workbench.colorTheme": "Default Dark+",
  "workbench.iconTheme": "vs-seti",

  // ==========================================
  // Accessibility & Audio (Personal)
  // ==========================================
  "editor.accessibilitySupport": "off",
  "audioCues.enabled": "off",
  "accessibility.signals.terminalBell": {
    "sound": "off"
  },
  "terminal.integrated.enableVisualBell": false,

  // ==========================================
  // Terminal (Personal)
  // ==========================================
  "terminal.integrated.defaultProfile.osx": "zsh",
  "terminal.integrated.fontSize": 12,
  "terminal.integrated.fontFamily": "Menlo, Monaco, monospace",

  // ==========================================
  // Git (Personal Preferences)
  // ==========================================
  "git.ignoreLimitWarning": true,
  "git.autofetch": true,

  // ==========================================
  // Editor Behavior (Personal)
  // ==========================================
  "editor.minimap.enabled": true,
  "editor.smoothScrolling": true,
  "editor.cursorBlinking": "smooth",
  "editor.wordWrap": "off",

  // ==========================================
  // Workbench (Personal)
  // ==========================================
  "workbench.settings.enableNaturalLanguageSearch": false,

  // ==========================================
  // Files (Personal)
  // ==========================================
  "files.encoding": "utf8",
  "files.eol": "\n"
}
```

---

## Alternative: Workspace User Override File

If you want project-specific user overrides, create `.vscode/settings.json.user` (gitignored):

```json
{
  // Personal overrides for this project only
  "editor.fontSize": 16,  // Larger font for this project
  "editor.fontFamily": "JetBrains Mono"
}
```

**Note**: This file is gitignored and only affects this workspace.

---

## Migration Checklist

### Move to User Settings:
- [ ] `editor.fontSize`
- [ ] `editor.fontFamily`
- [ ] `editor.fontLigatures`
- [ ] `editor.lineHeight`
- [ ] `workbench.colorTheme`
- [ ] `workbench.iconTheme`
- [ ] `workbench.colorCustomizations`
- [ ] `editor.accessibilitySupport`
- [ ] `audioCues.enabled`
- [ ] `accessibility.signals.terminalBell`
- [ ] `terminal.integrated.enableVisualBell`
- [ ] `terminal.integrated.defaultProfile.osx`
- [ ] `terminal.integrated.fontSize`
- [ ] `terminal.integrated.fontFamily`
- [ ] `git.ignoreLimitWarning` (optional)
- [ ] `git.autofetch` (optional)
- [ ] `workbench.settings.enableNaturalLanguageSearch`

### Keep in Workspace:
- [ ] All C++ compiler/path settings
- [ ] CMake settings
- [ ] File exclusions
- [ ] Code style (tab size, rulers)
- [ ] Formatter assignments
- [ ] Language-specific settings
- [ ] Terminal environment variables
- [ ] Project-specific include paths

---

## Benefits of Separation

1. **Consistency**: Team shares project-specific settings
2. **Personalization**: Developers can customize appearance
3. **Portability**: User settings follow you across projects
4. **Clarity**: Clear separation of concerns

---

## How to Access User Settings

1. **Via UI**: `Cmd+,` → Click "Open User Settings (JSON)" icon
2. **Via Command**: `Cmd+Shift+P` → "Preferences: Open User Settings (JSON)"
3. **Via File**: `~/Library/Application Support/Cursor/User/settings.json` (macOS)

---

## Related Documentation

- [Cursor Setup Guide](CURSOR_SETUP.md)
- [VS Code Settings Documentation](https://code.visualstudio.com/docs/getstarted/settings)
