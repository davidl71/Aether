# Project Split - Public/Private Boundaries Definition

**Date**: 2025-11-22
**Task**: T-198
**Status**: ✅ Complete
**Purpose**: Define clear boundaries between public and private components for project split

---

## Executive Summary

This document establishes **enforceable boundaries** between public (open source) and private (proprietary) components. These boundaries ensure:

1. **No sensitive information** leaks to public repositories
2. **Public components** are truly reusable and broker-agnostic
3. **Dependencies** flow only in allowed directions (private → public, never public → private)
4. **Boundaries are enforceable** through automated checks

---

## Boundary Rules

### Rule 1: Public Components Must Be Broker-Agnostic

**Definition**: Public components cannot depend on any broker-specific code or APIs.

**Enforcement**:

- ✅ No `#include "tws_client.h"` in public headers
- ✅ No `import ibapi` in public Python code
- ✅ No broker API patterns in public code
- ✅ Abstract broker interfaces only

**Violations Found**:

- `native/include/box_spread_strategy.h` includes `tws_client.h` → **MUST FIX**
- `native/src/box_spread_strategy.cpp` references `tws::TWSClient*` → **MUST ABSTRACT**

---

### Rule 2: No Sensitive Information in Public Components

**Definition**: Public components cannot contain:

- API keys or secrets (even in examples)
- Account identifiers or patterns
- Trading strategies or risk parameters
- Internal research or insights
- Configuration patterns that reveal infrastructure

**Enforcement**:

- ✅ All config files use environment variables or placeholders
- ✅ No hardcoded credentials
- ✅ No account IDs in examples
- ✅ No trading strategy parameters

**Violations Found**:

- `config/lean_broker_config.example.json` has account patterns (`DU123456`, `U123456`) → **ACCEPTABLE** (examples only)
- `config/config.example.json` uses `${ENV_VAR}` pattern → **ACCEPTABLE** (correct pattern)

---

### Rule 3: Dependencies Flow One Direction Only

**Definition**:

- ✅ **ALLOWED**: Private → Public (private code can use public libraries)
- ❌ **FORBIDDEN**: Public → Private (public code cannot depend on private code)

**Enforcement**:

- Public repos have no dependencies on private repos
- Public repos are self-contained
- Private repos reference public repos via package managers

**Dependency Graph**:

```
Public Repos (Independent)
    ↑
    │ (used by)
    │
Private Repos (Depend on public)
```

---

## Public Component Inventory

### ✅ Safe for Public (No Sensitive Information)

#### 1. Core C++ Engine (`box-spread-cpp`)

**Location**: `native/include/box_spread_strategy.h`, `native/src/box_spread_strategy.cpp`

**Components**:

- Box spread calculation algorithms
- Risk calculation logic
- Option chain data structures
- Order management core logic

**Dependencies**:

- ✅ Standard library
- ✅ spdlog (logging)
- ✅ nlohmann/json (JSON parsing)
- ❌ **VIOLATION**: Includes `tws_client.h` → **MUST REMOVE**

**Action Required**:

1. Abstract broker interface (create `broker_interface.h`)
2. Remove `tws_client.h` dependency
3. Make strategy accept abstract broker interface

---

#### 2. Python Bindings (`box-spread-python`)

**Location**: `python/bindings/`, `python/dsl/`, `python/tools/`, `python/tui/`, `python/ml/`

**Components**:

- Cython bindings to C++ engine
- DSL for strategy definition
- TUI (as example)
- ML utilities

**Dependencies**:

- ✅ C++ engine (via bindings)
- ✅ Standard Python libraries
- ❌ **VIOLATION**: `python/integration/` contains broker-specific code → **EXCLUDE**

**Action Required**:

1. Exclude `python/integration/` from public package
2. Exclude `python/lean_integration/` from public package
3. Make TUI use abstract broker interface

---

#### 3. MCP Servers (`trading-mcp-servers`) ✅ **EXTRACTED**

**Location**: `mcp/trading_server/` → **Moved to**: https://github.com/davidl71/trading-mcp-servers

**Components**:

- MCP server implementations
- Generic trading tool integrations

**Dependencies**:

- ✅ MCP protocol libraries
- ✅ Standard Python libraries
- ✅ **AUDIT COMPLETE**: Broker-agnostic (TWS_HOST → BROKER_HOST, TWS_PORT → BROKER_PORT)

