# PWA Missing Features Prioritization Plan

**Date:** 2025-11-23
**Purpose:** Prioritize and align missing PWA features (cash_flow, simulation, relationships) with investment strategy goals

---

## Executive Summary

**Missing Features Identified:**

1. **Cash Flow** - Not implemented (HIGH priority)
2. **Simulation** - Not implemented (HIGH priority)
3. **Relationships** - Not implemented (MEDIUM priority)

**Current Goal Alignment:** 23.1%
**Target Goal Alignment:** 80%+

---

## Feature Prioritization

### Phase 1: Cash Flow (HIGHEST PRIORITY) 🥇

**Why First:**

- Foundation for all other features
- Required for simulation scenarios
- Core requirement for opportunity analysis
- Enables liquidity planning

**Tasks:**

- **T-69**: Design future cash flow forecasting system ✅ (Research complete)
- **T-70**: Implement cash flow calculation for all asset types
- **T-71**: Integrate cash flow forecasting with backend and investment strategy
- **T-150**: Research cash flow calculation methods (supporting)
- **T-151**: Research cash flow forecasting integration patterns (supporting)

**Priority Order:**

1. T-70 (Implementation) - **Start immediately**
2. T-71 (Integration) - **After T-70**
3. T-150, T-151 (Research) - **Can run in parallel with T-70**

**Expected Impact:**

- Enables cash flow visibility
- Foundation for simulation
- Improves goal alignment by ~15%

---

### Phase 2: Simulation (HIGH PRIORITY) 🥈

**Why Second:**

- Core user value proposition
- Depends on cash flow (Phase 1)
- Enables what-if analysis
- Key differentiator for PWA

**Tasks:**

- **T-127**: Opportunity simulation engine

**Priority Order:**

1. T-127 (Simulation Engine) - **After T-71 complete**

**Expected Impact:**

- Enables what-if scenarios
- Core user workflow
- Improves goal alignment by ~25%

---

### Phase 3: Relationships (MEDIUM PRIORITY) 🥉

**Why Third:**

- Builds on cash flow and simulation
- Complex optimization feature
- Nice-to-have enhancement
- Lower immediate value

**Tasks:**

- **T-128**: Multi-instrument relationship modeling

**Priority Order:**

1. T-128 (Relationship Modeling) - **After T-127 complete**

**Expected Impact:**

- Enables complex optimization
- Advanced feature
- Improves goal alignment by ~10%

---

## Task Alignment Updates

### Tasks to Update with Goal Alignment Tags

**Cash Flow Tasks:**

- T-69, T-70, T-71 → Add `goal-aligned`, `pwa-core`, `primary-goal` tags
- T-150, T-151 → Add `goal-aligned`, `research` tags

**Simulation Tasks:**

- T-127 → Add `goal-aligned`, `pwa-core`, `primary-goal` tags

**Relationship Tasks:**

- T-128 → Add `goal-aligned`, `pwa-enhancement` tags

**Priority Updates:**

- All Phase 1 tasks: Ensure `high` priority
- T-127: Ensure `high` priority
- T-128: Can remain `medium` priority (builds on Phase 1 & 2)

---

## Implementation Roadmap

### Week 1-2: Cash Flow Foundation

- ✅ T-69: Design complete (research done)
- 🔄 T-70: Implement cash flow calculations
- ⏳ T-71: Integrate with backend

### Week 3-4: Simulation Engine

- ⏳ T-127: Opportunity simulation engine

### Week 5-6: Relationships (Optional)

- ⏳ T-128: Multi-instrument relationship modeling

---

## Success Metrics

**Goal Alignment Improvement:**

- **Current:** 23.1%
- **After Phase 1 (Cash Flow):** ~38% (+15%)
- **After Phase 2 (Simulation):** ~63% (+25%)
- **After Phase 3 (Relationships):** ~73% (+10%)
- **Target:** 80%+

**Feature Completion:**

- Phase 1: Cash flow fully implemented and integrated
- Phase 2: Simulation engine operational
- Phase 3: Relationship modeling functional

---

**Status:** Ready for implementation - tasks identified and prioritized
