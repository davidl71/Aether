# API Documentation Index

<!--
@index: api-documentation
@category: reference
@tags: api, trading, options, market-data, fix-protocol, quantitative-finance
@last-updated: 2025-01-27
-->

This file serves as a reference for all external APIs and libraries used in this project. Use `@docs API_DOCUMENTATION_INDEX.md` in Cursor to give
the AI context about these APIs.

> 💡 **AI Assistant Hint:** For up-to-date, version-specific documentation on any library or framework mentioned in this index, use the Context7 MCP
server by appending `use context7` to your prompts. For example:
>
>
> - "How do I use FastAPI async endpoints? use context7"
> - "Show me React hooks patterns use context7"
> - "CMake best practices 2025 use context7"
>
> Context7 provides current documentation (2025), version-specific API references, and real code examples without hallucinations.

## Project Documentation

### Multi-programming-language codebase

- **`docs/MULTI_LANGUAGE_CODEBASE.md`** – Map of programming languages (C++, Python, Rust, Go, TypeScript, Swift) to directories, build/test/lint commands, and cross-language boundaries (proto, NATS, REST, ledger).

### TWS, ORATS, Client Portal & QuestDB (Quick Reference)

- **`docs/TWS_ORATS_PORTAL_QUESTDB.md`** – How **TWS API**, **ORATS**, **IB Client Portal**, and **QuestDB** fit in the platform: roles, ports, code locations, and data flow (Gateway → IB service → PWA; optional ORATS enrichment and QuestDB archiving).

### Planning & lifecycle

- **`docs/planning/BACKGROUND_TASK_LIFECYCLE.md`** – Current state and suggested improvements for background tasks (TUI threads, FastAPI lifespan, standalone scripts). Covers provider/aggregator lifecycle, health dashboard lifespan, Tastytrade DXLink, strategy runner, and Go collection-daemon fanout; suggests task registry, lifespan migration, and signal handlers.
- **`docs/planning/NATS_KV_REDIS_LIFECYCLE_TIMESCALE.md`** – Prioritized plan: NATS KV first, Redis later, background-task lifecycle, TimescaleDB optional.
- **`docs/planning/PRIMARY_CURRENCY_AND_TASE_HEDGING.md`** – Primary currency per account/portfolio (config, reporting, hedging). Hedging suggestions combining IB box spread (USD) with TASE put/call (TA-35, TA-125, ILS/USD options); data needs, logic, and implementation order.

### Box Spread Trading Guide

- **Comprehensive Box Spread Guide**: `docs/strategies/box-spread/BOX_SPREAD_COMPREHENSIVE_GUIDE.md`
  - Complete reference for box spread mechanics, risks, and implementation
  - Covers long/short box spreads, assignment risk, tax implications
  - Includes practical examples and implementation recommendations
  - Synthesizes information from multiple educational resources
  - **Note**: Box spreads are one strategy component of the Synthetic Financing Platform (7-10% spare cash allocation)

### 1Password (Secrets & credentials)

- **1Password Integration**: `docs/ONEPASSWORD_INTEGRATION.md`
- **Backend secrets providers (generate & configure)**: `docs/BACKEND_SECRETS_PROVIDERS.md`
  - CLI (`op`) and **formal API (SDKs)** for loading secrets
  - [1Password SDKs](https://developer.1password.com/docs/sdks/) – Go, JavaScript, Python; [Load secrets](https://developer.1password.com/docs/sdks/load-secrets)
  - This project: shell scripts use CLI (`scripts/include/onepassword.sh`); Israeli bank scrapers service optionally uses **JavaScript SDK** (`@1password/sdk`); optional Python helper uses **Python SDK** (`onepassword-sdk`)

### Device Task Delegation & Apple Intelligence

- **Device Task Delegation Guide**: `docs/DEVICE_TASK_DELEGATION.md`
  - Optimize workflow across multiple Apple devices (iPads, Macs)
  - Apple Intelligence integration for development and monitoring
  - Distributed compilation setup across all machines
  - Task delegation strategy for each device type
  - Practical workflows for development, monitoring, and testing

- **T-Bills and T-Bill Futures Guide**: `docs/T_BILLS_AND_FUTURES_GUIDE.md`
  - Comprehensive guide to Treasury Bills and T-bill futures
  - Risk-free rate calculations for option pricing
  - Box spread vs. T-bill arbitrage opportunities
  - Integration considerations for IB box spread application
  - Based on CME Group, Interactive Brokers, and Investopedia resources

- **Trading Infrastructure Guide**: `docs/TRADING_INFRASTRUCTURE.md`
  - VPS provider comparison (QuantVPS, TradingVPS, Ninja Mobile Trader)
  - Server operating system options (FreeBSD, Linux)
  - Latency optimization strategies
  - Deployment architecture recommendations
  - Cost analysis and ROI considerations
  - Based on Elite Trader resources and infrastructure best practices

- **Feature Tracking**: (archived; doc removed)
  - Comprehensive feature parity tracking between TUI and Web App
  - Feature status (implemented, partial, missing)
  - Implementation locations and notes
  - Feature gaps and priorities
  - Data schema alignment verification
  - Testing checklist for feature consistency

- **StockSharp Platform Analysis**: `docs/STOCKSHARP_ANALYSIS.md`
  - Comprehensive analysis of StockSharp C# trading platform
  - Architecture comparison with current project
  - Connector pattern insights and recommendations
  - Multi-broker integration strategies
  - Based on StockSharp GitHub repository and documentation

- **Ticker TUI Analysis**: `docs/TICKER_TUI_ANALYSIS.md`
  - Comprehensive analysis of ticker Go-based terminal UI
  - Comparison with current C++ FTXUI implementation
  - UI pattern insights and recommendations
  - Configuration and refresh strategy comparisons
  - Based on ticker GitHub repository

- **Open Trading Platform Analysis**: `docs/OPEN_TRADING_PLATFORM_ANALYSIS.md`
  - Comprehensive analysis of OTP microservices trading platform
  - Comparison with current multi-agent architecture
  - Kafka order management and service patterns
  - Market data distribution and aggregation insights
  - Based on OTP GitHub repository

- **FOSSology License Compliance Analysis**: `docs/FOSSOLOGY_ANALYSIS.md`
  - Comprehensive analysis of FOSSology license compliance tool
  - Comparison with existing scancode-toolkit setup
  - License scanning and copyright detection capabilities
  - SPDX generation and compliance workflow insights
  - Based on FOSSology official documentation

- **tasks.md Task Manager Analysis**: `docs/TASKS_MD_ANALYSIS.md`
  - Comprehensive analysis of tasks.md markdown task manager
  - Comparison with Todo2 MCP system and shared TODO table
  - Privacy-focused task management features
  - Markdown format and storage options
  - Based on tasks.md official website

- **Cline AI Coding Agent Analysis**: `docs/CLINE_ANALYSIS.md`
  - Comprehensive analysis of Cline open-source AI coding agent
  - Comparison with current Cursor AI assistant setup
  - Open-source transparency and model flexibility features
  - Client-side execution and privacy considerations
  - Based on Cline official documentation

- **FinceptTerminal Financial Platform Analysis**: `docs/FINCEPT_TERMINAL_ANALYSIS.md`
  - Comprehensive analysis of FinceptTerminal open-source financial platform
  - Comparison with current IBKR box spread project capabilities
  - CFA-level analytics, AI agents, and data connector features
  - Cross-domain intelligence and workflow builder capabilities
  - Based on FinceptTerminal GitHub repository

## Core Trading APIs

<!--
@index: api-documentation
@category: trading-apis
@tags: trading, broker-api, tws-api, alpaca, options
@last-updated: 2025-01-27
-->

### Interactive Brokers TWS API

- **Official Docs**: <https://interactivebrokers.github.io/tws-api/>
- **GitHub**: <https://github.com/InteractiveBrokers/tws-api>
- **Version**: 10.40.01 (updated from 10.33.01)
- **Release Notes**: <https://ibkrguides.com/releasenotes/prod-2025.htm>
- **Key Features (10.40)**:
  - ✅ Full Protocol Buffers support (all requests/responses)
  - ✅ Order Recovery: Automatic order resubmission on connection restore
  - ✅ Enhanced error handling

- **Protocol Buffers Migration**: See `docs/PROTOBUF_MIGRATION_PLAN.md` for future migration plan
- **Key Classes**:
  - `EClient` / `EClientSocket`: Client connection to TWS/Gateway
  - `EWrapper`: Callback interface (93+ methods)
  - `DefaultEWrapper`: Base implementation with default stubs
  - `Contract`: Security definition (stock, option, etc.)
  - `Order`: Order details (price, quantity, type)
  - `OrderState`: Order status and fills

- **Ports**:
  - `7497`: Paper Trading (TWS) - safe for testing
  - `7496`: Live Trading (TWS) - real money!
  - `4002`: Paper Trading (IB Gateway)
  - `4001`: Live Trading (IB Gateway)
  - **Note**: Different ports allow simultaneous access to both production and simulated accounts

- **Location**: `native/third_party/tws-api/IBJts/source/cppclient/client/`
- **Headers**: `native/include/ib_box_spread/tws_client.h`
- **Additional Features**:
  - **Fixed Income Orders**: Quantity in $1k units; bypass confirmation via API Precautions settings
  - **Realtime News**: Generic tick 292 for news; topic news via exchange parameter (use "\*" for symbol to list topics)
  - **News Providers**: Configure in Global Configuration → Pre-Configured API News Providers

- **IBKR Campus Resources**:
  - **TWS API Documentation (main)**: <https://www.interactivebrokers.com/campus/ibkr-api-page/twsapi-doc/>
  - **Sync API**: <https://www.interactivebrokers.com/campus/ibkr-api-page/twsapi-doc/#sync-api> – synchronous request/response patterns (aligns with our `request_*_sync` wrappers)
  - **EClient and EWrapper Architecture**:
    <https://www.interactivebrokers.com/campus/ibkr-quant-news/the-eclient-and-ewrapper-api-classes/> - Official explanation of EClient/EWrapper pattern
  - See also: `docs/ECLIENT_EWRAPPER_ARCHITECTURE.md` - Detailed architecture documentation based on IBKR Campus

### Alpaca Markets (API-First Brokerage Platform)

- **Official Website**: <https://alpaca.markets/>
- **Trading API Docs**: <https://docs.alpaca.markets/>
- **Broker API Docs**: <https://alpaca.markets/broker-api-docs/>
- **This project – Alpaca OAuth**: `docs/ALPACA_OAUTH.md` (client credentials flow, env and 1Password)
- **GitHub**: <https://github.com/alpacahq>
- **Founded**: 2015 (Yoshi Yokokawa, Hitoshi Harada)
- **Backing**: Y Combinator, Spark Capital, Tribe Capital, Horizon Ventures, Portage Ventures, Eldridge, Unbound
- **Description**: Developer-first, API-driven brokerage platform providing commission-free trading for U.S.
  stocks, ETFs, options, and cryptocurrencies. Serves over 5 million brokerage accounts and 200+ financial clients across 40 countries.
- **Key Products**:
  - **Trading API**: Execute trades algorithmically (stocks, options, crypto)
  - **Broker API**: End-to-end brokerage infrastructure for embedded trading
  - **Market Data API**: Real-time stock market and crypto data
  - **OAuth Integration**: Secure API access without exposing keys
  - **Paper Trading**: Risk-free testing environment

- **Key Features**:
  - **Commission-Free Trading**: No commissions for U.S.-listed securities and options via API
  - **Options Trading**: Multi-leg options for U.S.-listed equities and ETFs (commission-free)
  - **Crypto Trading**: Cryptocurrency trading support
  - **Margin and Short Selling**: Advanced strategies supported
  - **Alpaca Securities**: FINRA and SIPC member (up to $500,000 protection)
  - **Local Currency API**: Display prices/trades in local currencies for app localization
  - **Funding Wallets**: Deposit/withdraw in local currencies using local rails
  - **OmniSub Technology**: Omnibus technology for sub-accounting (Broker API)

- **Alpaca Elite Smart Router**:
  - **Documentation**: <https://docs.alpaca.markets/docs/alpaca-elite-smart-router>
  - **Target Audience**: Institutional clients and experienced algorithmic traders
  - **Key Features**:
    - **DMA Gateway (Direct Market Access)**:
      Control where orders are sent (NYSE, NASDAQ, ARCA, with plans for 10+ additional destinations including BATS, IEX, AMEX)
    - **Advanced Order Types**: VWAP (Volume-Weighted Average Price) and TWAP (Time-Weighted Average Price) orders
    - **Higher API Limits**: Enhanced rate limits for high-volume trading
    - **Cost-Effective Pricing**: Competitive pricing for institutional clients
  - **DMA Gateway Benefits**:
    - Efficiently manage large orders
    - Execution customization (choose target exchange)
    - Help minimize market impact
    - Meet specific trading objectives
    - Supports extended hours trading (pre-market and after-hours for NASDAQ and ARCA)
  - **DMA Gateway Parameters**:
    - `algorithm`: "DMA" (mandatory)
    - `destination`: Target exchange ("NYSE", "NASDAQ", "ARCA") (mandatory)
    - `display_qty`: Maximum shares/contracts displayed (optional, must be in round lot increments)
    - **Supported Orders**: Market and limit orders with Time in Force = "day" only
    - **Limitation**: Parameter replacement not supported; must cancel and resubmit
  - **VWAP Orders**:
    - **Purpose**: Execute at or near volume-weighted average price over specified time period
    - **Benefits**: Market impact management, benchmark alignment with volume trends
    - **Parameters**:
      - `algorithm`: "VWAP" (mandatory)
      - `start_time`: RFC3339 timestamp (optional, defaults to immediate or market open)
      - `end_time`: RFC3339 timestamp (optional, defaults to market close)
      - `max_percentage`: Maximum percentage of ticker's period volume (optional, 0 < max_percentage < 1)
    - **Calculation**: Total dollar value traded (price × volume) / total volume traded
  - **TWAP Orders**:
    - **Purpose**: Execute evenly over specified time period regardless of market volume
    - **Benefits**: Reduces market impact, execution predictability, effective in low-liquidity environments
    - **Parameters**:
      - `algorithm`: "TWAP" (mandatory)
      - `start_time`: RFC3339 timestamp (optional, does NOT participate in Open Auction)
      - `end_time`: RFC3339 timestamp (optional, does NOT participate in Close Auction)
      - `max_percentage`: Maximum percentage of ticker's period volume (optional, 0 < max_percentage < 1)
    - **Execution**: Equal-sized trades at regular intervals
  - **Implementation**: Advanced order types configured via `advanced_instructions` in order request payload
  - **Provider**: DMA Gateway provided by DASH Financial Technologies ("DASH"), member of listed exchanges
  - **Paper Trading**: `advanced_instructions` accepted in paper trading but orders not simulated
  - **Status**: Requires enrollment in Alpaca Elite Program (user has account)

- **SDKs & Languages**:
  - **Python**: `alpaca-py` (official SDK)
  - **.NET/C#**: Full SDK support
  - **Go**: SDK available
  - **Node.js/JavaScript**: SDK available
  - Additional language support

- **API Types**:
  - **REST API**: Standard HTTP RESTful interface
  - **WebSocket API**: Real-time streaming for market data and account updates
  - **FIX Protocol**: Available for institutional clients

- **Market Data**:
  - Real-time stock market data
  - Cryptocurrency market data
  - Historical data access
  - Options data (quotes, Greeks, chain data)

- **Authentication**:
  - **API Keys**: Traditional API key authentication
  - **OAuth 2.0**: Third-party app integration without exposing user keys

- **Testing Environment**:
  - **Paper Trading API**: Free paper trading environment for testing
  - **Sandbox**: Separate sandbox for Broker API development
  - No real money required for testing

- **Use Cases**:
  - Algorithmic trading systems
  - Embedded trading in fintech apps
  - Robo-advisors
  - Trading bots and automation
  - Box spread strategies (options support)
  - Multi-venue arbitrage

- **Relevance to This Project**:
  - **Alternative to IBKR**: Commission-free API trading platform
  - **Options Support**: Direct multi-leg options trading for box spreads
  - **Developer-Friendly**: Modern REST API vs TWS API's socket-based protocol
  - **Paper Trading**: Risk-free testing environment
  - **Multi-Venue**: Can complement IBKR for multi-broker strategies
  - **Lower Barrier**: Easier integration than TWS API (REST vs sockets)
  - **Elite Smart Router**: User has account with access to DMA Gateway and advanced order types (VWAP/TWAP)
  - **DMA Gateway**: Direct control over order routing for optimal execution on box spread strategies
  - **Advanced Order Types**: VWAP/TWAP can help manage large multi-leg option orders for box spreads

- **Integration Considerations**:
  - **Advantages**: Commission-free, REST API (easier than TWS sockets), modern documentation, OAuth support
  - **Elite Features**: User has account with DMA Gateway and advanced order types available
  - **DMA Gateway**: Direct exchange routing (NYSE, NASDAQ, ARCA) for better execution control
  - **VWAP/TWAP Orders**: Useful for managing large box spread orders while minimizing market impact
  - **Limitations**: Less flexible than IBKR for exotic instruments, smaller broker vs IBKR's global presence
  - **Options Trading**: Commission-free options (may be better cost structure than IBKR for high-volume box spread trading)
  - **Market Data**: Separate market data costs may apply
  - **Current Status**: Account available; integration pending

- **Cost Structure**:
  - **Trading**: Commission-free for U.S.-listed securities and options via API
  - **Market Data**: May require subscription for premium real-time data
  - **Broker API**: Platform fees may apply (contact Alpaca for pricing)

- **Awards & Recognition**:
  - Best API Award 2024 (Postman API Network Awards)
  - Fintech 250 List 2022
  - Best Automated Trading Software 2023
  - Best API Solution 2024
  - Best Broker for Sophisticated Traders 2025

- **Recent Product Launches**:
  - **OmniSub for Broker API**: Omnibus technology for sub-accounting
  - **Commission-free Options Trading**: Multi-leg options now available on Trading API
  - **Local Currency Trading API**: App localization support
  - **Funding Wallets**: Local currency deposit/withdrawal

- **Comparison with IBKR TWS API**:
  - **Alpaca**: REST API (simpler), commission-free options, modern developer experience, U.S.-focused
  - **IBKR**: Socket-based API (more complex), global market access, more instrument types, established infrastructure
  - **Best For**: Alpaca suits U.S.-focused algorithmic trading; IBKR suits global/institutional needs

- **Potential Integration Opportunities**:
  - Alternative execution venue for box spread strategies
  - Lower transaction costs for high-volume options trading
  - Simplified API integration for rapid prototyping
  - Multi-broker arbitrage (compare rates between Alpaca and IBKR)
  - Paper trading for strategy development before live trading on IBKR
  - **DMA Gateway**: Direct exchange routing for optimal execution on specific exchanges (NYSE, NASDAQ, ARCA)
  - **VWAP Orders**: Execute large box spread orders at volume-weighted average price to minimize market impact
  - **TWAP Orders**: Execute box spread orders evenly over time for predictable execution in low-liquidity scenarios
  - **Elite API Limits**: Higher rate limits for high-frequency box spread scanning and execution

- **Example Use Case**: Execute box spread strategies on Alpaca's commission-free options API with DMA Gateway for direct exchange routing (e.g.,
  route to NASDAQ for SPXW options).
  Use VWAP orders for large multi-leg box spread positions to minimize market impact.
  Compare execution prices and rates across both Alpaca (Elite) and IBKR platforms for optimal routing.
  For time-sensitive arbitrage, use TWAP orders to execute evenly over a specified window.

- **Note**: Alpaca is particularly well-suited for U.S.-focused algorithmic trading with a developer-friendly API.
The commission-free options trading could be advantageous for high-volume box spread strategies where transaction costs significantly impact
profitability.
With Elite Smart Router access, DMA Gateway provides direct exchange routing control, and VWAP/TWAP orders enable sophisticated execution algorithms
for large multi-leg positions.
  Consider Alpaca (Elite) as a complementary execution venue alongside IBKR for multi-venue strategies with enhanced execution control.

### Zorro Trading Platform

- **Official Docs**: <https://zorro-project.com/>
- **Manual**: <https://zorro-project.com/manual/>
- **Script Examples**: <https://zorro-project.com/scripts/>
- **Version**: Latest (free download)
- **Purpose**: Institutional-grade backtesting, optimization, and visualization for algorithmic trading
- **Key Features**:
  - Fast tick-level backtesting (10-year test in 0.3 seconds)
  - Walk-forward optimization (12-parameter system in <25 seconds)
  - Interactive option payoff diagrams
  - Multi-broker support (including IBKR)
  - C/C++ scripting interface
  - DLL interface for external integration

- **Integration Plan**: See `docs/ZORRO_INTEGRATION_PLAN.md` for comprehensive integration roadmap
- **Use Cases**:
  - Historical backtesting of box spread strategies
  - Parameter optimization using walk-forward analysis
  - Visualization of option payoff diagrams and strategy performance

- **License**: Free for personal use
- **Note**: Optional integration for enhanced backtesting and optimization capabilities

### Intel Decimal Floating-Point Math Library

- **Official Docs**: <https://www.intel.com/content/www/us/en/developer/articles/tool/intel-decimal-floating-point-math-library.html>
- **Version**: 20U2
- **Purpose**: Precision decimal arithmetic for financial calculations
- **Key Functions**: `__bid64_add`, `__bid64_div`, `__bid64_mul`, etc. (from libbid)
- **Location**: `native/third_party/IntelRDFPMathLib20U4/LIBRARY/libbid.a`
- **Note**: Required by TWS API for decimal price handling

### Protocol Buffers

- **Official Docs**: <https://protobuf.dev/>
- **C++ API**: <https://protobuf.dev/cpp/>
- **Version**: 6.33.0+
- **Purpose**: Serialization for TWS API messages
- **Generated Files**: `*.pb.cc`, `*.pb.h` in TWS API client directory
- **Location**: macOS: `/usr/local/lib/libprotobuf.dylib`; Linux: e.g. `/usr/lib/x86_64-linux-gnu/libprotobuf.so` (install `libprotobuf-dev`)

## Logging & Utilities

### spdlog

- **Official Docs**: <https://github.com/gabime/spdlog>
- **API Reference**: <https://spdlog.docsforge.com/>
- **Version**: Latest (via CMake FetchContent)
- **Usage**: `spdlog::info()`, `spdlog::error()`, `spdlog::warn()`, etc.
- **Key Features**: Fast, header-only, async logging, multiple sinks
- **Example**:

  ```cpp
  #include <spdlog/spdlog.h>
  spdlog::info("Connection established");
  spdlog::error("Failed to connect: {}", error_msg);
  ```

### CLI11

- **Official Docs**: <https://cliutils.github.io/CLI11/book/>
- **GitHub**: <https://github.com/CLIUtils/CLI11>
- **Version**: Latest (via CMake FetchContent)
- **Purpose**: Command-line argument parsing
- **Usage**: See `native/src/ib_box_spread.cpp` for examples

### nlohmann/json

- **Official Docs**: <https://json.nlohmann.me/>
- **GitHub**: <https://github.com/nlohmann/json>
- **Version**: Latest (via CMake FetchContent)
- **Purpose**: JSON parsing and serialization
- **Usage**: `json::parse()`, `json::dump()`, etc.

## Testing

### Catch2

- **Official Docs**: <https://github.com/catchorg/Catch2>
- **Documentation**: <https://github.com/catchorg/Catch2/blob/devel/docs/Readme.md>
- **Version**: Latest (via CMake FetchContent)
- **Usage**: `TEST_CASE()`, `REQUIRE()`, `CHECK()`, etc.
- **Location**: Tests in `native/tests/`

## Build System

### CMake

- **Official Docs**: <https://cmake.org/documentation/>
- **CMake Tutorial**: <https://cmake.org/cmake/help/latest/guide/tutorial/index.html>
- **Version**: 3.21+
- **Key Files**:
  - `CMakeLists.txt`: Main build configuration
  - `CMakePresets.json`: Build presets
  - `native/CMakeLists.txt`: Native code build

- **Presets**:
  - `macos-universal-debug`: Development build
  - `macos-universal-release`: Production build

- **Build parallelization and modularity**: `docs/BUILD_PARALLELIZATION_AND_MODULARITY.md` — How C++ (Ninja/CMAKE_BUILD_PARALLEL_LEVEL), Rust (Cargo workspace), and lint (--parallel) are parallelized; module layout for incremental builds.

### Abseil (Google C++ Libraries)

- **Official Docs**: <https://abseil.io/docs/cpp/>
- **GitHub**: <https://github.com/abseil/abseil-cpp>
- **Version**: 20250814+
- **Purpose**: Required dependency of Protocol Buffers
- **Location**: `/usr/local/lib/libabsl*.dylib`
- **Note**: 184+ individual libraries linked

## Python Integration

### Cython

- **Official Docs**: <https://cython.readthedocs.io/>
- **Purpose**: Python bindings for C++ code
- **Location**: `python/bindings/`
- **Build**: `cmake --build build --target python_bindings`

### Nautilus Trader

- **Official Docs**: <https://docs.nautilustrader.io/>
- **GitHub**: <https://github.com/nautechsystems/nautilus_trader>
- **Version**: 1.221.0+
- **Purpose**: Historical/deprecated framework reference only
- **Location**: Historical vendor/scaffold references only
- **Note**: Not part of the active supported runtime

## Market Data Providers

<!--
@index: api-documentation
@category: market-data
@tags: market-data, options, analytics, c++, fix-api, rest-api
@last-updated: 2025-01-27
-->

This section covers market data providers for real-time and historical financial data, options analytics, and market information.

**Quick Comparison**:

| Provider          | Focus          | API Types                    | Options Analytics | C++ Support   | Best For                         |
| ----------------- | -------------- | ---------------------------- | ----------------- | ------------- | -------------------------------- |
| **dxFeed**        | Multi-asset    | FIX, C++, Java, Python, REST | ✅ Greeks, IV     | ✅ Native C++ | C++ integration, FIX protocol    |
| **ORATS**         | Options        | REST API                     | ✅ Extensive      | ❌            | Options-specific analytics       |
| **Massive.com**   | Historical     | REST, WebSocket              | ⚠️ Limited        | ❌            | Historical data, backtesting     |
| **Alpha Vantage** | Multi-asset    | REST, MCP                    | ⚠️ Basic          | ❌            | Free tier, technical indicators  |
| **Finnhub**       | Multi-asset    | REST, WebSocket              | ⚠️ Basic          | ❌            | Generous free tier, fundamentals |
| **OpenBB**        | Financial Data | API                          | ⚠️ Unknown        | ❌            | Financial analytics platform     |

### dxFeed - Market Data Provider with FIX API and C/C++ APIs

- **Website**: <https://dxfeed.com/>
- **FIX API**: <https://dxfeed.com/api/fix-api/>
- **C/C++ and .NET APIs**: <https://dxfeed.com/api/c-and-net-apis/>
- **Provider**: dxFeed Solutions IE Limited (Devexperts)
- **Description**:
  Market data provider delivering financial market data and services to buy- and sell-side institutions in the global financial industry.
  Provides multiple API access methods including FIX API, C/C++ APIs, Java API, JavaScript/REST APIs, and Python API.
- **Key Features**:
  - **FIX API**: Electronic messaging protocol based on FIX protocol v4.4, widely adopted by financial institutions
  - **C/C++ and .NET APIs**: Graal C/C++ and .NET APIs for native integration
  - **Java API**: Java-based market data API
  - **JavaScript and REST APIs**: Web-based API access
  - **Python API**: Python integration for market data

- **Market Data Coverage**:
  - **Equities & ETFs**: Stock and ETF market data
  - **Futures**: Futures market data
  - **Options**: Options market data with analytics
  - **Indices**: Index data
  - **Fixed Income**: Fixed income instruments
  - **Forex**: Foreign exchange data
  - **Cryptocurrencies**: Cryptocurrency market data
  - **Spot**: Spot market data

- **Data Analytics**:
  - **Options Analytics**: Options-specific analytics and calculations
  - **Greeks and Implied Volatility**: Greeks (delta, gamma, theta, vega) and implied volatility data
  - **Market Indicators**: Market indicators and metrics

- **Reference Data**:
  - **Global Fundamentals**: Fundamental data for global markets
  - **Corporate Actions**: Corporate action data
  - **Trading Schedules**: Trading schedule information

- **Data Services**:
  - **Real-time and Delayed Data Service**: Real-time and delayed market data feeds
  - **Historical Data Services**: Historical market data access
    - **Order Historical Market Data**: <https://dxfeed.com/order-historical-market-data/>
    - **Sample Data**: Try out data presets and explore dxFeed market data samples
    - **Data Formats**: CSV format available for download
    - **Coverage**: Equities & ETFs, Equity Options, Futures and Futures Options, Indices, Fixed Income
    - **Data Types**: Bid/Ask, Last Sale, TnS (Trades and Quotes), Price Level Book, Full Order Depth, Minute bars
  - **Market Replay**: Market replay functionality for backtesting
  - **Aggregated Data Services**: Aggregated data services
  - **ORCS (Multidimensional Aggregation Service)**: Advanced aggregation service
  - **News Data Feed**: News data integration

- **Platform Integrations**:
  - **dxFeed ATAS**: Integration with ATAS (Advanced Trading Analysis Software) platform
    - **Order Page**: <https://get.dxfeed.com/orders/new/atas>
    - **Features**: dxFeed market data integration for ATAS trading platform
    - **Configuration**: Connect dxFeed data feed to ATAS for advanced trading analysis
    - **Use Case**: Use ATAS platform with dxFeed data for options trading analysis
  - **Option Traders Assistant**: dxFeed integration for options trading assistance
    - **Order Page**: <https://get.dxfeed.com/orders/new/optiontradersassistant>
    - **Purpose**: Specialized product for options traders using dxFeed market data
    - **Use Case**: Options trading assistance and analysis with dxFeed data
  - **Other Platforms**: dxFeed integrates with multiple trading platforms including Bookmap, NinjaTrader, TradingView, cTrader, and more

- **Geographic Coverage**:
  - **United States**: US market data
  - **European Union**: EU market data
  - **Turkey**: Turkish market data
  - **APAC**: Asia-Pacific market data
  - **Brazil**: Brazilian market data
  - **Australia**: Australian market data
  - **Global**: Global market coverage

- **Relevance to Box Spread Trading**:
  - **Options Market Data**: Comprehensive options data for box spread analysis
  - **Options Analytics**: Greeks and implied volatility for strategy validation
  - **FIX API**: Industry-standard FIX protocol for institutional integration
  - **C/C++ APIs**: Native C++ integration matches project technology stack
  - **Real-Time Data**: Real-time options data for live trading
  - **Historical Data**: Historical data for backtesting box spread strategies
  - **Multi-Asset Support**: Access to equities, options, futures, and forex for comprehensive market analysis

- **Integration Considerations**:
  - **C/C++ APIs**: Native C++ integration available (matches project stack)
  - **FIX API**: FIX 4.4 protocol for institutional-grade integration
  - **Multiple API Options**: Choose between FIX, C/C++, Java, Python, or REST based on needs
  - **Options Analytics**: Pre-calculated Greeks and IV can complement internal calculations
  - **Data Subscription**: Market data subscription required (contact for pricing)
  - **Institutional Focus**: Designed for buy- and sell-side institutions

- **Comparison with Current Solutions**:
  - **vs. TWS API**: dxFeed provides market data only (no trading), TWS provides both data and execution
  - **vs. ORATS**: Both provide options analytics; dxFeed offers FIX API and C++ APIs, ORATS focuses on REST API
  - **vs. Internal Calculations**: dxFeed provides pre-calculated Greeks/IV, can validate against internal calculations

