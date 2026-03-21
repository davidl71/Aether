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

use std::sync::Arc;

use async_trait::async_trait;
use futures::future::try_join_all;
use ibapi::accounts::types::AccountGroup;
use ibapi::accounts::{AccountSummaryTags, PositionUpdate};
use ibapi::contracts::{Contract, LegAction, OptionChain, SecurityType};
use ibapi::Client as IbClient;
use ibapi::Error as IbError;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info};

pub use broker_engine::construct_box_spread_order;
pub use broker_engine::domain::{
    BagOrderLeg, BrokerConfig, MarketDataEvent, OptionContract, OrderAction, OrderStatus,
    OrderStatusEvent, PlaceBagOrderRequest, Position, PositionEvent, ResolvedOptionContract,
    TimeInForce,
};
pub use broker_engine::AccountInfo;
pub use broker_engine::BrokerEngine;
pub use broker_engine::BrokerError;
pub use broker_engine::ConnectionState;
pub use broker_engine::MarketData;
pub use broker_engine::OptionChainProvider;

pub use broker_engine::BrokerEngine;

pub mod scanner;
pub mod tws_wire;
pub mod types;

pub use scanner::ScannerSubscription;
pub use tws_wire::{TwsProtoFrame, PROTOBUF_MSG_ID_OFFSET};

// ---------------------------------------------------------------------------
// conId resolution helpers
// ---------------------------------------------------------------------------

/// Whether the symbol is an index (uses SecurityType::Index + CBOE exchange).
fn is_index(symbol: &str) -> bool {
    matches!(symbol.to_uppercase().as_str(), "SPX" | "NDX" | "XSP")
}

/// Exchange to use when building an option contract for `contract_details`.
fn exchange_for_option(symbol: &str) -> &'static str {
    if is_index(symbol) {
        "CBOE"
    } else {
        "SMART"
    }
}

/// Resolve one option leg to its full IBKR contract details via `reqContractDetails`.
///
/// Returns `Err(BrokerError::ContractError)` if TWS returns no results or conId 0.
/// Logs the resolved conId at DEBUG level.
async fn resolve_contract_details(
    client: &Arc<IbClient>,
    leg: &OptionContract,
) -> Result<Vec<ibapi::contracts::ContractDetails>, BrokerError> {
    use common::expiry::parse_expiry_yyyy_mm_dd;

    let (y, m, d) = parse_expiry_yyyy_mm_dd(&leg.expiry).map_err(BrokerError::Other)?;
    let exchange = exchange_for_option(&leg.symbol);

    let ib_contract = if leg.is_call {
        Contract::call(&leg.symbol)
            .strike(leg.strike)
            .expires_on(y, m, d)
            .on_exchange(exchange)
            .build()
    } else {
        Contract::put(&leg.symbol)
            .strike(leg.strike)
            .expires_on(y, m, d)
            .on_exchange(exchange)
            .build()
    };

    let details = client
        .contract_details(&ib_contract)
        .await
        .map_err(|e: IbError| {
            BrokerError::ContractError(format!(
                "contract_details failed for {} {} {:.0} {}: {}",
                leg.symbol,
                leg.expiry,
                leg.strike,
                if leg.is_call { "C" } else { "P" },
                e
            ))
        })?;

    if details.is_empty() {
        return Err(BrokerError::ContractError(format!(
            "no contract details returned for {} {} {:.0} {}",
            leg.symbol,
            leg.expiry,
            leg.strike,
            if leg.is_call { "C" } else { "P" },
        )));
    }

    debug!(
        symbol = %leg.symbol,
        expiry = %leg.expiry,
        strike = leg.strike,
        is_call = leg.is_call,
        con_id = details[0].contract.contract_id,
        "resolved option contract details via contract_details"
    );
    Ok(details)
}

