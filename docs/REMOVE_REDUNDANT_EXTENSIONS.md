# Remove Redundant Extensions - Quick Guide

## Usage

Run the script:
```bash
./scripts/remove_redundant_extensions.sh
```

## What It Does

The script identifies and removes redundant extensions based on the redundancy analysis:

### High Priority Removals (8 extensions)
- `rust-lang.rust` - Legacy, redundant with rust-analyzer
- `syntaxsyndicate.turbo-vsc` - Redundant with vercel.turbo-vsc
- `ms-azuretools.vscode-containers` - Disabled, not used
- `ms-azuretools.vscode-docker` - Disabled, not used
- `ms-kubernetes-tools.vscode-kubernetes-tools` - Disabled, not used
- `golang.go` - Disabled, Go not used in project
- `neonxp.gotools` - Disabled, Go not used
- `shivamkumar.go-extras` - Disabled, Go not used

### Medium Priority Removals (7 extensions)
- `bbenoist.doxygen` - Redundant with cschlosser.doxdocgen
- `pinage404.rust-extension-pack` - Extension pack (may include individual tools)
- `franneck94.vscode-c-cpp-dev-extension-pack` - Extension pack
- `llvm-vs-code-extensions.vscode-clangd` - Redundant with cpptools
- `franneck94.c-cpp-runner` - Optional C++ runner
- `franneck94.vscode-c-cpp-config` - Optional config helper
- `jbenden.c-cpp-flylint` - Optional linter

### Manual Review Recommended

**AI Assistants (6 extensions)** - Keep 1-2 favorites:
- `anthropic.claude-code`
- `amazonwebservices.amazon-q-vscode`
- `google.gemini-cli-vscode-ide-companion`
- `google.geminicodeassist`
- `openai.chatgpt`
- `fridaplatform.fridagpt`

**MCP Extensions (6 extensions)** - Keep 1-2 you use:
- `cjl.lsp-mcp`
- `daninemonic.mcp4humans`
- `interactive-mcp.interactive-mcp`
- `kirigaya.openmcp`
- `pimzino.agentic-tools-mcp-companion`
- `raz-labs.interactive-mcp`

## Options

When you run the script, you'll see:

1. **Remove HIGH priority only** - Removes 8 high-priority redundant extensions
2. **Remove HIGH + MEDIUM priority** - Removes all 15 redundant extensions
3. **Custom selection** - Interactive mode to pick specific extensions
4. **Show details only** - Preview what would be removed (no changes)
5. **Exit** - Cancel without changes

## Safety Features

- ✅ Shows what will be removed before doing it
- ✅ Interactive confirmation
- ✅ Handles errors gracefully
- ✅ Shows summary after removal
- ✅ Doesn't remove AI/MCP extensions automatically (manual review)

## Expected Results

After running with option 2 (HIGH + MEDIUM):
- **Before**: ~94 extensions
- **After**: ~79 extensions
- **Removed**: ~15 extensions

## Verification

After running the script, verify with:
```bash
./scripts/check_extension_redundancy.sh
./scripts/analyze_all_extensions.sh
```

## Notes

- The script won't remove `ms-vscode.cpptools` if it's not installed (you may only have `anysphere.cpptools`)
- Extension packs are removed if you have individual tools installed
- AI and MCP extensions are shown but not auto-removed (you decide which to keep)

