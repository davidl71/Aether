# Migration Plan: Exarp CLI to MCP Tools

**Date**: 2025-12-10
**Status**: Planning
**Goal**: Migrate from Exarp CLI calls (`uvx exarp`) to direct Python function calls to Exarp tools

---

## Executive Summary

Currently, the daily automation scripts use a mixed approach:
- `exarp_daily_automation_wrapper.py` calls Exarp via CLI (`uvx exarp`)
- Other wrappers call local scripts that may import from Exarp package

**Target State**: All scripts call Exarp tools directly as Python functions, eliminating CLI overhead and improving reliability.

---

## Current Architecture

### Phase 1: Exarp Checks (via CLI)
- **Script**: `scripts/exarp_daily_automation_wrapper.py`
- **Method**: Subprocess calls to `uvx exarp check-documentation-health`, etc.
- **Issues**:
  - Requires `uvx` and Exarp package installation
  - Subprocess overhead
  - Error handling via exit codes
  - No direct access to structured results

### Phase 2: Documentation Automation (Local Scripts)
- **Scripts**:
  - `exarp_fix_documentation_links.py` → calls `automate_documentation_link_fixing.py`
  - `exarp_validate_docs_format.py` → local validation
  - `exarp_sync_shared_todo.py` → local TODO sync
- **Status**: These are project-specific and can remain local

---

## Migration Strategy

### Approach: Direct Python Function Calls

Instead of:
```python
subprocess.run(['uvx', 'exarp', 'check-documentation-health', project_dir])
```

Use:
```python
from project_management_automation.tools.docs_health import check_documentation_health
result = check_documentation_health(project_dir=project_dir, create_tasks=True)
```

---

## Migration Steps

### Step 1: Create Exarp Tool Client Module

**File**: `scripts/exarp_mcp_client.py`

**Purpose**: Centralized module for calling Exarp tools programmatically

**Implementation**:
```python
#!/usr/bin/env python3
"""
Exarp MCP Tool Client

Provides programmatic access to Exarp tools without CLI overhead.
Can be used by automation scripts to call Exarp functions directly.
"""

import sys
import json
from pathlib import Path
from typing import Dict, Any, Optional

# Add Exarp package to path
EXARP_PACKAGE_PATH = Path(__file__).parent.parent.parent / "project-management-automation"
if EXARP_PACKAGE_PATH.exists():
    sys.path.insert(0, str(EXARP_PACKAGE_PATH))

try:
    from project_management_automation.tools.docs_health import check_documentation_health
    from project_management_automation.tools.todo2_alignment import analyze_todo2_alignment
    from project_management_automation.tools.duplicate_detection import detect_duplicate_tasks
    EXARP_AVAILABLE = True
except ImportError:
    EXARP_AVAILABLE = False
    # Fallback: try via package installation
    try:
        import exarp_project_management
        from exarp_project_management.tools.docs_health import check_documentation_health
        from exarp_project_management.tools.todo2_alignment import analyze_todo2_alignment
        from exarp_project_management.tools.duplicate_detection import detect_duplicate_tasks
        EXARP_AVAILABLE = True
    except ImportError:
        EXARP_AVAILABLE = False


class ExarpToolClient:
    """Client for calling Exarp tools programmatically"""

    def __init__(self, project_dir: Path):
        self.project_dir = Path(project_dir).resolve()
        if not EXARP_AVAILABLE:
            raise RuntimeError("Exarp package not available. Install with: pip install exarp-automation-mcp")

    def check_documentation_health(
        self,
        create_tasks: bool = True,
        output_path: Optional[str] = None,
        dry_run: bool = False
    ) -> Dict[str, Any]:
        """Check documentation health"""
        result = check_documentation_health(
            project_dir=str(self.project_dir),
            create_tasks=create_tasks,
            output_path=output_path
        )
        # Parse JSON string result
        if isinstance(result, str):
            return json.loads(result)
        return result

    def analyze_todo2_alignment(
        self,
        create_followup_tasks: bool = True,
        output_path: Optional[str] = None,
        dry_run: bool = False
    ) -> Dict[str, Any]:
        """Analyze Todo2 alignment"""
        result = analyze_todo2_alignment(
            create_followup_tasks=create_followup_tasks,
            output_path=output_path
        )
        if isinstance(result, str):
            return json.loads(result)
        return result

    def detect_duplicate_tasks(
        self,
        similarity_threshold: float = 0.85,
        auto_fix: bool = False,
        output_path: Optional[str] = None,
        dry_run: bool = False
    ) -> Dict[str, Any]:
        """Detect duplicate tasks"""
        result = detect_duplicate_tasks(
            project_dir=str(self.project_dir),
            similarity_threshold=similarity_threshold,
            auto_fix=auto_fix,
            output_path=output_path
        )
        if isinstance(result, str):
            return json.loads(result)
        return result
```

