# Automation Tool Prompting Design

**Date**: 2025-11-23
**Purpose**: Design system for automatically prompting relevant project automation tools based on project context, task lifecycle, and user activity.

---

## Overview

This document outlines multiple approaches for automatically suggesting project automation tools to AI assistants and users based on context. The goal is to make automation tools more discoverable and contextually relevant without being intrusive.

---

## Design Principles

1. **Non-Intrusive**: Suggest tools, don't force them
2. **Context-Aware**: Only suggest relevant tools for current context
3. **Actionable**: Clear guidance on when and why to use tools
4. **Integrated**: Fits naturally into existing workflows
5. **Progressive**: More specific suggestions as context becomes clearer

---

## Approach 1: Cursor Rules with Context Triggers

### Concept

Add Cursor rules that detect context and suggest relevant automation tools.

### Implementation

Create `.cursor/rules/automation-tool-suggestions.mdc` with context-aware rules:

```markdown

# Automation Tool Suggestions

## Context Detection Rules

### When Documentation Changes
**Trigger**: Files in `docs/` directory are modified
**Suggest**: `check_documentation_health_tool`
**Reason**: "Documentation was modified - consider checking documentation health"

### When Task Status Changes to "Review"
**Trigger**: Todo2 task moves to "Review" status
**Suggest**: `analyze_todo2_alignment_tool` (if task is high priority)
**Reason**: "Task in review - verify alignment with project goals"

### When Multiple Tasks Created
**Trigger**: 5+ tasks created in single session
**Suggest**: `detect_duplicate_tasks_tool`
**Reason**: "Multiple tasks created - check for duplicates"

### When Dependencies Complete
**Trigger**: Task dependencies are marked "Done"
**Suggest**: Check if blocked tasks are now ready
**Reason**: "Dependencies completed - review blocked tasks"

### Daily Maintenance Reminder
**Trigger**: First interaction of the day (time-based)
**Suggest**: `run_daily_automation_tool`
**Reason**: "Daily maintenance tasks available"
```

### Pros

- ✅ Simple to implement
- ✅ Works with existing Cursor rules system
- ✅ No code changes required

### Cons

