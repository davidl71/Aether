# Project Split Strategy - Sequential Thinking Workflow

**Date**: 2025-11-20
**Purpose**: Implementation workflow for project split based on Tractatus Thinking structural analysis

**Based on**: [PROJECT_SPLIT_TRACTATUS_ANALYSIS.md](PROJECT_SPLIT_TRACTATUS_ANALYSIS.md)

---

## Problem Statement

Convert the structural understanding from Tractatus analysis into actionable implementation steps for splitting `ib_box_spread_full_universal` monorepo into multiple focused projects.

---

## Sequential Implementation Workflow

### Step 1: Define Boundaries and Audit Dependencies

**Purpose**: Establish clear public/private boundaries and understand all dependencies

**Tasks**:
1. Create comprehensive boundary definition document
   - List all public components (no sensitive info)
   - List all private components (sensitive info)
   - Document what stays and what goes
2. Audit all code dependencies
   - Create dependency graph
   - Identify public → private dependencies (FORBIDDEN)
   - Identify private → public dependencies (ALLOWED)
   - Identify circular dependencies (FORBIDDEN)
3. Audit documentation dependencies
   - Identify docs that reference private code
   - Identify docs that can be public
   - Create doc migration plan
4. Audit configuration dependencies
   - Identify configs with sensitive info
   - Identify configs that can be public (examples)
   - Create config migration plan

**Output**:
- Boundary definition document
- Dependency graph
- Migration readiness assessment

**Dependencies**: None (can start immediately)

---

### Step 2: Set Up Dependency Management Mechanism

**Purpose**: Establish how split projects will reference each other

**Tasks**:
1. Choose dependency management approach
   - Option A: Git submodules (simple)
   - Option B: Package managers (Conan, PyPI, npm)
   - Option C: Monorepo tools (Nx, Turborepo)
   - Recommendation: Start with Option B for long-term
2. Set up package registry accounts
   - PyPI account for Python packages
   - Conan/vcpkg for C++ (if needed)
   - npm registry for JS (if needed)
3. Create dependency versioning strategy
   - Semantic versioning scheme
   - Version pinning rules
   - Update automation strategy
4. Create dependency documentation
   - How to add dependencies
   - How to update dependencies
   - Version compatibility matrix

**Output**:
- Dependency management configuration
- Registry accounts set up
- Versioning strategy documented

**Dependencies**: Step 1 (need to know dependencies before choosing mechanism)

---

### Step 3: Extract Quick Win Projects (Low Risk)

**Purpose**: Start with easiest extractions to build momentum

**Note**: These can be done in parallel for efficiency.

#### 3.1 Extract MCP Servers

**Tasks**:
1. Create `trading-mcp-servers` repository
   - Initialize with proper README
   - Set up MIT license
   - Add GitHub templates (issues, PRs)
2. Copy MCP server code
   - Copy `mcp/trading_server/`
   - Copy related documentation
   - Copy configuration examples
3. Remove broker-specific code
   - Make it generic
   - Add configuration for customization
4. Set up CI/CD
   - GitHub Actions for tests
   - Release automation
5. Publish and update main repo
   - Push to GitHub
   - Update main repo to reference it
   - Remove from main repo

**Output**: Working `trading-mcp-servers` repository

**Dependencies**: Steps 1-2 (boundaries defined, dependency mechanism ready)

---

#### 3.2 Extract Notebooks

**Tasks**:
1. Create `box-spread-notebooks` repository
   - Initialize with proper README
   - Set up MIT license
   - Add notebook structure
2. Copy public notebooks
   - Copy all notebooks from `notebooks/`
   - Remove any private strategies/data
   - Add example data files
3. Create notebook index
   - Document each notebook's purpose
   - Add usage examples
   - Create quickstart guide
4. Set up Jupyter environment
   - `requirements.txt` for dependencies
   - `environment.yml` for conda
   - Installation instructions
5. Publish and update main repo
   - Push to GitHub
   - Update main repo to reference it

**Output**: Working `box-spread-notebooks` repository

**Dependencies**: Steps 1-2

---

#### 3.3 Extract Build Tools

**Tasks**:
1. Create `trading-build-tools` repository
   - Initialize with proper README
   - Set up MIT license
   - Add structure
2. Copy build scripts
   - Copy generic build scripts
   - Copy CMake presets
   - Copy build automation scripts
3. Remove project-specific code
   - Parameterize hardcoded paths
   - Make scripts configurable
   - Add usage documentation
4. Create examples
   - Example projects using the tools
   - Integration guides
5. Publish and update main repo
   - Push to GitHub
   - Update main repo to use it

**Output**: Working `trading-build-tools` repository