---

### Step 2: Migrate `exarp_daily_automation_wrapper.py`

**Current**: Uses subprocess calls to `uvx exarp`

**New**: Uses `ExarpToolClient` for direct function calls

**Changes**:
```python
# OLD
command = ['uvx', 'exarp', 'check-documentation-health', str(self.project_dir)]
result = subprocess.run(command, ...)

# NEW
from exarp_mcp_client import ExarpToolClient
client = ExarpToolClient(self.project_dir)
result = client.check_documentation_health(dry_run=self.dry_run)
```

**Benefits**:
- ✅ No subprocess overhead
- ✅ Direct access to structured results
- ✅ Better error handling (exceptions vs exit codes)
- ✅ Faster execution
- ✅ Works even if `uvx` not in PATH

---

### Step 3: Update `daily_automation_with_link_fixing.sh`

**Changes**: Minimal - script already calls Python wrapper, no changes needed

**Verification**: Test that migrated wrapper works correctly

---

### Step 4: Update Local Scripts (Optional)

**Scripts to Review**:
- `scripts/automate_docs_health_v2.py` - Already tries to import from Exarp
- `scripts/automate_todo2_alignment_v2.py` - Already tries to import from Exarp
- `scripts/automate_todo2_duplicate_detection.py` - Already tries to import from Exarp

**Action**: These can use `ExarpToolClient` instead of direct imports

---

### Step 5: Testing & Validation

**Test Plan**:

1. **Unit Tests**:
   ```bash
   # Test ExarpToolClient
   python3 -c "from scripts.exarp_mcp_client import ExarpToolClient; client = ExarpToolClient('.'); print('✅ Client initialized')"
   ```

2. **Integration Tests**:
   ```bash
   # Test migrated wrapper
   python3 scripts/exarp_daily_automation_wrapper.py . --dry-run
   ```

3. **End-to-End Tests**:
   ```bash
   # Test full daily automation
   ./scripts/daily_automation_with_link_fixing.sh . --dry-run
   ```

4. **Compare Results**:
   - Run old CLI version
   - Run new direct-call version
   - Compare outputs for consistency

---

### Step 6: Documentation Updates

**Files to Update**:
- `docs/DAILY_AUTOMATION_SETUP_COMPLETE.md` - Update usage examples
- `docs/EXARP_MCP_TOOLS_USAGE.md` - Add programmatic usage section
- `scripts/exarp_daily_automation_wrapper.py` - Update docstring

---

## Implementation Checklist

### Phase 1: Foundation
- [ ] Create `scripts/exarp_mcp_client.py` module
- [ ] Add error handling and fallback logic
- [ ] Add logging for debugging
- [ ] Test Exarp package import paths

### Phase 2: Migration
- [ ] Migrate `exarp_daily_automation_wrapper.py` to use `ExarpToolClient`
- [ ] Update all three tool calls (docs health, alignment, duplicates)
- [ ] Maintain backward compatibility (same CLI interface)
- [ ] Add `--use-cli` flag for fallback

### Phase 3: Testing
- [ ] Unit tests for `ExarpToolClient`
- [ ] Integration tests for migrated wrapper
- [ ] End-to-end test with daily automation script
- [ ] Compare old vs new results

### Phase 4: Cleanup
- [ ] Remove unused subprocess code
- [ ] Update documentation
- [ ] Update cron job setup scripts if needed
- [ ] Archive old CLI-based version

---

## Rollback Plan

**If migration fails**:
1. Keep old CLI-based version as `exarp_daily_automation_wrapper.py.old`
2. Add `--use-cli` flag to fallback to subprocess calls
3. Revert git commit if needed

