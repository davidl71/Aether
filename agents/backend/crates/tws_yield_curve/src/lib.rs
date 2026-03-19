//! TWS-only yield curve: connect to TWS, fetch option chain + option quotes, return box spread
//! opportunities as `Vec<serde_json::Value>` (each `{ "spread": BoxSpreadInput }`).
//!
//! No NATS dependency. Use from CLI (direct), tests, or backend (which may then publish to NATS).
//! Env: TWS_HOST, TWS_PORT (optional), TWS_CLIENT_ID (default 10 → client id 12), YIELD_CURVE_REFERENCE_SPOT_{SYMBOL}.
//! For out-of-hours/closing: YIELD_CURVE_USE_CLOSING=1. Market data type: YIELD_CURVE_MARKET_DATA_TYPE=delayed|delayed_frozen|frozen
//! (default delayed for closing; delayed/delayed_frozen need no subscription; frozen needs same as live).
//! Log full session to a file: YIELD_CURVE_LOG_FILE=/path/to/session.log (tracing only; for stdout+stderr use shell redirect or tee).

use chrono::{Datelike, Timelike, Utc};
use common::expiry::parse_expiry_yyyy_mm_dd;
use ibapi::contracts::Symbol;
use ibapi::contracts::{Contract, LegAction, OptionChain, SecurityType};
use ibapi::market_data::realtime::{TickType, TickTypes};
use ibapi::market_data::MarketDataType;
use ibapi::Client;
use serde_json::{json, Value};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};

const PORTS: &[u16] = &[7497, 4002, 7496, 4001];
/// Default base client ID; we use base + 2. 10 avoids collision with tools using 0/1/2.
const DEFAULT_CLIENT_ID_BASE: i32 = 10;
const CLIENT_ID_OFFSET: i32 = 2;

/// Index symbols use SecurityType::Index and CBOE; others use Stock and SMART.
fn underlying_security_type(symbol: &str) -> SecurityType {
    match symbol.to_uppercase().as_str() {
        "SPX" | "NDX" | "XSP" => SecurityType::Index,
        _ => SecurityType::Stock,
    }
}

/// Exchange for building option contracts and market data (CBOE for index, SMART for stock).
fn option_exchange(symbol: &str) -> &'static str {
    match symbol.to_uppercase().as_str() {
        "SPX" | "NDX" | "XSP" => "CBOE",
        _ => "SMART",
    }
}
/// Exchange for reqSecDefOptParams only. Empty string so TWS returns data; specifying an exchange often returns no data.
fn option_chain_exchange(_symbol: &str) -> &'static str {
    ""
}

/// Parse TWS_CLIENT_ID env var; use default if unset or invalid. Logs a warning when set but unparseable.
fn parse_tws_client_id(default: i32) -> i32 {
    match std::env::var("TWS_CLIENT_ID") {
        Ok(s) => match s.trim().parse::<i32>() {
            Ok(n) => n,
            Err(_) => {
                tracing::warn!(
                    value = %s,
                    default = %default,
                    "TWS_CLIENT_ID is set but not a valid i32; using default (use an integer to avoid this)"
                );
                default
            }
        },
        Err(_) => default,
    }
}
const QUOTE_TIMEOUT_MS: u64 = 3000;
/// Longer timeout when using Frozen (closing) data so TWS has time to send last bid/ask.
const QUOTE_TIMEOUT_CLOSING_MS: u64 = 12_000;
const REFERENCE_SPOT_DEFAULT: f64 = 6000.0;
const STRIKE_WIDTH: f64 = 4.0;
const MIN_LIQUIDITY_SCORE: f64 = 50.0;

