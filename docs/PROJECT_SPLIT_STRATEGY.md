# Project Split Strategy

**Date**: 2025-11-20
**Purpose**: Recommendations for splitting the sprawling `ib_box_spread_full_universal` repository into multiple focused projects, considering:
- Public vs private boundaries
- Reuse potential for other projects
- **NotebookLM analysis efficiency** (one notebook per project)

---

## Executive Summary

**Key Insight**: Project splits optimize for **NotebookLM analysis** - each split project can be analyzed in one focused notebook (staying within 50 source limit).

The current monorepo contains **16+ distinct components** that would benefit from separation:

1. **Core Trading Engine** (C++) - **PUBLIC** - Reusable library
2. **Python Integration Layer** - **PUBLIC** - Standalone package
3. **Rust Backend Services** - **PRIVATE** - Your trading infrastructure
4. **Web Frontend** (TypeScript/React) - **PRIVATE** - Your trading UI
5. **Desktop/iOS Apps** - **PRIVATE** - Platform-specific clients
6. **MCP Servers** - **PUBLIC** - Useful for other projects
7. **Project Housekeeping Tools** - **PUBLIC** - Generic maintenance automation
8. **Documentation** - **PUBLIC** (split into 5 focused repos) + **PRIVATE** (research)
9. **Notebooks** - **PUBLIC** - Educational examples
10. **Build Automation** - **PUBLIC** - Reusable scripts
11. **Agent Framework** - **PRIVATE** - Your multi-agent system

---

## Recommended Project Split

### 🟢 PUBLIC Projects (Open Source)

#### 1. `box-spread-cpp` - Core C++ Trading Engine
**Visibility**: Public
**License**: MIT
**Purpose**: Reusable box spread calculation library

**Components**:
- `native/include/box_spread_strategy.h`
- `native/src/box_spread_strategy.cpp`
- `native/include/risk_calculator.h`
- `native/src/risk_calculator.cpp`
- `native/include/option_chain.h`
- `native/src/option_chain.cpp`
- `native/include/order_manager.h` (core logic only)
- `native/tests/` (strategy tests)

**Dependencies**:
- spdlog, nlohmann/json, CLI11, Catch2
- **NO TWS API** (make broker-agnostic)

**Benefits**:
- ✅ Useful to other developers building options trading systems
- ✅ Can be used with any broker API
- ✅ Core algorithms are broker-independent
- ✅ Good portfolio piece showing C++20 expertise

**Considerations**:
- Remove all broker-specific code
- Make it a header-only or static library
- Add comprehensive documentation
- Publish to Conan/vcpkg package managers

---

#### 2. `box-spread-python` - Python Integration Package
**Visibility**: Public
**License**: MIT
**Purpose**: Python bindings and high-level trading utilities

**Components**:
- `python/bindings/` (Cython bindings)
- `python/dsl/` (Domain-Specific Language for strategies)
- `python/tools/` (Analysis tools)
- `python/tui/` (Terminal UI - useful example)
- `python/ml/` (Machine learning utilities)
- `python/setup.py`, `python/pyproject.toml`

**Dependencies**:
- Depends on `box-spread-cpp` (as package or submodule)
- pandas, numpy, networkx, textual

**Benefits**:
- ✅ PyPI package that others can install
- ✅ Useful for Python trading developers
- ✅ DSL is innovative and valuable
- ✅ TUI can inspire other developers

**Exclude**:
- `python/integration/` (broker-specific integrations stay private)
- `python/lean_integration/` (QuantConnect-specific)
- Broker API clients

---

#### 3. `trading-mcp-servers` - MCP Servers for Trading
**Visibility**: Public
**License**: MIT
**Purpose**: Model Context Protocol servers for trading systems

**Components**:
- `mcp/trading_server/`
- Any other MCP server implementations

**Benefits**:
- ✅ MCP is becoming popular in AI coding assistants
- ✅ Useful for other developers building trading tools
- ✅ Small, focused, easy to maintain

