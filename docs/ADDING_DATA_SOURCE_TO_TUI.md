# Adding a New Data Source to the TUI

A step-by-step guide for wiring a new data source (positions, market data, or health) into the TUI.

## Overview

There are three types of data sources you might add:

1. **Market Data Source** - Real-time quotes/prices (e.g., Alpaca, Polygon, Yahoo)
   - Use `MarketDataSource` trait + `MarketDataSourceFactory`
   - Register in `provider_registry()` via `create_provider()`
2. **Options Data Source** - Option chains, greeks (e.g., Yahoo, Polygon)
   - Use `OptionsDataSource` trait
   - Register in `options_registry()` via `create_options_provider()`
3. **Position Source** - Account positions and balances (e.g., Alpaca positions, IB positions)
4. **Health Monitor** - API connectivity status (e.g., Alpaca health checks)

Each follows a similar pattern but touches different files.

---

## Pattern: Background Task → UI Communication

All data sources use the same communication pattern:

```
Background Task ──event_tx──► AppEvent ──apply_event()──► App State ──render()──► UI
```

Key components:
- `AppEvent` enum in `events.rs` - defines all possible UI events
- `event_tx: tokio::sync::mpsc::UnboundedSender<AppEvent>` - sender from background tasks
- `apply_event()` in `app_updates.rs` - handles events and updates App state
- `render()` methods in `ui/` - display the updated state

---

## Type 1: Adding a Position Source

### Files to modify:

1. **`api/src/{source}_positions.rs`** - Create position source
   ```rust
   pub struct AlpacaPositionSource { is_paper: bool }
   
   impl AlpacaPositionSource {
       pub fn from_env() -> Option<Self> { /* check credentials */ }
       pub fn from_paper(is_paper: bool) -> Option<Self> { /* credential lookup */ }
       pub async fn fetch_positions(&self) -> Result<Vec<Position>, Error> { /* API call */ }
       pub async fn fetch_account(&self) -> Result<AccountInfo, Error> { /* API call */ }
   }
   ```

2. **`api/src/lib.rs`** - Export the new source
   ```rust
   pub use alpaca_positions::{AlpacaPositionSource, AlpacaAccountInfo};
   ```

3. **`services/tui_service/src/app.rs`** - Add state fields
   ```rust
   pub alpaca_positions: Vec<AlpacaPosition>,
   pub alpaca_account: Option<AlpacaAccountInfo>,
   ```

4. **`services/tui_service/src/events.rs`** - Add event variant
   ```rust
   pub enum AppEvent {
       // ... existing variants
       AlpacaPositionsUpdate(Vec<AlpacaPosition>),
       AlpacaAccountUpdate(AlpacaAccountInfo),
   }
   ```

5. **`services/tui_service/src/app_updates.rs`** - Handle events
   ```rust
   pub fn apply_event(app: &mut App, event: AppEvent) {
       match event {
           // ... existing handlers
           AppEvent::AlpacaPositionsUpdate(positions) => {
               app.alpaca_positions = positions;
           }
           AppEvent::AlpacaAccountUpdate(account) => {
               app.alpaca_account = Some(account);
           }
       }
   }
   ```

6. **`services/tui_service/src/main.rs`** - Spawn background task
   ```rust
   let event_tx_clone = event_tx.clone();
   tokio::spawn(async move {
       run_alpaca_position_fetcher(event_tx_clone).await;
   });
   ```

7. **Create fetcher function** (e.g., in new `alpaca_fetcher.rs`)
   ```rust
   pub async fn run_alpaca_position_fetcher(event_tx: UnboundedSender<AppEvent>) {
       let source = match AlpacaPositionSource::from_env() {
           Some(s) => s,
           None => {
               tracing::warn!("Alpaca credentials not configured");
               return;
           }
       };
       
       let mut interval = tokio::time::interval(Duration::from_secs(30));
       loop {
           interval.tick().await;
           
           match source.fetch_positions().await {
               Ok(positions) => {
                   let _ = event_tx.send(AppEvent::AlpacaPositionsUpdate(positions));
               }
               Err(e) => {
                   tracing::error!("Failed to fetch Alpaca positions: {}", e);
               }
           }
       }
   }
   ```

---

## Type 2: Adding a Market Data Source

### Files to modify:

