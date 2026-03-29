# Extension Audit Instructions

You reported having 99 installed extensions. The CLI tool may not show all of them.

## To Check All 99 Extensions

1. Open VS Code Extensions view: `Cmd+Shift+X`
2. Filter by "Installed" to see all extensions
3. Review each extension against these categories:

### Should Be Workspace-Only

- Language extensions (C++, Python, Rust, TypeScript, Swift, etc.)
- Project-specific tools (MCP, project-specific formatters)

### Should Be Disabled/Uninstalled

- Unused language extensions
- Enterprise/Mainframe extensions (IBM i, COBOL, etc.)
- See `.vscode/extensions.json` → `unwantedRecommendations`

### Can Stay Global

- Universal tools (EditorConfig, GitLens, Markdown, Spell Check)
- AI tools (Copilot, CodeWhisperer) - though workspace configs may help

## Quick Actions

1. Manually review in VS Code/Cursor Extensions view (there is no bundled repo script to list all IDE extensions).
2. See `docs/EXTENSION_AUDIT_GUIDE.md` for detailed instructions