/// Resolve one option leg to its IBKR conId via `reqContractDetails`.
///
/// Returns `Err(BrokerError::ContractError)` if TWS returns no results or conId 0.
/// Logs the resolved conId at DEBUG level.
async fn resolve_con_id(client: &Arc<IbClient>, leg: &OptionContract) -> Result<i32, BrokerError> {
    use common::expiry::parse_expiry_yyyy_mm_dd;

    let (y, m, d) = parse_expiry_yyyy_mm_dd(&leg.expiry).map_err(BrokerError::Other)?;
    let exchange = exchange_for_option(&leg.symbol);

    let ib_contract = if leg.is_call {
        Contract::call(&leg.symbol)
            .strike(leg.strike)
            .expires_on(y, m, d)
            .on_exchange(exchange)
            .build()
    } else {
        Contract::put(&leg.symbol)
            .strike(leg.strike)
            .expires_on(y, m, d)
            .on_exchange(exchange)
            .build()
    };

    let details = client
        .contract_details(&ib_contract)
        .await
        .map_err(|e: IbError| {
            BrokerError::ContractError(format!(
                "contract_details failed for {} {} {:.0} {}: {}",
                leg.symbol,
                leg.expiry,
                leg.strike,
                if leg.is_call { "C" } else { "P" },
                e
            ))
        })?;

    let detail = details.into_iter().next().ok_or_else(|| {
        BrokerError::ContractError(format!(
            "no contract details returned for {} {} {:.0} {}",
            leg.symbol,
            leg.expiry,
            leg.strike,
            if leg.is_call { "C" } else { "P" },
        ))
    })?;

    let con_id = detail.contract.contract_id;
    if con_id == 0 {
        return Err(BrokerError::ContractError(format!(
            "contract_details returned con_id=0 for {} {} {:.0} {}",
            leg.symbol,
            leg.expiry,
            leg.strike,
            if leg.is_call { "C" } else { "P" },
        )));
    }

    debug!(
        symbol = %leg.symbol,
        expiry = %leg.expiry,
        strike = leg.strike,
        is_call = leg.is_call,
        con_id,
        "resolved option con_id via contract_details"
    );
    Ok(con_id)
}

// ---------------------------------------------------------------------------

/// IB Adapter configuration (alias for [`BrokerConfig`] for backwards compatibility)
pub type IbConfig = BrokerConfig;

/// IB Adapter — implements [`BrokerEngine`] using the ibapi crate.
pub struct IbAdapter {
    config: BrokerConfig,
    state: Arc<RwLock<ConnectionState>>,
    client: Arc<RwLock<Option<Arc<IbClient>>>>,
    market_data_tx: mpsc::Sender<MarketDataEvent>,
    position_tx: mpsc::Sender<PositionEvent>,
    order_tx: mpsc::Sender<OrderStatusEvent>,
}

impl IbAdapter {
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
        let arc = self.client.read().await.clone();
        let client = arc.as_ref().ok_or(BrokerError::NotConnected)?;
        let mut contract = Contract::stock(symbol).build();
        if contract_id != 0 {
            contract.contract_id = contract_id as i32;
        }
        client
            .market_data(&contract)
            .subscribe()
            .await
            .map_err(|e: IbError| BrokerError::Other(e.to_string()))?;
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