---

#### 4. `box-spread-notebooks` - Educational Jupyter Notebooks
**Visibility**: Public
**License**: MIT
**Purpose**: Educational examples and analysis templates

**Components**:
- `notebooks/`
- Template notebooks
- Example analyses

**Benefits**:
- ✅ Educational value
- ✅ Shows how to use the public libraries
- ✅ Good for portfolio/teaching

**Exclude**:
- Private trading strategies
- Personal research data

---

#### 5. `trading-build-tools` - Reusable Build Scripts
**Visibility**: Public
**License**: MIT
**Purpose**: CMake presets, build scripts, automation

**Components**:
- `scripts/build_universal.sh`
- `scripts/build_fast.sh`
- CMake presets for trading projects
- Generic build automation scripts

**Benefits**:
- ✅ Other developers can reuse
- ✅ Shows best practices
- ✅ Helps community

---

#### 6. `project-housekeeping-tools` - Automated Project Maintenance
**Visibility**: Public
**License**: MIT
**Purpose**: Generic project housekeeping and maintenance automation tools

**Components**:
- `scripts/base/intelligent_automation_base.py` (base class)
- `scripts/automate_docs_health_v2.py` (documentation health)
- `scripts/automate_todo2_alignment_v2.py` (task alignment)
- `scripts/automate_todo2_duplicate_detection.py` (duplicate task detection)
- `scripts/automate_dependency_security.py` (security scanning)
- `scripts/automate_pwa_review.py` (PWA analysis)
- `scripts/automate_todo_sync.py` (task synchronization)
- `scripts/automate_automation_opportunities.py` (opportunity finder)
- `python/tools/project_analyzer.py` (NetworkX project analysis)
- Configuration templates and cron setup scripts

**Dependencies**:
- NetworkX (for graph analysis)
- Python standard library
- Optional: AI APIs for insights (OpenAI, Anthropic)

**Benefits**:
- ✅ **Highly Reusable**: Generic automation framework
- ✅ **Well-Designed**: Base class pattern for extensibility
- ✅ **Useful to Community**: Every project needs maintenance
- ✅ **Innovative**: IntelligentAutomationBase with Tractatus/Sequential thinking
- ✅ **NetworkX Integration**: Graph-based project analysis
- ✅ **PyPI Package Potential**: Could be published as `project-housekeeping-tools`

**Exclude**:
- Project-specific build scripts (stay in `trading-build-tools`)
- TWS-specific scripts
- Broker-specific automation
- Project-specific configuration

**Considerations**:
- Make base class framework-agnostic
- Parameterize project-specific paths
- Create configuration templates
- Add comprehensive documentation
- Publish to PyPI for easy installation

---

#### 7-11. `trading-*-docs` - Public Documentation (Split into 5 Repos)

**Recommendation**: Split into 5 focused repositories for optimal NotebookLM analysis

**See**: [PROJECT_SPLIT_DOCS_STRATEGY.md](PROJECT_SPLIT_DOCS_STRATEGY.md) for detailed split strategy

**Repositories**:

**7. `trading-api-docs`** - API Documentation & Integration
- API_DOCUMENTATION_INDEX.md (chunked if needed)
- API_DOCUMENTATION_SUMMARY.md
- Integration guides
- API indices (FIX, Market Data, etc.)
- **NotebookLM**: ✅ Single notebook (30-40 sources)

**8. `trading-architecture-docs`** - Architecture & Design
- Architecture documentation
- Design patterns
- System design guides
- **NotebookLM**: ✅ Single notebook (20-30 sources)

**9. `trading-setup-docs`** - Setup & Deployment
- Setup guides
- Configuration documentation
- Deployment guides
- **NotebookLM**: ✅ Single notebook (25-35 sources)

**10. `trading-automation-docs`** - Automation & Maintenance
- Automation guides
- Maintenance workflows
- Health monitoring docs
- **NotebookLM**: ✅ Single notebook (20-30 sources)