- ❌ Limited context detection (file changes, not semantic)
- ❌ Static rules (don't adapt to project state)

---

## Approach 2: Task Lifecycle Integration

### Concept

Integrate tool suggestions into Todo2 task lifecycle states.

### Implementation

Extend Todo2 workflow rules to suggest tools at lifecycle transitions:

```markdown

## Task Lifecycle Tool Suggestions

### Todo → In Progress
**When**: Task moves to "In Progress"
**Suggest**:

- If task mentions documentation: `check_documentation_health_tool`
- If task is high priority: `analyze_todo2_alignment_tool`
- If task has many dependencies: Check dependency health

### In Progress → Review
**When**: Task moves to "Review"
**Suggest**:

- `analyze_todo2_alignment_tool` (verify alignment)
- Check for duplicate tasks
- Review documentation if task involved docs

### Review → Done
**When**: Task moves to "Done"

**Suggest**:

- `detect_duplicate_tasks_tool` (check for similar completed tasks)
- `run_daily_automation_tool` (if multiple tasks completed)
```

### Pros

- ✅ Natural workflow integration
- ✅ Context-aware (based on task state)
- ✅ Actionable (suggestions at decision points)

### Cons

- ❌ Requires Todo2 workflow awareness
- ❌ May be too frequent if not filtered

---

## Approach 3: File Change Detection

### Concept

Monitor file changes and suggest relevant tools.

### Implementation

Add rules that detect file patterns and suggest tools:

```markdown

## File Change Detection

### Documentation Files
**Pattern**: `docs/**/*.md` modified
**Suggest**: `check_documentation_health_tool`
**Frequency**: Once per session (rate limit)

### Dependency Files
**Pattern**: `requirements.txt`, `Cargo.toml`, `package.json` modified
**Suggest**: `scan_dependency_security_tool`
**Frequency**: Once per change


### Configuration Files
**Pattern**: `*.config.json`, `CMakeLists.txt` modified
**Suggest**: Review configuration (no specific tool, but suggest review)

### Multiple Files Changed

**Pattern**: 10+ files modified in single session
**Suggest**: `run_daily_automation_tool` (comprehensive check)
```

### Pros

- ✅ Direct correlation with user activity
- ✅ Immediate relevance
- ✅ Easy to implement

### Cons

- ❌ May be too frequent
- ❌ Requires rate limiting
- ❌ File patterns may be too broad

---

## Approach 4: Prompt Templates with Tool Suggestions

### Concept

Create prompt templates that include tool suggestions based on query type.

### Implementation

Create templates for common query patterns:

```markdown

## Prompt Templates

### Documentation Queries
**Template**: "When user asks about documentation:"

- Suggest: `check_documentation_health_tool`
- Context: "Documentation health check available"

### Task Management Queries
**Template**: "When user asks about tasks:"


- Suggest: `analyze_todo2_alignment_tool` or `detect_duplicate_tasks_tool`
- Context: "Task analysis tools available"

### Security Queries
**Template**: "When user asks about security:"


- Suggest: `scan_dependency_security_tool`
- Context: "Dependency security scan available"

### Project Health Queries
**Template**: "When user asks about project status:"

- Suggest: `run_daily_automation_tool`
- Context: "Comprehensive project health check available"
```

### Pros

- ✅ Natural language integration
- ✅ Context-aware based on user intent
- ✅ Non-intrusive (suggested, not forced)

### Cons

- ❌ Requires query classification
- ❌ May miss implicit needs

---

## Approach 5: Workflow Hooks

### Concept

Add hooks at key workflow points to suggest tools.

### Implementation

Define workflow hooks:

```markdown

## Workflow Hooks

### Before Major Commits
**Hook**: User mentions "commit" or "push"
**Suggest**:


- `check_documentation_health_tool` (if docs changed)
- `detect_duplicate_tasks_tool` (if tasks created)
- `scan_dependency_security_tool` (if dependencies changed)


### After Task Completion
**Hook**: Task marked "Done"
**Suggest**:

- `analyze_todo2_alignment_tool` (if high priority)
- `detect_duplicate_tasks_tool` (check for similar tasks)

### Weekly Review
**Hook**: Day of week = Monday
**Suggest**: `run_daily_automation_tool` (weekly comprehensive check)

### Project Milestones
**Hook**: User mentions "release", "milestone", "sprint"
**Suggest**: `run_daily_automation_tool` (comprehensive check)
```

### Pros

- ✅ Natural workflow integration
- ✅ Actionable at decision points
- ✅ Context-aware

### Cons

- ❌ Requires workflow detection
- ❌ May be too frequent

---

## Approach 6: Context-Aware AI Assistant Rules

### Concept

Add rules that make AI assistants proactively suggest tools based on detected context.

### Implementation

Create rules for AI assistant behavior:

```markdown


## AI Assistant Tool Suggestion Rules

### Proactive Suggestions
**Rule**: "When AI detects relevant context, suggest automation tools"


**Examples**:

- User asks about documentation → Suggest `check_documentation_health_tool`
- User creates multiple tasks → Suggest `detect_duplicate_tasks_tool`
- User mentions security → Suggest `scan_dependency_security_tool`
- User asks about project status → Suggest `run_daily_automation_tool`

### Suggestion Format
**Template**:
"💡 **Automation Tool Available**: [tool_name] - [brief description]
   - **When to use**: [context]
   - **Why**: [benefit]
   - **Usage**: [example]"

### Rate Limiting
- Maximum 1 suggestion per conversation turn
- Don't suggest if user already using tool
- Don't suggest same tool twice in session
```

### Pros

- ✅ Natural conversation flow
- ✅ Context-aware
- ✅ Non-intrusive

### Cons

- ❌ Requires AI assistant awareness
- ❌ May be inconsistent

---

## Recommended Hybrid Approach

### Combination Strategy

Use multiple approaches together for comprehensive coverage:

1. **Cursor Rules** (Approach 1) - Static context triggers
2. **Task Lifecycle** (Approach 2) - Workflow integration
3. **AI Assistant Rules** (Approach 6) - Proactive suggestions
4. **Prompt Templates** (Approach 4) - Query-based suggestions

### Implementation Priority

1. **Phase 1**: Cursor rules with context triggers (simple, immediate value)
2. **Phase 2**: Task lifecycle integration (workflow-aware)
3. **Phase 3**: AI assistant proactive suggestions (advanced)
4. **Phase 4**: Prompt templates (refinement)

---

## Tool Suggestion Matrix

| Context | Suggested Tool | Trigger | Frequency |
|---------|---------------|---------|-----------|
| Documentation modified | `check_documentation_health_tool` | File change | Once per session |
| Task → Review | `analyze_todo2_alignment_tool` | Status change | Per task |
| Multiple tasks created | `detect_duplicate_tasks_tool` | Task creation | 5+ tasks |
| Dependencies changed | `scan_dependency_security_tool` | File change | Per change |
| Daily maintenance | `run_daily_automation_tool` | Time-based | Daily |
| Documentation queries | `check_documentation_health_tool` | Query type | Per query |
| Security queries | `scan_dependency_security_tool` | Query type | Per query |
| Project status queries | `run_daily_automation_tool` | Query type | Per query |

---

## Implementation Guidelines

### Rate Limiting

- **Per Tool**: Maximum 1 suggestion per tool per session
- **Per Context**: Maximum 1 suggestion per context change
- **Per Conversation**: Maximum 3 suggestions per conversation turn

### Suggestion Format

```markdown
💡 **Automation Tool Available**

**Tool**: `[tool_name]`
**Context**: [why this tool is relevant]
**When to use**: [specific trigger]
**Benefit**: [what it provides]
**Usage**: [example command or description]
```

### Non-Intrusive Design

- Suggestions are **informational**, not mandatory
- User can dismiss or ignore suggestions
- Don't block workflow for suggestions
- Provide clear "why" for each suggestion

---

## Example Implementations

### Example 1: Cursor Rule for Documentation Changes

```markdown

# Automation Tool Suggestions

## Documentation Context

**When**: Files in `docs/` directory are modified
**Suggest**: Consider running `check_documentation_health_tool` to:

- Validate links and references
- Check format compliance
- Identify stale documents
- Generate health report

**Usage**: "Check documentation health after recent changes"
```

### Example 2: Task Lifecycle Integration

```markdown

## Task Review Workflow

**When**: Task moves to "Review" status
**If**: Task is high priority
**Suggest**: Run `analyze_todo2_alignment_tool` to verify:

- Task aligns with project goals
- No misalignment issues
- Dependencies are satisfied

**Usage**: "Analyze task alignment before review"
```

### Example 3: AI Assistant Proactive Suggestion

```markdown

## AI Assistant Behavior

**Rule**: When user asks about project status or health
**Action**: Proactively suggest `run_daily_automation_tool`

**Format**:
"💡 **Daily Automation Available**: Run comprehensive daily maintenance
   - **Includes**: Documentation health, task alignment, duplicate detection
   - **Benefit**: Complete project health overview
   - **Usage**: `run_daily_automation_tool()`"
```

---

## Testing Strategy

### Validation Criteria

1. **Relevance**: Suggestions are contextually appropriate
2. **Frequency**: Not too frequent (rate limited)
3. **Actionability**: Clear when and why to use
4. **Non-Intrusive**: Don't disrupt workflow
5. **Effectiveness**: Users actually use suggested tools

### Test Scenarios

1. **Documentation Changes**: Modify docs, verify suggestion appears
2. **Task Lifecycle**: Move task to Review, verify suggestion
3. **Query Types**: Ask about docs/security/status, verify suggestions
4. **Rate Limiting**: Multiple triggers, verify limited suggestions
5. **Workflow Integration**: Verify suggestions fit naturally

---

## Next Steps

1. **Implement Phase 1**: Cursor rules with context triggers
2. **Test and Refine**: Validate suggestions are relevant
3. **Add Phase 2**: Task lifecycle integration
4. **Monitor Usage**: Track if suggestions lead to tool usage
5. **Iterate**: Refine based on feedback

---

## Related Documentation

- `.cursor/rules/project-automation.mdc` - Automation tool documentation
- `.cursor/rules/todo2.mdc` - Todo2 workflow rules
- `docs/ROUTINE_AUTOMATION_PLAN.md` - Automation task schedule
- `mcp-servers/project-management-automation/USAGE.md` - Tool usage guide

---

**Last Updated**: 2025-11-23
**Status**: Design Document
