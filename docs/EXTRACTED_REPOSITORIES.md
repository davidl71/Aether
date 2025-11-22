# Extracted Repositories

**Date**: 2025-11-20
**Status**: Quick-win extractions complete

This document tracks repositories that have been extracted from the main `ib_box_spread_full_universal` monorepo as part of the project split strategy.

## ✅ Completed Extractions

### 1. trading-mcp-servers
- **Repository**: https://github.com/davidl71/trading-mcp-servers
- **Status**: Private, code pushed
- **Description**: MCP servers for trading operations - broker-agnostic
- **Components Extracted**:
  - `mcp/trading_server/` - Trading MCP server implementation
  - Broker-agnostic REST API bridge
  - PyPI package configuration
- **Files**: 9 files (Python package, README, LICENSE, CI/CD)
- **NotebookLM**: ✅ Single notebook, <10 sources

### 2. box-spread-notebooks
- **Repository**: https://github.com/davidl71/box-spread-notebooks
- **Status**: Private, code pushed
- **Description**: Educational Jupyter notebooks for box spread trading strategies
- **Components Extracted**:
  - `notebooks/` - All Jupyter notebooks
  - Notebook utilities and helpers
  - Example data loaders
- **Files**: 15 files (5 notebooks + utilities + documentation)
- **NotebookLM**: ✅ Entire repo in one notebook

### 3. trading-build-tools
- **Repository**: https://github.com/davidl71/trading-build-tools
- **Status**: Private, code pushed
- **Description**: Reusable CMake build scripts and presets for C++ trading projects
- **Components Extracted**:
  - `scripts/build_*.sh` - Build automation scripts
  - `cmake-presets/CMakePresets.json` - CMake presets
  - Universal binary, fast build, distributed build scripts
- **Files**: 8 files (build scripts + CMake presets)
- **NotebookLM**: ✅ Single focused notebook

### 4. trading-automation-tools
- **Repository**: https://github.com/davidl71/trading-automation-tools
- **Status**: Private, code pushed
- **Description**: Reusable project housekeeping and analysis automation tools
- **Components Extracted**:
  - `scripts/base/intelligent_automation_base.py` - Base automation framework
  - `scripts/automate_*.py` - All automation scripts
  - `python/tools/project_analyzer.py` - NetworkX project analyzer
  - Configuration templates and cron setup scripts
- **Files**: 24 files (automation scripts + base classes + configs)
- **NotebookLM**: ✅ Single focused notebook

### 5. box-spread-cpp
- **Repository**: https://github.com/davidl71/box-spread-cpp
- **Status**: Private, code pushed
- **Description**: Broker-agnostic C++ library for box spread arbitrage calculations and risk management
- **Components Extracted**:
  - `native/include/box_spread/` - Core library headers (broker-agnostic)
  - `native/src/` - Core library implementation
  - Abstract broker interface (`IBroker`)
  - Generic configuration structures
  - Risk calculator, option chain, order manager, strategy
- **Files**: 24 files (headers + sources + CMake + examples + tests)
- **NotebookLM**: ✅ Single focused notebook
- **Key Achievement**: All broker-specific code removed (TWSClient → IBroker interface)

### 6. box-spread-python
- **Repository**: https://github.com/davidl71/box-spread-python
- **Status**: Private, code pushed
- **Description**: Broker-agnostic Python utilities for box spread trading strategies
- **Components Extracted**:
  - `python/bindings/` - Cython bindings for C++ library
  - `python/dsl/` - Domain-Specific Language for strategies
  - `python/tools/` - Analysis and utility tools
  - `python/tui/` - Terminal UI example
  - `python/ml/` - Machine learning utilities
- **Files**: 27+ Python files + package configuration
- **NotebookLM**: ✅ Single focused notebook
- **Key Achievement**: All broker-specific code excluded (integration/, lean_integration/ stay private)

### 7. trading-api-docs
- **Repository**: https://github.com/davidl71/trading-api-docs
- **Status**: Private, code pushed
- **Description**: Trading API documentation and integration guides
- **Components Extracted**:
  - API documentation index and summaries
  - FIX protocol documentation
  - Market data API documentation
  - Integration guides
- **Files**: API-related documentation files
- **NotebookLM**: ✅ Single focused notebook