**11. `trading-tools-docs`** - Tools & Frameworks
- Tool usage guides
- Framework documentation
- Best practices
- **NotebookLM**: ✅ Single notebook (30-40 sources)

**Exclude** (from all docs repos):
- Private research (`docs/research/analysis/`, `docs/research/external/`)
- Internal design decisions
- Broker-specific strategies
- Project-specific configurations

---

### 🔴 PRIVATE Projects (Internal/Personal)

#### 8. `ib-box-spread-trading` - Main Trading Application
**Visibility**: Private
**Purpose**: Your complete trading system

**Components**:
- `agents/` (all microservices)
- `native/src/ib_box_spread.cpp` (main entry point)
- `native/src/tws_client.cpp` (broker integration)
- `native/wasm/` (web assembly module)
- `config/` (your configurations)
- `ib-gateway/` (gateway setup)
- Broker integrations from `python/integration/`

**Why Private**:
- Contains broker API keys (even if in config)
- Trading strategies and risk parameters
- Account information patterns
- Internal architecture decisions

**Dependencies**:
- Uses `box-spread-cpp` as library
- Uses `box-spread-python` as package
- All broker-specific code

---

#### 9. `ib-box-spread-web` - Web Trading Dashboard
**Visibility**: Private
**Purpose**: Your React/TypeScript trading UI

**Components**:
- `web/` (entire directory)
- `desktop/Sources/` (if it shares code)

**Why Private**:
- Connected to your trading backend
- May expose internal APIs
- Trading-specific UI patterns

---

#### 10. `ib-box-spread-mobile` - iOS/iPad Apps
**Visibility**: Private
**Purpose**: Your mobile trading clients

**Components**:
- `ios/`
- `desktop/` (macOS app)
- `ondevice/`

**Why Private**:
- Platform-specific implementations
- Tied to your backend
- May contain app store keys

---

#### 11. `ib-box-spread-research` - Research & Analysis
**Visibility**: Private
**Purpose**: Internal research, analysis, learnings

**Components**:
- `docs/research/` (all subdirectories)
- Private notebooks with strategies
- Backtesting results
- Performance analysis

**Why Private**:
- Contains trading insights
- Internal decision-making processes
- Competitive advantage information

---

#### 12. `ib-box-spread-infra` - Infrastructure & Automation
**Visibility**: Private
**Purpose**: Deployment, monitoring, automation

**Components**:
- `ansible/`
- `roles/`
- `playbooks/`
- Infrastructure automation scripts
- Deployment configs

**Why Private**:
- Contains server information
- Deployment patterns
- Security configurations

---

## Migration Strategy

### Phase 1: Extract Public Libraries (Low Risk)
1. Create `box-spread-cpp` repository
2. Extract core C++ engine (no broker code)
3. Add comprehensive tests
4. Publish as library

### Phase 2: Extract Python Package (Medium Risk)
1. Create `box-spread-python` repository
2. Extract bindings and DSL
3. Make it depend on `box-spread-cpp`
4. Publish to PyPI

### Phase 3: Extract MCP & Tools (Low Risk)
1. Extract MCP servers
2. Extract notebooks
3. Extract build tools
4. Extract public docs

### Phase 4: Reorganize Private Monorepo (High Risk)
1. Keep private components in one repo
2. Use Git submodules or package managers to reference public libraries
3. Update build systems to use published packages
4. Migrate incrementally

---

## Project Structure After Split

