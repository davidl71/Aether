//! IB Adapter - Async wrapper for Interactive Brokers TWS/Gateway
//!
//! This crate provides a modern async Rust interface to Interactive Brokers TWS API.
//! Integrates with the ibapi crate for actual TWS/Gateway communication.
//!
//! **Connection retry:** This crate does not retry on connection failure. Callers should
//! implement retry with exponential backoff (e.g. 2s → 60s cap); see TWS reconnect
//! behavior in `backend_service` (tws_market_data, tws_positions) and
//! `docs/platform/TWS_RECONNECT_BACKOFF.md`.

use std::sync::Arc;

use ibapi::accounts::types::AccountGroup;
use ibapi::accounts::{AccountSummaryTags, PositionUpdate};
use ibapi::contracts::{Contract, LegAction, OptionChain, SecurityType};
use ibapi::Client as IbClient;
use ibapi::Error as IbError;
use tokio::sync::{mpsc, RwLock};
use tracing::info;

pub mod scanner;
pub mod types;

pub use scanner::ScannerSubscription;
pub use types::*;

/// IB Adapter configuration
#[derive(Debug, Clone)]
pub struct IbConfig {
    pub host: String,
    pub port: u16,
    pub client_id: u32,
    pub paper_trading: bool,
}

impl Default for IbConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 7497,
            client_id: 0,
            paper_trading: true,
        }
    }
}

/// Connection state
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

/// Market data event
#[derive(Debug, Clone)]
pub struct MarketDataEvent {
    pub contract_id: i64,
    pub symbol: String,
    pub bid: f64,
    pub ask: f64,
    pub last: f64,
    pub volume: i64,
}

/// Position event
#[derive(Debug, Clone)]
pub struct PositionEvent {
    pub account: String,
    pub symbol: String,
    pub position: i32,
    pub avg_cost: f64,
}

/// Order status event
#[derive(Debug, Clone)]
pub struct OrderStatusEvent {
    pub order_id: i32,
    pub status: String,
    pub filled: i32,
    pub remaining: i32,
    pub avg_fill_price: f64,
}

/// IB Adapter - main entry point
pub struct IbAdapter {
    config: IbConfig,
    state: Arc<RwLock<ConnectionState>>,
    /// Connected ibapi client; Some when state is Connected (Arc allows use without holding lock across await)
    client: Arc<RwLock<Option<Arc<IbClient>>>>,
    /// Channel for market data events (reserved for future TWS integration)
    #[allow(dead_code)]
    market_data_tx: mpsc::Sender<MarketDataEvent>,
    /// Channel for position events (reserved for future TWS integration)
    #[allow(dead_code)]
    position_tx: mpsc::Sender<PositionEvent>,
    /// Channel for order status events (reserved for future TWS integration)
    #[allow(dead_code)]
    order_tx: mpsc::Sender<OrderStatusEvent>,
}

impl IbAdapter {
    /// Create a new IB Adapter
    pub fn new(config: IbConfig) -> Self {
        let (market_data_tx, _) = mpsc::channel(100);
        let (position_tx, _) = mpsc::channel(100);
        let (order_tx, _) = mpsc::channel(100);

        Self {
            config,
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            client: Arc::new(RwLock::new(None)),
            market_data_tx,
            position_tx,
            order_tx,
        }
    }

    /// Get current connection state
    pub async fn state(&self) -> ConnectionState {
        self.state.read().await.clone()
    }

