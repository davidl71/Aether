# Parallel Execution Plan for TUI/PWA Alignment Tasks

**Date**: 2025-11-20
**Purpose**: Identify parallel execution opportunities for TUI/PWA alignment tasks

---

## Executive Summary

**Key Parallel Opportunities:**

1. **T-203 (PWA) + T-204 (TUI)** - Unified positions panels can be built simultaneously
2. **Backend API work** - Can proceed in parallel with frontend work
3. **Component development** - Multiple components can be built in parallel within each task
4. **Testing** - Can be done in parallel with development

---

## Task Dependency Graph

```
T-203 (PWA Unified Positions)
  ↓
T-205 (PWA Cash Flow)
  ↓
T-207 (PWA Simulation)
  ↓
T-209 (PWA Relationships)

T-204 (TUI Unified Positions)
  ↓
T-206 (TUI Cash Flow)
  ↓
T-208 (TUI Simulation)
  ↓
T-210 (TUI Relationships)
```

**Key Insight**: PWA and TUI tracks are **completely independent** - can be worked on in parallel!

---

## Parallel Execution Opportunities

### 1. Phase 1: Foundation (HIGH PRIORITY)

#### ✅ **FULLY PARALLEL: T-203 + T-204**

**T-203: Unified Positions Panel in PWA**

- **Can run in parallel with**: T-204
- **Parallel work within task**:
  - Component development (UnifiedPositionsPanel.tsx)
  - Hook development (useSnapshot.ts updates)
  - Type definitions (snapshot.ts updates)
  - Testing (component tests, integration tests)

**T-204: Unified Positions Panel in TUI**

- **Can run in parallel with**: T-203
- **Parallel work within task**:
  - Component development (unified_positions.py)
  - Provider updates (providers.py)
  - Model updates (models.py)
  - Testing (component tests, integration tests)

**Backend API Work (Can run in parallel with T-203/T-204)**

- Extend existing `/api/v1/snapshot` endpoint
- Add unified positions aggregation logic
- Add NATS topics for real-time updates
- **No blocking dependencies** - frontend can use mock data initially

---

#### ✅ **FULLY PARALLEL: T-205 + T-206** (After T-203/T-204 complete)

**T-205: Cash Flow Modeling in PWA**

- **Can run in parallel with**: T-206
- **Parallel work within task**:
  - CashFlowPanel.tsx component
  - CashFlowChart.tsx component (chart library integration)
  - useCashFlow.ts hook
  - Type definitions
  - Testing

**T-206: Cash Flow Modeling in TUI**

- **Can run in parallel with**: T-205
- **Parallel work within task**:
  - cash_flow_panel.py component
  - cash_flow_chart.py (text-based chart rendering)
  - cash_flow.py hook
  - Model updates
  - Testing

**Backend Cash Flow Engine (Can run in parallel with T-205/T-206)**

- Implement CashFlowCalculator (Rust backend)
- Add `/api/v1/cash-flow/timeline` endpoint
- Add `/api/v1/cash-flow/projection` endpoint
- Database schema for cash_flow_events table
- **Frontend can use mock data initially**

---

### 2. Phase 2: Simulation (HIGH PRIORITY)

#### ✅ **FULLY PARALLEL: T-207 + T-208** (After T-205/T-206 complete)

**T-207: Opportunity Simulation in PWA**

- **Can run in parallel with**: T-208
- **Parallel work within task**:
  - SimulationPanel.tsx (main interface)
  - ScenarioComparison.tsx (side-by-side comparison)
  - ScenarioBuilder.tsx (form builder)
  - useSimulation.ts hook
  - Testing

**T-208: Basic Simulation in TUI**

- **Can run in parallel with**: T-207
- **Parallel work within task**:
  - simulation_panel.py component
  - scenario_builder.py component
  - simulation.py hook
  - Testing

**Backend Simulation Engine (Can run in parallel with T-207/T-208)**

- Implement SimulationEngine (Rust backend)
- Add `/api/v1/simulation/run` endpoint
- Add `/api/v1/simulation/scenarios` endpoint
- **Frontend can use mock calculations initially**

---

### 3. Phase 3: Relationships (MEDIUM/LOW PRIORITY)

#### ✅ **FULLY PARALLEL: T-209 + T-210** (After T-207/T-208 complete)

**T-209: Relationship Visualization in PWA**