fn reference_spot(symbol: &str) -> f64 {
    let key = format!("YIELD_CURVE_REFERENCE_SPOT_{}", symbol.to_uppercase());
    std::env::var(&key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(REFERENCE_SPOT_DEFAULT)
}

/// Normalize expiry string from option chain (e.g. "2025-03-21" or "20250321") to YYYYMMDD for parsing.
fn normalize_expiry(expiry: &str) -> Option<String> {
    let digits: String = expiry.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() >= 8 {
        Some(digits[..8].to_string())
    } else {
        None
    }
}

fn days_to_expiry(expiry: &str) -> Option<i32> {
    let normalized = normalize_expiry(expiry)?;
    let (y, m, d) = parse_expiry_yyyy_mm_dd(&normalized).ok()?;
    let expiry_date = chrono::NaiveDate::from_ymd_opt(y as i32, m as u32, d as u32)?;
    let now = Utc::now().date_naive();
    Some((expiry_date - now).num_days() as i32)
}

/// Connect to TWS (tries paper then live ports, or TWS_PORT if set). Returns client and address on success.
/// Single connection only; client_id is base (from TWS_CLIENT_ID or default) + CLIENT_ID_OFFSET.
async fn connect_tws() -> Result<(Client, String), String> {
    let host = std::env::var("TWS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let base_id: i32 = parse_tws_client_id(DEFAULT_CLIENT_ID_BASE);
    let client_id = base_id + CLIENT_ID_OFFSET;

    let ports: Vec<u16> = std::env::var("TWS_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .map(|p| vec![p])
        .unwrap_or_else(|| PORTS.to_vec());

    info!(client_id = %client_id, ports = ?ports, "TWS yield curve: single connection only");
    let mut last_err = None;
    for port in &ports {
        let address = format!("{}:{}", host, port);
        match Client::connect(&address, client_id).await {
            Ok(client) => {
                info!(%client_id, %address, "TWS yield curve: connected (this process uses exactly one TWS client)");
                return Ok((client, address));
            }
            Err(e) => last_err = Some(format!("{}: {}", address, e)),
        }
    }
    Err(format!(
        "TWS connection failed (tried all ports). Last error: {}",
        last_err.unwrap_or_else(|| "unknown".to_string())
    ))
}

/// Resolve index underlying to conid so we can pass it to option_chain (some TWS versions want non-zero underlyingConId for IND).
async fn resolve_index_conid(client: &Client, symbol: &str) -> i32 {
    if underlying_security_type(symbol) != SecurityType::Index {
        return 0;
    }
    let index_contract = Contract::index(symbol);
    match client.contract_details(&index_contract).await {
        Ok(details) if !details.is_empty() => {
            let conid = details[0].contract.contract_id;
            if conid != 0 {
                debug!(%symbol, conid, "resolved index underlying for option_chain");
            }
            conid
        }
        _ => 0,
    }
}

/// Request option chain for symbol; returns (expirations, strikes, trading_class) or error.
/// Uses Index for SPX/NDX/XSP (avoids "Invalid contract id" from TWS when using STK).
/// The returned trading_class (e.g. "SPX" or "SPXW") must be used when requesting market data for those options.
async fn request_option_chain(
    client: &Client,
    symbol: &str,
) -> Result<(Vec<String>, Vec<f64>, String), String> {
    let sec_type = underlying_security_type(symbol);
    let exchange_for_chain = option_chain_exchange(symbol);
    let underlying_conid = resolve_index_conid(client, symbol).await;
    debug!(%symbol, exchange_for_chain = %exchange_for_chain, ?sec_type, underlying_conid = underlying_conid, "requesting option chain");
    let mut sub = client
        .option_chain(symbol, exchange_for_chain, sec_type, underlying_conid)
        .await
        .map_err(|e| format!("option_chain: {}", e))?;
    const CHAIN_TIMEOUT: Duration = Duration::from_secs(30);
    let chain: OptionChain = match tokio::time::timeout(CHAIN_TIMEOUT, sub.next()).await {
        Ok(Some(Ok(c))) => c,
        Ok(Some(Err(e))) => return Err(format!("option_chain: {}", e)),
        Ok(None) => return Err(
            "option_chain: no response (stream ended). Check TWS is running, API enabled, and symbol/exchange are valid (e.g. SPX = index, CBOE).".to_string()
        ),
        Err(_) => return Err(
            "option_chain: timeout waiting for TWS (30s). Check TWS is running and API enabled.".to_string()
        ),
    };
    let expirations: Vec<String> = chain.expirations.to_vec();
    let strikes: Vec<f64> = chain.strikes.clone();
    let trading_class = if chain.trading_class.is_empty() {
        symbol.to_uppercase()
    } else {
        chain.trading_class.clone()
    };
    if expirations.is_empty() || strikes.is_empty() {
        return Err("option_chain: empty expirations or strikes".to_string());
    }
    Ok((expirations, strikes, trading_class))
}

/// Resolve option contract to a conid via contract_details; use resolved contract for market data to avoid "Invalid contract id" (200).
async fn resolve_option_contract(client: &Client, contract: &Contract) -> Option<Contract> {
    match client.contract_details(contract).await {
        Ok(details) if !details.is_empty() => {
            let one = details.into_iter().next().unwrap();
            if one.contract.contract_id == 0 {
                debug!("contract_details returned contract_id 0, using original contract");
                Some(contract.clone())
            } else {
                debug!(
                    conid = one.contract.contract_id,
                    "resolved option via contract_details"
                );
                Some(one.contract)
            }
        }
        Ok(_) => {
            warn!("contract_details returned no results for option");
            None
        }
        Err(e) => {
            warn!(error = %e, "contract_details failed for option");
            None
        }
    }
}

/// Resolve one option leg to conId for BAG construction. Returns None if resolution fails.
async fn resolve_option_to_conid(
    client: &Client,
    symbol: &str,
    expiry: &str,
    strike: f64,
    is_call: bool,
    trading_class: &str,
) -> Option<i32> {
    let normalized = normalize_expiry(expiry)?;
    let (y, m, d) = parse_expiry_yyyy_mm_dd(&normalized).ok()?;
    let exchange = option_exchange(symbol);
    let is_index = underlying_security_type(symbol) == SecurityType::Index;
    let mut contract = if is_call {
        Contract::call(symbol)
            .strike(strike)
            .expires_on(y, m, d)
            .on_exchange(exchange)
            .trading_class(trading_class)
    } else {
        Contract::put(symbol)
            .strike(strike)
            .expires_on(y, m, d)
            .on_exchange(exchange)
            .trading_class(trading_class)
    };
    if is_index {
        contract = contract.primary(exchange);
    }
    let contract = contract.build();
    let contract = if is_index {
        resolve_option_contract(client, &contract).await?
    } else {
        contract
    };
    let cid = contract.contract_id;
    if cid != 0 {
        Some(cid)
    } else {
        debug!("resolved option has contract_id 0");
        None
    }
}

/// Collect bid/ask for one option contract with a timeout.
/// For index symbols we resolve the option via contract_details first and use the resolved contract (with conid) for market data to avoid TWS error 200.
async fn get_option_bid_ask(
    client: &Client,
    symbol: &str,
    expiry: &str,
    strike: f64,
    is_call: bool,
    trading_class: &str,
    timeout_ms: u64,
) -> Option<(f64, f64)> {
    let normalized = normalize_expiry(expiry)?;
    let (y, m, d) = parse_expiry_yyyy_mm_dd(&normalized).ok()?;
    let exchange = option_exchange(symbol);
    let is_index = underlying_security_type(symbol) == SecurityType::Index;
    let mut contract = if is_call {
        Contract::call(symbol)
            .strike(strike)
            .expires_on(y, m, d)
            .on_exchange(exchange)
            .trading_class(trading_class)
    } else {
        Contract::put(symbol)
            .strike(strike)
            .expires_on(y, m, d)
            .on_exchange(exchange)
            .trading_class(trading_class)
    };
    if is_index {
        contract = contract.primary(exchange);
    }
    let contract = contract.build();
    let contract = if is_index {
        match resolve_option_contract(client, &contract).await {
            Some(resolved) => resolved,
            None => {
                debug!(
                    "skipping option (resolution failed): {} {} {} {}",
                    symbol,
                    expiry,
                    strike,
                    if is_call { "C" } else { "P" }
                );
                return None;
            }
        }
    } else {
        contract
    };
    let mut sub = match client.market_data(&contract).subscribe().await {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, conid = contract.contract_id, "market_data subscribe failed");
            return None;
        }
    };
    let deadline = tokio::time::Instant::now() + Duration::from_millis(timeout_ms);
    let mut bid = 0.0_f64;
    let mut ask = 0.0_f64;
    while tokio::time::Instant::now() < deadline {
        match tokio::time::timeout(Duration::from_millis(500), sub.next()).await {
            Ok(Some(Ok(tick))) => {
                let (price, tick_type_opt): (f64, Option<&TickType>) = match &tick {
                    TickTypes::Price(p) => (p.price, Some(&p.tick_type)),
                    TickTypes::PriceSize(ps) => (ps.price, Some(&ps.price_tick_type)),
                    TickTypes::RequestParameters(params) if params.min_tick > 0.0 => {
                        // TWS sometimes sends tick 81 (DelayedAskOption) as RequestParameters with price in min_tick (ibapi decoding quirk). Use as single quote.
                        if bid <= 0.0 {
                            bid = params.min_tick;
                        }
                        if ask <= 0.0 {
                            ask = params.min_tick;
                        }
                        (0.0, None)
                    }
                    other => {
                        debug!(tick_variant = ?other, "TWS tick received (not Price/PriceSize); check if price is in another variant");
                        (0.0, None)
                    }
                };
                if let Some(tt) = tick_type_opt {
                    match tt {
                        TickType::Bid | TickType::DelayedBid | TickType::DelayedBidOption => {
                            bid = price;
                            if ask <= 0.0 {
                                ask = price;
                            }
                        }
                        TickType::Ask | TickType::DelayedAsk | TickType::DelayedAskOption => {
                            ask = price;
                            if bid <= 0.0 {
                                bid = price;
                            }
                        }
                        TickType::Last | TickType::DelayedLast | TickType::DelayedLastOption => {
                            if bid <= 0.0 {
                                bid = price;
                            }
                            if ask <= 0.0 {
                                ask = price;
                            }
                        }
                        TickType::Close | TickType::DelayedClose => {
                            if bid <= 0.0 {
                                bid = price;
                            }
                            if ask <= 0.0 {
                                ask = price;
                            }
                        }
                        other_tt => {
                            debug!(tick_type = ?other_tt, price, "TWS price tick not used for bid/ask (unhandled type)");
                        }
                    }
                }
                if bid > 0.0 && ask > 0.0 {
                    return Some((bid, ask));
                }
            }
            Ok(Some(Err(e))) => {
                debug!(error = %e, "TWS tick stream error");
            }
            _ => continue,
        }
    }
    if bid > 0.0 && ask > 0.0 {
        Some((bid, ask))
    } else {
        debug!(%symbol, %expiry, strike, call = is_call, "no bid/ask within {}ms (market may be closed or illiquid)", timeout_ms);
        None
    }
}

/// Placeholder rates when no bid/ask (e.g. out of hours). Exposed for tests.
const PLACEHOLDER_BUY_RATE: f64 = 0.04;
const PLACEHOLDER_SELL_RATE: f64 = 0.05;

/// True when env YIELD_CURVE_COMPARE_BAG_LEGS=1: run both BAG and 4-leg and include comparison in output.
fn yield_curve_compare_bag_legs() -> bool {
    std::env::var("YIELD_CURVE_COMPARE_BAG_LEGS")
        .map(|s| s == "1")
        .unwrap_or(false)
}

/// Get box spread net_debit, net_credit, buy_rate, sell_rate via four separate option subscriptions.
/// Returns None if any leg has no bid/ask.
async fn get_box_bid_ask_via_legs(
    client: &Client,
    symbol: &str,
    expiry: &str,
    strike_low: f64,
    strike_high: f64,
    trading_class: &str,
    timeout_ms: u64,
) -> Option<(f64, f64, f64, f64)> {
    let width = strike_high - strike_low;
    if width <= 0.0 {
        return None;
    }
    let dte = days_to_expiry(expiry)?;
    let t = (dte as f64) / 365.0;
    if t <= 0.0 {
        return None;
    }
    let (c_low, c_high, p_low, p_high) = tokio::join!(
        get_option_bid_ask(
            client,
            symbol,
            expiry,
            strike_low,
            true,
            trading_class,
            timeout_ms
        ),
        get_option_bid_ask(
            client,
            symbol,
            expiry,
            strike_high,
            true,
            trading_class,
            timeout_ms
        ),
        get_option_bid_ask(
            client,
            symbol,
            expiry,
            strike_low,
            false,
            trading_class,
            timeout_ms
        ),
        get_option_bid_ask(
            client,
            symbol,
            expiry,
            strike_high,
            false,
            trading_class,
            timeout_ms
        ),
    );
    let (c_low_bid, c_low_ask) = c_low?;
    let (c_high_bid, _) = c_high?;
    let (p_low_bid, _) = p_low?;
    let (_, p_high_ask) = p_high?;
    let net_debit = c_low_ask + p_high_ask - c_high_bid - p_low_bid;
    let net_credit = c_high_bid + p_low_bid - c_low_bid - p_high_ask;
    let buy_rate = if net_debit > 0.0 && net_debit < width {
        (1.0 - net_debit / width) / t
    } else {
        PLACEHOLDER_BUY_RATE
    };
    let sell_rate = if net_credit > 0.0 && net_credit < width {
        (1.0 - net_credit / width) / t
    } else {
        PLACEHOLDER_SELL_RATE
    };
    let buy_rate = buy_rate.clamp(0.001, 0.15);
    let sell_rate = sell_rate.clamp(0.001, 0.15);
    Some((net_debit, net_credit, buy_rate, sell_rate))
}

/// Try to get box spread bid/ask by subscribing to the whole box as a BAG (combo).
/// Returns Some((bid, ask)) where bid = net_credit (sell box), ask = net_debit (buy box).
/// Returns None if leg resolution fails, BAG build fails, or no quote within timeout.
async fn get_box_bid_ask_via_bag(
    client: &Client,
    symbol: &str,
    expiry: &str,
    strike_low: f64,
    strike_high: f64,
    trading_class: &str,
    timeout_ms: u64,
) -> Option<(f64, f64)> {
    let (c_low, c_high, p_low, p_high) = tokio::join!(
        resolve_option_to_conid(client, symbol, expiry, strike_low, true, trading_class),
        resolve_option_to_conid(client, symbol, expiry, strike_high, true, trading_class),
        resolve_option_to_conid(client, symbol, expiry, strike_low, false, trading_class),
        resolve_option_to_conid(client, symbol, expiry, strike_high, false, trading_class),
    );
    let conid_c_low = c_low?;
    let conid_c_high = c_high?;
    let conid_p_low = p_low?;
    let conid_p_high = p_high?;

    let mut builder = Contract::spread().in_currency("USD").on_exchange("SMART");
    builder = builder.add_leg(conid_c_low, LegAction::Buy).ratio(1).done();
    builder = builder
        .add_leg(conid_c_high, LegAction::Sell)
        .ratio(1)
        .done();
    builder = builder.add_leg(conid_p_low, LegAction::Buy).ratio(1).done();
    builder = builder
        .add_leg(conid_p_high, LegAction::Sell)
        .ratio(1)
        .done();
    let mut bag_contract = match builder.build() {
        Ok(c) => c,
        Err(e) => {
            warn!(error = %e, "BAG contract build failed");
            return None;
        }
    };
    bag_contract.symbol = Symbol::from(symbol);

    let mut sub = match client.market_data(&bag_contract).subscribe().await {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "BAG market_data subscribe failed; falling back to leg quotes");
            return None;
        }
    };

    let deadline = tokio::time::Instant::now() + Duration::from_millis(timeout_ms);
    let mut bid = 0.0_f64;
    let mut ask = 0.0_f64;
    while tokio::time::Instant::now() < deadline {
        match tokio::time::timeout(Duration::from_millis(500), sub.next()).await {
            Ok(Some(Ok(tick))) => {
                let (price, tick_type_opt): (f64, Option<&TickType>) = match &tick {
                    TickTypes::Price(p) => (p.price, Some(&p.tick_type)),
                    TickTypes::PriceSize(ps) => (ps.price, Some(&ps.price_tick_type)),
                    TickTypes::RequestParameters(params) if params.min_tick > 0.0 => {
                        if bid <= 0.0 {
                            bid = params.min_tick;
                        }
                        if ask <= 0.0 {
                            ask = params.min_tick;
                        }
                        (0.0, None)
                    }
                    other => {
                        debug!(tick_variant = ?other, "BAG tick (not Price/PriceSize)");
                        (0.0, None)
                    }
                };
                if let Some(tt) = tick_type_opt {
                    match tt {
                        TickType::Bid | TickType::DelayedBid | TickType::DelayedBidOption => {
                            bid = price;
                            if ask <= 0.0 {
                                ask = price;
                            }
                        }
                        TickType::Ask | TickType::DelayedAsk | TickType::DelayedAskOption => {
                            ask = price;
                            if bid <= 0.0 {
                                bid = price;
                            }
                        }
                        TickType::Last
                        | TickType::DelayedLast
                        | TickType::DelayedLastOption
                        | TickType::Close
                        | TickType::DelayedClose => {
                            if bid <= 0.0 {
                                bid = price;
                            }
                            if ask <= 0.0 {
                                ask = price;
                            }
                        }
                        other_tt => {
                            debug!(tick_type = ?other_tt, price, "BAG price tick not used");
                        }
                    }
                }
                if bid > 0.0 && ask > 0.0 {
                    info!(%symbol, %expiry, strike_low, strike_high, bid, ask, "BAG quote received");
                    return Some((bid, ask));
                }
            }
            Ok(Some(Err(e))) => {
                debug!(error = %e, "BAG tick stream error");
            }
            _ => continue,
        }
    }
    if bid > 0.0 && ask > 0.0 {
        Some((bid, ask))
    } else {
        debug!(%symbol, %expiry, "no BAG bid/ask within {}ms", timeout_ms);
        None
    }
}

