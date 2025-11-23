# Apple Intelligence Quick Reference for Development

**Date:** 2025-01-20
**Purpose:** Quick reference guide for using Apple Intelligence on M4 remote agent
**Status:** ✅ **Active Guide**

---

## Overview

This guide provides quick reference commands and workflows for leveraging Apple Intelligence on your M4 macOS remote agent to enhance parallel development workflows.

---

## Key Apple Intelligence Features

### 1. Writing Tools (System-Wide)

**Available in:** All text fields, Notes, Mail, Pages, TextEdit, code editors

**Access Methods:**
- **Right-click** → "Rewrite" or "Proofread"
- **Keyboard shortcut:** Select text → Context menu
- **Menu bar:** Edit → Writing Tools

**Use Cases for Development:**

| Task | How to Use | Benefit |
|------|------------|---------|
| **Improve code comments** | Select comment → Right-click → "Rewrite" → "Professional" | Clearer, more professional documentation |
| **Better commit messages** | Select commit message → "Rewrite" → "Concise" | More descriptive, follow conventions |
| **Documentation clarity** | Select doc text → "Proofread" | Fix grammar, improve readability |
| **Error message improvement** | Select error text → "Rewrite" → "Friendly" | User-friendly error descriptions |

**Example:**
```cpp
// Before (select and use Writing Tools):
// This function does stuff with options

// After (AI-improved):
// Calculates box spread profitability by comparing net premium received
// against the theoretical expiration value (strike width minus net premium)
```

---

### 2. Summarization (System-Wide)

**Available in:** Safari, Notes, Mail, most text fields

**How to Use:**
- Select long text (research papers, API docs, logs)
- Right-click → "Summarize"
- AI generates concise summary

**Use Cases:**

| Source | Action | Output |
|--------|--------|--------|
| **Research paper** | Summarize | Key findings, methodology, conclusions |
| **API documentation** | Summarize | Main concepts, usage patterns, examples |
| **Build logs** | Summarize | Errors, warnings, key build steps |
| **Error stack traces** | Summarize | Root cause, affected components |

**Workflow:**
1. Copy long documentation or error log
2. Paste into Notes or TextEdit
3. Select all → Right-click → "Summarize"
4. Review AI-generated summary
5. Use summary for TODO updates or documentation

---

### 3. Image Playground

**Available in:** Shortcuts app, system integration

**Use Cases:**

| Task | Description | Example |
|------|-------------|---------|
| **Architecture diagrams** | System design visuals | "Box spread trading system architecture" |
| **Flow charts** | Process workflows | "NATS message flow diagram" |
| **Concept diagrams** | Visual explanations | "Options pricing relationships" |
| **Documentation images** | Technical illustrations | "Git workflow branching strategy" |

**How to Access:**
1. Open Shortcuts app
2. Create new shortcut
3. Add "Generate Image" action
4. Describe what you want
5. Save and run

**Or via Command Line:**
```bash
# Generate diagram via Shortcuts
shortcuts run "Generate Architecture Diagram" \
  --input-text "Box spread trading system with NATS, TWS API, Rust backend"
```

---

### 4. Siri Intelligence (Voice & Context)

**Available in:** System-wide, all apps

**Use Cases:**

| Task | Voice Command | Benefit |
|------|---------------|---------|
| **Quick information** | "What's the status of the trading system?" | Context-aware system queries |
| **App suggestions** | Automatic based on context | Smart workflow recommendations |
| **Quick actions** | "Run tests on Mac Pro" | Voice-activated automation |

**Enable:**
1. System Settings → Siri & Search
2. Enable "Listen for 'Hey Siri'"
3. Enable "Allow Siri When Locked"
4. Customize activation phrase

---

## Development Workflow Examples

### Example 1: Improve Code Comments

**Context:** Writing C++ trading code on macOS M4 remote agent

**Steps:**
1. Write initial code comment:
   ```cpp
   // This function calculates profit
   ```

2. Select comment → Right-click → "Rewrite" → "Professional"

3. AI improves to:
   ```cpp
   // Calculates box spread profit by comparing net premium received
   // against the strike width, accounting for transaction costs
   ```

4. Accept improvement → Continue coding

**Time Saved:** 30 seconds per comment × 20 comments = 10 minutes

---

### Example 2: Generate Commit Message

**Context:** About to commit changes after Ubuntu agent review

**Steps:**
1. Review changes:
   ```bash
   git diff
   ```

2. Draft initial message:
   ```
   fixed bug
   ```

3. Select text → Right-click → "Rewrite" → "Concise"

4. AI improves to:
   ```
   Fix box spread calculation error when strike width is zero

   - Add validation to prevent division by zero
   - Update error handling for edge cases
   - Add unit tests for zero-width scenarios
   ```

5. Use improved message:
   ```bash
   git commit -m "Fix box spread calculation error when strike width is zero"
   ```

**Time Saved:** 2-3 minutes per commit × 5 commits = 10-15 minutes

---

### Example 3: Summarize Research Documentation

**Context:** Ubuntu agent shared long API documentation that needs to be understood

**Steps:**
1. Copy documentation text from Ubuntu agent's work
2. Paste into Notes app
3. Select all → Right-click → "Summarize"
4. AI generates:
   ```
   Summary: NATS API Documentation

   Key Concepts:
   - Publish/subscribe messaging pattern
   - Topic-based routing
   - At-least-once delivery guarantees

   Main Functions:
   - publish(): Send messages to topics
   - subscribe(): Receive messages from topics
   - request(): Request-reply pattern

   Usage Pattern:
   1. Connect to NATS server
   2. Subscribe to topics
   3. Publish messages
   4. Handle responses
   ```

