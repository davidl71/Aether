//! yatws Adapter — implements [`BrokerEngine`] backed by the yatws TWS client.
//!
//! yatws is a synchronous TWS API (parking_lot + blocking Condvar waits). Every
//! manager call is wrapped in `tokio::task::spawn_blocking` so the async trait
//! interface is preserved without blocking the Tokio runtime.
//!
//! # Box spread placement
//!
//! `place_bag_order` uses yatws' [`OptionsStrategyBuilder`] with
//! `.box_spread_nearest_expiry()` — the builder handles conId resolution
//! automatically via `DataRefManager`.
//!
//! # Safety gate
//!
//! `place_bag_order` rejects `config.paper_trading == false` at the call site,
//! mirroring IbAdapter behaviour.
//!
//! # Streaming
//!
//! After connecting, the adapter creates an [`AccountSubscription`] and spawns a
//! background task that forwards [`PositionUpdate`] events to `position_tx` and
//! [`ExecutionUpdate`] events to `order_tx` as [`OrderStatusEvent`] fills.

use std::collections::HashMap;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;

use async_trait::async_trait;
use backoff::backoff::Backoff;
use backoff::exponential::ExponentialBackoffBuilder;
use chrono::NaiveDate;
use tokio::sync::{mpsc, watch, RwLock};
use tracing::{error, info, warn};
use yatws::account_subscription::AccountEvent;
use yatws::contract::{Contract, OptionRight, SecType};
use yatws::data_subscription::{MarketDataSubscription, TickDataEvent};
use yatws::{IBKRClient, IBKRError, OptionsStrategyBuilder};

pub use broker_engine::domain::{
    BagOrderLeg, BrokerConfig, MarketDataEvent, OptionContract, OrderAction, OrderStatus,
    OrderStatusEvent, PlaceBagOrderRequest, Position, PositionEvent, QuoteQuality,
    ResolvedOptionContract, TimeInForce,
};
pub use broker_engine::AccountInfo;
pub use broker_engine::BrokerEngine;
pub use broker_engine::BrokerError;
pub use broker_engine::ConnectionState;
pub use broker_engine::MarketData;
pub use broker_engine::OptionChainProvider;

// ---------------------------------------------------------------------------
// Helper utilities
// ---------------------------------------------------------------------------

#[cfg(test)]
fn is_index(symbol: &str) -> bool {
    matches!(symbol.to_uppercase().as_str(), "SPX" | "NDX" | "XSP")
}

/// Security type to pass to `OptionsStrategyBuilder`.
///
/// The builder only accepts `Stock` or `Future`; indices like SPX are
/// passed as `Stock` so the builder can still fetch option chain params.
/// Exchange selection is handled by `.with_highest_liquidity()`.
fn sec_type_for(_symbol: &str) -> SecType {
    SecType::Stock
}

/// Parse a `"YYYYMMDD"` expiry string into a [`NaiveDate`].
fn parse_expiry_to_naive_date(yyyymmdd: &str) -> Result<NaiveDate, BrokerError> {
    NaiveDate::parse_from_str(yyyymmdd, "%Y%m%d")
        .map_err(|e| BrokerError::Other(format!("invalid expiry '{}': {}", yyyymmdd, e)))
}

/// Map a yatws `IBKRError` to `BrokerError`.
fn map_ibkr_error(e: IBKRError) -> BrokerError {
    BrokerError::Other(e.to_string())
}

// ---------------------------------------------------------------------------
// Reconnect configuration
// ---------------------------------------------------------------------------

const RECONNECT_INITIAL_INTERVAL: Duration = Duration::from_secs(2);
const RECONNECT_MAX_INTERVAL: Duration = Duration::from_secs(60);
const RECONNECT_MULTIPLIER: f64 = 2.0;
const RECONNECT_MAX_ELAPSED: Option<Duration> = None;

// ---------------------------------------------------------------------------
// YatWSEngine
// ---------------------------------------------------------------------------

