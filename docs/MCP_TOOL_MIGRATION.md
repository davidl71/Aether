# MCP Tool Migration Guide

**Date:** 2025-01-27
**Purpose:** Map deprecated/redundant tools from other MCP servers to our project-specific replacements

---

## Overview

Our `project-management-automation` MCP server provides enhanced, project-specific versions of common automation tools. This guide helps identify when to use our tools instead of generic MCP server tools.

---

## Tool Migration Map

| Generic Tool (Other Server) | Our Replacement | Why Use Ours |
|------------------------------|-----------------|--------------|
| Generic documentation analysis | `check_documentation_health_tool` | Project-specific with Todo2 integration, historical tracking |
| Generic task alignment analysis | `analyze_todo2_alignment_tool` | Todo2-aware, investment strategy framework alignment |
| Generic duplicate detection | `detect_duplicate_tasks_tool` | Todo2-specific, configurable similarity, auto-fix |
| Generic security scanning | `scan_dependency_security_tool` | Multi-language (Python/Rust/npm), project-configured |
| Generic automation discovery | `find_automation_opportunities_tool` | Project-aware, value-scored opportunities |
| Generic TODO sync | `sync_todo_tasks_tool` | Bidirectional Todo2 ↔ Shared TODO sync |
| Generic PWA review | `review_pwa_config_tool` | Project-specific PWA analysis with goal alignment |

---

## Detailed Tool Comparisons

### Documentation Health Check

**Generic Tools (e.g., `filesystem` server):**

- Basic file existence checks
- Simple link validation
- No project context

**Our Tool: `check_documentation_health_tool`**

- ✅ Project-aware link validation
- ✅ Todo2 task creation for issues
- ✅ Cross-reference graph analysis
- ✅ Historical trend tracking
- ✅ Format compliance checking
- ✅ Date currency validation

**When to Use:** Always prefer our tool for project documentation analysis.

---

### Task Alignment Analysis

**Generic Tools (e.g., `agentic-tools` server):**

- Generic task analysis
- No project context
- No strategy alignment

**Our Tool: `analyze_todo2_alignment_tool`**

- ✅ Todo2-specific analysis
- ✅ Investment strategy framework alignment
- ✅ Phase-based categorization
- ✅ Goal alignment scoring
- ✅ Actionable recommendations

**When to Use:** Always prefer our tool for Todo2 task alignment.

---

### Duplicate Detection

**Generic Tools:**

- Basic similarity matching
- No context awareness
- No auto-fix

**Our Tool: `detect_duplicate_tasks_tool`**

- ✅ Todo2-aware duplicate detection
- ✅ Configurable similarity threshold
- ✅ Auto-fix capability
- ✅ Dependency-aware consolidation
- ✅ Self-dependency detection

**When to Use:** Always prefer our tool for Todo2 duplicate detection.

---

### Security Scanning

**Generic Tools (e.g., `semgrep` server):**

- Single-language focus
- Basic vulnerability detection
- No trend tracking

**Our Tool: `scan_dependency_security_tool`**

- ✅ Multi-language support (Python, Rust, npm)
- ✅ Multiple scanning tools (osv-scanner, pip-audit, cargo-audit, npm audit)
- ✅ Severity-based prioritization
- ✅ Historical trend tracking
- ✅ Project-configured scanning

**When to Use:** Prefer our tool for comprehensive multi-language security scanning.

---

## Server-Level Guidance

### When to Use Our MCP Server

Use `project-management-automation` server tools when:

- ✅ You need project-specific analysis
- ✅ You want Todo2 integration
- ✅ You need historical tracking
- ✅ You want project-configured behavior

### When to Use Other Servers

Continue using other MCP servers for:

- ✅ General file operations (`filesystem`)
- ✅ Git operations (`git`)
- ✅ Task management CRUD (`agentic-tools`)
- ✅ Security code scanning (`semgrep`)
- ✅ Documentation lookup (`context7`)

---

## AI Assistant Guidance

When AI assistants see both:

- Generic tool: `analyze_tasks` (from `agentic-tools`)
- Our tool: `analyze_todo2_alignment_tool` (from `project-management-automation`)

**Prefer:** `analyze_todo2_alignment_tool` for project-specific Todo2 analysis.

**Reasoning:** Our tool provides project-aware analysis with strategy framework alignment, which is more valuable for this specific project.

---

## Migration Checklist

- [ ] Review tool descriptions in our MCP server
- [ ] Identify overlapping functionality
- [ ] Update AI assistant prompts to prefer our tools
- [ ] Document any remaining generic tool usage
- [ ] Monitor tool usage patterns

---

**Last Updated:** 2025-01-27