**Dependencies**: Steps 1-2

---

#### 3.4 Extract Project Housekeeping Tools

**Tasks**:
1. Create `project-housekeeping-tools` repository
   - Initialize with proper README
   - Set up MIT license
   - Set up Python package structure
2. Extract automation framework
   - Copy `scripts/base/intelligent_automation_base.py`
   - Copy `scripts/base/mcp_client.py` (if generic)
   - Copy automation scripts (docs health, todo2 alignment, etc.)
   - Copy `python/tools/project_analyzer.py`
3. Remove project-specific code
   - Parameterize project paths
   - Make configs template-based
   - Remove trading-specific logic
4. Create PyPI package
   - Set up `pyproject.toml`
   - Create package structure
   - Add installation instructions
5. Create documentation
   - Framework usage guide
   - Example implementations
   - API reference
   - Integration patterns
6. Set up CI/CD
   - GitHub Actions
   - Test matrix
   - PyPI publishing
7. Publish to PyPI
   - Release v1.0.0
   - Update main repo to use it

**Output**: Working `project-housekeeping-tools` v1.0.0 PyPI package

**Dependencies**: Steps 1-2

**Why This is Valuable**:
- ✅ Generic automation framework
- ✅ IntelligentAutomationBase is innovative
- ✅ NetworkX integration for graph analysis
- ✅ Useful to ALL projects, not just trading
- ✅ Could be a popular open-source tool

---

### Step 4: Extract Core C++ Engine (Medium Risk)

**Purpose**: Extract the foundation library that other projects depend on

**Tasks**:
1. Create `box-spread-cpp` repository
   - Initialize with proper README
   - Set up MIT license
   - Set up CMake structure
2. Extract core engine code
   - Copy `native/include/box_spread_strategy.h`
   - Copy `native/src/box_spread_strategy.cpp`
   - Copy `native/include/risk_calculator.h`
   - Copy `native/src/risk_calculator.cpp`
   - Copy `native/include/option_chain.h`
   - Copy `native/src/option_chain.cpp`
   - Copy `native/include/order_manager.h` (core only)
3. Remove all broker-specific code
   - Remove TWS API dependencies
   - Make broker interface abstract
   - Remove configuration references
4. Extract and adapt tests
   - Copy relevant tests from `native/tests/`
   - Remove broker-specific tests
   - Add unit tests for public API
5. Create public API documentation
   - API reference
   - Usage examples
   - Architecture documentation
6. Set up packaging
   - CMake install targets
   - Conan recipe (optional)
   - vcpkg port (optional)
7. Set up CI/CD
   - GitHub Actions
   - Test matrix (multiple compilers)
   - Release automation
8. Publish v1.0.0
   - Tag release
   - Publish to package manager
   - Update main repo to use it

**Output**: Working `box-spread-cpp` v1.0.0 library

**Dependencies**: Steps 1-2, Step 3 (practice with quick wins first)

---

### Step 5: Extract Python Package (Medium Risk)

**Purpose**: Extract Python bindings and utilities

**Tasks**:
1. Create `box-spread-python` repository
   - Initialize with proper README
   - Set up MIT license
   - Set up Python package structure
2. Extract Python components
   - Copy `python/bindings/` (Cython)
   - Copy `python/dsl/` (Domain-Specific Language)
   - Copy `python/tools/` (Analysis tools)
   - Copy `python/tui/` (Terminal UI example)
   - Copy `python/ml/` (ML utilities)
3. Remove broker-specific code
   - Exclude `python/integration/` (stays private)
   - Exclude `python/lean_integration/` (stays private)
   - Make dependencies on public code only
4. Update to depend on `box-spread-cpp`
   - Reference published package
   - Update Cython bindings
   - Test integration
5. Create PyPI package
   - Set up `pyproject.toml`
   - Create package structure
   - Add build configuration
6. Create documentation
   - API reference
   - DSL documentation
   - Usage examples
   - TUI tutorial
7. Set up CI/CD
   - GitHub Actions
   - Test matrix (Python versions)
   - PyPI publishing
8. Publish to PyPI
   - Release v1.0.0
   - Update main repo to use it

**Output**: Working `box-spread-python` v1.0.0 PyPI package

**Dependencies**: Step 4 (needs `box-spread-cpp` published first)

---

### Step 6: Extract Public Documentation (Low Risk)

**Purpose**: Consolidate public documentation

**Tasks**:
1. Create `trading-docs` repository
   - Initialize with proper README
   - Set up MIT license
   - Set up documentation structure
2. Copy public documentation
   - Architecture guides
   - Integration guides
   - API documentation templates
   - Best practices
   - Tutorials