**Status**: ✅ **EXTRACTED** - Repository created, code pushed, broker-agnostic

---

#### 4. Notebooks (`box-spread-notebooks`) ✅ **EXTRACTED**

**Location**: `notebooks/` → **Moved to**: https://github.com/davidl71/box-spread-notebooks

**Components**:

- Educational Jupyter notebooks
- Example data
- Tutorial notebooks

**Dependencies**:

- ✅ Public Python package
- ✅ Standard data science libraries
- ✅ **AUDIT COMPLETE**: No private strategies, example data only

**Status**: ✅ **EXTRACTED** - Repository created, code pushed

---

#### 5. Build Tools (`trading-build-tools`) ✅ **EXTRACTED**

**Location**: `scripts/build_*.sh`, `CMakePresets.json` → **Moved to**: https://github.com/davidl71/trading-build-tools

**Components**:

- Generic build scripts
- CMake presets
- Cross-platform build automation

**Dependencies**:

- ✅ CMake, build tools
- ✅ **AUDIT COMPLETE**: Paths parameterized, project-agnostic

**Status**: ✅ **EXTRACTED** - Repository created, code pushed

---

#### 6. Project Housekeeping Tools (`trading-automation-tools`) ✅ **EXTRACTED**

**Location**: `scripts/base/intelligent_automation_base.py`, `scripts/automate_*.py`, `python/tools/project_analyzer.py` → **Moved to**: https://github.com/davidl71/trading-automation-tools

**Components**:

- IntelligentAutomationBase framework
- Generic automation scripts
- NetworkX project analyzer

**Dependencies**:

- ✅ Standard Python libraries
- ✅ NetworkX, MCP clients
- ✅ **AUDIT COMPLETE**: Framework project-agnostic, paths parameterized

**Status**: ✅ **EXTRACTED** - Repository created, code pushed

---

#### 7-11. Documentation Repos (`trading-*-docs`)

**Location**: `docs/` (selected files)

**Components**:

- API documentation
- Architecture guides
- Setup guides
- Automation guides
- Tools documentation

**Dependencies**:

- ✅ Markdown, documentation tools
- ❌ **VIOLATION**: May contain private research → **EXCLUDE `docs/research/`**

**Action Required**:

1. Exclude `docs/research/` from all public docs repos
2. Audit all docs for sensitive information
3. Remove internal design decisions

---

## Private Component Inventory

### 🔒 Must Stay Private (Contains Sensitive Information)

#### 1. Main Trading Application (`ib-box-spread-trading`)

**Location**: `native/src/ib_box_spread.cpp`, `native/src/tws_client.cpp`, `config/`, `ib-gateway/`

**Why Private**:

- Contains broker API integration code
- Contains trading strategies and risk parameters
- Contains account configuration patterns
- Contains internal architecture decisions

**Dependencies**:

- ✅ Uses `box-spread-cpp` (public library)
- ✅ Uses `box-spread-python` (public package)
- ✅ All broker-specific code

---

#### 2. Web Frontend (`ib-box-spread-web`)

**Location**: `web/`

**Why Private**:

- Connected to private trading backend
- May expose internal APIs
- Contains trading-specific UI patterns

**Dependencies**:

- ✅ Uses public libraries (React, TypeScript)
- ✅ Connects to private backend

---

#### 3. Mobile Apps (`ib-box-spread-mobile`)

**Location**: `ios/`, `desktop/`, `ondevice/`

**Why Private**:

- Platform-specific implementations
- Tied to private backend
- May contain app store keys

**Dependencies**:

- ✅ Uses platform SDKs
- ✅ Connects to private backend

---

#### 4. Research (`ib-box-spread-research`)

**Location**: `docs/research/`

**Why Private**:

- Contains trading insights
- Internal decision-making processes
- Competitive advantage information

**Dependencies**:

- ✅ References public documentation
- ✅ Contains private research

---

#### 5. Infrastructure (`ib-box-spread-infra`)

**Location**: `ansible/`, `roles/`, `playbooks/`

**Why Private**:

- Contains server information
- Deployment patterns
- Security configurations

**Dependencies**:

- ✅ Uses Ansible, deployment tools
- ✅ Contains private infrastructure configs

---

## Dependency Audit Results

### Code Dependencies

#### Public → Private Violations (FORBIDDEN)