### 8. trading-architecture-docs
- **Repository**: https://github.com/davidl71/trading-architecture-docs
- **Status**: Private, code pushed
- **Description**: Trading system architecture and design documentation
- **Components Extracted**:
  - System architecture documentation
  - Design patterns and best practices
  - Component relationship diagrams
  - Integration patterns
- **Files**: Architecture-related documentation files
- **NotebookLM**: ✅ Single focused notebook

### 9. trading-setup-docs
- **Repository**: https://github.com/davidl71/trading-setup-docs
- **Status**: Private, code pushed
- **Description**: Trading system setup, configuration, and deployment documentation
- **Components Extracted**:
  - Setup and installation guides
  - Configuration documentation
  - Deployment guides
  - Platform-specific settings
  - Environment setup instructions
- **Files**: Setup-related documentation files
- **NotebookLM**: ✅ Single focused notebook

### 10. trading-automation-docs
- **Repository**: https://github.com/davidl71/trading-automation-docs
- **Status**: Private, code pushed
- **Description**: Trading project automation and maintenance documentation
- **Components Extracted**:
  - Automation guides and patterns
  - Maintenance workflows
  - Health monitoring documentation
  - Repository management guides
  - Dependency security automation
- **Files**: Automation-related documentation files
- **NotebookLM**: ✅ Single focused notebook

### 11. trading-tools-docs
- **Repository**: https://github.com/davidl71/trading-tools-docs
- **Status**: Private, code pushed
- **Description**: Trading tools, frameworks, and best practices documentation
- **Components Extracted**:
  - MCP (Model Context Protocol) guides
  - NotebookLM setup and usage
  - Cursor AI integration guides
  - Agentic tools documentation
  - Framework usage guides
  - Best practices
- **Files**: Tools-related documentation files
- **NotebookLM**: ✅ Single focused notebook

## 📋 Pending Extractions

### Documentation Repositories (Future)
- **T-208**: Extract trading-api-docs repository
- **T-209**: Extract trading-architecture-docs repository
- **T-210**: Extract trading-setup-docs repository
- **T-211**: Extract trading-automation-docs repository
- **T-212**: Extract trading-tools-docs repository

## 🔗 Integration

### Using Extracted Repositories

#### Option 1: Git Submodules
```bash
# In main repository
git submodule add https://github.com/davidl71/trading-build-tools.git libs/trading-build-tools
git submodule add https://github.com/davidl71/trading-mcp-servers.git libs/trading-mcp-servers
```

#### Option 2: Package Managers
- **Python**: Install from PyPI (when published)
  ```bash
  pip install trading-mcp-servers
  ```
- **C++**: Use Conan or vcpkg (when published)

#### Option 3: Direct Clone
```bash
# Clone and reference in build system
git clone https://github.com/davidl71/trading-build-tools.git
# Update CMakeLists.txt to reference cloned location
```

## 📝 Migration Notes

### What Changed in Main Repo
- MCP server code moved to `trading-mcp-servers`
- Notebooks moved to `box-spread-notebooks`
- Build scripts moved to `trading-build-tools`
- Automation tools moved to `trading-automation-tools`

### What Stayed in Main Repo
- Core trading engine (C++ and Python)
- Broker-specific integrations
- Web frontend
- Mobile apps
- Private research and documentation
- Infrastructure automation

## 🎯 Next Steps

1. **Review extracted repositories** on GitHub
2. **Make repositories public** when ready (Settings → Danger Zone → Change visibility)
3. **Create v1.0.0 releases** for each repository
4. **Update main repo** to reference extracted repositories (submodules or packages)
5. **Continue with core library extractions** (T-204, T-205)

## 📚 Related Documentation

- [Project Split Strategy](PROJECT_SPLIT_STRATEGY.md) - Overall split strategy and rationale
- [Project Split Boundaries](PROJECT_SPLIT_BOUNDARIES.md) - Public/private boundaries and dependency audit
- [Project Split Sequential Workflow](PROJECT_SPLIT_SEQUENTIAL_WORKFLOW.md) - Implementation workflow
- [Project Split Dependency Management](PROJECT_SPLIT_DEPENDENCY_MANAGEMENT.md) - Dependency management strategy