    /// Connect to TWS/Gateway using the ibapi crate.
    /// Callers should retry on failure with exponential backoff (e.g. 2s → 60s cap); this crate does not retry.
    pub async fn connect(&self) -> Result<(), String> {
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
                Err(msg)
            }
        }
    }

    /// Disconnect from TWS/Gateway. Explicitly drops the ibapi client so the connection is closed.
    pub async fn disconnect(&self) {
        if self.client.write().await.take().is_some() {
            // client dropped here; ibapi connection closes on drop
        }
        *self.state.write().await = ConnectionState::Disconnected;
        info!("Disconnected from IB");
    }

    /// Request market data for a contract. Starts a subscription; ticks are not yet forwarded to market_data_tx.
    pub async fn request_market_data(&self, symbol: &str, contract_id: i64) -> Result<(), String> {
        if *self.state.read().await != ConnectionState::Connected {
            return Err("Not connected".to_string());
        }
        let arc = self.client.read().await.clone();
        let client = arc.as_ref().ok_or("Not connected")?;
        let mut contract = Contract::stock(symbol).build();
        if contract_id != 0 {
            contract.contract_id = contract_id as i32;
        }
        client
            .market_data(&contract)
            .subscribe()
            .await
            .map_err(|e: IbError| e.to_string())?;
        Ok(())
    }

    /// Request option chain for underlying via ibapi security definition option parameters.
    pub async fn request_option_chain(
        &self,
        symbol: &str,
    ) -> Result<Vec<types::OptionContract>, String> {
        if *self.state.read().await != ConnectionState::Connected {
            return Err("Not connected".to_string());
        }
        let arc = self.client.read().await.clone();
        let client = arc.as_ref().ok_or("Not connected")?;
        let mut sub = client
            .option_chain(symbol, "SMART", SecurityType::Stock, 0)
            .await
            .map_err(|e: IbError| e.to_string())?;
        let mut out = Vec::new();
        if let Some(chain_result) = sub.next().await {
            let chain: OptionChain = chain_result.map_err(|e| e.to_string())?;
            for exp in &chain.expirations {
                for &strike in &chain.strikes {
                    out.push(types::OptionContract::new(symbol, exp, strike, true));
                    out.push(types::OptionContract::new(symbol, exp, strike, false));
                }
            }
        }
        Ok(out)
    }

    /// Place an order via ibapi.
    pub async fn place_order(
        &self,
        contract: types::OptionContract,
        action: types::OrderAction,
        quantity: i32,
        limit_price: f64,
    ) -> Result<i32, String> {
        if *self.state.read().await != ConnectionState::Connected {
            return Err("Not connected".to_string());
        }
        let arc = self.client.read().await.clone();
        let client = arc.as_ref().ok_or("Not connected")?;
        let (y, m, d) = common::expiry::parse_expiry_yyyy_mm_dd(&contract.expiry)?;
        let ib_contract = if contract.is_call {
            Contract::call(&contract.symbol)
                .strike(contract.strike)
                .expires_on(y, m, d)
                .build()
        } else {
            Contract::put(&contract.symbol)
                .strike(contract.strike)
                .expires_on(y, m, d)
                .build()
        };
        let order_id = client.next_order_id();
        let qty = quantity as f64;
        let result = match action {
            types::OrderAction::Buy => {
                client
                    .order(&ib_contract)
                    .buy(qty)
                    .limit(limit_price)
                    .submit()
                    .await
            }
            types::OrderAction::Sell => {
                client
                    .order(&ib_contract)
                    .sell(qty)
                    .limit(limit_price)
                    .submit()
                    .await
            }
        };
        result.map_err(|e: IbError| e.to_string())?;
        Ok(order_id)
    }

    /// Place a BAG (combo) order for multi-leg strategies (e.g. box spread).
    /// Builds a BAG contract with combo legs (conId, ratio, action, SMART); paper port 7497,
    /// live trading gated by config. See docs/platform/TWS_BAG_COMBO_POSITIONS.md and
    /// docs/platform/TWS_COMPLEX_ORDER_ECOSYSTEM.md.
    pub async fn place_bag_order(
        &self,
        request: types::PlaceBagOrderRequest,
    ) -> Result<i32, String> {
        if *self.state.read().await != ConnectionState::Connected {
            return Err("Not connected".to_string());
        }
        if !self.config.paper_trading {
            return Err(
                "BAG order placement is disabled for live trading; use paper port 7497 and paper_trading=true".to_string()
            );
        }
        if request.legs.is_empty() {
            return Err("BAG order must have at least one leg".to_string());
        }
        for (i, leg) in request.legs.iter().enumerate() {
            if leg.con_id.is_none() {
                return Err(format!(
                    "BAG leg {} missing con_id (resolve option contract to conId via TWS contract details)",
                    i + 1
                ));
            }
        }
        let exchange = if request.exchange.is_empty() {
            "SMART".to_string()
        } else {
            request.exchange.clone()
        };
        let mut builder = Contract::spread()
            .in_currency(&request.currency)
            .on_exchange(&exchange);
        for leg in &request.legs {
            let con_id = leg.con_id.unwrap();
            let action = match leg.action {
                types::OrderAction::Buy => LegAction::Buy,
                types::OrderAction::Sell => LegAction::Sell,
            };
            builder = builder.add_leg(con_id, action).ratio(leg.ratio).done();
        }
        let mut bag_contract = builder.build().map_err(|e| e.to_string())?;
        bag_contract.symbol = ibapi::contracts::Symbol::from(request.underlying_symbol.as_str());

        let arc = self.client.read().await.clone();
        let client = arc.as_ref().ok_or("Not connected")?;
        let order_id = client.next_order_id();
        let qty = request.quantity as f64;
        let limit_price = request.limit_price.unwrap_or(0.0);
        let result = match request.order_action {
            types::OrderAction::Buy => {
                client
                    .order(&bag_contract)
                    .buy(qty)
                    .limit(limit_price)
                    .submit()
                    .await
            }
            types::OrderAction::Sell => {
                client
                    .order(&bag_contract)
                    .sell(qty)
                    .limit(limit_price)
                    .submit()
                    .await
            }
        };
        result.map_err(|e: IbError| e.to_string())?;
        Ok(order_id)
    }

    /// Cancel an order via ibapi.
    pub async fn cancel_order(&self, order_id: i32) -> Result<(), String> {
        if *self.state.read().await != ConnectionState::Connected {
            return Err("Not connected".to_string());
        }
        let arc = self.client.read().await.clone();
        let client = arc.as_ref().ok_or("Not connected")?;
        let mut sub = client
            .cancel_order(order_id, "")
            .await
            .map_err(|e: IbError| e.to_string())?;
        let _ = sub.next().await;
        Ok(())
    }

    /// Request current positions via ibapi; collects initial snapshot until PositionEnd.
    pub async fn request_positions(&self) -> Result<Vec<PositionEvent>, String> {
        if *self.state.read().await != ConnectionState::Connected {
            return Err("Not connected".to_string());
        }
        let arc = self.client.read().await.clone();
        let client = arc.as_ref().ok_or("Not connected")?;
        let mut sub = client
            .positions()
            .await
            .map_err(|e: IbError| e.to_string())?;
        let mut out = Vec::new();
        while let Some(update) = sub.next().await {
            match update.map_err(|e| e.to_string())? {
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

    /// Request account information via ibapi account summary (first managed account).
    pub async fn request_account(&self) -> Result<types::AccountInfo, String> {
        if *self.state.read().await != ConnectionState::Connected {
            return Err("Not connected".to_string());
        }
        let arc = self.client.read().await.clone();
        let client = arc.as_ref().ok_or("Not connected")?;
        let accounts = client
            .managed_accounts()
            .await
            .map_err(|e: IbError| e.to_string())?;
        let account_id = accounts.first().cloned().unwrap_or_else(|| "".to_string());
        let group = AccountGroup("All".to_string());
        let tags = AccountSummaryTags::ALL;
        let mut sub = client
            .account_summary(&group, tags)
            .await
            .map_err(|e: IbError| e.to_string())?;
        let mut net_liq = 0.0;
        let mut cash = 0.0;
        let mut buying_power = 0.0;
        let mut maint_margin = 0.0;
        let mut init_margin = 0.0;
        use ibapi::accounts::AccountSummaryResult;
        while let Some(summary) = sub.next().await {
            let s = summary.map_err(|e| e.to_string())?;
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
        Ok(types::AccountInfo {
            account_id,
            net_liquidation: net_liq,
            cash_balance: cash,
            buying_power,
            maintenance_margin: maint_margin,
            initial_margin: init_margin,
        })
    }
}
