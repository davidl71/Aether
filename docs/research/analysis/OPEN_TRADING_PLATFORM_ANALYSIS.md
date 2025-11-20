# Open Trading Platform (OTP) Analysis

**Date**: 2025-01-27
**Status**: Research Complete
**Related Task**: T-6

---

## Executive Summary

The Open Trading Platform (OTP) is an open-source, highly scalable microservices-based trading platform designed for cross-asset execution-oriented applications. This analysis compares OTP's Kubernetes-based architecture with our current multi-language agent-based approach to identify potential integration opportunities, architectural insights, and learning opportunities.

**Key Finding**: While OTP is designed for enterprise-scale multi-tenant deployments and our project focuses on single-user box spread trading, OTP's microservices patterns, order management architecture, and Kafka-based event distribution offer valuable insights for scaling our platform.

---

## OTP Overview

### Project Details

- **Language**: Go (25.4%), TypeScript (22.7%), JavaScript (32.5%), Java (16.1%)
- **License**: GPL-3.0
- **Stars**: 159
- **Purpose**: Enterprise-scale execution-oriented trading platform
- **Deployment**: Kubernetes (on-prem or cloud)

### Key Features

1. **Microservices Architecture**
   - Kubernetes-managed services
   - Service mesh communication
   - Independent scaling per service

2. **Order Management**
   - Kafka-based order distribution
   - Full order history via Kafka log
   - State-by-state order tracking

3. **Market Data Distribution**
   - Real-time quote aggregation
   - Multi-venue market data gateway
   - FIX protocol support

4. **Strategy Execution**
   - VWAP strategy example
   - Smart routing
   - Order router service

### Technology Stack

**Backend**:

- **Go**: Server-side services (core in `otp-common` module)
- **Kubernetes**: Container orchestration
- **Kafka**: Distributed order/execution log
- **gRPC**: Inter-service communication
- **Protobuf**: Domain model and API definitions

**Frontend**:

- **React + TypeScript**: Single-page application
- **Envoy/grpc-web**: Gateway for client-server communication
- **Ag Grid**: Trading grid components
- **Caplin Flexlayout**: GUI layout manager
- **BlueprintJS**: Data-dense UI components

**Infrastructure**:

- **Helm**: Deployment automation
- **Prometheus**: Metrics collection
- **Grafana**: Monitoring dashboards
- **PostgreSQL**: Static data storage
- **QuickFixGo/QuickFixJ**: FIX protocol engines

---

## Current Project Architecture

### Technology Stack

- **Core**: C++20 (native performance)
- **Agents**: Rust (backend services), Go (some agents), Python (integration), TypeScript (web)
- **Build System**: CMake (C++), Cargo (Rust), npm (TypeScript)
- **Testing**: Catch2 (C++), Rust tests, Python pytest

### Current Architecture

**Multi-Agent System**:

```
agents/
  ├── backend/              # Rust: Market data, strategy, risk
  ├── backend-market-data/  # Rust: Market data service
  ├── backend-data/         # Data aggregation
  ├── backend-mock/         # Mock services
  ├── tui/                  # C++ TUI
  ├── web/                  # TypeScript React
  ├── ipad/                 # SwiftUI
  └── desktop/              # Swift
```

**Communication**:

- **REST API**: HTTP endpoints for clients
- **gRPC**: Service-to-service (Rust services)
- **WebSocket**: Real-time updates (planned)
- **Shared State**: In-memory snapshots

**Data Storage**:

- **QuestDB**: Time-series data (quotes, trades)
- **In-Memory**: Current state snapshots
- **JSON Config**: Configuration files

---

## Comparison Analysis

### Architecture Pattern

| Aspect                | OTP                        | IBKR Box Spread Project     |
| --------------------- | -------------------------- | --------------------------- |
| **Pattern**           | Microservices (Kubernetes) | Multi-Agent (Process-based) |
| **Orchestration**     | Kubernetes                 | Manual/scripts              |
| **Scaling**           | Horizontal (K8s pods)      | Vertical (single process)   |
| **Deployment**        | Helm charts                | CMake/Cargo builds          |
| **Service Discovery** | K8s DNS                    | Static configuration        |
| **Target Scale**      | Enterprise multi-tenant    | Single-user                 |

**Verdict**: OTP targets enterprise scale with multi-tenant support. Our project is optimized for single-user, low-latency trading. Different scales, different approaches.

### Order Management

#### OTP Approach

**Kafka-Based Order Log**:

- All orders written to Kafka topics
- Full order history preserved
- State-by-state tracking
- Distributed order store

**Architecture**:

```
Order Router → Kafka Topic → Order Data Service
                              ↓
                         Order History (full log)
```

**Benefits**:

- Complete audit trail
- Easy to replay orders
- Scalable (Kafka partitions)
- Multiple consumers can process orders

