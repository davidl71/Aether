# MCP Prompts - Project Management Automation

**Status:** ✅ 14 Prompts Available

This server provides reusable prompt templates that guide users through common project management workflows.

---

## 📋 Available Prompts

### Documentation Management

#### `doc_health_check`
**Description:** Analyze documentation health and create tasks for issues

**Use Case:** Before releases, after major changes, or during regular maintenance

**Tool:** `check_documentation_health_tool(create_tasks=True)`

---

#### `doc_quick_check`
**Description:** Quick documentation health check without creating tasks

**Use Case:** Quick assessment before committing changes

**Tool:** `check_documentation_health_tool(create_tasks=False)`

---

### Task Management

#### `task_alignment`
**Description:** Analyze Todo2 task alignment with project goals

**Use Case:** Before starting new work, during sprint planning

**Tool:** `analyze_todo2_alignment_tool(create_followup_tasks=True)`

---

#### `duplicate_cleanup`
**Description:** Find and consolidate duplicate Todo2 tasks

**Use Case:** Regular cleanup, before starting new work

**Tool:** `detect_duplicate_tasks_tool(similarity_threshold=0.85, auto_fix=False)`

---

#### `task_sync`
**Description:** Synchronize tasks between shared TODO table and Todo2

**Use Case:** Keep task systems in sync, after bulk updates

**Tool:** `sync_todo_tasks_tool(dry_run=True)` (preview) or `sync_todo_tasks_tool(dry_run=False)` (apply)

---

### Security & Dependencies

#### `security_scan_all`
**Description:** Scan all project dependencies for security vulnerabilities

**Use Case:** Before releases, after dependency updates, regular security audits

**Tool:** `scan_dependency_security_tool(languages=None)`

---

#### `security_scan_python`
**Description:** Scan Python dependencies for security vulnerabilities

**Use Case:** Python-specific security checks

**Tool:** `scan_dependency_security_tool(languages=['python'])`

---

#### `security_scan_rust`
**Description:** Scan Rust dependencies for security vulnerabilities

**Use Case:** Rust-specific security checks

**Tool:** `scan_dependency_security_tool(languages=['rust'])`

---

### Automation Discovery

#### `automation_discovery`
**Description:** Discover new automation opportunities in the codebase

**Use Case:** During refactoring, after completing features, regular optimization

**Tool:** `find_automation_opportunities_tool(min_value_score=0.7)`

---

#### `automation_high_value`
**Description:** Find only high-value automation opportunities (score >= 0.8)

**Use Case:** Focus on highest-impact automations

**Tool:** `find_automation_opportunities_tool(min_value_score=0.8)`

---

### PWA Configuration

#### `pwa_review`
**Description:** Review PWA configuration and generate improvement recommendations

**Use Case:** Before PWA deployment, after major updates

**Tool:** `review_pwa_config_tool()`

---

### Workflow Prompts

#### `pre_sprint_cleanup`
**Description:** Pre-sprint cleanup workflow

**Workflow:**
1. `detect_duplicate_tasks_tool` - Find and consolidate duplicates
2. `analyze_todo2_alignment_tool` - Check task alignment
3. `check_documentation_health_tool` - Ensure docs are up to date

**Use Case:** Before starting a new sprint or iteration

---

#### `post_implementation_review`
**Description:** Post-implementation review workflow

**Workflow:**
1. `check_documentation_health_tool` - Update documentation
2. `scan_dependency_security_tool` - Check for new vulnerabilities
3. `find_automation_opportunities_tool` - Discover new automation needs

**Use Case:** After completing a feature or major change

---

#### `weekly_maintenance`
**Description:** Weekly maintenance workflow

**Workflow:**
1. `check_documentation_health_tool` - Keep docs healthy
2. `detect_duplicate_tasks_tool` - Clean up duplicates
3. `scan_dependency_security_tool` - Check security
4. `sync_todo_tasks_tool` - Sync across systems

**Use Case:** Regular weekly maintenance routine

---

## 🚀 Usage

### In Cursor

Prompts are available in Cursor's MCP prompt menu. Select a prompt to:
1. Get structured guidance for the workflow
2. See which tools to use
3. Understand the recommended parameters

### Example

1. Open Cursor's prompt menu
2. Select "Pre-Sprint Cleanup"
3. Follow the workflow steps
4. Use the recommended tools with suggested parameters

---

## 📊 Prompt Categories

| Category | Count | Prompts |
|----------|-------|---------|
| Documentation | 2 | `doc_health_check`, `doc_quick_check` |
| Task Management | 3 | `task_alignment`, `duplicate_cleanup`, `task_sync` |
| Security | 3 | `security_scan_all`, `security_scan_python`, `security_scan_rust` |
| Automation | 2 | `automation_discovery`, `automation_high_value` |
| PWA | 1 | `pwa_review` |
| Workflows | 3 | `pre_sprint_cleanup`, `post_implementation_review`, `weekly_maintenance` |

**Total: 14 prompts**

---

## 🔄 Integration with Tools

All prompts are designed to work seamlessly with the project management automation tools:

- ✅ **Tool-specific prompts** guide users to the right tool with correct parameters
- ✅ **Workflow prompts** sequence multiple tools for complete workflows
- ✅ **Context-aware** prompts provide use case guidance

---

**Last Updated:** 2025-11-23