/// yatws-backed broker engine — implements [`BrokerEngine`].
pub struct YatWSEngine {
    config: BrokerConfig,
    state: Arc<RwLock<ConnectionState>>,
    client: Arc<RwLock<Option<Arc<IBKRClient>>>>,
    market_data_tx: mpsc::Sender<MarketDataEvent>,
    position_tx: mpsc::Sender<PositionEvent>,
    order_tx: mpsc::Sender<OrderStatusEvent>,
    /// Account subscription for position/order streaming. Created on connect.
    account_sub: Arc<RwLock<Option<yatws::account_subscription::AccountSubscription>>>,
    /// Shutdown signal for the streaming task.
    shutdown_tx: Arc<StdMutex<Option<watch::Sender<bool>>>>,
    /// Reconnect trigger signal when connection is lost.
    reconnect_tx: Arc<StdMutex<Option<watch::Sender<bool>>>>,
    /// Active market data streaming handles keyed by contract_id.
    /// Each handle owns a OS thread that holds the !Send TickDataSubscription.
    market_data_handles: Arc<StdMutex<HashMap<i64, MarketDataHandle>>>,
}

/// Owned handle to a market data streaming OS thread.
/// The OS thread owns the !Send TickDataSubscription; this handle
/// only holds the cancellation signal and join handle.
struct MarketDataHandle {
    cancel_tx: watch::Sender<bool>,
    handle: std::thread::JoinHandle<()>,
}

