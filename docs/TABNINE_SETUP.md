# Tabnine Setup and Configuration Guide

**Date:** November 18, 2025
**Status:** ✅ Installed and Configured
**Version:** Tabnine v3.324.0

---

## Installation Status

✅ **Tabnine extension has been successfully installed** in Cursor IDE.

- **Extension ID:** `TabNine.tabnine-vscode`
- **Version:** 3.324.0
- **Location:** Added to `.vscode/extensions.json` as workspace recommendation

---

## Quick Start

### 1. Activate Tabnine

After installation, Tabnine should be active. To verify:

1. **Check Status Bar**
   - Look for the Tabnine logo in the bottom status bar
   - Click it to open the Tabnine Hub

2. **Sign In/Register** (if needed)
   - Click the Tabnine logo in status bar
   - Click "Register" or "Sign In" in the Tabnine Hub
   - Create a free account or sign in with existing credentials

3. **Start Coding**
   - Tabnine will automatically provide suggestions as you type
   - Press `Tab` to accept suggestions
   - Press `Esc` to dismiss suggestions

---

## Configuration

### Current Settings (`.vscode/settings.json`)

The following Tabnine settings have been configured for this project:

```json
{
  // Tabnine Settings (AI Code Completion)
  "tabnine.enableAutoImport": true,
  "tabnine.enableDeepCompletions": true,
  "tabnine.enableInlinePredictions": true,
  "tabnine.enableForLanguages": [
    "cpp", "c", "python", "typescript", "typescriptreact",
    "javascript", "javascriptreact", "rust", "go"
  ],
  "tabnine.cloudModelEnabled": false,  // Prefer local inference
  "tabnine.codePrivacyEnabled": true,  // Keep code private
  "tabnine.maxNumResults": 5,
  "tabnine.debounceMs": 200
}
```

### Key Configuration Decisions

#### ✅ Privacy-First Settings

- **`tabnine.codePrivacyEnabled: true`** - Ensures your trading algorithms stay private
- **`tabnine.cloudModelEnabled: false`** - Prefers on-device inference when available
- **Important:** For proprietary trading code, these settings help protect your intellectual property

#### ✅ Language Support

Enabled for all project languages:

- **C++20** (`cpp`, `c`) - Core trading logic
- **Python** - TUI, integration services, bindings
- **TypeScript/React** - Web dashboard
- **Rust** - Backend services
- **Go** - Multi-language agents

#### ✅ Performance Optimization

- **`tabnine.maxNumResults: 5`** - Limits suggestions to reduce resource usage
- **`tabnine.debounceMs: 200`** - Reduces completion frequency for better performance

---

## Usage

### Basic Usage

1. **Start Typing**
   - Tabnine provides suggestions automatically
   - Suggestions appear inline as you type

2. **Accept Suggestions**
   - Press `Tab` to accept the suggestion
   - Press `→` (right arrow) to accept word-by-word
   - Click the suggestion to accept

3. **Dismiss Suggestions**
   - Press `Esc` to dismiss
   - Continue typing to ignore

### Advanced Features

#### Tabnine Chat

1. **Open Tabnine Hub**
   - Click Tabnine logo in status bar
   - Or use Command Palette: `Tabnine: Open Tabnine Hub`

2. **Use Chat**
   - Ask questions about your code
   - Get explanations and suggestions
   - Request code generation

#### Deep Completions

- Tabnine analyzes your entire codebase for context
- Provides multi-line completions
- Understands project patterns and conventions

---

## Integration with Cursor AI

### Complementary Tools

**Tabnine** and **Cursor AI** work well together:

| Feature | Tabnine | Cursor AI |
|---------|---------|-----------|
| **Primary Use** | Real-time autocomplete | Code generation, refactoring, chat |
| **When to Use** | While typing code | Planning, debugging, explanations |
| **Speed** | Instant suggestions | Thoughtful responses |
| **Context** | Local codebase | Full project understanding |

### Best Practices

1. **Use Tabnine for:**
   - Quick code completions while typing
   - Boilerplate code generation
   - Common patterns and idioms

2. **Use Cursor AI for:**
   - Complex refactoring tasks
   - Debugging and problem-solving
   - Architecture decisions
   - Code explanations

3. **Avoid Conflicts:**
   - Both tools may suggest different approaches
   - Choose the suggestion that fits your coding style
   - Disable Tabnine for specific files if needed (see below)

---

## Customization

### Disable Tabnine for Specific Files

If Tabnine conflicts with Cursor AI or other tools, you can disable it:

**Option 1: Per-File (via settings.json.user)**

```json
{
  "[cpp]": {
    "tabnine.enableInlinePredictions": false
  }
}
```

**Option 2: Per-Workspace**

- Open Tabnine Hub
- Go to Settings
- Disable for specific languages

### Adjust Suggestion Frequency

If suggestions are too frequent or distracting:

