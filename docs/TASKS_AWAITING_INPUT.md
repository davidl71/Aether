# Tasks Awaiting Your Input

**Date:** 2025-11-24
**Status:** 12 tasks in Review status

---

## Summary

**Total Tasks Awaiting Input:** 12

These tasks have been moved to Review status because they require clarification or design decisions before implementation can proceed.

---

## Tasks by Category

### 🔴 Multi-Broker & Account Management (4 tasks)

#### T-36: Implement IB Client Portal API adapter

- **Priority:** High
- **Question:** Use alongside TWS API or as alternative?
- **Context:** Need to decide if Client Portal API complements or replaces TWS API
- **Recommendation:** Consider using alongside TWS for different use cases (web-based vs desktop)

#### T-37: Implement broker selection and switching mechanism

- **Priority:** High
- **Question:** Can users switch brokers at runtime or only at startup?
- **Context:** Affects architecture - runtime switching requires more complex state management
- **Recommendation:** Start with startup-only, add runtime switching later if needed

#### T-78: Implement multi-account connection and authentication

- **Priority:** High
- **Question:** Credential storage approach, connection retry strategy, session timeout handling
- **Context:** Security and reliability decisions needed
- **Recommendation:**
  - Credential storage: Use system keychain/credential store
  - Retry strategy: Exponential backoff with max retries
  - Session timeout: Configurable, default 30 minutes

#### T-79: Implement portfolio position aggregation logic

- **Priority:** High
- **Question:** Duplicate handling strategy (merge vs separate), update frequency (real-time vs periodic)
- **Context:** How to handle positions from multiple accounts/brokers
- **Recommendation:**
  - Duplicate handling: Merge by symbol, show source accounts
  - Update frequency: Real-time for active positions, periodic for historical

---

### 🔴 Investment Strategy (2 tasks)

#### T-60: Design investment strategy framework with allocation rules

- **Priority:** High
- **Question:** User's risk tolerance, return targets, liquidity needs
- **Context:** Need to define default values or gather user requirements
- **Recommendation:** Create configurable framework with sensible defaults, allow user customization

#### T-61: Document user requirements and assumptions for strategy

- **Priority:** Medium
- **Question:** User input needed for risk tolerance, goals, preferences
- **Context:** Documentation task - can proceed with template structure
- **Recommendation:** Create template with placeholders, fill in user-specific values later

---

### 🔴 Bank Loan Integration (2 tasks)

#### T-76: Implement bank loan position data model and storage

- **Priority:** High
- **Question:** Storage format preference (JSON config vs database), loan update API design
- **Context:** Need to decide on persistence layer
- **Recommendation:**
  - Start with JSON config for simplicity
  - Design for future database migration
  - REST API for updates

#### T-77: Implement loan position entry interface

- **Priority:** High
- **Question:** Preferred interface (TUI form, CLI commands, config file editing), import file format
- **Context:** User experience decision
- **Recommendation:**
  - Primary: TUI form (interactive)
  - Secondary: Config file editing (power users)
  - Import: CSV format (common, easy to generate)

---

### 🔴 Configuration System (4 tasks)

#### T-111: Design shared configuration file format for data sources

- **Priority:** High
- **Question:** Should config support multiple active sources or single source selection?
- **Context:** Architecture decision for data source management
- **Recommendation:** Support multiple active sources with priority/fallback order

#### T-112: Implement shared configuration loader for TUI, PWA, and standalone

- **Priority:** High
- **Question:** Should loader be in Python, TypeScript, or both?
- **Context:** Multi-language codebase decision
- **Recommendation:** Both - Python for TUI/backend, TypeScript for PWA, shared JSON schema

#### T-113: Add data source configuration UI to PWA

- **Priority:** High
- **Question:** Should this be a settings page, modal, or both?
- **Context:** UI/UX decision
- **Recommendation:** Settings page (primary) with modal for quick edits

#### T-114: Update TUI to use shared configuration file

- **Priority:** High
- **Question:** Should TUI watch for config file changes or only read on startup?
- **Context:** Real-time vs static configuration
- **Recommendation:** Watch for changes (better UX, allows hot-reload)

---

## Quick Decision Guide

### If You Want to Proceed Quickly

1. **Use Defaults/Recommendations Above**
   - Each task has a recommendation
   - These are sensible defaults based on best practices
   - Can be refined later

2. **Update Task Descriptions**
   - Add your decisions to task descriptions
   - Update "Clarification Required" to "None" or remove it
   - Add implementation notes

3. **Move to Todo Status**

   ```bash
   python3 scripts/batch_update_todos.py update-status \
     --task-ids T-36,T-37,T-60,T-61,T-76,T-77,T-78,T-79,T-111,T-112,T-113,T-114 \
     --new-status Todo \
     --yes
   ```

### If You Want to Review First

1. **Review Each Category**
   - Multi-broker decisions affect architecture
   - Investment strategy affects user experience
   - Bank loan integration affects data model
   - Configuration system affects all components

2. **Make Decisions**
   - Consider your specific use case
   - Think about future scalability
   - Balance simplicity vs flexibility

3. **Update Tasks**
   - Document decisions in task descriptions
   - Add implementation notes
   - Move to Todo when ready

---

## Task Details

For detailed information about each task, see:

- `.todo2/state.todo2.json` - Full task details
- `docs/TASK_REVIEW_ANALYSIS.md` - Detailed analysis

---

## Next Steps

1. **Review** the clarification questions above
2. **Decide** on your preferred approach for each category
3. **Update** task descriptions with your decisions
4. **Approve** tasks to move them to Todo status
5. **Begin** implementation once tasks are approved

---

**Last Updated:** 2025-11-24
**Status:** Awaiting your input on 12 tasks
