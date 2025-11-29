# IBM i CL and RPG Integration Analysis

**Date**: 2025-11-20
**Purpose**: Evaluate potential integration of IBM i CL/RPG with the IBKR Box Spread trading system

---

## Executive Summary

**Key Finding**: IBM i CL (Control Language) and RPG (Report Program Generator) are **IBM i platform-specific languages** (formerly AS/400). They cannot be used directly for this modern C++20/Python trading application, but **integration is possible** if you have existing IBM i infrastructure.

**Recommendation**:
- ❌ **Do NOT use CL/RPG for core trading system** - Platform/language mismatch
- ✅ **Consider integration** if you have IBM i systems that need to interface with trading data
- ✅ **Use REST APIs** for modern integration patterns

---

## What Are IBM i CL and RPG?

### IBM i Platform

**IBM i** (formerly AS/400, iSeries):
- **Platform**: IBM midrange system (not mainframe, not PC)
- **Operating System**: IBM i OS (proprietary)
- **Database**: DB2 for i (integrated)
- **Languages**: CL, RPG, COBOL, C, C++, Java, Python, Node.js

### Control Language (CL)

**CL** is:
- **Purpose**: System control and automation language
- **Use Cases**: Job scheduling, program calls, system commands
- **Syntax**: Command-based (similar to shell scripts)
- **Platform**: IBM i only

**Example CL Program**:
```cl
PGM
  DCL VAR(&ACCOUNT) TYPE(*CHAR) LEN(10)
  DCL VAR(&BALANCE) TYPE(*DEC) LEN(15 2)

  CALL PGM(TRADING/GET_BALANCE) PARM(&ACCOUNT &BALANCE)

  IF COND(&BALANCE *GT 10000) THEN(DO)
    CALL PGM(TRADING/PLACE_ORDER)
  ENDDO
ENDPGM
```

### RPG (Report Program Generator)

**RPG** is:
- **Purpose**: Business application programming
- **Use Cases**: Financial systems, ERP, manufacturing, distribution
- **Syntax**: Originally fixed-format, modern RPG IV is free-format
- **Platform**: IBM i only
- **Market**: Powers many manufacturing and distribution systems

**Example RPG IV (Free-Format)**:
```rpg
**free
dcl-proc GetTradingBalance;
  dcl-pi *n dec(15:2);
    account char(10);
  end-pi;

  dcl-s balance dec(15:2);

  exec sql select balance into :balance
           from trading_accounts
           where account_id = :account;

  return balance;
end-proc;
```

---

## Why CL/RPG Cannot Be Used Directly

### 1. Platform Incompatibility

**IBM i Requirements**:
- IBM i hardware (Power Systems)
- IBM i operating system
- IBM i development tools (RDi, ACS)

**Your Trading System**:
- Runs on macOS/Linux (modern Unix)
- Uses CMake, C++20, Python
- No IBM i infrastructure

### 2. Language Mismatch

**CL/RPG Characteristics**:
- Platform-specific languages
- Cannot compile on Unix/macOS/Linux
- Require IBM i runtime environment
- No direct C++/Python interop

**Your Trading System**:
- C++20 with modern libraries (FTXUI, spdlog, nlohmann/json)
- Python with modern frameworks (Textual, FastAPI)
- Cross-platform compatibility required

### 3. Architecture Mismatch

**IBM i Architecture**:
- Integrated database (DB2 for i)
- Native file system
- Job-based execution model
- Green-screen interfaces (5250)

**Your Trading System**:
- REST APIs (FastAPI services)
- Time-series database (QuestDB)
- Real-time market data (TWS, ORATS)
- Modern TUI/Web interfaces

---

## Integration Possibilities

### Scenario 1: You Have Existing IBM i Infrastructure

**If you have IBM i systems that need trading data:**

#### Option A: REST API Integration ✅

**Modern Approach**: Expose IBM i data via REST APIs

**Tools**:
- **Rest4i**: Generate REST APIs from RPG code
- **CGIDEV2**: Web interfaces for IBM i
- **IBM i HTTP Server**: Native web server

**Integration Pattern**:
```
IBM i System (RPG/CL)
    ↓ (REST API)
Python FastAPI Service
    ↓ (REST API)
Trading System
```

**Example Use Case**:
- IBM i system tracks account balances
- Expose via REST API: `GET /api/ibmi/account/{id}/balance`
- Trading system consumes this data

#### Option B: Database Integration ✅

**Approach**: Direct database access to DB2 for i

**Tools**:
- **ibm_db** (Python): DB2 for i driver
- **ODBC**: Standard database connectivity
- **JDBC**: Java database connectivity

**Integration Pattern**:
```
IBM i System (DB2 for i)
    ↓ (ODBC/JDBC)
Python Service
    ↓ (REST API)
Trading System
```

**Example Use Case**:
- IBM i stores historical trading data
- Python service queries DB2 for i
- Provides data to trading system

#### Option C: Message Queue Integration ✅

**Approach**: Use IBM MQ or other message queues

**Tools**:
- **IBM MQ**: Enterprise message queuing
- **NATS**: Modern message queue (your system uses this)

**Integration Pattern**:
```
IBM i System (RPG)
    ↓ (MQ Messages)
NATS Message Queue
    ↓ (NATS)
Trading System
```

**Example Use Case**:
- IBM i publishes trade confirmations
- Trading system subscribes via NATS
- Real-time integration

### Scenario 2: You Don't Have IBM i Infrastructure

**If you don't have IBM i systems:**

**Recommendation**: ❌ **Do NOT add IBM i infrastructure**