- **Use Cases**:
  - Real-time options market data for box spread detection
  - Options analytics (Greeks, IV) for strategy validation
  - Historical data for backtesting box spread strategies
  - Multi-asset market data for comprehensive market analysis
  - FIX API integration for institutional-grade data feeds
  - C++ native integration for high-performance data processing
  - Platform integration via ATAS or Option Traders Assistant for visual analysis

- **Platform Integration Options**:
  - **ATAS Integration**: Use ATAS trading platform with dxFeed data for advanced options analysis
  - **Option Traders Assistant**: Specialized options trading tool with dxFeed data integration
  - **Direct API Integration**: Integrate dxFeed APIs (FIX, C/C++, Java, Python, REST) directly into custom trading systems

- **Contact**: Contact dxFeed sales for market data subscriptions and API access
- **Order Pages**:
  - **Historical Market Data**: <https://dxfeed.com/order-historical-market-data/>
  - **ATAS Integration**: <https://get.dxfeed.com/orders/new/atas>
  - **Option Traders Assistant**: <https://get.dxfeed.com/orders/new/optiontradersassistant>

- **Note**: dxFeed is a comprehensive market data provider with multiple API access methods including FIX API and native C/C++ APIs.
  Particularly relevant for options trading with pre-calculated Greeks and implied volatility.
  The C/C++ APIs provide native integration that matches the project's technology stack.
  dxFeed complements TWS API by providing additional market data sources and analytics, but does not provide trading execution capabilities.
  Evaluate as alternative/complement to ORATS for options market data and analytics.

### Massive.com REST API

- **Official Docs**: <https://massive.com/docs/rest/quickstart>
- **Base URL**: <https://api.massive.com> (verify in official docs)
- **Purpose**: Comprehensive historical and real-time market data from major U.S. exchanges
- **Key Features**:
  - Historical and real-time market data
  - Dividends, trades, quotes, fundamental data
  - Flat Files (CSV format) via S3-compatible interface
  - WebSocket API for real-time streaming
  - Client libraries: Python, Go, Kotlin, JavaScript

- **Authentication**: API key via query parameter or Authorization header
- **Data Coverage**: Major U.S. exchanges
- **Integration Opportunities**: See `docs/MASSIVE_INTEGRATION.md` for detailed integration analysis
- **Note**: Alternative/complement to ORATS for historical data and real-time quotes

### Alpha Vantage

- **URL**: <https://www.alphavantage.co/>
- **Official API Docs**: <https://www.alphavantage.co/documentation/>
- **MCP Server**: <https://www.alphavantage.co/> (MCP + AI Agents support)
- **Description**: Enterprise-grade stock market data API provider, backed by Y Combinator and officially licensed by NASDAQ
- **Key Features**:
  - Real-time and historical stock market data
  - Options, forex, cryptocurrency data
  - 60+ technical indicators
  - Economic indicators
  - Market news API with sentiment analysis
  - MCP (Model Context Protocol) server support for AI agents
  - Spreadsheet integration

- **Data Coverage**:
  - Traditional asset classes (stocks, ETFs, mutual funds)
  - Foreign exchange rates
  - Commodities
  - Fundamental data
  - Technical indicators
  - Global market data

- **Auth**: apiKey required (free tier available)
- **HTTPS**: Yes
- **API Limits**:
  - Free tier: 5 API calls per minute, 500 calls per day
  - Paid plans: Starting at $49.99/month with higher limits

- **Integration**:
  - REST API for direct integration
  - MCP server for AI agent integration
  - Spreadsheet add-ons
  - Python, JavaScript, and other language support

- **Relevance**:
  - Complements TWS API with additional market data sources
  - Useful for technical analysis with 60+ indicators
  - News sentiment analysis for market research
  - Cross-validation of TWS data
  - MCP support enables AI agent integration

- **Pricing**:
  - Free tier: Limited to 5 calls/minute, 500 calls/day
  - Premium: Subscription-based with higher limits
  - Enterprise: Custom pricing for high-volume usage

- **Partnerships**: Officially licensed by NASDAQ as a US market data provider

### Finnhub

- **URL**: <https://finnhub.io/>
- **Official API Docs**: <https://finnhub.io/docs/api>
- **API reference (quick)**:
  - Base: `https://finnhub.io/api/v1`
  - Auth: query param `token=<API_KEY>` (e.g. `?symbol=AAPL&token=xxx`)
  - Common: `/quote`, `/stock/candle`, `/stock/profile2`, `/company/news`, `/stock/recommendation`
  - WebSocket: `wss://ws.finnhub.io?token=<API_KEY>`
- **Description**: Comprehensive financial data API with real-time stock prices, fundamental data, news sentiment, and alternative data
- **Key Features**:
  - Real-time stock prices and quotes
  - Historical market data
  - Financial statements (income, balance sheet, cash flow)
  - Company fundamentals and profiles
  - News sentiment analysis (AI-powered)
  - Forex and cryptocurrency data
  - Alternative data sources
  - WebSocket support for real-time data

- **Data Coverage**:
  - Global stock markets
  - Options data
  - Forex pairs
  - Cryptocurrency markets
  - Economic indicators
  - Company fundamentals
  - News and sentiment

- **Auth**: apiKey required (free tier available)
- **HTTPS**: Yes
- **API Limits**:
  - Free tier: 60 API calls per minute (generous free tier)
  - Paid plans: Higher rate limits and advanced features

- **Integration**:
  - REST API
  - WebSocket API for real-time data
  - SDKs available for multiple languages (Python, JavaScript, Go, etc.)
  - Comprehensive documentation

- **Relevance**:
  - More generous free tier than Alpha Vantage (60 calls/min vs 5 calls/min)
  - Strong fundamental data for research
  - AI-powered sentiment analysis
  - WebSocket support for real-time updates
  - Options data available

- **Pricing**:
  - Free tier: 60 calls/minute (generous for free tier)
  - Paid plans: Higher limits and premium features
  - Enterprise: Custom pricing

### Massive (Market Data)

