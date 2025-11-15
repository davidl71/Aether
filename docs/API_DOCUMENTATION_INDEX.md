# API Documentation Index

This file serves as a reference for all external APIs and libraries used in this project. Use `@docs API_DOCUMENTATION_INDEX.md` in Cursor to give the AI context about these APIs.

## Core Trading APIs

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
  - **EClient and EWrapper Architecture**: <https://www.interactivebrokers.com/campus/ibkr-quant-news/the-eclient-and-ewrapper-api-classes/> - Official explanation of EClient/EWrapper pattern
  - See also: `docs/ECLIENT_EWRAPPER_ARCHITECTURE.md` - Detailed architecture documentation based on IBKR Campus

### Alpaca Markets (API-First Brokerage Platform)

- **Official Website**: <https://alpaca.markets/>
- **Trading API Docs**: <https://docs.alpaca.markets/>
- **Broker API Docs**: <https://alpaca.markets/broker-api-docs/>
- **GitHub**: <https://github.com/alpacahq>
- **Founded**: 2015 (Yoshi Yokokawa, Hitoshi Harada)
- **Backing**: Y Combinator, Spark Capital, Tribe Capital, Horizon Ventures, Portage Ventures, Eldridge, Unbound
- **Description**: Developer-first, API-driven brokerage platform providing commission-free trading for U.S. stocks, ETFs, options, and cryptocurrencies. Serves over 5 million brokerage accounts and 200+ financial clients across 40 countries.
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
    - **DMA Gateway (Direct Market Access)**: Control where orders are sent (NYSE, NASDAQ, ARCA, with plans for 10+ additional destinations including BATS, IEX, AMEX)
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
- **Example Use Case**: Execute box spread strategies on Alpaca's commission-free options API with DMA Gateway for direct exchange routing (e.g., route to NASDAQ for SPXW options). Use VWAP orders for large multi-leg box spread positions to minimize market impact. Compare execution prices and rates across both Alpaca (Elite) and IBKR platforms for optimal routing. For time-sensitive arbitrage, use TWAP orders to execute evenly over a specified window.
- **Note**: Alpaca is particularly well-suited for U.S.-focused algorithmic trading with a developer-friendly API. The commission-free options trading could be advantageous for high-volume box spread strategies where transaction costs significantly impact profitability. With Elite Smart Router access, DMA Gateway provides direct exchange routing control, and VWAP/TWAP orders enable sophisticated execution algorithms for large multi-leg positions. Consider Alpaca (Elite) as a complementary execution venue alongside IBKR for multi-venue strategies with enhanced execution control.

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
- **Key Functions**: `___bid64_add`, `___bid64_div`, `___bid64_mul`, etc.
- **Location**: `native/third_party/IntelRDFPMathLib20U2/LIBRARY/libbid.a`
- **Note**: Required by TWS API for decimal price handling

### Protocol Buffers

- **Official Docs**: <https://protobuf.dev/>
- **C++ API**: <https://protobuf.dev/cpp/>
- **Version**: 6.33.0+
- **Purpose**: Serialization for TWS API messages
- **Generated Files**: `*.pb.cc`, `*.pb.h` in TWS API client directory
- **Location**: `/usr/local/lib/libprotobuf.dylib`

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
- **Purpose**: High-performance Python trading framework
- **Location**: `native/third_party/nautilus/`
- **Note**: Optional integration, Python wheel file

## Market Data APIs

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
- **Note**: Market Gear is a useful reference tool for options strategy research and visualization, but does not provide API access for integration into automated trading systems. Use it for manual strategy design and backtesting before implementing automated strategies in code.

### SpeedBot Enterprise (B2B Algo Trading Platform)

