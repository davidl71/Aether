# Bank Loan Tasks Parallelization Plan

**Date**: 2025-12-30
**Status**: Planning Complete
**Related Tasks**: T-74, T-76, T-77, T-152

## Executive Summary

**Total Tasks**: 4

- **T-74**: ✅ Design Complete (design document exists)
- **T-152**: Research (can start immediately, no dependencies)
- **T-76**: Implementation (depends on T-74, T-152)
- **T-77**: Entry Interface (depends on T-76)

**Parallelization Opportunities**: High

- **T-152**: Can run in parallel with T-76 preparation
- **T-76**: Can be split into 4 parallel sub-tasks
- **T-77**: Can be split into 3 parallel sub-tasks

**Estimated Total Time**: 12-16 hours (can be reduced to 6-8 hours with parallelization)

---

## Task Dependency Graph

```
T-74 (Design) ✅
    ↓
T-152 (Research) ──┐
    ↓              │
    └──────────────┼──> T-76 (Data Model)
                   │         ↓
                   │    T-77 (Entry Interface)
                   │
    (Can run in parallel)
```

---

## Phase 1: Research & Preparation (Parallel)

### ✅ T-74: Design Bank Loan Position System

**Status**: Complete

- Design document exists: `docs/research/architecture/BANK_LOAN_POSITION_SYSTEM_DESIGN.md`
- Data model defined
- Storage format specified
- Integration points identified

### 🔬 T-152: Research Bank Loan Position Data Models

**Status**: Can start immediately
**Dependencies**: None
**Estimated Time**: 2-3 hours
**Can Run In Parallel With**: T-76 preparation work

**Research Areas**:

1. **Data Model Patterns**:
   - Loan position structure best practices
   - SHIR-based vs CPI-linked loan modeling
   - Principal adjustment calculations
   - Interest rate calculation patterns

2. **Storage Patterns**:
   - JSON vs database trade-offs
   - Migration strategies
   - Versioning and schema evolution

3. **Integration Patterns**:
   - Portfolio value calculation with liabilities
   - Currency conversion (ILS → USD)
   - Cash flow integration

**Deliverables**:

- Research document with findings
- Recommendations for implementation
- Comparison of approaches

**Parallel Work**:

- Can research while T-76 components are being implemented
- Findings can inform T-76 implementation decisions

---

## Phase 2: Core Data Model (T-76) - Parallelizable

**Dependencies**: T-74 ✅, T-152 (can start before completion)
**Estimated Time**: 6-8 hours (can be 3-4 hours with parallelization)
**Status**: Ready to start

### T-76 Breakdown into Parallel Sub-Tasks

#### Sub-Task 1: LoanPosition Structure (Header)

**Files**: `native/include/ib_box_spread/loan_position.h`
**Dependencies**: T-74 ✅
**Estimated Time**: 1-2 hours
**Can Run In Parallel With**: Sub-tasks 2, 3, 4

**Work**:

- Define `LoanType` enum (SHIR_BASED, CPI_LINKED)
- Define `LoanStatus` enum (ACTIVE, PAID_OFF, DEFAULTED)
- Define `LoanPosition` struct with all fields
- Declare helper methods (get_adjusted_principal, get_current_interest_rate, etc.)

**Deliverable**: Complete header file with struct definition

---

#### Sub-Task 2: LoanPosition Implementation (Source)

**Files**: `native/src/loan_position.cpp`
**Dependencies**: Sub-task 1 (header must exist)
**Estimated Time**: 1-2 hours
**Can Run In Parallel With**: Sub-tasks 3, 4 (after Sub-task 1 complete)

**Work**:

- Implement `get_adjusted_principal()` (CPI adjustment logic)
- Implement `get_current_interest_rate()` (SHIR + spread calculation)
- Implement `get_usd_value()` (ILS → USD conversion)
- Implement `is_overdue()` (payment date checking)
- Implement `days_until_next_payment()` (date calculation)

**Deliverable**: Complete implementation file with all methods

---

#### Sub-Task 3: LoanManager Class (Storage)

**Files**:

- `native/include/ib_box_spread/loan_manager.h`
- `native/src/loan_manager.cpp`
**Dependencies**: Sub-task 1 (LoanPosition must exist)
**Estimated Time**: 2-3 hours
**Can Run In Parallel With**: Sub-task 4 (after Sub-task 1 complete)

**Work**:

- Design `LoanManager` class interface
- Implement JSON file loading (`load_from_json()`)
- Implement JSON file saving (`save_to_json()`)
- Implement CRUD operations:
  - `add_loan()`
  - `update_loan()`
  - `delete_loan()`
  - `get_loan()`
  - `get_all_loans()`
  - `get_active_loans()`
