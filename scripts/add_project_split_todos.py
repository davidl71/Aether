#!/usr/bin/env python3
"""
Add Project Split Todo2 Tasks

Creates Todo2 tasks for the project split strategy based on:
- PROJECT_SPLIT_TRACTATUS_ANALYSIS.md
- PROJECT_SPLIT_SEQUENTIAL_WORKFLOW.md
- PROJECT_SPLIT_TODO2_PLAN.md
"""

import json
import sys
from datetime import datetime
from pathlib import Path

# Project root
PROJECT_ROOT = Path(__file__).parent.parent
TODO2_PATH = PROJECT_ROOT / '.todo2' / 'state.todo2.json'

# Load existing todos
with open(TODO2_PATH, 'r') as f:
    data = json.load(f)

todos = data.get('todos', [])

# Find highest task number
max_task_num = max([int(t['id'][2:]) for t in todos if t['id'].startswith('T-') and t['id'][2:].isdigit()] or [0])

# Project split tasks
new_tasks = [
    {
        'id': f'T-{max_task_num + 1}',
        'name': 'Define public/private boundaries and audit dependencies',
        'long_description': '''🎯 **Objective:** Establish clear public/private boundaries and understand all dependencies for project split

📋 **Acceptance Criteria:**
- Comprehensive boundary definition document created
- Dependency graph generated (code, docs, configs)
- Public → private dependencies identified (FORBIDDEN)
- Private → public dependencies identified (ALLOWED)
- Circular dependencies identified (FORBIDDEN)
- Migration readiness assessment completed

🚫 **Scope Boundaries (CRITICAL):**
- **Included:** Analysis and documentation only
- **Excluded:** Actual code extraction, migration execution
- **Clarification Required:** None

🔧 **Technical Requirements:**
- Use codebase search to identify all dependencies
- Create dependency graph visualization
- Document boundary rules clearly
- Identify sensitive information patterns

📁 **Files/Components:**
- Create: docs/PROJECT_SPLIT_BOUNDARIES.md
- Create: docs/PROJECT_SPLIT_DEPENDENCY_GRAPH.md
- Analyze: All source code, documentation, configuration files

🧪 **Testing Requirements:**
- Verify no public → private dependencies exist
- Verify dependency graph is complete
- Validate boundary definitions are enforceable

⚠️ **Edge Cases:**
- Hidden dependencies in build scripts
- Documentation cross-references
- Configuration file dependencies
- Shared utility code

📚 **Dependencies:** None (foundation task)''',
        'status': 'Todo',
        'priority': 'high',
        'tags': ['project-split', 'foundation', 'analysis'],
        'dependencies': [],
        'created': datetime.now().isoformat(),
        'lastModified': datetime.now().isoformat(),
        'taskNumber': max_task_num + 1
    },
    {
        'id': f'T-{max_task_num + 2}',
        'name': 'Set up dependency management mechanism',
        'long_description': '''🎯 **Objective:** Establish how split projects will reference each other

📋 **Acceptance Criteria:**
- Dependency management approach chosen (Git submodules, package managers, or monorepo tools)
- Package registry accounts set up (PyPI, Conan/vcpkg, npm if needed)
- Semantic versioning strategy documented
- Dependency documentation created
- Version compatibility matrix established

🚫 **Scope Boundaries (CRITICAL):**
- **Included:** Setup and configuration only
- **Excluded:** Actual package publishing (happens in extraction tasks)
- **Clarification Required:** None

🔧 **Technical Requirements:**
- Choose between: Git submodules, package managers (recommended), or monorepo tools
- Set up PyPI account for Python packages
- Document versioning strategy
- Create dependency management guide

📁 **Files/Components:**
- Create: docs/PROJECT_SPLIT_DEPENDENCY_MANAGEMENT.md
- Create: Configuration templates
- Update: Build system documentation

🧪 **Testing Requirements:**
- Verify registry accounts are accessible
- Test dependency resolution
- Validate versioning scheme

⚠️ **Edge Cases:**
- Multiple package managers needed
- Cross-language dependencies
- Version compatibility issues

📚 **Dependencies:** T-{max_task_num + 1} (need boundaries defined first)''',
        'status': 'Todo',
        'priority': 'high',
        'tags': ['project-split', 'foundation', 'infrastructure'],
        'dependencies': [f'T-{max_task_num + 1}'],
        'created': datetime.now().isoformat(),
        'lastModified': datetime.now().isoformat(),
        'taskNumber': max_task_num + 2
    },
    {
        'id': f'T-{max_task_num + 3}',
        'name': 'Extract MCP servers repository',
        'long_description': '''🎯 **Objective:** Extract MCP servers to standalone public repository

📋 **Acceptance Criteria:**
- `trading-mcp-servers` repository created on GitHub
- MCP server code copied and broker-specific code removed
- MIT license added
- GitHub templates (issues, PRs) configured
- CI/CD set up (GitHub Actions)
- Repository published and main repo updated to reference it

🚫 **Scope Boundaries (CRITICAL):**
- **Included:** MCP server code, generic configuration examples
- **Excluded:** Broker-specific implementations, private trading logic
- **Clarification Required:** None

🔧 **Technical Requirements:**
- Make MCP servers generic/configurable
- Remove all broker-specific code
- Add comprehensive README
- Set up automated testing

📁 **Files/Components:**
- Extract: mcp/trading_server/
- Create: New repository structure
- Update: Main repo to reference new repo

🧪 **Testing Requirements:**
- Verify MCP servers work independently
- Test with example configurations
- Verify main repo still works after extraction

⚠️ **Edge Cases:**
- Configuration dependencies
- Shared utility code
- Documentation references

📚 **Dependencies:** T-{max_task_num + 1}, T-{max_task_num + 2} (boundaries and dependency management)''',
        'status': 'Todo',
        'priority': 'high',
        'tags': ['project-split', 'quick-win', 'extraction'],
        'dependencies': [f'T-{max_task_num + 1}', f'T-{max_task_num + 2}'],
        'created': datetime.now().isoformat(),
        'lastModified': datetime.now().isoformat(),
        'taskNumber': max_task_num + 3
    },
    {
        'id': f'T-{max_task_num + 4}',
        'name': 'Extract notebooks repository',
        'long_description': '''🎯 **Objective:** Extract educational notebooks to standalone public repository

📋 **Acceptance Criteria:**
- `box-spread-notebooks` repository created
- All public notebooks copied (private strategies/data removed)
- Notebook index created with usage examples
- Jupyter environment setup (requirements.txt, environment.yml)
- Repository published and main repo updated

🚫 **Scope Boundaries (CRITICAL):**
- **Included:** Educational notebooks, example data, templates
- **Excluded:** Private trading strategies, personal research data, account information
- **Clarification Required:** None

🔧 **Technical Requirements:**
- Remove any sensitive data
- Add example data files
- Create comprehensive README
- Document notebook purposes

📁 **Files/Components:**
- Extract: notebooks/ directory
- Create: Notebook index and quickstart guide
- Create: Environment setup files

🧪 **Testing Requirements:**
- Verify notebooks run with example data
- Test environment setup
- Verify no sensitive information leaked

⚠️ **Edge Cases:**
- Hidden sensitive data in notebooks
- Large data files
- Complex dependencies

📚 **Dependencies:** T-{max_task_num + 1}, T-{max_task_num + 2}''',
        'status': 'Todo',
        'priority': 'high',
        'tags': ['project-split', 'quick-win', 'extraction'],
        'dependencies': [f'T-{max_task_num + 1}', f'T-{max_task_num + 2}'],
        'created': datetime.now().isoformat(),
        'lastModified': datetime.now().isoformat(),
        'taskNumber': max_task_num + 4
    },
    {
        'id': f'T-{max_task_num + 5}',
        'name': 'Extract build tools repository',
        'long_description': '''🎯 **Objective:** Extract reusable build scripts to standalone public repository

📋 **Acceptance Criteria:**
- `trading-build-tools` repository created
- Generic build scripts copied and parameterized
- CMake presets extracted
- Usage documentation created
- Example projects added
- Repository published

🚫 **Scope Boundaries (CRITICAL):**
- **Included:** Generic build scripts, CMake presets, automation patterns
- **Excluded:** Project-specific paths, broker-specific build configs
- **Clarification Required:** None

🔧 **Technical Requirements:**
- Parameterize hardcoded paths
- Make scripts configurable
- Remove project-specific code
- Add comprehensive examples

📁 **Files/Components:**
- Extract: scripts/build_*.sh, CMake presets
- Create: Usage documentation
- Create: Example projects

🧪 **Testing Requirements:**
- Verify scripts work with example projects
- Test configuration options
- Verify main repo still builds

⚠️ **Edge Cases:**
- Platform-specific build logic
- Complex dependency chains
- Configuration file dependencies

📚 **Dependencies:** T-{max_task_num + 1}, T-{max_task_num + 2}''',
        'status': 'Todo',
        'priority': 'high',
        'tags': ['project-split', 'quick-win', 'extraction'],
        'dependencies': [f'T-{max_task_num + 1}', f'T-{max_task_num + 2}'],
        'created': datetime.now().isoformat(),
        'lastModified': datetime.now().isoformat(),
        'taskNumber': max_task_num + 5
    },
    {
        'id': f'T-{max_task_num + 6}',
        'name': 'Extract project housekeeping tools repository',
        'long_description': '''🎯 **Objective:** Extract generic project maintenance automation tools to standalone public repository

📋 **Acceptance Criteria:**
- `project-housekeeping-tools` repository created
- IntelligentAutomationBase framework extracted
- All automation scripts copied and parameterized
- Project analyzer (NetworkX) extracted
- PyPI package created and published
- Comprehensive documentation added

🚫 **Scope Boundaries (CRITICAL):**
- **Included:** Generic automation framework, reusable scripts, NetworkX analyzer
- **Excluded:** Project-specific paths, trading-specific logic, private configurations
- **Clarification Required:** None

🔧 **Technical Requirements:**
- Make framework project-agnostic
- Parameterize all project paths
- Create configuration templates
- Set up PyPI packaging
- Add comprehensive examples

📁 **Files/Components:**
- Extract: scripts/base/intelligent_automation_base.py
- Extract: scripts/automate_*.py (generic ones)
- Extract: python/tools/project_analyzer.py
- Create: PyPI package structure
- Create: Documentation and examples

🧪 **Testing Requirements:**
- Verify framework works with example projects
- Test all automation scripts
- Verify PyPI package installs correctly
- Test NetworkX analyzer

⚠️ **Edge Cases:**
- MCP client dependencies
- Project-specific assumptions
- Configuration file formats

📚 **Dependencies:** T-{max_task_num + 1}, T-{max_task_num + 2}''',
        'status': 'Todo',
        'priority': 'high',
        'tags': ['project-split', 'quick-win', 'extraction', 'automation'],
        'dependencies': [f'T-{max_task_num + 1}', f'T-{max_task_num + 2}'],
        'created': datetime.now().isoformat(),
        'lastModified': datetime.now().isoformat(),
        'taskNumber': max_task_num + 6
    },
    {
        'id': f'T-{max_task_num + 7}',
        'name': 'Extract core C++ engine library',
        'long_description': '''🎯 **Objective:** Extract broker-agnostic C++ trading engine to standalone public library

📋 **Acceptance Criteria:**
- `box-spread-cpp` repository created
- Core engine code extracted (no broker dependencies)
- All broker-specific code removed
- Public API documentation created
- Tests extracted and adapted
- CMake packaging set up
- CI/CD configured
- v1.0.0 published to package manager

🚫 **Scope Boundaries (CRITICAL):**
- **Included:** Core algorithms, risk calculations, option chain logic (broker-agnostic)
- **Excluded:** TWS API code, broker-specific implementations, configuration patterns
- **Clarification Required:** None

🔧 **Technical Requirements:**
- Remove all TWS API dependencies
- Make broker interface abstract
- Create clean public API
- Set up CMake install targets
- Configure CI/CD (multiple compilers)

📁 **Files/Components:**
- Extract: native/include/box_spread_strategy.h, native/src/box_spread_strategy.cpp
- Extract: native/include/risk_calculator.h, native/src/risk_calculator.cpp
- Extract: native/include/option_chain.h, native/src/option_chain.cpp
- Extract: native/include/order_manager.h (core only)
- Extract: Relevant tests from native/tests/

🧪 **Testing Requirements:**
- All unit tests pass
- Public API is well-tested
- No broker dependencies remain
- Library builds on multiple platforms

⚠️ **Edge Cases:**
- Hidden broker dependencies
- Shared utility code
- Test dependencies

📚 **Dependencies:** T-{max_task_num + 1}, T-{max_task_num + 2}, T-{max_task_num + 3} (practice with quick wins first)''',
        'status': 'Todo',
        'priority': 'high',
        'tags': ['project-split', 'core-library', 'extraction'],
        'dependencies': [f'T-{max_task_num + 1}', f'T-{max_task_num + 2}', f'T-{max_task_num + 3}'],
        'created': datetime.now().isoformat(),
        'lastModified': datetime.now().isoformat(),
        'taskNumber': max_task_num + 7
    },
    {
        'id': f'T-{max_task_num + 8}',
        'name': 'Extract Python package',
        'long_description': '''🎯 **Objective:** Extract Python bindings and utilities to standalone PyPI package

📋 **Acceptance Criteria:**
- `box-spread-python` repository created
- Python components extracted (bindings, DSL, tools, TUI, ML)
- Broker-specific code excluded
- Updated to depend on published box-spread-cpp
- PyPI package created and published
- Comprehensive documentation added
- CI/CD configured

🚫 **Scope Boundaries (CRITICAL):**
- **Included:** Python bindings, DSL, tools, TUI (as example), ML utilities
- **Excluded:** python/integration/ (broker-specific), python/lean_integration/ (QuantConnect-specific)
- **Clarification Required:** None

🔧 **Technical Requirements:**
- Update Cython bindings to use published box-spread-cpp
- Remove broker-specific dependencies
- Set up pyproject.toml
- Configure PyPI publishing
- Create comprehensive documentation

📁 **Files/Components:**
- Extract: python/bindings/, python/dsl/, python/tools/, python/tui/, python/ml/
- Create: PyPI package structure
- Update: Dependencies to use box-spread-cpp package

🧪 **Testing Requirements:**
- All Python tests pass
- Package installs correctly
- DSL examples work
- TUI runs with example data

⚠️ **Edge Cases:**
- Cython binding dependencies
- Native library linking
- Platform-specific issues

📚 **Dependencies:** T-{max_task_num + 7} (needs box-spread-cpp published first)''',
        'status': 'Todo',
        'priority': 'high',
        'tags': ['project-split', 'core-library', 'extraction'],
        'dependencies': [f'T-{max_task_num + 7}'],
        'created': datetime.now().isoformat(),
        'lastModified': datetime.now().isoformat(),
        'taskNumber': max_task_num + 8
    },
    {
        'id': f'T-{max_task_num + 9}',
        'name': 'Extract trading-api-docs repository',
        'long_description': '''🎯 **Objective:** Extract API documentation and integration guides to standalone repository

📋 **Acceptance Criteria:**
- `trading-api-docs` repository created
- API_DOCUMENTATION_INDEX.md extracted (chunked if >50 sources)
- API_DOCUMENTATION_SUMMARY.md extracted
- Integration guides extracted
- API indices (FIX, Market Data, etc.) extracted
- Documentation site set up
- Repository published

🚫 **Scope Boundaries (CRITICAL):**
- **Included:** API documentation, integration guides, API decision trees, broker API patterns (generic)
- **Excluded:** Architecture docs, setup guides, private research, broker-specific strategies
- **Clarification Required:** None

🔧 **Technical Requirements:**
- Chunk API_DOCUMENTATION_INDEX.md if needed (2,611 lines)
- Organize by API category
- Set up documentation site
- Create navigation structure

📁 **Files/Components:**
- Extract: API_DOCUMENTATION_INDEX.md, API_DOCUMENTATION_SUMMARY.md
- Extract: docs/indices/ (FIX, Market Data, Trading Simulators, Quantitative Finance)
- Extract: Integration guides from docs/research/integration/ (public ones)
- Create: Documentation site structure

🧪 **Testing Requirements:**
- All links work
- Documentation site builds
- No private information leaked
- Fits in one NotebookLM notebook (<50 sources)

⚠️ **Edge Cases:**
- API_INDEX might need chunking
- Cross-reference updates
- External API links

📚 **Dependencies:** T-{max_task_num + 1}, T-{max_task_num + 2}''',
        'status': 'Todo',
        'priority': 'high',
        'tags': ['project-split', 'documentation', 'extraction', 'api'],
        'dependencies': [f'T-{max_task_num + 1}', f'T-{max_task_num + 2}'],
        'created': datetime.now().isoformat(),
        'lastModified': datetime.now().isoformat(),
        'taskNumber': max_task_num + 9
    },
    {
        'id': f'T-{max_task_num + 10}',
        'name': 'Extract trading-architecture-docs repository',
        'long_description': '''🎯 **Objective:** Extract architecture and design documentation to standalone repository

📋 **Acceptance Criteria:**
- `trading-architecture-docs` repository created
- Architecture documentation extracted
- Design pattern documents extracted
- System design guides extracted
- Documentation site set up
- Repository published

🚫 **Scope Boundaries (CRITICAL):**
- **Included:** Architecture docs, design patterns, system design, multi-language architecture
- **Excluded:** Setup guides, API docs, implementation guides, private research
- **Clarification Required:** None

🔧 **Technical Requirements:**
- Organize by architecture topic
- Set up documentation site
- Create navigation structure
- Cross-reference with other docs repos

📁 **Files/Components:**
- Extract: ARCHITECTURE_DOCUMENTATION_OPTIONS.md
- Extract: MULTI_LANGUAGE_ARCHITECTURE.md
- Extract: MULTI_BROKER_ARCHITECTURE_DESIGN.md
- Extract: LEAN_STRATEGY_ARCHITECTURE.md
- Extract: ECLIENT_EWRAPPER_ARCHITECTURE.md
- Extract: Architecture guides from docs/research/architecture/ (public ones)
- Create: Documentation site structure

🧪 **Testing Requirements:**
- All links work
- Documentation site builds
- No private information leaked
- Fits in one NotebookLM notebook (<50 sources)

⚠️ **Edge Cases:**
- Cross-reference updates
- Design decision dependencies
- Architecture diagram assets

📚 **Dependencies:** T-{max_task_num + 1}, T-{max_task_num + 2}''',
        'status': 'Todo',
        'priority': 'high',
        'tags': ['project-split', 'documentation', 'extraction', 'architecture'],
        'dependencies': [f'T-{max_task_num + 1}', f'T-{max_task_num + 2}'],
        'created': datetime.now().isoformat(),
        'lastModified': datetime.now().isoformat(),
        'taskNumber': max_task_num + 10
    },
    {
        'id': f'T-{max_task_num + 11}',
        'name': 'Extract trading-setup-docs repository',
        'long_description': '''🎯 **Objective:** Extract setup, configuration, and deployment documentation to standalone repository

📋 **Acceptance Criteria:**
- `trading-setup-docs` repository created
- Setup guides extracted
- Configuration documentation extracted
- Deployment guides extracted
- Documentation site set up
- Repository published

🚫 **Scope Boundaries (CRITICAL):**
- **Included:** Setup guides, configuration docs, deployment guides, platform settings
- **Excluded:** Architecture docs, API docs, automation guides, private configs
- **Clarification Required:** None

🔧 **Technical Requirements:**
- Organize by setup topic
- Set up documentation site
- Create navigation structure
- Include configuration templates

📁 **Files/Components:**
- Extract: DEPLOYMENT_GUIDE.md
- Extract: PYTHON_ENVIRONMENT_SETUP.md
- Extract: CURSOR_SETUP.md, CURSOR_GLOBAL_DOCS_SETUP.md
- Extract: NATS_SETUP.md, TWS_BUILD_PROGRESS.md
- Extract: AUTOMATED_SETUP.md, PLATFORM_SPECIFIC_SETTINGS.md
- Extract: Setup guides from docs/research/integration/ (public setup docs)
- Create: Documentation site structure

🧪 **Testing Requirements:**
- All links work
- Documentation site builds
- No private information leaked
- Fits in one NotebookLM notebook (<50 sources)

⚠️ **Edge Cases:**
- Configuration template dependencies
- Platform-specific instructions
- External tool dependencies

📚 **Dependencies:** T-{max_task_num + 1}, T-{max_task_num + 2}''',
        'status': 'Todo',
        'priority': 'high',
        'tags': ['project-split', 'documentation', 'extraction', 'setup'],
        'dependencies': [f'T-{max_task_num + 1}', f'T-{max_task_num + 2}'],
        'created': datetime.now().isoformat(),
        'lastModified': datetime.now().isoformat(),
        'taskNumber': max_task_num + 11
    },
    {
        'id': f'T-{max_task_num + 12}',
        'name': 'Extract trading-automation-docs repository',
        'long_description': '''🎯 **Objective:** Extract automation and maintenance documentation to standalone repository

📋 **Acceptance Criteria:**
- `trading-automation-docs` repository created
- Automation guides extracted
- Maintenance workflow documentation extracted
- Health monitoring docs extracted
- Documentation site set up
- Repository published

🚫 **Scope Boundaries (CRITICAL):**
- **Included:** Automation guides, maintenance workflows, health monitoring, automation patterns
- **Excluded:** Setup guides, architecture docs, tool usage guides, private automation configs
- **Clarification Required:** None

🔧 **Technical Requirements:**
- Organize by automation topic
- Set up documentation site
- Create navigation structure
- Cross-reference with project-housekeeping-tools repo

📁 **Files/Components:**
- Extract: INTELLIGENT_AUTOMATION_GUIDE.md
- Extract: ROUTINE_AUTOMATION_PLAN.md
- Extract: DOCUMENTATION_HEALTH_AUTOMATION.md
- Extract: TODO2_ALIGNMENT_AUTOMATION.md
- Extract: PWA_REVIEW_AUTOMATION.md
- Extract: DEPENDENCY_SECURITY_AUTOMATION.md
- Extract: Automation guides and patterns
- Create: Documentation site structure

🧪 **Testing Requirements:**
- All links work
- Documentation site builds
- No private information leaked
- Fits in one NotebookLM notebook (<50 sources)

⚠️ **Edge Cases:**
- Automation script dependencies
- Configuration file references
- Cross-repo references

📚 **Dependencies:** T-{max_task_num + 1}, T-{max_task_num + 2}''',
        'status': 'Todo',
        'priority': 'high',
        'tags': ['project-split', 'documentation', 'extraction', 'automation'],
        'dependencies': [f'T-{max_task_num + 1}', f'T-{max_task_num + 2}'],
        'created': datetime.now().isoformat(),
        'lastModified': datetime.now().isoformat(),
        'taskNumber': max_task_num + 12
    },
    {
        'id': f'T-{max_task_num + 13}',
        'name': 'Extract trading-tools-docs repository',
        'long_description': '''🎯 **Objective:** Extract tools, frameworks, and best practices documentation to standalone repository

📋 **Acceptance Criteria:**
- `trading-tools-docs` repository created
- Tool usage guides extracted
- Framework documentation extracted
- Best practices documents extracted
- Documentation site set up
- Repository published

🚫 **Scope Boundaries (CRITICAL):**
- **Included:** Tool usage guides, framework docs, best practices, MCP/NotebookLM guides
- **Excluded:** Setup guides, architecture docs, API docs, private tool configs
- **Clarification Required:** None

🔧 **Technical Requirements:**
- Organize by tool/framework topic
- Set up documentation site
- Create navigation structure
- Include usage examples

📁 **Files/Components:**
- Extract: MCP_QUICK_REFERENCE.md, MCP_TRADING_SERVER_COMPLETE.md
- Extract: NOTEBOOKLM_SETUP_GUIDE.md, NOTEBOOKLM_USAGE.md
- Extract: CURSOR_AI_TUTORIAL.md, CURSOR_RECOMMENDATIONS.md
- Extract: AGENTIC_TOOLS_USAGE.md, AGENTIC_TOOLS_WORKFLOW_EXAMPLES.md
- Extract: Tool-specific guides (TWS, NATS, etc.)
- Extract: Best practices documents
- Create: Documentation site structure

🧪 **Testing Requirements:**
- All links work
- Documentation site builds
- No private information leaked
- Fits in one NotebookLM notebook (<50 sources)

⚠️ **Edge Cases:**
- Tool version dependencies
- External tool documentation links
- Framework-specific examples

📚 **Dependencies:** T-{max_task_num + 1}, T-{max_task_num + 2}''',
        'status': 'Todo',
        'priority': 'high',
        'tags': ['project-split', 'documentation', 'extraction', 'tools'],
        'dependencies': [f'T-{max_task_num + 1}', f'T-{max_task_num + 2}'],
        'created': datetime.now().isoformat(),
        'lastModified': datetime.now().isoformat(),
        'taskNumber': max_task_num + 13
    },
    {
        'id': f'T-{max_task_num + 14}',
        'name': 'Reorganize private monorepo to use extracted libraries',
        'long_description': '''🎯 **Objective:** Update main private repository to use extracted public libraries

📋 **Acceptance Criteria:**
- All dependency references updated to use published packages
- Extracted code removed from main repo
- Build system updated (CMake, Python setup)
- All tests pass
- CI/CD updated and working
- Documentation updated

🚫 **Scope Boundaries (CRITICAL):**
- **Included:** Dependency updates, code removal, build system updates
- **Excluded:** Further private repo splits (optional future work)
- **Clarification Required:** None

🔧 **Technical Requirements:**
- Update CMakeLists.txt to use box-spread-cpp package
- Update Python requirements to use box-spread-python package
- Update references to MCP servers, notebooks, build tools, docs repos
- Remove extracted code
- Update CI/CD configuration

📁 **Files/Components:**
- Update: CMakeLists.txt, pyproject.toml, package.json
- Remove: Extracted code directories
- Update: Build scripts, CI/CD configs
- Update: README and documentation

🧪 **Testing Requirements:**
- All builds succeed
- All tests pass
- No broken dependencies
- Functionality preserved

⚠️ **Edge Cases:**
- Circular dependency issues
- Version compatibility
- Build system complexity
- Migration rollback needs

📚 **Dependencies:** T-{max_task_num + 3}, T-{max_task_num + 4}, T-{max_task_num + 5}, T-{max_task_num + 6}, T-{max_task_num + 7}, T-{max_task_num + 8}, T-{max_task_num + 9}, T-{max_task_num + 10}, T-{max_task_num + 11}, T-{max_task_num + 12}, T-{max_task_num + 13} (all extractions complete)''',
        'status': 'Todo',
        'priority': 'high',
        'tags': ['project-split', 'reorganization', 'migration'],
        'dependencies': [f'T-{max_task_num + 3}', f'T-{max_task_num + 4}', f'T-{max_task_num + 5}', f'T-{max_task_num + 6}', f'T-{max_task_num + 7}', f'T-{max_task_num + 8}', f'T-{max_task_num + 9}', f'T-{max_task_num + 10}', f'T-{max_task_num + 11}', f'T-{max_task_num + 12}', f'T-{max_task_num + 13}'],
        'created': datetime.now().isoformat(),
        'lastModified': datetime.now().isoformat(),
        'taskNumber': max_task_num + 14
    },
    {
        'id': f'T-{max_task_num + 15}',
        'name': 'Evaluate and optionally split private repos further',
        'long_description': '''🎯 **Objective:** Evaluate if further splitting of private repositories is needed

📋 **Acceptance Criteria:**
- Private repo size and complexity evaluated
- Team structure reviewed
- Deployment needs assessed
- Decision made on further splits
- If splitting: Web, mobile, research, or infra repos created

🚫 **Scope Boundaries (CRITICAL):**
- **Included:** Evaluation and optional extraction
- **Excluded:** Required splits (already planned)
- **Clarification Required:** Depends on evaluation results

🔧 **Technical Requirements:**
- Review private repo size
- Assess team structure
- Evaluate deployment needs
- Make informed decision

📁 **Files/Components:**
- Evaluate: Current private repo structure
- Optionally create: ib-box-spread-web, ib-box-spread-mobile, ib-box-spread-research, ib-box-spread-infra

🧪 **Testing Requirements:**
- If splitting: All repos work independently
- If not splitting: Current structure is manageable

⚠️ **Edge Cases:**
- Team size changes
- Deployment complexity
- Maintenance overhead

📚 **Dependencies:** T-{max_task_num + 14} (main repo reorganized first)''',
        'status': 'Todo',
        'priority': 'low',
        'tags': ['project-split', 'optional', 'evaluation'],
        'dependencies': [f'T-{max_task_num + 14}'],
        'created': datetime.now().isoformat(),
        'lastModified': datetime.now().isoformat(),
        'taskNumber': max_task_num + 15
    }
]

# Add new tasks
for task in new_tasks:
    # Fix dependency references
    if 'dependencies' in task:
        task['dependencies'] = [dep.replace(f'T-{max_task_num + 1}', task['id']) if 'T-' in str(dep) else dep for dep in task['dependencies']]
        # Fix the actual IDs
        deps = []
        for dep in task['dependencies']:
            if isinstance(dep, str) and dep.startswith('T-'):
                # Find the corresponding task
                dep_num = int(dep[2:])
                if dep_num <= max_task_num + 10:
                    deps.append(f'T-{dep_num}')
            else:
                deps.append(dep)
        task['dependencies'] = deps

    todos.append(task)

# Update data
data['todos'] = todos

# Save
with open(TODO2_PATH, 'w') as f:
    json.dump(data, f, indent=2)

print(f"✅ Added {len(new_tasks)} project split tasks to Todo2")
print(f"Task IDs: {', '.join([t['id'] for t in new_tasks])}")