```
# Public repositories
davidl71/box-spread-cpp          (Core engine)
davidl71/box-spread-python       (Python package)
davidl71/trading-mcp-servers     (MCP servers)
davidl71/box-spread-notebooks    (Notebooks)
davidl71/trading-build-tools     (Build scripts)
davidl71/project-housekeeping-tools  (Maintenance automation)
davidl71/trading-api-docs        (API documentation)
davidl71/trading-architecture-docs (Architecture guides)
davidl71/trading-setup-docs      (Setup & deployment)
davidl71/trading-automation-docs (Automation guides)
davidl71/trading-tools-docs      (Tools & frameworks)

# Private repositories
davidl71/ib-box-spread-trading   (Main application - monorepo)
  ├── agents/                    (All microservices)
  ├── native/                    (Broker-specific C++ code)
  ├── web/                       (Or separate repo)
  └── config/                    (Private configs)

davidl71/ib-box-spread-web       (Web dashboard)
davidl71/ib-box-spread-mobile    (iOS/iPad apps)
davidl71/ib-box-spread-research  (Research docs)
davidl71/ib-box-spread-infra     (Infrastructure)
```

---

## Dependency Management Strategy

### Option 1: Git Submodules (Simple)
```bash
# In private repo
git submodule add https://github.com/davidl71/box-spread-cpp.git libs/box-spread-cpp
git submodule add https://github.com/davidl71/box-spread-python.git libs/box-spread-python
```

### Option 2: Package Managers (Recommended)
- **C++**: Use Conan or vcpkg for `box-spread-cpp`
- **Python**: Use PyPI for `box-spread-python`
- **NPM**: Use npm for any JS libraries

### Option 3: Monorepo with Workspaces
- Keep everything in one repo but organize clearly
- Use tools like Nx, Turborepo, or Bazel
- Easier to refactor but harder to share

---

## Benefits of Split

### ✅ Public Projects Benefits
1. **Portfolio Value**: Shows expertise in C++, Python, trading systems
2. **Community Contribution**: Others can use and improve
3. **Documentation**: Forces better documentation
4. **Code Quality**: Public scrutiny improves code
5. **Reusability**: Can use your own libraries in other projects

### ✅ Private Projects Benefits
1. **Security**: Trading strategies and credentials stay private
2. **Focus**: Each repo has clear purpose
3. **Deployment**: Can deploy services independently
4. **Team Access**: Control who sees what
5. **Flexibility**: Can experiment without exposing

### ✅ Overall Benefits
1. **Faster CI/CD**: Smaller repos = faster builds
2. **Clearer Boundaries**: Dependencies are explicit
3. **Version Control**: Each component has own versioning
4. **Better Testing**: Test public APIs thoroughly
5. **Professional Structure**: Looks more polished

---

## Risks & Mitigation

### Risk 1: Breaking Changes
**Mitigation**:
- Version public APIs carefully
- Use semantic versioning
- Maintain changelogs
- Test thoroughly before releases

### Risk 2: Migration Complexity
**Mitigation**:
- Migrate incrementally
- Keep old structure working during migration
- Use feature flags
- Test each extraction thoroughly

### Risk 3: Dependency Management
**Mitigation**:
- Use package managers
- Pin versions carefully
- Document dependencies clearly
- Use dependency update automation

### Risk 4: Overhead of Multiple Repos
**Mitigation**:
- Use GitHub organizations
- Automate releases
- Use monorepo tools if needed
- Start with 2-3 repos, add more gradually

---

## Quick Wins (Start Here)

### ✅ Week 1: Extract MCP Servers - **COMPLETE**
- Small, self-contained
- Already useful to others
- Low risk
- ✅ **NotebookLM**: One notebook, <10 sources
- **Repository**: https://github.com/davidl71/trading-mcp-servers (Private)

### ✅ Week 2: Extract Notebooks - **COMPLETE**
- Educational value
- No dependencies on private code
- Easy to share
- ✅ **NotebookLM**: Entire repo in one notebook
- **Repository**: https://github.com/davidl71/box-spread-notebooks (Private)

### ✅ Week 3: Extract Build Tools - **COMPLETE**
- Reusable scripts
- Helps community
- Shows best practices
- ✅ **NotebookLM**: Single focused notebook
- **Repository**: https://github.com/davidl71/trading-build-tools (Private)