- **Official Website**: <https://speedbot.tech/speedbot-enterprise-for-brokers>
- **API Access Page**: <https://speedbot.tech/speedbot-enterprise-for-brokers#api-access>
- **Description**: B2B white-label Platform-as-a-Service (PaaS) for brokers, sub-brokers, and algo strategy creators to offer algorithmic trading services under their own brand
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
- **Note**: SpeedBot Enterprise is a B2B white-label algo trading platform designed for brokers to offer algorithmic trading services to their clients. While it provides API access, the documentation is not publicly available and requires an enterprise agreement. This project is a self-hosted, open-source alternative that provides direct IBKR TWS API integration for individual traders and algorithmic trading systems. SpeedBot could serve as a reference for B2B platform architecture and enterprise features, but is not a direct integration target for this project.

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
- **Note**: ECN Execution is an educational and informational resource about algorithmic trading, ECN brokers, and trading platforms. It does not provide an API or trading services - it serves as a reference for understanding algorithmic trading concepts, researching brokers, and comparing trading platforms. While useful for broker research and educational purposes, this project provides direct algorithmic trading implementation via IBKR TWS API integration.

## Open Data APIs & Resources

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

#### Alpha Vantage

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

#### Finnhub

- **URL**: <https://finnhub.io/>
- **Official API Docs**: <https://finnhub.io/docs/api>
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

### FLOX (Modular Trading Framework)

- **GitHub**: <https://github.com/FLOX-Foundation/flox>
- **Documentation**: <https://flox-foundation.github.io/flox/>
- **License**: MIT License
- **Language**: Modern C++ (C++20)
- **Description**: Modular framework for building trading systems, providing low-level infrastructure for execution pipelines, market data processing, strategy logic, and exchange integration
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
- **Disclaimer**: FLOX is provided for educational and research purposes. All strategies, connectors, and logic in test and demo code are demonstrative only and not intended for production use.

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
- **Research Documentation**: See `docs/SMARTQUANT_CPP_FRAMEWORK_RESEARCH.md` for comprehensive research, integration strategies, comparison with current architecture, and implementation considerations
- **Status**: Research complete, integration not yet planned
- **Note**: Framework designed for institutional-grade HFT systems. Evaluate licensing, cost, and integration complexity before proceeding.

### FIX Protocol & FIX Trading Community

- **Official Website**: <https://www.fixtrading.org/>
- **FIXimate (Interactive FIX Reference)**: <https://fiximate.fixtrading.org/>
- **GitHub Organization**: <https://github.com/FIXTradingCommunity>
- **License**: Standards organization (non-profit)
- **Description**: The FIX (Financial Information eXchange) Protocol is the industry-standard messaging protocol for electronic trading across global financial markets. The FIX Trading Community is the non-profit standards organization that maintains and develops FIX specifications.
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
- **Potential Integration Opportunities**:
  - Direct CME Globex connectivity for futures trading
  - Direct Cboe connectivity for options trading (SPX, SPXW)
  - Market data feeds via FIX (quotes, trades, order book)
  - High-frequency execution using SBE encoding
  - Multi-venue arbitrage strategies with low latency
- **Example Use Case**: Direct CME/Cboe FIX connectivity could enable faster execution for box spread strategies by bypassing IBKR's TWS API, potentially reducing latency and improving fill rates.
- **Note**: FIX protocol is the industry standard for institutional trading. While more complex than REST APIs, it offers lower latency, better control, and direct exchange access. FIXimate provides an excellent reference for understanding FIX message structures and field definitions.

### Tools for Brokers (TFB) FIX API Platform

- **Website**: <https://t4b.com/fix-api/>
- **Provider**: Tools for Brokers (TFB) - Technology provider for retail brokers
- **Description**: FIX API platform for retail brokers, hedge funds, and liquidity providers enabling ultra-low latency trading execution and liquidity aggregation
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
- **Note**: TFB FIX API platform could complement or replace broker APIs for direct exchange access. Contact TFB for pricing and capabilities. Evaluate as alternative execution venue for box spread trading, particularly for direct CBOE access and multi-venue arbitrage strategies.

### OnixS directConnect - Ultra Low Latency DMA SDKs

- **Website**: <https://www.onixs.biz/directconnect.html>
- **Provider**: OnixS - Financial technology solutions
- **Description**: Ultra-low latency Direct Market Access SDKs for exchanges and liquidity pools, with multi-platform implementations of Market Data Handlers, Order Routing, and DropCopy/Trade Capture
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
- **Note**: OnixS directConnect provides ultra-low latency SDKs for direct CBOE access, particularly valuable for SPX/SPXW box spread trading. The C++ SDK aligns with the project's technology stack. Contact OnixS for pricing, 30-day evaluation, and integration support. Consider as alternative to broker APIs for direct CBOE access and ultra-low latency execution.

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
- **Note**: OnixS FIX Protocol tools are essential for FIX protocol development, debugging, and optimization. Use FIX Dictionary for reference, FIX Analyser for debugging, and SBE/FAST codecs for high-performance implementations. Particularly valuable when implementing direct exchange access (CBOE, CME) for box spread trading.

