# Go Market Data Ingestion Gateway Design

**Date**: 2025-11-17
**Status**: Design Document
**Purpose**: Design specification for Go-based market data ingestion gateway (Agent TODO #1)

---

## Overview

Design a high-performance Go service for ingesting market data from multiple sources and storing it in QuestDB for time-series analysis.

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│              Market Data Sources                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │ TWS Client  │  │ Alpaca API   │  │ ORATS API    │  │
│  └──────┬──────┘  └──────┬───────┘  └──────┬───────┘  │
└─────────┼─────────────────┼─────────────────┼──────────┘
          │                 │                 │
┌─────────┼─────────────────┼─────────────────┼──────────┐
│         │  Go Market Data Gateway                       │
│  ┌──────▼────────────────────────────────────────────┐  │
│  │  Data Ingestion Service                           │  │
│  │  - Receive from multiple sources                  │  │
│  │  - Normalize data format                          │  │
│  │  - Validate data quality                          │  │
│  │  - Route to consumers                             │  │
│  └──────┬────────────────────────────────────────────┘  │
│         │                                                 │
│  ┌──────▼────────────────────────────────────────────┐  │
│  │  QuestDB Writer                                   │  │
│  │  - Batch writes                                   │  │
│  │  - ILP protocol                                   │  │
│  │  - Error handling                                 │  │
│  └──────┬────────────────────────────────────────────┘  │
└─────────┼────────────────────────────────────────────────┘
          │
┌─────────▼──────────────────────────────────────────────┐
│              QuestDB                                    │
│  - Time-series storage                                  │
│  - Historical data                                      │
│  - Query interface                                      │
└─────────────────────────────────────────────────────────┘
```

---

## Component Design

### 1. Data Ingestion Service

**Responsibilities:**

- Accept market data from multiple sources
- Normalize data format
- Validate data quality
- Route to consumers (QuestDB, real-time subscribers)

**Interfaces:**

```go
type MarketDataEvent struct {
    Symbol      string
    Timestamp   time.Time
    Bid         float64
    Ask         float64
    Last        float64
    Volume      int64
    Source      string  // "TWS", "Alpaca", "ORATS"
}

type IngestionService interface {
    Ingest(event MarketDataEvent) error
    Subscribe(callback func(MarketDataEvent)) error
    Start() error
    Stop() error
}
```

---

### 2. QuestDB Writer

**Responsibilities:**

- Batch market data events
- Write to QuestDB via ILP (Influx Line Protocol)
- Handle write errors
- Retry failed writes

**Implementation:**

```go
type QuestDBWriter struct {
    client    *questdb.Client
    batchSize int
    buffer    []MarketDataEvent
    mutex     sync.Mutex
}

func (w *QuestDBWriter) Write(event MarketDataEvent) error {
    // Add to buffer
    // Flush when buffer full
    // Write via ILP
}
```

---

### 3. Data Source Adapters

**TWS Adapter:**

- Receive data from TWS client (via gRPC or file)
- Convert to normalized format
- Send to ingestion service

**Alpaca Adapter:**

- Connect to Alpaca WebSocket/API
- Receive market data
- Convert to normalized format
- Send to ingestion service

**ORATS Adapter:**

- Connect to ORATS API
- Receive option chain data
- Convert to normalized format
- Send to ingestion service

---

## Data Flow

```
1. Market Data Source → Adapter
2. Adapter → Normalize Format
3. Normalized Data → Ingestion Service
4. Ingestion Service → Validate
5. Validated Data → QuestDB Writer
6. QuestDB Writer → Batch & Write
7. QuestDB Writer → QuestDB (ILP)
```

---

## Implementation Details

### Go Module Structure

```
agents/go-market-data/
├── main.go              # Service entry point
├── go.mod               # Go module
├── internal/
│   ├── gateway/
│   │   └── gateway.go  # Main gateway service
│   ├── questdb/
│   │   └── writer.go    # QuestDB writer
│   ├── adapters/
│   │   ├── tws.go      # TWS adapter
│   │   ├── alpaca.go   # Alpaca adapter
│   │   └── orats.go    # ORATS adapter
│   └── models/
│       └── events.go   # Data models
└── proto/
    └── market_data.proto # gRPC definitions
```

---

### Configuration

```go
type Config struct {
    QuestDB struct {
        Host     string
        Port     int
        Database string
    }
    Sources struct {
        TWS    bool
        Alpaca bool
        ORATS  bool
    }
    BatchSize int
    FlushInterval time.Duration
}
```

---

### Performance Considerations

**Concurrency:**

- Use goroutines for each data source
- Channel-based communication
- Worker pool for QuestDB writes

**Batching:**

- Batch writes to QuestDB (100-1000 events)
- Flush on interval (1-5 seconds)
- Flush on buffer full

**Error Handling:**

- Retry failed writes with exponential backoff
- Dead letter queue for persistent failures
- Monitoring and alerting

---

## gRPC Interface

**Service Definition:**

```protobuf
service MarketDataGateway {
  rpc StreamMarketData(StreamRequest) returns (stream MarketDataEvent);
  rpc IngestMarketData(MarketDataEvent) returns (IngestResponse);
  rpc GetHealth(HealthRequest) returns (HealthResponse);
}
```

**Usage:**

- Rust backend can stream data to gateway
- Other services can subscribe to data stream
- Health check for monitoring

---

## Integration Points

**With Existing Services:**

- Rust backend: gRPC client → gateway
- Python services: gRPC client → gateway
- C++ TWS client: File or gRPC → gateway

**QuestDB Integration:**

- Use QuestDB Go client library
- ILP protocol for ingestion
- SQL for queries (separate service)

---

## Implementation Steps

1. **Setup Go Module**
   - Create `agents/go-market-data/` directory
   - Initialize Go module
   - Add dependencies (QuestDB client, gRPC)

2. **Implement Core Gateway**
   - Create gateway service
   - Implement ingestion interface
   - Add data validation

3. **Implement QuestDB Writer**
   - Create QuestDB writer
   - Implement batching
   - Add error handling

4. **Implement Adapters**
   - TWS adapter
   - Alpaca adapter
   - ORATS adapter

5. **Add gRPC Interface**
   - Define protobuf schema
   - Generate Go code
   - Implement service

6. **Testing**
   - Unit tests
   - Integration tests
   - Performance tests

---

## Success Criteria

- [ ] Gateway service runs and accepts data
- [ ] QuestDB integration works
- [ ] Multiple data sources supported
- [ ] High throughput (> 10,000 events/sec)
- [ ] Low latency (< 10ms processing)
- [ ] Error handling robust
- [ ] Health check endpoint
- [ ] Monitoring and metrics

---

## Dependencies

**Go Packages:**

- `github.com/questdb/go-questdb` - QuestDB client
- `google.golang.org/grpc` - gRPC framework
- `google.golang.org/protobuf` - Protocol Buffers

**External:**

- QuestDB server (running)
- Data source APIs (TWS, Alpaca, ORATS)

---

**Document Status**: ✅ Complete - Design specification ready for implementation