**Safety Measures**:
- Test in dry-run mode first
- Keep old version until new version validated
- Monitor first few automated runs

---

## Benefits

### Performance
- **Faster**: No subprocess overhead
- **More Reliable**: Direct function calls vs CLI parsing
- **Better Errors**: Python exceptions vs exit codes

### Maintainability
- **Simpler**: Direct imports vs CLI command construction
- **Type Safety**: Python types vs string arguments
- **Debugging**: Easier to debug Python code vs subprocess

### Flexibility
- **No uvx Dependency**: Works if `uvx` not installed
- **Direct Access**: Can access intermediate results
- **Extensible**: Easy to add new tools

---

## Risks & Mitigation

### Risk 1: Exarp Package Not Available
**Mitigation**:
- Check for package at import time
- Provide clear error message
- Fallback to CLI if needed

### Risk 2: API Changes
**Mitigation**:
- Pin Exarp package version
- Test with specific version
- Document version requirements

### Risk 3: Breaking Changes
**Mitigation**:
- Maintain backward compatibility
- Keep CLI fallback option
- Gradual migration with testing

---

## Timeline

**Estimated Duration**: 2-3 hours

1. **Step 1-2** (Foundation + Migration): 1 hour
2. **Step 3-4** (Testing): 30 minutes
3. **Step 5** (Documentation): 30 minutes
4. **Step 6** (Validation): 30 minutes

---

## Success Criteria

✅ All three Exarp tools work via direct calls
✅ No CLI dependency required
✅ Same or better performance
✅ Backward compatible interface
✅ All tests pass
✅ Documentation updated

---

## Next Steps

1. Review this plan
2. Create `exarp_mcp_client.py`
3. Migrate `exarp_daily_automation_wrapper.py`
4. Test thoroughly
5. Deploy and monitor

---

## References

- Exarp Tool Functions:
  - `project_management_automation.tools.docs_health.check_documentation_health`
  - `project_management_automation.tools.todo2_alignment.analyze_todo2_alignment`
  - `project_management_automation.tools.duplicate_detection.detect_duplicate_tasks`

- Current Implementation:
  - `scripts/exarp_daily_automation_wrapper.py`
  - `scripts/daily_automation_with_link_fixing.sh`

---

## Future Enhancements

### Wisdom/DevWisdom Integration

**Note**: Wisdom functionality has been migrated to the devwisdom-go MCP server (separate from Exarp). The current daily automation scripts do not use wisdom, but it can be added as a future enhancement.

**If adding wisdom to daily automation:**

1. **Create DevWisdom MCP Client** (similar to `ExarpToolClient`):
   ```python
   # scripts/devwisdom_mcp_client.py
   from project_management_automation.utils.wisdom_client import (
       call_wisdom_tool_sync,
       read_wisdom_resource_sync
   )

   class DevWisdomClient:
       def get_daily_briefing(self, score: int = 50) -> Dict[str, Any]:
           return call_wisdom_tool_sync("get_daily_briefing", {"score": score})

       def consult_advisor(self, metric: str, score: int) -> Dict[str, Any]:
           return call_wisdom_tool_sync("consult_advisor", {
               "metric": metric,
               "score": score
           })
   ```

2. **Integration Points**:
   - Add wisdom consultation after project health checks
   - Include wisdom quotes in daily automation summary
   - Use advisor recommendations for task prioritization

3. **Dependencies**:
   - devwisdom-go MCP server must be configured in `.cursor/mcp.json`
   - Requires `mcp` Python package for MCP client functionality
   - See `project_management_automation/utils/wisdom_client.py` for reference implementation

4. **Example Usage**:
   ```python
   # In exarp_daily_automation_wrapper.py
   from devwisdom_mcp_client import DevWisdomClient

   # After running health checks
   wisdom_client = DevWisdomClient()
   briefing = wisdom_client.get_daily_briefing(score=health_score)
   ```

**References**:
- DevWisdom MCP Server: `devwisdom-go` repository
- Exarp Wisdom Client: `project_management_automation/utils/wisdom_client.py`
- MCP Client Library: `pip install mcp>=1.0.0`