impl YatWSEngine {
    pub fn new(config: BrokerConfig) -> Self {
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
            account_sub: Arc::new(RwLock::new(None)),
            shutdown_tx: Arc::new(StdMutex::new(None)),
            reconnect_tx: Arc::new(StdMutex::new(None)),
            market_data_handles: Arc::new(StdMutex::new(HashMap::new())),
        }
    }

    pub async fn connect(&self) -> Result<(), BrokerError> {
        let backoff: backoff::ExponentialBackoff = ExponentialBackoffBuilder::new()
            .with_initial_interval(RECONNECT_INITIAL_INTERVAL)
            .with_multiplier(RECONNECT_MULTIPLIER)
            .with_max_interval(RECONNECT_MAX_INTERVAL)
            .with_randomization_factor(0.1)
            .with_max_elapsed_time(RECONNECT_MAX_ELAPSED)
            .build();

        self.connect_with_backoff(backoff).await
    }

    async fn connect_with_backoff<B: Backoff>(&self, mut backoff: B) -> Result<(), BrokerError> {
        loop {
            // Create reconnect channel for this connection attempt.
            let (reconnect_tx, mut reconnect_rx) = watch::channel(false);
            *self.reconnect_tx.lock().unwrap() = Some(reconnect_tx);

            *self.state.write().await = ConnectionState::Connecting;
            let host = self.config.host.clone();
            let port = self.config.port;
            let client_id = self.config.client_id as i32;
            info!("Connecting to TWS via yatws at {}:{}", host, port);

            match tokio::task::spawn_blocking(move || IBKRClient::new(&host, port, client_id, None))
                .await
                .map_err(|e| BrokerError::Other(e.to_string()))?
            {
                Ok(client) => {
                    let client_arc = Arc::new(client);
                    *self.client.write().await = Some(client_arc.clone());
                    *self.state.write().await = ConnectionState::Connected;
                    info!(
                        "Connected to TWS via yatws at {}:{}",
                        self.config.host, self.config.port
                    );

                    match tokio::task::spawn_blocking(move || {
                        yatws::account_manager::AccountManager::create_account_subscription(
                            client_arc.account(),
                        )
                    })
                    .await
                    .map_err(|e| BrokerError::Other(e.to_string()))?
                    {
                        Ok(account_sub) => {
                            *self.account_sub.write().await = Some(account_sub);
                            let reconnect_tx = self
                                .reconnect_tx
                                .lock()
                                .unwrap()
                                .clone()
                                .expect("reconnect_tx was just set");
                            self.spawn_streaming_task(reconnect_tx);

                            // Wait for either shutdown or reconnect signal.
                            // If reconnect is signaled (streaming task detected connection loss),
                            // break and retry connection with backoff.
                            let shutdown_rx = {
                                let tx = self.shutdown_tx.lock().unwrap();
                                tx.as_ref().map(|t| t.subscribe())
                            };

                            if let Some(mut shutdown_rx) = shutdown_rx {
                                loop {
                                    tokio::select! {
                                        _ = shutdown_rx.changed() => {
                                            if *shutdown_rx.borrow() {
                                                info!("Shutdown signal received, stopping connection loop");
                                                return Ok(());
                                            }
                                        }
                                        _ = reconnect_rx.changed() => {
                                            if *reconnect_rx.borrow() {
                                                warn!("Reconnect signal received, restarting connection");
                                                // Clean up before reconnecting.
                                                self.cleanup_connection().await;
                                                break;
                                            }
                                        }
                                    }
                                }
                            } else {
                                // No shutdown receiver means disconnect was called.
                                return Ok(());
                            }
                        }
                        Err(e) => {
                            let msg = format!("failed to create account subscription: {}", e);
                            error!("{}", msg);
                            *self.state.write().await = ConnectionState::Error(msg.clone());
                            return Err(BrokerError::ConnectionFailed(msg));
                        }
                    }
                }
                Err(e) => {
                    let msg = format!("yatws connection failed: {}", e);
                    warn!("Connection attempt failed: {}", msg);

                    if let Some(delay) = backoff.next_backoff() {
                        *self.state.write().await = ConnectionState::Connecting;
                        info!(
                            "Reconnecting in {:.1}s (max: {:.1}s)",
                            delay.as_secs_f64(),
                            RECONNECT_MAX_INTERVAL.as_secs_f64()
                        );
                        tokio::time::sleep(delay).await;
                    } else {
                        *self.state.write().await = ConnectionState::Error(msg.clone());
                        return Err(BrokerError::ConnectionFailed(msg));
                    }
                }
            }
            // Reset backoff on successful connection attempt (even if it fails later,
            // we want to start from the beginning on the next manual connect).
            backoff.reset();
        }
    }

    pub async fn disconnect(&self) -> Result<(), BrokerError> {
        self.cleanup_connection().await;
        *self.state.write().await = ConnectionState::Disconnected;
        info!("Disconnected from TWS (yatws)");
        Ok(())
    }

    async fn cleanup_connection(&self) {
        // Signal streaming task to shut down.
        if let Some(tx) = self.shutdown_tx.lock().unwrap().take() {
            let _ = tx.send(true);
        }

        // Clear reconnect trigger.
        if let Some(tx) = self.reconnect_tx.lock().unwrap().take() {
            let _ = tx.send(true);
        }

        // Close account subscription.
        if let Some(sub) = self.account_sub.write().await.take() {
            tokio::task::spawn_blocking(move || {
                let mut s = sub;
                s.close()
            });
        }

        // Cancel all market data handles.
        let handles: Vec<_> = self.market_data_handles.lock().unwrap().drain().collect();
        for (_contract_id, handle) in handles {
            let _ = handle.cancel_tx.send(true);
            let _ = handle.handle.join();
        }

        self.client.write().await.take();
    }

    fn spawn_streaming_task(&self, reconnect_tx: watch::Sender<bool>) {
        let account_sub = self.account_sub.clone();
        let position_tx = self.position_tx.clone();
        let order_tx = self.order_tx.clone();
        let (shutdown_tx, shutdown_rx) = watch::channel(false);

        // Store shutdown sender synchronously.
        *self.shutdown_tx.lock().unwrap() = Some(shutdown_tx);

        tokio::spawn(async move {
            loop {
                // Check shutdown signal first.
                if *shutdown_rx.borrow() {
                    info!("Streaming task received shutdown signal");
                    break;
                }

                // Poll for next event without blocking.
                let event = {
                    let sub_guard = account_sub.read().await;
                    sub_guard.as_ref().map(|s| s.try_next_event())
                };

                match event {
                    Some(Ok(event)) => match event {
                        AccountEvent::PositionUpdate {
                            position,
                            timestamp: _,
                        } => {
                            let position_event = PositionEvent {
                                account: String::new(),
                                symbol: position.symbol.clone(),
                                position: position.quantity as i32,
                                avg_cost: position.average_cost,
                            };
                            if position_tx.send(position_event).await.is_err() {
                                warn!("position_tx dropped, stopping streaming");
                                break;
                            }
                        }
                        AccountEvent::ExecutionUpdate {
                            execution,
                            timestamp: _,
                        } => {
                            let order_id = execution.order_id.parse::<i32>().unwrap_or(0);
                            let filled = execution.cum_qty as i32;
                            let remaining = (execution.quantity - execution.cum_qty) as i32;
                            let order_event = OrderStatusEvent {
                                order_id,
                                status: format!("{:?}", execution.side),
                                filled,
                                remaining,
                                avg_fill_price: execution.avg_price,
                            };
                            if order_tx.send(order_event).await.is_err() {
                                warn!("order_tx dropped, stopping streaming");
                                break;
                            }
                        }
                        AccountEvent::SummaryUpdate { .. } => {}
                        AccountEvent::Error {
                            error,
                            timestamp: _,
                        } => {
                            error!("Account subscription error: {}", error);
                        }
                        AccountEvent::Closed {
                            account_id,
                            timestamp: _,
                        } => {
                            info!(
                                "Account subscription closed for account: {}, triggering reconnect",
                                account_id
                            );
                            let _ = reconnect_tx.send(true);
                            break;
                        }
                    },
                    Some(Err(e)) => {
                        error!("Error receiving account event: {}", e);
                    }
                    None => {
                        // No event ready, yield to allow shutdown check.
                        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                    }
                }
            }
            info!("Streaming task exiting");
        });
    }

    pub async fn request_market_data(
        &self,
        symbol: &str,
        contract_id: i64,
    ) -> Result<(), BrokerError> {
        if *self.state.read().await != ConnectionState::Connected {
            return Err(BrokerError::NotConnected);
        }

        let client = self
            .client
            .read()
            .await
            .clone()
            .ok_or(BrokerError::NotConnected)?;

        let contract = Contract::stock(symbol);

        let subscription = tokio::task::spawn_blocking({
            let client = client.clone();
            let contract = contract.clone();
            move || {
                client
                    .data_market()
                    .subscribe_market_data(&contract)
                    .submit()
            }
        })
        .await
        .map_err(|e| BrokerError::Other(e.to_string()))?
        .map_err(|e| BrokerError::Other(format!("market data subscription failed: {}", e)))?;

        let key = if contract_id != 0 {
            contract_id
        } else {
            let mut h: i64 = 0;
            for b in symbol.bytes() {
                h = h.wrapping_mul(31).wrapping_add(b as i64);
            }
            h
        };

        let sub_arc = Arc::new(StdMutex::new(Some(subscription)));
        let (std_tx, std_rx) = std::sync::mpsc::channel::<MarketDataEvent>();
        let (cancel_tx, cancel_rx) = watch::channel(false);

        // Spawn a tokio task to bridge std channel → async market_data_tx.
        let market_data_tx = self.market_data_tx.clone();
        tokio::spawn(async move {
            while let Ok(event) = std_rx.recv() {
                if market_data_tx.send(event).await.is_err() {
                    break;
                }
            }
        });

        let handle = std::thread::spawn({
            let symbol = symbol.to_string();
            let sub = sub_arc.clone();
            move || {
                let subscription = {
                    let mut guard = sub.lock().unwrap();
                    guard.take()
                };
                if subscription.is_none() {
                    return;
                }
                let subscription = subscription.unwrap();
                let mut iter = subscription.events();
                loop {
                    if *cancel_rx.borrow() {
                        break;
                    }
                    let event = iter.next();

                    match event {
                        Some(TickDataEvent::Price(tick_type, price, attrib)) => {
                            let bid = matches!(
                                tick_type,
                                yatws::data::TickType::BidPrice | yatws::data::TickType::DelayedBid
                            )
                            .then_some(price)
                            .unwrap_or(0.0);
                            let ask = matches!(
                                tick_type,
                                yatws::data::TickType::AskPrice | yatws::data::TickType::DelayedAsk
                            )
                            .then_some(price)
                            .unwrap_or(0.0);
                            let last = matches!(
                                tick_type,
                                yatws::data::TickType::LastPrice
                                    | yatws::data::TickType::DelayedLast
                            )
                            .then_some(price)
                            .unwrap_or(0.0);
                            let quote_quality = QuoteQuality::from_tick_attrib(
                                attrib.can_auto_execute,
                                attrib.past_limit,
                                attrib.pre_open,
                            );
                            let market_event = MarketDataEvent {
                                contract_id: key,
                                symbol: symbol.clone(),
                                bid,
                                ask,
                                last,
                                volume: 0,
                                timestamp: chrono::Utc::now(),
                                quote_quality: quote_quality.bits() as u32,
                            };
                            if std_tx.send(market_event).is_err() {
                                break;
                            }
                        }
                        Some(TickDataEvent::SnapshotEnd) | Some(TickDataEvent::Error(_)) | None => {
                            break;
                        }
                        _ => {}
                    }
                }
                drop(subscription);
                info!("Market data streaming thread exiting for {}", symbol);
            }
        });

        let mh = MarketDataHandle { cancel_tx, handle };
        self.market_data_handles.lock().unwrap().insert(key, mh);

        Ok(())
    }

    pub async fn request_option_chain(
        &self,
        symbol: &str,
    ) -> Result<Vec<OptionContract>, BrokerError> {
        if *self.state.read().await != ConnectionState::Connected {
            return Err(BrokerError::NotConnected);
        }
        let client = self
            .client
            .read()
            .await
            .clone()
            .ok_or(BrokerError::NotConnected)?;
        let sym_owned = symbol.to_string();
        let sec_type = sec_type_for(&sym_owned);

        let (sym_for_result, results) = tokio::task::spawn_blocking(move || {
            let res = client
                .data_ref()
                .get_option_chain_params(&sym_owned, "", sec_type, 0)
                .map_err(map_ibkr_error);
            res.map(|v| (sym_owned, v))
        })
        .await
        .map_err(|e| BrokerError::Other(e.to_string()))??;

        let mut out = Vec::new();
        for chain in &results {
            for exp in &chain.expirations {
                for &strike in &chain.strikes {
                    out.push(OptionContract::new(&sym_for_result, exp, strike, true));
                    out.push(OptionContract::new(&sym_for_result, exp, strike, false));
                }
            }
        }
        Ok(out)
    }

    pub async fn place_order(
        &self,
        _contract: OptionContract,
        _action: OrderAction,
        _quantity: i32,
        _limit_price: f64,
    ) -> Result<i32, BrokerError> {
        if *self.state.read().await != ConnectionState::Connected {
            return Err(BrokerError::NotConnected);
        }
        Err(BrokerError::Other(
            "place_order not implemented for YatWSEngine; use place_bag_order".to_string(),
        ))
    }

    pub async fn place_bag_order(&self, request: PlaceBagOrderRequest) -> Result<i32, BrokerError> {
        if *self.state.read().await != ConnectionState::Connected {
            return Err(BrokerError::NotConnected);
        }
        if !self.config.paper_trading {
            return Err(BrokerError::OrderFailed(
                "BAG order placement is disabled for live trading; \
                 use paper port 7497 and paper_trading=true"
                    .to_string(),
            ));
        }
        if request.legs.is_empty() {
            return Err(BrokerError::OrderFailed(
                "BAG order must have at least one leg".to_string(),
            ));
        }

        let client = self
            .client
            .read()
            .await
            .clone()
            .ok_or(BrokerError::NotConnected)?;

        // Extract box spread parameters.
        // Legs are ordered: call_low, put_low, call_high, put_high (per construct_box_spread_order).
        let symbol = request.underlying_symbol.clone();
        let expiry_str = request.legs[0].contract.expiry.clone();
        let k_low = request.legs[0].contract.strike;
        let k_high = request
            .legs
            .get(2)
            .map(|l| l.contract.strike)
            .unwrap_or(k_low);
        let quantity = request.quantity as f64;
        let limit_price = request.limit_price.unwrap_or(0.0);
        let sec_type = sec_type_for(&symbol);
        let underlying_price = (k_low + k_high) / 2.0;
        let expiry_date = parse_expiry_to_naive_date(&expiry_str)?;

        let order_id_str: String = tokio::task::spawn_blocking(move || {
            let data_ref = client.data_ref();
            let orders = client.orders();

            let (combo, order_req) = OptionsStrategyBuilder::new(
                data_ref,
                &symbol,
                underlying_price,
                quantity,
                sec_type,
            )
            .map_err(|e: IBKRError| BrokerError::Other(e.to_string()))?
            .box_spread_nearest_expiry(expiry_date, k_low, k_high)
            .map_err(|e: IBKRError| BrokerError::Other(e.to_string()))?
            .with_limit_price(limit_price)
            .with_highest_liquidity()
            .build()
            .map_err(|e: IBKRError| BrokerError::Other(e.to_string()))?;

            orders
                .place_order(combo, order_req)
                .map_err(|e: IBKRError| BrokerError::OrderFailed(e.to_string()))
        })
        .await
        .map_err(|e| BrokerError::Other(e.to_string()))??;

        order_id_str.parse::<i32>().map_err(|e| {
            BrokerError::Other(format!("order_id '{}' is not i32: {}", order_id_str, e))
        })
    }

    pub async fn cancel_order(&self, order_id: i32) -> Result<(), BrokerError> {
        if *self.state.read().await != ConnectionState::Connected {
            return Err(BrokerError::NotConnected);
        }
        let client = self
            .client
            .read()
            .await
            .clone()
            .ok_or(BrokerError::NotConnected)?;
        let id_str = order_id.to_string();
        tokio::task::spawn_blocking(move || client.orders().cancel_order(&id_str))
            .await
            .map_err(|e| BrokerError::Other(e.to_string()))?
            .map_err(map_ibkr_error)?;
        Ok(())
    }

    pub async fn cancel_all_orders(&self) -> Result<(), BrokerError> {
        if *self.state.read().await != ConnectionState::Connected {
            return Err(BrokerError::NotConnected);
        }
        let client = self
            .client
            .read()
            .await
            .clone()
            .ok_or(BrokerError::NotConnected)?;
        tokio::task::spawn_blocking(move || client.orders().cancel_all_orders_globally())
            .await
            .map_err(|e| BrokerError::Other(e.to_string()))?
            .map_err(map_ibkr_error)?;
        Ok(())
    }

    pub async fn request_positions(&self) -> Result<Vec<PositionEvent>, BrokerError> {
        if *self.state.read().await != ConnectionState::Connected {
            return Err(BrokerError::NotConnected);
        }
        let client = self
            .client
            .read()
            .await
            .clone()
            .ok_or(BrokerError::NotConnected)?;
        let client2 = client.clone();
        let positions = tokio::task::spawn_blocking(move || client.account().list_open_positions())
            .await
            .map_err(|e| BrokerError::Other(e.to_string()))?
            .map_err(map_ibkr_error)?;

        let account_id = tokio::task::spawn_blocking(move || {
            client2.account().get_account_info().map(|i| i.account_id)
        })
        .await
        .map_err(|e| BrokerError::Other(e.to_string()))?
        .map_err(map_ibkr_error)?;

        Ok(positions
            .into_iter()
            .map(|p| PositionEvent {
                account: account_id.clone(),
                symbol: p.symbol.clone(),
                position: p.quantity as i32,
                avg_cost: p.average_cost,
            })
            .collect())
    }

    pub async fn request_account(&self) -> Result<AccountInfo, BrokerError> {
        if *self.state.read().await != ConnectionState::Connected {
            return Err(BrokerError::NotConnected);
        }
        let client = self
            .client
            .read()
            .await
            .clone()
            .ok_or(BrokerError::NotConnected)?;
        let info = tokio::task::spawn_blocking(move || client.account().get_account_info())
            .await
            .map_err(|e| BrokerError::Other(e.to_string()))?
            .map_err(map_ibkr_error)?;

        Ok(AccountInfo {
            account_id: info.account_id,
            net_liquidation: info.net_liquidation,
            cash_balance: info.total_cash_value,
            buying_power: info.buying_power,
            maintenance_margin: info.maint_margin_req,
            initial_margin: info.init_margin_req,
        })
    }
}