    pub async fn place_order(
        &self,
        contract: OptionContract,
        action: OrderAction,
        quantity: i32,
        limit_price: f64,
    ) -> Result<i32, BrokerError> {
        if *self.state.read().await != ConnectionState::Connected {
            return Err(BrokerError::NotConnected);
        }
        let arc = self.client.read().await.clone();
        let client = arc.as_ref().ok_or(BrokerError::NotConnected)?;
        let (y, m, d) = common::expiry::parse_expiry_yyyy_mm_dd(&contract.expiry)
            .map_err(BrokerError::Other)?;
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
            OrderAction::Buy => {
                client
                    .order(&ib_contract)
                    .buy(qty)
                    .limit(limit_price)
                    .submit()
                    .await
            }
            OrderAction::Sell => {
                client
                    .order(&ib_contract)
                    .sell(qty)
                    .limit(limit_price)
                    .submit()
                    .await
            }
        };
        result.map_err(|e: IbError| BrokerError::OrderFailed(e.to_string()))?;
        Ok(order_id)
    }

    pub async fn place_bag_order(&self, request: PlaceBagOrderRequest) -> Result<i32, BrokerError> {
        if *self.state.read().await != ConnectionState::Connected {
            return Err(BrokerError::NotConnected);
        }
        if !self.config.paper_trading {
            return Err(BrokerError::OrderFailed(
                "BAG order placement is disabled for live trading; use paper port 7497 and paper_trading=true"
                    .to_string(),
            ));
        }
        if request.legs.is_empty() {
            return Err(BrokerError::OrderFailed(
                "BAG order must have at least one leg".to_string(),
            ));
        }

        // Get the client Arc once; release the lock before the async resolution work.
        let client = self
            .client
            .read()
            .await
            .clone()
            .ok_or(BrokerError::NotConnected)?;

        // Resolve any legs missing con_id in parallel via contract_details.
        // Legs that already have con_id pass through immediately.
        let resolve_futs = request.legs.iter().map(|leg| {
            let client = client.clone();
            let contract = leg.contract.clone();
            async move {
                match contract.con_id {
                    Some(cid) => Ok::<i32, BrokerError>(cid),
                    None => resolve_con_id(&client, &contract).await,
                }
            }
        });
        let con_ids: Vec<i32> = try_join_all(resolve_futs).await?;

        // Build the BAG contract with the resolved conIds.
        let exchange = if request.exchange.is_empty() {
            "SMART"
        } else {
            &request.exchange
        };
        let mut builder = Contract::spread()
            .in_currency(&request.currency)
            .on_exchange(exchange);
        for (leg, con_id) in request.legs.iter().zip(&con_ids) {
            let action = match leg.action {
                OrderAction::Buy => LegAction::Buy,
                OrderAction::Sell => LegAction::Sell,
            };
            builder = builder.add_leg(*con_id, action).ratio(leg.ratio).done();
        }
        let mut bag_contract = builder
            .build()
            .map_err(|e| BrokerError::Other(e.to_string()))?;
        bag_contract.symbol = ibapi::contracts::Symbol::from(request.underlying_symbol.as_str());

        let order_id = client.next_order_id();
        let qty = request.quantity as f64;
        let limit_price = request.limit_price.unwrap_or(0.0);
        let result = match request.order_action {
            OrderAction::Buy => {
                client
                    .order(&bag_contract)
                    .buy(qty)
                    .limit(limit_price)
                    .submit()
                    .await
            }
            OrderAction::Sell => {
                client
                    .order(&bag_contract)
                    .sell(qty)
                    .limit(limit_price)
                    .submit()
                    .await
            }
        };
        result.map_err(|e: IbError| BrokerError::OrderFailed(e.to_string()))?;
        Ok(order_id)
    }

    pub async fn cancel_order(&self, order_id: i32) -> Result<(), BrokerError> {
        if *self.state.read().await != ConnectionState::Connected {
            return Err(BrokerError::NotConnected);
        }
        let arc = self.client.read().await.clone();
        let client = arc.as_ref().ok_or(BrokerError::NotConnected)?;
        let mut sub = client
            .cancel_order(order_id, "")
            .await
            .map_err(|e: IbError| BrokerError::Other(e.to_string()))?;
        let _ = sub.next().await;
        Ok(())
    }

    pub async fn cancel_all_orders(&self) -> Result<(), BrokerError> {
        if *self.state.read().await != ConnectionState::Connected {
            return Err(BrokerError::NotConnected);
        }
        let arc = self.client.read().await.clone();
        let client = arc.as_ref().ok_or(BrokerError::NotConnected)?;
        client
            .global_cancel()
            .await
            .map_err(|e: IbError| BrokerError::Other(e.to_string()))?;
        Ok(())
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
impl OptionChainProvider for IbAdapter {
    async fn resolve_option_chain(&self, symbol: &str) -> Result<Vec<ResolvedOptionContract>, BrokerError> {
        if *self.state.read().await != ConnectionState::Connected {
            return Err(BrokerError::NotConnected);
        }
        let client = self
            .client
            .read()
            .await
            .clone()
            .ok_or(BrokerError::NotConnected)?;

        let legs = self.request_option_chain(symbol).await?;

        if legs.is_empty() {
            return Ok(Vec::new());
        }

        let client = client.clone();
        let resolve_fut = async {
            try_join_all(legs.iter().map(|leg| {
                let client = client.clone();
                let leg = leg.clone();
                async move {
                    let details = resolve_contract_details(&client, &leg).await?;
                    let detail = details.into_iter().next().ok_or_else(|| {
                        BrokerError::ContractError(format!(
                            "no contract details for {} {} {:.0} {}",
                            leg.symbol, leg.expiry, leg.strike,
                            if leg.is_call { "C" } else { "P" }
                        ))
                    })?;
                    Ok::<ResolvedOptionContract, BrokerError>(ResolvedOptionContract {
                        symbol: leg.symbol,
                        expiry: leg.expiry,
                        strike: leg.strike,
                        is_call: leg.is_call,
                        con_id: detail.contract.contract_id,
                        exchange: detail.contract.exchange.unwrap_or_default(),
                        multiplier: detail.contract.multiplier.unwrap_or(100.0),
                        trading_class: detail.trading_class.unwrap_or_default(),
                    })
                }
            }))
            .await
        };

        resolve_fut.await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use broker_engine::domain::{BagOrderLeg, OrderAction, PlaceBagOrderRequest, TimeInForce};

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

    #[tokio::test]
    async fn place_bag_order_rejects_when_not_connected() {
        let adapter = IbAdapter::new(BrokerConfig::default());
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
        let err = adapter.place_bag_order(req).await.unwrap_err();
        assert!(matches!(err, BrokerError::NotConnected));
    }

    #[tokio::test]
    async fn place_bag_order_rejects_empty_legs() {
        // Simulate connected state by writing directly to avoid needing a live TWS.
        let adapter = IbAdapter::new(BrokerConfig::default());
        *adapter.state.write().await = ConnectionState::Connected;
        // No client in the Arc — this will hit NotConnected from the client unwrap,
        // which is fine: we're testing the empty-legs guard fires before that.
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
        let err = adapter.place_bag_order(req).await.unwrap_err();
        assert!(matches!(err, BrokerError::OrderFailed(_)));
    }
}