### ✅ Week 4: Extract Project Housekeeping Tools - **COMPLETE**
- Generic automation framework
- Highly reusable
- Innovative base class pattern
- ✅ **NotebookLM**: Single focused notebook
- **Repository**: https://github.com/davidl71/trading-automation-tools (Private)

### Week 5-6: Extract Documentation Repos (5 repos)
- Extract API docs (highest value)
- Extract Architecture docs
- Extract Setup docs
- Extract Automation docs
- Extract Tools docs
- ✅ **NotebookLM**: One notebook per repo (<50 sources each)

---

## Long-Term Vision

After splitting, you'll have:

1. **Public Ecosystem**: 11 reusable libraries/tools/docs repos
2. **Private Trading System**: Clean, focused monorepo
3. **Clear Boundaries**: Public vs private is obvious
4. **Professional Portfolio**: Shows software engineering skills
5. **Community Impact**: Others can benefit from your work
6. **Maintainability**: Each project has clear purpose
7. **NotebookLM Optimized**: Each project fits in one focused notebook (<50 sources)

---

## NotebookLM Analysis Considerations

**Critical Factor**: Project splits should optimize for NotebookLM analysis efficiency.

### NotebookLM Limitations (2025)
- **Free Plan**: 50 sources per notebook, ~25 million words total
- **Per Source**: 500,000 words or 200 MB maximum
- **NotebookLM Plus**: 300 sources, 150 million words (paid)

### How Project Split Affects NotebookLM

#### ✅ Benefits of Split for NotebookLM

1. **Focused Notebooks Per Project**
   - Each repo = one focused notebook topic
   - Better context and synthesis
   - Easier to stay within source limits

2. **Clear Boundaries**
   - Related documentation grouped together
   - No mixing of unrelated topics
   - Better cross-references and context

3. **Smaller Repos = Better Analysis**
   - Entire codebase can fit in one notebook
   - All related docs in one place
   - Easier to understand relationships

4. **Easier Source Management**
   - Add entire repo as source(s)
   - Or select specific files/folders
   - Clear organization

#### 📊 NotebookLM Strategy Per Split Project

##### Public Projects (Ideal for NotebookLM)

**1. `box-spread-cpp`**
- **Notebook**: "Box Spread C++ Engine"
- **Sources**:
  - All source files (headers + implementation)
  - Test files
  - README + docs
- **Analysis**: Code patterns, API design, algorithm explanations
- **Benefit**: ✅ Entire project fits in one focused notebook

**2. `box-spread-python`**
- **Notebook**: "Box Spread Python Integration"
- **Sources**:
  - Python bindings
  - DSL implementation
  - TUI code (as example)
  - Tool scripts
- **Analysis**: Python integration patterns, DSL usage, binding techniques
- **Benefit**: ✅ Self-contained, focused topic

**3. `trading-mcp-servers`**
- **Notebook**: "Trading MCP Servers"
- **Sources**: All MCP server code + docs
- **Analysis**: MCP server patterns, trading integrations
- **Benefit**: ✅ Very small, fits easily

**4. `box-spread-notebooks`**
- **Notebook**: "Box Spread Examples & Analysis"
- **Sources**: All notebooks
- **Analysis**: Strategy examples, backtesting patterns
- **Benefit**: ✅ Already organized for analysis

**5. `trading-build-tools`**
- **Notebook**: "Trading Build Tools"
- **Sources**: Scripts + CMake configs
- **Analysis**: Build patterns, automation techniques
- **Benefit**: ✅ Useful for other projects

**6. `project-housekeeping-tools`**
- **Notebook**: "Project Housekeeping Automation"
- **Sources**:
  - Base automation framework
  - All automation scripts
  - Configuration templates
  - Project analyzer
- **Analysis**: Automation patterns, maintenance workflows, NetworkX usage
- **Benefit**: ✅ Highly reusable, innovative framework