```json
{
  "tabnine.debounceMs": 500,  // Increase delay (default: 200ms)
  "tabnine.maxNumResults": 3  // Reduce suggestions (default: 5)
}
```

### Enable Cloud Model (if needed)

For better suggestions (requires internet, less privacy):

```json
{
  "tabnine.cloudModelEnabled": true
}
```

**⚠️ Warning:** This sends code to Tabnine servers. Not recommended for proprietary trading algorithms.

---

## Privacy and Security

### For Trading Software

This project uses **privacy-first settings** to protect proprietary trading algorithms:

✅ **Enabled:**

- `tabnine.codePrivacyEnabled: true` - Code stays local
- `tabnine.cloudModelEnabled: false` - Prefer local inference

✅ **Benefits:**

- Trading algorithms remain private
- No code sent to external servers
- Compliance with trading regulations

### Enterprise Features

For teams, Tabnine offers:

- **Private model training** on your codebase
- **Team-wide consistency** in suggestions
- **Enhanced privacy controls**

Contact Tabnine for enterprise pricing if needed.

---

## Troubleshooting

### Tabnine Not Showing Suggestions

1. **Check Installation**

   ```bash
   code --list-extensions | grep -i tabnine
   ```

   Should show: `TabNine.tabnine-vscode`

2. **Reload Window**
   - `Cmd+Shift+P` → "Developer: Reload Window"

3. **Check Status Bar**
   - Look for Tabnine logo
   - Click to open Tabnine Hub
   - Verify account is signed in

4. **Check Settings**
   - Verify language is in `tabnine.enableForLanguages`
   - Check that `tabnine.enableInlinePredictions` is `true`

### High Resource Usage

If Tabnine uses too much CPU/RAM:

1. **Reduce Suggestions**

   ```json
   {
     "tabnine.maxNumResults": 3,
     "tabnine.debounceMs": 500
   }
   ```

2. **Disable for Large Files**
   - Tabnine may struggle with very large files (>10,000 lines)
   - Consider disabling for specific file types

3. **Disable Deep Completions**

   ```json
   {
     "tabnine.enableDeepCompletions": false
   }
   ```

### Conflicts with Cursor AI

If both tools suggest different code:

1. **Choose Based on Context**
   - Tabnine: Quick completions
   - Cursor AI: Complex logic

2. **Disable Tabnine Temporarily**
   - Use Tabnine Hub to disable for current session
   - Or disable for specific file types

3. **Use Both Strategically**
   - Tabnine for typing speed
   - Cursor AI for code quality

---

## Performance Monitoring

### System Requirements

Tabnine requires:

- **Minimum:** 16 GB RAM, 8 CPU cores, 100 GB storage
- **Recommended:** 32 GB RAM, 12+ CPU cores for on-device inference

### Monitor Resource Usage

1. **Activity Monitor** (macOS)
   - Check CPU usage for Tabnine processes
   - Monitor memory usage

2. **Tabnine Hub**
   - View performance metrics
   - Check suggestion quality

3. **VS Code Output**
   - `View` → `Output` → Select "Tabnine"
   - View logs and diagnostics

---

## Testing Tabnine

### Test with Project Languages

1. **C++ Test** (`native/src/`)

   ```cpp
   // Start typing a function
   void calculate_box_spread(
   // Tabnine should suggest parameter types and names
   ```

2. **Python Test** (`python/`)

   ```python
   # Start typing
   def fetch_option_chain(
   # Tabnine should suggest based on project patterns
   ```

3. **TypeScript Test** (`web/src/`)

   ```typescript
   // Start typing
   const useBoxSpreadData = (
   # Tabnine should suggest React hook patterns
   ```

4. **Rust Test** (`agents/backend/`)

   ```rust
   // Start typing
   pub fn process_market_data(
   # Tabnine should suggest Rust patterns
   ```

---

## Next Steps

1. **✅ Installation Complete** - Tabnine is installed and configured
2. **Sign In** - Create account or sign in via Tabnine Hub
3. **Test** - Try coding in C++, Python, TypeScript, or Rust
4. **Customize** - Adjust settings based on your preferences
5. **Monitor** - Watch resource usage and adjust if needed

---

## References

- [Tabnine Documentation](https://docs.tabnine.com/)
- [Tabnine Privacy Policy](https://www.tabnine.com/privacy)
- [Tabnine System Requirements](https://docs.tabnine.com/main/welcome/readme/system-requirements)
- [Tabnine Settings Reference](https://docs.tabnine.com/main/reference/vscode)

---

## Support

- **Tabnine Support:** [support@tabnine.com](mailto:support@tabnine.com)
- **Tabnine Community:** [Community Forum](https://www.tabnine.com/community)
- **Project Issues:** See project README for issue reporting

---

**Setup Complete** ✅

Tabnine is now ready to help accelerate your development workflow!