- Implement calculation methods:
  - `get_total_loan_liabilities_ils()`
  - `get_total_loan_liabilities_usd()`
  - `get_monthly_payment_total_ils()`
- Implement update methods:
  - `update_cpi_for_all_loans()`
  - `update_shir_for_all_loans()`
  - `refresh_loan_calculations()`

**Deliverable**: Complete LoanManager class with JSON storage

---

#### Sub-Task 4: Portfolio Calculator Integration

**Files**: `native/src/portfolio_calculator.cpp` (update existing)
**Dependencies**: Sub-task 1 (LoanPosition must exist), existing portfolio calculator
**Estimated Time**: 1-2 hours
**Can Run In Parallel With**: Sub-task 2, 3 (after Sub-task 1 complete)

**Work**:

- Update `calculate_net_portfolio_value()` to accept `LoanManager`
- Integrate loan liabilities into net portfolio calculation
- Add ILS → USD conversion for loan values
- Update portfolio allocation calculations to account for loan liabilities

**Deliverable**: Updated portfolio calculator with loan integration

---

### T-76 Parallel Execution Strategy

**Sequential Dependencies**:

1. **First**: Sub-task 1 (Header) - Must complete first
2. **Then (Parallel)**: Sub-tasks 2, 3, 4 can run simultaneously

**Timeline**:

- **Hour 0-2**: Sub-task 1 (Header) - Sequential
- **Hour 2-4**: Sub-tasks 2, 3, 4 - **PARALLEL** (3 developers/agents)
- **Total**: 4 hours (vs 6-8 hours sequential)

---

## Phase 3: Entry Interface (T-77) - Parallelizable

**Dependencies**: T-76 (all sub-tasks complete)
**Estimated Time**: 6-8 hours (can be 3-4 hours with parallelization)
**Status**: Blocked until T-76 complete

### T-77 Breakdown into Parallel Sub-Tasks

#### Sub-Task 1: TUI Manual Entry Form

**Files**:

- `native/include/ib_box_spread/loan_entry_ui.h`
- `native/src/loan_entry_ui.cpp`
**Dependencies**: T-76 ✅
**Estimated Time**: 2-3 hours
**Can Run In Parallel With**: Sub-tasks 2, 3

**Work**:

- Design TUI form using FTXUI
- Implement form fields:
  - Bank name (dropdown: Fibi, Discount, Other)
  - Account number (text input)
  - Loan type (dropdown: SHIR-based, CPI-linked)
  - Principal (numeric input)
  - Interest rate (numeric input)
  - Spread (numeric input, for SHIR-based)
  - Base CPI (numeric input, for CPI-linked)
  - Origination date (date picker)
  - Maturity date (date picker)
  - Monthly payment (numeric input)
  - Payment frequency (dropdown)
- Implement form validation
- Implement form submission to LoanManager

**Deliverable**: Complete TUI entry form

---

#### Sub-Task 2: CSV/JSON Import

**Files**:

- `native/include/ib_box_spread/loan_importer.h`
- `native/src/loan_importer.cpp`
**Dependencies**: T-76 ✅
**Estimated Time**: 2-3 hours
**Can Run In Parallel With**: Sub-tasks 1, 3

**Work**:

- Implement CSV parsing
  - Define CSV format/columns
  - Parse CSV file
  - Convert rows to LoanPosition objects
- Implement JSON parsing
  - Parse JSON file
  - Convert JSON objects to LoanPosition objects
- Implement validation
  - `validate_loan()` method
  - Error collection and reporting
- Implement bulk import
  - Import multiple loans at once
  - Handle errors gracefully

**Deliverable**: Complete import functionality

---

#### Sub-Task 3: Display & View Integration

**Files**:

- `native/src/tui_app.cpp` (update existing)
- `native/src/ib_box_spread.cpp` (update existing)
**Dependencies**: T-76 ✅
**Estimated Time**: 2-3 hours
**Can Run In Parallel With**: Sub-tasks 1, 2

**Work**:

- Add loan positions to portfolio view
- Display loan information in TUI
- Show loan liabilities in net portfolio value
- Add loan management menu/commands
- Integrate with existing position display

**Deliverable**: Complete display integration

---

### T-77 Parallel Execution Strategy

**All Sub-Tasks Can Run In Parallel** (after T-76 complete)

**Timeline**:

- **Hour 0-3**: Sub-tasks 1, 2, 3 - **PARALLEL** (3 developers/agents)
- **Total**: 3 hours (vs 6-8 hours sequential)

---

## Complete Parallelization Timeline

### Sequential Phases