**7-11. `trading-*-docs` (5 repositories)**
- **Notebook 7**: "API Documentation & Integration" (30-40 sources)
- **Notebook 8**: "Architecture & Design" (20-30 sources)
- **Notebook 9**: "Setup & Deployment" (25-35 sources)
- **Notebook 10**: "Automation & Maintenance" (20-30 sources)
- **Notebook 11**: "Tools & Frameworks" (30-40 sources)
- **Analysis**: Focused analysis per topic
- **Benefit**: ✅ Each repo fits in one notebook, better organization

##### Private Projects (Internal NotebookLM Use)

**7. `ib-box-spread-trading` (Main App)**
- **Challenge**: Large, complex system
- **Strategy**: Split into multiple notebooks
  - **Notebook 1**: "Backend Services Architecture"
    - Rust backend code + docs (agents/backend/)
    - Stay under 50 sources
  - **Notebook 2**: "TWS Integration & Brokers"
    - Broker-specific C++ code
    - TWS client implementations
  - **Notebook 3**: "Trading Strategy Implementation"
    - Strategy engine
    - Risk management
  - **Notebook 4**: "System Integration & Deployment"
    - Config management
    - Deployment scripts
    - Integration patterns
- **Synthesis Notebook**: "Complete Trading System"
  - Key findings from all 4 notebooks
  - System-wide patterns and decisions

**8. `ib-box-spread-web`**
- **Notebook**: "Trading Web Dashboard"
- **Sources**: React/TypeScript code + docs
- **Analysis**: UI patterns, state management, API integration
- **Benefit**: ✅ Self-contained frontend

**9. `ib-box-spread-mobile`**
- **Notebook**: "Mobile Trading Apps"
- **Sources**: iOS/Swift code
- **Analysis**: Platform patterns, mobile UI
- **Benefit**: ✅ Platform-specific focus

**10. `ib-box-spread-research`**
- **Challenge**: Large research collection
- **Strategy**: Already organized by topic (architect/analysis/integration/external/learnings)
  - **Notebook 1**: "Architecture Research"
    - docs/research/architecture/ (26 files)
  - **Notebook 2**: "Analysis & Evaluation"
    - docs/research/analysis/ (31 files)
  - **Notebook 3**: "Integration Research"
    - docs/research/integration/ (59 files - might need splitting)
  - **Notebook 4**: "External API Research"
    - docs/research/external/ (external URLs)
  - **Notebook 5**: "TWS API Learnings"
    - docs/research/learnings/ (TWS-specific)
- **Synthesis**: "Complete Research Synthesis"
  - Key findings from all research notebooks

**11. `ib-box-spread-infra`**
- **Notebook**: "Infrastructure & Automation"
- **Sources**: Ansible, scripts, configs
- **Analysis**: Deployment patterns, automation
- **Benefit**: ✅ Infrastructure focus

### 📋 NotebookLM Optimization Strategy

#### Rule 1: One Repository = One Primary Notebook Topic
- Each split project naturally becomes one notebook
- Related docs stay together
- Clear boundaries prevent confusion

#### Rule 2: Split Large Projects Before Analysis
- If repo has >50 source files, split the analysis
- Create multiple focused notebooks per repo
- Use synthesis notebook to combine findings

#### Rule 3: Prioritize External Sources
- External URLs count as sources
- Add official docs, API docs to notebooks
- Combine internal docs to save slots

#### Rule 4: Use Synthesis Notebooks Strategically
- Create "overview" notebooks that summarize multiple projects
- Cross-project insights in synthesis notebooks
- Don't duplicate sources across notebooks

### 🎯 Recommended NotebookLM Workflow After Split

#### Phase 1: Project-Level Notebooks
1. Create one notebook per public project
2. Add entire repo or key files
3. Analyze patterns, APIs, usage

#### Phase 2: Integration Notebooks
1. Create "Integration" notebooks that reference multiple projects
2. Focus on how projects work together
3. Use synthesis from project notebooks