// ---------------------------------------------------------------------------
// BrokerEngine impl
// ---------------------------------------------------------------------------

#[async_trait]
impl BrokerEngine for YatWSEngine {
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

    async fn place_order(
        &self,
        contract: OptionContract,
        action: OrderAction,
        quantity: i32,
        limit_price: f64,
    ) -> Result<i32, BrokerError> {
        self.place_order(contract, action, quantity, limit_price)
            .await
    }

    async fn place_bag_order(&self, request: PlaceBagOrderRequest) -> Result<i32, BrokerError> {
        self.place_bag_order(request).await
    }

    async fn cancel_order(&self, order_id: i32) -> Result<(), BrokerError> {
        self.cancel_order(order_id).await
    }

    async fn cancel_all_orders(&self) -> Result<(), BrokerError> {
        self.cancel_all_orders().await
    }

    async fn request_positions(&self) -> Result<Vec<PositionEvent>, BrokerError> {
        self.request_positions().await
    }

    async fn request_account(&self) -> Result<AccountInfo, BrokerError> {
        self.request_account().await
    }

    fn market_data_tx(&self) -> mpsc::Sender<MarketDataEvent> {
        self.market_data_tx.clone()
    }

    fn position_tx(&self) -> mpsc::Sender<PositionEvent> {
        self.position_tx.clone()
    }