/// Build one box spread point from TWS quotes for one expiry and two strikes.
/// If any leg has no bid/ask (e.g. market closed), uses placeholder rates and sets "delayed": true.
async fn box_spread_point(
    client: &Client,
    symbol: &str,
    expiry: &str,
    strike_low: f64,
    strike_high: f64,
    trading_class: &str,
    quote_timeout_ms: u64,
) -> Option<Value> {
    let width = strike_high - strike_low;
    if width <= 0.0 {
        return None;
    }
    let dte = days_to_expiry(expiry).unwrap_or(30);
    let t = (dte as f64) / 365.0;
    if t <= 0.0 {
        return None;
    }
    let expiry_ymd = {
        let norm = normalize_expiry(expiry)?;
        let (y, m, d) = parse_expiry_yyyy_mm_dd(&norm).ok()?;
        format!("{:04}-{:02}-{:02}", y, m, d)
    };

    let compare = yield_curve_compare_bag_legs();
    let (buy_rate, sell_rate, net_debit, net_credit, delayed, comparison) = if compare {
        let t0 = Instant::now();
        let bag = get_box_bid_ask_via_bag(
            client,
            symbol,
            expiry,
            strike_low,
            strike_high,
            trading_class,
            quote_timeout_ms,
        )
        .await;
        let bag_ms = t0.elapsed().as_millis() as u64;
        let t1 = Instant::now();
        let legs = get_box_bid_ask_via_legs(
            client,
            symbol,
            expiry,
            strike_low,
            strike_high,
            trading_class,
            quote_timeout_ms,
        )
        .await;
        let legs_ms = t1.elapsed().as_millis() as u64;

        let (bag_net_debit, bag_net_credit, bag_buy, bag_sell) = match bag {
            Some((bid, ask)) => {
                let nd = ask;
                let nc = bid;
                let br = ((1.0 - nd / width) / t).clamp(0.001, 0.15);
                let sr = ((1.0 - nc / width) / t).clamp(0.001, 0.15);
                (nd, nc, br, sr)
            }
            None => (0.0, 0.0, PLACEHOLDER_BUY_RATE, PLACEHOLDER_SELL_RATE),
        };
        let (legs_net_debit, legs_net_credit, legs_buy, legs_sell) = legs.unwrap_or((
            width * (1.0 - PLACEHOLDER_BUY_RATE * t),
            width * (1.0 - PLACEHOLDER_SELL_RATE * t),
            PLACEHOLDER_BUY_RATE,
            PLACEHOLDER_SELL_RATE,
        ));

        let (buy_rate, sell_rate, net_debit, net_credit, delayed) = match (bag, legs) {
            (Some((bag_bid, bag_ask)), _) => {
                let net_credit = bag_bid;
                let net_debit = bag_ask;
                let buy_rate = if net_debit > 0.0 && net_debit < width {
                    (1.0 - net_debit / width) / t
                } else {
                    PLACEHOLDER_BUY_RATE
                };
                let sell_rate = if net_credit > 0.0 && net_credit < width {
                    (1.0 - net_credit / width) / t
                } else {
                    PLACEHOLDER_SELL_RATE
                };
                (
                    buy_rate.clamp(0.001, 0.15),
                    sell_rate.clamp(0.001, 0.15),
                    net_debit,
                    net_credit,
                    false,
                )
            }
            (_, Some(_)) => (legs_buy, legs_sell, legs_net_debit, legs_net_credit, false),
            _ => {
                let buy_rate = PLACEHOLDER_BUY_RATE;
                let sell_rate = PLACEHOLDER_SELL_RATE;
                (
                    buy_rate,
                    sell_rate,
                    width * (1.0 - buy_rate * t),
                    width * (1.0 - sell_rate * t),
                    true,
                )
            }
        };
        let faster = if bag_ms <= legs_ms { "BAG" } else { "Legs" };
        let bag_bid_v = bag.map(|(b, _)| (b * 100.0).round() / 100.0);
        let bag_ask_v = bag.map(|(_, a)| (a * 100.0).round() / 100.0);
        let comparison = json!({
            "bag_ms": bag_ms,
            "legs_ms": legs_ms,
            "faster": faster,
            "bag_bid": bag_bid_v,
            "bag_ask": bag_ask_v,
            "bag_net_debit": (bag_net_debit * 100.0).round() / 100.0,
            "bag_net_credit": (bag_net_credit * 100.0).round() / 100.0,
            "bag_buy_pct": (bag_buy * 100.0).round() / 100.0,
            "bag_sell_pct": (bag_sell * 100.0).round() / 100.0,
            "legs_net_debit": (legs_net_debit * 100.0).round() / 100.0,
            "legs_net_credit": (legs_net_credit * 100.0).round() / 100.0,
            "legs_buy_pct": (legs_buy * 100.0).round() / 100.0,
            "legs_sell_pct": (legs_sell * 100.0).round() / 100.0,
        });
        (
            buy_rate,
            sell_rate,
            net_debit,
            net_credit,
            delayed,
            Some(comparison),
        )
    } else {
        let (buy_rate, sell_rate, net_debit, net_credit, delayed) = match get_box_bid_ask_via_bag(
            client,
            symbol,
            expiry,
            strike_low,
            strike_high,
            trading_class,
            quote_timeout_ms,
        )
        .await
        {
            Some((bag_bid, bag_ask)) => {
                let net_credit = bag_bid;
                let net_debit = bag_ask;
                let buy_rate = if net_debit > 0.0 && net_debit < width {
                    (1.0 - net_debit / width) / t
                } else {
                    PLACEHOLDER_BUY_RATE
                };
                let sell_rate = if net_credit > 0.0 && net_credit < width {
                    (1.0 - net_credit / width) / t
                } else {
                    PLACEHOLDER_SELL_RATE
                };
                (
                    buy_rate.clamp(0.001, 0.15),
                    sell_rate.clamp(0.001, 0.15),
                    net_debit,
                    net_credit,
                    false,
                )
            }
            None => {
                let legs = get_box_bid_ask_via_legs(
                    client,
                    symbol,
                    expiry,
                    strike_low,
                    strike_high,
                    trading_class,
                    quote_timeout_ms,
                )
                .await;
                match legs {
                    Some((legs_net_debit, legs_net_credit, legs_buy, legs_sell)) => {
                        (legs_buy, legs_sell, legs_net_debit, legs_net_credit, false)
                    }
                    None => {
                        let buy_rate = PLACEHOLDER_BUY_RATE;
                        let sell_rate = PLACEHOLDER_SELL_RATE;
                        (
                            buy_rate,
                            sell_rate,
                            width * (1.0 - buy_rate * t),
                            width * (1.0 - sell_rate * t),
                            true,
                        )
                    }
                }
            }
        };
        (buy_rate, sell_rate, net_debit, net_credit, delayed, None)
    };

    let mut out = json!({
        "spread": {
            "symbol": symbol,
            "expiry": expiry_ymd,
            "days_to_expiry": dte,
            "strike_width": width,
            "strike_low": strike_low,
            "strike_high": strike_high,
            "buy_implied_rate": buy_rate,
            "sell_implied_rate": sell_rate,
            "net_debit": (net_debit * 100.0).round() / 100.0,
            "net_credit": (net_credit * 100.0).round() / 100.0,
            "liquidity_score": MIN_LIQUIDITY_SCORE,
            "spread_id": null,
            "convenience_yield": null,
            "delayed": delayed
        }
    });
    if let Some(ref comp) = comparison {
        out["comparison"] = comp.clone();
    }
    Some(out)
}

