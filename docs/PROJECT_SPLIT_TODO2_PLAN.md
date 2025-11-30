# Project Split - Todo2 Implementation Plan

**Date**: 2025-11-20
**Purpose**: Todo2 tasks for executing project split strategy

**Based on**:

- [PROJECT_SPLIT_TRACTATUS_ANALYSIS.md](PROJECT_SPLIT_TRACTATUS_ANALYSIS.md) - Structural analysis
- [PROJECT_SPLIT_SEQUENTIAL_WORKFLOW.md](PROJECT_SPLIT_SEQUENTIAL_WORKFLOW.md) - Implementation workflow
- [PROJECT_SPLIT_STRATEGY.md](PROJECT_SPLIT_STRATEGY.md) - Overall strategy

---

## Todo2 Tasks Overview

### Phase 1: Foundation (Steps 1-2)

- T-300: Define public/private boundaries and audit dependencies
- T-301: Set up dependency management mechanism

### Phase 2: Quick Wins (Step 3) - Can execute in parallel

- T-302: Extract MCP servers repository
- T-303: Extract notebooks repository
- T-304: Extract build tools repository
- T-310: Extract project housekeeping tools repository

### Phase 3: Core Libraries (Steps 4-5) - Sequential

- T-305: Extract core C++ engine library
- T-306: Extract Python package

### Phase 4: Documentation Extraction (Step 6) - Can execute in parallel

- T-307: Extract trading-api-docs repository
- T-308: Extract trading-architecture-docs repository
- T-309: Extract trading-setup-docs repository
- T-310: Extract trading-automation-docs repository
- T-311: Extract trading-tools-docs repository

### Phase 5: Reorganization (Step 7)

- T-312: Reorganize private monorepo to use extracted libraries

### Phase 6: Optional Future Work (Step 8)

- T-313: Evaluate and optionally split private repos further

---

## Task Dependencies

```
T-300 (Define Boundaries)
    ↓
T-301 (Dependency Management)
    ↓
T-302 (MCP Servers) ─┐
T-303 (Notebooks)   ─┤
T-304 (Build Tools) ─┤ (Parallel)
T-310 (Housekeeping) ─┘
    ↓
T-305 (C++ Engine)
    ↓
T-306 (Python Package)
    ↓
T-307 (API Docs) ─┐
T-308 (Arch Docs) ─┤
T-309 (Setup Docs) ─┤ (Parallel)
T-310 (Auto Docs) ─┤
T-311 (Tools Docs) ─┘
    ↓
T-312 (Reorganize Private)
    ↓
T-313 (Optional Further Splits)
```

---

## Execution Strategy

1. **Start with T-300** (Define Boundaries) - Must be done first
2. **Then T-301** (Dependency Management) - Required for all extractions
3. **Execute T-302, T-303, T-304 in parallel** (Quick wins) - Independent of each other
4. **Sequentially execute T-305 → T-306** (Core libraries) - Python depends on C++
5. **Execute T-307 through T-311 in parallel** (5 documentation repos) - Independent of each other
6. **Execute T-312** (Reorganize private monorepo) - After all extractions complete
7. **T-313 is optional** - Only if further splitting is needed

---

## Success Metrics

- ✅ All boundaries clearly defined
- ✅ Dependency management working
- ✅ All quick wins extracted and published
- ✅ Core libraries extracted and published
- ✅ Private monorepo updated to use extracted libraries
- ✅ All tests passing
- ✅ All builds succeeding
- ✅ Documentation complete

---

## Next Steps

1. Create Todo2 tasks using this plan
2. Start with T-300 (Define Boundaries)
3. Execute tasks in dependency order
4. Track progress and update this document