    fn order_tx(&self) -> mpsc::Sender<OrderStatusEvent> {
        self.order_tx.clone()
    }
}

#[async_trait]
impl OptionChainProvider for YatWSEngine {
    async fn resolve_option_chain(
        &self,
        symbol: &str,
    ) -> Result<Vec<ResolvedOptionContract>, BrokerError> {
        if *self.state.read().await != ConnectionState::Connected {
            return Err(BrokerError::NotConnected);
        }
        let client = self
            .client
            .read()
            .await
            .clone()
            .ok_or(BrokerError::NotConnected)?;

        let sym_owned = symbol.to_string();
        let sec_type = sec_type_for(&sym_owned);
        let sym_for_chain = sym_owned.clone();
        let client_for_chain = client.clone();

        let chains = tokio::task::spawn_blocking(move || {
            client_for_chain
                .data_ref()
                .get_option_chain_params(&sym_for_chain, "", sec_type, 0)
                .map_err(map_ibkr_error)
        })
        .await
        .map_err(|e| BrokerError::Other(e.to_string()))??;

        let mut out = Vec::new();
        for chain in &chains {
            for exp in &chain.expirations {
                for &strike in &chain.strikes {
                    let expiry_date = parse_expiry_to_naive_date(exp)?;
                    let is_call = true;
                    let contract = Contract::option(
                        &sym_owned,
                        &expiry_date,
                        strike,
                        if is_call {
                            OptionRight::Call
                        } else {
                            OptionRight::Put
                        },
                        &chain.exchange,
                        "USD",
                    );

                    let details = tokio::task::spawn_blocking({
                        let client = client.clone();
                        move || {
                            client
                                .data_ref()
                                .get_contract_details(&contract)
                                .map_err(map_ibkr_error)
                        }
                    })
                    .await
                    .map_err(|e| BrokerError::Other(e.to_string()))??;

                    if let Some(detail) = details.into_iter().next() {
                        out.push(ResolvedOptionContract {
                            symbol: sym_owned.clone(),
                            expiry: exp.clone(),
                            strike,
                            is_call: true,
                            con_id: detail.contract.con_id,
                            exchange: detail.contract.exchange.clone(),
                            multiplier: detail
                                .contract
                                .multiplier
                                .as_deref()
                                .unwrap_or("100")
                                .parse()
                                .unwrap_or(100.0),
                            trading_class: detail
                                .contract
                                .trading_class
                                .as_deref()
                                .unwrap_or_default()
                                .to_string(),
                        });
                    }

                    let contract_put = Contract::option(
                        &sym_owned,
                        &expiry_date,
                        strike,
                        OptionRight::Put,
                        &chain.exchange,
                        "USD",
                    );
                    let details_put = tokio::task::spawn_blocking({
                        let client = client.clone();
                        move || {
                            client
                                .data_ref()
                                .get_contract_details(&contract_put)
                                .map_err(map_ibkr_error)
                        }
                    })
                    .await
                    .map_err(|e| BrokerError::Other(e.to_string()))??;

                    if let Some(detail) = details_put.into_iter().next() {
                        out.push(ResolvedOptionContract {
                            symbol: sym_owned.clone(),
                            expiry: exp.clone(),
                            strike,
                            is_call: false,
                            con_id: detail.contract.con_id,
                            exchange: detail.contract.exchange.clone(),
                            multiplier: detail
                                .contract
                                .multiplier
                                .as_deref()
                                .unwrap_or("100")
                                .parse()
                                .unwrap_or(100.0),
                            trading_class: detail
                                .contract
                                .trading_class
                                .as_deref()
                                .unwrap_or_default()
                                .to_string(),
                        });
                    }
                }
            }
        }
        Ok(out)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use broker_engine::domain::{BagOrderLeg, OrderAction, PlaceBagOrderRequest, TimeInForce};

    #[test]
    fn is_index_recognises_known_symbols() {
        assert!(is_index("SPX"));
        assert!(is_index("spx"));
        assert!(is_index("NDX"));
        assert!(is_index("XSP"));
        assert!(!is_index("SPY"));
        assert!(!is_index("AAPL"));
        assert!(!is_index("QQQ"));
    }

    #[test]
    fn parse_expiry_to_naive_date_valid() {
        let d = parse_expiry_to_naive_date("20250620").unwrap();
        assert_eq!(d.to_string(), "2025-06-20");

        let d2 = parse_expiry_to_naive_date("20261231").unwrap();
        assert_eq!(d2.to_string(), "2026-12-31");

        assert!(parse_expiry_to_naive_date("bad-date").is_err());
    }

    #[tokio::test]
    async fn yatws_engine_not_connected_rejects_place_bag_order() {
        let engine = YatWSEngine::new(BrokerConfig::default());
        let req = PlaceBagOrderRequest {
            underlying_symbol: "SPX".into(),
            currency: "USD".into(),
            exchange: "SMART".into(),
            legs: vec![BagOrderLeg {
                contract: OptionContract::new("SPX", "20250620", 5000.0, true),
                ratio: 1,
                action: OrderAction::Buy,
            }],
            quantity: 1,
            limit_price: Some(8.0),
            tif: TimeInForce::Day,
            order_action: OrderAction::Buy,
        };
        let err = engine.place_bag_order(req).await.unwrap_err();
        assert!(matches!(err, BrokerError::NotConnected));
    }

    #[tokio::test]
    async fn yatws_engine_not_connected_rejects_empty_legs() {
        let engine = YatWSEngine::new(BrokerConfig::default());
        // Manually advance to Connected state (no live client needed to test empty-leg guard).
        *engine.state.write().await = ConnectionState::Connected;

        let req = PlaceBagOrderRequest {
            underlying_symbol: "SPX".into(),
            currency: "USD".into(),
            exchange: "SMART".into(),
            legs: vec![],
            quantity: 1,
            limit_price: None,
            tif: TimeInForce::Day,
            order_action: OrderAction::Buy,
        };
        let err = engine.place_bag_order(req).await.unwrap_err();
        assert!(matches!(err, BrokerError::OrderFailed(_)));
    }
}
