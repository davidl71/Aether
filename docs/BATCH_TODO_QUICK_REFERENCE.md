# Batch TODO Update - Quick Reference

## Most Common Commands

### ✅ Batch Approve Research Tasks

```bash
python3 scripts/batch_update_todos.py approve --status Review --clarification-none --yes
```

### 📋 List Tasks Needing Decisions

```bash
python3 scripts/batch_update_todos.py list --status Review
```

### 📋 List Tasks Ready to Approve

```bash
python3 scripts/batch_update_todos.py list --status Review --clarification-none

```

### 🔄 Update Specific Tasks

```bash

python3 scripts/batch_update_todos.py update-status --task-ids T-156,T-157 --new-status Todo --yes
```

### 💬 Add Comment to Task

```bash
python3 scripts/batch_update_todos.py add-comment --task-ids T-156 --comment "Approved for execution"
```

---

## Workflow: Review → Approve → Execute

### Step 1: See What Needs Your Input

```bash
python3 scripts/batch_update_todos.py list --status Review
```

### Step 2: Approve Tasks That Don't Need Clarification

```bash
python3 scripts/batch_update_todos.py approve --status Review --clarification-none --yes
```

### Step 3: Tasks Are Now in Todo Status (Ready for Automation)

The nightly automation will pick them up automatically.

---

## Comparison: Script vs Python Heredoc

### ❌ Old Way (Python Heredoc)

```bash
python3 << 'EOF'
import json
from pathlib import Path


# ... 100+ lines of code ...

EOF
```

### ✅ New Way (Script)

```bash
python3 scripts/batch_update_todos.py approve --status Review --clarification-none --yes
```

**Benefits:**

- ✅ Reusable
- ✅ Documented
- ✅ Testable
- ✅ Maintainable
- ✅ Easy to remember

---

## Full Documentation

See `docs/BATCH_TODO_UPDATE_SCRIPT.md` for complete documentation.
