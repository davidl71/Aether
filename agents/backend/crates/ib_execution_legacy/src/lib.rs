//! Legacy IBKR/TWS execution adapter (opt-in tooling only).
//!
//! Excluded from the default workspace build. Retains order placement, combo
//! placement, cancellation, and resolved contract lookup for explicit legacy
//! workflows — **not** the default product path (`docs/DATA_EXPLORATION_MODE.md`).

use std::sync::Arc;

use async_trait::async_trait;
use broker_engine::{BrokerConfig, BrokerError, ConnectionState, OptionContract};
use broker_execution_legacy::{
    BagOrderLeg, BrokerExecution, OptionChainProvider, OrderAction, PlaceBagOrderRequest,
    ResolvedOptionContract, TimeInForce,
};
use futures::future::try_join_all;
use ibapi::contracts::{Contract, LegAction};
use ibapi::Client as IbClient;
use ibapi::Error as IbError;
use tokio::sync::RwLock;

/// Legacy execution adapter for explicit opt-in workflows only.
pub struct IbExecutionAdapter {
    config: BrokerConfig,
    state: Arc<RwLock<ConnectionState>>,
    client: Arc<RwLock<Option<Arc<IbClient>>>>,
}

impl IbExecutionAdapter {
    pub fn new(config: BrokerConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            client: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn connect(&self) -> Result<(), BrokerError> {
        *self.state.write().await = ConnectionState::Connecting;
        let address = format!("{}:{}", self.config.host, self.config.port);
        match IbClient::connect(&address, self.config.client_id as i32).await {
            Ok(client) => {
                *self.client.write().await = Some(Arc::new(client));
                *self.state.write().await = ConnectionState::Connected;
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
        Ok(())
    }

    pub async fn state(&self) -> ConnectionState {
        self.state.read().await.clone()
    }

    async fn request_option_chain(&self, symbol: &str) -> Result<Vec<OptionContract>, BrokerError> {
        use ibapi::contracts::{OptionChain, SecurityType};

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
}

fn to_ib_tif(tif: TimeInForce) -> ibapi::orders::builder::TimeInForce {
    match tif {
        TimeInForce::Day => ibapi::orders::builder::TimeInForce::Day,
        TimeInForce::GTC => ibapi::orders::builder::TimeInForce::GoodTillCancel,
        TimeInForce::IOC => ibapi::orders::builder::TimeInForce::ImmediateOrCancel,
        TimeInForce::FOK => ibapi::orders::builder::TimeInForce::FillOrKill,
    }
}

fn is_index(symbol: &str) -> bool {
    matches!(symbol.to_uppercase().as_str(), "SPX" | "NDX" | "XSP")
}

fn exchange_for_option(symbol: &str) -> &'static str {
    if is_index(symbol) {
        "CBOE"
    } else {
        "SMART"
    }
}

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
            BrokerError::ContractError(format!("contract_details failed: {}", e))
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
    Ok(details)
}

async fn resolve_con_id(client: &Arc<IbClient>, leg: &OptionContract) -> Result<i32, BrokerError> {
    let detail = resolve_contract_details(client, leg)
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| BrokerError::ContractError("missing contract details".to_string()))?;
    let con_id = detail.contract.contract_id;
    if con_id == 0 {
        return Err(BrokerError::ContractError(
            "contract_details returned con_id=0".to_string(),
        ));
    }
    Ok(con_id)
}

#[async_trait]
impl BrokerExecution for IbExecutionAdapter {
    async fn place_order(
        &self,
        contract: OptionContract,
        action: OrderAction,
        quantity: i32,
        limit_price: f64,
    ) -> Result<i32, BrokerError> {
        use common::expiry::parse_expiry_yyyy_mm_dd;

        if *self.state.read().await != ConnectionState::Connected {
            return Err(BrokerError::NotConnected);
        }
        let arc = self.client.read().await.clone();
        let client = arc.as_ref().ok_or(BrokerError::NotConnected)?;
        let (y, m, d) = parse_expiry_yyyy_mm_dd(&contract.expiry).map_err(BrokerError::Other)?;
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

    async fn place_bag_order(&self, request: PlaceBagOrderRequest) -> Result<i32, BrokerError> {
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

        let client = self
            .client
            .read()
            .await
            .clone()
            .ok_or(BrokerError::NotConnected)?;

        let con_ids: Vec<i32> = try_join_all(request.legs.iter().map(|leg| {
            let client = client.clone();
            let contract = leg.contract.clone();
            async move {
                match contract.con_id {
                    Some(cid) => Ok::<i32, BrokerError>(cid),
                    None => resolve_con_id(&client, &contract).await,
                }
            }
        }))
        .await?;

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
                    .time_in_force(to_ib_tif(request.tif))
                    .submit()
                    .await
            }
            OrderAction::Sell => {
                client
                    .order(&bag_contract)
                    .sell(qty)
                    .limit(limit_price)
                    .time_in_force(to_ib_tif(request.tif))
                    .submit()
                    .await
            }
        };
        result.map_err(|e: IbError| BrokerError::OrderFailed(e.to_string()))?;
        Ok(order_id)
    }

    async fn cancel_order(&self, order_id: i32) -> Result<(), BrokerError> {
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

    async fn cancel_all_orders(&self) -> Result<(), BrokerError> {
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
}

#[async_trait]
impl OptionChainProvider for IbExecutionAdapter {
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

        let legs = self.request_option_chain(symbol).await?;
        if legs.is_empty() {
            return Ok(Vec::new());
        }

        try_join_all(legs.iter().map(|leg| {
            let client = client.clone();
            let leg = leg.clone();
            async move {
                let details = resolve_contract_details(&client, &leg).await?;
                let detail = details.into_iter().next().ok_or_else(|| {
                    BrokerError::ContractError(format!(
                        "no contract details for {} {} {:.0} {}",
                        leg.symbol,
                        leg.expiry,
                        leg.strike,
                        if leg.is_call { "C" } else { "P" }
                    ))
                })?;
                Ok::<ResolvedOptionContract, BrokerError>(ResolvedOptionContract {
                    symbol: leg.symbol,
                    expiry: leg.expiry,
                    strike: leg.strike,
                    is_call: leg.is_call,
                    con_id: detail.contract.contract_id,
                    exchange: detail.contract.exchange.0.clone(),
                    multiplier: detail.contract.multiplier.parse().unwrap_or(100.0),
                    trading_class: detail.contract.trading_class.clone(),
                })
            }
        }))
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn place_bag_order_rejects_when_not_connected() {
        let adapter = IbExecutionAdapter::new(BrokerConfig::default());
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
        let adapter = IbExecutionAdapter::new(BrokerConfig::default());
        *adapter.state.write().await = ConnectionState::Connected;
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

    #[tokio::test]
    async fn place_bag_order_rejects_live_trading() {
        let adapter = IbExecutionAdapter::new(BrokerConfig {
            host: "127.0.0.1".into(),
            port: 7496,
            client_id: 0,
            paper_trading: false,
        });
        *adapter.state.write().await = ConnectionState::Connected;
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
        assert!(matches!(err, BrokerError::OrderFailed(_)));
    }
}