/// Market data type when we want closing data. Delayed (3) is free and often works when "part of requested market data is not available";
/// DelayedFrozen (4) also needs no subscription; Frozen (2) requires same subscription as live.
fn closing_market_data_type() -> MarketDataType {
    match std::env::var("YIELD_CURVE_MARKET_DATA_TYPE")
        .ok()
        .as_deref()
    {
        Some("frozen") => MarketDataType::Frozen,
        Some("delayed_frozen") => MarketDataType::DelayedFrozen,
        _ => MarketDataType::Delayed,
    }
}

/// True when we should request closing data: env YIELD_CURVE_USE_CLOSING=1 or outside US regular session (Mon–Fri 9:30–16:00 ET).
fn use_closing_data() -> bool {
    if std::env::var("YIELD_CURVE_USE_CLOSING")
        .map(|s| s == "1")
        .unwrap_or(false)
    {
        return true;
    }
    let now = Utc::now();
    let weekday = now.weekday().number_from_monday();
    let hour = now.hour();
    let min = now.minute();
    let minutes_since_midnight = hour * 60 + min;
    let is_weekday = (1..=5).contains(&weekday);
    let utc_session_start = 13 * 60 + 30;
    let utc_session_end = 22 * 60;
    let in_session = is_weekday
        && minutes_since_midnight >= utc_session_start
        && minutes_since_midnight < utc_session_end;
    !in_session
}

