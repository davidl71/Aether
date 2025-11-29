# Exarp Script Path Issue - Todo2 Plan

**Date**: 2025-11-29  
**Status**: Plan Created

---

## Overview

This document outlines the Todo2 plan for addressing the Exarp script path issue and implementing workarounds.

---

## Tasks Created

### 1. T-20251129180920-1: Investigate Exarp script discovery mechanism
**Priority**: Medium 🟡  
**Status**: Todo  
**Tags**: #exarp #investigation #debugging #automation

**Objective**: Understand how Exarp daily automation discovers and executes scripts to resolve the "Script not found" issue.

**Key Deliverables**:
- Document how Exarp finds scripts
- Identify why scripts in `scripts/` directory aren't being found
- Create findings document with recommendations
- File: `docs/EXARP_SCRIPT_DISCOVERY_INVESTIGATION.md`

---

### 2. T-20251129180920-2: Create wrapper script for Exarp daily automation using MCP tools
**Priority**: High 🟠  
**Status**: Todo  
**Tags**: #exarp #automation #mcp #scripting #workaround

**Objective**: Create a wrapper script that orchestrates Exarp MCP tools to replicate daily automation functionality.

**Key Deliverables**:
- Python/bash script that calls all three Exarp MCP tools
- Generates combined report similar to daily automation
- Handles errors gracefully
- Can be scheduled via cron or systemd
- File: `scripts/exarp_daily_automation_wrapper.py`

**Why High Priority**: This provides an immediate workaround for the script discovery issue.

---

### 3. T-20251129180920-3: Integrate Exarp MCP tools into daily automation script
**Priority**: High 🟠  
**Status**: Todo  
**Tags**: #exarp #automation #integration #daily-automation

**Objective**: Enhance existing daily automation script to include Exarp MCP tools alongside current automation tasks.

**Key Deliverables**:
- Modify `scripts/daily_automation_with_link_fixing.sh` to call Exarp MCP tools
- Combine Exarp reports with existing reports
- Maintain backward compatibility
- Update documentation

**Why High Priority**: Integrates Exarp checks into our existing daily automation workflow.

---

### 4. T-20251129180920-4: Monitor Exarp package updates for script discovery fix
**Priority**: Low 🟢  
**Status**: Todo  
**Tags**: #exarp #monitoring #maintenance #updates

**Objective**: Set up monitoring to detect when Exarp package updates fix the script discovery issue.

**Key Deliverables**:
- Document current Exarp version
- Create update check script
- Create test script to verify script discovery fix
- File: `scripts/check_exarp_updates.sh`
- File: `scripts/test_exarp_script_discovery.py`

**Why Low Priority**: This is a long-term monitoring task, not urgent.

---

### 5. T-20251129180920-5: Document Exarp script path workaround and best practices
**Priority**: Medium 🟡  
**Status**: Todo  
**Tags**: #exarp #documentation #best-practices #workaround

**Objective**: Create comprehensive documentation for using Exarp MCP tools as workaround for script discovery issue.

**Key Deliverables**:
- Enhance `docs/EXARP_SCRIPT_PATH_ISSUE_RESOLVED.md` with examples
- Create `docs/EXARP_MCP_TOOLS_USAGE.md` (if needed)
- Add to main project documentation index
- Include troubleshooting guide

---

## Task Dependencies

```
T-20251129180920-1 (Investigate)
    ↓
T-20251129180920-2 (Wrapper Script) ──┐
    ↓                                   │
T-20251129180920-3 (Integration) ──────┼──→ T-20251129180920-5 (Documentation)
    ↓                                   │
T-20251129180920-4 (Monitor Updates) ──┘
```

**Notes**:
- Tasks 2 and 3 can be worked on in parallel
- Task 5 (Documentation) should be done after tasks 2 and 3 are complete
- Task 4 (Monitoring) is independent and can be done anytime

---

## Recommended Execution Order

### Phase 1: Immediate Workarounds (High Priority)
1. **T-20251129180920-2**: Create wrapper script for Exarp daily automation
   - Provides immediate workaround
   - Can be used independently
   - Estimated: 2-4 hours

2. **T-20251129180920-3**: Integrate Exarp MCP tools into daily automation script
   - Enhances existing automation
   - Can be done in parallel with task 2
   - Estimated: 2-3 hours

### Phase 2: Investigation and Documentation (Medium Priority)
3. **T-20251129180920-1**: Investigate Exarp script discovery mechanism
   - Helps understand root cause
   - May reveal permanent fix
   - Estimated: 3-5 hours

4. **T-20251129180920-5**: Document workaround and best practices
   - Should be done after tasks 2 and 3
   - Ensures knowledge is captured
   - Estimated: 1-2 hours

### Phase 3: Long-term Monitoring (Low Priority)
5. **T-20251129180920-4**: Monitor Exarp package updates
   - Ongoing task
   - Can be automated
   - Estimated: 1-2 hours initial setup

---

## Success Criteria

### Task 2 (Wrapper Script) ✅
- [ ] Script executes all three Exarp MCP tools
- [ ] Generates combined report
- [ ] Handles errors gracefully
- [ ] Can be scheduled via cron
- [ ] Returns proper exit codes

### Task 3 (Integration) ✅
- [ ] Daily automation script calls Exarp MCP tools
- [ ] Combined report includes Exarp results
- [ ] No breaking changes to existing functionality
- [ ] Documentation updated

### Task 1 (Investigation) ✅
- [ ] Documented how Exarp finds scripts
- [ ] Identified root cause of script discovery issue
- [ ] Created findings document with recommendations

### Task 5 (Documentation) ✅
- [ ] Enhanced existing documentation with examples
- [ ] Created usage guide (if needed)
- [ ] Added to documentation index
- [ ] Included troubleshooting guide

### Task 4 (Monitoring) ✅
- [ ] Current Exarp version documented
- [ ] Update check script created
- [ ] Test script created to verify fixes
- [ ] Process documented

---

## Current Status

**Workaround**: ✅ Working
- Individual MCP tools work perfectly
- Our daily automation script works perfectly

**Scripts Created**: ✅ Complete
- `scripts/automate_docs_health_v2.py`
- `scripts/automate_todo2_alignment_v2.py`
- `scripts/automate_todo2_duplicate_detection.py`

**Configuration**: ✅ Complete
- `.exarp/config.json` created

**Issue**: ❌ Still Present
- Exarp daily automation still reports "Script not found"
- Likely an Exarp internal issue with script discovery

---

## Next Steps

1. **Start with Task 2** (High Priority): Create wrapper script
2. **Work on Task 3** (High Priority): Integrate into daily automation
3. **Investigate Task 1** (Medium Priority): Understand root cause
4. **Document Task 5** (Medium Priority): Capture knowledge
5. **Monitor Task 4** (Low Priority): Watch for Exarp updates

---

## Related Documentation

- `docs/EXARP_SCRIPT_PATH_ISSUE_RESOLVED.md` - Current status and workarounds
- `docs/EXARP_SCRIPT_PATH_FIX.md` - Initial investigation
- `docs/DAILY_AUTOMATION_SETUP_COMPLETE.md` - Daily automation setup
- `.exarp/config.json` - Exarp configuration

---

**Last Updated**: 2025-11-29  
**Status**: Plan created, ready for execution