### Brokeree Solutions

- **Website**: <https://brokeree.com/solutions/>
- **Provider**: Brokeree Solutions - Technology provider for retail brokers
- **Description**: Turnkey technology solutions for retail forex and CFD brokers, primarily focused on MetaTrader 4/5 (MT4/MT5) and cTrader platforms
- **Primary Focus**: Retail forex/CFD broker infrastructure (liquidity bridges, PAMM systems, social trading, prop trading solutions, MetaTrader plugins)
- **Key Solutions**:
  - **Liquidity Bridge**: Multi-server liquidity aggregation and risk management
  - **TradingView API**: Connect TradingView to trading platforms
  - **MT4/MT5 FIX API**: FIX protocol integration for MetaTrader
  - **MT4/MT5 REST API**: REST API for MetaTrader platforms
  - **MT5 Gateways**: Gateways to various liquidity providers (DASTrader, Exante, LMAX, SAXO Bank, AC Markets)
- **Potentially Relevant Components**:
  - **MT5 Gateway to DASTrader**: Direct Access Software for US exchanges (CBOE/BATS/EDGE, CBSX, Nasdaq, AMEX/NYSE/ARCA, OTC) - provides CBOE access but requires MetaTrader 5
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
- **Note**: Brokeree Solutions is primarily for retail forex/CFD brokers using MetaTrader platforms. While some components (DASTrader gateway) provide CBOE access, the platform is not well-suited for institutional C++ options trading systems. For box spread trading, OnixS directConnect, TFB FIX API, or direct FIX implementation would be more appropriate alternatives.

## Brokerage API Resources

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
  - **Lightspeed Trader API**: C++ native API, high performance (1,500 orders/sec), low latency, co-location available, options support, no additional market data fees for API use
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
- **Note**: QuantPedia provides a valuable resource for discovering brokerage APIs. The project already uses IBKR (integrated) and has Alpaca documented. Lightspeed Trader API (C++ native) and Lime Brokerage (FIX/API) are potentially valuable alternatives worth evaluating for high-performance options trading. Always verify current API status, pricing, and capabilities before integration.

### QuantPedia Subscription - Trading Strategy Research Platform

- **Website**: <https://quantpedia.com/pricing/>
- **Provider**: QuantPedia - Encyclopedia of Quantitative Trading Strategies
- **Description**: Subscription-based research platform providing access to quantitative trading strategies, academic research papers, backtests, and portfolio analysis tools
- **Subscription Tiers**:
  - **Prime**: 100+ essential strategies, essential portfolio modeling, no research papers, no reports (entry-level)
  - **Premium**: 900+ full strategies, essential portfolio modeling, 1000s of research papers, 800+ backtests, regular updates (recommended for research)
  - **Pro**: 900+ full strategies, full portfolio modeling, 1000s of research papers, 800+ backtests, 30+ Quant Reports, AI Chatbot, regular updates (best value for professionals)
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
- **Note**: QuantPedia is a research and educational platform, not a trading execution platform. It provides strategy ideas, academic research, and backtesting examples that can inform box spread trading strategies. The arbitrage strategy category is particularly relevant. Premium tier recommended for strategy research; Pro tier for advanced portfolio analysis.

### eToro - Social Trading Platform

- **Website**: <https://www.etoro.com/discover>
- **Provider**: eToro - Social trading and investment platform
- **Description**: Social trading platform enabling users to invest in various assets and copy trades from experienced investors using CopyTrader™ technology
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
- **Note**: eToro is a social trading platform for retail investors to copy trades. It does not offer options trading, algorithmic trading APIs, or capabilities required for box spread trading. For box spread strategies, use IBKR, Alpaca, or other options-capable brokers with API access.