1. **`crates/market_data/src/{source}.rs`** - Implement `MarketDataSource`
   ```rust
   #[async_trait]
   impl MarketDataSource for AlpacaSource {
       async fn next(&self) -> anyhow::Result<MarketDataEvent> {
           // Return next market data event
           // Can use WebSocket, polling, or HTTP requests
       }
   }
   
   pub struct AlpacaSourceFactory;
   impl MarketDataSourceFactory for AlpacaSourceFactory {
       fn name(&self) -> &'static str { "alpaca" }
       fn requires_config(&self) -> bool { true }
       fn create(&self, symbols: &[String], interval: Duration) -> Result<Box<dyn MarketDataSource>> {
           // Create and return source
       }
   }
   ```

2. **`crates/market_data/src/lib.rs`** - Register in provider registry
   ```rust
   pub fn provider_registry() -> &'static HashMap<&'static str, DynFactory> {
       static REGISTRY: OnceLock<HashMap<&'static str, DynFactory>> = OnceLock::new();
       REGISTRY.get_or_init(|| {
           let mut m = HashMap::new();
           register(&mut m, "yahoo", YahooFinanceSourceFactory);
           register(&mut m, "alpaca", AlpacaSourceFactory);  // <-- add this
           m
       })
   }
   ```

3. **`services/tui_service/src/ui/settings_sources.rs`** - Add to settings display
   ```rust
   let sources = [
       // ... existing sources
       SourceDef {
           name: "alpaca_paper",
           priority: "55",
           cred_key: "ALPACA_PAPER",
           note: "Alpaca paper trading",
       },
   ];
   ```

The market data source integrates with the existing `MarketDataAggregator` which feeds into the snapshot system automatically.

---

## Type 2b: Adding an Options Data Source

Options sources (for option chains, greeks) follow a similar pattern to Market Data but use a separate registry.

### Files to modify:

1. **`crates/market_data/src/{provider}.rs`** - Implement OptionsDataSource
    ```rust
    use async_trait::async_trait;
    use market_data::{OptionsDataSource, OptionContractData, OptionsExpiration};
    
    pub struct YahooOptionsSource { /* ... */ }
    
    impl YahooOptionsSource {
        pub fn new() -> Self { Self { /* ... */ } }
    }
    
    #[async_trait]
    impl OptionsDataSource for YahooOptionsSource {
        async fn get_expirations(&self, symbol: &str) -> anyhow::Result<Vec<i64>> {
            // Fetch available expiration dates
        }
        async fn get_chain(&self, symbol: &str, expiration_ts: i64) -> anyhow::Result<OptionsExpiration> {
            // Fetch option chain for expiration
        }
    }
    
    // Factory for registry
    pub struct YahooOptionsSourceFactory;
    impl YahooOptionsSourceFactory {
        pub fn new() -> Self { Self }
        pub fn create(&self) -> anyhow::Result<Box<dyn OptionsDataSource>> {
            Ok(Box::new(YahooOptionsSource::new()))
        }
    }
    ```

2. **`crates/market_data/src/lib.rs`** - Add exports and register
    ```rust
    // Add to exports
    pub use yahoo::{YahooOptionsSource, YahooOptionsSourceFactory};
    
    // Add factory to options_registry()
    pub fn options_registry() -> &'static HashMap<&'static str, DynOptionsFactory> {
        static REGISTRY: OnceLock<HashMap<&'static str, DynOptionsFactory>> = OnceLock::new();
        REGISTRY.get_or_init(|| {
            let mut m = HashMap::new();
            m.insert("yahoo", Box::new(|| {
                Ok(Box::new(YahooOptionsSource::new()) as Box<dyn OptionsDataSource>)
            }) as DynOptionsFactory);
            m
        })
    }
    
    // Helper function
    pub fn create_options_provider(name: &str) -> anyhow::Result<Box<dyn OptionsDataSource>> {
        let registry = options_registry();
        let factory = registry.get(name.to_lowercase().trim())
            .ok_or_else(|| anyhow::anyhow!("unknown options provider: {name}"))?;
        factory()
    }
    ```

**Usage:**
```rust
use market_data::{create_options_provider, OptionsDataSource};

let source = create_options_provider("yahoo")?;
let expirations = source.get_expirations("SPY").await?;
let chain = source.get_chain("SPY", expirations[0]).await?;
```

---

## Type 3: Adding a Health Monitor

Health monitors check API connectivity and report status to the UI.

### Files to modify:

1. **`services/tui_service/src/{source}_health.rs`** - Create health monitor
   ```rust
   pub struct AlpacaHealth {
       pub is_paper: bool,
       pub connected: bool,
       pub status: String,
       pub last_check: DateTime<Utc>,
       // ... other fields
   }
   
   pub struct AlpacaHealthMonitor {
       paper_health: AlpacaHealth,
       live_health: AlpacaHealth,
   }
   
   impl AlpacaHealthMonitor {
       pub fn new() -> Self { /* initialize */ }
       
       pub fn spawn_health_checks(self, event_tx: UnboundedSender<AppEvent>) -> JoinHandle<()> {
           tokio::spawn(async move {
               let mut interval = tokio::time::interval(Duration::from_secs(30));
               let mut monitor = self;
               
               loop {
                   interval.tick().await;
                   
                   // Check paper
                   let paper = monitor.check_health(true).await;
                   let _ = event_tx.send(AppEvent::AlpacaHealthUpdate {
                       is_paper: true,
                       connected: paper.connected,
                       status: paper.status.clone(),
                   });
                   
                   // Check live
                   let live = monitor.check_health(false).await;
                   let _ = event_tx.send(AppEvent::AlpacaHealthUpdate {
                       is_paper: false,
                       connected: live.connected,
                       status: live.status.clone(),
                   });
               }
           })
       }
   }
   ```

2. **`services/tui_service/src/events.rs`** - Add event variant
   ```rust
   pub enum AppEvent {
       // ... existing variants
       AlpacaHealthUpdate {
           is_paper: bool,
           connected: bool,
           status: String,
       },
   }
   ```

3. **`services/tui_service/src/app.rs`** - Add state fields
   ```rust
   pub alpaca_paper_status: ConnectionStatus,
   pub alpaca_live_status: ConnectionStatus,
   ```

4. **`services/tui_service/src/app_updates.rs`** - Handle events
   ```rust
   AppEvent::AlpacaHealthUpdate { is_paper, connected, status } => {
       let status = if connected {
           ConnectionStatus::new(ConnectionState::Connected, &status)
       } else {
           ConnectionStatus::new(ConnectionState::Error, &status)
       };
       
       if is_paper {
           app.alpaca_paper_status = status;
       } else {
           app.alpaca_live_status = status;
       }
   }
   ```

5. **`services/tui_service/src/main.rs`** - Spawn with event channel
   ```rust
   let alpaca_health_monitor = crate::alpaca_health::AlpacaHealthMonitor::new();
   tokio::spawn(async move {
       alpaca_health_monitor.spawn_health_checks(event_tx.clone()).await;
   });
   ```

---

## Checklist Summary

### For Position Sources:
- [ ] Create `{source}_positions.rs` with source struct and fetch methods
- [ ] Export from `api/src/lib.rs`
- [ ] Add state fields to `App`
- [ ] Add event variant to `AppEvent`
- [ ] Handle event in `apply_event()`
- [ ] Spawn background task in `main.rs`
- [ ] Create fetcher function with interval loop

### For Market Data Sources:
- [ ] Create `{source}.rs` implementing `MarketDataSource`
- [ ] Register in `provider_registry()`
- [ ] Add to `settings_sources.rs` display table
- [ ] Ensure credentials are checked in `create()` method

### For Health Monitors:
- [ ] Create `{source}_health.rs` with health struct and monitor
- [ ] Add state fields to `App` (ConnectionStatus)
- [ ] Add event variant to `AppEvent`
- [ ] Handle event in `apply_event()`
- [ ] Spawn with `event_tx` in `main.rs`

---

## Common Patterns

### Credential Checking

Always check credentials before making API calls:

```rust
// In source implementation
pub fn from_env() -> Option<Self> {
    let key_id = std::env::var("APCA_API_KEY_ID").ok()?;
    let secret_key = std::env::var("APCA_API_SECRET_KEY").ok()?;
    Some(Self { key_id, secret_key })
}

// Or using credential store
use api::credentials::{credential_source, CredentialKey};

fn has_credentials(is_paper: bool) -> bool {
    let key = if is_paper {
        CredentialKey::AlpacaPaperApiKeyId
    } else {
        CredentialKey::AlpacaLiveApiKeyId
    };
    credential_source(key).is_some()
}
```

### Error Handling in Background Tasks

Always log errors but don't crash the task:

```rust
loop {
    interval.tick().await;
    
    match do_work().await {
        Ok(result) => {
            let _ = event_tx.send(AppEvent::Update(result));
        }
        Err(e) => {
            tracing::error!("Background task error: {}", e);
            // Optionally send error event
            let _ = event_tx.send(AppEvent::Error(e.to_string()));
        }
    }
}
```

### Testing Without Credentials

For development without real credentials:

```rust
#[cfg(feature = "mock")]
pub fn mock() -> Self {
    Self {
        positions: vec![/* test data */],
    }
}
```

---

## See Also

- `TUI_ARCHITECTURE.md` - Main loop and data flow overview
- `MARKET_DATA_INTEGRATION.md` - Market data aggregation details
- `docs/DATA_EXPLORATION_MODE.md` - Product context for data sources