#### Phase 3: System-Wide Notebooks (Private)
1. Create focused notebooks per major component
2. Stay within 50 sources per notebook
3. Create synthesis notebook for system-wide view

### ✅ Benefits of Split for NotebookLM

1. **Clear Organization**
   - Each repo = one notebook topic
   - No mixing of unrelated code
   - Better context and synthesis

2. **Fits Within Limits**
   - Smaller repos fit in 50 sources
   - Related docs grouped together
   - No need to chunk large repos

3. **Easier Analysis**
   - Focused questions per project
   - Better understanding of boundaries
   - Clearer relationships

4. **Reusability**
   - Public project notebooks can be shared
   - Others can learn from your analysis
   - Template for similar projects

5. **Maintenance**
   - Update notebooks as repos change
   - Clear ownership per project
   - Easier to keep in sync

### ⚠️ Considerations for NotebookLM After Split

#### Challenge 1: Cross-Project References
- **Solution**: Use synthesis notebooks
- Reference findings from project notebooks
- Document relationships clearly

#### Challenge 2: Private Project Size
- **Solution**: Split large private repos into multiple notebooks
- One notebook per major component
- Use synthesis for system view

#### Challenge 3: Keeping Notebooks Updated
- **Solution**: Automate notebook updates
- Scripts to sync repo changes
- Regular review and updates

### 📊 NotebookLM Source Count Estimates

| Project | Estimated Sources | Notebook Strategy |
|---------|------------------|-------------------|
| `box-spread-cpp` | 20-30 files | ✅ Single notebook |
| `box-spread-python` | 30-40 files | ✅ Single notebook |
| `trading-mcp-servers` | 5-10 files | ✅ Single notebook |
| `box-spread-notebooks` | 10-15 notebooks | ✅ Single notebook |
| `trading-build-tools` | 20-30 scripts | ✅ Single notebook |
| `project-housekeeping-tools` | 15-25 scripts | ✅ Single notebook |
| `trading-api-docs` | 30-40 docs | ✅ Single notebook |
| `trading-architecture-docs` | 20-30 docs | ✅ Single notebook |
| `trading-setup-docs` | 25-35 docs | ✅ Single notebook |
| `trading-automation-docs` | 20-30 docs | ✅ Single notebook |
| `trading-tools-docs` | 30-40 docs | ✅ Single notebook |
| `ib-box-spread-trading` | 100+ files | ⚠️ Split into 4+ notebooks |
| `ib-box-spread-web` | 50-70 files | ✅ Single notebook |
| `ib-box-spread-mobile` | 10-20 files | ✅ Single notebook |
| `ib-box-spread-research` | 100+ files | ⚠️ Split into 5+ notebooks |
| `ib-box-spread-infra` | 20-30 files | ✅ Single notebook |

**Note**: Source count includes code files, documentation, configs, and external URLs.

---

## Questions to Consider

1. **Which components have the most reuse potential?**
   → Core engine, Python DSL, MCP servers, **Project housekeeping tools**

2. **What contains sensitive information?**
   → Broker integrations, strategies, configs, research

3. **What would benefit from community contribution?**
   → Core libraries, documentation, tools

4. **What's purely internal to your trading system?**
   → Agents, web UI, mobile apps, infrastructure

5. **What can be monetized or licensed?**
   → Advanced strategies (keep private), basic engine (open source)

6. **How will I analyze each project with NotebookLM?**
   → One notebook per public project, multiple focused notebooks for large private repos

---

## Next Steps

1. ✅ Review this document
2. ⬜ Decide which public projects to start with
3. ⬜ Create first public repository (`box-spread-cpp` recommended)
4. ⬜ Extract core engine with tests
5. ⬜ Update private repo to use extracted library
6. ⬜ Repeat for other components

---

**Remember**: You can always start small and expand. Even extracting just the MCP servers and notebooks would be a good start!