## Market Structure & Efficiency References

### CME Group – Capital Efficiencies and AIR TRFs

- **Whitepaper**: <https://www.cmegroup.com/articles/whitepapers/capital-efficiencies-and-air-trfs.html>
- **Focus**: Explains capital efficiency benefits of Alternative Index Replication (AIR) Total Return Futures.
- **Relevance**: Useful for comparing margin treatment and financing costs when evaluating box-spread arbitrage versus futures-based replication strategies.
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

- **PDF**: <https://www.optionseducation.org/getmedia/2ae6c8bd-9a8e-4d2f-8168-19b6ff9e3589/listed-options-box-spread-strategies-for-borrowing-or-lending-cash.pdf>
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
- **Focus**: Explains Cboe's Quoted Spread Book service that allows Market Makers to rest orders directly in Complex Order Books (COBs) for select spread instruments during Regular Trading Hours (RTH).
- **Key Features**:
  - **Box Spreads**: Box Spreads on first four serial, first three quarterly, and first three December standard SPX contracts at 4000 and 5000 strikes (~10 quotable instruments daily).
  - **Box Swaps**: Box Spread rolls from first four serial and first two quarterly expirations to forward contracts (~25 quotable instruments daily).
  - **Jelly Rolls**: Combo rolls from 0-DTE, 1-DTE, and nearest Friday SPXW, plus first three serial and first two quarterly SPX contracts (~120 quotable instruments daily).
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
- **Note**: Prior to QSB, Market Makers were not allowed to rest orders in SPX/SPXW Option COBs during RTH. QSB enables electronic trading that was previously predominantly open outcry.

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
- **Relevance**: Complements box-spread financing analysis by framing how equity financing risk and basis can be hedged or benchmarked against futures.
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
- **Relevance**: Helps align our automation with CME interface changes (e.g., market data, clearing, settlement timings) when evaluating financing trades that interact with futures infrastructure.

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
- **Note**: Fee schedules are product-specific and updated periodically. Always verify current fees and rebates with CME Group before making trading decisions.

## Risk Management & Hedging

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
  - **Alternative Sources**: OANDA API, FXCM API, Alpha Vantage, Finnhub
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
- **Note**: Currency exchange rates are highly volatile and can significantly impact returns. Always assess currency risk before entering positions and consider hedging when exposure is significant. The codebase includes currency hedging infrastructure; integrate real-time exchange rate data for production use.

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
- **Note**: Free web-based tools for educational and analytical purposes. Always verify calculations with your own analysis. Use built-in project tools for production trading decisions.

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
  - **TradeStation Global**: Powerful API, platform support, options trading
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
- **Note**: Broker selection is critical for algorithmic trading success. The project uses IBKR as primary broker (already integrated) with Alpaca as documented alternative. Always verify regulatory compliance and test APIs thoroughly before live trading.

## Rust (Agents)

### Rust Standard Library

- **Official Docs**: <https://doc.rust-lang.org/std/>
- **Location**: `agents/backend/`, `agents/backend-mock/`, etc.
- **Cargo**: `Cargo.toml` files in agent directories

## Go (TUI)

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
- **Location**: `desktop/`, `ios/`
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
- **Nautilus Trader**: <https://docs.nautilustrader.io/>
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
- **Cboe Frequent Trader Program (FTID)**: <https://cdn.cboe.com/resources/membership/us-options-frequent-trader-program.pdf> (Fee rebate program for high-volume options trading)
- **Cboe EDGX Fee Schedule**: <https://www.cboe.com/us/equities/membership/fee_schedule/edgx/> (Market data and connectivity fee structure)
- **CME Group Fee Schedules**: <https://www.cmegroup.com/markets/products.html> (Futures and options fee structures and rebate programs)
- **Market Gear Options Platform**: <https://www.marketgear.com/options/> (Web-based options trading platform with strategy templates and backtesting)
- **SpeedBot Enterprise**: <https://speedbot.tech/speedbot-enterprise-for-brokers> (B2B white-label algo trading platform with API access for brokers)
- **ECN Execution**: <https://ecnexecution.com/algorithmic-trading/> (Educational resource about algorithmic trading, ECN brokers, and trading platforms)
