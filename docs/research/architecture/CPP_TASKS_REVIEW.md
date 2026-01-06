# C++ Tasks Review - Migration Impact Analysis

**Date**: 2025-01-04
**Status**: Review Complete
**Purpose**: Identify C++ tasks that need updates due to Python migration

---

## Executive Summary

**Total C++ Related Tasks**: 77
**Completed**: 70
**Active/In Progress**: 7
**Needs Update**: 1 task (T-232)

---

## Tasks Requiring Updates

### T-232: Implement portfolio Greeks aggregation with currency conversion

**Status**: Todo
**Priority**: High 🟠
**Issue**: References C++ `RiskCalculator` and `GreeksCalculator` which are being migrated to Python

**Current Description**:
- Mentions `RiskCalculator` (C++ class)
- References `GreeksCalculator from T-67` (C++ implementation)
- Implementation location not explicitly specified

**Required Updates**:
1. ✅ **Update to use Python implementations**:
   - Change `RiskCalculator` → `python.integration.risk_calculator`
   - Change `GreeksCalculator` → `python.integration.greeks_calculator`
   - Update file paths to Python files

2. ✅ **Update dependencies**:
   - Add dependency on T-239 (Risk calculator migration)
   - Add dependency on T-240 (Greeks calculator migration)
   - Keep existing dependencies (T-66, T-67, T-149)

3. ✅ **Update technical requirements**:
   - Use NumPy for aggregation calculations
   - Use Python dataclasses for data structures
   - Follow Python code style

**Action**: Update task description and dependencies to reflect Python migration

---

## Tasks That Are OK (No Changes Needed)

### T-237: Create stub broker adapters for Alpaca and IB Client Portal
- **Status**: In Progress
- **Reason**: Broker adapters stay in C++ (required for TWS API)
- **Action**: None needed

### T-35: Implement Alpaca API adapter
- **Status**: Todo
- **Reason**: Broker adapter stays in C++ (can be migrated later, but not required)
- **Action**: None needed

### T-36: Implement IB Client Portal API adapter
- **Status**: Todo
- **Reason**: Broker adapter stays in C++ (can be migrated later, but not required)
- **Action**: None needed

### T-37: Implement broker selection and switching mechanism
- **Status**: Todo
- **Reason**: Uses broker adapters (C++), but business logic can be Python
- **Action**: None needed (hybrid approach)

### T-229: Implement BrokerManager and BrokerFactory classes
- **Status**: Todo
- **Reason**: Uses broker adapters (C++), but business logic can be Python
- **Action**: None needed (hybrid approach)

### T-230: Implement broker switching mechanism and runtime selection
- **Status**: Todo
- **Reason**: Uses broker adapters (C++), but business logic can be Python
- **Action**: None needed (hybrid approach)

---

## Migration Tasks (Newly Created)

These tasks are the migration work itself - they're correct as-is:

- **T-238**: Migrate box spread strategy from C++ to Python ✅
- **T-239**: Migrate risk calculator from C++ to Python ✅
- **T-240**: Migrate Greeks calculator from C++ to Python ✅
- **T-241**: Migrate order management logic from C++ to Python ✅
- **T-242**: Migrate loan management from C++ to Python ✅

---

## Completed C++ Tasks (No Action Needed)

**70 completed tasks** - These are historical and don't need updates:
- TWS client improvements (T-4, T-5, T-6, T-7, T-8, T-10)
- Integration tests (T-6, T-7, T-8)
- QuantLib integration (T-96)
- Eigen integration (T-97)
- NLopt integration (T-98)
- And many more...

**Note**: Completed tasks are historical records and don't need updates.

---

## Recommendations

### Immediate Actions

1. **Update T-232**:
   - Modify task description to use Python implementations
   - Add dependencies on T-239 and T-240
   - Update file paths and technical requirements

2. **Monitor Migration Progress**:
   - As T-239 and T-240 complete, verify T-232 can proceed
   - Ensure Python implementations match C++ API for compatibility

3. **Review Dependencies**:
   - Check if T-233 (depends on T-232) needs similar updates
   - Verify other tasks that depend on migrated components

### Long-term Considerations

- **Broker Adapters**: Keep in C++ (required for TWS API)
- **Business Logic**: Migrate to Python (easier maintenance)
- **Hybrid Approach**: Maintain C++ for adapters, Python for logic
- **Testing**: Update tests to use Python implementations where applicable

---

## Related Documentation

- [Native to Python Migration Analysis](./NATIVE_TO_PYTHON_MIGRATION_ANALYSIS.md) - Full migration strategy
- [Native Migration Tasks](./NATIVE_MIGRATION_TASKS.md) - Migration task definitions

---

**Last Updated**: 2025-01-04
**Status**: Review Complete - 1 Task Needs Update