#### Our Current Approach

**In-Memory Order Tracking**:

- Orders tracked in memory
- State snapshots via REST API
- QuestDB for historical data
- No distributed log

**Architecture**:

```
Order Manager → In-Memory State → REST API
                              ↓
                         QuestDB (historical)
```

**Benefits**:

- Low latency (no network overhead)
- Simple for single-user
- Fast access

**Drawbacks**:

- No distributed order log
- Limited scalability
- No order replay capability

### Market Data Distribution

#### OTP Approach

**Quote Aggregator Service**:

- Centralized quote aggregation
- Multi-venue support
- FIX market data gateway
- gRPC streaming to services

**Architecture**:

```
Market Data Gateway → Quote Aggregator → gRPC Stream
                                          ↓
                                    Services (subscribers)
```

**Benefits**:

- Centralized distribution
- Efficient fan-out
- Multi-venue support
- Service isolation

#### Our Current Approach

**Provider Abstraction**:

- Multiple providers (REST, WebSocket, Mock)
- Direct provider access
- No centralized aggregator
- REST polling + WebSocket push

**Architecture**:

```
Market Data Sources → Provider Abstraction → TUI/Web/iPad
                     (REST, WebSocket, Mock)
```

**Benefits**:

- Simple for single-user
- Direct access (low latency)
- Flexible provider switching

**Drawbacks**:

- No centralized aggregation
- Each client polls independently
- Limited fan-out efficiency

### Service Communication

#### OTP Approach

**gRPC + Protobuf**:

- Strongly typed APIs
- Binary protocol (efficient)
- Streaming support
- Cross-language (Go, Java, etc.)

**Service Mesh**:

- Kubernetes service discovery
- Load balancing
- Health checks
- Circuit breakers

**Example**:

```go
// OTP service communication
client := grpc.NewClient("order-data-service:50051")
order, err := client.GetOrder(ctx, orderId)
```

#### Our Current Approach

**REST + gRPC Hybrid**:

- REST for client access (TUI, web, iPad)
- gRPC for service-to-service (Rust agents)
- JSON for REST, Protobuf for gRPC
- Static service addresses

**Example**:

```rust
// Our service communication
let client = BackendClient::connect("http://127.0.0.1:8080").await?;
let snapshot = client.get_snapshot().await?;
```

**Comparison**:

- **OTP**: Enterprise-grade service mesh
- **Our Project**: Simpler, single-machine deployment
- **Both**: Use gRPC for efficient communication

### Data Persistence

#### OTP Approach

**Kafka + PostgreSQL**:

- **Kafka**: Order/execution log (append-only)
- **PostgreSQL**: Static data (instruments, markets)
- **Full History**: Complete order lifecycle in Kafka

**Benefits**:

- Complete audit trail
- Replay capability
- Scalable storage
- Time-travel queries

#### Our Current Approach

**QuestDB + In-Memory**:

- **QuestDB**: Time-series data (quotes, trades)
- **In-Memory**: Current state snapshots
- **JSON Config**: Static configuration

**Benefits**:

- Fast time-series queries
- Low latency access
- Simple for single-user

**Drawbacks**:

- No complete order log
- Limited replay capability

### Frontend Architecture

#### OTP Approach

**React + TypeScript SPA**:

- Single-page application
- Ag Grid for trading grids
- Caplin Flexlayout for layouts
- BlueprintJS for components
- Envoy/grpc-web gateway

**Features**:

- Professional trading UI
- Real-time updates via gRPC streams
- Order blotter with history
- Market data visualization

#### Our Current Approach

**Multi-Platform**:

- **Web**: React + TypeScript (similar to OTP)
- **TUI**: C++ FTXUI
- **iPad**: SwiftUI
- **Desktop**: Swift

**Features**:

- Platform-native UIs
- Shared REST API backend
- Real-time updates (WebSocket planned)

**Comparison**:

- **OTP**: Single web UI (enterprise focus)
- **Our Project**: Multi-platform (user choice)
- **Both**: React for web, real-time updates

---

## Architectural Insights & Learning Opportunities

### 1. Kafka for Order History

**OTP Pattern**:

```go
// All orders written to Kafka
producer.Send("orders", orderEvent)
// Full history available via Kafka log
```

**Potential Application**:
We could add Kafka for order history:

- Complete audit trail
- Order replay capability
- Better debugging
- Compliance requirements

**Trade-offs**:

- Adds complexity (Kafka cluster)
- Overkill for single-user?
- But valuable for production trading

### 2. Quote Aggregator Service

**OTP Pattern**:

- Centralized quote aggregation
- Single source of truth
- Efficient fan-out to services

**Potential Application**:
We could add a quote aggregator:

- Reduce duplicate market data requests
- Centralized quote normalization
- Better caching

