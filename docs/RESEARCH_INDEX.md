# Research Documents Master Index

**Purpose**: Central navigation hub for research, learnings, analysis, and integration documents in this project.

> Historical note: this index intentionally includes older evaluation material about deferred Apple clients, deprecated Nautilus work, and removed broker/service paths. Treat it as research/archive context, not as the source of truth for the active runtime. For the current system shape, start with `docs/platform/CURRENT_TOPOLOGY.md`.

**Last Updated**: 2025-11-20
**Total Documents**: 300+ research documents indexed

---

## Quick Navigation

| Category | Count | Primary Use |
|----------|-------|-------------|
| [External API Research](#external-api-research) | 15+ | External APIs, whitepapers, market data |
| [Framework Learnings](#framework-learnings) | 20+ | Tool/framework patterns and best practices |
| [Architecture & Design](#architecture--design) | 30+ | System architecture, design decisions |
| [Integration Guides](#integration-guides) | 25+ | Step-by-step integration instructions |
| [Analysis Documents](#analysis-documents) | 40+ | Code analysis, evaluations, comparisons |
| [Implementation Guides](#implementation-guides) | 20+ | How-to guides, setup instructions |
| [Topic-Specific Indices](#topic-specific-indices) | 6 | Focused indices by topic |

---

## External API Research

Documents containing external API documentation, whitepapers, and market data research.

### Market Data & Options Data

- **[CME_RESEARCH.md](research/external/CME_RESEARCH.md)** - CME financing & integration research
  - CME Group whitepapers on capital efficiency
  - Cboe box spread borrowing/lending strategies
  - CME Client Systems Wiki integration portals
  - **NotebookLM Priority**: HIGH - Contains 4 external links to synthesize

- **[ORATS_INTEGRATION.md](research/external/ORATS_INTEGRATION.md)** - ORATS options data integration
  - Live/delayed options data APIs
  - Proprietary indicators and analytics
  - Integration opportunities for box spread detection
  - **NotebookLM Priority**: HIGH - External API documentation

- **[DATA_FEEDS_BOX_SPREADS.md](strategies/box-spread/DATA_FEEDS_BOX_SPREADS.md)** - Market data feed analysis
- **[MARKET_DATA_INDEX.md](indices/MARKET_DATA_INDEX.md)** - Comprehensive market data provider index

### Trading APIs & Brokers

- **[API_DOCUMENTATION_INDEX.md](API_DOCUMENTATION_INDEX.md)** - Complete API documentation (2,611 lines)
  - TWS API, Alpaca, FIX protocols, market data providers
  - **NotebookLM Priority**: MEDIUM - Break into topic-specific notebooks

- **[RESEARCH_TRADING_ECONOMICS_API.md](research/external/RESEARCH_TRADING_ECONOMICS_API.md)** - Trading Economics API research
- **[RESEARCH_IB_PYTHON_TRADING.md](research/external/RESEARCH_IB_PYTHON_TRADING.md)** - IB Python trading patterns
- **[QUANTPEDIA_BROKERAGE_APIS.md](research/external/QUANTPEDIA_BROKERAGE_APIS.md)** - Brokerage API research
- **[ALPACA_INTEGRATION_PLAN_V2.md](research/integration/ALPACA_INTEGRATION_PLAN_V2.md)** - Alpaca API integration
- **[IB_CLIENT_PORTAL_API_INTEGRATION_DESIGN.md](research/external/IB_CLIENT_PORTAL_API_INTEGRATION_DESIGN.md)** - IB Client Portal API

### Financial Platforms & Tools

- **[RESEARCH_FINANCIAL_LEDGER_PLATFORMS.md](research/external/RESEARCH_FINANCIAL_LEDGER_PLATFORMS.md)** - Ledger platform research
- **[RESEARCH_FPGA_OPTION_PRICING.md](research/external/RESEARCH_FPGA_OPTION_PRICING.md)** - FPGA pricing research
- **[RESEARCH_CPP_FINANCIAL_LIBRARIES.md](research/external/RESEARCH_CPP_FINANCIAL_LIBRARIES.md)** - C++ financial library research

---

## Framework Learnings

Documents capturing patterns, best practices, and learnings from specific tools and frameworks.

### TWS API Learnings

- **[TWS_API_BEST_PRACTICES.md](research/learnings/TWS_API_BEST_PRACTICES.md)** - TWS API best practices
- **[TWS_API_CODE_EXAMPLES_LEARNINGS.md](research/learnings/TWS_API_CODE_EXAMPLES_LEARNINGS.md)** - Code examples and patterns
- **[TWS_API_MARKET_DATA_LEARNINGS.md](research/learnings/TWS_API_MARKET_DATA_LEARNINGS.md)** - Market data handling
- **[TWS_API_TROUBLESHOOTING_LEARNINGS.md](research/learnings/TWS_API_TROUBLESHOOTING_LEARNINGS.md)** - Common issues and solutions
- **[TWS_API_DOCKER_LEARNINGS.md](research/learnings/TWS_API_DOCKER_LEARNINGS.md)** - Docker integration patterns
- **[TWS_API_IMPLEMENTATION_COMPARISON.md](research/learnings/TWS_API_IMPLEMENTATION_COMPARISON.md)** - Implementation comparisons
- **[IB_API_QUICK_REFERENCE_LEARNINGS.md](research/learnings/IB_API_QUICK_REFERENCE_LEARNINGS.md)** - Quick reference patterns
- **[IB_ASYNC_LEARNINGS.md](research/learnings/IB_ASYNC_LEARNINGS.md)** - Asynchronous patterns
- **[IBC_LEARNINGS.md](research/learnings/IBC_LEARNINGS.md)** - IBC (IB Controller) learnings
- **[ECLIENT_EWRAPPER_ARCHITECTURE.md](research/learnings/ECLIENT_EWRAPPER_ARCHITECTURE.md)** - EClient/EWrapper architecture (Note: This is learnings about TWS API patterns, correctly categorized)
- **[EWRAPPER_STATUS.md](research/learnings/EWRAPPER_STATUS.md)** - EWrapper implementation status
- **NotebookLM Priority**: HIGH - Consolidate into unified TWS API best practices

### Trading Frameworks

- **[NAUTILUS_LEARNINGS.md](research/learnings/NAUTILUS_LEARNINGS.md)** - NautilusTrader patterns and architecture
  - Rust/Python hybrid architecture
  - Event-driven design patterns
  - Performance optimization techniques

- **[NAUTILUS_IMPLEMENTATION_SUMMARY.md](research/learnings/NAUTILUS_IMPLEMENTATION_SUMMARY.md)** - Implementation details
- **[TRADE_FRAME_LEARNINGS.md](research/learnings/TRADE_FRAME_LEARNINGS.md)** - TradeFrame patterns
- **[TRADE_FRAME_TWS_PATTERNS.md](research/learnings/TRADE_FRAME_TWS_PATTERNS.md)** - TradeFrame TWS integration
- **[YATWS_LEARNINGS.md](research/learnings/YATWS_LEARNINGS.md)** - Yet Another TWS learnings
- **[ICLI_LEARNINGS.md](research/learnings/ICLI_LEARNINGS.md)** - ICLI (Interactive Brokers CLI) patterns
- **[IBKRBOX_LEARNINGS.md](research/learnings/IBKRBOX_LEARNINGS.md)** - IBKRBox patterns

### Development Tools

- **[LEAN_LEARNINGS.md](research/learnings/LEAN_LEARNINGS.md)** - QuantConnect LEAN platform learnings
- **[MULTITHREADED_TRADING_LEARNINGS.md](research/learnings/MULTITHREADED_TRADING_LEARNINGS.md)** - Multithreading patterns

---

## Architecture & Design

System architecture documents, design decisions, and architectural patterns.

### Core Architecture

- **[CODEBASE_ARCHITECTURE.md](research/architecture/CODEBASE_ARCHITECTURE.md)** - Overall system architecture
- **[MULTI_LANGUAGE_ARCHITECTURE.md](research/architecture/MULTI_LANGUAGE_ARCHITECTURE.md)** - Multi-language coordination
- **[MESSAGE_QUEUE_ARCHITECTURE.md](research/architecture/MESSAGE_QUEUE_ARCHITECTURE.md)** - Message queue architecture
- **[MESSAGE_QUEUE_RESEARCH.md](research/architecture/MESSAGE_QUEUE_RESEARCH.md)** - Message queue solution research
  - NATS, RabbitMQ, Redis, ZeroMQ comparison
  - **NotebookLM Priority**: HIGH - External documentation to synthesize

- **[SYNTHETIC_FINANCING_ARCHITECTURE.md](platform/SYNTHETIC_FINANCING_ARCHITECTURE.md)** - Synthetic financing design
- **[REST_API_LAYER_DESIGN.md](research/architecture/REST_API_LAYER_DESIGN.md)** - REST API architecture
- **[WEBSOCKET_IMPLEMENTATION_DESIGN.md](research/architecture/WEBSOCKET_IMPLEMENTATION_DESIGN.md)** - WebSocket architecture

### Component Design

- **[BOX_SPREAD_BAG_IMPLEMENTATION.md](strategies/box-spread/BOX_SPREAD_BAG_IMPLEMENTATION.md)** - Box spread bag design
- **[BANK_LOAN_POSITION_SYSTEM_DESIGN.md](research/architecture/BANK_LOAN_POSITION_SYSTEM_DESIGN.md)** - Bank loan system
- **[PORTFOLIO_GREEKS_SYSTEM.md](research/architecture/PORTFOLIO_GREEKS_SYSTEM.md)** - Portfolio Greeks calculation
- **[CASH_FLOW_FORECASTING_SYSTEM.md](research/architecture/CASH_FLOW_FORECASTING_SYSTEM.md)** - Cash flow forecasting
- **[BACKEND_DATA_STORAGE_ARCHITECTURE.md](research/architecture/BACKEND_DATA_STORAGE_ARCHITECTURE.md)** - Data storage design
- **[DATABASE_ABSTRACTION_LAYER.md](research/architecture/DATABASE_ABSTRACTION_LAYER.md)** - Database abstraction

### DSL & Domain Modeling

- **[DSL_RESEARCH_AND_DESIGN.md](research/architecture/DSL_RESEARCH_AND_DESIGN.md)** - Domain-specific language research
- **[DSL_ARCHITECTURE_DESIGN.md](research/architecture/DSL_ARCHITECTURE_DESIGN.md)** - DSL architecture
- **[MULTI_ASSET_RELATIONSHIP_DSL_DESIGN.md](research/architecture/MULTI_ASSET_RELATIONSHIP_DSL_DESIGN.md)** - Multi-asset DSL

### Multi-Broker & Aggregation

- **[MULTI_BROKER_ARCHITECTURE_DESIGN.md](research/architecture/MULTI_BROKER_ARCHITECTURE_DESIGN.md)** - Multi-broker design
- **[MULTI_ACCOUNT_AGGREGATION_DESIGN.md](platform/MULTI_ACCOUNT_AGGREGATION_DESIGN.md)** - Account aggregation
- **[UNIVERSAL_BROKERAGE_AGGREGATION.md](research/architecture/UNIVERSAL_BROKERAGE_AGGREGATION.md)** - Brokerage aggregation

---

## Integration Guides

Step-by-step integration instructions for external services, APIs, and tools.

### Broker Integrations

- **[TWS_INTEGRATION_STATUS.md](research/integration/TWS_INTEGRATION_STATUS.md)** - TWS integration status
- **[ALPACA_BACKEND_SETUP.md](research/integration/ALPACA_BACKEND_SETUP.md)** - Alpaca backend setup
- **[ALPACA_API_INTEGRATION_DESIGN.md](research/integration/ALPACA_API_INTEGRATION_DESIGN.md)** - Alpaca API design
- **[LEAN_IBKR_SETUP.md](research/integration/LEAN_IBKR_SETUP.md)** - LEAN IBKR setup
- **[LEAN_ALPACA_SETUP.md](research/integration/LEAN_ALPACA_SETUP.md)** - LEAN Alpaca setup
- **[LEAN_SETUP.md](research/integration/LEAN_SETUP.md)** - LEAN platform setup

### Data & Market Data

- **[MASSIVE_INTEGRATION.md](research/integration/MASSIVE_INTEGRATION.md)** - Massive.com integration
- **[LIVEVOL_QUICK_START.md](research/integration/LIVEVOL_QUICK_START.md)** - LiveVol setup
- **[LIVEVOL_AUTHENTICATION_SUMMARY.md](research/integration/LIVEVOL_AUTHENTICATION_SUMMARY.md)** - LiveVol auth
- **[LIVEVOL_API_CREDENTIALS_GUIDE.md](research/integration/LIVEVOL_API_CREDENTIALS_GUIDE.md)** - LiveVol credentials
- **[LIVEVOL_CREDENTIALS_WHERE_TO_FIND.md](research/integration/LIVEVOL_CREDENTIALS_WHERE_TO_FIND.md)** - Credential location
- **[LIVEVOL_QUOTED_SPREADS_GUIDE.md](research/integration/LIVEVOL_QUOTED_SPREADS_GUIDE.md)** - Quoted spreads guide
- **[LIVEVOL_TRIAL_SETUP.md](research/integration/LIVEVOL_TRIAL_SETUP.md)** - LiveVol trial

### Financial Tools

- **[LEDGER_INTEGRATION_GUIDE.md](research/integration/LEDGER_INTEGRATION_GUIDE.md)** - Ledger CLI integration
- **[LEDGER_CORE_LIBRARY_DESIGN.md](research/integration/LEDGER_CORE_LIBRARY_DESIGN.md)** - Ledger library design
- **[LEDGER_IMPORT_USAGE.md](research/integration/LEDGER_IMPORT_USAGE.md)** - Ledger import
- **[LEDGER_PERSISTENCE_USAGE.md](research/integration/LEDGER_PERSISTENCE_USAGE.md)** - Ledger persistence

### Development Tools

- **[QUANTLIB_INTEGRATION_GUIDE.md](research/integration/QUANTLIB_INTEGRATION_GUIDE.md)** - QuantLib integration
- **[EIGEN_INTEGRATION.md](research/integration/EIGEN_INTEGRATION.md)** - Eigen library integration
- **[NLOPT_INTEGRATION_GUIDE.md](research/integration/NLOPT_INTEGRATION_GUIDE.md)** - NLopt optimization
- **[OPENALGO_INTEGRATION_PATTERNS.md](research/integration/OPENALGO_INTEGRATION_PATTERNS.md)** - OpenAlgo patterns
- **[ONIXS_DIRECTCONNECT.md](research/integration/ONIXS_DIRECTCONNECT.md)** - ONIXS DirectConnect
- **[ONIXS_FIX_DICTIONARY_TOOLS.md](research/integration/ONIXS_FIX_DICTIONARY_TOOLS.md)** - ONIXS FIX tools

### Platform Integrations

- **[LEAN_PWA_TUI_INTEGRATION.md](research/integration/LEAN_PWA_TUI_INTEGRATION.md)** - LEAN PWA/TUI integration
- **[LEAN_PWA_TUI_INTEGRATION_ANALYSIS.md](research/integration/LEAN_PWA_TUI_INTEGRATION_ANALYSIS.md)** - Integration analysis
- **[LEAN_PYBIND11_INTEGRATION_ANALYSIS.md](research/integration/LEAN_PYBIND11_INTEGRATION_ANALYSIS.md)** - Pybind11 analysis
- **[LEAN_REST_API_WRAPPER_DESIGN.md](research/integration/LEAN_REST_API_WRAPPER_DESIGN.md)** - REST API wrapper
- **[LEAN_BROKER_ADAPTERS.md](research/integration/LEAN_BROKER_ADAPTERS.md)** - Broker adapters
- **[LEAN_MIGRATION_SUMMARY.md](research/integration/LEAN_MIGRATION_SUMMARY.md)** - Migration summary

---

## Analysis Documents

Code analysis, framework evaluations, and comparison documents.

### Framework Evaluations

- **[TRADING_FRAMEWORK_EVALUATION.md](research/analysis/TRADING_FRAMEWORK_EVALUATION.md)** - Trading framework comparison
  - **NotebookLM Priority**: HIGH - Synthesize framework comparisons

- **[FRAMEWORK_ANALYSIS_AND_RECOMMENDATIONS.md](research/analysis/FRAMEWORK_ANALYSIS_AND_RECOMMENDATIONS.md)** - Framework analysis
- **[SMARTQUANT_CPP_FRAMEWORK_RESEARCH.md](research/analysis/SMARTQUANT_CPP_FRAMEWORK_RESEARCH.md)** - SmartQuant research
- **[STOCKSHARP_ANALYSIS.md](research/analysis/STOCKSHARP_ANALYSIS.md)** - StockSharp analysis
- **[FINCEPT_TERMINAL_ANALYSIS.md](research/analysis/FINCEPT_TERMINAL_ANALYSIS.md)** - Fincept Terminal analysis
- **[OPEN_TRADING_PLATFORM_ANALYSIS.md](research/analysis/OPEN_TRADING_PLATFORM_ANALYSIS.md)** - Open Trading Platform

### Code & Architecture Analysis

- **[CODE_VERIFICATION_REVIEW.md](research/analysis/CODE_VERIFICATION_REVIEW.md)** - Code verification
- **[CODE_IMPROVEMENTS_FROM_NOTEBOOKLM.md](research/analysis/CODE_IMPROVEMENTS_FROM_NOTEBOOKLM.md)** - NotebookLM code analysis
- **[STATIC_ANALYSIS.md](research/analysis/STATIC_ANALYSIS.md)** - Static analysis results
- **[STATIC_ANALYSIS_ANNOTATIONS.md](research/analysis/STATIC_ANALYSIS_ANNOTATIONS.md)** - Analysis annotations
- **[TEST_COVERAGE_ANALYSIS.md](research/analysis/TEST_COVERAGE_ANALYSIS.md)** - Test coverage
- **[TUI_TEST_COVERAGE_ANALYSIS.md](research/analysis/TUI_TEST_COVERAGE_ANALYSIS.md)** - TUI test coverage
- **[SECURITY_VULNERABILITIES_REVIEW.md](research/analysis/SECURITY_VULNERABILITIES_REVIEW.md)** - Security review
- **[FOSSOLOGY_ANALYSIS.md](research/analysis/FOSSOLOGY_ANALYSIS.md)** - License analysis

### Strategy & Trading Analysis

- **[SYNTHETICFI_LENDING_BORROWING_ANALYSIS.md](research/analysis/SYNTHETICFI_LENDING_BORROWING_ANALYSIS.md)** - SyntheticFi analysis
- **[BUY_SELL_DISPARITY_ANALYSIS.md](research/analysis/BUY_SELL_DISPARITY_ANALYSIS.md)** - Buy/sell analysis
- **[RISK_FREE_RATE_METHODOLOGY.md](research/analysis/RISK_FREE_RATE_METHODOLOGY.md)** - Risk-free rate analysis
- **[CURRENCY_EXCHANGE_RISK.md](research/analysis/CURRENCY_EXCHANGE_RISK.md)** - Currency risk analysis

### System Analysis

- **[NETWORKX_PROJECT_ANALYSIS.md](research/analysis/NETWORKX_PROJECT_ANALYSIS.md)** - NetworkX analysis
- **[CLAUDE_SKILLS_ANALYSIS.md](research/analysis/CLAUDE_SKILLS_ANALYSIS.md)** - Claude skills analysis
- **[MCP_OPTIMIZATION_RECOMMENDATIONS.md](research/analysis/MCP_OPTIMIZATION_RECOMMENDATIONS.md)** - MCP optimization
- **[SETTINGS_DUPLICATION_ANALYSIS.md](research/analysis/SETTINGS_DUPLICATION_ANALYSIS.md)** - Settings duplication
- **[EXTENSION_AUDIT_RESULTS.md](research/analysis/EXTENSION_AUDIT_RESULTS.md)** - Extension audit
- **[EXTENSION_REDUNDANCY_REPORT.md](research/analysis/EXTENSION_REDUNDANCY_REPORT.md)** - Extension redundancy
- **[TASKS_MD_ANALYSIS.md](research/analysis/TASKS_MD_ANALYSIS.md)** - Tasks analysis
- **[TABNINE_EVALUATION.md](research/analysis/TABNINE_EVALUATION.md)** - Tabnine evaluation
- **[CLINE_ANALYSIS.md](research/analysis/CLINE_ANALYSIS.md)** - Cline analysis

### Alignment & Prioritization

- **[TASK_ALIGNMENT_ANALYSIS.md](research/analysis/TASK_ALIGNMENT_ANALYSIS.md)** - Task alignment
- **[TODO2_PRIORITY_ALIGNMENT_ANALYSIS.md](TODO2_PRIORITY_ALIGNMENT_ANALYSIS.md)** - Priority alignment
- **[TODO2_SYNTHETIC_FINANCING_ALIGNMENT_ANALYSIS.md](TODO2_SYNTHETIC_FINANCING_ALIGNMENT_ANALYSIS.md)** - Synthetic financing alignment

---

## Implementation Guides

How-to guides, setup instructions, and implementation documentation.

### Quick Start & Setup

- **[QUICK_START.md](research/integration/QUICK_START.md)** - Quick start guide
- **[QUICK_START_CROSS_PLATFORM.md](research/integration/QUICK_START_CROSS_PLATFORM.md)** - Cross-platform setup
- **[IMPLEMENTATION_GUIDE.md](research/integration/IMPLEMENTATION_GUIDE.md)** - Complete implementation guide
- **[IMPLEMENTATION_COMPLETE_SUMMARY.md](research/integration/IMPLEMENTATION_COMPLETE_SUMMARY.md)** - Implementation summary
- **[PLAN_IMPLEMENTATION_SUMMARY.md](research/integration/PLAN_IMPLEMENTATION_SUMMARY.md)** - Plan summary

### Platform Setup

- **[WINDOWS_SETUP_GUIDE.md](research/integration/WINDOWS_SETUP_GUIDE.md)** - Windows setup
- **[CROSS_PLATFORM_SETUP.md](research/integration/CROSS_PLATFORM_SETUP.md)** - Cross-platform setup
- **[WORKTREE_SETUP.md](research/integration/WORKTREE_SETUP.md)** - Git worktree setup
- **[CURSOR_SETUP.md](research/integration/CURSOR_SETUP.md)** - Cursor IDE setup
- **[CURSOR_IGNORE_SETUP.md](research/integration/CURSOR_IGNORE_SETUP.md)** - Cursor ignore configuration
- **[CURSOR_DOCS_USAGE.md](research/integration/CURSOR_DOCS_USAGE.md)** - Cursor @docs usage

### Build & Development

- **[DISTRIBUTED_COMPILATION.md](research/integration/DISTRIBUTED_COMPILATION.md)** - Distributed builds
- **[WASM_INTEGRATION_PLAN.md](research/integration/WASM_INTEGRATION_PLAN.md)** - WebAssembly integration
- **[WASM_QUICK_START.md](research/integration/WASM_QUICK_START.md)** - WASM quick start
- **[EMSCRIPTEN_SETUP.md](research/integration/EMSCRIPTEN_SETUP.md)** - Emscripten setup
- **[PROTOBUF_MIGRATION_PLAN.md](research/integration/PROTOBUF_MIGRATION_PLAN.md)** - Protocol Buffers migration
- **[DECIMAL_MIGRATION_PLANNING.md](decimal_migration_planning.md)** - Decimal migration

### Testing & Validation

- **[TESTING_STRATEGY.md](research/integration/TESTING_STRATEGY.md)** - Testing strategy
- **[TUI_TESTING.md](research/integration/TUI_TESTING.md)** - TUI testing guide
- **[LEAN_TESTING.md](research/integration/LEAN_TESTING.md)** - LEAN testing
- **[INTEGRATION_TESTING.md](research/integration/INTEGRATION_TESTING.md)** - Integration testing
- **[PAPER_TRADING_VALIDATION_PLAN.md](research/integration/PAPER_TRADING_VALIDATION_PLAN.md)** - Paper trading validation
- **[VALIDATION_SETUP_COMPLETE.md](research/integration/VALIDATION_SETUP_COMPLETE.md)** - Validation setup
- **[VALIDATION_SETUP_FINAL.md](research/integration/VALIDATION_SETUP_FINAL.md)** - Final validation

### TUI & UI Implementation

- **[TUI_DESIGN.md](research/architecture/TUI_DESIGN.md)** - TUI design guide
- **[TUI_SCENARIO_EXPLORER_DESIGN.md](research/architecture/TUI_SCENARIO_EXPLORER_DESIGN.md)** - Scenario explorer
- **[TUI_MULTISCREEN_RESEARCH.md](research/architecture/TUI_MULTISCREEN_RESEARCH.md)** - Multi-screen research
- **[TUI_PYTHON_MIGRATION.md](research/architecture/TUI_PYTHON_MIGRATION.md)** - Python migration
- **[IPAD_APP_DESIGN.md](research/architecture/IPAD_APP_DESIGN.md)** - iPad app design
- **[PWA_PATTERNS_APPLICABILITY.md](research/architecture/PWA_PATTERNS_APPLICABILITY.md)** - PWA patterns

### Specific Features

- **[SYNTHETIC_LENDING_BORROWING_IMPLEMENTATION.md](research/integration/SYNTHETIC_LENDING_BORROWING_IMPLEMENTATION.md)** - Synthetic lending
- **[COMMISSIONS_AND_HEDGING_IMPLEMENTATION.md](research/integration/COMMISSIONS_AND_HEDGING_IMPLEMENTATION.md)** - Commissions & hedging
- **[SWIFTNESS_IMPORT_DESIGN.md](research/integration/SWIFTNESS_IMPORT_DESIGN.md)** - Swiftness import
- **[SWIFTNESS_DATA_MODEL.md](research/integration/SWIFTNESS_DATA_MODEL.md)** - Swiftness data model
- **[BOX_SPREAD_COMPREHENSIVE_GUIDE.md](strategies/box-spread/BOX_SPREAD_COMPREHENSIVE_GUIDE.md)** - Box spread guide

---

## Topic-Specific Indices

Focused indices organized by specific topics. See individual index files for detailed entries.

### Trading & Market Data

- **[indices/BOX_SPREAD_RESOURCES_INDEX.md](strategies/box-spread/BOX_SPREAD_RESOURCES_INDEX.md)** - Box spread resources
- **[indices/MARKET_DATA_INDEX.md](indices/MARKET_DATA_INDEX.md)** - Market data providers
- **[indices/TRADING_FRAMEWORKS_INDEX.md](indices/TRADING_FRAMEWORKS_INDEX.md)** - Trading frameworks
- **[indices/TRADING_SIMULATORS_INDEX.md](indices/TRADING_SIMULATORS_INDEX.md)** - Trading simulators

### Technical Resources

- **[indices/FIX_PROTOCOL_INDEX.md](indices/FIX_PROTOCOL_INDEX.md)** - FIX protocol resources
- **[indices/QUANTITATIVE_FINANCE_INDEX.md](indices/QUANTITATIVE_FINANCE_INDEX.md)** - Quantitative finance libraries

---

## NotebookLM Research Priorities

Documents marked for deeper research using NotebookLM MCP to synthesize external sources.

### High Priority (External Sources)

1. **[CME_RESEARCH.md](research/external/CME_RESEARCH.md)** ⭐⭐⭐
   - 4 external CME/Cboe whitepaper links
   - Financing strategies comparison
   - Integration portal documentation

2. **[MESSAGE_QUEUE_RESEARCH.md](research/architecture/MESSAGE_QUEUE_RESEARCH.md)** ⭐⭐⭐
   - NATS, RabbitMQ, Redis, ZeroMQ documentation
   - Performance comparisons
   - Integration patterns

3. **[ORATS_INTEGRATION.md](research/external/ORATS_INTEGRATION.md)** ⭐⭐⭐
   - ORATS API documentation
   - Options data integration strategies

4. **TWS API Learnings Consolidation** ⭐⭐⭐
   - Multiple `*_LEARNINGS.md` files
   - Create unified best practices document

5. **[TRADING_FRAMEWORK_EVALUATION.md](research/analysis/TRADING_FRAMEWORK_EVALUATION.md)** ⭐⭐
   - Framework comparison synthesis

### Medium Priority

6. **[API_DOCUMENTATION_INDEX.md](API_DOCUMENTATION_INDEX.md)** ⭐⭐
   - Break into topic-specific notebooks
   - Focused research on specific APIs

7. **Architecture Decisions** ⭐
   - Multiple architecture documents
   - Synthesize patterns and decisions

---

## Document Status & Maintenance

### Recently Updated

- **2025-11-20**: MESSAGE_QUEUE_RESEARCH.md, CME_RESEARCH.md
- **2025-01-27**: API_DOCUMENTATION_INDEX.md, ORATS_INTEGRATION.md

### Archive

Deprecated or superseded documents:

- **[archive/ACTION_PLAN.md](archive/ACTION_PLAN.md)**
- **[archive/CODE_IMPROVEMENTS_ACTION_PLAN.md](archive/CODE_IMPROVEMENTS_ACTION_PLAN.md)**
- **[archive/ALPACA_INTEGRATION_PLAN_DEPRECATED.md](archive/ALPACA_INTEGRATION_PLAN_DEPRECATED.md)**

### External References

External documentation and quick references:

- **[external/TWS_API_QUICK_REFERENCE.md](external/TWS_API_QUICK_REFERENCE.md)**
- **[external/CMake_PRESETS_GUIDE.md](external/CMake_PRESETS_GUIDE.md)**
- **[external/ECLIENT_EWRAPPER_PATTERNS.md](external/ECLIENT_EWRAPPER_PATTERNS.md)**
- **[external/CPP20_FEATURES.md](external/CPP20_FEATURES.md)**

---

## How to Use This Index

### Finding Research Documents

1. **By Category**: Browse sections above for documents by type
2. **By Topic**: Use topic-specific indices in `indices/` directory
3. **By Priority**: Check NotebookLM Research Priorities for external source synthesis
4. **By Status**: See Document Status & Maintenance for recent updates

### For NotebookLM Research

1. Start with **High Priority** documents (marked with ⭐⭐⭐)
2. Upload external links from research documents to NotebookLM
3. Create topic-specific notebooks (e.g., "CME Financing Strategies")
4. Synthesize findings and update original research documents

### For Code Implementation

1. Check **Implementation Guides** for step-by-step instructions
2. Reference **Framework Learnings** for patterns and best practices
3. Review **Architecture & Design** for system design decisions
4. Consult **Integration Guides** for external service setup

### For Analysis

1. Review **Analysis Documents** for evaluations and comparisons
2. Check **Topic-Specific Indices** for comprehensive resource lists
3. See **External API Research** for market data and API documentation

---

## Related Documentation

- **[API_DOCUMENTATION_INDEXING.md](research/analysis/API_DOCUMENTATION_INDEXING.md)** - Indexing strategy
- **[API_DOCUMENTATION_CONSOLIDATION_PLAN.md](research/analysis/API_DOCUMENTATION_CONSOLIDATION_PLAN.md)** - Consolidation plan
- **[NOTEBOOKLM_USAGE.md](research/integration/NOTEBOOKLM_USAGE.md)** - NotebookLM usage guide
- **[MCP_SERVERS.md](research/integration/MCP_SERVERS.md)** - MCP server configuration

---

**Maintained by**: AI Assistant
**Last Review**: 2025-11-20
**Next Review**: Quarterly or as needed