/// Ensure a tracing subscriber is active so RUST_LOG takes effect when running from CLI.
/// If YIELD_CURVE_LOG_FILE is set, the full session is also written to that path (stderr unchanged).
fn ensure_tracing() {
    let filter = EnvFilter::from_default_env();
    match std::env::var("YIELD_CURVE_LOG_FILE") {
        Ok(path) => {
            if let Ok(file) = std::fs::File::create(&path) {
                let _ = tracing_subscriber::registry()
                    .with(filter)
                    .with(fmt::layer().with_writer(std::io::stderr))
                    .with(fmt::layer().with_writer(file).with_ansi(false))
                    .try_init();
                info!(path = %path, "TWS yield curve: logging full session to file");
            } else {
                let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
            }
        }
        Err(_) => {
            let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
        }
    }
}

/// Fetch yield curve opportunities from TWS for the given symbol.
/// Returns `Ok(vec)` on success, `Err(msg)` on TWS/chain failure. No NATS.
pub async fn fetch_yield_curve_from_tws(symbol: &str) -> Result<Vec<Value>, String> {
    ensure_tracing();
    let (client, address) = connect_tws().await.map_err(|e| {
        warn!(error = %e, "TWS yield curve: connect failed");
        e
    })?;
    debug!(%address, "TWS yield curve: connected");

    if use_closing_data() {
        let data_type = closing_market_data_type();
        if let Err(e) = client.switch_market_data_type(data_type).await {
            warn!(error = %e, "switch to {:?} (closing) market data failed; will try live then placeholders", data_type);
        } else {
            info!(
                "TWS yield curve: using {:?} (closing) data (out of hours or YIELD_CURVE_USE_CLOSING=1); delayed/delayed_frozen need no subscription",
                data_type
            );
        }
    }

    let quote_timeout_ms = if use_closing_data() {
        QUOTE_TIMEOUT_CLOSING_MS
    } else {
        QUOTE_TIMEOUT_MS
    };

    let (expirations, strikes, trading_class) =
        request_option_chain(&client, symbol).await.map_err(|e| {
            warn!(error = %e, "TWS yield curve: option chain failed");
            e
        })?;
    debug!(%trading_class, "TWS yield curve: using trading_class from option chain");

    let spot = reference_spot(symbol);
    let strike_low = (spot - STRIKE_WIDTH / 2.0).round();
    let strike_high = (spot + STRIKE_WIDTH / 2.0).round();
    let (strike_low, strike_high) =
        if strikes.contains(&strike_low) && strikes.contains(&strike_high) {
            (strike_low, strike_high)
        } else {
            let closest = strikes
                .iter()
                .min_by(|a, b| {
                    let da = (spot - **a).abs();
                    let db = (spot - **b).abs();
                    da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                })
                .ok_or("option_chain: no strikes")?;
            let low = (closest - STRIKE_WIDTH / 2.0).round();
            let high = (closest + STRIKE_WIDTH / 2.0).round();
            if strikes.contains(&low) && strikes.contains(&high) {
                (low, high)
            } else {
                let mut sorted: Vec<f64> = strikes.iter().filter(|s| **s > 0.0).copied().collect();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let idx = sorted
                    .iter()
                    .position(|s| *s >= spot)
                    .unwrap_or(sorted.len().saturating_sub(1));
                let high = sorted.get(idx).copied().unwrap_or(spot + 2.0);
                let low = sorted
                    .iter()
                    .find(|s| **s < high)
                    .copied()
                    .unwrap_or(high - STRIKE_WIDTH);
                if (high - low).abs() < 0.01 {
                    return Err("no valid strike pair".to_string());
                }
                (low, high)
            }
        };

    let expiry_futures: Vec<_> = expirations
        .iter()
        .take(3)
        .map(|expiry| {
            box_spread_point(
                &client,
                symbol,
                expiry,
                strike_low,
                strike_high,
                &trading_class,
                quote_timeout_ms,
            )
        })
        .collect();
    let results = futures::future::join_all(expiry_futures).await;
    let opportunities: Vec<Value> = results.into_iter().flatten().collect();
    if opportunities.is_empty() {
        return Err(
            "no box spread points from TWS (option chain and resolution succeeded; no bid/ask quotes in time—check market hours and liquidity)".to_string()
        );
    }
    Ok(opportunities)
}