1. **`native/include/box_spread_strategy.h`**
   - ❌ Includes `tws_client.h` (private)
   - **Fix**: Create abstract `broker_interface.h`, remove TWS dependency

2. **`native/src/box_spread_strategy.cpp`**
   - ❌ References `tws::TWSClient*` (private)
   - **Fix**: Use abstract broker interface

3. **`python/bindings/`**
   - ✅ No violations (only depends on C++ headers)

4. **`mcp/trading_server/`**
   - ⚠️ **AUDIT REQUIRED**: Check for broker-specific code

#### Private → Public Dependencies (ALLOWED)

1. **`native/src/ib_box_spread.cpp`**
   - ✅ Uses `box_spread_strategy.h` (will be public)
   - ✅ Uses `risk_calculator.h` (will be public)
   - ✅ Uses `order_manager.h` (core will be public)

2. **`python/integration/`**
   - ✅ Uses `python/bindings/` (will be public)
   - ✅ Uses `python/dsl/` (will be public)

---

### Documentation Dependencies

#### Public Documentation Boundaries

**Include in Public Docs**:

- ✅ Architecture guides (generic patterns)
- ✅ API documentation (generic APIs)
- ✅ Setup guides (generic setup)
- ✅ Integration guides (generic integrations)
- ✅ Tool usage guides

**Exclude from Public Docs**:

- ❌ `docs/research/` (all subdirectories)
- ❌ Private trading strategies
- ❌ Internal design decisions
- ❌ Broker-specific configurations
- ❌ Account patterns

---

### Configuration Dependencies

#### Public Configuration Patterns

**Safe Patterns**:

- ✅ Environment variables: `${ENV_VAR}`
- ✅ Placeholders: `YOUR_API_KEY`
- ✅ Example accounts: `DU123456` (paper trading example)
- ✅ Generic endpoints: `127.0.0.1:7497`

**Forbidden Patterns**:

- ❌ Real API keys or secrets
- ❌ Real account IDs
- ❌ Production endpoints
- ❌ Internal server addresses

**Current Status**:

- ✅ `config/config.example.json` uses `${ENV_VAR}` pattern → **SAFE**
- ✅ `config/lean_broker_config.example.json` uses example accounts → **SAFE**

---

## Migration Readiness Assessment

### ✅ Ready for Extraction (Low Risk)

1. **MCP Servers** - Minimal dependencies, mostly generic
2. **Notebooks** - Self-contained, need audit for sensitive data
3. **Build Tools** - Generic scripts, need path parameterization
4. **Project Housekeeping Tools** - Generic framework, need project-agnostic changes

### ⚠️ Requires Refactoring (Medium Risk)

1. **Core C++ Engine** - Must remove TWS dependency, create abstract interface
2. **Python Package** - Must exclude integration code, make broker-agnostic
3. **Documentation** - Must exclude research, audit for sensitive info

### 🔴 High Risk (Requires Significant Work)

1. **Main Trading Application** - Complex dependencies, stay private
2. **Web Frontend** - Tied to private backend, stay private
3. **Mobile Apps** - Platform-specific, stay private

---

## Enforcement Mechanisms

### Automated Checks

1. **Pre-commit Hook**: Check for TWS includes in public headers
2. **CI/CD**: Verify no public → private dependencies
3. **Documentation Audit**: Scan for sensitive patterns
4. **Configuration Audit**: Verify no hardcoded secrets

### Manual Reviews

1. **Code Review**: Verify broker-agnostic abstractions
2. **Documentation Review**: Verify no sensitive information
3. **Dependency Review**: Verify dependency direction

---

## Next Steps

1. ✅ **Boundaries Defined** (this document)
2. ⬜ **Fix Public → Private Violations** (T-198 follow-up)
3. ⬜ **Create Dependency Graph** (T-198 follow-up)
4. ⬜ **Set Up Dependency Management** (T-199)
5. ⬜ **Begin Extraction** (T-200+)

---

## Summary

**Public Components**: 11 repositories (6 code/tools, 5 documentation)
**Private Components**: 5 repositories (trading app, web, mobile, research, infra)
**Boundary Violations**: 3 code violations (TWS dependencies), 0 config violations
**Migration Readiness**: 4 ready, 3 need refactoring, 3 stay private

**Status**: ✅ Boundaries clearly defined and enforceable
