# Claude Code Instructions

See [AGENTS.md](AGENTS.md) for complete project guidelines.

## Claude-Specific Notes

### MCP Tools Available
This project has extensive MCP tool support (see `.cursor/mcp.json`):
- **Exarp**: Project automation (docs health, task alignment, duplicate detection)
- **Context7**: Up-to-date library documentation
- **GitKraken**: Git operations
- **Sequential/Tractatus Thinking**: Structured problem-solving

### Key Commands
```bash
# Build
ninja -C build

# Test
ctest --test-dir build --output-on-failure

# Lint
./scripts/run_linters.sh
```

### Code Style Summary
- C++20, 2-space indentation, Allman braces
- `snake_case` functions, `PascalCase` types, `k` prefix constants
- Add tests for all trading logic

### Documentation References
- **API Index**: `docs/API_DOCUMENTATION_INDEX.md`
- **Cursor Rules**: `.cursor/rules/` (glob-based auto-attach)
- **Architecture**: `docs/research/architecture/`

### When Working on This Project

1. **Before Implementation**: Check `.cursor/rules/` for context-specific guidelines
2. **For Complex Problems**: Use Tractatus Thinking (structure) → Sequential Thinking (process)
3. **Security**: Never commit credentials, use paper trading port (7497)
4. **Testing**: All trading calculations must have tests

For the authoritative source of project conventions, always refer to [AGENTS.md](AGENTS.md).