- **Can run in parallel with**: T-210
- **Parallel work within task**:
  - RelationshipGraph.tsx (graph visualization)
  - OptimizationChains.tsx (chain display)
  - useRelationships.ts hook
  - Graph library integration (React Flow/D3.js)
  - Testing

**T-210: Basic Relationship Visualization in TUI**

- **Can run in parallel with**: T-209
- **Parallel work within task**:
  - relationship_graph.py (text-based graph)
  - optimization_chains.py component
  - relationships.py hook
  - Testing

**Backend Relationship Engine (Can run in parallel with T-209/T-210)**

- Implement AssetRelationshipGraph (Rust backend)
- Add `/api/v1/relationships/graph` endpoint
- Add `/api/v1/relationships/chains` endpoint
- **Frontend can use mock data initially**

---

## Cross-Task Parallel Work

### Backend Development (Independent Track)

**Can run in parallel with ALL frontend tasks:**

1. **Unified Positions Backend** (Week 1-2)
   - Extend snapshot aggregation
   - Add position type support (pension loans, bonds, T-bills)
   - NATS integration for real-time updates
   - **No blocking dependencies**

2. **Cash Flow Engine** (Week 2-3)
   - CashFlowCalculator implementation
   - Database schema for cash_flow_events
   - API endpoints
   - **Can start after unified positions backend**

3. **Simulation Engine** (Week 3-4)
   - SimulationEngine implementation
   - Scenario calculation logic
   - API endpoints
   - **Can start after cash flow engine**

4. **Relationship Engine** (Week 4-5)
   - AssetRelationshipGraph implementation
   - Optimization chain algorithms
   - API endpoints
   - **Can start after simulation engine**

---

## Component-Level Parallel Work

### Within Each Task, Multiple Components Can Be Built in Parallel

#### Example: T-203 (PWA Unified Positions)

**Parallel Component Development:**

1. **UnifiedPositionsPanel.tsx** (main component)
2. **PositionGroup.tsx** (grouping by instrument type)
3. **PositionRow.tsx** (individual position row)
4. **PositionFilters.tsx** (filtering UI)
5. **useUnifiedPositions.ts** (data hook)

**Parallel Testing:**

- Unit tests for each component
- Integration tests for data flow
- E2E tests for user workflows

---

## Recommended Parallel Execution Strategy

### Week 1-2: Foundation (Maximum Parallelism)

**Team 1: PWA Track**

- T-203: Unified Positions Panel in PWA
- Components: UnifiedPositionsPanel, PositionGroup, PositionRow, Filters
- **Can work in parallel**: All components + testing

**Team 2: TUI Track**

- T-204: Unified Positions Panel in TUI
- Components: unified_positions.py, position_group.py, position_row.py
- **Can work in parallel**: All components + testing

**Team 3: Backend Track**

- Unified Positions Backend API
- Extend snapshot endpoint
- NATS integration
- **Can work in parallel**: API + NATS + testing

**Result**: 3 teams working in parallel, no blocking dependencies!

---

### Week 3-4: Cash Flow (Maximum Parallelism)

**Team 1: PWA Track**

- T-205: Cash Flow Modeling in PWA
- Components: CashFlowPanel, CashFlowChart, useCashFlow
- **Can work in parallel**: All components + testing

**Team 2: TUI Track**

- T-206: Cash Flow Modeling in TUI
- Components: cash_flow_panel.py, cash_flow_chart.py
- **Can work in parallel**: All components + testing

**Team 3: Backend Track**

- Cash Flow Engine
- CashFlowCalculator, API endpoints, database schema
- **Can work in parallel**: Engine + API + database + testing

**Result**: 3 teams working in parallel!

---

### Week 5-6: Simulation (Maximum Parallelism)

**Team 1: PWA Track**

- T-207: Opportunity Simulation in PWA
- Components: SimulationPanel, ScenarioComparison, ScenarioBuilder
- **Can work in parallel**: All components + testing

**Team 2: TUI Track**

- T-208: Basic Simulation in TUI
- Components: simulation_panel.py, scenario_builder.py
- **Can work in parallel**: All components + testing

**Team 3: Backend Track**

- Simulation Engine
- SimulationEngine, API endpoints
- **Can work in parallel**: Engine + API + testing

**Result**: 3 teams working in parallel!

---

### Week 7-8: Relationships (Maximum Parallelism)

**Team 1: PWA Track**