- **URL**: <https://massive.com/>
- **Official API Docs**: <https://massive.com/docs/>
- **REST Quickstart**: <https://massive.com/docs/rest/quickstart>
- **Full index (LLM-friendly)**: <https://massive.com/docs/llms.txt>
- **Official Python client**: <https://github.com/massive-com/client-python> — REST + WebSocket; install: `pip install -U massive`. Formerly Polygon.io; rebranded Oct 2025; API base defaults to `api.massive.com`; `api.polygon.io` still supported.
  - REST: `from massive import RESTClient` → `client = RESTClient(api_key="<API_KEY>")`; methods e.g. `list_aggs`, `get_last_trade`, `list_trades`, `get_last_quote`, `list_quotes`, `list_snapshot_options_chain` (with filter params `.gte`, `.lte`, etc.). Pagination on by default; use `pagination=False` for a fixed result count.
  - WebSocket: `from massive import WebSocketClient` → `WebSocketClient(api_key=..., subscriptions=["T.AAPL"])`; `ws.run(handle_msg=...)`. [Examples](https://github.com/massive-com/client-python/tree/master/examples/websocket).
  - API keys: <https://massive.com/dashboard/api-keys>
- **Description**: Market data API with REST and flat-file (S3) delivery for stocks, options, forex, crypto, indices, futures, and economy data.
- **Key Features**:
  - REST: snapshots (single ticker, full market, unified multi-asset), OHLC aggregates (custom bars, daily, previous day), trades/quotes, technical indicators (EMA, SMA, MACD, RSI), fundamentals (income, balance sheet, cash flow, ratios), corporate actions (dividends, splits, IPOs), SEC filings (10-K sections, 8-K, risk factors), news with sentiment
  - Economy: Treasury yields, inflation, inflation expectations, labor market
  - Partners: Benzinga (analyst ratings, news, earnings), ETF Global (analytics, constituents, fund flows), TMX (corporate events)
  - Flat files: daily/minute aggregates, trades, quotes as S3-downloadable files for crypto, forex, indices, options, stocks
- **Data Coverage**:
  - U.S. equities, options, indices; forex (1,750+ pairs); crypto; futures
  - Nanosecond timestamps for trades/quotes; ET/CT/UTC timezone options per asset class
- **Auth**: API key (typical for REST); check docs for headers/query params
- **Relevance**:
  - Single-ticker and full-market snapshots for cross-validation with TWS/Alpha Vantage/Finnhub
  - Treasury yields endpoint for risk-free rate / discount curve use cases
  - Options aggregates and option chain snapshot for box spread research
  - Flat files for backtesting and bulk historical analysis

### OpenBB - Financial Data Platform

- **Website**: <https://openbb.co/>
- **Description**: Financial data platform providing access to market data, financial information, and analytics
- **Key Features**:
  - **Market Data**: Access to financial market data
  - **Analytics**: Financial analytics and insights
  - **Data Integration**: Integration with multiple data sources
  - **API Access**: API for programmatic access to financial data

- **Relevance to Box Spread Trading**:
  - **Market Data**: Access to market data for box spread analysis
  - **Financial Analytics**: Analytics tools for strategy development
  - **Data Integration**: Multiple data sources for comprehensive analysis

- **Note**: OpenBB provides financial data and analytics platform.
  Evaluate for market data access and analytics capabilities relevant to box spread trading strategies.

### Bank of Israel (BOI) - Economic Data API

- **Official Website**: <https://www.boi.org.il/en/>
- **New Website**: <https://www.boi.org.il/en/the-bank-of-israel-s-new-website/>
- **Series Database**: <https://edge.boi.gov.il>
- **Contact**: <https://boi.org.il/en/contact-us/>
- **Description**: Central bank of Israel providing economic data APIs including interest rates, exchange rates, and financial indicators.
  APIs are updated multiple times daily and provide real-time access to Israeli economic data.
- **Key Features**:
  - **Exchange Rates API**: Up-to-date exchange rate data (updated multiple times daily)
  - **Economic Data API**: Interest rates, financial indicators, and economic statistics
  - **Series Database**: External database at `edge.boi.gov.il` for querying data series
  - **Live Data Export**: Export information live through the API
  - **Metadata Access**: View metadata for data series
  - **Graphs and Charts**: Generate visualizations from data

- **Data Coverage**:
  - **Interest Rates**: Bank of Israel interest rates (SHIR - Shekel Interest Rate)
  - **Exchange Rates**: Currency exchange rates
  - **Economic Indicators**: Various economic statistics
  - **Monetary Policy Data**: FOMC-equivalent data (Monetary Committee decisions)
  - **Financial Market Data**: Market-related statistics

- **Update Frequency**: Multiple times daily
- **API Access**:
  - APIs available through BOI's official website
  - External series database at `edge.boi.gov.il` for programmatic access
  - Dashboard technology for querying and downloading data files
  - Statistics section provides data access

- **Open Banking**: BOI promotes open banking practices, encouraging banks to provide secure API connections
- **Use Cases**:
  - Access Israeli interest rates for risk-free rate calculations
  - Monitor exchange rates (USD/ILS, EUR/ILS, etc.)
  - Economic data for trading strategies involving Israeli markets
  - Integration with trading systems requiring Israeli economic indicators
  - Research and analysis of Israeli financial markets

- **Relevance to This Project**:
  - **Risk-Free Rate**: Israeli interest rates could be used as risk-free rate benchmark for options pricing
  - **Currency Hedging**: Exchange rate data for USD/ILS hedging strategies
  - **Market Analysis**: Economic indicators for understanding Israeli market conditions
  - **Multi-Currency Trading**: Support for traders operating in Israeli markets

- **Documentation**: Available on BOI website; detailed API documentation and guidelines provided
- **Note**: BOI has been actively developing APIs to enhance transparency and facilitate access to financial data.
  The new website includes improved API services and data accessibility.

### Bank of Israel - Digital Shekel (CBDC) APIs

- **Official Website**: <https://www.boi.org.il/en/>
- **Digital Shekel Challenge**: <https://www.boi.org.il/en/NewsAndPublications/PressReleases/Pages/24-10-24-Digital-Shekel-Challenge.aspx>
- **API Documentation**:
  <https://www.boi.org.il/media/mi1llyjv/%D7%95%D7%AA%D7%A8%D7%97%D7%99%D7%A9%D7%99-%D7%A9%D7%99%D7%9E%D7%95%D7%A9-%D7%9E%D7%A8-%D7%90%D7%9E%D7%99%D7%A8-%D7%9E%D7%A9%D7%94-apis.pdf>
- **Description**: Central Bank Digital Currency (CBDC) APIs for the Digital Shekel (שקל דיגיטלי) payment system.
  Part of BOI's Digital Shekel Challenge initiative to develop innovative use cases and payment solutions.
- **Architecture**: Two-tier model (Two-Tier Model)
  - **Bank of Israel**: Exclusive authority for issuance/burning of Digital Shekel, system management
  - **Payment Service Providers (DS-PSP)**: Provide technological access for end users, KYC processes, wallet services, customer support
  - **Funding Institutions (FI)**: Financial institutions supporting conversion between cash, digital money, and Digital Shekel
  - **Additional Service Providers (ASP)**: Optional advanced services (budget management, conditional payments, etc.)

- **Key API Categories**:
  - **Basic Functionality APIs**: Support for end-user journey
    - Wallet operations (Open Wallet, Connect FI, Connect ASP)
    - Payment operations
    - Account management
    - Transaction history
    - Balance inquiries
  - **Advanced Platform APIs**: Innovation support platform
    - Sub-wallet management
    - Alias services (phone number-based wallet lookup)
    - Conditional payments
    - Programmable money features
    - Delivery vs. Payment (DvP) functionality

- **API Features**:
  - **Open Wallet**: Create and manage Digital Shekel wallets
  - **Connect FI**: Connect to Funding Institutions for conversion
  - **Connect ASP**: Connect to Additional Service Providers
  - **Payment Operations**: Send and receive Digital Shekel payments
  - **Alias Management**: Create, delete, and lookup wallet aliases (phone number-based)
  - **Sub-Wallet Management**: Create and manage linked sub-wallets
  - **Transaction History**: Query transaction history and balances

- **Innovative Use Cases**:
  - **Delivery vs. Payment (DvP)**: Atomic settlement of assets and payments
  - **Linked Sub-Wallets**: Hierarchical wallet structures for budgeting or organizational purposes
  - **Split Payments**: Advanced payment splitting functionality
  - **Conditional Payments**: Programmable money with conditions
  - **Programmable Finance**: Smart contract-like functionality for financial services

- **Challenge Context**:
  - Part of BOI's Digital Shekel Challenge competition
  - Participants can simulate multiple participant types (DS-PSP, ASP, FI)
  - Default challenge setup includes 2 DS-PSPs, 1 ASP, and 1 FI per participant
  - BOI simulates 3 FI institutions in the challenge

- **Use Cases**:
  - CBDC payment system integration
  - Digital wallet development
  - Payment service provider integration
  - Financial innovation and DeFi applications
  - Programmable money and smart contract functionality
  - Atomic settlement and DvP operations

- **Relevance to This Project**:
  - **Payment Integration**: Potential for integrating Digital Shekel payments into trading systems
  - **Settlement**: DvP functionality for atomic asset settlement
  - **Multi-Currency**: Support for Digital Shekel as additional currency option
  - **Innovation Platform**: APIs for developing financial innovation solutions

- **Documentation**: PDF documentation available from BOI website with API specifications and use case examples
- **Status**: Challenge/development phase - APIs available for Digital Shekel Challenge participants
- **Note**: This is separate from the Economic Data APIs.
  The Digital Shekel APIs are for CBDC payment system integration, while Economic Data APIs are for accessing economic statistics and market data.

### Discount Bank (Israel) - Open Banking API

- **Developer Portal**: <https://developer.discountbank.co.il/openapi/>
- **Official Website**: <https://www.discountbank.co.il/>
- **Contact**: <abuse@dbank.co.il> (for cyber security incidents)
- **Description**: Discount Bank's Open Banking API portal providing access to banking services and financial data through REST APIs.
  Part of Israel's open banking initiative, allowing third-party providers to access banking services with customer consent.
- **Key Features**:
  - **Open Banking APIs**: RESTful APIs for banking services
  - **Sandbox Environment**: Sandbox configuration for testing
  - **Implementer Options**: Multiple implementation options available
  - **Corporate Services**: Business banking APIs (עסקים פלוס - Businesses Plus)
  - **Authorization Management**: Corporate authorization and signature management

- **API Access**:
  - Developer portal registration required
  - Sandbox environment for testing
  - Production access requires corporate authorization
  - Corporate authorization requires authorized signatories approval

- **Corporate Authorization**:
  - Requires authorization of signatories for open banking operations
  - Multiple signatories require all to approve consent
  - Authorization form available through "עסקים פלוס" (Businesses Plus) service
  - Separate form required for each corporation

- **Use Cases**:
  - Account information access
  - Payment initiation services
  - Transaction history retrieval
  - Balance inquiries
  - Corporate banking automation
  - Financial data aggregation

- **Relevance to This Project**:
  - **Multi-Currency Trading**: Access to ILS account information for Israeli traders
  - **Settlement**: Potential integration for ILS-denominated settlement
  - **Account Management**: Automated account monitoring and management
  - **Israeli Market Integration**: Support for traders operating in Israeli markets

- **Security**:
  - Cyber security incident reporting: <abuse@dbank.co.il>
  - Contact requests through portal for security incidents
  - Corporate authorization required for production access

- **Maintenance**: Scheduled maintenance windows announced (e.g., November 22nd, 23:00-02:00)
- **Documentation**: Available through developer portal; sandbox configuration and implementer options provided
- **Note**: Part of Israel's open banking ecosystem,
  allowing third-party financial service providers to access banking data and services with proper authorization and customer consent.
  Useful for Israeli traders requiring automated account access and ILS-denominated operations.

### Bank Jerusalem (בנק ירושלים) - API Portal

- **API portal**: <https://apiportal.bankjerusalem.co.il/>
- **API catalog**: <https://apiportal.bankjerusalem.co.il/api_catalog> (catalog path may require login or vary)
- **Official Website**: <https://www.bankjerusalem.co.il/>
- **Description**: Bank Jerusalem (בנק ירושלים) API portal for developer access to banking APIs. Part of the Israeli banking landscape; portal may require registration or login.
- **Relevance**: Additional Israeli bank for multi-account aggregation or open-banking integration if supported; independent provider in the same sense as Discount, FIBI, and Israeli bank scrapers companies.

### Israeli Bank Scrapers (Node) - israeli-bank-scrapers

- **GitHub**: <https://github.com/eshaham/israeli-bank-scrapers>
- **NPM**: `israeli-bank-scrapers`
- **Description**: Node/TypeScript library using Puppeteer to scrape Israeli bank and credit-card websites (when Open Banking is not available). Supports **Fibi**, **Max**, **Visa Cal**, Bank Hapoalim, Leumi, Discount, Mercantile, Mizrahi, Otsar Hahayal, Union, Beinleumi, Massad, Yahav, Isracard, Amex, and others. Use `companyId` (e.g. `fibi`, `max`, `visaCal`, `discount`) when calling the scrapers service.
- **Integration in This Project**:
  - **Service**: `services/israeli-bank-scrapers-service` — runs scrapers on-demand (HTTP POST /scrape or CLI), maps results to the shared ledger (`Assets:Bank:{BankName}:{accountNumber}`), so existing Discount Bank service and TUI/Web continue to show accounts via GET /api/bank-accounts.
  - **Port**: 8010 (config: `services.israeli_bank_scrapers.port`).
  - **Endpoints**: GET /api/health, POST /scrape (credentials via env only).
- **Use Cases**: Alternative to Open Banking when corporate auth is not in place; multi-bank and credit-card aggregation; cron-based sync into ledger.
- **License**: MIT.

### First International Bank of Israel (FIBI) - Open Banking API

- **Developer Portal (Sandbox)**: <https://devapi.test.fibi.co.il/fibi/sb/api>
- **Official Website**: <https://www.fibi.co.il/>
- **Description**: First International Bank of Israel's Open Banking API portal providing NextGenPSD2-compliant APIs for account information,
  payments, and securities accounts.
  Part of Israel's open banking initiative following European PSD2 standards.
- **API Framework**: NextGenPSD2 XS2A Framework
  - Modern, open, harmonized, and interoperable set of APIs
  - Reduces XS2A (Access to Account) complexity and costs
  - Addresses multiple competing standards in Europe
  - Aligned with Euro Retail Payments Board goals
  - Enables "Banking as a Service" through secure TPP (Third Party Provider) access

- **Available APIs**:
  - **NextGenPSD2 XS2A Framework**: Core account access and payment services
    - Refresh Token API (v1.0.0, v1.0.1)
    - Account Information Services (AIS)
    - Payment Initiation Services (PIS)
  - **Card Information**: Card account access and information
    - Single Cards API (v1.7.0, v1.8.0)
    - Card-specific consent management
    - Access to both cards and card accounts
  - **Savings and Loans**: Savings and loan account access
    - Savings Accounts API (v1.4.0, v1.5.0)
    - Loan Accounts API
    - IBAN-based account identification
    - Cash account type specification (SVGS/LOAN)
  - **Securities Accounts**: Securities account information
    - Securities Accounts API (v1.0.0, v1.1.0)
    - Extension of NextGenPSD2 XS2A specification
    - Securities account information services
  - **Sandbox Bypass**: Testing and development utilities (v1.8.0)

- **API Categories**:
  - **Account Information**: Account balances, transactions, account details
  - **Identity and Security**: Authentication and authorization
  - **Payments**: Payment initiation and processing
  - **Public Information**: Publicly available banking information

- **Consent Management**:
  - Bank-offered consent (simplified)
  - Detailed consent (granular access control)
  - Card-specific consent for card accounts
  - Savings/loan-specific consent with cashAccountType specification
  - Consent-based access control for all services

- **Use Cases**:
  - Account information aggregation
  - Payment initiation services
  - Card transaction monitoring
  - Savings and loan account management
  - Securities account information
  - Financial data aggregation for trading systems

- **Relevance to This Project**:
  - **Account Monitoring**: Automated monitoring of ILS-denominated accounts
  - **Settlement**: Payment initiation for ILS-denominated settlement
  - **Securities Integration**: Access to securities account information for trading
  - **Multi-Bank Aggregation**: Combine data from multiple Israeli banks
  - **Israeli Market Integration**: Support for traders operating in Israeli markets

- **Environment**: Sandbox environment available for testing
- **Standards Compliance**: NextGenPSD2 framework (European PSD2 adapted for Israel)
- **Documentation**: Available through developer portal with API specifications and examples
- **Note**: FIBI provides comprehensive open banking APIs following NextGenPSD2 standards, including specialized APIs for cards, savings, loans,
  and securities accounts.
  Useful for Israeli traders requiring integrated banking services and securities account access alongside trading operations.

### Finanda Smart Aggregation - Open Banking API Aggregator

- **Official Website**: <https://www.finanda.com/open-banking/>
- **Description**: Finanda Smart Aggregation is Israel's veteran and experienced open banking aggregator,
  the first to provide Open Banking API services to information consumers in the banking system.
  Licensed by the Israel Securities Authority to provide financial information services.
- **Key Features**:
  - **Multi-Bank Aggregation**: Integrates with all major Israeli banks
    - Bank Hapoalim (בנק הפועלים)
    - Mizrahi Tefahot (מזרחי טפחות)
    - Otsar HaHayal (אוצר החייל)
    - Bank Leumi (בנק לאומי)
    - FIBI (First International Bank of Israel)
    - Discount Bank
  - **PSD2 Compliance**: Adapted to Open Banking regulation (PSD2)
  - **Smart Data Processing**: Processes and categorizes data from tens of thousands of accounts and millions of transactions
  - **Financial Data Quality**: High-quality financial data aggregation
  - **Over 10 Years Experience**: Proven technology in use by banking system information consumers

- **Authorization Server Endpoints**:
  - **FIBI**: `https://api.test.fibi.co.il/fibi/`
  - **Discount Bank**: `https://mtls-api-nonprod.discountbank.co.il/devapi/`
  - **Bank Hapoalim**: `https://openbankingsb.poalim-api.co.il/xs2a/`
  - **Mizrahi Tefahot**: `https://sboapi.mizrahi-tefahot.co.il/accesscodeoauthprovider/`

- **Consent Management**:
  - Consent status tracking (received, rejected, valid, expired, etc.)
  - Partially authorized consent support
  - Revocation by PSU (Payment Service User)
  - Termination by TPP (Third Party Provider)
  - Suspension/blocking by ASPSP (Account Servicing Payment Service Provider)

- **Data Buckets**:
  - **Accounts**: Account information
  - **Balances**: Account balances
  - **Transactions**: Transaction history

- **Use Cases**:
  - Financial data for decision making
  - Credit assessment assistance
  - Product comparison and cheaper offers for customers
  - Fraud detection and risk reduction
  - Smart investment knowledge base
  - Advanced smart categorizations
  - Financial improvement control systems
  - Economic empowerment leverage
  - Smart financial management for businesses
  - Income and expense information
  - Smart product matching according to financial needs
  - Transition from form-based to technology-based processes

- **Security & Compliance**:
  - **Licensed**: Licensed by Israel Securities Authority for financial information services
  - **Banking Standards**: Secure, backed-up systems meeting banking standards
  - **Regulatory Compliance**: Compliance with regulatory requirements
  - **24/7 Monitoring**: Continuous system monitoring and activity tracking
  - **Management Portal**: Dedicated management portal for control and oversight

- **Deployment Options**:
  - **Cloud Deployment**: Cloud deployment options
  - **Private Cloud**: Private cloud deployment in your organization
  - **Finanda Cloud**: Deployment via Finanda's cloud using Amazon Web Services

- **Integration**:
  - Fast implementation based on proven system
  - Used by information consumers in the banking system
  - Quick integration with all data sources according to regulation

- **Relevance to This Project**:
  - **Multi-Bank Aggregation**: Single API access to multiple Israeli banks
  - **Account Monitoring**: Automated monitoring across multiple bank accounts
  - **Transaction Analysis**: Comprehensive transaction history from all banks
  - **Financial Data**: High-quality financial data for trading decisions
  - **ILS Operations**: Support for ILS-denominated operations across banks
  - **Risk Management**: Fraud detection and risk reduction capabilities

- **Documentation**: Available through Finanda website; API documentation and integration guides provided
- **Note**: Finanda provides a unified API layer over multiple Israeli banks,
  simplifying integration for developers who need access to multiple banking institutions.
  Particularly useful for Israeli traders requiring comprehensive financial data aggregation and multi-bank account management.

### Yael Group - Open Banking Solutions & Consulting

- **Official Website**: <https://yaelgroup.com/solution_category/open-banking-solutions/>
- **Description**: Yael Group provides open banking solutions and consulting services for implementing open banking in Israel.
They specialize in helping financial institutions comply with Israeli open banking regulations and implement the Berlin Standard (תקן ברלין)
published by the Bank of Israel.
- **Key Services**:
  - **Open Banking Implementation**: Implementation of open banking solutions
  - **Regulatory Compliance**: Assistance with compliance to Israeli Financial Information Services Law (חוק שירות מידע פיננסי, November 2021)
  - **Berlin Standard Implementation**: Expertise in the Berlin Standard published by Bank of Israel
  - **API Management**: API management solutions for open banking
  - **Banking Mobility**: Mobile banking solutions
  - **Credit Reporting**: Credit reporting integration
  - **Integration Services**: Integration and API management services

- **Regulatory Context**:
  - **Financial Information Services Law**: Israeli law (November 2021) enabling open banking
  - **Berlin Standard**: Standard published by Bank of Israel for open banking implementation
  - **Capital Market, Insurance and Savings Authority**: Regulatory body for open banking in Israel
  - **Guidelines**: Guidelines published in March 2023 for institutional financial information sources

- **Regulated Entities**:
  - Banks
  - Credit card companies
  - Non-bank credit providers
  - Insurance companies
  - Pension fund management companies

- **Standards & Compliance**:
  - **Berlin Standard**: Bank of Israel standard for open banking
  - **Unified Standard**: Maintains uniform standard across financial information market
  - **Innovative Technology Environment**: Fast and easy access to accurate and up-to-date information
  - **Regulatory Requirements**: Compliance with Capital Market, Insurance and Savings Authority requirements

- **Use Cases**:
  - Open banking implementation for financial institutions
  - Regulatory compliance consulting
  - API management and integration
  - Banking mobility solutions
  - Credit reporting integration
  - Open finance solutions

- **Relevance to This Project**:
  - **Implementation Support**: Consulting for implementing open banking integrations
  - **Regulatory Understanding**: Understanding Israeli open banking regulations
  - **Standards Compliance**: Implementation of Berlin Standard for open banking
  - **Integration Services**: API management and integration support
  - **Multi-Bank Integration**: Support for integrating with multiple Israeli banks

- **Documentation**: Available through Yael Group website; information about open banking solutions and regulatory compliance
- **Note**: Yael Group is a consulting and implementation company, not an API provider.
  They help organizations implement open banking solutions and comply with Israeli regulations.
  Useful for understanding the Israeli open banking regulatory landscape and finding implementation support for open banking integrations.

### Meitav Dash - OpenHub Open Banking API

- **Developer Portal (Sandbox)**: <https://portal.openhub-sbx.meitav.co.il/instructions>
- **Official Website**: <https://www.meitav.co.il/>
- **Description**:
  Meitav Dash (מאית דש) is an Israeli investment management company providing OpenHub developer portal for open banking API integration.
  The sandbox environment provides instructions and resources for integrating with Meitav's open banking APIs.
- **Key Features**:
  - **Open Banking APIs**: Open banking API access through OpenHub portal
  - **Sandbox Environment**: Sandbox portal for testing and development
  - **Developer Resources**: Instructions and documentation for API integration
  - **Investment Management**: Integration with investment management services

- **API Access**:
  - Sandbox portal available at `portal.openhub-sbx.meitav.co.il`
  - Instructions and documentation through developer portal
  - JavaScript required for portal access

- **Use Cases**:
  - Investment account information access
  - Portfolio data integration
  - Investment management services integration
  - Financial data aggregation for investment accounts

- **Relevance to This Project**:
  - **Investment Accounts**: Access to investment account information
  - **Portfolio Integration**: Integration with investment portfolios
  - **Multi-Provider**: Additional Israeli financial services provider
  - **Trading Integration**: Potential integration with investment management services

- **Documentation**: Available through OpenHub sandbox portal; instructions and API documentation provided
- **Note**: Meitav Dash is an investment management company providing open banking APIs through their OpenHub portal.
  Part of the Israeli open banking ecosystem, providing access to investment account information and portfolio data.
  Useful for Israeli traders requiring integration with investment management services alongside trading operations.

### Ordernet Spark API (Nesua, Meitav, Psagot) - ordernet-api

- **GitHub**: <https://github.com/assafmo/OrdernetAPI>
- **NPM**: `ordernet-api` ([npm](https://www.npmjs.com/package/ordernet-api))
- **API base**: `https://spark{nesua,meitav,psagot}.ordernet.co.il/api` — Spark system used by Israeli brokers Nesua, Meitav, and Psagot.
- **Description**: CLI and Node library for querying the Ordernet Spark API. Authenticate and fetch account list and total balance per account. Alternative to scraping for Meitav (and Nesua, Psagot) when Spark API is available.
- **Brokers supported**: `nesua`, `meitav`, `psagot` (choice via `--broker` or `authenticate(..., broker)`).
- **Endpoints used**: `/api/Auth/Authenticate`, `/api/DataProvider/GetStaticData`, `/api/Account/GetAccountSecurities`.
- **Library API**: `authenticate(username, password, broker)` → then `getAccounts()`, `getAccountBalance(account)`, `accountKeyToNumber(key)`. Account keys format `ACC_XXX-YYYYYY`.
- **CLI**: `ordernet-api -u USER -p PASS -b meitav`; optional `-a ACC_XXX-YYYYYY` to limit to specific account(s); `-v` verbose.
- **Relevance**: Meitav is in this project's `broker.priorities`; OrdernetAPI offers a programmatic way to get Meitav (and Nesua, Psagot) balances/accounts via Spark instead of or in addition to scrapers/OpenHub. Independent provider pattern; can run in parallel with other brokers.
- **Ref**: [ordernet-scraper](https://github.com/danielbraun/ordernet-scraper) (related scraper approach).

### Spark-Ordernet client (TypeScript) - spark-ordernet-client

- **GitHub**: <https://github.com/itamarco/spark-ordernet-client>
- **NPM**: `spark-ordernet-client` ([npm](https://www.npmjs.com/package/spark-ordernet-client))
- **Description**: TypeScript/JavaScript REST client for the same Spark-Ordernet API. Authenticate once, then fetch **transactions** (from a start date) and **holdings** (positions with symbol, quantity, price, cost, margin, etc.). Richer data than OrdernetAPI (which focuses on account list + total balance).
- **Tested with**: `sparknesua.ordernet.co.il`, `sparkpsagot.ordernet.co.il` (same host pattern as Meitav: `sparkmeitav.ordernet.co.il`).
- **Usage**: `new SparkClient({ sparkHost: 'https://sparkpsagot.ordernet.co.il', userId, password, accountKey?, logger? })` → `await client.auth()` → optional `getAccountKey()` / `setSparkAccountKey()` → `getTransactions(fromDate)`, `getHoldings()`.
- **Returns**: Transactions (Account, Date, Bno_Number, Ref, Action, Balance, Price, NetCredit, NetDebit, etc.); holdings (ID, BNO, SYMBOL_NAM, BNO_NAME, PRC, NV, COST, VL, EXT_MARGIN, REQ_MARGIN, etc.).
- **Relevance**: Alternative or complement to [OrdernetAPI](#ordernet-spark-api-nesua-meitav-psagot---ordernet-api) when you need transaction history and holdings detail from Nesua/Psagot (and likely Meitav via same API). Same independent-provider pattern.

### StoreNext Meteor - Financial Data Import & Open Banking Platform

- **Official Website**: <https://www.storenext.co.il/en/e-commerce/meteor/>
- **Description**:
  StoreNext Meteor is an automated financial data import platform that
  aggregates data from all of an organization's financial databases in Israel and around the world.
The platform processes and streams financial data in a uniform structure through a single central path to enterprise information systems, fully
integrated with ERP systems or treasury management.
- **Key Features**:
  - **Bank Account Aggregation**: Complete overview of balances and transactions from all bank accounts and subsidiaries
  - **Open Banking**: Open banking integration for financial data access
  - **Cash Positioning**: Cash management and positioning services
  - **Foreign Exchange Rates Import**: Import foreign exchange rates from worldwide sources
  - **BANKonnect**: Multibank payment system for secure payment instructions
  - **Cash Management**: Cash flow forecasting and analysis

- **BANKonnect - Multibank Payments**:
  - Internet-cloud system for secure payment instructions
  - Payment instructions from Israeli banks transmitted abroad
  - Internal approval process management
  - Advanced fraud prevention controls
  - Digital certificate workflow management
  - Security controls and beneficiary management
  - Withholding tax processes and VAT reporting
  - Secure private cloud with peripheral security
  - ERP2Bank and WEB2Bank interfaces

- **Bank Account Aggregation**:
  - Complete overview of all organization bank accounts (Israel and abroad)
  - Access to data based on client-defined permissions
  - Broad financial picture of all company accounts at all banks
  - Transaction history and balance information

- **Cash Management**:
  - Cash flow forecasting including retrospective analysis
  - Cumulative information generation
  - Analysis of material performance trends
  - Ongoing control of gap between forecast and actual execution

- **Foreign Exchange Rates Import**:
  - Import foreign exchange rates from worldwide sources
  - Access to central bank databases in different countries
  - Information from commercial banks
  - Exchange rate databases including cross-rates
  - Various indices (metal prices, stock exchanges)
  - Direct integration with corporate ERP systems

- **Data Processing**:
  - Automated data import from financial databases
  - Data processing and streaming in uniform structure
  - Single central path to enterprise information systems
  - Full integration with ERP systems or treasury

- **Use Cases**:
  - Daily connection with financial institutions (banks, credit cards, investment companies)
  - Accurate and relevant financial status picture
  - Ongoing and continuous flow of data from multiple financial sources
  - Digital-financial transformation
  - Multibank payment management
  - Cash flow management and forecasting
  - Foreign exchange rate monitoring

- **Relevance to This Project**:
  - **Multi-Bank Aggregation**: Single platform for multiple bank accounts
  - **Cash Management**: Cash positioning and flow forecasting for trading operations
  - **FX Rates**: Foreign exchange rate data for multi-currency trading
  - **Payment Processing**: Multibank payment capabilities for settlement
  - **ERP Integration**: Integration with enterprise systems
  - **Financial Data**: Comprehensive financial data aggregation for trading decisions

- **Industries Served**:
  - Construction
  - Retail FMCG
  - Pharmaceuticals
  - Automotive
  - Catering
  - Recycling
  - Hi-Tech & Electronics

- **Documentation**: Available through StoreNext website; information about Meteor platform and integration options
- **Note**: StoreNext Meteor provides a comprehensive financial data aggregation and open banking platform,
  particularly useful for organizations requiring integration with multiple financial institutions and ERP systems.
  Useful for Israeli traders and businesses requiring automated financial data import, cash management, and multibank payment processing.

### Tel Aviv Stock Exchange (TASE) - Market Data Vendors

- **Official Website**: <https://www.tase.co.il/en>
- **Data Vendors Page**: <https://www.tase.co.il/en/content/about/data_vendors>
- **Description**: The Tel Aviv Stock Exchange (TASE) is Israel's primary stock exchange, providing market data through authorized data vendors.
  TASE offers real-time and historical market data for Israeli equities, ETFs, bonds, and other securities.
- **Market Data Access**:
  - **Authorized Data Vendors**: TASE provides market data through various authorized data vendors
  - **Real-Time Data**: Real-time market data feeds
  - **Historical Data**: Historical market data access
  - **Global Distribution**: Market data available globally through vendor networks

- **Notable Data Vendors**:
  - **Transaction Network Services (TNS)**:
    - Established connectivity with TASE (December 2022)
    - Delivers TASE market data globally via high-availability backbone
    - Minimal network latency
    - Low-latency network for TASE alongside European, US, and Asia Pacific exchanges
    - Market data for TASE equities
    - Enhanced number of market data feeds available to clients

- **Market Coverage**:
  - **Equities**: Israeli stocks listed on TASE
  - **ETFs**: Over 1,000 stocks and ETFs listed on TASE
  - **Bonds**: Government and corporate bonds
  - **Other Securities**: Various financial instruments traded on TASE

- **Data Services**:
  - Real-time quotes and trades
  - Historical price data
  - Market depth and order book data
  - Corporate actions and dividend information
  - Performance reporting
  - Tax reporting support

- **Use Cases**:
  - Real-time market data for Israeli securities
  - Historical data analysis and backtesting
  - Portfolio tracking and performance reporting
  - Dividend tracking
  - Tax reporting for Israeli securities
  - Multi-exchange market data (TASE alongside other global exchanges)

- **Relevance to This Project**:
  - **Israeli Market Data**: Access to real-time and historical data for Israeli securities
  - **Options Trading**: Market data for underlying Israeli stocks for options trading
  - **Multi-Currency**: ILS-denominated securities data
  - **Portfolio Management**: Tracking and analysis of Israeli securities positions
  - **Risk Management**: Market data for risk assessment of Israeli market exposure
  - **Arbitrage Opportunities**: Cross-market arbitrage between TASE and other exchanges

- **Integration**:
  - Access through authorized data vendors
  - Low-latency network connectivity
  - Integration with global market data platforms
  - Support for multiple exchange connectivity

- **Documentation**: Available through TASE website; comprehensive list of authorized data vendors and integration information
- **Note**: TASE is Israel's primary stock exchange. Market data is available through authorized data vendors rather than direct API access.
  TNS is a notable vendor providing low-latency global distribution of TASE market data.
Useful for Israeli traders requiring real-time and historical market data for Israeli securities, options trading on Israeli underlyings, and
portfolio management of ILS-denominated positions.

### TASE Data Hub API (direct)

- **Base URL**: <https://datahubapi.tase.co.il/>
- **API documentation**: <https://datahubapi.tase.co.il/docs/1626b30a-9369-4f6e-b0ec-b0340d8515bf/1748180086472>
- **API guide (PDF, English)**: <https://content.tase.co.il/media/l5xjhjmz/2000_api_guide_eng.pdf>
- **Data file distribution**: <https://www.tase.co.il/en/content/data/file_distribution> – TASE data files / bulk distribution (alternative or complement to API).
- **Market data (Hebrew)**: <https://market.tase.co.il/he/market_data> – TASE market site, market data section (Hebrew).
- **Derivatives major data (Hebrew)**: <https://market.tase.co.il/he/market_data/derivatives/major_data/details> – TASE derivatives “major data” details (index options, options on USD, etc.).
- **Derivatives EoD history (10 years)**: <https://datahubapi.tase.co.il/spec/f5196aac-357f-49e1-8984-2a93d0160758/c77e97c5-120d-4bf8-b54d-e6e8c6d359da#/APIs/getDerivativesEoDHistory10YearsData> – API spec for derivatives end-of-day historical data (10 years); use for index options, options on USD, and other TASE derivatives when building hedge suggestions or backtests.
- **Third-party (scraper)**: [algonell/tase](https://github.com/algonell/tase) – Tel Aviv Stock Exchange data scraper (Python, Jupyter notebook, TA35). Alternative to official API for scraping TASE data; use with appropriate licensing and respect for TASE terms of use.
- **Description**: TASE Data Hub API – official developer/data portal for programmatic access to TASE market data. Use for securities, derivatives (including index options and options on USD / currency options), and related reference data when building hedge suggestions (IB box spread + TASE options) or position aggregation.
- **Relevance**: Primary candidate for Phase 2 TASE option chain integration in `docs/planning/PRIMARY_CURRENCY_AND_TASE_HEDGING.md` (option chain for TA-35, TA-125, options on USD / ILS/USD). Use the docs link or PDF guide above for current endpoints, parameters, authentication, and rate limits. Use the file distribution page for bulk or file-based data if needed.
- **Note**: Portal may require registration or API key; verify terms of use and data scope (e.g. delayed vs real-time) before integration.

### IBKR (TWS API) – Israeli product data

- **What IBKR provides (confirmed in this project)**:
  - **FX / currency pairs**: USD/ILS and other pairs via TWS API with `secType="CASH"`, `exchange="IDEALPRO"`. Used for exchange rate in currency hedging (see `docs/research/analysis/CURRENCY_EXCHANGE_RISK.md`). Request with `Contract`: symbol `"USD"`, currency `"ILS"`, exchange `"IDEALPRO"`.
- **Israeli equities and TASE (to verify)**:
  - IBKR may offer **Israeli stocks** (TASE-listed) and possibly **TASE index/currency options**; product and data availability can depend on account region and market data subscriptions.
  - **How to check**: Use [IBKR Symbol and Exchange Search](https://www.interactivebrokers.com/en/trading/products-exchanges.php) (Products → Symbol and Exchange Search) and search for exchange **TASE** or **Tel Aviv**; or use TWS/API contract search (`reqContractDetails`, `reqMktData`) with a TASE contract (e.g. symbol for a TASE stock or index).
  - **Typical use**: If available, use TWS for USD/ILS FX and optionally for TASE securities/options data alongside TASE Data Hub API, so hedge suggestions can combine IB box spread (USD) with Israeli product data from one broker feed where possible.
- **References**: `docs/research/analysis/CURRENCY_EXCHANGE_RISK.md` (TWS CASH/IDEALPRO), `docs/planning/PRIMARY_CURRENCY_AND_TASE_HEDGING.md` (hedge suggestions IB + TASE).

### Options-IT - Trading Infrastructure & Connectivity

- **Official Website**: <https://www.options-it.com/>
- **Description**: Options Technology provides managed trading infrastructure and connectivity services for financial institutions.
  They offer solutions that support trading operations, including low-latency connectivity, cloud services, and managed hosting.
- **Key Products**:
  - **Atlas**: Trading infrastructure platform
  - **AtlasWorkplace**: Workplace solutions for trading operations
  - **AtlasApps**: Application services
  - **PrivateMind**: Private cloud solutions
  - **Managed Security**: Security services for trading infrastructure

- **Key Features**:
  - **Low-Latency Connectivity**: High-speed connectivity for trading operations
  - **Cloud Services**: Cloud-based trading infrastructure
  - **Managed Hosting**: Managed hosting services for trading systems
  - **Infrastructure Management**: Complete trading infrastructure management

- **Target Clients**:
  - Asset managers
  - Hedge funds
  - Sell-side firms
  - Software providers
  - Private equity firms

- **Use Cases**:
  - Trading infrastructure setup and management
  - Low-latency connectivity for algorithmic trading
  - Cloud-based trading system deployment
  - Managed hosting for trading applications
  - Security services for trading operations

- **Relevance to This Project**:
  - **Infrastructure**: Trading infrastructure and connectivity services
  - **Low-Latency**: High-speed connectivity for box spread execution
  - **Managed Services**: Managed hosting and infrastructure for trading systems
  - **Cloud Deployment**: Cloud-based deployment options

- **Documentation**: Available through Options-IT website
- **Note**: Options-IT provides infrastructure and connectivity services rather than market data APIs.
  Useful for setting up and managing trading infrastructure, low-latency connectivity, and cloud-based trading systems.

### Bloomberg - Financial Data & Analytics Platform

- **Official Website**: <https://www.bloomberg.com/>
- **Middle East Portal**: <https://www.bloomberg.com/middleeast>
- **Description**: Bloomberg is a global financial data and analytics platform providing real-time market data, news, and analytical tools.
  Bloomberg Terminal and B-PIPE provide comprehensive market data coverage including Middle Eastern markets.
- **Key Features**:
  - **B-PIPE**: Real-time market data feed delivering standardized, up-to-the-minute data
  - **Bloomberg Terminal**: Comprehensive trading and analytics platform
  - **Real-Time Data**: Instant market insights and data delivery
  - **Customizable Tools**: Customizable visual tools and reporting options
  - **Global Coverage**: Coverage of global markets including Middle East

- **Market Data Services**:
  - Real-time quotes and trades
  - Historical data
  - News and analytics
  - Portfolio management tools
  - Risk management analytics

- **Use Cases**:
  - Real-time market data for trading decisions
  - Portfolio management and analysis
  - Risk management and compliance
  - News and market analysis
  - Best execution compliance

- **Relevance to This Project**:
  - **Market Data**: Real-time and historical market data for box spread analysis
  - **Options Data**: Options market data and analytics
  - **Middle East Coverage**: Coverage of Israeli and Middle Eastern markets
  - **Analytics**: Advanced analytics for trading strategies

- **Documentation**: Available through Bloomberg Professional Services
- **Note**: Bloomberg provides comprehensive financial data and analytics through Bloomberg Terminal and B-PIPE.
  Premium service with extensive market coverage including Middle Eastern markets.
  Useful for institutional traders requiring comprehensive market data and analytics.

### BMLL Technologies - Historical Order Book Data & Analytics

- **Official Website**: <https://www.bmlltech.com/>
- **Description**: BMLL Technologies is an independent provider of harmonized, historical Level 3, 2, and 1 data and analytics across global equity,
  ETFs, and futures markets.
  They offer clients access to deep data to derive predictive insights, enabling researchers and quants to understand market behaviors comprehensively.
- **Key Features**:
  - **Historical Order Book Data**: Level 3, 2, and 1 historical data
  - **Harmonized Data**: Harmonized data across multiple markets
  - **Global Coverage**: Equity, ETFs, and futures markets
  - **Options Data**: Six years of nanosecond unconflated OPRA options data
  - **Analytics**: Advanced analytics and insights

- **Data Coverage**:
  - **Equity Markets**: Global equity markets
  - **ETFs**: Exchange-traded funds
  - **Futures**: Futures markets
  - **Options**: OPRA options data (6 years of nanosecond unconflated data)

- **Partnerships**:
  - **FactSet Integration**: Cloud-based granular historical tick data and analytics through FactSet platform
  - **INQDATA Partnership**: Access to BMLL data within kdb+ environment

- **Use Cases**:
  - Quantitative research and backtesting
  - Market impact analysis
  - Pre- and post-trade analytics
  - Order book simulation
  - Compliance and surveillance
  - Risk management
  - Transaction cost analysis
  - Best execution analysis

- **Relevance to This Project**:
  - **Historical Data**: Extensive historical order book data for backtesting
  - **Options Data**: Historical options data for box spread analysis
  - **Analytics**: Advanced analytics for strategy development
  - **Order Book Analysis**: Deep order book analysis for execution optimization
  - **Research**: Quantitative research capabilities

- **Documentation**: Available through BMLL Technologies website
- **Note**: BMLL Technologies specializes in historical order book data and analytics.
  Particularly useful for quantitative research, backtesting, and understanding market microstructure.
  The partnership with FactSet provides cloud-based access to granular historical data.

### FactSet - Financial Data & Analytics Platform

- **Official Website**: <https://www.factset.com/>
- **Description**: FactSet is a global provider of integrated financial information, analytical applications, and industry-leading services.
  They deliver financial data, analytics, and open technology in a digital platform to help users gain comprehensive insights.
- **Key Features**:
  - **Integrated Financial Information**: Comprehensive financial data platform
  - **Analytical Applications**: Advanced analytics and applications
  - **Open Technology**: Open technology platform
  - **Digital Platform**: Cloud-based digital platform
  - **Global Coverage**: Approximately 180,000 users worldwide

- **Services**:
  - Financial data and content
  - Flexible workflow solutions
  - Innovative technologies
  - Investment decision support

- **Partnerships**:
  - **BMLL Technologies**: Collaboration to provide cloud-based granular historical tick data and analytics
  - Enhanced access to Level 3 order book data through FactSet platform

- **Use Cases**:
  - Investment research and analysis
  - Portfolio management
  - Risk management
  - Compliance and surveillance
  - Quantitative research
  - Backtesting
  - Transaction cost analysis

- **Relevance to This Project**:
  - **Financial Data**: Comprehensive financial data for trading decisions
  - **Analytics**: Advanced analytics for strategy development
  - **Historical Data**: Access to historical data through BMLL partnership
  - **Research Tools**: Research and analysis tools for box spread strategies
  - **Portfolio Management**: Portfolio management and analysis

- **Documentation**: Available through FactSet website
- **Note**: FactSet provides comprehensive financial data and analytics platform.
  The collaboration with BMLL Technologies enhances access to granular historical order book data.
  Useful for institutional traders requiring comprehensive financial data, analytics, and research tools.

### Intercontinental Exchange (ICE) - Fixed Income Data Services

- **Developer Portal**: <https://developer.ice.com/fixed-income-data-services/catalog>
- **Official Website**: <https://www.ice.com/>
- **Description**:
  Intercontinental Exchange (ICE) provides comprehensive fixed income and data services designed to enhance market transparency and support informed decision-making.
  ICE offers continuous evaluated pricing, end-of-day evaluations, and fair value information services for fixed income instruments.
- **Key Services**:
  - **Continuous Evaluated Pricing (CEP)**: High-quality, continuous evaluated pricing for approximately 2.5 million fixed income instruments globally
  - **End-of-Day Evaluations**: End-of-day evaluation services
  - **Fair Value Information**: Fair value information services
  - **Price Transparency**: Enhanced price transparency for fixed income markets
  - **Performance Analysis**: Performance analysis tools

- **Data Delivery Mechanisms**:
  - **ICE Connect**: Connectivity platform
  - **ICE Consolidated Feed**: Consolidated data feed
  - **ICE Data API**: API access to data services

- **Fixed Income Solutions**:
  - **Price Transparency**: Enhanced price transparency
  - **Efficient Execution**: Execution services
  - **Performance Analysis**: Performance analysis tools
  - **Risk Management**: Real-time risk monitoring
  - **Price Discovery**: Price discovery services

- **Recent Innovations**:
  - **Price Improvement Volume Clearing (PIVC)**: Enhancement to Risk Matching Auction protocol
  - Deepens liquidity and improves pricing outcomes

- **Market Coverage**:
  - Approximately 2.5 million fixed income instruments globally
  - Multiple asset classes
  - Global market coverage

- **Use Cases**:
  - Fixed income price discovery
  - Real-time risk monitoring
  - Portfolio valuation
  - Risk management
  - Investment decisions
  - Compliance and reporting

- **Relevance to This Project**:
  - **Fixed Income Data**: Access to fixed income pricing data for hedging
  - **T-Bill Data**: Fixed income data including T-bills and government securities
  - **Risk Management**: Real-time risk monitoring for fixed income positions
  - **Valuation**: Portfolio valuation services
  - **Price Discovery**: Price discovery for fixed income instruments

- **Documentation**: Available through ICE Developer portal; catalog of fixed income data services
- **Note**: ICE provides comprehensive fixed income data services including continuous evaluated pricing for millions of fixed income instruments.
  Particularly useful for traders requiring fixed income pricing data, T-bill valuations, and fixed income risk management.

### ION Group - Trading Technology & Workflow Automation

- **Official Website**: <https://iongroup.com/>
- **Markets Portal**: <https://iongroup.com/markets/>
- **Fixed Income Solutions**: <https://iongroup.com/solutions/markets/fixed-income/>
- **Description**: ION Group specializes in workflow automation across the complete trade lifecycle,
  unifying operations from front to back and simplifying processes across asset classes.
  Their solutions automate the trade lifecycle, reduce operational risk, and deliver actionable insights in real-time.
- **Key Solutions**:
  - **Fixed Income Solutions**: Connectivity to multiple markets, client inquiry management, trading workflow automation
  - **Equities Trading**: Fidessa platform for equities trading
  - **Workflow Automation**: Complete trade lifecycle automation
  - **Risk Management**: Operational risk reduction
  - **Real-Time Insights**: Actionable insights in real-time

- **Key Features**:
  - **Front-to-Back Integration**: Unifies operations from front to back
  - **Multi-Asset Class**: Solutions across multiple asset classes
  - **Automation**: Trade lifecycle automation
  - **Risk Reduction**: Operational risk reduction
  - **Real-Time Analytics**: Real-time actionable insights
  - **Customer Understanding**: Better customer understanding
  - **Proactive Risk Management**: Proactive risk management capabilities

- **Fixed Income Capabilities**:
  - Connectivity to multiple fixed income markets
  - Client inquiry management
  - Trading workflow automation
  - Competitive edge in complex markets

- **Use Cases**:
  - Trade lifecycle automation
  - Multi-asset class trading
  - Fixed income trading
  - Risk management
  - Operational efficiency
  - Real-time analytics

- **Relevance to This Project**:
  - **Trading Workflows**: Workflow automation for box spread trading
  - **Fixed Income**: Fixed income trading solutions
  - **Risk Management**: Operational risk reduction
  - **Integration**: Front-to-back integration capabilities
  - **Automation**: Trade lifecycle automation

- **Documentation**: Available through ION Group website; information about trading solutions and workflow automation
- **Note**: ION Group provides trading technology and workflow automation solutions rather than direct market data APIs.
  Useful for automating trading workflows, managing fixed income operations, and reducing operational risk in trading systems.

### Pico Quantitative Trading - Market Data & Infrastructure Services

- **Official Website**: <https://www.pico.net/>
- **Description**: Pico is a leading global provider of technology services connecting the world's most liquid markets.
They offer access to over 900 exchange and venue products to power high-performance trading with real-time network monitoring and performance
analytics.
- **Key Services**:
  - **Analytics**: Corvil Analytics for real-time trading and network operations analytics
  - **Trading Applications**: InRush Ticker Plant, RedlineFeed, Order Execution Gateway, Pre-Trade Risk, Historical Market Data
  - **Market Data**: Raw, normalized, and historical market data, market data analytics
  - **Connectivity**: Network connectivity, venue access, network analytics, network services
  - **Infrastructure**: Colocation hosting, cloud services (public, private, hybrid), device management
  - **Expert Services**: Technology procurement, delivery management, service operations, security

- **Corvil Analytics**:
  - Real-time trading and network operations analytics
  - Machine learning and AI capabilities (Corvil Analytics 10.0)
  - Corvil Appliances, Decoders, Certification
  - 200Gbps continuous network capture and analytics
  - AI-ready network observability pipelines

- **Trading Applications**:
  - **InRush Ticker Plant**: Market data ticker plant
  - **RedlineFeed**: Ultra-low latency market data feed
  - **Order Execution Gateway**: Order execution services
  - **Pre-Trade Risk**: Pre-trade risk management
  - **Historical Market Data**: Historical data services
  - **Managed Service**: Managed trading services

- **Market Data Services**:
  - Raw market data
  - Normalized market data
  - Historical market data
  - Market data analytics

- **Connectivity**:
  - Network connectivity to global markets
  - Venue access to exchanges
  - Network analytics
  - Network services
  - IntelliVUE network monitoring

- **Infrastructure**:
  - **Colocation Hosting**: Data center colocation
  - **Cloud Services**: Public, private, and hybrid cloud
  - **Device Management**: Infrastructure device management
  - **Intellihands Services**: Remote hands services
  - **Infrastructure Analytics**: Infrastructure monitoring

- **Scale**:
  - 24/7 technical support
  - 55+ data centers globally
  - 900+ exchange and venue products

- **Target Industries**:
  - Global Banks
  - Regional Banks
  - Quantitative Hedge Funds
  - Exchanges
  - Fintech Service Providers
  - Electronic Market Makers
  - Macro Hedge Funds
  - Retail Brokers
  - Crypto

- **Use Cases**:
  - High-performance trading infrastructure
  - Ultra-low latency market data
  - Network monitoring and analytics
  - Order execution services
  - Pre-trade risk management
  - Historical data for backtesting
  - Colocation and cloud services

- **Relevance to This Project**:
  - **Market Data**: Ultra-low latency market data feeds
  - **Infrastructure**: Colocation and cloud infrastructure for trading
  - **Analytics**: Real-time trading and network analytics
  - **Execution**: Order execution gateway services
  - **Risk Management**: Pre-trade risk management
  - **Historical Data**: Historical market data for backtesting
  - **Connectivity**: Global connectivity to exchanges

- **Documentation**: Available through Pico website; comprehensive information about services and solutions
- **Note**: Pico provides comprehensive technology services for financial markets including market data, connectivity, infrastructure, and analytics.
Particularly useful for high-performance trading systems requiring ultra-low latency market data, colocation services, and comprehensive market
connectivity.

### QuantHouse - Systematic Trading Solutions

- **Official Website**: <https://www.quanthouse.com/>
- **Description**: QuantHouse is a provider of end-to-end systematic trading solutions, offering services such as market data,
  algorithmic trading development frameworks, and infrastructure solutions.
  Their offerings are designed to support trading firms in developing, testing, and deploying quantitative trading strategies efficiently.
- **Key Services**:
  - **Market Data**: Market data services for trading
  - **Algo Trading Development**: Algorithmic trading development frameworks
  - **Infrastructure Solutions**: Trading infrastructure solutions
  - **Systematic Trading**: End-to-end systematic trading solutions

- **Key Features**:
  - **Development Frameworks**: Frameworks for algo trading development
  - **Testing Capabilities**: Testing tools for trading strategies
  - **Deployment Solutions**: Deployment solutions for quantitative strategies
  - **Efficiency**: Efficient development and deployment of trading strategies

- **Use Cases**:
  - Quantitative trading strategy development
  - Algorithmic trading system development
  - Market data integration
  - Trading infrastructure setup
  - Strategy testing and deployment

- **Relevance to This Project**:
  - **Trading Infrastructure**: Infrastructure solutions for box spread trading
  - **Market Data**: Market data services for options and underlying securities
  - **Development Frameworks**: Frameworks for developing automated trading strategies
  - **Systematic Trading**: Support for systematic/quantitative trading approaches

- **Documentation**: Available through QuantHouse website
- **Note**: QuantHouse provides end-to-end systematic trading solutions including market data, development frameworks, and infrastructure.
  Useful for quantitative trading firms developing and deploying algorithmic trading strategies including box spread automation.

### London Stock Exchange Group (LSEG) - Data & Analytics

- **Official Website**: <https://www.lseg.com/en/data-analytics>
- **Description**: LSEG Data & Analytics is one of the world's largest providers of financial markets data and infrastructure.
  With over 40,000 customers and 400,000 end users across approximately 190 markets, LSEG is an essential partner to the global financial community.
- **Key Services**:
  - **Data & Feeds**: Best-in-class global market data and feeds
  - **Analytics**: AI-powered analytics platform
  - **Workflows**: Interoperable and collaborative workflows through LSEG Workspace
  - **Reuters News**: Exclusive provider of Reuters news to global financial marketplace

- **Key Features**:
  - **Global Coverage**: Approximately 190 markets worldwide
  - **AI-Powered Analytics**: Next-generation AI-driven products using Large Language Models
  - **Cloud-Collaborative**: Open, cloud-collaborative data distribution
  - **Market-Leading Technology**: Market-leading distribution and management technology
  - **Microsoft Partnership**: Strategic partnership with Microsoft for AI-ready data and cloud infrastructure

- **Data & Feeds Services**:
  - Global market data and feeds
  - Open, cloud-collaborative distribution
  - Market-leading distribution and management technology
  - Comprehensive data coverage

- **Analytics Solutions**:
  - AI-powered analytics platform
  - Rapid transformation of data into actionable insights
  - Custom analytics development
  - Large Language Model orchestration
  - Live financial data transformation

- **Workflow Solutions**:
  - **LSEG Workspace**: Open ecosystem for workflows
  - Interoperable and collaborative workflows
  - Integration of insights, news, and analytics
  - Customizable workflow solutions

- **Historical Analytics**:
  - **Historical Analytics via Snowflake**: Launched December 2024
  - Over 20 years of pricing data
  - More than 2.9 million bonds
  - Integration of LSEG Pricing Services with Yield Book Analytics
  - Use cases: Regulatory reporting, risk management, portfolio analysis

- **Fixed Income Solutions**:
  - **Yield Book Analytics**: Fixed income analytics tools
  - Bond pricing and analytics
  - Fixed income risk management
  - Portfolio analysis

- **News Services**:
  - **Reuters News**: Exclusive provider of Reuters news
  - 170-year history and reputation
  - Market-moving headlines
  - Insight and commentary
  - Coverage of all major market sectors

- **Customer Consulting**:
  - Global in-house team of consultants
  - Solution architects and project managers
  - Technical expertise delivery
  - Custom solution implementation

- **Recent Developments**:
  - **Q1 2025**: 8.7% year-on-year increase in total income
  - **Data & Analytics Growth**: 5.1% growth driven by Analytics and Data & Feeds units
  - **Cloud Solutions**: New cloud-based solutions developed with Microsoft partnership
  - **Historical Analytics**: Historical Analytics solution via Snowflake platform

- **Use Cases**:
  - Investment research and analysis
  - Risk management
  - Portfolio management
  - Regulatory reporting
  - Market data access
  - News and market analysis
  - Fixed income analytics

- **Relevance to This Project**:
  - **Market Data**: Comprehensive global market data for box spread analysis
  - **Fixed Income Data**: Yield Book Analytics for fixed income and T-bill data
  - **Historical Data**: Historical analytics for backtesting
  - **Analytics**: AI-powered analytics for strategy development
  - **News**: Reuters news for market-moving events
  - **Workflows**: Workflow solutions for trading operations

- **Documentation**: Available through LSEG website; comprehensive product documentation and support
- **Note**: LSEG Data & Analytics is one of the world's largest financial data providers with extensive global coverage.
The strategic partnership with Microsoft and AI-powered analytics capabilities make it particularly valuable for institutional traders requiring
comprehensive market data, fixed income analytics, and advanced analytics tools.

### SIX Group - Swiss Financial Infrastructure & Market Data

- **Official Website**: <https://www.six-group.com/en/home.html>
- **Developer Portal**: <https://www.six-group.com/en/specialized-offerings/six-developer-portal.html>
- **Description**: SIX Group operates the infrastructure for the Swiss and Spanish financial centers, providing services in securities,
  financial information, and payments.
  SIX operates the Swiss Stock Exchange and provides comprehensive market data, indices, and financial services.
- **Key Services**:
  - **SIX Swiss Exchange**: Swiss stock exchange operations
  - **BME Exchange**: Spanish stock exchanges (acquired 2020)
  - **Aquis Exchange**: Pan-European stock exchange (acquired July 2025)
  - **Securities Services**: Clearing, settlement, custody, securities finance
  - **Data Products**: Market data, indices, reference data, regulatory services
  - **Banking Services**: Billing, payments, open banking, interbank clearing

- **Market Data Services**:
  - **Real-Time Data**: Real-time market data feeds
  - **Historical Data**: Historical prices and market data
  - **Indices**: Swiss indices (SMI, SPI), Spanish indices, Nordic indices, crypto indices
  - **Reference Data**: Market and reference data services
  - **Regulatory Services**: Regulatory data and services
  - **ESG Data**: ESG data and services

- **SARON (Swiss Average Rate Overnight)**:
  - **SARON Calculator**: Available through SIX market data services
  - **Risk-Free Rate**: Swiss equivalent of risk-free rate for CHF-denominated options
  - **Overnight Rate**: Swiss overnight interest rate benchmark

- **Indices**:
  - **Swiss Indices**: SMI (Swiss Market Index), SPI (Swiss Performance Index)
  - **Spanish Indices**: BME exchange indices
  - **Nordic Indices**: Nordic market indices
  - **Crypto Indices**: Cryptocurrency indices
  - **Global Indices**: Global index coverage
  - **Customized Indices**: Custom index creation

- **Data Products**:
  - **Indices Data Center**: Index data and analytics
  - **Market & Reference Data**: Comprehensive market data
  - **Regulatory Services**: Regulatory reporting and data
  - **ESG Data & Services**: Environmental, social, and governance data
  - **Delivery Methods**: Multiple data delivery methods

- **Securities Services**:
  - **Clearing Services**: Trade clearing
  - **Settlement and Custody**: Securities settlement and custody
  - **Securities Finance**: Securities lending and financing
  - **Tax Services**: Tax-related services
  - **Trade Repository**: Trade reporting repository

- **Banking Services**:
  - **Open Banking**: Open banking services
  - **Interbank Clearing**: Interbank clearing services
  - **Payment Services**: Billing and payment services
  - **Data Analytics & AI**: Banking data analytics

- **Developer Portal**:
  - **SIX Developer Portal**: Developer resources and APIs
  - **API Access**: Programmatic access to SIX services
  - **Documentation**: Technical documentation and integration guides

- **Recent Acquisitions**:
  - **BME (2020)**: Acquisition of Spanish stock exchanges
  - **Aquis (July 2025)**: Acquisition of pan-European stock exchange

- **Use Cases**:
  - Swiss and Spanish market data access
  - Index data and analytics
  - Securities trading and settlement
  - Risk-free rate data (SARON) for CHF options
  - Regulatory reporting
  - ESG data and analysis

- **Relevance to This Project**:
  - **Swiss Market Data**: Access to Swiss stock exchange data
  - **SARON Data**: Swiss risk-free rate for CHF-denominated options pricing
  - **Index Data**: Swiss and European index data
  - **Multi-Exchange**: Access to Swiss, Spanish, and pan-European exchanges
  - **Regulatory Data**: Regulatory reporting and compliance data
  - **Developer APIs**: Developer portal for programmatic access

- **Documentation**: Available through SIX website and Developer Portal
- **Note**: SIX Group operates the infrastructure for Swiss and Spanish financial centers.
  Provides comprehensive market data, indices, and financial services.
  SARON (Swiss Average Rate Overnight) is the Swiss risk-free rate equivalent, available through SIX market data services.
  Useful for traders requiring Swiss market data, SARON for CHF options pricing, and access to Swiss and Spanish exchanges.

### Allocator - Alternative Investment Data Management Platform

- **Official Website**: <https://www.allocator.com/>
- **Description**:
  AI-driven data management platform for Limited Partners (LPs) to manage alternative investment data from hedge funds and private capital funds.
  Provides portfolio analytics, risk monitoring, performance attribution, and comprehensive fund data aggregation.
- **Key Services**:
  - **Data Aggregation**: Automated collection and organization of fund reports from GPs and hedge funds
  - **Risk Analytics**: Portfolio risk exposure analysis, VaR, CVaR, MVaR, sensitivity analysis
  - **Performance Analytics**: Returns analysis, performance attribution, benchmarking
  - **Portfolio Analytics**: Diversification analysis, correlation tracking, overlap detection
  - **API Access**: Programmatic access to aggregated fund data
  - **Data Harvesting**: Automated extraction of data from financial statements, quarterly reports, questionnaires, factsheets, risk reports

- **Coverage**:
  - **Hedge Funds**: Thousands of hedge funds regularly uploading data
  - **Private Capital Funds**: Private equity, venture capital, and other private capital funds
  - **Long-Only Funds**: Traditional long-only investment funds
  - **Fund Types**: Fund of Funds, Secondary Funds, Fund of Hedge Funds

- **Client Types**:
  - Private Capital Fund of Funds
  - Secondary Funds
  - Fund of Hedge Funds
  - Endowments
  - Foundations
  - Sovereign Wealth Funds
  - Insurance Companies
  - Pension Funds
  - Investment Consultants
  - Private Banks
  - Family Offices

- **Analytics Capabilities**:
  - **Performance Analysis**: Returns over different time periods, benchmarking against indices
  - **Risk Monitoring**: Exposure tracking by asset class, sector, strategy, currency, geography
  - **Sensitivity Analysis**: Price sensitivities (long, short, net, gross exposures), interest rate sensitivities (duration, DV01),
    credit spread sensitivities (CS01), volatility sensitivities (vega)
  - **Tail Risk**: VaR, CVaR, MVaR calculations
  - **Diversification**: Portfolio overlap and correlation analysis
  - **Value Creation**: Revenue, EBITDA, and KPI tracking at portfolio company level
  - **Alpha/Beta Decomposition**: Return decomposition for active managers

- **Data Sources**:
  - Financial statements
  - Quarterly reports
  - Questionnaires
  - Factsheets
  - Risk reports
  - Open Protocol templates
  - ILPA templates

- **API Features**:
  - **Data Warehouse Integration**: Direct feed to CRM, portfolio monitoring systems, dashboards
  - **Real-Time Updates**: Up-to-date, accurate, and comprehensive data
  - **Custom Scorecards**: Create custom investment criteria filters
  - **Peer Groups**: Benchmark funds and maintain peer groups

- **Geographic Presence**:
  - **United Kingdom**: London office
  - **Switzerland**: Zürich office
  - **South Africa**: Somerset West office

- **Use Cases**:
  - Portfolio risk monitoring and analysis
  - Performance attribution and benchmarking
  - Fund due diligence and selection
  - Diversification analysis
  - Risk exposure tracking
  - Value creation analysis
  - Peer group benchmarking

- **Relevance to This Project**:
  - **Portfolio-Level Analytics**: If expanding to portfolio management features, Allocator could provide hedge fund strategy data and benchmarking
  - **Risk Analytics**: Advanced risk metrics (VaR, CVaR, sensitivity analysis) could be useful for multi-strategy portfolios
  - **Hedge Fund Data**: Access to hedge fund strategies that may use box spreads or similar arbitrage strategies
  - **Performance Benchmarking**: Benchmark box spread strategy performance against hedge fund peers
  - **Multi-Asset Portfolio**: If expanding beyond box spreads to multi-asset portfolios
  - **Note**: Less directly relevant to core box spread trading functionality, but potentially useful for portfolio-level analysis and benchmarking

- **Documentation**: Available through Allocator website; API documentation available to clients
- **Note**: Allocator is primarily designed for Limited Partners managing alternative investment portfolios.
While not directly relevant to individual box spread trading, it could be useful for portfolio-level analytics, benchmarking against hedge fund
strategies, or if the project expands to include multi-strategy portfolio management features.

### CppTrader - High-Performance Trading Components

- **GitHub**: <https://github.com/chronoxor/CppTrader>
- **API Docs**: <https://github.com/chronoxor/CppTrader/tree/master/documents>
- **Version**: Latest (master branch)
- **License**: MIT License
- **Purpose**: Ultra-fast matching engine, order book processor, and NASDAQ ITCH handler
- **Key Features**:
  - **Ultra-fast matching engine**: Millions of operations per second
  - **Order book processor**: 9.7M+ messages/second throughput
  - **NASDAQ ITCH handler**: 41M+ messages/second throughput
  - **Market manager**: Handles orders and builds order books
  - **Optimized versions**: Standard, optimized (2.5x faster), aggressive (3x faster)

- **Performance Benchmarks**:
  - Market manager (standard): 3.2M messages/second
  - Market manager (optimized): 8.3M messages/second
  - Market manager (aggressive): 9.7M messages/second
  - ITCH handler: 41M+ messages/second

- **Integration Plan**: See `docs/CPPTRADER_INTEGRATION_PLAN.md` for comprehensive integration roadmap
- **Use Cases**:
  - Replace Python `MarketDataHandler` with C++ order book processor
  - High-performance order book management for box spread calculations
  - Market depth (level 2) data processing
  - Real-time order book reconstruction from TWS tick data

- **Dependencies**: None (standalone library)
- **Build Requirements**:
  - C++17 or higher
  - CMake 3.21+
  - Optional: `gil` tool for git submodule management (`pip3 install gil`)

- **Location**: `native/third_party/cpptrader/` (git submodule)
- **Integration Points**:
  - `native/include/order_book_manager.h`: CppTrader wrapper for order book management
  - `native/src/tws_client.cpp`: TWS tick data → CppTrader order book updates
  - `native/include/box_spread_calc.h`: Enhanced calculations using order book depth

- **Note**: Primary migration target for moving market data processing from Python to C++

### Market Gear (Options Trading Platform)

- **Official Website**: <https://www.marketgear.com/options/>
- **Description**: Web-based options trading platform with visual strategy templates and comprehensive options chain analysis
- **Key Features**:
  - **20+ Options Strategy Templates**: Iron Condors, Bull/Bear Spreads, Box Spreads, and more
  - **Options Chain Visualization**: Interactive options chain with strike prices and expirations
  - **Trade Execution**: Trade directly from chain or use standard order ticket
  - **Backtesting**: Historical backtesting with nearly two decades of data
  - **Real-Time Options Scanning**: Intraday scanning for options opportunities (Master Kit feature)
  - **Broker Integration**: Connects with TD Ameritrade, E\*Trade, Ally Invest, ChoiceTrade
  - **Virtual Trading**: Paper trading accounts for strategy testing

- **Platform Type**: Web-based platform (no public API available)
- **Pricing**:
  - 14-day free trial (full access except Live Benzinga News and Audio Squawk)
  - Master Kit includes real-time options scanning
  - Mobile apps available for iPhone, Android, and iPad

- **Relevance**:
  - **Strategy Visualization**: Useful for visualizing box spread and other options strategies before implementing in code
  - **Backtesting Reference**: Can backtest strategies manually to validate assumptions
  - **Options Chain Analysis**: Interactive options chain can help understand market structure
  - **Strategy Research**: 20+ templates provide reference for strategy implementation

- **Integration Considerations**:
  - **No API Available**: Market Gear does not provide a public API for programmatic access
  - **Manual Platform**: Primarily designed for manual trading and strategy visualization
  - **Reference Tool**: Best used as a reference/comparison tool for strategy design and backtesting
  - **Not Suitable for Automation**: Cannot integrate directly into automated trading systems

- **Use Cases**:
  - Manual strategy design and visualization
  - Backtesting strategy ideas before implementing in code
  - Understanding options market structure and pricing
  - Comparing strategy performance with manual backtesting
  - Reference for options strategy templates

- **Comparison with This Project**:
  - **Market Gear**: Manual/web-based platform with visual tools, no API
  - **This Project**: Automated algorithmic trading via APIs (TWS API, Alpaca, etc.)
  - **Best Use**: Market Gear for strategy research/visualization; This project for automated execution

- **Note**: Market Gear is a useful reference tool for options strategy research and visualization,
  but does not provide API access for integration into automated trading systems.
  Use it for manual strategy design and backtesting before implementing automated strategies in code.

### SpeedBot Enterprise (B2B Algo Trading Platform)

- **Official Website**: <https://speedbot.tech/speedbot-enterprise-for-brokers>
- **API Access Page**: <https://speedbot.tech/speedbot-enterprise-for-brokers#api-access>
- **Description**: B2B white-label Platform-as-a-Service (PaaS) for brokers, sub-brokers,
  and algo strategy creators to offer algorithmic trading services under their own brand
- **Platform Type**: Enterprise white-label solution with API access
- **Target Market**: Brokers, sub-brokers, and professional algo trading strategy creators
- **Geographic Focus**: India and USA brokers
- **Key Features**:
  - **API Access**: API access for algo trading services (details not publicly documented)
  - **Custom Branding**: White-label solution with custom branding for brokers
  - **Multiple Broker Integration**: Supports integration with various brokers in India and USA
  - **Options Strategy Automation**: Automated options trading strategy execution
  - **Strategy Creation**: No-code strategy builder with templates
  - **Backtesting**: Historical backtesting with accurate data
  - **Risk Management**: Advanced risk management tools with AI/ML
  - **Admin Panel**: Enterprise admin panel for monitoring and control
  - **Performance Tracking**: Real-time performance tracking and reporting
  - **Scalability**: Cloud-based, scalable infrastructure for multiple strategies and contracts
  - **Execution Error Handling**: Built-in execution error and retry mechanism (1-2% error rate reported)

- **Strategy Support**:
  - Options strategy templates
  - Custom strategy creation
  - Strategy backtesting
  - Strategy monetization (brokers can sell strategies to clients)

- **Broker Integration**:
  - Multiple broker support (India & USA)
  - Seamless broker API integration
  - Unified cloud-based platform

- **Enterprise Features**:
  - Custom branding for brokers
  - Admin panel for position monitoring
  - Client retention tools
  - Real-time performance tracking
  - Detailed reporting
  - Enterprise support (dedicated support team)

- **API Access**:
  - **Status**: API access mentioned but documentation not publicly available
  - **Use Case**: Enables brokers to integrate SpeedBot services into their systems
  - **Access**: Likely requires enterprise agreement/partnership

- **Pricing**:
  - B2B pricing (custom pricing, contact for details)
  - Contact: <sales@speedbot.tech> or +91 8488949163

- **Relevance**:
  - **Platform Architecture Reference**: Useful reference for B2B algo trading platform architecture
  - **White-Label Model**: Example of white-label PaaS model for trading services
  - **Feature Comparison**: Comparison point for features (backtesting, risk management, admin panels)
  - **Enterprise Patterns**: Reference for enterprise-level trading platform patterns

- **Integration Considerations**:
  - **B2B Focus**: Primarily designed for brokers offering services to their clients
  - **API Access**: API exists but requires enterprise agreement (documentation not public)
  - **White-Label Model**: Not a direct integration target for individual traders
  - **Competitive/Complementary**: Could be a competitor or complementary service depending on use case

- **Comparison with This Project**:
  - **SpeedBot Enterprise**: B2B white-label PaaS for brokers, API access (not public), enterprise support
  - **This Project**: Self-hosted, open-source algorithmic trading system, direct IBKR TWS API integration
  - **SpeedBot**: Platform-as-a-Service model for brokers to offer to clients
  - **This Project**: Direct integration for individual/algorithmic trading
  - **Best Use**: SpeedBot for brokers wanting to offer algo trading services; This project for direct IBKR integration

- **Use Cases**:
  - Brokers offering algo trading services to clients
  - Sub-brokers monetizing trading strategies
  - Algo strategy creators selling strategies
  - Enterprise white-label trading platform deployment

- **Potential Integration Opportunities**:
  - **Reference Architecture**: Study SpeedBot's architecture for B2B platform patterns
  - **Feature Ideas**: Risk management tools, admin panels, reporting features
  - **White-Label Consideration**: Could inform potential white-label offering of this project

- **Limitations**:
  - API documentation not publicly available (requires enterprise agreement)
  - Primarily B2B focus (not for individual traders)
  - Geographic focus on India and USA brokers
  - Enterprise pricing model (may not be suitable for individual use)

- **Note**:
  SpeedBot Enterprise is a B2B white-label algo trading platform designed for brokers to offer algorithmic trading services to their clients.
  While it provides API access, the documentation is not publicly available and requires an enterprise agreement.
This project is a self-hosted, open-source alternative that provides direct IBKR TWS API integration for individual traders and algorithmic trading
systems.
  SpeedBot could serve as a reference for B2B platform architecture and enterprise features, but is not a direct integration target for this project.

### ECN Execution (Algorithmic Trading Information Resource)

- **Official Website**: <https://ecnexecution.com/algorithmic-trading/>
- **Description**: Informational and educational resource about algorithmic trading, ECN brokers, and trading platforms
- **Platform Type**: Educational/informational website (not an API or service provider)
- **Purpose**: Provides information, reviews, and educational content about algorithmic trading brokers and platforms
- **Key Content Areas**:
  - **Algorithmic Trading**: Information about algorithmic trading strategies, high-frequency trading, VWAP, TWAP
  - **ECN Brokers**: Reviews and information about Electronic Communication Network (ECN) brokers
  - **Broker Reviews**: Reviews of various brokers (FP Markets, IC Markets, BlackBull Markets, Eightcap, etc.)
  - **Trading Platforms**: Information about MT4, MT5, TradingView, cTrader platforms
  - **Trading Accounts**: Information about ECN accounts, CFD accounts, Islamic accounts, swap-free accounts
  - **Trading Strategies**: Educational content about trading strategies and algorithmic trading
  - **Broker Comparison**: Broker comparison tools and recommendations

- **Key Topics Covered**:
  - Algorithmic trading concepts and strategies
  - High-frequency trading (microsecond/nanosecond execution)
  - ECN vs STP vs Market Maker brokers
  - Direct Market Access (DMA) brokers
  - VWAP (Volume-Weighted Average Price) trading
  - TWAP (Time-Weighted Average Price) trading
  - Low-latency trading
  - Broker API capabilities
  - Trading platform features

- **Brokers Covered**:
  - FP Markets, IC Markets, BlackBull Markets, Eightcap, Exness, InstaForex, Axi, NordFX
  - Various ECN brokers and their features
  - Broker comparisons and recommendations

- **Trading Platforms Covered**:
  - MetaTrader 4 (MT4) ECN brokers
  - MetaTrader 5 (MT5) ECN brokers
  - TradingView ECN brokers
  - cTrader ECN brokers

- **Educational Content**:
  - What is algorithmic trading?
  - How algorithmic trading works
  - Algorithmic trading strategies
  - Broker selection criteria
  - Trading platform comparison
  - Risk management in algorithmic trading

- **API Access**:
  - **Status**: No API provided (informational resource only)
  - **Purpose**: Educational and informational content about brokers and platforms
  - **Not a Service Provider**: Does not offer trading services or API access

- **Relevance**:
  - **Educational Resource**: Useful for understanding algorithmic trading concepts and broker options
  - **Broker Research**: Information about brokers that support algorithmic trading
  - **Platform Comparison**: Comparison of trading platforms (MT4, MT5, TradingView, cTrader)
  - **Strategy Reference**: Educational content about trading strategies (VWAP, TWAP, etc.)
  - **Broker Selection**: Broker reviews and comparisons for selecting algo-friendly brokers

- **Integration Considerations**:
  - **Not an API**: ECN Execution does not provide an API or trading services
  - **Informational Only**: Serves as an educational/informational resource
  - **Broker Research**: Useful for researching brokers that support algorithmic trading
  - **Educational Content**: Reference for understanding algorithmic trading concepts

- **Use Cases**:
  - Researching brokers that support algorithmic trading
  - Understanding algorithmic trading strategies and concepts
  - Comparing trading platforms (MT4, MT5, TradingView, cTrader)
  - Learning about ECN brokers and their features
  - Broker selection for algorithmic trading
  - Educational reference for trading strategies

- **Comparison with This Project**:
  - **ECN Execution**: Informational resource about algorithmic trading and brokers
  - **This Project**: Direct implementation of algorithmic trading system with IBKR TWS API
  - **Best Use**: ECN Execution for research and education; This project for actual trading implementation

- **Note**: ECN Execution is an educational and informational resource about algorithmic trading, ECN brokers, and trading platforms.
It does not provide an API or trading services - it serves as a reference for understanding algorithmic trading concepts, researching brokers, and
comparing trading platforms.
While useful for broker research and educational purposes, this project provides direct algorithmic trading implementation via IBKR TWS API
integration.

## Open Data APIs & Resources

<!--
@index: api-documentation
@category: open-data
@tags: open-data, public-apis, government-data, free
@last-updated: 2025-01-27
-->

### Public APIs Repository (Curated List)

- **GitHub**: <https://github.com/public-apis/public-apis>
- **Open Data Section**: <https://github.com/public-apis/public-apis?tab=readme-ov-file#open-data>
- **Description**: Community-curated list of free public APIs organized by category
- **Total APIs**: 1,000+ APIs across 50+ categories
- **Use Case**: Reference when looking for free, public APIs to complement trading data
- **Key Categories for Trading Applications**:
  - **Open Data**: Government and public datasets
  - **Finance**: Financial data and market information
  - **Cryptocurrency**: Crypto market data
  - **Business**: Company and business data
  - **Weather**: Economic indicator correlations

### FRED (Federal Reserve Economic Data) – St. Louis Fed

- **FRED** = Federal Reserve Economic Data (the service name).
- **API reference**: <https://fred.stlouisfed.org/docs/api/fred/>
- **API key (free)**: <https://fred.stlouisfed.org/docs/api/api_key.html>
- **Description**: Time series for SOFR, Treasury rates, and other economic indicators.
- **This project**: `python/integration/sofr_treasury_client.py` uses the FRED API for SOFR overnight/term and Treasury rates; credentials via `FRED_API_KEY`, `OP_FRED_API_KEY_SECRET`, or 1Password vault item titled "FRED API".
- **Use case**: Benchmark risk-free rates for box spread yield comparison and option pricing.

### Treasury Fiscal Data API (U.S. Treasury)

- **API documentation**: <https://fiscaldata.treasury.gov/api-documentation/>
- **Base URL**: <https://api.fiscaldata.treasury.gov/services/api/fiscal_service>
- **Description**: U.S. Treasury fiscal and interest-rate data (average interest rates on Treasury securities, no API key required).
- **This project**: `python/integration/treasury_api_client.py` uses the Fiscal Data API for Treasury benchmarks; `risk_free_rate_extractor` and `yield_curve_comparison` can use it alongside or instead of FRED for Treasury rates.
- **Use case**: Risk-free rate benchmarks (Treasury bills/notes) for comparison with box spread implied rates.

### Notable Open Data APIs (from public-apis repository)

#### Data.gov

- **URL**: <https://www.data.gov/>
- **Description**: United States government open data portal
- **Use Case**: Economic indicators, employment data, inflation metrics
- **Auth**: No (public data)
- **HTTPS**: Yes
- **Relevance**: Useful for macroeconomic analysis and correlation with market movements

#### Quandl / Nasdaq Data Link

- **URL**: <https://www.quandl.com/> (legacy) | <https://data.nasdaq.com/> (current)
- **Official API Docs**: <https://docs.data.nasdaq.com/>
- **Python Library**: <https://github.com/Nasdaq/data-link-python>
- **Description**: Comprehensive financial and economic data platform (acquired by Nasdaq in 2018)
- **Acquisition**: Nasdaq acquired Quandl in December 2018; now integrated into Nasdaq Data Link
- **Use Case**:
  - Historical market data (stocks, options, futures)
  - Economic indicators (GDP, inflation, employment)
  - Alternative data sources (satellite, sentiment, web traffic)
  - Cross-validation of TWS API data

- **Data Coverage**:
  - Over 20 million datasets
  - 350+ data providers
  - Historical data going back decades
  - Real-time data available (requires subscription)

- **Auth**: apiKey required (free tier available)
- **HTTPS**: Yes
- **API Limits**:
  - Free tier: Limited daily calls
  - Premium: Higher limits, access to premium datasets

- **Integration**:
  - Python: `nasdaq-data-link` package (formerly `quandl`)
  - R, Excel, MATLAB, Ruby support
  - REST API for direct integration

- **Relevance**:
  - Complements TWS API with historical data and economic indicators
  - Useful for backtesting strategies
  - Alternative data for market analysis
  - Cross-validation of real-time TWS data with historical trends

- **Pricing**:
  - Free tier: Limited to free public datasets
  - Premium: Subscription-based access to premium datasets
  - Commercial use may require licensing agreements

- **Note**: The original `quandl-python` library has been archived. Use `nasdaq-data-link` package for current integration.

### Unmeshed - API Orchestration Platform

- **Website**: <https://unmeshed.io/>
- **Blog Post**: <https://unmeshed.io/blog/api-orchestration-efficently-aggregate-api-data-using-orchestration>
- **Provider**: Unmeshed, Inc.
- **Description**: Developer-centric API orchestration platform that enables visual workflow building for aggregating data from multiple APIs,
  executing API calls in parallel, and managing complex API workflows without writing glue code
- **Key Features**:
  - **Visual Workflow Builder**: Drag-and-drop interface for building API orchestration workflows
  - **Parallel Execution**: Execute multiple API calls in parallel and use the first response received
  - **Sequential to Parallel Optimization**: Easily switch from sequential to parallel execution without code changes
  - **JavaScript Tasks**: Built-in JavaScript task support for shaping response JSON and data transformation
  - **Secret Management**: Built-in secrets management without hardcoding API keys
  - **Visual Timeline**: Visual timeline for debugging and tracing performance
  - **No Glue Code**: Eliminates need for repetitive glue code, retries, and edge case handling
  - **No Redeploys**: Modify workflows without redeploying code
  - **SDKs**: SDKs for integrating services instantly

- **Use Cases**:
  - **API Data Aggregation**: Aggregate data from multiple market data providers (e.g., TWS API, ORATS, dxFeed)
  - **Parallel API Calls**: Execute multiple API calls in parallel for faster data retrieval
  - **First Response Wins**: Use first available response when speed is critical
  - **Multi-Source Market Data**: Combine data from TWS API, ORATS, dxFeed, and other sources
  - **Workflow Orchestration**: Orchestrate complex workflows involving multiple API calls
  - **Data Transformation**: Transform and merge responses from multiple APIs into unified format

- **Relevance to Box Spread Trading**:
  - **Multi-Source Market Data**: Aggregate options data from TWS API, ORATS, dxFeed, and other sources
  - **Parallel Data Fetching**: Fetch market data from multiple sources in parallel for faster box spread detection
  - **Data Validation**: Compare data from multiple sources to validate pricing and identify arbitrage opportunities
  - **Workflow Automation**: Orchestrate complex workflows for box spread scanning, validation, and execution
  - **Performance Optimization**: Reduce latency by executing API calls in parallel instead of sequentially
  - **Data Aggregation**: Combine options chain data, Greeks, IV, and market data from multiple providers

- **Integration Considerations**:
  - **Visual Workflow Builder**: No code required for basic workflows (drag-and-drop interface)
  - **JavaScript Tasks**: Use JavaScript for custom data transformation and merging
  - **Secret Management**: Secure API key management without hardcoding
  - **Parallel Execution**: Optimize workflows by running API calls in parallel
  - **SDK Integration**: Integrate Unmeshed workflows into existing applications via SDKs
  - **Azure Marketplace**: Available in Microsoft Azure Marketplace
  - **No Backend Code Changes**: Modify workflows without changing backend code

- **Example Use Case for Box Spread Trading**:
  - **Parallel Market Data Fetching**: Fetch options chain data from TWS API, Greeks from ORATS, and IV from dxFeed in parallel
  - **Data Aggregation**: Merge responses into unified format for box spread analysis
  - **First Response Strategy**: Use first available data source when speed is critical for arbitrage detection
  - **Multi-Source Validation**: Compare pricing from multiple sources to identify mispricing opportunities

- **Comparison with Current Solutions**:
  - **vs. Custom Glue Code**: Unmeshed eliminates need for custom API aggregation code
  - **vs. Sequential API Calls**: Parallel execution reduces total latency
  - **vs. Manual Integration**: Visual builder simplifies workflow creation and modification

- **Benefits**:
  - **Faster Development**: Build API aggregation workflows faster without writing glue code
  - **Better Performance**: Parallel execution reduces total latency
  - **Easier Maintenance**: Visual workflows easier to understand and modify
  - **No Redeploys**: Modify workflows without redeploying code
  - **Built-in Secrets**: Secure API key management
  - **Visual Debugging**: Visual timeline for debugging and performance analysis

- **Contact**: Sign up at <https://unmeshed.io/> or find in Microsoft Azure Marketplace
- **Note**: Unmeshed is an API orchestration platform that simplifies aggregating data from multiple APIs.
Particularly useful for box spread trading when combining data from multiple market data providers (TWS API, ORATS, dxFeed) in parallel for faster
data retrieval and validation.
The visual workflow builder eliminates need for custom glue code, and parallel execution can significantly reduce latency when fetching data from
multiple sources.
  Evaluate as tool for orchestrating multi-source market data aggregation workflows.

#### Alpha Vantage (Market Data)

- **URL**: <https://www.alphavantage.co/>
- **Official API Docs**: <https://www.alphavantage.co/documentation/>
- **MCP Server**: <https://www.alphavantage.co/> (MCP + AI Agents support)
- **Description**: Enterprise-grade stock market data API provider, backed by Y Combinator and officially licensed by NASDAQ
- **Key Features**:
  - Real-time and historical stock market data
  - Options, forex, cryptocurrency data
  - 60+ technical indicators
  - Economic indicators
  - Market news API with sentiment analysis
  - MCP (Model Context Protocol) server support for AI agents
  - Spreadsheet integration

- **Data Coverage**:
  - Traditional asset classes (stocks, ETFs, mutual funds)
  - Foreign exchange rates
  - Commodities
  - Fundamental data
  - Technical indicators
  - Global market data

- **Auth**: apiKey required (free tier available)
- **This project**: `python/integration/alpha_vantage_client.py` uses the Alpha Vantage API for quotes, daily series, SMA, and symbol search; credentials via `ALPHA_VANTAGE_API_KEY`, `OP_ALPHA_VANTAGE_API_KEY_SECRET`, or 1Password item titled "Alpha Vantage API".
- **HTTPS**: Yes
- **API Limits**:
  - Free tier: 5 API calls per minute, 500 calls per day
  - Paid plans: Starting at $49.99/month with higher limits

- **Integration**:
  - REST API for direct integration
  - MCP server for AI agent integration
  - Spreadsheet add-ons
  - Python, JavaScript, and other language support

- **Relevance**:
  - Complements TWS API with additional market data sources
  - Useful for technical analysis with 60+ indicators
  - News sentiment analysis for market research
  - Cross-validation of TWS data
  - MCP support enables AI agent integration

- **Pricing**:
  - Free tier: Limited to 5 calls/minute, 500 calls/day
  - Premium: Subscription-based with higher limits
  - Enterprise: Custom pricing for high-volume usage

- **Partnerships**: Officially licensed by NASDAQ as a US market data provider

#### Finnhub (Market Data)

- **URL**: <https://finnhub.io/>
- **Official API Docs**: <https://finnhub.io/docs/api>
- **API reference (quick)**:
  - Base: `https://finnhub.io/api/v1`
  - Auth: query param `token=<API_KEY>` (e.g. `?symbol=AAPL&token=xxx`)
  - Common: `/quote`, `/stock/candle`, `/stock/profile2`, `/company/news`, `/stock/recommendation`
  - WebSocket: `wss://ws.finnhub.io?token=<API_KEY>`
- **Description**: Comprehensive financial data API with real-time stock prices, fundamental data, news sentiment, and alternative data
- **Key Features**:
  - Real-time stock prices and quotes
  - Historical market data
  - Financial statements (income, balance sheet, cash flow)
  - Company fundamentals and profiles
  - News sentiment analysis (AI-powered)
  - Forex and cryptocurrency data
  - Alternative data sources
  - WebSocket support for real-time data

- **Data Coverage**:
  - Global stock markets
  - Options data
  - Forex pairs
  - Cryptocurrency markets
  - Economic indicators
  - Company fundamentals
  - News and sentiment

- **Auth**: apiKey required (free tier available)
- **HTTPS**: Yes
- **API Limits**:
  - Free tier: 60 API calls per minute (generous free tier)
  - Paid plans: Higher rate limits and advanced features

- **Integration**:
  - REST API
  - WebSocket API for real-time data
  - SDKs available for multiple languages (Python, JavaScript, Go, etc.)
  - Comprehensive documentation

- **Relevance**:
  - More generous free tier than Alpha Vantage (60 calls/min vs 5 calls/min)
  - Strong fundamental data for research
  - AI-powered sentiment analysis
  - Real-time WebSocket support
  - Useful for backtesting and research
  - Cross-validation of TWS API data

- **Pricing**:
  - Free tier: 60 calls/minute (suitable for development/testing)
  - Premium: Subscription-based with higher limits and advanced features
  - Enterprise: Custom pricing

- **Comparison with Alpha Vantage**:
  - **Finnhub**: Better for fundamental analysis, news sentiment, and real-time data (WebSocket)
  - **Alpha Vantage**: Better for technical indicators (60+ indicators) and economic data
  - **Finnhub**: More generous free tier (60 vs 5 calls/min)
  - **Alpha Vantage**: MCP server support for AI agents

**Note**: This repository serves as a reference guide. Evaluate each API for:

- Rate limits and usage terms
- Data accuracy and update frequency
- Cost (free tier vs. paid)
- Authentication requirements
- Terms of service compliance for trading applications

## Trading Frameworks & Infrastructure

<!--
@index: api-documentation
@category: trading-frameworks
@tags: trading-framework, c++, low-latency, infrastructure
@last-updated: 2025-01-27
-->

### FLOX (Modular Trading Framework)

- **GitHub**: <https://github.com/FLOX-Foundation/flox>
- **Documentation**: <https://flox-foundation.github.io/flox/>
- **License**: MIT License
- **Language**: Modern C++ (C++20)
- **Description**: Modular framework for building trading systems, providing low-level infrastructure for execution pipelines,
  market data processing, strategy logic, and exchange integration
- **Key Features**:
  - Composable, testable architecture
  - Execution pipeline infrastructure
  - Market data processing
  - Strategy logic management
  - Exchange integration support
  - Storage backend integration
  - Suitable for both research and production environments

- **Architecture**:
  - Modular design for flexibility
  - Low-level infrastructure components
  - Separation of concerns (execution, data, strategy, storage)
  - Testable components

- **Connectors**:
  - Community implementations: <https://github.com/FLOX-Foundation/flox-connectors>
  - Open-source connector implementations built on top of FLOX

- **Use Cases**:
  - Building custom trading systems
  - Research and backtesting
  - Production trading infrastructure
  - Exchange connectivity
  - Market data processing pipelines

- **Relevance**:
  - Alternative architecture reference for trading system design
  - Useful for understanding modular trading system patterns
  - C++20 framework aligns with project's C++ focus
  - MIT license allows for study and potential integration
  - Connectors repository provides implementation examples

- **Integration Considerations**:
  - Currently not integrated into this project
  - Could serve as reference architecture for future modularization
  - Connectors repository may provide useful patterns
  - MIT license allows for code study and adaptation

- **Documentation**:
  - Comprehensive documentation at flox-foundation.github.io/flox
  - Getting Started guide available
  - Contribution guidelines included

- **Code Style**:
  - Enforced via `clang-format`
  - Pre-commit hooks available
  - Follows existing structure and naming conventions

- **Disclaimer**: FLOX is provided for educational and research purposes.
  All strategies, connectors, and logic in test and demo code are demonstrative only and not intended for production use.

### SmartQuant C++ Ultra-Low Latency Framework

- **Official Website**: <https://www.smartquant.com/cpp.html>
- **Description**: Cross-platform ultra-low latency algorithmic trading framework designed for high-frequency trading applications
- **Key Features**:
  - Ultra-low latency: 0.2 microseconds (200 nanoseconds) per event
  - High throughput: 5 million events/second (single-core), 35 million events/second (multicore)
  - Cross-platform: Windows, Linux, macOS support
  - RTOS support: Can compile under Real-Time OS for guaranteed low interrupt latency
  - Multithreaded: Non-locking event queues, atomic operations, object pools
  - Memory management: Custom memory pools, ring buffers, garbage collection
  - Framework foundation: Built on Qt framework, native C++ with aggressive optimizations
  - Scenario mechanism: Inherits best practices from SmartQuant C# framework (10+ years)

- **Performance Metrics**:
  - Event processing latency: 0.2 microseconds
  - Single-core throughput: 5M events/second (i7 CPU)
  - Multi-core throughput: 35M events/second (i7 CPU, 4 physical/8 logical cores)

- **Architecture**:
  - Ring buffers for high-speed event queuing
  - Non-locking event queues for reduced contention
  - Object pools for efficient memory allocation
  - Custom memory management and garbage collection
  - Parallel multicore optimization
  - Cloud/cluster optimization support

- **Use Cases**:
  - High-frequency trading systems
  - Ultra-low latency order execution
  - Real-time market data processing
  - Parallel strategy execution
  - Production-grade algorithmic trading

- **Relevance**:
  - Potential integration for high-performance event processing
  - Could replace mutex-based event queues with ring buffers
  - Object pools could optimize memory allocation in hot paths
  - Multicore optimization could parallelize strategy scanning
  - Built-in backtesting capabilities

- **Integration Considerations**:
  - Currently not integrated into this project
  - Hybrid integration approach recommended (see research document)
  - Licensing verification required before integration
  - Significant refactoring would be needed for full migration

- **Research Documentation**: See `docs/SMARTQUANT_CPP_FRAMEWORK_RESEARCH.md` for comprehensive research, integration strategies,
  comparison with current architecture, and implementation considerations
- **Status**: Research complete, integration not yet planned
- **Note**: Framework designed for institutional-grade HFT systems. Evaluate licensing, cost, and integration complexity before proceeding.

### FIX Protocol & FIX Trading Community

<!--
@index: api-documentation
@category: fix-protocol
@tags: fix-protocol, fix-api, trading-protocol, standards
@last-updated: 2025-01-27
-->

- **Official Website**: <https://www.fixtrading.org/>
- **FIXimate (Interactive FIX Reference)**: <https://fiximate.fixtrading.org/>
- **GitHub Organization**: <https://github.com/FIXTradingCommunity>
- **License**: Standards organization (non-profit)
- **Description**:
  The FIX (Financial Information eXchange) Protocol is the industry-standard messaging protocol for electronic trading across global financial markets.
  The FIX Trading Community is the non-profit standards organization that maintains and develops FIX specifications.
- **Key Components**:
  - **FIX Protocol**: Messaging standard for the entire trade lifecycle (pre-trade, trade, post-trade)
  - **FIX Orchestra**: Machine-readable rules of engagement framework for automating trading workflows
  - **Simple Binary Encoding (SBE)**: High-performance binary message encoding standard
  - **FIX Performance Session Layer (FIXP)**: Performance-optimized session layer specification
  - **FIXimate**: Interactive web-based reference tool for FIX messages, components, fields, and datatypes

- **FIXimate Features**:
  - **Comprehensive Navigation**: Three-panel interface for browsing FIX specifications
  - **Advanced Search**: String/regex search for messages, components, fields, and code sets
  - **Detailed Displays**: Field descriptions, required statuses, usage contexts
  - **Direct URLs**: Stable URLs for bookmarking specific messages, components, or fields
  - **Autocompletion**: Name-based search with autocomplete

- **FIX Orchestra**:
  - Machine-readable XML format for expressing trading rules
  - Enables automated validation and code generation
  - Successor to the FIX Repository
  - GitHub: <https://github.com/FIXTradingCommunity/fix-orchestra>

- **Simple Binary Encoding (SBE)**:
  - Ultra-low latency binary encoding for FIX messages
  - Optimized for high-frequency trading
  - GitHub: <https://github.com/FIXTradingCommunity/fix-simple-binary-encoding>

- **Key Use Cases**:
  - Direct exchange connectivity (many exchanges require FIX)
  - Multi-broker integration beyond IBKR
  - High-performance order execution
  - Market data feeds from exchanges
  - Trade reporting and compliance

- **Relevance**:
  - **Direct Exchange Integration**: Many exchanges (CME, Cboe, etc.) offer FIX connectivity for direct market access
  - **Alternative to TWS API**: FIX allows direct exchange connectivity without broker middleware
  - **Performance**: SBE provides ultra-low latency encoding for high-frequency strategies
  - **Standardization**: Industry-standard protocol enables easier integration with multiple venues
  - **Regulatory Compliance**: FIX supports trade reporting and regulatory workflows

- **Integration Considerations**:
  - **Exchange Requirements**: Many exchanges require FIX for direct market access
  - **Performance Benefits**: SBE provides faster message encoding/decoding than text-based FIX
  - **Complexity**: FIX protocol is more complex than REST APIs but offers more control
  - **Learning Curve**: Requires understanding FIX message structure and session management
  - **Current Status**: Not integrated; using TWS API for IBKR connectivity

- **Key Resources**:
  - **FIXimate User Guide**: <https://fiximate.fixtrading.org/userguide.html>
  - **FIXimate Release Notes**: <https://fiximate.fixtrading.org/releasenotes.html>
  - **FIX Orchestra Specification**: <https://github.com/FIXTradingCommunity/fix-orchestra-spec>
  - **FIX Trading Community GitHub**: <https://github.com/FIXTradingCommunity>
  - **FIX Online Specification**:
    <https://www.fixtrading.org/online-specification/introduction/> - Official FIX protocol specification and documentation

- **Potential Integration Opportunities**:
  - Direct CME Globex connectivity for futures trading
  - Direct Cboe connectivity for options trading (SPX, SPXW)
  - Market data feeds via FIX (quotes, trades, order book)
  - High-frequency execution using SBE encoding
  - Multi-venue arbitrage strategies with low latency

- **Example Use Case**: Direct CME/Cboe FIX connectivity could enable faster execution for box spread strategies by bypassing IBKR's TWS API,
  potentially reducing latency and improving fill rates.
- **Note**: FIX protocol is the industry standard for institutional trading.
  While more complex than REST APIs, it offers lower latency, better control, and direct exchange access.
  FIXimate provides an excellent reference for understanding FIX message structures and field definitions.

### FIX Protocol Development Tools & Libraries

#### QuickFIX Engine

- **Website**: <https://quickfixengine.org/>
- **Description**: Open-source FIX protocol engine implementation supporting multiple programming languages
- **Key Features**:
  - **Multi-Language Support**: C++, Java, Python, Ruby, .NET implementations
  - **FIX Protocol Versions**: Supports FIX 4.0 through FIX 5.0 SP2
  - **Session Management**: Automatic session management, message sequencing, and recovery
  - **Message Validation**: Built-in message validation and field validation
  - **Logging**: Comprehensive logging for debugging and compliance
  - **File-Based Storage**: File-based message store for persistence
  - **Database Storage**: Optional database storage for message persistence

- **Language Implementations**:
  - **QuickFIX/J**: Java implementation
  - **QuickFIX/n**: .NET implementation
  - **QuickFIX/py**: Python implementation
  - **QuickFIX++**: C++ implementation

- **Relevance to Box Spread Trading**:
  - **FIX Protocol Implementation**: Use QuickFIX to implement FIX API connectivity for direct exchange access
  - **C++ Support**: QuickFIX++ provides C++ implementation matching project technology stack
  - **Direct CBOE/CME Access**: Implement FIX connectivity to exchanges for box spread execution

- **Use Cases**:
  - Implementing FIX API client for broker/exchange connectivity
  - Building FIX protocol infrastructure for trading systems
  - Testing FIX message handling and session management
  - Direct exchange connectivity for options trading

- **Note**: QuickFIX is one of the most widely used open-source FIX protocol engines.
  QuickFIX++ (C++) is particularly relevant for this project's technology stack.

#### fix8.org - FIX Protocol Library

- **Website**: <https://fix8.org/>
- **Description**: FIX protocol library and development tools
- **Key Features**:
  - **FIX Protocol Library**: FIX protocol implementation library
  - **Development Tools**: Tools for FIX protocol development
  - **Documentation**: FIX protocol documentation and resources

- **Relevance to Box Spread Trading**:
  - **FIX Protocol Implementation**: Library for implementing FIX API connectivity
  - **Development Resources**: Tools and documentation for FIX development

- **Note**: Additional FIX protocol library option for implementing FIX connectivity.

#### FIX Simulator Tools

##### FIXSim.com - FIX Simulator

- **Website**: <https://www.fixsim.com/>
- **FIX Simulator**: <https://www.fixsim.com/fix-simulator>
- **Description**: FIX protocol simulator for testing and development
- **Key Features**:
  - **FIX Protocol Simulation**: Simulate FIX protocol exchanges for testing
  - **Message Testing**: Test FIX message handling and responses
  - **Session Management**: Test FIX session management and recovery
  - **Development Tool**: Useful for developing and testing FIX implementations

- **Relevance to Box Spread Trading**:
  - **Testing FIX Implementation**: Test FIX API connectivity before connecting to real exchanges
  - **Development**: Develop and validate FIX message handling
  - **Integration Testing**: Test FIX integration without live exchange connectivity

- **Use Cases**:
  - Testing FIX client implementations
  - Validating FIX message formats
  - Simulating exchange responses
  - Development and debugging of FIX connectivity

##### Esprow FIX Exchange Simulator

- **Website**: <https://www.esprow.com/index.php>
- **FIX Exchange Simulator**: <https://www.esprow.com/products/fix-testing/fix-exchange-simulator.php>
- **Provider**: Esprow - Financial technology solutions
- **Description**: Professional FIX exchange simulator for testing and development
- **Key Features**:
  - **FIX Exchange Simulation**: Simulate exchange behavior for FIX protocol testing
  - **Message Handling**: Test FIX message handling and responses
  - **Session Management**: Test FIX session management and recovery
  - **Professional Tool**: Enterprise-grade FIX testing tool

- **Relevance to Box Spread Trading**:
  - **Testing FIX Implementation**: Test FIX API connectivity before connecting to real exchanges
  - **Development**: Develop and validate FIX message handling for direct exchange access
  - **Integration Testing**: Test FIX integration without live exchange connectivity

- **Use Cases**:
  - Testing FIX client implementations
  - Validating FIX message formats
  - Simulating exchange responses
  - Professional FIX development and testing

##### B2Bits FIX Client Simulator

- **Website**: <https://www.b2bits.com/trading_solutions/fix-tools/fix-client-simulator>
- **Provider**: B2Bits - Financial technology solutions
- **Description**: FIX client simulator for testing FIX server implementations
- **Key Features**:
  - **FIX Client Simulation**: Simulate FIX client behavior for testing FIX servers
  - **Message Generation**: Generate FIX messages for testing
  - **Session Management**: Test FIX session management from client perspective
  - **Testing Tool**: Tool for testing FIX server implementations

- **Relevance to Box Spread Trading**:
  - **Testing FIX Servers**: Test FIX server implementations (if building FIX server)
  - **Development**: Develop and validate FIX server message handling
  - **Integration Testing**: Test FIX server integration

- **Use Cases**:
  - Testing FIX server implementations
  - Validating FIX server message handling
  - Simulating client behavior
  - FIX server development and testing

##### FIX Trading Simulator (Open Source)

- **GitHub**: <https://github.com/felipewind/fix-trading-simulator>
- **License**: MIT License
- **Description**: Open-source trading simulator between a Broker and a Stock Exchange using FIX Protocol
- **Technology Stack**:
  - **QuickFIX/J**: Java FIX protocol engine
  - **Quarkus**: Java framework for back-end
  - **Angular**: Front-end framework
  - **Docker & Docker Compose**: Containerization
  - **PostgreSQL**: Database

- **Key Features**:
  - **Broker-Exchange Simulation**: Complete trading simulator with broker and exchange components
  - **FIX Protocol Communication**: Broker and exchange communicate via QuickFIX/J
  - **Order Management**: Submit, negotiate, cancel, and list orders
  - **Session Management**: FIX session logon/logout and status monitoring
  - **Real-Time Updates**: WebSocket-based real-time updates to front-end
  - **Message Logging**: FIX message logging and event tracking
  - **Docker Support**: Easy deployment with Docker Compose

- **System Architecture**:
  - **Broker Back-End**: Quarkus-based broker system
  - **Broker Front-End**: Angular-based broker interface
  - **Exchange Back-End**: Quarkus-based exchange system
  - **Exchange Front-End**: Angular-based exchange interface
  - **PostgreSQL**: Database for both broker and exchange

- **Relevance to Box Spread Trading**:
  - **FIX Protocol Learning**: Study FIX protocol implementation in a complete trading system
  - **Testing FIX Connectivity**: Test FIX client implementations against simulated exchange
  - **Development Reference**: Reference implementation for FIX protocol integration
  - **Box Spread Testing**: Test box spread order execution via FIX protocol

- **Use Cases**:
  - Learning FIX protocol implementation
  - Testing FIX client connectivity
  - Developing FIX protocol integration
  - Testing trading strategies via FIX protocol
  - Reference implementation for FIX-based trading systems

- **Running the Project**:
  - **Docker Compose**: Easy setup with `docker-compose up`
  - **Access Points**:
    - Broker Front-End: <http://localhost/>
    - Broker Back-End Swagger: <http://localhost:8080/q/swagger-ui/>
    - Exchange Front-End: <http://localhost:90/>
    - Exchange Back-End Swagger: <http://localhost:8090/q/swagger-ui/>

- **Note**: This is an excellent open-source reference implementation for FIX protocol trading systems.
  Particularly useful for learning FIX protocol implementation and testing FIX connectivity before connecting to real exchanges.
  The complete broker-exchange simulation provides a realistic testing environment.

##### FIXPusher

- **Website**: <https://fixpusher.sourceforge.net/>
- **SourceForge**: Open-source project hosting
- **Description**: FIX protocol message pusher tool
- **Key Features**:
  - **FIX Message Pushing**: Tool for pushing FIX messages
  - **Testing Tool**: Useful for testing FIX message handling
  - **Open Source**: Open-source FIX protocol tool

- **Relevance to Box Spread Trading**:
  - **FIX Message Testing**: Test FIX message handling and responses
  - **Development Tool**: Tool for FIX protocol development and testing

- **Use Cases**:
  - Testing FIX message handling
  - Pushing FIX messages for testing
  - FIX protocol development and debugging

- **Note**: Open-source FIX protocol tool for message pushing and testing.

### Trading Simulators & Testing Tools

<!--
@index: api-documentation
@category: trading-simulators
@tags: simulator, backtesting, testing, strategy-validation
@last-updated: 2025-01-27
-->

#### QuantReplay - Open-Source Multi-Asset Trading Simulator

- **Website**: <https://www.quodfinancial.com/quantreplay-open-source-trading-simulator/>
- **GitHub**: <https://github.com/quodfinancial/quantreplay>
- **Provider**: Quod Financial
- **License**: Open-source
- **Description**: Open-source, multi-asset trading simulator designed for testing and validating trading strategies in realistic market environments.
  Originally developed for Quod Financial's internal use, now freely available to the global trading community.
- **Key Features**:
  - **Multi-Asset Support**: Equities, FX, futures, and digital assets
  - **Historical Data Playback**: Reconstruct real market scenarios for backtesting
  - **Synthetic Order Flow Modeling**: Simulate unpredictable market behavior
  - **Stochastic Order Generation**: Simulate edge cases and stress conditions
  - **Real-Time Simulation Engine**: Model auction periods, continuous trading, and other market phases
  - **Order Book-Driven Environment**: High-fidelity simulation with realistic order book dynamics
  - **Fully-Featured Matching Engine**: Complete matching engine for realistic execution simulation
  - **Market Data Generation**: Customizable market data generation
  - **Self-Hosted**: Deploy and run on your own infrastructure

- **Relevance to Box Spread Trading**:
  - **Strategy Testing**: Test box spread strategies in realistic market environments
  - **Multi-Asset Backtesting**: Backtest multi-asset strategies with precision
  - **Order Book Simulation**: Test box spread execution with realistic order book dynamics
  - **Stress Testing**: Test strategies under varied liquidity and volatility conditions
  - **Execution Validation**: Validate execution algorithms before live trading

- **Use Cases**:
  - Backtesting box spread strategies with historical data
  - Testing execution algorithms in realistic market conditions
  - Validating multi-leg options strategies
  - Stress testing under various market conditions
  - Developing and fine-tuning trading algorithms

- **Integration Considerations**:
  - **Self-Hosted Deployment**: Deploy on your own infrastructure
  - **GitHub Repository**: Open-source, contributions welcome
  - **Institutional-Grade**: Originally developed for Quod Financial's internal use
  - **Extensible Platform**: Extensible and customizable for specific needs

- **Comparison with Other Simulators**:
  - **vs. Paper Trading**: More realistic than simple paper trading (order book simulation)
  - **vs. Basic Backtesting**: Includes order book dynamics and synthetic order flow
  - **vs. FIX Trading Simulator**: Focuses on strategy testing rather than FIX protocol

- **Contact**: Visit <https://quantreplay.com> or contact <info@quodfinancial.com>
- **Note**: QuantReplay is an institutional-grade trading simulator now available as open-source.
  Particularly valuable for testing box spread strategies in realistic market environments with order book dynamics.
  The multi-asset support and historical data playback make it ideal for comprehensive strategy validation before live trading.

#### Stotra - Multiplayer Stock Trading Simulator

- **GitHub**: <https://github.com/spikecodes/stotra>
- **Live Demo**: <https://stotra.spike.codes>
- **License**: MIT License
- **Description**: Multiplayer stock trading simulator allowing real-time virtual trading of stocks, currencies, and cryptocurrencies.
  Built with React (MERN stack) and deployed on AWS.
- **Technology Stack**:
  - **Frontend**: React, TypeScript, Chakra UI, Axios, Highcharts
  - **Backend**: Node.js, Express, document database, JWT
  - **Deployment**: AWS Amplify (frontend), AWS EC2 (backend), hosted document database
  - **Authentication**: JWT-based authentication with Cloudflare Turnstile

- **Key Features**:
  - **Real-Time Virtual Trading**: Trade stocks, currencies, and cryptocurrencies without real money
  - **Multiplayer Leaderboard**: Competitive trading with friends
  - **Interactive Charts**: Highcharts-based visualizations for decision-making
  - **Financial News**: Access to financial news for informed trading
  - **Dark Mode**: Beautiful design with dark mode and customizable accent colors
  - **Responsive Design**: Trading on-the-go with mobile support

- **Relevance to Box Spread Trading**:
  - **Learning Tool**: Practice trading concepts before implementing automated strategies
  - **UI/UX Reference**: Reference for building trading interfaces
  - **MERN Stack Reference**: Reference implementation for full-stack trading applications
  - **Real-Time Updates**: WebSocket-based real-time updates (reference for real-time trading UIs)

- **Use Cases**:
  - Learning trading concepts and market dynamics
  - UI/UX reference for trading applications
  - Full-stack trading application reference
  - Testing trading ideas in a virtual environment

- **Integration Considerations**:
  - **Open Source**: MIT licensed, can be used as reference or modified
  - **MERN Stack**: Modern full-stack architecture reference
  - **AWS Deployment**: Reference for cloud deployment patterns
  - **Real-Time Features**: WebSocket implementation reference

- **Limitations**:
  - Focuses on stocks/currencies/crypto, not options
  - Virtual trading simulator, not for live trading
  - No API for automated trading

- **Note**: Stotra is a well-designed virtual trading simulator useful as a reference for building trading UIs and full-stack trading applications.
While it doesn't support options trading, it provides excellent examples of real-time trading interfaces, authentication, and full-stack architecture
patterns.

#### PyMarketSim / TradingAgents - Financial Market Simulation for Deep Reinforcement Learning

- **GitHub**: <https://github.com/TauricResearch/TradingAgents>
- **Paper**: <https://openreview.net/forum?id=EXFIW61dG8>
- **License**: Open-source (check repository for license)
- **Description**:
  Financial market simulation environment designed for training and evaluating trading agents using deep reinforcement learning (dRL).
  Agent-based environment with private valuations, asymmetric information, and flexible limit order book mechanism.
- **Key Features**:
  - **Deep Reinforcement Learning**: Designed for training and evaluating trading agents using dRL
  - **Agent-Based Environment**: Incorporates private valuations and asymmetric information
  - **Limit Order Book**: Flexible limit order book mechanism
  - **Single-Agent & Multi-Agent**: Supports both single-agent and multi-agent dRL settings
  - **TRON Agents**: Trained response order networks (TRON agents) implemented as recurrent neural networks
  - **Empirical Game Theory**: Multi-agent equilibrium identification using empirical game-theoretic techniques
  - **Market Dynamics**: Study complex market dynamics and emergent behaviors

- **Relevance to Box Spread Trading**:
  - **Strategy Development**: Develop and test box spread strategies using reinforcement learning
  - **Market Dynamics**: Study market dynamics and order book behavior
  - **Agent Training**: Train trading agents for automated box spread execution
  - **Multi-Agent Simulation**: Test strategies in multi-agent environments

- **Use Cases**:
  - Training trading agents using deep reinforcement learning
  - Studying market dynamics and emergent behaviors
  - Developing advanced trading algorithms
  - Testing strategies in agent-based environments
  - Research on financial market simulation

- **Integration Considerations**:
  - **Open Source**: Available on GitHub for research and development
  - **Python-Based**: Python implementation for dRL
  - **Research Tool**: Primarily designed for research and algorithm development
  - **Reinforcement Learning**: Requires understanding of dRL concepts

- **Note**: PyMarketSim/TradingAgents is a research-oriented tool for training trading agents using deep reinforcement learning.
Particularly useful for developing and testing automated trading strategies, including box spread strategies, in simulated market environments with
realistic order book dynamics.

#### MarS (Market Simulation) - Financial Market Simulation Engine

- **Website**: <https://mars-lmm.github.io/>
- **GitHub**: <https://github.com/mars-lmm> (check for repository)
- **Provider**: Microsoft Research Asia
- **Description**:
  Financial market simulation engine powered by Large Market Model (LMM), an order-level generative foundation model for financial market simulation.
  Addresses domain-specific needs for realistic, interactive, and controllable order generation.
- **Key Features**:
  - **Large Market Model (LMM)**: Order-level generative foundation model trained on historical financial market data
  - **Realistic Simulations**: Generates realistic market simulations with order-level granularity
  - **Interactive Order Generation**: Dynamically generates order series in response to user-injected interactive orders
  - **Controllable Generation**: Supports vague target scenario descriptions and market impact analysis
  - **Real-Time Simulation**: Real-time order matching in simulated clearing house
  - **Fine-Grained Trajectories**: Produces fine-grained simulated market trajectories
  - **Scalability**: Strong scalability across data size and model complexity

- **Applications**:
  - **Forecast**: Market trend prediction and forecasting
  - **Detection**: Anomaly detection and market abuse regulation
  - **"What IF" Analysis**: Market impact analysis and trading strategy evaluation
  - **Reinforcement Learning Environment**: Training environment for RL trading agents

- **Market Impact Analysis**:
  - **Square-Root-Law**: Confirms market impact follows Square-Root-Law model
  - **New Factors**: Identifies factors beyond Square-Root-Law (resiliency, LOB pressure, LOB depth)
  - **Long-Term Impact**: Models dynamics of long-term market impact using ODE

- **Relevance to Box Spread Trading**:
  - **Strategy Testing**: Test box spread strategies in realistic market simulations
  - **Market Impact Analysis**: Analyze market impact of box spread execution
  - **Forecasting**: Forecast market trends for box spread opportunities
  - **Anomaly Detection**: Detect market anomalies that could affect box spread execution
  - **RL Agent Training**: Train RL agents for automated box spread trading

- **Use Cases**:
  - Testing trading strategies in realistic market environments
  - Analyzing market impact of large trades
  - Forecasting market trends
  - Training reinforcement learning agents
  - Market anomaly detection
  - "What IF" analysis for trading strategies

- **Integration Considerations**:
  - **Foundation Model**: Powered by Large Market Model (LMM)
  - **Order-Level Granularity**: Fine-grained order-level simulation
  - **Research Tool**: Developed by Microsoft Research Asia
  - **Scalability**: Strong scalability with larger datasets and models

- **Note**: MarS is a cutting-edge financial market simulation engine powered by generative foundation models.
Particularly valuable for testing box spread strategies with realistic market dynamics, analyzing market impact, and training RL agents for automated
trading.
  The order-level granularity and realistic simulations make it ideal for comprehensive strategy validation.

### Quantitative Finance Libraries

<!--
@index: api-documentation
@category: quantitative-finance
@tags: quantitative-finance, options-pricing, greeks, risk-management
@last-updated: 2025-11-18
-->

#### C++ Financial Libraries Research

- **Research Document**: `docs/RESEARCH_CPP_FINANCIAL_LIBRARIES.md`
  - Comprehensive analysis of 10 C++ financial software resources
  - Integration priority matrix and recommendations
  - CMake integration examples for QuantLib, Eigen, NLopt
  - License compatibility analysis
  - Risk assessment and mitigation strategies
  - **Key Recommendations**: QuantLib (high priority), Eigen (high priority), NLopt (medium priority)
  - **Resources Analyzed**:
    QuantLib, Option Pricer (GitHub), Eigen, NLopt, OnixS FIX Engine, StockChartX, UnoAPI/SYCL, C++ for Quants, Medium articles
  - Created: 2025-11-18

#### Eigen - Linear Algebra Library

- **Status**: ✅ **Integrated** (2025-11-18)
- **Integration Guide**: `docs/EIGEN_INTEGRATION.md`
- **Version**: Eigen 3.4.0
- **License**: MPL2 (Mozilla Public License 2.0)
- **Use Cases**: Portfolio optimization, convexity calculations, matrix operations
- **CMake Integration**: Via FetchContent (GitLab repository)
- **Testing**: Integration tests in `native/tests/eigen_integration_test.cpp`
- **Documentation**: Comprehensive usage examples and performance considerations

#### QuantLib - Quantitative Finance Library

- **Status**: 📋 **Documentation Prepared** (2025-11-18)
- **Integration Guide**: `docs/QUANTLIB_INTEGRATION_GUIDE.md`
- **License**: BSD 3-Clause
- **Use Cases**: Option pricing, Greeks calculations, volatility modeling, yield curves
- **Prerequisites**: Boost libraries (date_time, filesystem, system)
- **CMake Integration**: Via FetchContent or find_package
- **Documentation**: Complete integration guide with usage examples

#### NLopt - Nonlinear Optimization Library

- **Status**: 📋 **Documentation Prepared** (2025-11-18)
- **Integration Guide**: `docs/NLOPT_INTEGRATION_GUIDE.md`
- **License**: LGPL or MIT (MIT recommended)
- **Use Cases**: Convexity optimization, portfolio rebalancing, spare cash allocation
- **Prerequisites**: None (self-contained)
- **CMake Integration**: Via FetchContent or find_package
- **Documentation**: Complete integration guide with algorithm selection guide

#### QuantLib - Free/Open-Source Library for Quantitative Finance

- **Website**: <https://www.quantlib.org/>
- **GitHub**: <https://github.com/lballabio/quantlib>
- **License**: Modified BSD License (suitable for both free software and proprietary applications)
- **Description**: Comprehensive software framework for quantitative finance.
  Free/open-source library for modeling, trading, and risk management in real-life financial applications.
  Used by banks, software companies, regulatory institutions, researchers, and students worldwide.
- **Documentation**: Available in several formats from multiple sources (see website)
- **Support**: Mailing list for questions and community support
- **Contributions**: Open to contributions via GitHub pull requests
- **Key Features**:
  - **C++ Core**: Written in C++ with clean object model
  - **Multi-Language Bindings**: Exported to C#, Java, Python, and R
  - **Excel Add-in**: QuantLibXL for Excel integration
  - **LibreOffice**: QuantLibAddin for LibreOffice Calc
  - **Comprehensive Tools**: Tools for practical implementation and advanced modeling
  - **Options Pricing**: Extensive options pricing models
  - **Risk Management**: Risk management tools and calculations
  - **Yield Curves**: Yield curve construction and analysis
  - **Monte Carlo**: Monte Carlo simulation capabilities

- **Language Support**:
  - **C++**: Core implementation with clean object model
  - **Python**: Python bindings (QuantLib-Python)
  - **Java**: Java bindings
  - **C#**: .NET bindings
  - **R**: R bindings

- **Extensions**:
  - **QuantLibXL**: Excel add-in for QuantLib
  - **QuantLibAddin**: Add-ins for LibreOffice Calc and other platforms
  - **AAD-Enabled Versions**: Automatic Differentiation (AAD) enabled versions available

- **Relevance to Box Spread Trading**:
  - **Options Pricing**: Comprehensive options pricing models for box spread valuation
  - **Greeks Calculation**: Calculate Greeks (delta, gamma, theta, vega) for options
  - **Yield Curve**: Yield curve construction for risk-free rate estimation
  - **Risk Management**: Risk management tools for position sizing and VaR
  - **C++ Integration**: Native C++ library matches project technology stack
  - **Theoretical Pricing**: Validate theoretical box spread prices

- **Use Cases**:
  - Options pricing and valuation
  - Greeks calculation for risk management
  - Yield curve construction
  - Monte Carlo simulation
  - Risk management calculations
  - Financial instrument modeling

- **Integration Considerations**:
  - **C++ Core**: Native C++ implementation (matches project stack)
  - **Python Bindings**: Can be used from Python if needed
  - **Well-Established**: Industry-standard library used by academics and practitioners
  - **Comprehensive**: Extensive feature set for quantitative finance

- **Note**: QuantLib is the industry-standard open-source library for quantitative finance.
  Particularly valuable for options pricing, Greeks calculation, and risk management in box spread trading.
The C++ core provides native integration that matches the project's technology stack, while Python bindings offer flexibility for analysis and
prototyping.

### Financial Infrastructure & Ledger Systems

<!--
@index: api-documentation
@category: financial-infrastructure
@tags: ledger, banking, double-entry, reconciliation
@last-updated: 2025-01-27
-->

#### Blnk - Open-Source Financial Ledger

- **GitHub**: <https://github.com/blnkfinance/blnk>
- **Documentation**: <https://docs.blnkfinance.com>
- **License**: Apache-2.0 License
- **Description**: Open-source ledger and financial core for shipping fintech products fast.
  Developer-first toolkit designed for developers who want to ship fintech products without compromising compliance and correctness.
- **Key Components**:
  - **Ledger**: Open-source double-entry ledger for managing balances and recording transaction workflows
  - **Reconciliation**: Automatically match external records (bank statements) to internal ledger with custom matching rules
  - **Identity Management**: Create and manage identities with PII tokenization and link to balances/transactions

- **Ledger Features**:
  - **Balance Monitoring**: Real-time balance monitoring and tracking
  - **Balance Snapshots**: Historical balance snapshots
  - **Historical Balances**: Track balance history over time
  - **Inflight Transactions**: Hold transactions and commit/void as needed
  - **Scheduling and Overdrafts**: Schedule transactions and handle overdrafts
  - **Bulk Transactions**: Process multiple transactions at once
  - **Backdated Transactions**: Record transactions with past dates

- **Technology Stack**:
  - **Language**: Go (95.4% of codebase)
  - **Database**: PostgreSQL
  - **Cache**: Redis
  - **Search**: Typesense
  - **Deployment**: Docker Compose, Kubernetes support

- **Use Cases**:
  - Wallet Management
  - Deposits & Withdrawals
  - Order Exchange
  - Lending
  - Loyalty Points System
  - Savings Application
  - Escrow Application

- **Relevance to Box Spread Trading**:
  - **Transaction Tracking**: Track box spread transactions and P&L
  - **Balance Management**: Manage trading account balances and positions
  - **Reconciliation**: Reconcile broker statements with internal records
  - **Financial Core**: Financial infrastructure for trading system
  - **Compliance**: Double-entry ledger ensures accounting compliance

- **Integration Considerations**:
  - **Go Implementation**: Go-based system (different from C++ project stack)
  - **REST API**: RESTful API for integration
  - **Docker Deployment**: Easy deployment with Docker Compose
  - **Self-Hosted**: Open-source, self-hosted solution
  - **Financial Compliance**: Double-entry accounting ensures compliance

- **Quick Start**:
  - **Installation**: Docker Compose setup
  - **Configuration**: JSON configuration file
  - **API**: REST API for all operations

- **Use Cases for Box Spread Trading**:
  - Track box spread transaction history
  - Manage trading account balances
  - Reconcile broker statements
  - Financial reporting and compliance
  - P&L tracking and accounting

- **Note**: Blnk is an open-source financial ledger system useful for building financial infrastructure in trading systems.
While the Go implementation differs from the C++ project stack, the REST API allows integration for transaction tracking, balance management, and
reconciliation.
  Particularly valuable for maintaining accurate financial records and compliance in box spread trading operations.

#### Apache Fineract - Core Banking System

- **Website**: <https://fineract.apache.org/>
- **License**: Apache License 2.0
- **Description**:
  Open-source software for financial services, designed to create a cloud-ready core banking system that enables digital financial services.
  Sophisticated core banking system with comprehensive financial technology solutions.
- **Key Features**:
  - **Client Data Management**: Comprehensive client data management system
  - **Loan and Savings Portfolio Management**: Complete loan and investment tracking
  - **Integrated Real-Time Accounting**: Real-time accounting integration
  - **Extensive Reporting**: Comprehensive reporting capabilities
  - **Flexible Product Configuration**: Customize financial products
  - **KYC Documentation Support**: Flexible customer system of record
  - **Business Rule Sets**: Four eyes principles and configurable workflows
  - **Payment Recognitions**: System of record for repayments

- **Deployment Options**:
  - **Cloud Deployment**: Scalable cloud-based solutions
  - **On-Premise**: Traditional deployment options
  - **Mobile Access**: Headless design allows third-party mobile solutions
  - **Open API**: Comprehensive API support since 2011
  - **Extensible Architecture**: Support for any organizational type or delivery channel

- **Relevance to Box Spread Trading**:
  - **Financial Infrastructure**: Core banking infrastructure for financial operations
  - **Transaction Management**: Transaction tracking and management
  - **Reporting**: Financial reporting capabilities
  - **Compliance**: Banking-grade compliance and security
  - **⚠️ Limited Direct Relevance**: Primarily designed for banking/financial services, not trading-specific

- **Integration Considerations**:
  - **Banking Focus**: Designed for banking/financial services, not trading
  - **Open API**: Comprehensive API for integration
  - **Cloud-Ready**: Cloud-native architecture
  - **Enterprise-Grade**: Proven in high-transaction environments

- **Use Cases**:
  - Building banking/financial services infrastructure
  - Core banking system implementation
  - Financial services platform development
  - Digital banking solutions

- **Note**: Apache Fineract is a comprehensive core banking system designed for financial services and banking operations.
  While it provides robust financial infrastructure, it is primarily designed for banking/financial services rather than trading-specific use cases.
  For box spread trading, it may be over-engineered unless building a broader financial services platform that includes trading capabilities.
  Consider for comprehensive financial infrastructure needs beyond just trading.

### FIX API Providers

This section covers FIX protocol API providers for direct exchange access and institutional trading. For FIX protocol development tools and
libraries, see the "FIX Protocol Development Tools & Libraries" section above.

**Quick Comparison**:

| Provider                    | Focus              | Latency         | Options Support | Best For                 | Contact         |
| --------------------------- | ------------------ | --------------- | --------------- | ------------------------ | --------------- |
| **Tools for Brokers (TFB)** | Platform           | Ultra-low       | ✅ Verify       | Direct CBOE, multi-venue | <sales@t4b.com> |
| **4T**                      | Institutional      | Ultra-low (LD4) | ✅ Verify       | LD4 proximity, PrimeXM   | Contact 4T      |
| **B2PRIME**                 | Prime of Prime     | Low             | ⚠️ FOREX/CFD    | FOREX/CFD strategies     | Contact B2PRIME |
| **ATFX**                    | Broker             | Low             | ⚠️ Verify       | Custom integration       | Contact ATFX    |
| **Kraken**                  | Crypto Derivatives | Ultra-low       | ⚠️ Crypto only  | Crypto derivatives       | Contact Kraken  |
| **OnixS directConnect**     | SDK                | Ultra-low       | ✅ Full         | Direct exchange SDK      | Contact OnixS   |

#### Tools for Brokers (TFB) FIX API Platform

- **Website**: <https://t4b.com/fix-api/>
- **Provider**: Tools for Brokers (TFB) - Technology provider for retail brokers
- **Description**:
  FIX API platform for retail brokers, hedge funds, and liquidity providers enabling ultra-low latency trading execution and liquidity aggregation
- **Key Features**:
  - **FIX API Platform**: Industry-standard FIX protocol implementation
  - **Liquidity Aggregation**: Connect to 100+ liquidity providers via single point of access
  - **Ultra-Low Latency**: Built-in Margin Engine for fast execution (no external platforms required)
  - **Direct Market Access**: Connect directly to exchanges and liquidity pools
  - **Platform Integration**: Integrate with trading platforms (MT4, MT5, custom) via FIX or REST API
  - **Liquidity Provider Capabilities**: Distribute liquidity to other brokers, White Label clients
  - **Web Interface**: User-friendly dashboard for monitoring exposure, positions, trading history

- **Technical Details**:
  - **Extended FIX API**: Full FIX protocol support with custom integration capabilities
  - **FIX API Emulator**: Migration tool for seamless transition from other bridging solutions
  - **Trade Processor**: Core platform with liquidity bridge, risk management, reporting
  - **Integration Support**: TFB tech team assists with integration

- **Relevance to Box Spread Trading**:
  - **Direct CBOE Access**: Execute SPX/SPXW box spreads directly via FIX protocol
  - **Multi-Venue Trading**: Access multiple liquidity providers for best execution prices
  - **Ultra-Low Latency**: Fast execution critical for arbitrage opportunities
  - **Platform Independence**: Integrate with existing C++ box spread trading system
  - **Risk Management**: Built-in Margin Engine for real-time margin calculations

- **Comparison with Current Solutions**:
  - **vs. TWS API**: FIX protocol vs. socket-based, direct exchange access vs. via broker, multi-venue vs. single venue
  - **vs. Alpaca REST API**: FIX protocol vs. REST, ultra-low latency vs. low latency, multi-venue vs. single venue

- **Use Cases**:
  - Direct CBOE access for box spread execution
  - Multi-venue arbitrage (execute legs across different venues)
  - High-frequency box spread scanning and execution
  - Risk management integration with built-in Margin Engine

- **Integration Considerations**:
  - **Complementary Execution Venue**: Use alongside TWS API for multi-venue trading
  - **Direct Exchange Access**: Bypass broker for faster execution and lower latency
  - **Multi-Venue Strategy**: Execute different legs on different venues for optimal prices
  - **FIX Protocol Implementation**: Requires FIX message handling, session management, order routing

- **Migration Support**:
  - **FIX API Emulator**: Seamless migration from alternative bridging solutions
  - **Integration Support**: TFB tech team assists with integration process
  - **Reduced Downtime**: Faster migration with minimal trading disruption

- **Contact**: <sales@t4b.com> (sales), <marketing@t4b.com> (general inquiries)
- **Documentation**: See `docs/TOOLS_FOR_BROKERS_FIX_API.md` for comprehensive platform details, integration considerations, and use cases
- **Note**: TFB FIX API platform could complement or replace broker APIs for direct exchange access. Contact TFB for pricing and capabilities.
  Evaluate as alternative execution venue for box spread trading, particularly for direct CBOE access and multi-venue arbitrage strategies.

#### 4T FIX API - Institutional Trading Solutions

- **Website**: <https://4t.com/en/institutional-trading-solutions/fix-api>
- **Provider**: 4T - Financial brokerage firm specializing in institutional trading
- **Description**: FIX API trading solution with PrimeXM XCore integration,
  providing ultra-low latency FIX API trading with cross-connected liquidity providers based in LD4 (London data center)
- **Key Features**:
  - **FIX Protocol**: Industry-standard Financial Information eXchange (FIX) protocol for electronic trading
  - **PrimeXM XCore Integration**: Partnership with PrimeXM's XCore trading and aggregation engine
  - **Ultra-Low Latency**: LD4 data center location for fast, low-latency execution
  - **Cross-Connected Liquidity**: Network of 250+ liquidity partners via PrimeXM XCore
  - **Multiple Asset Classes**: Supports stocks, bonds, options, and other financial instruments
  - **Security**: Advanced encryption techniques for secure data transmission

- **PrimeXM XCore Technology**:
  - **Trading Engine**: Top-rated trading and aggregation engine
  - **Order Management**: Ultra-low latency order management
  - **Risk Management**: Built-in risk management capabilities
  - **Reporting**: Comprehensive reporting and monitoring
  - **Liquidity Network**: 250+ partners for efficient liquidity exchange
  - **Multi-Asset Support**: Supports multiple asset classes

- **LD4 Data Center**:
  - **Location**: LD4 (London) - one of the world's most important data centers for financial trading
  - **Benefits**: Fast, low-latency, reliable access to global financial markets
  - **Infrastructure**: Cross-connected liquidity providers for optimal execution

- **FIX Protocol Advantages**:
  - **Flexibility**: Transmit data between any financial institutions regardless of size or location
  - **Industry Standard**: Constantly updated to reflect changes in financial industry
  - **Security**: Advanced encryption for protection from tampering or interception
  - **Speed**: Simple, fast protocol designed for electronic exchange
  - **Comprehensive**: Supports trade confirmations, market data, order status updates

- **Relevance to Box Spread Trading**:
  - **FIX Protocol**: Direct FIX API access for algorithmic trading
  - **Low Latency**: LD4 data center location critical for arbitrage strategies
  - **Options Support**: FIX protocol supports options trading (verify specific capabilities)
  - **Multi-Venue**: Access to 250+ liquidity partners via PrimeXM XCore
  - **Institutional Focus**: Designed for institutional clients and sophisticated traders

- **Integration Considerations**:
  - **FIX Protocol Implementation**: Requires FIX message handling, session management
  - **PrimeXM XCore**: Integration with PrimeXM's aggregation engine
  - **LD4 Connectivity**: May require co-location or proximity hosting for optimal latency
  - **Institutional Requirements**: Contact 4T for pricing, minimums, and capabilities

- **Comparison with Current Solutions**:
  - **vs. TWS API**: FIX protocol vs. socket-based, multi-venue vs. single venue, LD4 latency vs. standard
  - **vs. Alpaca REST API**: FIX protocol vs. REST, institutional vs. retail-focused, LD4 vs. cloud

- **Use Cases**:
  - Institutional box spread execution via FIX API
  - Multi-venue arbitrage with access to 250+ liquidity partners
  - High-frequency trading with LD4 data center proximity
  - Direct market access for options and equities

- **Contact**: Contact 4T directly for institutional FIX API access, pricing, and capabilities
- **Note**: 4T FIX API with PrimeXM XCore integration offers institutional-grade FIX protocol access with ultra-low latency via LD4 data center.
  Particularly relevant for institutional traders requiring direct FIX connectivity and multi-venue liquidity access.
  Verify options trading capabilities and minimum requirements before integration.
  Evaluate as alternative to TWS API for institutional FIX-based trading.

### B2PRIME - Prime of Prime Liquidity Provider with FIX API

- **Website**: <https://b2prime.com/>
- **RAW Account Types**: <https://b2prime.com/account-types/raw>
- **Provider**: B2PRIME - Global group of regulated prime of prime liquidity providers
- **Description**: Prime of prime liquidity provider offering FIX API connectivity,
  RAW account types with ultra-low raw spreads and fixed commissions, and multiple connectivity options for institutional and professional traders
- **Regulated Entities**:
  - **B2B Prime Services EU Limited**: Cyprus (CySEC license 370/18)
  - **B2B Prime Services**: Mauritius (FSC license C117017139)
  - **B2B Prime Services SC Ltd**: Seychelles (FSA license SD 192)
  - **B2B Prime Services Africa (Pty) Ltd**: South Africa (FSCA license 54191)
  - **B2B Prime Services Mena Limited**: Dubai (DFSA license F009446)

- **Key Features**:
  - **FIX API**: Low-latency trading protocol enabling direct market access for high-speed execution
  - **RAW Account Types**: Ultra-low raw spreads with fixed commissions (ideal for high-volume trading)
  - **Standard Account Types**: Most popular choice, ideal for all types of traders
  - **Multiple Connectivity Options**: FIX API, PrimeXM XCORE, oneZero hub, Bridge for MT4/MT5, Centroid, FXCubic, B2CONNECT
  - **Trading Platforms**: cTrader, B2TRADER, TradingView integration
  - **Multi-Asset Support**: FOREX, Cryptocurrencies, Metals, NDFs, Indices, Commodities, Energies

- **Account Types**:
  - **Standard**: Most popular choice, ideal for all types of traders
  - **RAW**: Ultra-low raw spreads and fixed commissions (optimal for algorithmic and high-volume trading)

- **Connectivity Options**:
  - **FIX API**: Low-latency trading protocol for direct market access
  - **PrimeXM XCORE**: High-performance liquidity aggregation and connectivity solution
  - **oneZero**: Smart bridge and liquidity hub for fast, reliable trade routing and risk management
  - **Bridge for MT4/MT5**: Connects MetaTrader platforms to liquidity providers
  - **B2CONNECT**: B2PRIME's proprietary connectivity solution
  - **Centroid**: Additional connectivity option
  - **FXCubic**: Additional connectivity option

- **Trading Platforms**:
  - **B2TRADER**: High-performance trading platform with advanced features for fast execution, deep liquidity
  - **cTrader**: Leading trading platform with modern UI and advanced trade management features
  - **TradingView**: Advanced charting capabilities integrated with B2PRIME

- **Asset Classes**:
  - **FOREX**: Major and exotic currency pairs
  - **Cryptocurrencies**: Crypto trading support
  - **Metals**: Precious metals trading
  - **NDFs**: Non-deliverable forwards
  - **Indices**: Index CFDs
  - **Commodities**: Commodity trading
  - **Energies**: Energy products

- **Relevance to Box Spread Trading**:
  - **FIX API Access**: Direct FIX protocol connectivity for algorithmic trading
  - **RAW Account**: Ultra-low spreads and fixed commissions ideal for high-frequency strategies
  - **Multi-Connectivity**: Multiple connectivity options (FIX, PrimeXM, oneZero) for flexibility
  - **Institutional Focus**: Prime of prime model suitable for sophisticated traders
  - **Low Latency**: FIX API designed for high-speed execution
  - **⚠️ Asset Limitations**: Primarily FOREX, CFDs, and derivatives - verify options trading capabilities

- **Integration Considerations**:
  - **FIX Protocol**: Requires FIX message handling and session management
  - **Account Type**: RAW account recommended for algorithmic trading (ultra-low spreads, fixed commissions)
  - **Connectivity Choice**: FIX API for direct integration, or PrimeXM/oneZero for platform integration
  - **Regulatory Entity**: Account opened through appropriate regulated entity based on jurisdiction
  - **Options Trading**: Verify if B2PRIME supports options trading (may be primarily FOREX/CFD focused)
  - **Minimum Requirements**: Contact B2PRIME for institutional account minimums and requirements

- **Comparison with Current Solutions**:
  - **vs. TWS API**: FIX protocol vs. socket-based, prime of prime vs. direct broker, multi-connectivity vs. single API
  - **vs. Alpaca REST API**: FIX protocol vs. REST, FOREX/CFD focus vs. equities/options, prime of prime vs. direct broker

- **Use Cases**:
  - High-frequency FOREX trading with RAW account (ultra-low spreads)
  - Multi-venue arbitrage via FIX API and PrimeXM XCORE
  - Algorithmic trading with direct FIX API connectivity
  - Professional trading with institutional-grade infrastructure

- **Contact**: Contact B2PRIME sales for account opening, FIX API access, and pricing
- **Note**:
  B2PRIME is a prime of prime liquidity provider offering FIX API access with RAW account types featuring ultra-low spreads and fixed commissions.
  Particularly relevant for high-frequency and algorithmic trading strategies.
  However, B2PRIME appears primarily focused on FOREX, CFDs, and derivatives - verify options trading capabilities before integration.
  The multiple connectivity options (FIX API, PrimeXM XCORE, oneZero) provide flexibility for different integration approaches.
  Evaluate as alternative execution venue for FOREX-based strategies, but may not support options box spread trading.

### ATFX FIX API

- **Website**: <https://www.atfx.com/en/atfx-fix-api>
- **Provider**: ATFX - Global online trading broker
- **Description**: FIX API solution allowing advanced traders to connect their own trading systems directly to ATFX's infrastructure,
  minimizing latency and providing real-time data access for efficient and scalable trading
- **Key Features**:
  - **FIX Protocol**: Industry-standard FIX protocol for electronic trading
  - **Direct System Integration**: Connect custom trading systems directly to ATFX infrastructure
  - **Low Latency**: Minimized latency for high-speed execution
  - **Real-Time Data**: Real-time market data access
  - **Scalable Trading**: Support for efficient and scalable trading across various custom strategies
  - **Custom Strategies**: Enable algorithmic and automated trading strategies

- **Relevance to Box Spread Trading**:
  - **FIX API Access**: Direct FIX protocol connectivity for algorithmic trading
  - **Low Latency**: Critical for time-sensitive arbitrage strategies
  - **Custom Integration**: Connect existing C++ box spread trading system
  - **Real-Time Data**: Access to real-time market data for strategy execution
  - **⚠️ Asset Limitations**: Verify if ATFX supports options trading (may be primarily FOREX/CFD focused)

- **Integration Considerations**:
  - **FIX Protocol Implementation**: Requires FIX message handling and session management
  - **Custom Trading Systems**: Designed for traders with existing trading infrastructure
  - **Direct Integration**: Connect directly to ATFX infrastructure without broker middleware
  - **Options Trading**: Verify if ATFX supports options trading capabilities
  - **Minimum Requirements**: Contact ATFX for institutional account requirements and FIX API access

- **Comparison with Current Solutions**:
  - **vs. TWS API**: FIX protocol vs. socket-based, direct integration vs. via broker middleware
  - **vs. Alpaca REST API**: FIX protocol vs. REST, institutional vs. retail-focused

- **Use Cases**:
  - Algorithmic trading with direct FIX API connectivity
  - High-frequency trading strategies with low latency
  - Custom trading system integration
  - Multi-strategy trading platform

- **Contact**: Contact ATFX for FIX API access, account requirements, and integration support
- **Note**: ATFX FIX API provides direct system integration for advanced traders with custom trading infrastructure.
  Particularly relevant for algorithmic and high-frequency trading strategies requiring low latency and real-time data access.
  Verify options trading capabilities before integration, as ATFX may be primarily focused on FOREX and CFDs.

### Kraken Derivatives FIX API

- **Website**: <https://blog.kraken.com/product/kraken-derivatives/fix-api>
- **Provider**: Kraken - Cryptocurrency exchange and derivatives platform
- **Description**: FIX API for derivatives trading built on industry-standard FIX 4.4 protocol,
  designed specifically for professional and institutional clients trading cryptocurrency derivatives (futures and options)
- **Key Features**:
  - **FIX 4.4 Protocol**: Industry-standard FIX protocol implementation
  - **Derivatives Trading**: Support for cryptocurrency futures and options trading
  - **High Performance**: Native FIX API built for demanding derivatives trading (microseconds matter)
  - **Level 3 (L3) Market Data**: Full order book access with visibility of individual orders and microsecond-precision sequencing
  - **Comprehensive Functionality**: All order types and instructions available through REST, with additional controls
  - **Cancel-on-Disconnect**: Session-based risk management tool
  - **UAT Environment**: User Acceptance Testing environment providing complete mirror of production systems

- **Performance Characteristics**:
  - **Ultra-Low Latency**: Built for microseconds-critical derivatives trading
  - **Basis Opportunities**: Capture basis opportunities in leveraged positions
  - **Funding Rate Changes**: Respond quickly to funding rate changes
  - **Multi-Leg Strategies**: Execute complex multi-leg strategies efficiently

- **Market Data**:
  - **Level 3 (L3) Order Book**: Full order book visibility (vs. L1 top of book or L2 price aggregated)
  - **Individual Orders**: Visibility of individual orders in the book
  - **Microsecond Precision**: Microsecond-precision sequencing for market data
  - **Deeper Market Insight**: More granular market dynamics compared to traditional feeds

- **Integration Benefits**:
  - **Unified API**: Consistent design with existing Kraken FIX API for spot trading
  - **Seamless Adoption**: If already using Kraken FIX API for spot, derivatives API feels seamless
  - **Robust Documentation**: Clear message protocols and dedicated technical support
  - **Vendor Compatibility**: Works with any vendors compliant with FIX standard

- **Relevance to Box Spread Trading**:
  - **⚠️ Cryptocurrency Focus**: Kraken Derivatives FIX API is for cryptocurrency derivatives (futures/options), not traditional equity options
  - **Multi-Leg Strategies**: Supports complex multi-leg strategies (relevant for box spread concept)
  - **High Performance**: Ultra-low latency critical for arbitrage strategies
  - **L3 Market Data**: Deep order book visibility for better execution
  - **Risk Management**: Cancel-on-Disconnect for session-based risk controls
  - **⚠️ Not Traditional Options**: This is for crypto derivatives, not SPX/SPXW box spreads

- **Integration Considerations**:
  - **Cryptocurrency Derivatives Only**: This API is for crypto futures/options, not traditional equity options
  - **FIX 4.4 Protocol**: Standard FIX protocol implementation
  - **UAT Environment**: Test thoroughly in UAT before production
  - **Account Manager**: Contact Account Manager for UAT access
  - **Developer Portal**: Visit developer portal for detailed documentation

- **Comparison with Current Solutions**:
  - **vs. TWS API**: Crypto derivatives vs. traditional options, FIX protocol vs. socket-based
  - **vs. Traditional Options**: Crypto derivatives vs. equity options (SPX/SPXW)

- **Use Cases**:
  - Cryptocurrency derivatives trading (futures and options)
  - Multi-leg crypto derivative strategies
  - High-frequency crypto derivatives trading
  - Basis trading and funding rate arbitrage
  - **⚠️ Not for Traditional Box Spreads**: This is crypto-focused, not for traditional equity options box spreads

- **Contact**: Contact Kraken API support team or Account Manager for FIX API access and UAT environment
- **Note**: Kraken Derivatives FIX API is a high-performance FIX 4.4 implementation for cryptocurrency derivatives trading (futures and options).
While it supports multi-leg strategies and ultra-low latency execution similar to box spread trading, it is **not applicable for traditional equity
options box spreads** (SPX/SPXW).
  This API is relevant only for cryptocurrency derivatives trading strategies.
The L3 market data and microsecond-precision sequencing make it suitable for sophisticated crypto derivatives trading, but traditional options
traders should use TWS API or other equity-focused FIX APIs.

### FIXAPI.cc - FIX API Consulting & Resource Platform

- **Website**: <https://www.fixapi.cc/>
- **Provider**: FIXAPI.cc - FIX API Trading Platform & Liquidity Provider Influencer
- **Description**:
  Consulting and resource platform that introduces FIX API trading platforms and liquidity providers to Forex, cryptocurrency, and stock traders.
  Provides FIX API programming services, sample source codes, and recommendations for financial institutions and traders.
- **Key Services**:
  - **FIX API Programming**: Assign developers to work on FIX API programming tasks
  - **Sample Source Codes**: Large codebase of FIX API sample source codes (some free, some paid)
  - **Free Source Codes (FOSS)**: Free and open-source FIX API code samples for subscribers
  - **Liquidity Provider Recommendations**: Recommendations for best FIX API trading platforms and liquidity providers
  - **Consulting Services**: Consultants for financial institutions and traders
  - **Newsletter**: Free newsletter with notifications about new resources and code samples

- **Free Resources**:
  - **FIX API Excel VBA**: Free FIX API Excel VBA for live and demo trading (Forex, stocks, cryptocurrency)
  - **Free Source Codes**: Some FOSS code samples available to subscribers
  - **Newsletter**: Free subscription for updates

- **Paid Services**:
  - **FIX API Development**: Custom FIX API algorithm trading system development
  - **App Development**: APP and DAPP development for financial institutions
  - **Branding Services**: Logo design, brand identity, broker branding
  - **Startup Services**: Help startups build Forex or cryptocurrency brokerage

- **Recommended Platforms** (from FIXAPI.cc):
  - **Advanced Markets**: Algorithmic and high-frequency trading with RFQ and ECN platforms in one DMA trading venue
  - **EXANTE**: Trading platform with access to 300,000+ assets from single multi-currency account
  - **FXCM FIX API**: FIX Protocol 4.4 with up to 200 price updates per second (fastest solution), full range of trading order types
    - **⚠️ Note**: FXCM does not allow residents of Israel (not available for Israeli traders)

- **Relevance to Box Spread Trading**:
  - **FIX API Resources**: Sample source codes and programming assistance for FIX API integration
  - **Liquidity Provider Recommendations**: Guidance on selecting appropriate FIX API platforms
  - **Development Support**: FIX API programming services for custom trading systems
  - **Code Samples**: Reference implementations for FIX protocol integration
  - **Educational Resource**: Learning resource for FIX API development

- **Integration Considerations**:
  - **Consulting Service**: Use for FIX API development guidance and programming tasks
  - **Code Samples**: Reference their codebase for FIX API implementation patterns
  - **Free Resources**: Start with free Excel VBA and FOSS code samples
  - **Newsletter**: Subscribe for updates on new resources and code samples
  - **Platform Recommendations**: Use their recommendations as starting point for evaluating FIX API platforms

- **Use Cases**:
  - Learning FIX API development with sample source codes
  - Getting FIX API programming assistance for custom trading systems
  - Finding recommended FIX API trading platforms and liquidity providers
  - Accessing free FIX API tools (Excel VBA, code samples)
  - Consulting services for FIX API integration projects

- **Contact**: Contact FIXAPI.cc through their website for consulting services, code samples, and platform recommendations
- **Note**: FIXAPI.cc is a consulting and resource platform for FIX API development, not a trading platform itself.
It provides valuable resources including sample source codes, programming services, and recommendations for FIX API trading platforms and liquidity
providers.
  Useful for developers learning FIX API or needing assistance with FIX API integration projects.
  The free Excel VBA and FOSS code samples can be helpful starting points for FIX API development.

### OnixS directConnect - Ultra Low Latency DMA SDKs

- **Website**: <https://www.onixs.biz/directconnect.html>
- **Provider**: OnixS - Financial technology solutions
- **Description**: Ultra-low latency Direct Market Access SDKs for exchanges and liquidity pools,
  with multi-platform implementations of Market Data Handlers, Order Routing, and DropCopy/Trade Capture
- **Key Features**:
  - **Ultra-Low Latency**: Designed for lowest latency, highest performance APIs
  - **Protocol Support**: FIX-based (via OnixS FIX Engine) and native binary protocols
  - **Multi-Platform**: C++, .NET Core/.NET, .NET Framework/C#, Java SDKs
  - **Service Guarantee**: SDKs maintained and updated with venue API changes
  - **Development Tools**: Source code samples, market data logging, backtesting support
  - **Free Evaluation**: 30-day trial available

- **CBOE Integration (Critical for Box Spreads)**:
  - **CFE Binary Order Entry (BOE)**: Ultra-low latency binary protocol for CBOE order entry
  - **CFE FIX Order Entry & FIX Drop**: FIX-based order routing and trade capture
  - **CFE Multicast PITCH**: High-throughput market data feed handler
  - **Direct CBOE Access**: Native CBOE connectivity for SPX/SPXW options

- **CME Integration (For Hedging)**:
  - **CME iLink 3 Binary Order Entry**: Low-latency futures order entry
  - **CME MDP Market Data**: Real-time futures market data (TCP/UDP, Premium)
  - **CME SBE Streamlined Handler**: Simple Binary Encoding for high performance
  - **CME STP API**: Straight-through processing

- **Other Supported Venues**:
  - ICE (Intercontinental Exchange): Binary Order Server, FIX Order Server, iMpact Multicast
  - Nasdaq: ITCH Handlers, Nasdaq Fixed Income (NFI)
  - Deutsche Börse: Eurex T7®, Xetra T7®
  - London Stock Exchange: LSE FIX Drop Copy, LSE GTP
  - Euronext: Optiq Market Data Gateway
  - And 15+ more exchanges

- **Relevance to Box Spread Trading**:
  - **Direct CBOE Access**: Execute SPX/SPXW box spreads directly via CFE BOE
  - **Ultra-Low Latency**: Critical for time-sensitive arbitrage opportunities
  - **Real-Time Market Data**: CFE Multicast PITCH for options chain scanning
  - **Multi-Leg Orders**: Support for 4-leg box spread execution
  - **Futures Hedging**: CME iLink 3 for interest rate futures hedging
  - **Multi-Venue Trading**: Execute across CBOE and CME for optimal execution

- **Comparison with Current Solutions**:
  - **vs. TWS API**: Binary/FIX protocols vs. socket-based, direct CBOE vs. via broker, ultra-low vs. low-medium latency
  - **vs. TFB FIX API**: SDK (integrate into app) vs. platform (use their system), native CBOE protocols vs. FIX-only

- **Integration Considerations**:
  - **C++ SDK Available**: Matches project's technology stack
  - **Complementary Market Data**: Use OnixS for CBOE market data, TWS for execution
  - **Direct CBOE Execution**: Use OnixS for both market data and execution
  - **Multi-Venue Strategy**: Use OnixS for CBOE/CME, TWS for other venues

- **Development Support**:
  - **Source Code Samples**: Fast-start reference implementations included
  - **Market Data Logging**: Log and replay for backtesting strategies
  - **Complete Data Model**: Full access to venue's data model
  - **Technical Support**: Available from OnixS
  - **Source Code Escrow**: Available for enterprise customers

- **Use Cases**:
  - Direct CBOE box spread execution via CFE BOE
  - High-frequency box spread scanning with CFE Multicast PITCH
  - Multi-venue arbitrage (CBOE options + CME futures)
  - Interest rate futures hedging with CME iLink 3

- **Contact**: <sales@onixs.biz> (sales), <support@onixs.biz> (technical support)
- **Phone**: +44 20 7117 0111 (UK), +1 312 999 6040 (US)
- **Documentation**: See `docs/ONIXS_DIRECTCONNECT.md` for comprehensive SDK details, CBOE integration, and use cases
- **Note**: OnixS directConnect provides ultra-low latency SDKs for direct CBOE access, particularly valuable for SPX/SPXW box spread trading.
  The C++ SDK aligns with the project's technology stack. Contact OnixS for pricing, 30-day evaluation, and integration support.
  Consider as alternative to broker APIs for direct CBOE access and ultra-low latency execution.

### OnixS FIX Protocol Dictionary and Tools

- **Website**: <https://www.onixs.biz/fix-protocol-dictionary-tools.html>
- **Provider**: OnixS - Financial technology solutions
- **Description**: Comprehensive FIX Protocol reference tools and utilities for FIX connectivity infrastructure development and maintenance
- **Key Tools**:
  - **FIX Dictionary**: Online reference for FIX protocol standards in tag/value and enumeration formats
  - **FIX Analyser**: High-performance FIX log file analysis tool with queries, validation, and monitoring
  - **SBE Encoder/Decoder**: Ultra-low latency Simple Binary Encoding implementations (C++, Java, .NET)
  - **FIX FAST Encoder/Decoder**: FIX Adapted for Streaming (FAST) 1.1/1.2 implementations (C#, C++, Java)
  - **FIX Protocol Overview**: Educational content about FIX standards, session layer, application layer, FIXT
  - **FIXP Overview**: FIX Performance Session Layer - lightweight protocol for session management

- **FIX Dictionary**:
  - **Purpose**: Deep reference to FIX protocol standards syntax
  - **Format**: Tag/value and enumeration formats
  - **Use Case**: Look up FIX field definitions, message structure, tag numbers
  - **Access**: <https://www.onixs.biz/fix-dictionary.html>

- **FIX Analyser**:
  - **Purpose**: Analyze FIX messaging interactions in human-readable format
  - **Features**: Log file analysis, queries, validation, monitoring, protocol compliance
  - **Use Case**: Debug FIX exchanges, validate compliance, troubleshoot connectivity
  - **Benefits**: Save time and costs in FIX development and support
  - **Access**: <https://www.onixs.biz/fix-analyser.html>

- **SBE Encoder/Decoder**:
  - **Purpose**: Ultra-low latency Simple Binary Encoding implementations
  - **Languages**: C++, Java, .NET
  - **Use Case**: High-performance FIX applications, SBE encoding/decoding, CME SBE integration
  - **Relevance**: Used by exchanges (CME, CBOE) for ultra-low latency messaging
  - **Access**: <https://www.onixs.biz/sbe-codec.html>

- **FIX FAST Encoder/Decoder**:
  - **Purpose**: FIX FAST protocol for high-performance streaming
  - **Standards**: FAST 1.1/1.2 support
  - **Languages**: C#, C++, Java (included in FIX Engine SDKs)
  - **Use Case**: Streaming market data feeds, efficient message compression
  - **Relevance**: Used for streaming options chain data (CBOE Multicast PITCH)

- **FIX Protocol Overview**:
  - **Purpose**: Educational content about FIX Protocol standards
  - **Topics**: FIX standards, session layer, application layer, FIXT, FIX dialects
  - **Use Case**: Learn FIX fundamentals, understand architecture, session management
  - **Access**: <https://www.onixs.biz/fix-protocol-overview.html>

- **FIXP (FIX Performance Session Layer)**:
  - **Purpose**: Lightweight protocol for session management
  - **Features**: Minimal overhead, performance-focused, endpoint communication
  - **Use Case**: High-performance FIX sessions, low-latency connections
  - **Access**: <https://www.onixs.biz/fixp-overview.html>

- **Relevance to Box Spread Trading**:
  - **FIX Development**: Essential for direct CBOE/CME FIX integration
  - **Debugging**: FIX Analyser critical for troubleshooting connectivity
  - **High Performance**: SBE/FAST for ultra-low latency execution
  - **Education**: FIX Protocol Overview for learning FIX fundamentals

- **Integration with OnixS directConnect**:
  - **FIX Engine SDKs**: Include FAST support, integrate with SBE Codec
  - **Direct Market Access**: Use SBE for CME, FAST for market data, Dictionary for reference
  - **Workflow**: Dictionary → FIX Engine → Analyser → Optimize with SBE/FAST

- **Use Cases**:
  - Direct CBOE FIX integration for box spread execution
  - CME SBE integration for futures hedging
  - Market data streaming via FAST protocol
  - Debugging FIX connectivity issues

- **Comparison with Other Tools**:
  - **vs. FIXimate**: OnixS Dictionary for standards reference, FIXimate for interactive exploration
  - **vs. FIX Trading Community**: OnixS for implementation tools/SDKs, FIX TC for standards
  - **Recommendation**: Use both - FIXimate for exploration, OnixS for implementation and debugging

- **Contact**: <sales@onixs.biz> (sales), <support@onixs.biz> (technical support)
- **Phone**: +44 20 7117 0111 (UK), +1 312 999 6040 (US)
- **Evaluation**: Free 30-day evaluation available for SDKs and tools
- **Documentation**: See `docs/ONIXS_FIX_DICTIONARY_TOOLS.md` for comprehensive tool descriptions, use cases, and integration workflows
- **Note**: OnixS FIX Protocol tools are essential for FIX protocol development, debugging, and optimization.
  Use FIX Dictionary for reference, FIX Analyser for debugging, and SBE/FAST codecs for high-performance implementations.
  Particularly valuable when implementing direct exchange access (CBOE, CME) for box spread trading.

### Brokeree Solutions

- **Website**: <https://brokeree.com/solutions/>
- **Provider**: Brokeree Solutions - Technology provider for retail brokers
- **Description**: Turnkey technology solutions for retail forex and CFD brokers, primarily focused on MetaTrader 4/5 (MT4/MT5) and cTrader platforms
- **Primary Focus**:
  Retail forex/CFD broker infrastructure (liquidity bridges, PAMM systems, social trading, prop trading solutions, MetaTrader plugins)
- **Key Solutions**:
  - **Liquidity Bridge**: Multi-server liquidity aggregation and risk management
  - **TradingView API**: Connect TradingView to trading platforms
  - **MT4/MT5 FIX API**: FIX protocol integration for MetaTrader
  - **MT4/MT5 REST API**: REST API for MetaTrader platforms
  - **MT5 Gateways**: Gateways to various liquidity providers (DASTrader, Exante, LMAX, SAXO Bank, AC Markets)

- **Potentially Relevant Components**:
  - **MT5 Gateway to DASTrader**:
    Direct Access Software for US exchanges (CBOE/BATS/EDGE, CBSX, Nasdaq, AMEX/NYSE/ARCA, OTC) - provides CBOE access but requires MetaTrader 5
  - **MT5 Gateway to Exante**: EXANTE multi-asset services including options (10,000+ instruments) - requires MetaTrader 5
  - **FIX API**: FIX protocol support - but MetaTrader-specific, not direct C++ integration

- **Relevance to Box Spread Trading**:
  - **Limited Direct Relevance**: Designed for retail forex/CFD brokers using MetaTrader, not institutional C++ options trading
  - **Architectural Mismatch**: MetaTrader-based vs. C++ native system
  - **Platform Dependency**: Requires MetaTrader 4/5 platform, not suitable for direct C++ integration
  - **Asset Class Focus**: Primarily forex/CFD, not options-focused

- **Comparison with Project**:
  - **Current Project**: C++ native, TWS API, direct CBOE options trading
  - **Brokeree**: MetaTrader-based, retail broker infrastructure, forex/CFD focus
  - **Conclusion**: Significant architectural mismatch, not directly applicable

- **Better Alternatives for Box Spread Trading**:
  - **OnixS directConnect**: C++ SDKs for direct CBOE access (recommended)
  - **TFB FIX API**: FIX platform for direct exchange access (recommended)
  - **TWS API**: Current solution - comprehensive options support
  - **Alpaca API**: Alternative broker API with options support

- **Recommendation**: **Not recommended** for this project - use OnixS directConnect, TFB FIX API, or direct FIX implementation instead
- **Contact**: <[email protected]>
- **Phone**: +372 602 71 22 (Estonia), +357 25 011886 (Cyprus)
- **Documentation**: See `docs/BROKEREE_SOLUTIONS.md` for detailed analysis and comparison with project requirements
- **Note**: Brokeree Solutions is primarily for retail forex/CFD brokers using MetaTrader platforms.
  While some components (DASTrader gateway) provide CBOE access, the platform is not well-suited for institutional C++ options trading systems.
  For box spread trading, OnixS directConnect, TFB FIX API, or direct FIX implementation would be more appropriate alternatives.

## Brokerage API Resources

<!--
@index: api-documentation
@category: brokerage-resources
@tags: brokerage, broker-api, trading-platforms
@last-updated: 2025-01-27
-->

### QuantPedia Brokerage APIs List

- **Website**: <https://quantpedia.com/links-tools/?category=brokerage-api>
- **Provider**: QuantPedia - Encyclopedia of Quantitative Trading Strategies
- **Description**: Comprehensive curated list of brokerage APIs and tools for quantitative traders
- **Listed Brokers**:
  - **Alpaca Markets**: ✅ Already documented - API-first commission-free broker, options support, Elite features
  - **Interactive Brokers**: ✅ Already integrated - TWS API, comprehensive options support, global markets
  - **TD Ameritrade**: ⚠️ Limited - US-focused, options support, but acquired by Charles Schwab (verify status)
  - **Drive Wealth**: ❌ Enterprise-only - API-driven brokerage for enterprise clients, no options
  - **Xignite**: ⚠️ Market data only - Cloud-native market data solutions, no trading
  - **IG UK**: ❌ Not relevant - Spread betting/CFD provider, UK-focused
  - **Lightspeed Trader API**: ✅ Potentially relevant - C++ API (matches project stack!), high performance (1,500 orders/sec), options support
  - **E\*TRADE**: ⚠️ Limited - Retail-focused, options support, limited API
  - **Ally**: ❌ Not relevant - Self-directed trading, retail-focused
  - **Lime Brokerage**: ✅ Potentially relevant - FIX/API access to US Equity and Options markets, performance-focused
  - **Infront**: ⚠️ Market data only - Excel/desktop APIs for market data and analysis

- **Highly Relevant for Box Spread Trading**:
  - **Lightspeed Trader API**: C++ native API, high performance (1,500 orders/sec), low latency, co-location available, options support,
    no additional market data fees for API use
  - **Lime Brokerage**: FIX/API access to US Options markets, performance-focused, comprehensive market data

- **Relevance Assessment**:
  - **Already Integrated**: IBKR (TWS API) - primary broker
  - **Already Documented**: Alpaca Markets - secondary broker option
  - **New Opportunities**: Lightspeed (C++ API) and Lime (FIX/API) - worth evaluating
  - **Technology Match**: Lightspeed C++ API matches project technology stack perfectly

- **Lightspeed Trader API Details**:
  - **C++ Libraries**: DLL integration for C++ programmers
  - **Performance**: Up to 1,500 orders per second per ID
  - **Latency**: Minimized latency, co-location options available
  - **Features**: List order entry, order management, real-time Level II quotes, risk management
  - **Pricing**: Minimum $25/month commission for accounts under $15,000
  - **Use Case**: Alternative C++ API for high-performance options trading

- **Lime Brokerage Details**:
  - **Interface Options**: APIs, FIX, or trading applications
  - **Market Access**: All U.S. Equity and Options markets
  - **Performance**: Innovative solutions for real-time performance challenges
  - **Infrastructure**: Lime Network Operations for automated trading
  - **Use Case**: Alternative broker with FIX/API access for US options

- **Comparison Summary**:
  - **Options Support**: IBKR ✅, Alpaca ✅, Lightspeed ✅, Lime ✅
  - **C++ API**: Lightspeed ✅ (perfect match), others require language adapters
  - **Performance**: Lightspeed (1,500/sec), Lime (high), IBKR (good), Alpaca (good)
  - **Cost**: Alpaca (free), IBKR ($1/stock), Lightspeed ($25/mo min), Lime (contact)

- **Recommendations**:
  - **Primary**: IBKR (already integrated)
  - **Secondary**: Alpaca (already documented)
  - **Evaluate**: Lightspeed (C++ API) and Lime (FIX/API) as alternatives

- **Action Items**:
  - Evaluate Lightspeed Trader API for C++ integration
  - Contact Lime Brokerage for pricing and capabilities
  - Verify TD Ameritrade API status after Charles Schwab acquisition

- **Documentation**: See `docs/QUANTPEDIA_BROKERAGE_APIS.md` for comprehensive broker analysis, comparison, and recommendations
- **Note**: QuantPedia provides a valuable resource for discovering brokerage APIs.
  The project already uses IBKR (integrated) and has Alpaca documented.
Lightspeed Trader API (C++ native) and Lime Brokerage (FIX/API) are potentially valuable alternatives worth evaluating for high-performance options
trading.
  Always verify current API status, pricing, and capabilities before integration.

### QuantPedia Subscription - Trading Strategy Research Platform

- **Website**: <https://quantpedia.com/pricing/>
- **Provider**: QuantPedia - Encyclopedia of Quantitative Trading Strategies
- **Description**: Subscription-based research platform providing access to quantitative trading strategies, academic research papers, backtests,
  and portfolio analysis tools
- **Subscription Tiers**:
  - **Prime**: 100+ essential strategies, essential portfolio modeling, no research papers, no reports (entry-level)
  - **Premium**:
    900+ full strategies, essential portfolio modeling, 1000s of research papers, 800+ backtests, regular updates (recommended for research)
  - **Pro**: 900+ full strategies, full portfolio modeling, 1000s of research papers, 800+ backtests, 30+ Quant Reports, AI Chatbot,
    regular updates (best value for professionals)

- **Key Features**:
  - **900+ Premium Strategies**: Short description, performance & risk characteristics, links to academic papers
  - **2000+ Research Papers**: Academic papers related to strategies (Premium/Pro)
  - **800+ Backtests**: Out-of-sample backtests with equity curves, statistics, complete Python code (Premium/Pro)
  - **Portfolio Analysis**: 400+ charts and tables in thematically focused reports
  - **30+ Quant Reports** (Pro only): Factor regression, risk scenarios, VaR, clustering, Monte Carlo, alternative weighting schemes
  - **AI Chatbot** (Pro only): Personal quant assistant trained on QuantPedia database

- **Update Frequency**:
  - **Strategies**: 10-15 new strategies added monthly (Premium/Pro)
  - **Backtests**: 5+ new backtests added bi-weekly (Premium/Pro)
  - **Reports**: New report types added periodically every month (Pro)

- **Relevance to Box Spread Trading**:
  - **Arbitrage Strategies**: ✅ High - Research arbitrage strategies similar to box spreads
  - **Options Strategies**: ✅ High - Options-based trading strategies and research
  - **Backtesting Examples**: ✅ High - 800+ Python code examples for backtesting methodologies
  - **Academic Research**: ✅ High - 2000+ academic papers on arbitrage and options strategies
  - **Portfolio Analysis**: ⚠️ Moderate - Advanced tools for portfolio performance analysis (Pro)

- **Use Cases**:
  - Research arbitrage and options strategies
  - Learn backtesting methodologies (Python code examples)
  - Access academic research papers on arbitrage
  - Analyze portfolio performance (Pro)
  - Develop new box spread variations

- **Pricing**:
  - **Prime**: See <https://quantpedia.com/prime-pricing>
  - **Premium**: See <https://quantpedia.com/premium-pricing>
  - **Pro**: See <https://quantpedia.com/pro-pricing>
  - **Enterprise**: Contact for team/company pricing

- **Payment Options**:
  - Credit/debit card, PayPal
  - Bank/wire transfer (contact for invoice)
  - Cryptocurrency (via PayPal or contact)

- **Subscription Details**:
  - **No Auto-Renewal**: Manual renewal required (subscriptions do NOT auto-renew)
  - **Upgrades**: Contact to upgrade (pay only difference)
  - **Account Retention**: Account remains after subscription ends but loses Premium/Pro access

- **Recommendations**:
  - **For Strategy Research**: Premium or Pro (full strategy access, research papers, backtests)
  - **For Academic Research**: Premium or Pro (2000+ research papers)
  - **For Portfolio Analysis**: Pro (advanced tools, 30+ reports, AI Chatbot)
  - **For Box Spread Trading**: Premium recommended (arbitrage strategies, research papers, backtest examples)

- **Documentation**: See `docs/QUANTPEDIA_SUBSCRIPTION.md` for comprehensive subscription details, feature comparison, and use cases
- **Note**: QuantPedia is a research and educational platform, not a trading execution platform.
  It provides strategy ideas, academic research, and backtesting examples that can inform box spread trading strategies.
  The arbitrage strategy category is particularly relevant. Premium tier recommended for strategy research; Pro tier for advanced portfolio analysis.

### eToro - Social Trading Platform

- **Website**: <https://www.etoro.com/discover>
- **Provider**: eToro - Social trading and investment platform
- **Description**:
  Social trading platform enabling users to invest in various assets and copy trades from experienced investors using CopyTrader™ technology
- **Key Features**:
  - **Social Trading**: CopyTrader™ technology to replicate trades in real-time
  - **Asset Classes**: Stocks, limited cryptocurrencies (U.S. restrictions), ETFs, CFDs
  - **Retail-Focused**: Designed for individual retail investors
  - **Community Features**: Social trading community, performance statistics, rankings

- **U.S. Market Restrictions**:
  - **Cryptocurrency Limitations**: U.S. customers can only trade Bitcoin, Bitcoin Cash, and Ethereum (SEC settlement, September 2024)
  - **Most Crypto Removed**: eToro ceased offering most cryptocurrency trading to U.S. customers

- **API and Integration**:
  - **Status**: ⚠️ Limited or no public API available
  - **Platform Type**: Web-based and mobile applications
  - **Not Designed For**: Algorithmic trading, institutional trading, API-driven systems

- **Relevance to Box Spread Trading**:
  - **Direct Relevance**: ❌ None - No options trading, no algorithmic trading API, retail social trading focus
  - **Asset Limitations**: Stocks, limited crypto, ETFs only - no options or futures
  - **Conclusion**: Not suitable for box spread trading

- **Comparison with Project Requirements**:
  - **Required**: Options trading, multi-leg orders, API, real-time market data, direct exchange access
  - **eToro Provides**: ❌ None of the above
  - **Conclusion**: Does not meet project requirements

- **Recommendation**: ❌ **Not Recommended** for box spread trading
- **Use Instead**: IBKR (TWS API), Alpaca Markets, Lightspeed Trader API, or Lime Brokerage
- **Documentation**: See `docs/ETORO_PLATFORM.md` for detailed analysis
- **Note**: eToro is a social trading platform for retail investors to copy trades.
  It does not offer options trading, algorithmic trading APIs, or capabilities required for box spread trading.
  For box spread strategies, use IBKR, Alpaca, or other options-capable brokers with API access.

## Market Structure & Efficiency References

<!--
@index: api-documentation
@category: market-structure
@tags: market-structure, box-spread, cboe, cme, research
@last-updated: 2025-01-27
-->

### CME Group – Capital Efficiencies and AIR TRFs

- **Whitepaper**: <https://www.cmegroup.com/articles/whitepapers/capital-efficiencies-and-air-trfs.html>
- **Focus**: Explains capital efficiency benefits of Alternative Index Replication (AIR) Total Return Futures.
- **Relevance**:
  Useful for comparing margin treatment and financing costs when evaluating box-spread arbitrage versus futures-based replication strategies.
- **Key Takeaways**:
  - AIR TRFs deliver equity index exposure with optimized capital usage.
  - Highlights clearing efficiencies and reduced balance-sheet impact relative to swaps.
  - Provides framework for assessing total-cost-of-carry trade structures.

### SyntheticFi – Box Spread-Based Securities-Backed Lending

- **Website**: <https://www.syntheticfi.com/>
- **Application**: <https://app.syntheticfi.com/cob>
- **YC Profile**: <https://www.ycombinator.com/companies/syntheticfi>
- **Description**: Y Combinator-backed fintech providing low-cost securities-backed lending using box spreads
- **Focus**: Leverages box spreads to provide loans at rates 1-3% lower than traditional lenders, with full tax-deductible interest expenses
- **Relevance**: Direct implementation example of box spread lending/borrowing strategy; informs our primary goal of synthetic lending and borrowing
- **Key Features**:
  - Loans starting at $10,000
  - Integration with existing brokerage accounts
  - No credit checks required (uses securities as collateral)
  - Interest expenses fully tax-deductible regardless of loan purpose
  - Access liquidity without selling investments or incurring capital gains taxes

- **Technical Approach**:
  - Uses SPX (S&P 500 Index) options for box spread construction
  - Four-leg structure: Long Call (K1), Short Call (K2), Long Put (K2), Short Put (K1)
  - Strike width (K2 - K1) represents loan principal
  - Net debit/credit represents implied interest rate
  - Typically 30-90 days to expiration for lending positions

- **Comparison with Our Implementation**:
  - **SyntheticFi**: Managed service (they execute trades for clients)
  - **Our Implementation**: Self-directed tool (you control execution and position management)
  - **Advantages of Self-Directed**: Full transparency, control over timing and rates, ability to improve positions intraday

- **Implementation Reference**: See `docs/SYNTHETICFI_LENDING_BORROWING_ANALYSIS.md` for comprehensive analysis
- **Key Takeaways**:
  - Box spreads can provide competitive implied interest rates vs T-bills and margin loans
  - OCC clearing eliminates counterparty risk
  - Intraday position monitoring can identify rate improvement opportunities
  - Self-directed approach provides transparency and control over position management

### Cboe – Box Spreads as Alternative Borrowing & Lending

- **Article**: <https://www.cboe.com/insights/posts/why-consider-box-spreads-as-an-alternative-borrowing-lending-strategy/>
- **Author**: Dr. Wesley R. Gray (Alpha Architect), October 16, 2024.
- **Focus**: Demonstrates how four-leg box spreads replicate risk-free borrowing/lending via put-call parity.
- **Relevance**: Aligns with our CLI's intent to capture financing edges by comparing box spread yields to Treasury bills.
- **Key Takeaways**:
  - Box spreads provide funding rates competitive with T-bills.
  - OCC clearing significantly mitigates counterparty risk.
  - Highlights practical use cases for retail and institutional investors seeking efficient cash management.

### OCC Options Education – Box Spread Strategies for Borrowing or Lending Cash

- **PDF**:
  <https://www.optionseducation.org/getmedia/2ae6c8bd-9a8e-4d2f-8168-19b6ff9e3589/listed-options-box-spread-strategies-for-borrowing-or-lending-cash.pdf>
- **Source**: Options Education (OCC Educational Foundation)
- **Focus**: Comprehensive guide to constructing and using box spreads as exchange-listed options strategies for borrowing or lending cash.
- **Relevance**: Educational foundation document explaining box spread mechanics, construction, and practical applications for financing.
- **Key Topics Covered**:
  - Box spread construction (bull call spread + bear put spread)
  - Synthetic long and synthetic short positions
  - Implied interest rate calculations
  - Competitive rates compared to market alternatives
  - OCC clearing benefits (counterparty risk mitigation)
  - Capital efficiency through portfolio margining
  - Tax treatment considerations (Section 1256)
  - Transaction costs, liquidity, and execution risks
  - Practical examples with SPX options

- **Key Takeaways**:
  - Box spreads can provide competitive implied interest rates aligned with market rates.
  - OCC guarantee reduces counterparty credit risk significantly.
  - Portfolio margining can lead to capital efficiencies.
  - Favorable tax treatment possible under Section 1256 (60% long-term, 40% short-term capital gains).
  - Exchange-listed options provide liquidity and transparency for entry/exit.
  - Transaction costs and execution timing are critical factors for profitability.

### Cboe – Quoted Spread Book (QSB) FAQ

- **Document**: <https://cdn.cboe.com/resources/membership/Quoted_Spread_Book_FAQ.pdf>
- **Published**: September 15, 2025
- **Focus**:
  Explains Cboe's Quoted Spread Book service that
  allows Market Makers to rest orders directly in Complex Order Books (COBs) for select spread instruments during Regular Trading Hours (RTH).
- **Key Features**:
  - **Box Spreads**: Box Spreads on first four serial, first three quarterly,
    and first three December standard SPX contracts at 4000 and 5000 strikes (~10 quotable instruments daily).
  - **Box Swaps**: Box Spread rolls from first four serial and first two quarterly expirations to forward contracts (~25 quotable instruments daily).
  - **Jelly Rolls**: Combo rolls from 0-DTE, 1-DTE, and nearest Friday SPXW,
    plus first three serial and first two quarterly SPX contracts (~120 quotable instruments daily).

- **Market Maker Access**:
  - Orders with `OrderCapacity=M` or `N` allowed to rest in QSB COBs during RTH.
  - No class appointments required for QSB instruments.
  - Bulk quoting interface supported via BOE Complex Quote Update messages.

- **Symbol Discovery**:
  - Reference data available via Cboe U.S. Options Reference Data webpage (JSON/HTML).
  - EDCID (Exchange Designated Complex Instrument Definition) messages on Complex PITCH and TOP feeds.
  - Complete list available by 7:00 a.m. ET each trading day.

- **Trading**:
  - Customers trade QSB instruments same as any other spread instrument.
  - No special fees or allocation differences (SPX: pro-rata, SPXW: price-time, cross: pro-rata).
  - No quoting obligations for Market Makers.

- **Relevance**:
  - Directly relevant to box spread trading strategies on Cboe.
  - Enables electronic trading of box spreads against lit Market Maker quotes during RTH.
  - Provides structured list of quotable instruments for automated discovery and monitoring.
  - Reference data feeds enable integration for real-time QSB instrument tracking.

- **Integration Opportunities**:
  - Parse QSB reference data (JSON) for available box spread instruments.
  - Subscribe to Complex PITCH/TOP feeds for EDCID messages.
  - Monitor QSB COBs for improved execution opportunities.
  - Request additional instruments via `cboelabs@cboe.com`.

- **Note**: Prior to QSB, Market Makers were not allowed to rest orders in SPX/SPXW Option COBs during RTH.
  QSB enables electronic trading that was previously predominantly open outcry.

### Cboe – Frequent Trader Program (FTID)

- **Document**: <https://cdn.cboe.com/resources/membership/us-options-frequent-trader-program.pdf>
- **Registration**: <https://www.cboe.com/FTID/registration.aspx>
- **Focus**: Fee rebate program for high-volume trading in RUT, VIX®, SPX, and SPXW options
- **Eligibility**: Non-Trading Permit Holder, non-broker/dealer customers (asset managers, hedge funds, individuals, etc.)
- **Key Features**:
  - Unique Frequent Trader ID (FTID) appended to orders
  - Volume aggregation across all executing agents
  - Rebates based on monthly contract volume tiers
  - Applies to both Regular Trading Hours (RTH) and Global Trading Hours (GTH)
  - FTID is private and not visible to counterparties

- **Rebate Schedule**:
  - **VIX Options**: 5% (10K-99K contracts), 15% (100K-299K), 25% (300K+)
  - **SPX/SPXW Options**: 3% (10K-49K), 6% (50K-99K), 9% (100K+)
  - **RUT Options**: 10% (10K-24K), 15% (25K-49K), 25% (50K+)

- **Tracking & Payment**:
  - Daily email statements with contracts traded and estimated rebates
  - Customized reports available at <https://www.cboe.org/tphreports/default.aspx>
  - Rebates paid to executing agent or directly to customer via EFT
  - Post-trade adjustments supported

- **Relevance**:
  - Directly applicable to box spread strategies using SPX/SPXW options
  - Volume thresholds achievable for active box spread traders
  - Rebates reduce transaction costs, improving net profitability
  - Privacy protection (FTID not visible to counterparties)

- **Integration Considerations**:
  - Store FTID securely (never commit to git)
  - Ensure FTID appended to all CBOE orders
  - Monitor monthly volume to track tier progression
  - Structure trading to maximize rebate tier qualification

- **Documentation**: See `docs/CBOE_FREQUENT_TRADER_PROGRAM.md` for comprehensive guide
- **Note**: Particularly valuable for high-volume box spread strategies where transaction cost reduction directly impacts profitability

### Cboe EDGX Equities Fee Schedule

- **Document**: <https://www.cboe.com/us/equities/membership/fee_schedule/edgx/>
- **Focus**: Comprehensive fee structure for CBOE EDGX Equities exchange, including market data, connectivity, and services
- **Key Components**:
  - **Cboe One Feed**: Market data feeds (Summary: $1,500-$5,000/month, Premium: $15,000-$12,500/month)
  - **OPRA**: Options market data ($4,500/month standalone, $6,390/month in bundle)
  - **Cboe Connect**: Market data connectivity ($250-$14,500/month per feed)
  - **Unicast Access**: Order entry bandwidth ($500-$3,500/month, free for CBOE exchanges)
  - **Edge Attribution Incentive**: Rebate program for liquidity providers
  - **Timestamping Service**: Order entry/cancellation reports ($1,000-$2,500/month)

- **Cost Optimization**:
  - Free BZX/BYX/EDGX/EDGA feeds
  - Bundle discounts (10-40% savings)
  - Small Retail Broker Program (reduced fees for ≥90% Non-Professional users)
  - Enterprise licenses for high-volume users

- **Relevance**:
  - Critical for calculating total transaction costs
  - Market data costs can be significant ($0.25-$100,000+/month)
  - OPRA feed essential for options trading
  - Free CBOE feeds reduce costs for CBOE-focused strategies
  - Combine with FTID rebates for cost optimization

- **Integration Considerations**:
  - Assess market data requirements based on trading strategy
  - Size connectivity bandwidth based on order volume
  - Leverage bundle discounts and free feeds
  - Track costs alongside FTID rebates

- **Documentation**: See `docs/CBOE_EDGX_FEE_SCHEDULE.md` for comprehensive fee breakdown and cost analysis
- **Note**: Essential reference for understanding total cost of trading, especially when combined with transaction fees and FTID rebates

### CME Group – Quantifying and Hedging Equity Financing Risk

- **Article**: <https://www.cmegroup.com/articles/2025/quantifying-and-hedging-equity-financing-risk.html>
- **Focus**: Examines equity financing costs, basis dynamics, and hedging techniques using listed derivatives.
- **Relevance**:
  Complements box-spread financing analysis by framing how equity financing risk and basis can be hedged or benchmarked against futures.
- **Key Takeaways**:
  - Provides methodologies to measure financing spreads between cash equities and futures.
  - Discusses hedging tools to manage equity repo and funding exposures.
  - Informs scenario analysis for arbitrage strategies sensitive to financing shifts.

### CME Group Client Systems Wiki (EPIC Sandbox)

- **Portal**: <https://cmegroupclientsite.atlassian.net/wiki/spaces/EPICSANDBOX/overview?homepageId=457314687>
- **Scope**: Documentation for CME client systems sandbox, including reference data, clearing, Globex connectivity, and post-trade services.
- **Access Notes**:
  - Some content requires authenticated CME client credentials; anonymous users may see restricted sections.
  - Useful for integration teams coordinating CME connectivity, testing flows, or referencing settlement schedules and API specs.

- **Relevance**: Helps align our automation with CME interface changes (e.g., market data, clearing,
  settlement timings) when evaluating financing trades that interact with futures infrastructure.

### CME Group Fee Schedules and Rebate Products

- **Products Page**: <https://www.cmegroup.com/markets/products.html>
- **Focus**: Comprehensive fee structures, rebate programs, and cost optimization for CME Group exchanges (CME, CBOT, NYMEX, COMEX)
- **Product Categories**:
  - Equity Indexes: E-mini S&P 500 (ES), E-mini NASDAQ-100 (NQ), E-mini Dow (YM), E-mini Russell 2000 (RTY)
  - Interest Rates: Treasury futures, Eurodollar futures
  - Energy: Crude oil, Natural gas, Heating oil, Gasoline
  - Metals: Gold, Silver, Copper, Platinum
  - Agriculture: Corn, Soybeans, Wheat, Live cattle
  - Foreign Exchange: Euro, Japanese Yen, British Pound, Australian Dollar

- **Fee Components**:
  - Exchange fees: Per-contract fees for executing trades
  - Clearing fees: Fees for clearing and settlement
  - Market data fees: Real-time and historical data access
  - Connectivity fees: Network access and co-location
  - Regulatory fees: NFA, CFTC, and other regulatory assessments

- **Rebate Programs**:
  - Volume-based rebates: Tiered rebates based on monthly contract volume
  - Product-specific rebates: Different rates for different product categories
  - Maker-taker model: Different fees/rebates for liquidity providers vs. takers
  - Liquidity provider incentives: Market maker, EMM, and DMM programs
  - New product incentives: Launch rebates and time-limited programs

- **Market Data**:
  - CME Globex Market Data: Real-time prices, depth, and trade data
  - Level 1/2/3 data: Top-of-book, market depth, full order book
  - Historical data: DataMine historical data access
  - Professional vs. Non-Professional pricing tiers

- **Connectivity**:
  - Co-location: Physical proximity to CME data centers
  - Direct Market Access (DMA): Direct connection to Globex
  - FIX Protocol and CME iLink connectivity

- **Cost Optimization**:
  - Volume-based optimization: Aggregate volume across CME Group exchanges
  - Liquidity provider programs: Market maker status for enhanced rebates
  - Market data optimization: Enterprise licenses and selective data access
  - Timing optimization: Leverage time-limited rebate programs

- **Relevance**:
  - Hedging: Using equity index futures to hedge box spread positions
  - Arbitrage: Comparing box spread yields to futures financing rates
  - Capital efficiency: Using futures for capital-efficient exposure
  - Basis trading: Trading basis between options and futures

- **Integration Considerations**:
  - Total cost analysis: Include CME fees when calculating total strategy costs
  - Rebate optimization: Structure CME trading to maximize rebates
  - Market data costs: Factor in CME market data fees separately
  - Connectivity: Consider co-location for low-latency arbitrage

- **Documentation**: See `docs/CME_FEE_SCHEDULE_REBATES.md` for comprehensive fee breakdown, rebate programs, and cost analysis
- **Note**: Fee schedules are product-specific and updated periodically.
  Always verify current fees and rebates with CME Group before making trading decisions.

## Risk Management & Hedging

<!--
@index: api-documentation
@category: risk-management
@tags: risk-management, hedging, currency-exchange
@last-updated: 2025-01-27
-->

### Currency Exchange Risk in Box Spread Trading

- **Documentation**: `docs/CURRENCY_EXCHANGE_RISK.md`
- **Focus**: Managing currency exchange risk when trading USD-denominated box spreads with non-USD account currencies (e.g., USD/ILS)
- **Key Concepts**:
  - **Currency Risk**: Exchange rate fluctuations can significantly impact returns for non-USD accounts
  - **Exposure Calculation**: Position notional × exchange rate
  - **Currency Delta**: Sensitivity of position value to exchange rate changes
  - **Currency VaR**: Potential loss from currency moves at given confidence level

- **Risk Components**:
  - **Trade Entry Risk**: Converting account currency to USD for trade entry
  - **Position Holding Risk**: Currency fluctuations during position life
  - **Trade Exit Risk**: Converting USD proceeds back to account currency

- **Hedging Strategies**:
  - **Currency Futures**: Short currency futures to hedge long USD exposure (exchange-traded, liquid)
  - **Currency Forwards**: Forward contracts to lock exchange rate (exact hedge, no basis risk)
  - **Currency Options**: Asymmetric hedging with limited downside (premium cost)
  - **Natural Hedging**: Offset currency exposure with other positions (no explicit cost)

- **Implementation**:
  - **Code Location**: `native/include/hedge_manager.h`, `native/src/hedge_manager.cpp`
  - **Key Structure**: `CurrencyHedge` struct with exposure calculation and hedge cost
  - **Configuration**: `HedgeStrategy` with currency hedging flags and ratios
  - **Exchange Rate Data**: Currently stub implementation; integrate with TWS API for real-time rates

- **Real-Time Integration**:
  - **TWS API**: Request currency market data via `reqMktData()` with `secType="CASH"`
  - **Available Pairs**: USD/ILS, USD/EUR, USD/GBP, USD/JPY, and many more
  - **Alternative Sources**: OANDA API, Alpha Vantage, Finnhub
  - **⚠️ FXCM Restriction**: FXCM does not allow residents of Israel (not available for Israeli traders)

- **Risk Management**:
  - **Currency Risk Limits**: Maximum exposure per position, total exposure, per currency pair
  - **Currency Risk Monitoring**: Real-time metrics (exposure, delta, VaR, P&L)
  - **Hedge Effectiveness**: Track hedge performance and rebalance if needed

- **Cost-Benefit Analysis**:
  - **Futures**: Margin requirement (2-5%), roll cost (0.05-0.1%), bid-ask spread (0.01-0.02%)
  - **Forwards**: Spread (0.1-0.3%), may require credit line
  - **Options**: Premium (1-3%), time decay
  - **When to Hedge**: Exposure > 10% of account, volatility > 10%, duration > 30 days

- **Relevance**:
  - **Critical for International Traders**: Non-USD accounts face currency risk on USD-denominated positions
  - **Impact on Returns**: Currency moves can turn profitable USD trades into losses (or vice versa)
  - **Example**: 4.1% USD/ILS move can turn 5.0% USD return into either 9.4% or 0.6% in ILS terms
  - **Box Spread Specific**: Box spreads are USD-denominated; currency hedging essential for non-USD accounts

- **Best Practices**:
  - Assess currency risk before trade entry
  - Monitor currency exposure and hedge effectiveness
  - Select appropriate hedge instrument based on position duration
  - Set currency risk limits
  - Analyze currency impact on returns after trade

- **Related Documentation**:
  - **Hedging Implementation**: `docs/COMMISSIONS_AND_HEDGING_IMPLEMENTATION.md` - Currency and interest rate hedging
  - **CME Fee Schedules**: `docs/CME_FEE_SCHEDULE_REBATES.md` - Currency futures trading costs
  - **Risk Calculator**: `native/include/risk_calculator.h` - Risk management framework

- **Note**: Currency exchange rates are highly volatile and can significantly impact returns.
  Always assess currency risk before entering positions and consider hedging when exposure is significant.
  The codebase includes currency hedging infrastructure; integrate real-time exchange rate data for production use.

## Financial Tools & Calculators

### ECN Execution Financial Tools

- **Website**: <https://ecnexecution.com/category/financial-tools/>
- **Description**: Comprehensive collection of free web-based financial calculators and trading tools
- **Key Tools**:
  - **Options Profit Calculator (Pro)**: Advanced options profit/loss calculation for complex strategies
  - **Options Profit Calculator**: Standard options profit/loss calculator
  - **Position Size Calculator**: Calculate optimal position size based on risk parameters
  - **Futures Contract Calculator**: Profit/loss calculation for futures contracts
  - **Win Rate & Breakeven Calculator**: Strategy performance analysis
  - **Swap Rate Calculator**: Calculate swap rates for forex positions
  - **Comprehensive Profit Margins Calculator**: Multi-scenario profitability analysis
  - **Lot Size Calculator**: Position sizing for forex and CFD trading

- **Relevance to Box Spread Trading**:
  - **Options Calculators**: Essential for box spread profit/loss analysis and risk assessment
  - **Position Sizing**: Critical for determining optimal box spread position size
  - **Futures Calculator**: Useful for analyzing interest rate futures hedges (SOFR, Eurodollar)
  - **Swap Calculator**: Assess currency hedge costs for non-USD accounts
  - **Win Rate Calculator**: Evaluate box spread strategy effectiveness

- **Usage**:
  - **Initial Analysis**: Quick calculations before detailed analysis
  - **Validation**: Cross-check built-in system calculations
  - **Education**: Learn about options strategies and calculations
  - **Screening**: Initial opportunity screening
  - **Scenario Analysis**: Multi-scenario profitability analysis

- **Limitations**:
  - Web-based only (no API access)
  - Manual data entry required
  - No integration with trading systems
  - No real-time data integration

- **Comparison with Built-In Tools**:
  - **ECN Tools**: Web-based, accessible, free, user-friendly
  - **Built-In Tools**: Integrated, automated, real-time, customizable
  - **Recommendation**: Use ECN tools for validation and education; use built-in tools for production trading

- **Additional Resources**:
  - ECN broker reviews and comparisons
  - Trading guides and educational content
  - Trading platform reviews (MT4, MT5, TradingView, cTrader)

- **Documentation**: See `docs/ECN_EXECUTION_FINANCIAL_TOOLS.md` for comprehensive tool descriptions and usage examples
- **Note**: Free web-based tools for educational and analytical purposes. Always verify calculations with your own analysis.
  Use built-in project tools for production trading decisions.

## Broker Selection & Regulatory

### Broker Selection for Algorithmic Trading in Israel

- **Source**: <https://brokerchooser.com/best-brokers/best-brokers-for-algo-trading-in-israel>
- **Focus**: Selecting brokers suitable for algorithmic box spread trading from Israel
- **Regulatory**: Israel Securities Authority (ISA) compliance requirements
- **Key Selection Criteria**:
  - **API Requirements**: Multi-leg options support, real-time market data, low latency, reliability
  - **Options Trading**: SPX/SPXW access, multi-leg orders, portfolio margining
  - **Trading Costs**: Commission per contract, exchange fees, market data costs, currency conversion
  - **Platform Compatibility**: MT4/MT5, cTrader, TradingView, custom APIs
  - **Currency Support**: USD/ILS considerations, currency hedging options

- **Top Broker Options**:
  - **Interactive Brokers (IBKR)**: ✅ Currently integrated, comprehensive API, excellent options access, portfolio margining, currency hedging
  - **Alpaca Markets**: ✅ Documented, account available, commission-free options (API), modern REST API, Elite features (DMA, VWAP/TWAP)
  - **OANDA**: Forex-focused, useful for currency hedging, not suitable for options
  - **TradeStation Global**: historical comparison reference only; not currently supported in this repo
  - **MEXEM**: Global markets, competitive fees, API infrastructure

- **Broker Comparison**:
  - Feature comparison table (options trading, API quality, costs)
  - Cost analysis for high-volume box spread trading
  - Regulatory compliance status

- **Recommended Strategy**:
  - **Primary**: IBKR (already integrated, comprehensive options access)
  - **Secondary**: Alpaca (commission-free, modern REST API, Elite features)
  - **Complementary**: OANDA (currency hedging if needed)

- **Currency Considerations**:
  - USD/ILS exchange rate risk management
  - Multi-currency account support
  - Currency hedging options
  - See `docs/CURRENCY_EXCHANGE_RISK.md` for detailed currency risk management

- **Integration Considerations**:
  - Multi-broker architecture (redundancy, cost optimization, arbitrage)
  - API integration patterns (TWS API, REST API, FIX Protocol)
  - Current project: IBKR primary, Alpaca secondary

- **Best Practices**:
  - Verify ISA regulatory compliance
  - Test APIs with paper trading
  - Compare costs (commissions, fees, market data, rebates)
  - Evaluate platform reliability and support
  - Manage currency risk (USD/ILS)

- **Documentation**: See `docs/BROKER_SELECTION_ISRAEL.md` for comprehensive broker comparison, selection criteria, and integration considerations
- **Note**: Broker selection is critical for algorithmic trading success.
  The project uses IBKR as primary broker (already integrated) with Alpaca as documented alternative.
  Always verify regulatory compliance and test APIs thoroughly before live trading.

## Rust (Agents)

### Rust Standard Library

- **Official Docs**: <https://doc.rust-lang.org/std/>
- **Location**: `agents/backend/`, `agents/backend-mock/`, etc.
- **Cargo**: `Cargo.toml` files in agent directories

## C++ TUI (FTXUI)

### Go Standard Library

- **Official Docs**: <https://pkg.go.dev/std>
- **Location**: `tui/`
- **Modules**: `go.mod`, `go.sum`

## TypeScript/JavaScript (Web)

### TypeScript

- **Official Docs**: <https://www.typescriptlang.org/docs/>
- **Location**: `web/`
- **Config**: `tsconfig.json`

### Vite

- **Official Docs**: <https://vitejs.dev/>
- **Config**: `vite.config.ts`

## Swift (Desktop/iOS)

### Swift Package Manager

- **Official Docs**: <https://www.swift.org/package-manager/>
- **Location**: historical references only; Apple clients were removed from the active repo surface
- **Config**: `Package.swift`

## How to Use This Index in Cursor

### Method 1: Reference in Prompts

When asking Cursor about API usage, reference this file:

```text
@docs API_DOCUMENTATION_INDEX.md How do I use spdlog for error logging?
```

### Method 2: Add to Code Comments

Add references in your code:

```cpp
// @docs API_DOCUMENTATION_INDEX.md - TWS API EWrapper implementation
class MyTWSClient : public EWrapper {
  // ...
};
```

### Method 3: Update .cursorrules

The `.cursorrules` file already references this documentation structure.

## Keeping This Index Updated

When adding new dependencies:

1. Add entry to this file with:
   - Official documentation URL
   - Version used
   - Key classes/functions
   - Location in codebase
2. Update version numbers when upgrading
3. Add usage examples for complex APIs

## Data Science & Modeling Tools

### Modeling Tools Overview

- **Source**: [Domino Data Lab Blog](https://domino.ai/blog/8-modeling-tools-to-build-complex-algorithms)
- **Documentation**: See `docs/MODELING_TOOLS_OVERVIEW.md` for comprehensive overview
- **Purpose**:
  Reference guide for machine learning and deep learning tools that could be used for strategy development, backtesting, and predictive modeling
- **Key Tools Covered**:
  - **Deep Learning**: PyTorch, TensorFlow, Keras, Ray, Horovod
  - **Machine Learning**: Scikit-Learn, XGBoost, Apache Spark

- **Use Cases**:
  - Strategy optimization and backtesting
  - Predictive modeling for market conditions
  - Pattern recognition in options data
  - Risk management models

- **Note**: While not currently integrated, these tools could be valuable for future ML-enhanced box spread detection, volatility forecasting,
  or execution optimization.
  Consider resource requirements (especially GPU for deep learning) and production stability when evaluating tools for trading applications.

### XGBoost (eXtreme Gradient Boosting)

- **Official Docs**: <https://xgboost.readthedocs.io/>
- **GitHub**: <https://github.com/dmlc/xgboost>
- **Website**: <https://xgboost.ai/>
- **License**: Apache-2.0
- **Version**: Latest (actively maintained, 27.6k+ stars)
- **Language**: Primarily C++ (43.5%), with Python (20.9%), CUDA (17.9%), R, Java, Scala support
- **Deep Research**: See `docs/XGBOOST_DEEP_RESEARCH.md` for comprehensive guide
- **Key Features**:
  - **High Performance**: One of the fastest gradient boosting implementations
  - **C++ Native**: Direct integration with C++ trading systems (no Python overhead)
  - **Regularization**: Built-in L1/L2 regularization to prevent overfitting
  - **Missing Values**: Automatic handling of missing data
  - **Feature Importance**: Built-in feature importance metrics
  - **SHAP Integration**: Model interpretability for regulatory compliance
  - **Distributed Training**: Support for Spark, Dask, Kubernetes
  - **Low Latency**: Sub-millisecond inference suitable for real-time trading

- **Trading Use Cases**:
  - **Opportunity Detection**: Predict profitable box spread opportunities
  - **Risk Assessment**: Predict execution risk or early assignment probability
  - **Execution Timing**: Determine optimal timing for order execution
  - **Position Sizing**: Adaptive position sizing based on market conditions
  - **Market Regime Detection**: Identify favorable market conditions

- **C++ Integration**:
  - **CMake Support**: Native CMake integration via FetchContent
  - **API**: C API (`xgboost/c_api.h`) for direct C++ usage
  - **Performance**: Direct C++ usage avoids Python overhead for production inference
  - **Model Loading**: Save models from Python, load in C++ for deployment

- **Recommended Approach**:
  1. **Phase 1**: Prototype in Python for rapid development
  2. **Phase 2**: Export trained models and integrate into C++ codebase
  3. **Phase 3**: Continuous learning with periodic retraining

- **Key Parameters**:
  - `max_depth`: Tree depth (3-10 typical)
  - `learning_rate` (eta): Step size (0.01-0.3)
  - `n_estimators`: Number of trees (100-1000+)
  - `subsample`: Row sampling (0.6-1.0)
  - `colsample_bytree`: Column sampling (0.6-1.0)
  - `lambda` (L2): L2 regularization
  - `alpha` (L1): L1 regularization

- **Hyperparameter Tuning**: Integration with Optuna, Hyperopt, Ray Tune
- **Production Considerations**:
  - Model versioning and A/B testing
  - Regular retraining to adapt to market changes
  - Monitoring model performance in production
  - SHAP values for regulatory compliance

- **Installation**:
  - **Python**: `pip install xgboost`
  - **C++**: Build from source or use CMake FetchContent

- **Note**: XGBoost is widely used in financial services for credit scoring, fraud detection, and risk assessment.
  Its C++ implementation makes it ideal for low-latency trading applications.
  The recommended workflow is to train models in Python for rapid iteration, then deploy trained models in C++ for production inference.

### Ollama (Local LLM Platform)

- **Official Website**: <https://ollama.ai/>
- **GitHub**: <https://github.com/ollama/ollama>
- **License**: MIT
- **Version**: Latest (actively maintained, 100k+ stars)
- **Description**: Open-source platform for running large language models (LLMs) locally on your machine.
  Provides both CLI and GUI interfaces for interacting with local AI models.
- **Key Features**:
  - **Local Execution**: Run LLMs entirely on your machine (no API calls, no data sent to cloud)
  - **Privacy**: All data stays local, important for proprietary trading strategies
  - **Cost-Effective**: No per-token API costs after initial setup
  - **Multiple Models**: Support for various models including:
    - OpenAI GPT series (via compatible models)
    - DeepSeek-R1
    - Gemma 3
    - Llama, Mistral, CodeLlama, and many others
  - **Cross-Platform**: macOS, Windows, and Linux support
  - **GUI Application**: Windows 11 GUI app available (no terminal required)
  - **REST API**: HTTP API for programmatic access
  - **Model Management**: Easy model pulling, removal, and customization

- **Trading Use Cases**:
  - **Code Analysis**: Analyze trading code for bugs, security issues, or optimization opportunities
  - **Documentation Generation**: Generate documentation from code comments and structure
  - **Strategy Research**: Research trading strategies and market analysis (with local data)
  - **Code Review**: Automated code review for trading logic
  - **Error Analysis**: Understand and debug complex trading system errors
  - **Learning Tool**: Learn about options trading, box spreads, and market mechanics

- **Integration Options**:
  - **CLI**: Command-line interface for direct interaction
  - **REST API**: HTTP API for programmatic integration
  - **Python**: Python bindings available
  - **MCP Server**: Potential MCP server integration for Cursor IDE

- **Installation**:
  - **macOS**: `brew install ollama` or download from website
  - **Linux**: `curl -fsSL https://ollama.ai/install.sh | sh`
  - **Windows**: Download installer from website or use Windows 11 GUI app

- **Usage Example**:

  ```bash
  # Pull a model
  ollama pull llama3

  # Run a model
  ollama run llama3 "Explain box spread arbitrage"

  # Use REST API
  curl http://localhost:11434/api/generate -d '{
    "model": "llama3",
    "prompt": "What are the risks of box spread trading?"
  }'
  ```

- **Relevance to Box Spread Trading**:
  - **Privacy**: Keep proprietary trading strategies and code analysis private
  - **Cost Savings**: No API costs for frequent code analysis or documentation tasks
  - **Offline Capability**: Work with AI assistance even without internet
  - **Custom Models**: Fine-tune models on trading-specific data if needed
  - **Code Quality**: Use for automated code review and documentation

- **Considerations**:
  - **Hardware Requirements**: Requires sufficient RAM and potentially GPU for larger models
  - **Model Size**: Larger models provide better results but require more resources
  - **Performance**: Local inference may be slower than cloud APIs depending on hardware
  - **Model Selection**: Choose models appropriate for code analysis vs. general chat

- **Note**: Ollama is particularly valuable for trading software development where code privacy and cost control are important.
  It can serve as a local alternative to cloud-based AI services for code analysis, documentation, and learning.
Consider using it alongside cloud services (like Cursor's AI) for a hybrid approach: Ollama for sensitive/proprietary analysis, cloud services for
general assistance.

## Project Management & Issue Tracking

### Linear.app (Issue Tracking & Project Management)

- **Official Website**: <https://linear.app/>
- **API Documentation**: <https://developers.linear.app/docs/graphql/working-with-the-graphql-api>
- **GraphQL API**: <https://api.linear.app/graphql>
- **License**: Proprietary (SaaS)
- **Integration Guide**: See `docs/LINEAR_INTEGRATION.md` for comprehensive setup and usage
- **Location**: `python/integration/linear_client.py`
- **Key Features**:
  - **GraphQL API**: Full GraphQL API for all operations
  - **Issue Tracking**: Create, update, and query issues
  - **Team Management**: Multi-team workspace support
  - **Workflow States**: Customizable workflow states (Backlog, In Progress, Done, etc.)
  - **Comments**: Add comments to issues
  - **Labels**: Organize issues with labels
  - **Priority Levels**: 0-4 priority system (0 = urgent)

- **Authentication**:
  - Personal Access Token from Linear Settings → API
  - Set `LINEAR_API_KEY` environment variable

- **Python Client**:
  - `LinearClient` class in `python/integration/linear_client.py`
  - Follows same pattern as `AlpacaClient` for consistency
  - Methods: `get_teams()`, `get_issues()`, `create_issue()`, `update_issue()`, `add_comment()`, `get_states()`

- **MCP Integration**:
  - Available through GitKraken MCP server (configured in `.cursor/mcp.json`)
  - Issue tracking through Cursor's AI assistant
  - Link commits to Linear issues
  - Create issues from code changes

- **Trading System Integration**:
  - Log trading errors to Linear for tracking
  - Track feature development and bug fixes
  - Link trading events to Linear issues
  - Monitor system health through Linear dashboards

- **Use Cases**:
  - Track TWS API integration progress
  - Log trading system errors and incidents
  - Manage feature development roadmap
  - Link code changes to issues
  - Coordinate multi-agent development (see `agents/shared/COORDINATION.md`)

- **Example Usage**:

  ```python
  from python.integration.linear_client import LinearClient

  client = LinearClient()
  teams = client.get_teams()
  issue = client.create_issue(
    team_id=teams[0]["id"],
    title="Fix box spread calculation",
    description="APR calculation incorrect for wide spreads",
    priority=1
  )
  ```

- **Note**: Linear provides both MCP integration (via GitKraken) for Cursor IDE and direct API access for programmatic issue tracking.
  The Python client enables automated issue creation from trading system events, error logging, and feature tracking.
  Particularly useful for coordinating multi-agent development workflows and tracking trading system incidents.

## Quick Reference Links

- **TWS API**: <https://interactivebrokers.github.io/tws-api/>
- **IBKR Campus - EClient/EWrapper**: <https://www.interactivebrokers.com/campus/ibkr-quant-news/the-eclient-and-ewrapper-api-classes/>
- **Alpaca Markets**: <https://alpaca.markets/> (API-first commission-free brokerage platform)
- **Alpaca Trading API Docs**: <https://docs.alpaca.markets/> (Trading API documentation)
- **Alpaca Broker API Docs**: <https://alpaca.markets/broker-api-docs/> (Broker API documentation)
- **Alpaca Elite Smart Router**: <https://docs.alpaca.markets/docs/alpaca-elite-smart-router> (DMA Gateway and advanced order types)
- **spdlog**: <https://github.com/gabime/spdlog>
- **CMake**: <https://cmake.org/documentation/>
- **Protocol Buffers**: <https://protobuf.dev/>
- **Catch2**: <https://github.com/catchorg/Catch2>
- **CLI11**: <https://cliutils.github.io/CLI11/book/>
- **nlohmann/json**: <https://json.nlohmann.me/>
- **Nautilus Trader**: historical/deprecated reference only
- **Public APIs Repository**: <https://github.com/public-apis/public-apis> (Curated list of free public APIs)
- **CppTrader**: <https://github.com/chronoxor/CppTrader> (High-performance trading components)
- **Alpha Vantage**: <https://www.alphavantage.co/> (Stock market data API with MCP support)
- **Finnhub**: <https://finnhub.io/> (Financial data API with generous free tier)
- **FLOX Framework**: <https://github.com/FLOX-Foundation/flox> (Modular C++ trading framework)
- **SmartQuant C++ Framework**: <https://www.smartquant.com/cpp.html> (Ultra-low latency trading framework)
- **FIX Trading Community**: <https://www.fixtrading.org/> (Industry-standard trading protocol)
- **FIXimate**: <https://fiximate.fixtrading.org/> (Interactive FIX protocol reference tool)
- **FIX Trading Community GitHub**: <https://github.com/FIXTradingCommunity> (FIX standards and tools)
- **Cboe Quoted Spread Book (QSB) FAQ**: <https://cdn.cboe.com/resources/membership/Quoted_Spread_Book_FAQ.pdf> (Cboe QSB service documentation)
- **Cboe Frequent Trader Program (FTID)**:
  <https://cdn.cboe.com/resources/membership/us-options-frequent-trader-program.pdf> (Fee rebate program for high-volume options trading)
- **Cboe EDGX Fee Schedule**: <https://www.cboe.com/us/equities/membership/fee_schedule/edgx/> (Market data and connectivity fee structure)
- **CME Group Fee Schedules**: <https://www.cmegroup.com/markets/products.html> (Futures and options fee structures and rebate programs)
- **Market Gear Options Platform**: <https://www.marketgear.com/options/> (Web-based options trading platform with strategy templates and backtesting)
- **SpeedBot Enterprise**: <https://speedbot.tech/speedbot-enterprise-for-brokers> (B2B white-label algo trading platform with API access for brokers)
- **ECN Execution**:
  <https://ecnexecution.com/algorithmic-trading/> (Educational resource about algorithmic trading, ECN brokers, and trading platforms)
- **Ollama**: <https://ollama.ai/> (Local LLM platform for running AI models on your machine)
- **Ollama GitHub**: <https://github.com/ollama/ollama> (Open-source local LLM platform)
- **Linear.app**: <https://linear.app/> (Issue tracking and project management)
- **Linear API Docs**: <https://developers.linear.app/docs/graphql/working-with-the-graphql-api> (GraphQL API documentation)
