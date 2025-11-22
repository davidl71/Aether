# Claude Skills MCP Servers Analysis

**Date:** 2025-01-20
**Issue:** Exceeding total tools limit
**Location:** User-level MCP config (`~/.cursor/mcp.json`)

## Current Claude Skills Servers

### 1. `claude-scientific-skills`
- **Purpose:** Provides scientific/mathematical workflows and domain knowledge
- **URL:** `https://mcp.k-dense.ai/claude-scientific-skills/mcp`
- **Use Case:** Scientific computing, mathematical analysis, research workflows

### 2. `claude-skills-mcp`
- **Purpose:** General Claude skills library for reusable workflows
- **Use Case:** General-purpose skill templates and workflows

## Analysis for IB Box Spread Project

### Project Context
- **Type:** Trading software (options box spread arbitrage)
- **Languages:** C++20, Python, Rust, TypeScript
- **Focus:** Financial calculations, TWS API integration, real-time trading

### Relevance Assessment

#### `claude-scientific-skills` - ⚠️ **PARTIALLY RELEVANT**
**Pros:**
- Could help with mathematical/statistical calculations
- May provide financial math workflows
- Useful for complex option pricing calculations

**Cons:**
- Trading software has specific domain knowledge (not general scientific)
- Financial calculations are well-documented and project-specific
- May not provide trading-specific workflows

**Verdict:** **OPTIONAL** - Could be useful for complex math, but not essential

#### `claude-skills-mcp` - ❌ **LOW RELEVANCE**
**Pros:**
- General-purpose skill templates
- Reusable workflows

**Cons:**
- Generic skills may not match trading software needs
- Project has specific patterns and workflows already established
- Redundant with project-specific documentation and rules

**Verdict:** **LOW PRIORITY** - Generic skills less useful for specialized trading software

## Recommendation

### Option 1: Remove Both (Recommended)
**Saves:** 2 MCP servers

**Rationale:**
- Trading software has specific domain knowledge (not general scientific)
- Project already has comprehensive documentation and rules
- Financial calculations are well-established patterns
- Generic skills may not align with trading software workflows

**Impact:**
- ✅ No loss of essential functionality
- ✅ Reduces tool count significantly
- ✅ Focuses AI on project-specific knowledge

### Option 2: Keep `claude-scientific-skills`, Remove `claude-skills-mcp`
**Saves:** 1 MCP server

**Rationale:**
- Scientific skills might help with complex mathematical calculations
- Financial math is a subset of scientific computing
- Could be useful for option pricing formulas

**Impact:**
- ⚠️ Keeps potentially useful math workflows
- ✅ Removes generic skills server

### Option 3: Keep Both (Not Recommended)
**Saves:** 0 MCP servers

**Rationale:**
- Maximum flexibility
- Access to all available skills

**Impact:**
- ❌ Doesn't solve tool limit issue
- ❌ May add unnecessary complexity

## Final Recommendation

**Remove both `claude-scientific-skills` and `claude-skills-mcp`**

**Reasoning:**
1. **Project-specific knowledge:** Trading software requires domain-specific knowledge, not general scientific skills
2. **Established patterns:** Project already has comprehensive rules, documentation, and established workflows
3. **Tool limit:** Removing both saves 2 server slots
4. **Focus:** Better to have AI focus on project-specific documentation and rules rather than generic skills

**Alternative:** If you need mathematical help, you can:
- Use web search for specific formulas
- Reference project documentation
- Use established C++/Python libraries for financial calculations

## How to Remove

Edit `~/.cursor/mcp.json` and remove these sections:

```json
{
  "mcpServers": {
    // Remove "claude-scientific-skills" section
    // Remove "claude-skills-mcp" section
    // Keep other servers...
  }
}
```

Then restart Cursor to apply changes.
