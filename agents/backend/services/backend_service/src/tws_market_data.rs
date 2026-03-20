//! Real TWS/IB Gateway market data: connect, subscribe to configured symbols, forward
//! bid/ask/last ticks into the shared snapshot and NATS.
//!
//! Enable with `market_data.provider = "tws"` and ensure TWS or IB Gateway is running.
//! Env: TWS_HOST, TWS_PORT (optional; if unset, tries TWS then IB Gateway ports), TWS_CLIENT_ID.
//!
//! Optional RTH filter: set `market_data.use_rth = true` to forward ticks only when time is
//! within the regular trading window (09:30–16:00 ET). Per-symbol tradingHours from
//! ContractDetails are not yet used; see MARKET_DATA_SUBSCRIPTIONS_AND_HOURS.md.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use backoff::backoff::Backoff;
use backoff::exponential::ExponentialBackoffBuilder;
use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use ibapi::contracts::Contract;
use ibapi::market_data::realtime::{TickType, TickTypes};
use ibapi::Client;
use tracing::{info, warn};

use api::SharedSnapshot;
use broker_engine::QuoteQuality;
use ibapi::market_data::realtime::TickAttribute;
use market_data::{MarketDataEvent, MarketDataEventBuilder};
use strategy::StrategySignal;
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio::sync::watch;

use crate::{handle_market_event, tws_env::parse_tws_client_id};

/// Conventional ports: TWS paper/live, then IB Gateway paper/live. Paper first when autodetecting.
const PORTS_AUTODETECT: &[u16] = &[
    7497, // TWS paper
    4002, // IB Gateway paper
    7496, // TWS live
    4001, // IB Gateway live
];

/// US Eastern (EST) offset for RTH window. Does not account for DST (approximation).
const EST_OFFSET_SECS: i32 = -5 * 3600;

#[inline]
fn tick_attribute_to_quote_quality(attrib: &TickAttribute) -> QuoteQuality {
    QuoteQuality::from_tick_attrib(attrib.can_auto_execute, attrib.past_limit, attrib.pre_open)
}

/// Returns true if `utc_now` is within 09:30–16:00 Eastern (EST). Used when `use_rth` is true.
fn is_within_rth(utc_now: DateTime<Utc>) -> bool {
    let est = FixedOffset::east_opt(EST_OFFSET_SECS).expect("valid offset");
    let et = utc_now.with_timezone(&est);
    let t = et.time();
    let start = chrono::NaiveTime::from_hms_opt(9, 30, 0).expect("valid time");
    let end = chrono::NaiveTime::from_hms_opt(16, 0, 0).expect("valid time");
    t >= start && t < end
}

/// Messages from TWS subscription tasks to the main processing loop.
enum TwsMarketMsg {
    /// Sent once at subscription start: carries the contract_id for the symbol.
    Setup { symbol: String, contract_id: i64 },
    /// Sent for each price tick.
    Tick { symbol: String, tick: TickTypes },
}

/// Spawn a task that connects to TWS/IB Gateway, subscribes to each symbol, and forwards
/// price ticks (bid/ask) into `handle_market_event` (state + NATS).
/// If TWS_PORT is not set, tries PORTS_AUTODETECT in order (paper before live).
/// When `use_rth` is true, only forwards ticks when current time is within 09:30–16:00 ET.
pub fn spawn_tws_market_data(
    symbols: Vec<String>,
    state: SharedSnapshot,
    strategy_signal: UnboundedSender<StrategySignal>,
    strategy_toggle: watch::Receiver<bool>,
    nats: Arc<Option<crate::nats_integration::NatsIntegration>>,
    use_rth: bool,
) {
    if symbols.is_empty() {
        warn!("TWS market data: no symbols configured");
        return;
    }

    let host = std::env::var("TWS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let ports: Vec<u16> = match std::env::var("TWS_PORT") {
        Ok(s) => match s.parse::<u16>() {
            Ok(p) => vec![p],
            Err(_) => PORTS_AUTODETECT.to_vec(),
        },
        Err(_) => PORTS_AUTODETECT.to_vec(),
    };
    let client_id: i32 = parse_tws_client_id(0);

    let backoff: backoff::ExponentialBackoff = ExponentialBackoffBuilder::new()
        .with_initial_interval(Duration::from_secs(2))
        .with_multiplier(2.0)
        .with_max_interval(Duration::from_secs(60))
        .with_randomization_factor(0.0)
        .with_max_elapsed_time(None)
        .build();

    tokio::spawn(async move {
        let mut backoff = backoff;
        let mut attempt: u32 = 0;
        loop {
            let mut last_err = None;
            for &port in &ports {
                let address = format!("{}:{}", host, port);
                match Client::connect(&address, client_id).await {
                    Ok(client) => {
                        let client = Arc::new(client);
                        backoff.reset();
                        attempt = 0;
                        info!(%address, "TWS market data connected");
                        {
                            let mut snap = state.write().await;
                            snap.metrics.tws_ok = true;
                            snap.metrics.tws_address = Some(address.clone());
                            snap.touch();
                        }
                        if let Err(e) = run_tws_subscriptions(
                            client,
                            &symbols,
                            state.clone(),
                            strategy_signal.clone(),
                            strategy_toggle.clone(),
                            nats.clone(),
                            use_rth,
                        )
                        .await
                        {
                            warn!(error = %e, "TWS market data loop ended");
                        }
                        last_err = None;
                        break;
                    }
                    Err(e) => {
                        last_err = Some((address, e));
                    }
                }
            }
            if let Some((address, e)) = last_err {
                attempt = attempt.saturating_add(1);
                let delay = backoff.next_backoff().unwrap_or(Duration::from_secs(60));
                let mut snap = state.write().await;
                snap.metrics.tws_ok = false;
                snap.metrics.tws_address = None;
                snap.touch();
                drop(snap);
                warn!(
                    error = %e,
                    %address,
                    attempt,
                    delay_secs = delay.as_secs(),
                    "TWS connection failed (tried all ports), reconnecting…"
                );
                tokio::time::sleep(delay).await;
            }
        }
    });
}