5. Use summary for:
   - TODO table updates
   - API contract documentation
   - Quick reference

**Time Saved:** 15-20 minutes reading → 2 minutes summary

---

### Example 4: Generate Architecture Diagram

**Context:** Need to document system architecture for both agents

**Steps:**
1. Open Shortcuts app
2. Create shortcut: "Generate Architecture Diagram"
3. Add action: "Generate Image"
4. Description:
   ```
   Box spread trading system architecture diagram showing:
   - Frontend (macOS AppKit UI)
   - Backend (Rust services)
   - Message Queue (NATS)
   - Market Data (TWS API)
   - Database (QuestDB)
   ```
5. Run shortcut
6. Export image → Add to documentation

**Time Saved:** 30-60 minutes manual diagram → 5 minutes AI-generated

---

### Example 5: Improve Error Messages

**Context:** Complex compiler error that needs explanation

**Steps:**
1. Copy error message:
   ```
   error: no matching function for call to 'calculate_profit'
   candidate template ignored: could not match 'Option<BoxSpread>' to 'const BoxSpread&'
   ```

2. Select error → Right-click → "Rewrite" → "Friendly"

3. AI improves to:
   ```
   Error: Function call mismatch

   Problem: The calculate_profit() function expects a const reference to BoxSpread,
   but you're passing an Option<BoxSpread> (which may be empty).

   Solution: Unwrap the Option first, or handle the None case:

   if let Some(spread) = spread_option {
       let profit = calculate_profit(&spread);
   }
   ```

**Time Saved:** 5-10 minutes debugging → Instant explanation

---

## Integration with Parallel Development

### macOS M4 Agent Workflow

**Recommended Task Distribution:**

| Task Type | Agent | Apple Intelligence Usage |
|-----------|-------|-------------------------|
| **Code Implementation** | Ubuntu + macOS | Use Cursor AI |
| **Documentation** | macOS M4 | Writing Tools, Summarization |
| **Visual Content** | macOS M4 | Image Playground |
| **Error Analysis** | macOS M4 | Summarization, Rewrite |
| **Commit Messages** | macOS M4 | Writing Tools |
| **Research Summaries** | macOS M4 | Summarization |

**Workflow:**
1. **Ubuntu Agent:** Implements feature, writes initial code
2. **macOS M4 Agent:**
   - Implements macOS-specific components
   - Uses Apple Intelligence to:
     - Improve Ubuntu agent's documentation
     - Generate visual diagrams
     - Summarize research/API docs
     - Improve all commit messages
     - Analyze and explain errors

3. **Both Agents:** Coordinate via shared TODO table

---

## Quick Command Reference

### Enable Apple Intelligence

```bash
# Check if Apple Intelligence is available
sysctl machdep.cpu.brand_string

# Verify M4 chip (should show "Apple M4")
# Enable in System Settings → General → Apple Intelligence
```

### Writing Tools Keyboard Shortcuts

| Action | Shortcut |
|--------|----------|
| Open Writing Tools menu | Select text → Right-click |
| Rewrite selected text | Right-click → "Rewrite" |
| Proofread selected text | Right-click → "Proofread" |
| Summarize selected text | Right-click → "Summarize" |

### Image Playground Shortcuts

```bash
# Create shortcut via command line
shortcuts create "Generate Diagram" \
  --action "Generate Image" \
  --description "Box spread trading system architecture"

# Run shortcut
shortcuts run "Generate Diagram"
```

---

## Best Practices

### ✅ DO

- **Use Writing Tools** for all documentation (comments, README, docs)
- **Summarize** long research papers and API documentation
- **Generate diagrams** for architecture and workflows
- **Improve commit messages** before committing
- **Explain errors** using AI before debugging

### ❌ DON'T

- Don't rely solely on Apple Intelligence for code generation (use Cursor AI)
- Don't use AI for security-sensitive descriptions
- Don't skip human review of AI-generated content
- Don't use AI for proprietary algorithm documentation

---

## Time Savings Estimation

| Task | Manual Time | With Apple Intelligence | Time Saved |
|------|-------------|------------------------|------------|
| Improve 20 code comments | 20 min | 5 min | 15 min |
| Write 5 commit messages | 15 min | 3 min | 12 min |
| Summarize research paper | 20 min | 2 min | 18 min |
| Create architecture diagram | 45 min | 5 min | 40 min |
| Explain 3 complex errors | 30 min | 3 min | 27 min |

**Total per session:** ~130 minutes → ~18 minutes = **~112 minutes saved**

---

## Resources

- [Apple Intelligence Overview](https://www.apple.com/apple-intelligence/)
- [Writing Tools Guide](https://support.apple.com/en-us/HT214146)
- [Image Playground Documentation](https://support.apple.com/en-us/HT214147)
- [Device Task Delegation](./DEVICE_TASK_DELEGATION.md) - Full Apple Intelligence integration guide

---

**Quick Start:**
1. Enable Apple Intelligence (System Settings → General → Apple Intelligence)
2. Try Writing Tools on any code comment
3. Generate first diagram with Image Playground
4. Summarize your first research document
5. Start saving time on every development session!
