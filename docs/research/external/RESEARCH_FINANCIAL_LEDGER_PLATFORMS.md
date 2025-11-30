# Financial Ledger and Accounting Platforms Research

**Version:** 1.0.0
**Last Updated:** 2025-11-18
**Status:** Research Document
**Related:** `docs/INVESTMENT_STRATEGY_FRAMEWORK.md`

## Executive Summary

This document provides comprehensive research and analysis of modern financial ledger and accounting platforms for potential integration with the IB box spread portfolio management system. The research evaluates **Ledger CLI**, Formance, Blnk, Firefly III, Akaunting, GnuCash, LedgerSMB, FINOS, ERPNext, and UniBee platforms to understand their architectures, features, and integration patterns.

**Key Finding:** The current system tracks positions and calculates PnL but lacks formal double-entry ledger accounting. **Ledger CLI** emerges as the most applicable solution due to its C++ codebase that aligns perfectly with the existing architecture. Modern platforms like Formance and Blnk offer developer-first approaches with REST APIs suitable for integration, while traditional accounting software provides proven patterns but may not fit trading-focused use cases.

**Recommended Approach:** Integrate Ledger CLI C++ core library directly into the IB box spread system, or use it as a reference implementation for a custom lightweight ledger module optimized for trading operations.

## Table of Contents

