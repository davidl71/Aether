//! IB Adapter - Async wrapper for Interactive Brokers TWS/Gateway
//!
//! This crate provides a modern async Rust interface to Interactive Brokers TWS API.
//! Integrates with the ibapi crate for actual TWS/Gateway communication.
//!
//! Implements [`broker_engine::BrokerEngine`] so the backend can use this via the
//! trait without coupling to `ibapi` directly.
//!
//! **Connection retry:** This crate does not retry on connection failure. Callers should
//! implement retry with exponential backoff (e.g. 2s → 60s cap); see TWS reconnect
//! behavior in `backend_service` (tws_market_data, tws_positions) and
//! `docs/platform/TWS_RECONNECT_BACKOFF.md`.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use ibapi::accounts::types::AccountGroup;
use ibapi::accounts::{AccountSummaryTags, PositionUpdate};
use ibapi::contracts::{Contract, OptionChain, SecurityType};
use ibapi::market_data::realtime::{TickType, TickTypes};
use ibapi::Client as IbClient;
use ibapi::Error as IbError;
use tokio::sync::{broadcast, mpsc, RwLock};
use tracing::{debug, info, warn};

pub use broker_engine::domain::{
    BrokerConfig, MarketDataEvent, OptionContract, OrderStatusEvent, Position, PositionEvent,
};
pub use broker_engine::AccountInfo;
pub use broker_engine::BrokerEngine;
pub use broker_engine::BrokerError;
pub use broker_engine::ConnectionState;
pub use broker_engine::MarketData;
pub use broker_engine::MarketDataSubscription;
pub use broker_engine::MarketDataSubscriptionError;

pub mod pacer;
pub mod scanner;
pub mod tws_wire;
pub mod types;

pub use pacer::TwsPacer;
pub use scanner::ScannerSubscription;
pub use tws_wire::{TwsProtoFrame, PROTOBUF_MSG_ID_OFFSET};

/// Whether the symbol is an index (uses SecurityType::Index + CBOE exchange).
#[cfg(test)]
fn is_index(symbol: &str) -> bool {
    matches!(symbol.to_uppercase().as_str(), "SPX" | "NDX" | "XSP")
}

/// Exchange to use when building an option contract for `contract_details`.
#[cfg(test)]
fn exchange_for_option(symbol: &str) -> &'static str {
    if is_index(symbol) {
        "CBOE"
    } else {
        "SMART"
    }
}

/// IB Adapter configuration (alias for [`BrokerConfig`] for backwards compatibility)
pub type IbConfig = BrokerConfig;

/// IB Adapter — implements [`BrokerEngine`] using the ibapi crate.
pub struct IbAdapter {
    config: BrokerConfig,
    state: Arc<RwLock<ConnectionState>>,
    client: Arc<RwLock<Option<Arc<IbClient>>>>,
    market_data_tx: mpsc::Sender<MarketDataEvent>,
    market_data_broadcast_tx: broadcast::Sender<MarketDataEvent>,
    /// Active market data subscriptions keyed by contract_id.
    /// Each entry is a cancellation trigger.
    market_data_subscriptions: Arc<RwLock<HashMap<i64, mpsc::Sender<()>>>>,
}

struct TokioBroadcastMarketDataSubscription {
    receiver: broadcast::Receiver<MarketDataEvent>,
}

#[async_trait]
impl MarketDataSubscription for TokioBroadcastMarketDataSubscription {
    async fn recv(&mut self) -> Result<MarketDataEvent, MarketDataSubscriptionError> {
        self.receiver.recv().await.map_err(|error| match error {
            broadcast::error::RecvError::Lagged(skipped) => {
                MarketDataSubscriptionError::Lagged(skipped)
            }
            broadcast::error::RecvError::Closed => MarketDataSubscriptionError::Closed,
        })
    }
}