- T-209: Relationship Visualization in PWA
- Components: RelationshipGraph, OptimizationChains
- **Can work in parallel**: All components + testing

**Team 2: TUI Track**

- T-210: Basic Relationship Visualization in TUI
- Components: relationship_graph.py, optimization_chains.py
- **Can work in parallel**: All components + testing

**Team 3: Backend Track**

- Relationship Engine
- AssetRelationshipGraph, API endpoints
- **Can work in parallel**: Engine + API + testing

**Result**: 3 teams working in parallel!

---

## Critical Path Analysis

### Sequential Dependencies (Cannot Be Parallelized)

1. **T-203 → T-205 → T-207 → T-209** (PWA track)
2. **T-204 → T-206 → T-208 → T-210** (TUI track)
3. **Backend: Unified Positions → Cash Flow → Simulation → Relationships**

### Parallel Opportunities (Can Be Parallelized)

1. **T-203 || T-204** ✅ (PWA and TUI unified positions)
2. **T-205 || T-206** ✅ (PWA and TUI cash flow)
3. **T-207 || T-208** ✅ (PWA and TUI simulation)
4. **T-209 || T-210** ✅ (PWA and TUI relationships)
5. **Frontend || Backend** ✅ (All frontend tasks can use mock data initially)
6. **Components within task** ✅ (Multiple components can be built in parallel)

---

## Resource Allocation Recommendations

### Optimal Team Structure (3 Teams)

**Team 1: PWA Frontend**

- Focus: T-203, T-205, T-207, T-209
- Skills: React/TypeScript, UI/UX, chart libraries
- **Can work independently** (no blocking dependencies)

**Team 2: TUI Frontend**

- Focus: T-204, T-206, T-208, T-210
- Skills: Python/Textual, terminal UI design
- **Can work independently** (no blocking dependencies)

**Team 3: Backend**

- Focus: API extensions, engines, NATS integration
- Skills: Rust, API design, database
- **Can work independently** (frontend uses mock data initially)

### Single Developer Strategy

**If working alone, recommended order:**

1. **Week 1**: T-203 (PWA unified positions) - Foundation for PWA track
2. **Week 2**: T-204 (TUI unified positions) - Foundation for TUI track
3. **Week 3**: T-205 (PWA cash flow) - Builds on T-203
4. **Week 4**: T-206 (TUI cash flow) - Builds on T-204
5. **Week 5**: T-207 (PWA simulation) - Builds on T-205
6. **Week 6**: T-208 (TUI simulation) - Builds on T-206
7. **Week 7**: T-209 (PWA relationships) - Builds on T-207
8. **Week 8**: T-210 (TUI relationships) - Builds on T-208

**Backend work**: Can be done in parallel with frontend work (use mock data initially)

---

## Risk Mitigation

### Mock Data Strategy

**All frontend tasks can use mock data initially:**

- T-203/T-204: Mock unified positions data
- T-205/T-206: Mock cash flow data
- T-207/T-208: Mock simulation results
- T-209/T-210: Mock relationship graphs

**Benefits:**

- Frontend development not blocked by backend
- Backend can be developed in parallel
- Integration can happen later
- Faster iteration cycles

### Integration Points

**Define clear API contracts early:**

- Unified positions API contract (for T-203/T-204)
- Cash flow API contract (for T-205/T-206)
- Simulation API contract (for T-207/T-208)
- Relationship API contract (for T-209/T-210)

**This allows:**

- Frontend and backend teams to work independently
- Mock data matches real API structure
- Easy integration when backend is ready

---

## Summary

### Maximum Parallelism Opportunities

1. ✅ **T-203 || T-204** - Unified positions (PWA + TUI)
2. ✅ **T-205 || T-206** - Cash flow (PWA + TUI)
3. ✅ **T-207 || T-208** - Simulation (PWA + TUI)
4. ✅ **T-209 || T-210** - Relationships (PWA + TUI)
5. ✅ **Frontend || Backend** - All tasks (use mock data)
6. ✅ **Components within task** - Multiple components per task

### Estimated Time Savings

**Sequential approach**: ~16 weeks (2 weeks per task × 8 tasks)
**Parallel approach**: ~8 weeks (2 weeks per phase × 4 phases)

**Time savings**: 50% reduction with 3-team parallel execution!

---

*This plan enables maximum parallel execution while respecting task dependencies.*