**Why**:
- Significant cost (hardware, software, licensing)
- Complexity (specialized skills required)
- No clear benefit for modern trading system
- Better alternatives available (PostgreSQL, QuestDB, etc.)

---

## Modern Alternatives to IBM i CL/RPG

### For System Automation (CL Alternative)

**Instead of CL**, use:
- **Python scripts**: `scripts/` directory
- **Shell scripts**: Bash/Zsh automation
- **CMake**: Build automation
- **GitHub Actions**: CI/CD automation

**Example** (Your codebase):
```bash
# scripts/build_universal.sh
# Modern alternative to CL job scheduling
```

### For Business Logic (RPG Alternative)

**Instead of RPG**, use:
- **C++**: Core trading logic (`native/src/`)
- **Python**: Business logic (`python/integration/`)
- **Rust**: Backend services (`agents/backend/`)

**Example** (Your codebase):
```cpp
// native/src/box_spread_strategy.cpp
// Modern C++20 alternative to RPG business logic
```

### For Database (DB2 for i Alternative)

**Instead of DB2 for i**, use:
- **QuestDB**: Time-series data (already in use)
- **PostgreSQL**: Relational data
- **SQLite**: Embedded database

**Example** (Your codebase):
```python
# python/integration/questdb_client.py
# Modern time-series database alternative
```

---

## Integration Architecture (If You Have IBM i)

### Recommended Integration Pattern

```
┌─────────────────────────────────────────────────┐
│           IBM i System (Existing)                │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐    │
│  │   RPG    │  │    CL    │  │  DB2 for i│    │
│  │ Programs │  │  Scripts │  │  Database │    │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘    │
│       │             │              │           │
│       └─────────────┼──────────────┘           │
│                     │                          │
│              ┌─────▼─────┐                    │
│              │  REST API  │                    │
│              │  (Rest4i)  │                    │
│              └─────┬─────┘                    │
└────────────────────┼──────────────────────────┘
                     │
                     │ HTTP/REST
                     │
┌────────────────────▼──────────────────────────┐
│        Trading System (Your Project)            │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐    │
│  │  Python  │  │   C++    │  │  Rust    │    │
│  │ FastAPI  │  │  Trading │  │ Backend  │    │
│  │ Service  │  │  Engine  │  │ Services │    │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘    │
│       │             │              │           │
│       └─────────────┼──────────────┘           │
│                     │                          │
│              ┌─────▼─────┐                    │
│              │   NATS    │                    │
│              │  Message  │                    │
│              │   Queue   │                    │
│              └───────────┘                    │
└─────────────────────────────────────────────────┘
```

### Implementation Steps (If Needed)

1. **Expose IBM i Data via REST API**
   - Use Rest4i or CGIDEV2
   - Create REST endpoints for trading data
   - Document API endpoints

2. **Create Python Integration Service**
   - Add `python/integration/ibmi_client.py`
   - Consume IBM i REST APIs
   - Transform data to trading system format

3. **Integrate with Trading System**
   - Add IBM i data provider
   - Update TUI/Web to show IBM i data
   - Add configuration for IBM i endpoints

---

## Codebase Analysis

### Current State

**IBM i References Found**:
- `docs/LEGACY_FINANCIAL_SYSTEMS.md` - Documents RPG/COBOL for reference
- `.vscode/extensions.json` - **Explicitly excludes IBM i extensions**
- No IBM i integration code found

**Extension Configuration**:
```json
// .vscode/extensions.json
"unwantedRecommendations": [
  "barrettotte.ibmi-languages",
  "halcyontechltd.code-for-ibmi",
  "halcyontechltd.vscode-displayfile",
  "halcyontechltd.vscode-ibmi-walkthroughs",
  "ibm.vscode-ibmi-projectexplorer"
]
```

**Conclusion**: Project explicitly excludes IBM i tooling - no current integration planned.

---

## Recommendations

### If You Have IBM i Infrastructure

**✅ Consider Integration**:
1. Expose IBM i data via REST APIs (Rest4i)
2. Create Python integration service
3. Integrate with existing trading system architecture
4. Use NATS for real-time messaging (if needed)

**Implementation Priority**: Low (only if you have IBM i systems)

### If You Don't Have IBM i Infrastructure

**❌ Do NOT Add IBM i**:
- Significant cost and complexity
- No clear benefit for trading system
- Modern alternatives are better suited
- Focus on current architecture (C++/Python/Rust)

---

## Conclusion

**IBM i CL and RPG are platform-specific languages** that cannot be used directly for your modern trading system. However, **integration is possible** if you have existing IBM i infrastructure.

**Key Points**:
1. ❌ CL/RPG cannot replace C++/Python in your trading system
2. ✅ REST API integration is possible (if you have IBM i)
3. ✅ Database integration is possible (DB2 for i → Python)
4. ✅ Message queue integration is possible (IBM MQ → NATS)
5. ❌ Do NOT add IBM i infrastructure if you don't already have it

**Recommendation**: Continue with current architecture (C++/Python/Rust). Only consider IBM i integration if you have existing IBM i systems that need to interface with trading data.

---

## References

- [IBM i Documentation](https://www.ibm.com/docs/en/i)
- [Rest4i - RPG REST API Framework](https://rest4i.com/)
- [CGIDEV2 - Web Interfaces for IBM i](https://en.wikipedia.org/wiki/Cgidev2)
- [IBM i HTTP Server](https://www.ibm.com/docs/en/i/7.4?topic=server-http)
- [Your Codebase: Legacy Financial Systems](../../LEGACY_FINANCIAL_SYSTEMS.md)

---

**Last Updated**: 2025-11-20
**Status**: Analysis Complete ✅