1. [Current System Analysis](#current-system-analysis)
2. [Platform Evaluations](#platform-evaluations)
   - [Ledger CLI](#9-ledger-cli-plain-text-accounting) ⭐ **HIGHLY RECOMMENDED**
   - [LedgerSMB](#10-ledgersmb)
   - [Other Platforms](#platform-evaluations)
3. [Architecture Comparison](#architecture-comparison)
4. [Integration Patterns](#integration-patterns)
5. [Recommendations](#recommendations)

---

## Current System Analysis

### Existing Position Tracking

The IB box spread system currently uses basic position tracking without formal accounting:

**Current Implementation (`agents/backend/crates/api/src/state.rs`):**

```rust

#[derive(Clone, Debug, Serialize, Deserialize)]

pub struct PositionSnapshot {
  pub id: String,
  pub symbol: String,
  pub quantity: i32,
  pub cost_basis: f64,
  pub mark: f64,
  pub unrealized_pnl: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]

pub struct HistoricPosition {
  pub id: String,
  pub symbol: String,
  pub quantity: i32,
  pub realized_pnl: f64,
  pub closed_at: DateTime<Utc>,
}
```

**Current Limitations:**

- ❌ No double-entry accounting (debits/credits)
- ❌ No balance reconciliation
- ❌ No account/chart of accounts structure
- ❌ No transaction journal
- ❌ No audit trail beyond order history
- ❌ Basic PnL calculation without formal accounting principles
- ❌ No cash flow tracking (planned but not implemented)

**Current System Strengths:**

- ✅ Real-time position tracking
- ✅ Order execution history
- ✅ Unrealized/realized PnL calculation
- ✅ Multi-currency awareness (ILS → USD conversion planned)
- ✅ Comprehensive portfolio allocation framework

### Integration Context

**Investment Strategy Framework:**

- Portfolio allocation framework exists (`docs/INVESTMENT_STRATEGY_FRAMEWORK.md`)
- Tracks cash allocation, box spreads, T-bills, ETFs
- Net portfolio value calculation (assets - loan liabilities)
- Cash flow forecasting planned but not yet implemented

**Architecture:**

- C++ core (`native/src/`) with box spread calculations
- Rust backend (`agents/backend/`) for state management
- Python integration layer
- **Gap:** No accounting/ledger module

---

## Platform Evaluations

### 1. Formance Platform

**Website:** <https://docs.formance.com/welcome>
**Type:** Open-source, cloud-native financial infrastructure
**License:** Unknown (check repository)
**Language:** Unknown (check repository)
**Status:** Active development

#### Overview

Formance is an open-source, cloud-native financial infrastructure platform designed to help developers build bespoke flows of funds with precision and scalability.

#### Core Modules

1. **Ledger Module**
   - Programmable accounting database for financial transactions
   - Accurately track money movements and account balances
   - Double-entry accounting principles

2. **NumScript DSL**
   - Domain-specific language for modeling complex financial transactions
   - Simple, powerful, extensible DSL built for money movements
   - Enables complex transaction workflows

3. **Wallets Module**
   - Fully managed white-label wallet service
   - Hold and capture flows
   - User fund management

4. **Flows Module**
   - End-to-end financial workflow orchestration service
   - Coordinate complex financial operations

5. **Reconciliation Module**
   - Reporting toolkit for auditing assets-under-management
   - Verify ledger balances against external sources

6. **Connectivity Module**
   - Unified data layer and API for payment processors
   - Support for Stripe, Wise, and payment initiation

#### Key Features

- ✅ Cloud-native architecture
- ✅ REST API for programmatic access
- ✅ Double-entry accounting
- ✅ Workflow orchestration
- ✅ Multi-currency support (implied)
- ✅ Audit and reconciliation tools

#### Architecture

- Cloud-native design (Docker/Kubernetes compatible)
- Microservices architecture (modular design)
- RESTful APIs

#### Use Cases

- Payment processing platforms
- Wallet applications
- Financial transaction workflows
- Asset management systems

#### Integration Assessment

**Pros:**

- Modern, developer-first approach
- REST APIs suitable for integration
- Cloud-native design aligns with modern architectures
- Comprehensive workflow orchestration

**Cons:**

- Unknown technology stack (need to verify)
- May require significant integration work
- External dependency for critical accounting functions
- Potential performance overhead for trading operations

**Applicability to IB Box Spread System:** ⭐⭐⭐⭐ (4/5)
Highly applicable for modern ledger architecture, but need to verify technology stack compatibility.

---

### 2. Blnk Finance

**Website:** <https://github.com/blnkfinance/blnk>
**Repository:** <https://github.com/blnkfinance/blnk>
**Type:** Open-source ledger & financial core
**License:** Apache-2.0
**Language:** Go (95.5% of codebase)
**Status:** Active development (284 stars, 75 forks)

#### Overview

Blnk is an open-source ledger and financial core designed to help developers ship fintech products fast without compromising compliance and correctness.

#### Core Features

1. **Ledger**
   - Open-source double-entry ledger for managing balances
   - Transaction workflow recording
   - Balance monitoring and snapshots
   - Historical balances
   - Inflight transactions (hold/capture patterns)
   - Scheduling and overdrafts
   - Bulk transactions

2. **Reconciliation**
   - Automatic matching of external records (bank statements) to internal ledger
   - Custom matching rules
   - Reconciliation strategies

3. **Identity Management**
   - Create & manage identities
   - PII tokenization features
   - Link identities to balances and transactions

#### Architecture

**Technology Stack:**

- **Language:** Go (95.5%)
- **Database:** PostgreSQL
- **Cache:** Redis
- **Search:** Typesense
- **Deployment:** Docker Compose

**Data Model:**

- PostgreSQL for ledger data
- Redis for caching and real-time operations
- Typesense for search functionality

#### Use Cases

1. Wallet Management
2. Deposits & Withdrawals
3. Order Exchange
4. Lending
5. Loyalty Points System
6. Savings Application
7. Escrow Application

#### API Design

- REST API available
- Docker-based deployment
- Configuration via JSON (`blnk.json`)

**Example Configuration:**

```json
{
  "project_name": "Blnk",
  "data_source": {
    "dns": "postgres://postgres:password@postgres:5432/blnk?sslmode=disable"
  },
  "redis": {
    "dns": "redis:6379"
  },
  "typesense": {
    "dns": "http://typesense:8108"
  },
  "server": {
    "port": "5001"
  }
}
```

#### Integration Assessment

**Pros:**

- Lightweight, Go-based architecture
- Well-documented with active community
- Reconciliation features for external record matching
- Identity management with PII tokenization
- PostgreSQL + Redis architecture familiar to developers

**Cons:**

- Go-based (may require integration layer for C++/Rust)
- External service dependency
- May add latency for real-time trading operations

**Applicability to IB Box Spread System:** ⭐⭐⭐⭐ (4/5)
Good fit for ledger architecture, but Go-based stack may require integration bridge.

---

### 3. Firefly III

**Website:** <https://docs.firefly-iii.org/>
**Type:** Self-hosted personal finance manager
**License:** AGPL-3.0
**Language:** PHP (Laravel)
**Status:** Active development

#### Overview

Firefly III is a self-hosted personal finance manager that helps users keep track of expenses, income, budgets, and more.

#### Key Features

- ✅ Double-entry bookkeeping
- ✅ Account management
- ✅ Transaction categories and tags
- ✅ Budget analysis
- ✅ Expense tracking
- ✅ Financial reports
- ✅ RESTful API
- ✅ Multi-currency support

#### Architecture

- PHP (Laravel framework)
- Web-based interface
- Self-hosted deployment
- MySQL/PostgreSQL database

#### Use Cases

- Personal finance tracking
- Expense management
- Budget planning
- Financial reporting

#### Integration Assessment

**Pros:**

- Well-established double-entry patterns
- RESTful API available
- Active community and documentation

**Cons:**

- Designed for personal finance, not trading/investment portfolios
- PHP/Laravel stack (may not integrate well with C++/Rust)
- Focus on expense tracking rather than portfolio accounting

**Applicability to IB Box Spread System:** ⭐⭐ (2/5)
Good reference for double-entry patterns, but not designed for trading portfolios.

---

### 4. Akaunting

**Website:** <https://akaunting.com/>
**Type:** Open-source online accounting software
**License:** GPL-3.0
**Language:** PHP (Laravel)
**Status:** Active development

#### Overview

Akaunting is a free, open-source, online accounting software designed for small businesses and freelancers.

#### Key Features

- ✅ Invoicing
- ✅ Expense tracking
- ✅ Financial reporting
- ✅ Multi-currency support
- ✅ Client portals
- ✅ Multi-language (50+ languages)
- ✅ Web-based access

#### Architecture

- PHP (Laravel framework)
- Web-based application
- MySQL/PostgreSQL database

#### Use Cases

- Small business accounting
- Freelancer financial management
- Invoice management
- Expense tracking

#### Integration Assessment

**Pros:**

- Comprehensive accounting features
- Multi-currency support
- Web-based access

**Cons:**

- Traditional business accounting focus
- Not designed for trading/investment portfolios
- PHP/Laravel stack (integration challenges)

**Applicability to IB Box Spread System:** ⭐⭐ (2/5)
Reference for traditional accounting features, but not trading-focused.

---

### 5. GnuCash

**Website:** <https://www.gnucash.org/>
**Type:** Desktop accounting software
**License:** GPL-2.0
**Language:** C/C++
**Status:** Active development

#### Overview

GnuCash is a free, open-source financial accounting software designed for individuals and small businesses.

#### Key Features

- ✅ Double-entry accounting
- ✅ Invoicing
- ✅ Accounts payable and receivable
- ✅ Multi-currency support
- ✅ **Investment tracking** (stocks/securities with price quotes)
- ✅ Financial reports
- ✅ Desktop application (GTK-based)

#### Architecture

- C/C++ codebase (may align with existing C++ core)
- GTK-based desktop application
- XML/SQLite for data storage
- Price quote integration for securities

#### Use Cases

- Personal finance management
- Small business accounting
- Investment portfolio tracking
- Financial reporting

#### Investment Tracking Features

- Stock/security position tracking
- Price quote integration
- Portfolio valuation
- Cost basis tracking

#### Integration Assessment

**Pros:**

- C/C++ codebase (potential direct integration)
- Investment tracking features relevant to trading
- Desktop application (can integrate as library)

**Cons:**

- Desktop-focused (not API-first)
- Traditional accounting focus
- May require significant code extraction for API use

**Applicability to IB Box Spread System:** ⭐⭐⭐ (3/5)
Good reference for investment tracking patterns, but desktop-focused design limits integration.

---

### 6. FINOS Foundation

**Website:** <https://community.finos.org/docs/collaboration-infrastructure/>
**GitHub:** <https://github.com/finos>
**Type:** Financial open source foundation
**License:** Various (per project)
**Status:** Active organization

#### Overview

FINOS (Fintech Open Source Foundation) provides collaborative infrastructure for open-source projects in the financial services industry.

#### Key Focus Areas

- Enterprise financial services
- Regulatory compliance
- Industry standards
- Financial technology frameworks

#### Projects

Various open-source projects hosted on GitHub:

- Financial modeling tools
- Regulatory reporting frameworks
- API standards
- Data management tools

#### Integration Assessment

**Pros:**

- Industry standards and best practices
- Financial services focus
- Collaborative community

**Cons:**

- Not a specific ledger platform
- More focused on enterprise/compliance than trading portfolios
- Various projects with different architectures

**Applicability to IB Box Spread System:** ⭐⭐ (2/5)
Resource for financial services tools and standards, but not a direct ledger solution.

---

### 7. ERPNext

**Website:** <https://frappe.io/erpnext/open-source-accounting>
**Type:** Open-source ERP with accounting module
**License:** GPL-3.0
**Language:** Python (Frappe framework)
**Status:** Active development

#### Overview

ERPNext is an open-source integrated Enterprise Resource Planning (ERP) software that includes comprehensive accounting functionality.

#### Accounting Features

- ✅ Double-entry bookkeeping
- ✅ General ledger
- ✅ Accounts payable and receivable
- ✅ Financial statements
- ✅ Multi-currency accounting
- ✅ Budget management
- ✅ Cost center accounting

#### Architecture

- Python-based (Frappe framework)
- Web-based application
- MySQL/MariaDB database
- REST API available

#### Use Cases

- Enterprise resource planning
- Business accounting
- Financial management
- Inventory and sales management

#### Integration Assessment

**Pros:**

- Comprehensive accounting features
- REST API available
- Active development and community

**Cons:**

- Heavyweight ERP system (overkill for trading ledger)
- Not designed for trading/investment portfolios
- Python-based (may require integration layer)

**Applicability to IB Box Spread System:** ⭐⭐ (2/5)
Reference for enterprise accounting features, but too heavyweight for trading-focused needs.

---

### 8. UniBee

**Website:** <https://unibee.dev/>
**Type:** Payment/subscription platform (not a ledger system)
**Status:** Unknown (limited documentation)

#### Overview

Limited public documentation available. Appears to be a payment/subscription platform rather than a ledger/accounting system.

#### Integration Assessment

**Pros:**

- May have payment processing features

**Cons:**

- Not a ledger/accounting system
- Limited documentation
- Unknown architecture and features

**Applicability to IB Box Spread System:** ⭐ (1/5)
Not relevant for ledger accounting needs.

---

### 9. Ledger CLI (Plain Text Accounting)

**Website:** <https://ledger-cli.org/>
**Repository:** <https://github.com/ledger/ledger>
**Type:** Command-line double-entry accounting system
**License:** BSD
**Language:** C++
**Status:** Active development (5.7k stars, 525 forks, updated Nov 15, 2025)

#### Overview

Ledger CLI is a powerful, double-entry accounting system that operates entirely via the command line. Created in 2003 by John Wiegley, it allows users to maintain financial records using plain text files with Ledger's transaction format.

#### Key Features

- ✅ Double-entry bookkeeping
- ✅ Plain text file format (human-readable)
- ✅ Command-line reporting interface
- ✅ **C++ codebase** (aligns with existing C++ core)
- ✅ No database required (text files)
- ✅ Multi-currency support
- ✅ Balance reporting
- ✅ Transaction queries and filtering
- ✅ Budget tracking
- ✅ Investment position tracking

#### Architecture

**Technology Stack:**

- **Language:** C++ (100% C++ core)
- **Storage:** Plain text files (`.ledger` files)
- **Interface:** Command-line tool (`ledger` command)
- **No external dependencies** (self-contained binary)

**Data Format:**
Transactions stored in plain text format:

```
2025/11/18 * Buy SPY
    Assets:IBKR:SPY           100 SPY @ $450.00
    Assets:IBKR:Cash       -$45,000.00
```

**Reporting:**
Command-line reports and queries:

```bash
ledger balance                    # Show account balances
ledger register Assets:IBKR:SPY  # Show SPY transaction history
ledger report                     # Generate financial reports
```

#### Use Cases

- Personal finance tracking
- Investment portfolio accounting
- Business accounting
- Financial reporting and analysis
- Budget management

#### Key Advantages

1. **C++ Codebase:** Native C++ aligns perfectly with existing IB box spread C++ core
2. **No Database:** Plain text files eliminate database dependencies
3. **Lightweight:** Self-contained binary, minimal overhead
4. **Trading-Friendly:** Can track investment positions and securities
5. **Portable:** Text files are portable and version-controllable (Git-friendly)
6. **Extensible:** Multiple language bindings (Python, Rust ports exist)

#### Integration Assessment

**Pros:**

- ✅ **C++ codebase** - can be embedded as library or integrated directly
- ✅ **Plain text format** - human-readable, easy to debug
- ✅ **No database dependency** - simple, lightweight
- ✅ **Investment tracking** - designed for securities/portfolio accounting
- ✅ **Command-line interface** - scriptable, automatable
- ✅ **Well-established** - 20+ years of development, 5.7k stars

**Cons:**

- ❌ **Text file format** - may not scale for high-frequency trading
- ❌ **Command-line focused** - not API-first (but can be wrapped)
- ❌ **File-based** - concurrent access may require file locking
- ❌ **Reporting-oriented** - designed for reports, not real-time transactions

**Possible Integration Approaches:**

1. **Embed as Library:** Extract core C++ ledger logic, embed in IB box spread system
2. **CLI Integration:** Call `ledger` command-line tool from system, parse output
3. **Text File Sync:** Write transactions to `.ledger` files, use for reconciliation
4. **C++ Port Integration:** Use C++ core directly, build API wrapper

**Applicability to IB Box Spread System:** ⭐⭐⭐⭐⭐ (5/5)
**Highly applicable** - C++ codebase aligns perfectly with existing architecture. Can be embedded as library or used for text-based transaction recording.

---

### 10. LedgerSMB

**Website:** <https://ledgersmb.org/>
**Type:** Open-source ERP with accounting
**License:** GPL-2.0
**Language:** Perl
**Status:** Active development (latest version 1.13, released Oct 4, 2025)

#### Overview

LedgerSMB is an open-source Enterprise Resource Planning (ERP) system designed for small and mid-sized businesses. It provides a comprehensive accounting foundation with multi-currency support and integrated business management features.

#### Key Features

- ✅ Double-entry bookkeeping
- ✅ General ledger
- ✅ Accounts payable and receivable
- ✅ **Multi-currency accounting**
- ✅ Invoicing and order processing
- ✅ Inventory management
- ✅ Fixed asset accounting
- ✅ Financial reporting
- ✅ Sales management
- ✅ **No vendor lock-in** (open-source)

#### Architecture

**Technology Stack:**

- **Language:** Perl
- **Database:** PostgreSQL
- **Interface:** Web-based application
- **Deployment:** Self-hosted

#### Use Cases

- Small and mid-size business accounting
- Enterprise resource planning
- Financial management
- Inventory and sales management
- Multi-currency business operations

#### Integration Assessment

**Pros:**

- ✅ Comprehensive accounting features
- ✅ Multi-currency support (relevant for ILS/USD)
- ✅ PostgreSQL database (familiar technology)
- ✅ Active development (recent release)

**Cons:**

- ❌ **Perl-based** - not aligned with C++/Rust stack
- ❌ **Web-based ERP** - heavyweight, overkill for trading ledger
- ❌ **Business-focused** - not designed for trading/investment portfolios
- ❌ **Web interface** - not API-first

**Applicability to IB Box Spread System:** ⭐⭐ (2/5)
Good reference for multi-currency accounting patterns, but Perl-based and ERP-focused design limits integration.

---

**Note on LedgerHQ (Hardware Wallet):**

The user also provided links to LedgerHQ's `lib-ledger-core`, which is **NOT a financial ledger/accounting system**. LedgerHQ is a cryptocurrency hardware wallet company, and `lib-ledger-core` is their C++ library for integrating with Ledger hardware wallets. This library was **archived on Feb 12, 2025** and is not relevant for financial accounting/ledger needs.

**Applicability to IB Box Spread System:** ⭐ (1/5)
Not relevant - hardware wallet integration library, not accounting ledger.

---

## Architecture Comparison

| Platform | Language | Architecture | API | Trading Focus | Integration Ease |
|----------|----------|--------------|-----|---------------|------------------|
| **Ledger CLI** | **C++** | Plain text files | CLI | ✅ Investment tracking | ⭐⭐⭐⭐⭐ |
| **Formance** | Unknown | Cloud-native microservices | REST ✅ | ⚠️ General financial | ⭐⭐⭐⭐ |
| **Blnk** | Go | PostgreSQL + Redis | REST ✅ | ⚠️ General fintech | ⭐⭐⭐ |
| **GnuCash** | C/C++ | Desktop app | Limited | ⚠️ Investment tracking | ⭐⭐⭐ |
| **Firefly III** | PHP | Laravel web app | REST ✅ | ❌ Personal finance | ⭐⭐ |
| **LedgerSMB** | Perl | PostgreSQL web app | Web UI | ❌ ERP/Business | ⭐⭐ |
| **ERPNext** | Python | Frappe web app | REST ✅ | ❌ Enterprise ERP | ⭐⭐ |
| **Akaunting** | PHP | Laravel web app | Web UI | ❌ Small business | ⭐ |
| **Custom Build** | C++/Rust | Direct integration | Native | ✅ Trading-focused | ⭐⭐⭐⭐⭐ |

---

## Integration Patterns

### Pattern 1: External Service Integration

**Approach:** Integrate Formance or Blnk as external ledger service

**Architecture:**

```
IB Box Spread System (C++/Rust)
    ↓ REST API
Formance/Blnk Ledger Service
    ↓
PostgreSQL/Redis Database
```

**Pros:**

- Leverages existing, tested ledger implementation
- Separation of concerns
- Can scale independently

**Cons:**

- Network latency for every transaction
- External dependency
- Potential performance bottlenecks for high-frequency trading
- Additional infrastructure to manage

### Pattern 2: Embedded Library

**Approach:** Extract ledger logic from GnuCash or build custom lightweight ledger

**Architecture:**

```
IB Box Spread System (C++/Rust)
    ↓ Native calls
Embedded Ledger Module (C++/Rust)
    ↓
Local Database (PostgreSQL/SQLite)
```

**Pros:**

- No network latency
- Direct integration with existing codebase
- Full control over implementation
- Trading-optimized design

**Cons:**

- Requires development effort
- Must maintain ledger logic
- Less battle-tested than external services

### Pattern 3: Hybrid Approach

**Approach:** Custom lightweight ledger with external reconciliation

**Architecture:**

```
IB Box Spread System
    ↓
Custom Ledger Module (high-performance, trading-focused)
    ↓
Local Database
    ↓ Periodic sync
External Reconciliation Service (Formance/Blnk for audit)
```

**Pros:**

- Best of both worlds
- High performance for trading operations
- External audit/reconciliation capabilities
- Flexible architecture

**Cons:**

- Most complex implementation
- Requires maintaining two systems
- Sync complexity

---

## Recommendations

### Recommended Approach: Ledger CLI Core Integration or Custom Lightweight Ledger Module

**Rationale:**

1. **Performance Requirements:**
   - Trading operations require low-latency transaction recording
   - External API calls add unnecessary overhead
   - Real-time position updates need fast ledger access

2. **Architecture Fit:**
   - **Ledger CLI:** C++ codebase aligns perfectly with existing C++ core
   - Can embed Ledger CLI core as library or integrate directly
   - No external dependencies for critical path operations
   - Plain text format allows human-readable audit trail

3. **Trading-Specific Needs:**
   - Need double-entry accounting optimized for trading operations
   - Multi-currency support (ILS → USD conversion)
   - Position-based accounting (not just cash flows)
   - Integration with box spread strategy calculations
   - Investment position tracking (Ledger CLI supports this)

4. **Reference Architecture:**
   - **Ledger CLI:** Extract C++ core, embed in IB box spread system
   - **Formance/Blnk:** Use as design reference for modern patterns
   - Learn from their double-entry transaction models
   - Adopt their reconciliation patterns for external validation

**NEW DISCOVERY: Ledger CLI is highly applicable:**

- ✅ **C++ codebase** matches existing architecture perfectly
- ✅ **Plain text format** provides human-readable audit trail
- ✅ **Investment tracking** designed for securities/portfolio accounting
- ✅ **No database dependency** keeps it lightweight
- ✅ **20+ years development** with proven double-entry logic
- ✅ **Can be embedded as library** or integrated directly into C++ core

### Implementation Strategy

**Option A: Ledger CLI Core Integration (Recommended)**

**Phase 1: Ledger CLI Integration**

- Extract or integrate Ledger CLI C++ core library
- Build C++ API wrapper for transaction recording
- Integrate with existing C++ box spread calculations
- Create `.ledger` file format for transaction storage

**Phase 2: Transaction Recording**

- Record all box spread transactions to ledger
- Record position changes (buys/sells)
- Record cash flows (deposits/withdrawals)
- Multi-currency transaction support (ILS → USD)

**Phase 3: Reporting Integration**

- Use Ledger CLI reporting commands for balance queries
- Generate financial reports from ledger files
- Integration with existing position tracking

**Phase 4: Advanced Features**

- Cash flow forecasting from ledger data
- Reconciliation with external sources (IBKR statements)
- Audit trail and compliance reporting
- Real-time balance updates (parse ledger files or embed core)

**Option B: Custom Lightweight Ledger Module**

**Phase 1: Core Ledger Module**

- Implement double-entry accounting in Rust (aligns with backend)
- Use Ledger CLI transaction format as reference
- Basic transaction journal
- Account/chart of accounts structure
- Balance calculation and reconciliation

**Phase 2: Integration**

- Integrate with existing position tracking
- Replace simple PnL calculation with ledger-based accounting
- Add transaction recording for all trading operations

**Phase 3: Advanced Features**

- Multi-currency support
- Cash flow tracking
- Reconciliation with external sources (IBKR statements)
- Audit trail and reporting

**Phase 4: External Validation (Optional)**

- Periodic sync with Formance/Blnk for audit
- External reconciliation service
- Compliance reporting

**Recommendation:** Start with **Option A (Ledger CLI Core Integration)** because:

1. C++ codebase matches existing architecture
2. Proven double-entry logic (20+ years)
3. Investment position tracking support
4. Human-readable text format for debugging
5. Can be embedded or used as reference implementation

### Key Design Principles

1. **Double-Entry Accounting:**
   - Every transaction has debit and credit
   - Accounts always balance
   - Immutable transaction journal

2. **Trading-Optimized:**
   - Fast transaction recording (< 1ms)
   - Position-based accounts
   - Real-time balance updates

3. **Multi-Currency:**
   - Support for USD, ILS, and other currencies
   - Currency conversion tracking
   - Exchange rate management

4. **Integration Points:**
   - TWS API transaction recording
   - Box spread strategy transaction recording
   - Cash flow forecasting integration
   - Portfolio allocation tracking

---

## Conclusion

While modern financial ledger platforms like Formance and Blnk offer excellent architectures and features, a custom lightweight ledger module integrated directly into the IB box spread system would provide:

- ✅ Better performance for trading operations
- ✅ Native integration with existing C++/Rust codebase
- ✅ Trading-optimized design
- ✅ No external dependencies for critical path

Use Formance and Blnk as **reference architectures** for:

- Double-entry transaction models
- Account structure patterns
- Reconciliation approaches
- API design principles

The investment in a custom ledger module will pay off through:

- Reduced latency for trading operations
- Full control over trading-specific features
- Direct integration with existing systems
- Optimized performance for real-time portfolio accounting

---

## References

1. **[Ledger CLI](https://ledger-cli.org/)** - Command-line double-entry accounting (C++)
2. **[Ledger CLI GitHub](https://github.com/ledger/ledger)** - C++ source code repository
3. **[Formance Platform Documentation](https://docs.formance.com/welcome)**
4. **[Blnk Finance Documentation](https://docs.blnkfinance.com/reference/create-ledger)**
5. **[Blnk Finance GitHub](https://github.com/blnkfinance/blnk)**
6. **[LedgerSMB](https://ledgersmb.org/)** - Open-source ERP with accounting
7. [Firefly III Documentation](https://docs.firefly-iii.org/)
8. [Akaunting Accounting Software](https://akaunting.com/)
9. [GnuCash Accounting Software](https://www.gnucash.org/)
10. [FINOS Foundation](https://community.finos.org/docs/collaboration-infrastructure/)
11. [ERPNext Open Source Accounting](https://frappe.io/erpnext/open-source-accounting)
12. Investment Strategy Framework: `docs/INVESTMENT_STRATEGY_FRAMEWORK.md`
13. Current System State: `agents/backend/crates/api/src/state.rs`

**Note:** LedgerHQ (`lib-ledger-core`) is a cryptocurrency hardware wallet library, not a financial accounting ledger. The repository was archived on Feb 12, 2025, and is not relevant for this research.

---

**Next Steps:**

1. Design lightweight ledger module architecture
2. Define double-entry transaction model for trading operations
3. Plan integration with existing position tracking
4. Implement core ledger functionality
5. Integrate with box spread strategy and portfolio allocation