/// Per-symbol state: last bid/ask for building full events from ticks.
#[derive(Default)]
struct SymbolState {
    contract_id: i64,
    bid: f64,
    ask: f64,
    attrib: TickAttribute,
}

async fn run_tws_subscriptions(
    client: Arc<Client>,
    symbols: &[String],
    state: SharedSnapshot,
    strategy_signal: UnboundedSender<StrategySignal>,
    strategy_toggle: watch::Receiver<bool>,
    nats: Arc<Option<crate::nats_integration::NatsIntegration>>,
    use_rth: bool,
) -> anyhow::Result<()> {
    // Set snapshot account_id from TWS managed accounts so TUI shows real account
    if let Ok(accounts) = client.managed_accounts().await {
        if let Some(account_id) = accounts.first() {
            let account_id = account_id.trim().to_string();
            if !account_id.is_empty() {
                let mut snap = state.write().await;
                snap.account_id = account_id.clone();
                snap.touch();
                info!(%account_id, "TWS account id set in snapshot");
            }
        }
    } else {
        warn!("TWS managed_accounts not available, snapshot account_id unchanged");
    }

    let (tx, mut rx) = mpsc::unbounded_channel::<TwsMarketMsg>();

    for symbol in symbols {
        let client = Arc::clone(&client);
        let tx = tx.clone();
        let symbol = symbol.clone();
        tokio::spawn(async move {
            let contract = Contract::stock(&symbol).build();
            let contract_id = contract.contract_id as i64;
            if let Err(e) = tx.send(TwsMarketMsg::Setup {
                symbol: symbol.clone(),
                contract_id,
            }) {
                warn!(%symbol, error = %e, "failed to send setup for contract_id");
                return;
            }
            let mut sub = match client.market_data(&contract).subscribe().await {
                Ok(s) => s,
                Err(e) => {
                    warn!(%symbol, error = %e, "TWS market_data subscribe failed");
                    return;
                }
            };
            while let Some(result) = sub.next().await {
                match result {
                    Ok(tick) => {
                        if tx.send(TwsMarketMsg::Tick {
                            symbol: symbol.clone(),
                            tick,
                        }).is_err() {
                            break;
                        }
                    }
                    Err(e) => warn!(%symbol, error = %e, "TWS tick error"),
                }
            }
        });
    }
    drop(tx);

    let mut per_symbol: std::collections::HashMap<String, SymbolState> =
        std::collections::HashMap::new();

    while let Some(msg) = rx.recv().await {
        match msg {
            TwsMarketMsg::Setup { symbol, contract_id } => {
                let entry = per_symbol.entry(symbol.clone()).or_default();
                entry.contract_id = contract_id;
            }
            TwsMarketMsg::Tick { symbol, tick } => {
                match &tick {
                    TickTypes::Price(p) => {
                        let price = p.price;
                        match p.tick_type {
                            TickType::Bid | TickType::DelayedBid | TickType::DelayedBidOption => {
                                let entry = per_symbol.entry(symbol.clone()).or_default();
                                entry.bid = price;
                                entry.attrib = p.attributes.clone();
                            }
                            TickType::Ask | TickType::DelayedAsk | TickType::DelayedAskOption => {
                                let entry = per_symbol.entry(symbol.clone()).or_default();
                                entry.ask = price;
                                entry.attrib = p.attributes.clone();
                            }
                            TickType::Last | TickType::DelayedLast => {
                                let entry = per_symbol.entry(symbol.clone()).or_default();
                                if entry.bid <= 0.0 {
                                    entry.bid = price;
                                }
                                if entry.ask <= 0.0 {
                                    entry.ask = price;
                                }
                                entry.attrib = p.attributes.clone();
                            }
                            _ => continue,
                        }
                    }
                    _ => continue,
                }

                let st = match per_symbol.get(&symbol) {
                    Some(s) if s.bid > 0.0 && s.ask > 0.0 => s,
                    _ => continue,
                };

                let now = Utc::now();
                if use_rth && !is_within_rth(now) {
                    continue;
                }

                let quote_quality = tick_attribute_to_quote_quality(&st.attrib).bits() as u32;
                let event = MarketDataEventBuilder::default()
                    .contract_id(st.contract_id)
                    .symbol(symbol.clone())
                    .bid(st.bid)
                    .ask(st.ask)
                    .last(0.0)
                    .volume(0)
                    .timestamp(now)
                    .quote_quality(quote_quality)
                    .build()
                    .unwrap();
                let running = *strategy_toggle.borrow();
                handle_market_event(
                    &state,
                    &strategy_signal,
                    &event,
                    running,
                    nats.as_ref().as_ref(),
                )
                .await;
            }
        }
    }

    Ok(())
}