**Trade-offs**:

- Adds service complexity
- But improves efficiency for multiple clients

### 3. Service Mesh Patterns

**OTP Pattern**:

- Kubernetes service discovery
- Health checks
- Circuit breakers
- Load balancing

**Potential Application**:
We could adopt service mesh patterns:

- Health check endpoints
- Circuit breakers for external APIs
- Service discovery (even if static)

**Trade-offs**:

- No need for full K8s
- But patterns are valuable

### 4. Protobuf Domain Model

**OTP Pattern**:

- Shared Protobuf definitions
- Cross-language compatibility
- Versioned APIs

**Potential Application**:
We already use Protobuf for gRPC, but could:

- Expand Protobuf usage
- Share domain models across languages
- Better API versioning

**Trade-offs**:

- Already partially adopted
- Could expand further

### 5. Order State Machine

**OTP Pattern**:

- Explicit order state transitions
- State-by-state tracking
- Full lifecycle history

**Potential Application**:
We could improve order state tracking:

- Explicit state machine
- Better state validation
- Complete transition history

**Trade-offs**:

- Adds complexity
- But improves reliability

---

## Integration Opportunities

### Option 1: Adopt Patterns (Recommended)

**Action**: Study OTP's patterns and apply similar approaches to our architecture.

**Specific Patterns**:

1. **Order History Log**: Add Kafka or similar for order history
2. **Quote Aggregator**: Centralize market data distribution
3. **Service Health Checks**: Add health check endpoints
4. **Circuit Breakers**: Add for external API calls
5. **State Machine**: Improve order state tracking

**Benefits**:

- Keep our single-user focus
- Improve reliability and observability
- Better production readiness
- No Kubernetes dependency

**Effort**: Medium (2-4 weeks per pattern)

### Option 2: Hybrid Approach

**Action**: Use OTP components for specific services (e.g., order management).

**Benefits**:

- Leverage OTP's order management
- Keep our C++ core for trading logic
- Best of both worlds

**Challenges**:

- Language mismatch (Go vs C++/Rust)
- Integration complexity
- License compatibility (GPL-3.0)

**Effort**: High (4-8 weeks)

### Option 3: Full Migration (Not Recommended)

**Action**: Migrate entire project to OTP architecture.

**Why Not Recommended**:

- Different scale requirements (enterprise vs single-user)
- Over-engineering for our use case
- Loss of C++ performance advantages
- Kubernetes overhead for single-user
- License incompatibility concerns

---

## Recommendations

### Short-Term (1-3 months)

1. **Add Order History Log**
   - Implement Kafka or file-based order log
   - Complete audit trail
   - Order replay capability

2. **Improve Order State Machine**
   - Explicit state transitions
   - State validation
   - Better error handling

3. **Add Health Check Endpoints**
   - Service health monitoring
   - Dependency status
   - Better observability

### Medium-Term (3-6 months)

1. **Quote Aggregator Service**
   - Centralize market data distribution
   - Reduce duplicate requests
   - Better caching

2. **Circuit Breakers**
   - Add for external APIs (TWS, Alpaca)
   - Better resilience
   - Graceful degradation

3. **Expand Protobuf Usage**
   - Share domain models across languages
   - Better API versioning
   - Type safety

### Long-Term (6+ months)

1. **Kubernetes Deployment** (Optional)
   - If scaling to multiple users
   - Better resource management
   - Service orchestration

2. **Service Mesh** (Optional)
   - If adopting microservices
   - Better service communication
   - Observability

---

## Key Takeaways

1. **Different Scales**: OTP is enterprise-scale, our project is single-user - complementary but different
2. **Kafka for Orders**: Valuable pattern for order history and audit trails
3. **Service Patterns**: Health checks, circuit breakers valuable even without K8s
4. **Quote Aggregation**: Centralized distribution improves efficiency
5. **Protobuf**: Already using, could expand further

---

## References

- **OTP GitHub**: <https://github.com/ettec/open-trading-platform>
- **OTP Common Module**: <https://github.com/ettec/otp-common>
- **GPL-3.0 License**: Note - incompatible with our license if direct integration
- **Kafka Documentation**: <https://kafka.apache.org/documentation/>
- **gRPC Documentation**: <https://grpc.io/docs/>

---

## Related Documentation

- [StockSharp Analysis](STOCKSHARP_ANALYSIS.md) - C# trading platform comparison
- [Ticker TUI Analysis](TICKER_TUI_ANALYSIS.md) - Terminal UI comparison
- [API Documentation Index](API_DOCUMENTATION_INDEX.md) - Complete API reference
- [Codebase Architecture](CODEBASE_ARCHITECTURE.md) - Current project architecture

---

**Last Updated**: 2025-01-27
**Next Review**: When implementing order history log or quote aggregator