```
Phase 1: Research (Parallel with Phase 2 prep)
├── T-152: Research (2-3 hours) ──┐
└── T-76 Prep: Review design (1 hour) ──┘
    └── Can run in parallel

Phase 2: Core Data Model (T-76)
├── Hour 0-2: Sub-task 1 (Header) - Sequential
└── Hour 2-4: Sub-tasks 2, 3, 4 - PARALLEL
    Total: 4 hours (vs 6-8 sequential)

Phase 3: Entry Interface (T-77)
└── Hour 0-3: Sub-tasks 1, 2, 3 - PARALLEL
    Total: 3 hours (vs 6-8 sequential)
```

### Total Time Comparison

| Approach | Time | Savings |
|----------|------|---------|
| **Sequential** | 12-16 hours | - |
| **Parallel** | 6-8 hours | **50% faster** |

---

## Parallel Work Assignment

### Option 1: Single Developer (Sequential)

- Complete tasks one at a time
- Total: 12-16 hours
- Simple, no coordination needed

### Option 2: Two Developers (Partial Parallel)

**Developer 1**:

- T-152 (Research) - 2-3 hours
- T-76 Sub-task 1 (Header) - 1-2 hours
- T-76 Sub-task 2 (Implementation) - 1-2 hours
- T-77 Sub-task 1 (TUI Form) - 2-3 hours

**Developer 2**:

- T-76 Sub-task 3 (LoanManager) - 2-3 hours
- T-76 Sub-task 4 (Portfolio Integration) - 1-2 hours
- T-77 Sub-task 2 (Import) - 2-3 hours
- T-77 Sub-task 3 (Display) - 2-3 hours

**Total**: ~6-8 hours (with coordination)

### Option 3: Three Developers (Maximum Parallel)

**Developer 1**:

- T-152 (Research) - 2-3 hours
- T-76 Sub-task 1 (Header) - 1-2 hours
- T-76 Sub-task 2 (Implementation) - 1-2 hours
- T-77 Sub-task 1 (TUI Form) - 2-3 hours

**Developer 2**:

- T-76 Sub-task 3 (LoanManager) - 2-3 hours
- T-77 Sub-task 2 (Import) - 2-3 hours

**Developer 3**:

- T-76 Sub-task 4 (Portfolio Integration) - 1-2 hours
- T-77 Sub-task 3 (Display) - 2-3 hours

**Total**: ~6-7 hours (with coordination)

---

## Critical Path

**Longest Path**: T-152 → T-76 Sub-task 1 → T-76 Sub-tasks 2/3/4 → T-77 Sub-tasks 1/2/3

**Bottlenecks**:

1. T-76 Sub-task 1 (Header) - Must complete before others
2. T-76 completion - Must complete before T-77

**Optimization**:

- Start T-152 immediately (no dependencies)
- Complete T-76 Sub-task 1 quickly
- Run T-76 Sub-tasks 2, 3, 4 in parallel
- Run T-77 Sub-tasks 1, 2, 3 in parallel

---

## Risk Mitigation

### Dependency Risks

- **Risk**: T-152 research might reveal design changes needed
- **Mitigation**: Start T-76 Sub-task 1 (header) based on existing design, update if needed

### Integration Risks

- **Risk**: Sub-tasks might have interface mismatches
- **Mitigation**:
  - Define interfaces clearly in Sub-task 1
  - Regular sync points between parallel workers
  - Integration testing after each phase

### Testing Strategy

- **Unit Tests**: Each sub-task should have tests
- **Integration Tests**: After T-76 complete, test with T-77
- **End-to-End Tests**: After T-77 complete, test full workflow

---

## Next Steps

1. **Immediate (Can Start Now)**:
   - ✅ Start T-152 (Research) - No dependencies
   - ✅ Review T-74 design document
   - ✅ Prepare T-76 Sub-task 1 (Header) structure

2. **After T-152 Complete**:
   - Start T-76 Sub-task 1 (Header)
   - Begin T-76 Sub-tasks 2, 3, 4 in parallel

3. **After T-76 Complete**:
   - Start T-77 Sub-tasks 1, 2, 3 in parallel

---

## Success Criteria

### T-76 Complete When

- ✅ LoanPosition struct defined and implemented
- ✅ LoanManager with JSON storage working
- ✅ Portfolio calculator integrates loan liabilities
- ✅ All unit tests passing

### T-77 Complete When

- ✅ TUI form allows manual loan entry
- ✅ CSV/JSON import works
- ✅ Loans display in portfolio view
- ✅ All integration tests passing

---

**Last Updated**: 2025-12-30
**Status**: Ready for Implementation
