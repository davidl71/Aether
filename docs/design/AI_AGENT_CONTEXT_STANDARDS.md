# AI Agent Context Standards Design

**Date**: 2025-11-27
**Purpose**: Define standard resources and files that AI agents (Cursor, ChatGPT, Gemini, GitHub Copilot, Claude Code) automatically look for during context priming.

---

## Overview

This document establishes design goals for providing standardized context files and resources that AI coding assistants discover automatically. The goal is cross-tool compatibility while maintaining our comprehensive Cursor-specific configuration.

---

## Design Principles

1. **Cross-Tool Compatibility**: Support multiple AI tools (Cursor, Copilot, Claude Code, etc.)
2. **Single Source of Truth**: Avoid duplication by using symlinks or includes where possible
3. **Progressive Context**: Provide layered context from high-level to detailed
4. **Automatic Discovery**: Use standard file locations that tools find automatically
5. **Minimal Maintenance**: New files should reference existing docs, not duplicate

---

## Standard File Conventions by Tool

### Universal (All Tools)

| File | Purpose | Status |
|------|---------|--------|
| `README.md` | Project overview | ✅ Exists |
| `CONTRIBUTING.md` | Contribution guidelines | ❌ Missing |
| `ARCHITECTURE.md` | System architecture | ⚠️ Check existence |
| `AGENTS.md` | AI agent instructions | ✅ Exists |
| `CHANGELOG.md` | Version history | ✅ Exists |

### Cursor-Specific

| File/Directory | Purpose | Status |
|----------------|---------|--------|
| `.cursor/rules/` | Glob-based rules (`.mdc` files) | ✅ Comprehensive |
| `.cursor/mcp.json` | MCP server configuration | ✅ Exists |
| `.cursor/commands.json` | Custom commands | ✅ Exists |
| `.cursor/global-docs.json` | Documentation paths | ✅ Exists |

### GitHub Copilot

| File | Purpose | Status |
|------|---------|--------|
| `.github/copilot-instructions.md` | Repository-level instructions | ❌ Missing |

### Claude Code

| File | Purpose | Status |
|------|---------|--------|
| `CLAUDE.md` | Claude-specific instructions | ❌ Missing |

### Windsurf/Codeium

| File | Purpose | Status |
|------|---------|--------|
| `.windsurfrules` | Windsurf-specific rules | ❌ Missing (low priority) |

### Aider

| File | Purpose | Status |
|------|---------|--------|
| `.aider.conf.yml` | Aider configuration | ❌ Missing (low priority) |

### Emerging Standards

| File | Purpose | Status |
|------|---------|--------|
| `llms.txt` | AI crawler discovery (like robots.txt) | ❌ Missing |
| `llms-full.txt` | Full context for LLMs | ❌ Missing |

---

## Implementation Strategy

### Phase 1: High-Priority Files (Immediate)

1. **`.github/copilot-instructions.md`**
   - Reference `AGENTS.md` for full guidelines
   - Keep minimal to avoid duplication

2. **`CLAUDE.md`**
   - Reference `AGENTS.md` for full guidelines
   - Add Claude-specific behavioral hints

3. **`CONTRIBUTING.md`**
   - Extract from existing documentation
   - Standard contribution workflow

### Phase 2: Cross-Tool Compatibility (Short-term)

4. **`ARCHITECTURE.md`** (if missing)
   - High-level system architecture
   - Reference detailed docs in `docs/`

5. **Validate existing files**
   - Ensure `README.md` provides good context
   - Ensure `AGENTS.md` is comprehensive

### Phase 3: Emerging Standards (Future)

6. **`llms.txt`** and **`llms-full.txt`**
   - For web-facing documentation
   - AI crawler discovery protocol

7. **`.windsurfrules`** and **`.aider.conf.yml`**
   - Only if users request these tools
   - Can be generated from `.cursor/rules/`

---

## File Content Guidelines

### Principle: Reference, Don't Duplicate

All cross-tool files should **reference** `AGENTS.md` as the source of truth:

```markdown
# Example: CLAUDE.md

See [AGENTS.md](AGENTS.md) for complete project guidelines.

## Claude-Specific Notes
- Use MCP tools for project automation (see .cursor/mcp.json)
- Follow coding style in AGENTS.md
```

### Principle: Keep Tool-Specific Files Minimal

```markdown
# Example: .github/copilot-instructions.md

This repository follows the guidelines in AGENTS.md.
Key points for Copilot:
- C++20 standard, 2-space indentation
- snake_case for functions, PascalCase for types
- See docs/API_DOCUMENTATION_INDEX.md for APIs
```

---

## MCP Resources for Context Priming

The following MCP resources should be available for AI agents:

### Essential Resources

1. **Project Layout** - Directory structure overview
2. **Build Commands** - How to build/test the project
3. **Current Tasks** - Active todo items
4. **Git Status** - Recent changes
5. **Architecture Summary** - System overview

### Implementation via Exarp MCP Server

Add resources to project-management-automation MCP server:
- `project://context/summary` - High-level project context
- `project://context/architecture` - Architecture overview
- `project://context/conventions` - Coding conventions

---

## Success Criteria

1. ✅ All major AI tools can find project context automatically
2. ✅ No duplication of guidelines across files
3. ✅ Single source of truth in `AGENTS.md`
4. ✅ Easy maintenance when guidelines change
5. ✅ Clear documentation of what each file provides

---

## Related Documents

- `AGENTS.md` - Primary AI agent instructions
- `.cursor/rules/` - Cursor-specific rules
- `docs/AUTOMATION_TOOL_PROMPTING_DESIGN.md` - Tool suggestion system
- `docs/MCP_QUICK_REFERENCE.md` - MCP server configuration

---

## Tasks

See Todo2 tasks with tag `ai-context-standards`:
- T-AI-CONTEXT-1: Create .github/copilot-instructions.md
- T-AI-CONTEXT-2: Create CLAUDE.md
- T-AI-CONTEXT-3: Create CONTRIBUTING.md
- T-AI-CONTEXT-4: Validate/create ARCHITECTURE.md
- T-AI-CONTEXT-5: Add MCP context resources

---

**Last Updated**: 2025-11-27
**Status**: Design Document