3. Remove private content
   - Remove internal research
   - Remove broker-specific strategies
   - Remove internal design decisions
4. Organize by topic
   - Architecture documentation
   - Integration guides
   - API references
   - Tutorials
5. Set up documentation site
   - GitHub Pages
   - MkDocs or Docusaurus
   - Search functionality
6. Publish and update references
   - Push to GitHub
   - Set up documentation site
   - Update all repos to reference it

**Output**: Working `trading-docs` repository with site

**Dependencies**: Steps 1-2 (boundaries defined)

---

### Step 7: Reorganize Private Monorepo (High Risk)

**Purpose**: Update main repo to use extracted libraries

**Tasks**:
1. Update dependency references
   - Replace local code with package references
   - Update CMakeLists.txt to use `box-spread-cpp`
   - Update Python requirements to use `box-spread-python`
   - Update references to MCP servers
2. Remove extracted code
   - Delete code that's now in public repos
   - Keep only private code
   - Update imports/references
3. Update build system
   - Update CMake configuration
   - Update Python setup
   - Test builds
4. Update CI/CD
   - Update test configuration
   - Update build scripts
   - Ensure all tests pass
5. Update documentation
   - Update README
   - Update internal docs
   - Document new structure
6. Test thoroughly
   - Run all tests
   - Test builds on all platforms
   - Test deployments
   - Verify functionality

**Output**: Updated private monorepo using public libraries

**Dependencies**: Steps 3-6 (all public libraries extracted)

---

### Step 8: Optional - Split Private Repos (Future)

**Purpose**: Further organize private code if needed

**Tasks**:
1. Evaluate if further splitting is needed
   - Review private repo size
   - Review team structure
   - Review deployment needs
2. If splitting web:
   - Create `ib-box-spread-web` repo
   - Extract web code
   - Set up independent deployment
3. If splitting mobile:
   - Create `ib-box-spread-mobile` repo
   - Extract iOS/macOS code
   - Set up independent deployment
4. If splitting research:
   - Create `ib-box-spread-research` repo
   - Extract research docs
   - Organize by topic
5. If splitting infrastructure:
   - Create `ib-box-spread-infra` repo
   - Extract deployment configs
   - Set up separate access controls

**Output**: Further organized private repos (if needed)

**Dependencies**: Step 7 (main repo stabilized first)

---

## Workflow Dependencies Graph

```
Step 1: Define Boundaries
    ↓
Step 2: Set Up Dependency Management
    ↓
Step 3: Extract Quick Wins (parallel)
    ├─ 3.1 MCP Servers
    ├─ 3.2 Notebooks
    └─ 3.3 Build Tools
    ↓
Step 4: Extract Core C++ Engine
    ↓
Step 5: Extract Python Package
    ↓
Step 6: Extract Public Docs
    ↓
Step 7: Reorganize Private Monorepo
    ↓
Step 8: Optional - Further Private Splits
```

---

## Critical Path

**Minimum viable path**:
1. Step 1 (Define Boundaries)
2. Step 2 (Dependency Management)
3. Step 4 (Core C++ Engine) - Blocks Step 5
4. Step 5 (Python Package)
5. Step 7 (Reorganize Private)

**Quick wins can happen in parallel** (Step 3) but don't block critical path.

---

## Success Criteria

### Per Step

1. **Boundaries Defined**: Complete boundary document, dependency graph
2. **Dependency Management**: Package managers set up, versioning strategy
3. **Quick Wins Extracted**: All three repos created and working
4. **Core Engine Extracted**: Library published and usable
5. **Python Package Extracted**: PyPI package published
6. **Docs Extracted**: Documentation site live
7. **Private Repo Updated**: All tests pass, using public libraries
8. **Optional Splits**: If done, all repos working independently

### Overall

- ✅ All public code extracted to focused repos
- ✅ All private code remains private
- ✅ No broken dependencies
- ✅ All tests pass
- ✅ All builds succeed
- ✅ Documentation is complete
- ✅ Migration is reversible (git history preserved)

---

## Risk Mitigation

### Risk: Extraction Breaks Main Repo

**Mitigation**:
- Extract incrementally
- Keep old code until new code proven
- Test thoroughly before removing
- Maintain git history for rollback

### Risk: Dependencies Break

**Mitigation**:
- Pin versions carefully
- Test dependency updates
- Maintain compatibility matrix
- Document breaking changes

### Risk: Too Much Work

**Mitigation**:
- Start with quick wins (Step 3)
- Do one extraction at a time
- Don't do Step 8 unless necessary
- Can pause between steps

---

## Next Step: Create Todo2 Plan

Convert this workflow into Todo2 tasks for tracking and execution.