impl IbAdapter {
    pub fn new(config: BrokerConfig) -> Self {
        let (market_data_tx, mut market_data_rx) = mpsc::channel(100);
        let (market_data_broadcast_tx, _) = broadcast::channel(1024);

        let market_data_broadcast_tx_clone = market_data_broadcast_tx.clone();
        tokio::spawn(async move {
            while let Some(event) = market_data_rx.recv().await {
                let _ = market_data_broadcast_tx_clone.send(event);
            }
        });

        Self {
            config,
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            client: Arc::new(RwLock::new(None)),
            market_data_tx,
            market_data_broadcast_tx,
            market_data_subscriptions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn connect(&self) -> Result<(), BrokerError> {
        *self.state.write().await = ConnectionState::Connecting;
        let address = format!("{}:{}", self.config.host, self.config.port);
        info!("Connecting to IB at {}", address);

        let client_id = self.config.client_id as i32;
        match IbClient::connect(&address, client_id).await {
            Ok(client) => {
                let mut guard = self.client.write().await;
                *guard = Some(Arc::new(client));
                drop(guard);
                *self.state.write().await = ConnectionState::Connected;
                info!("Connected to IB at {}", address);
                Ok(())
            }
            Err(e) => {
                let msg = format!("IB connection failed: {}", e);
                *self.state.write().await = ConnectionState::Error(msg.clone());
                Err(BrokerError::ConnectionFailed(msg))
            }
        }
    }

    pub async fn disconnect(&self) -> Result<(), BrokerError> {
        self.client.write().await.take();
        *self.state.write().await = ConnectionState::Disconnected;
        info!("Disconnected from IB");
        Ok(())
    }

    pub async fn request_market_data(
        &self,
        symbol: &str,
        contract_id: i64,
    ) -> Result<(), BrokerError> {
        if *self.state.read().await != ConnectionState::Connected {
            return Err(BrokerError::NotConnected);
        }

        let key = if contract_id != 0 {
            contract_id
        } else {
            let mut h: i64 = 0;
            for b in symbol.bytes() {
                h = h.wrapping_mul(31).wrapping_add(b as i64);
            }
            h
        };

        // Check if already subscribed
        {
            let subs = self.market_data_subscriptions.read().await;
            if subs.contains_key(&key) {
                debug!(symbol, key, "already subscribed to market data");
                return Ok(());
            }
        }

        let arc = self.client.read().await.clone();
        let client = arc.as_ref().ok_or(BrokerError::NotConnected)?;
        let mut contract = Contract::stock(symbol).build();
        if contract_id != 0 {
            contract.contract_id = contract_id as i32;
        }

        let sub = client
            .market_data(&contract)
            .subscribe()
            .await
            .map_err(|e: IbError| BrokerError::Other(e.to_string()))?;

        // Create cancellation channel for this subscription
        let (cancel_tx, mut cancel_rx) = mpsc::channel::<()>(1);
        {
            let mut subs = self.market_data_subscriptions.write().await;
            subs.insert(key, cancel_tx);
        }

        let market_data_tx = self.market_data_tx.clone();
        let subs = self.market_data_subscriptions.clone();
        let symbol_owned = symbol.to_string();

        // Spawn task to forward ticks to market_data_tx
        tokio::spawn(async move {
            let symbol = symbol_owned;
            let mut sub = sub;
            let mut bid = 0.0_f64;
            let mut ask = 0.0_f64;
            let mut last = 0.0_f64;
            let mut volume = 0u64;

            loop {
                tokio::select! {
                    _ = cancel_rx.recv() => {
                        debug!(%symbol, "market data subscription cancelled");
                        break;
                    }
                    tick_opt = sub.next() => {
                        match tick_opt {
                            Some(Ok(tick)) => {
                                match &tick {
                                    TickTypes::Price(pt) => {
                                        match pt.tick_type {
                                            TickType::Bid | TickType::DelayedBid | TickType::DelayedBidOption => bid = pt.price,
                                            TickType::Ask | TickType::DelayedAsk | TickType::DelayedAskOption => ask = pt.price,
                                            TickType::Last | TickType::DelayedLast | TickType::DelayedLastOption => last = pt.price,
                                            TickType::Close | TickType::DelayedClose => {
                                                if last == 0.0 { last = pt.price; }
                                                if bid == 0.0 { bid = pt.price; }
                                                if ask == 0.0 { ask = pt.price; }
                                            }
                                            _ => {}
                                        }
                                    }
                                    TickTypes::PriceSize(ps) => {
                                        match ps.price_tick_type {
                                            TickType::Bid | TickType::DelayedBid | TickType::DelayedBidOption => bid = ps.price,
                                            TickType::Ask | TickType::DelayedAsk | TickType::DelayedAskOption => ask = ps.price,
                                            _ => {}
                                        }
                                        volume = ps.size as u64;
                                    }
                                    TickTypes::Size(s) => {
                                        volume = s.size as u64;
                                    }
                                    _ => {}
                                }

                                // Send event when we have bid and ask
                                if bid > 0.0 && ask > 0.0 {
                                    let event = MarketDataEvent {
                                        contract_id: key,
                                        symbol: symbol.clone(),
                                        bid,
                                        ask,
                                        last,
                                        volume,
                                        timestamp: chrono::Utc::now(),
                                        quote_quality: 0,
                                        source: "tws".to_string(),
                                        source_priority: 100,
                                    };
                                    if market_data_tx.send(event).await.is_err() {
                                        warn!(%symbol, "market_data_tx dropped, stopping");
                                        break;
                                    }
                                }
                            }
                            Some(Err(e)) => {
                                debug!(error = %e, %symbol, "market data tick error");
                            }
                            None => {
                                debug!(%symbol, "market data subscription ended");
                                break;
                            }
                        }
                    }
                }
            }

            // Cleanup subscription
            let mut subs = subs.write().await;
            subs.remove(&key);
            debug!(%symbol, "market data subscription task exiting");
        });

        debug!(symbol, key, "started market data subscription");
        Ok(())
    }

    pub async fn request_option_chain(
        &self,
        symbol: &str,
    ) -> Result<Vec<OptionContract>, BrokerError> {
        if *self.state.read().await != ConnectionState::Connected {
            return Err(BrokerError::NotConnected);
        }
        let arc = self.client.read().await.clone();
        let client = arc.as_ref().ok_or(BrokerError::NotConnected)?;
        let mut sub = client
            .option_chain(symbol, "SMART", SecurityType::Stock, 0)
            .await
            .map_err(|e: IbError| BrokerError::Other(e.to_string()))?;
        let mut out = Vec::new();
        if let Some(chain_result) = sub.next().await {
            let chain: OptionChain = chain_result.map_err(|e| BrokerError::Other(e.to_string()))?;
            for exp in &chain.expirations {
                for &strike in &chain.strikes {
                    out.push(OptionContract::new(symbol, exp, strike, true));
                    out.push(OptionContract::new(symbol, exp, strike, false));
                }
            }
        }
        Ok(out)
    }

    pub async fn request_positions(&self) -> Result<Vec<PositionEvent>, BrokerError> {
        if *self.state.read().await != ConnectionState::Connected {
            return Err(BrokerError::NotConnected);
        }
        let arc = self.client.read().await.clone();
        let client = arc.as_ref().ok_or(BrokerError::NotConnected)?;
        let mut sub = client
            .positions()
            .await
            .map_err(|e: IbError| BrokerError::Other(e.to_string()))?;
        let mut out = Vec::new();
        while let Some(update) = sub.next().await {
            match update.map_err(|e| BrokerError::Other(e.to_string()))? {
                PositionUpdate::Position(p) => {
                    let symbol = p.contract.symbol.to_string();
                    out.push(PositionEvent {
                        account: p.account.clone(),
                        symbol,
                        position: p.position as i32,
                        avg_cost: p.average_cost,
                    });
                }
                PositionUpdate::PositionEnd => break,
            }
        }
        Ok(out)
    }

    pub async fn request_account(&self) -> Result<AccountInfo, BrokerError> {
        if *self.state.read().await != ConnectionState::Connected {
            return Err(BrokerError::NotConnected);
        }
        let arc = self.client.read().await.clone();
        let client = arc.as_ref().ok_or(BrokerError::NotConnected)?;
        let accounts = client
            .managed_accounts()
            .await
            .map_err(|e: IbError| BrokerError::Other(e.to_string()))?;
        let account_id = accounts.first().cloned().unwrap_or_else(|| "".to_string());
        let group = AccountGroup("All".to_string());
        let tags = AccountSummaryTags::ALL;
        let mut sub = client
            .account_summary(&group, tags)
            .await
            .map_err(|e: IbError| BrokerError::Other(e.to_string()))?;
        let mut net_liq = 0.0;
        let mut cash = 0.0;
        let mut buying_power = 0.0;
        let mut maint_margin = 0.0;
        let mut init_margin = 0.0;
        use ibapi::accounts::AccountSummaryResult;
        while let Some(summary) = sub.next().await {
            let s = summary.map_err(|e| BrokerError::Other(e.to_string()))?;
            if let AccountSummaryResult::Summary(s) = s {
                if s.account != account_id {
                    continue;
                }
                match s.tag.as_str() {
                    "NetLiquidation" => net_liq = s.value.parse().unwrap_or(0.0),
                    "TotalCashValue" => cash = s.value.parse().unwrap_or(0.0),
                    "BuyingPower" => buying_power = s.value.parse().unwrap_or(0.0),
                    "MaintMarginReq" => maint_margin = s.value.parse().unwrap_or(0.0),
                    "InitMarginReq" => init_margin = s.value.parse().unwrap_or(0.0),
                    _ => {}
                }
            }
        }
        Ok(AccountInfo {
            account_id,
            net_liquidation: net_liq,
            cash_balance: cash,
            buying_power,
            maintenance_margin: maint_margin,
            initial_margin: init_margin,
        })
    }
}

#[async_trait]
impl BrokerEngine for IbAdapter {
    async fn connect(&self) -> Result<(), BrokerError> {
        self.connect().await
    }

    async fn disconnect(&self) -> Result<(), BrokerError> {
        self.disconnect().await
    }

    async fn state(&self) -> ConnectionState {
        self.state.read().await.clone()
    }

    async fn request_market_data(&self, symbol: &str, contract_id: i64) -> Result<(), BrokerError> {
        self.request_market_data(symbol, contract_id).await
    }

    async fn request_option_chain(&self, symbol: &str) -> Result<Vec<OptionContract>, BrokerError> {
        self.request_option_chain(symbol).await
    }

    fn subscribe_market_data(&self) -> Box<dyn MarketDataSubscription> {
        Box::new(TokioBroadcastMarketDataSubscription {
            receiver: self.market_data_broadcast_tx.subscribe(),
        })
    }

    async fn request_positions(&self) -> Result<Vec<PositionEvent>, BrokerError> {
        self.request_positions().await
    }

    async fn request_account(&self) -> Result<AccountInfo, BrokerError> {
        self.request_account().await
    }

    fn request_positions_sync(&self, timeout_ms: u64) -> Result<Vec<PositionEvent>, BrokerError> {
        let state = futures::executor::block_on(self.state.read());
        if *state != ConnectionState::Connected {
            return Err(BrokerError::NotConnected);
        }
        drop(state);

        let client = futures::executor::block_on(self.client.read()).clone();
        let client = client.ok_or(BrokerError::NotConnected)?;

        let timeout = Duration::from_millis(timeout_ms);

        let rt = tokio::runtime::Handle::current();
        rt.block_on(async {
            let mut sub = client
                .positions()
                .await
                .map_err(|e: IbError| BrokerError::Other(e.to_string()))?;
            let mut out = Vec::new();
            loop {
                let update = tokio::time::timeout(timeout, sub.next()).await;
                match update {
                    Ok(Some(Ok(update))) => match update {
                        PositionUpdate::Position(p) => {
                            out.push(PositionEvent {
                                account: p.account.clone(),
                                symbol: p.contract.symbol.to_string(),
                                position: p.position as i32,
                                avg_cost: p.average_cost,
                            });
                        }
                        PositionUpdate::PositionEnd => break,
                    },
                    Ok(Some(Err(e))) => {
                        return Err(BrokerError::Other(e.to_string()));
                    }
                    Ok(None) | Err(_) => {
                        return Err(BrokerError::Other(
                            "request_positions_sync timed out".to_string(),
                        ));
                    }
                }
            }
            Ok(out)
        })
    }

    fn supports_options(&self) -> bool {
        true
    }

    fn supports_box_spreads(&self) -> bool {
        true
    }

    fn supports_combo_orders(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_index_recognises_known_index_symbols() {
        assert!(is_index("SPX"));
        assert!(is_index("spx"));
        assert!(is_index("NDX"));
        assert!(is_index("XSP"));
        assert!(!is_index("SPY"));
        assert!(!is_index("AAPL"));
        assert!(!is_index("QQQ"));
    }

    #[test]
    fn exchange_for_option_returns_cboe_for_indices() {
        assert_eq!(exchange_for_option("SPX"), "CBOE");
        assert_eq!(exchange_for_option("NDX"), "CBOE");
        assert_eq!(exchange_for_option("XSP"), "CBOE");
        assert_eq!(exchange_for_option("SPY"), "SMART");
        assert_eq!(exchange_for_option("AAPL"), "SMART");
    }
}
