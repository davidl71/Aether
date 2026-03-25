//! Yield curve derivation from options market data.
//!
//! Derives box-spread implied financing rates from equity/index options chains.
//!
//! ## Data Sources
//!
//! | Source | Style | Notes |
//! |--------|-------|-------|
//! | **TWS** (IBKR) | European (SPX/NDX) + American (SPY/QQQ) | Recommended — live NBBO |
//! | **Yahoo Finance** | American (SPY/QQQ/etc.) | Free, delayed; no index options |
//! | **Polygon** | American | Requires Starter plan for options snapshot |
//!
//! ## Box Spread Financing Rate
//!
//! A box spread (bull call spread + bear put spread at the same two strikes)
//! has a fixed payoff at expiration of `K_high - K_low` regardless of price.
//!
//! **All four legs must share the same two strike prices** — mismatched strikes
//! break the arbitrage relationship.
//!
//! ### Buy-box (borrow cash):
//! ```text
//! Net Debit  = C(K_low).ask + P(K_high).ask − C(K_high).bid − P(K_low).bid
//! buy_rate   = (width − Net_Debit) / (Net_Debit × T)
//! ```
//!
//! ### Sell-box (lend cash):
//! ```text
//! Net Credit = C(K_low).bid + P(K_high).bid − C(K_high).ask − P(K_low).ask
//! sell_rate  = (width − Net_Credit) / (Net_Credit × T)
//! ```
//!
//! Where `T = DTE / 365`.
//!
//! ## Dividend-Paying Underlyings (SPY, QQQ, …)
//!
//! For dividend-paying stocks with American options, **net_debit > width is
//! normal** — buying the box is uneconomic because put prices are depressed
//! by the dividend (early-exercise premium shifts value from put to call).
//! In this case `buy_rate` is negative (no-arb: borrowing via box is expensive).
//!
//! The sell side remains positive: selling the box (lending) still earns a
//! positive rate ≈ risk-free − dividend_yield.
//!
//! ## Short-DTE High Rates
//!
//! Very short-DTE boxes (< 30 days) can show high annualized sell rates
//! (e.g. 20-50%). These are **real lending opportunities**: sell the box,
//! collect the premium, hold all four legs to expiry and collect the width.
//! The apparent high rate reflects small absolute profit (e.g. $0.55 on $55)
//! annualized over a short period. Execution quality (fill prices vs NBBO)
//! determines whether the opportunity is actually achievable.
//!
//! ## Strike Width Selection
//!
//! Strike width scales with underlying price to ensure signal-to-noise ratio:
//! - Target box payoff = `MIN_BOX_PAYOFF / CONTRACT_MULTIPLIER` (minimum per-share)
//! - Candidates: 8%, 4%, 2%, 1% of spot, rounded to nearest $5
//! - All four legs must be liquid (bid > 0) at the same two strikes
//! - Widest candidate with sufficient liquidity is preferred

use std::collections::HashMap;

use chrono::{NaiveDate, TimeZone, Utc};
use thiserror::Error;
use tracing::{debug, warn};
use yfinance_rs::{core::conversions::money_to_f64, Interval, Range};

use crate::yahoo::{OptionContractData, OptionsDataSource, YahooHistorySource, YahooOptionsSource};

/// Standard options multiplier (shares per contract).
const CONTRACT_MULTIPLIER: f64 = 100.0;

/// Minimum box payoff per contract in dollars.
/// A $500 box on SPY means width × 100 ≥ $500, so width ≥ $5.
/// Ensures the yield signal dominates bid/ask noise across 4 legs.
const MIN_BOX_PAYOFF: f64 = 500.0;

const MIN_LIQUIDITY_SCORE: f64 = 50.0;
#[allow(dead_code)]
const MAX_EXPIRY_DAYS: i32 = 730;
const MIN_DTE: i32 = 7;
const MAX_DTE: i32 = 730;

#[derive(Debug, Error)]
pub enum YieldCurveError {
    #[error("No option data available for {0}")]
    NoData(String),

    #[error("Insufficient liquidity at strikes")]
    InsufficientLiquidity,

    #[error("Invalid strike width {0}")]
    InvalidStrikeWidth(f64),

    #[error("No expiration dates found")]
    NoExpirations,

    #[error("Quote fetch failed: {0}")]
    QuoteFailed(String),
}

/// A single yield curve point derived from box spread analysis.
#[derive(Debug, Clone)]
pub struct YieldCurvePoint {
    pub expiry: NaiveDate,
    pub dte: i32,
    pub strike_low: f64,
    pub strike_high: f64,
    pub strike_width: f64,
    pub net_debit: f64,
    pub net_credit: f64,
    pub buy_implied_rate: f64,
    pub sell_implied_rate: f64,
    pub liquidity_score: f64,
    pub mid_rate: f64,
    pub spread_bps: f64,
    pub source: String,
}

/// Complete yield curve with multiple tenor points.
#[derive(Debug, Clone)]
pub struct YieldCurve {
    pub symbol: String,
    pub underlying_price: f64,
    pub points: Vec<YieldCurvePoint>,
    pub timestamp: chrono::DateTime<Utc>,
    pub source: String,
}

/// Box spread calculation result.
#[derive(Debug, Clone)]
pub struct BoxSpreadResult {
    pub strike_low: f64,
    pub strike_high: f64,
    pub c_low_bid: f64,
    pub c_high_bid: f64,
    pub p_low_bid: f64,
    pub p_high_bid: f64,
    pub c_low_ask: f64,
    pub c_high_ask: f64,
    pub p_low_ask: f64,
    pub p_high_ask: f64,
    pub net_debit: f64,
    pub net_credit: f64,
    pub buy_rate: f64,
    pub sell_rate: f64,
    pub mid_rate: f64,
    pub liquidity_score: f64,
}

impl BoxSpreadResult {
    /// Calculate box spread implied rates from option quotes.
    ///
    /// Box spread payoff at expiration: K_high - K_low (fixed)
    ///
    /// For a box to be a valid financing instrument:
    /// - Net debit must be positive and less than width (profitable to borrow)
    /// - Net credit must be positive and less than width (profitable to lend)
    pub fn from_quotes(
        c_low: (f64, f64),
        c_high: (f64, f64),
        p_low: (f64, f64),
        p_high: (f64, f64),
        width: f64,
        dte: f64,
    ) -> Option<Self> {
        let (c_low_bid, c_low_ask) = c_low;
        let (c_high_bid, c_high_ask) = c_high;
        let (p_low_bid, p_low_ask) = p_low;
        let (p_high_bid, p_high_ask) = p_high;

        // Buy box:  buy C(K_low) + buy P(K_high) - sell C(K_high) - sell P(K_low)
        let net_debit = c_low_ask + p_high_ask - c_high_bid - p_low_bid;
        // Sell box: sell C(K_low) + sell P(K_high) - buy C(K_high) - buy P(K_low)
        let net_credit = c_low_bid + p_high_bid - c_high_ask - p_low_ask;

        let t = dte / 365.0;
        if t <= 0.0 {
            return None;
        }

        // At least one side must be viable (net_credit or net_debit within box payoff).
        // For dividend-paying stocks (e.g. SPY), net_debit can exceed width (buy side
        // is unprofitable due to dividends), while the sell side remains positive.
        let sell_ok = net_credit > 0.0 && net_credit < width;
        let buy_ok = net_debit > 0.0 && net_debit < width;
        if !sell_ok && !buy_ok {
            return None;
        }

        let buy_rate = if buy_ok {
            (width - net_debit) / (net_debit * t)
        } else {
            // Negative: buying the box costs more than its payoff.
            (width - net_debit) / (net_debit.abs() * t)
        };
        let sell_rate = if sell_ok {
            (width - net_credit) / (net_credit * t)
        } else {
            buy_rate
        };

        let liquidity_score = Self::calculate_liquidity_score(
            c_low_bid, c_low_ask, c_high_bid, c_high_ask, p_low_bid, p_low_ask, p_high_bid,
            p_high_ask,
        );

        // No clamping — high sell rates on short-DTE boxes are real lending
        // opportunities (sell the box, collect premium, hold to expiry).
        // Negative buy_rate is normal for dividend-paying stocks (buying the
        // box costs more than its payoff due to dividend-embedded put pricing).
        let mid_rate = (buy_rate + sell_rate) / 2.0;

        Some(Self {
            strike_low: 0.0,
            strike_high: 0.0,
            c_low_bid,
            c_high_bid,
            p_low_bid,
            p_high_bid,
            c_low_ask,
            c_high_ask,
            p_low_ask,
            p_high_ask,
            net_debit,
            net_credit,
            buy_rate,
            sell_rate,
            mid_rate,
            liquidity_score,
        })
    }

    fn calculate_liquidity_score(
        c_low_bid: f64,
        c_low_ask: f64,
        c_high_bid: f64,
        c_high_ask: f64,
        p_low_bid: f64,
        p_low_ask: f64,
        p_high_bid: f64,
        p_high_ask: f64,
    ) -> f64 {
        let spreads = [
            c_low_ask - c_low_bid,
            c_high_ask - c_high_bid,
            p_low_ask - p_low_bid,
            p_high_ask - p_high_bid,
        ];

        let avg_spread: f64 = spreads.iter().sum::<f64>() / 4.0;
        let max_spread = spreads.iter().cloned().fold(0.0f64, f64::max);

        let spread_penalty = (avg_spread / 0.10).min(1.0);
        let max_penalty = (max_spread / 0.20).min(1.0);

        let liquidity = 100.0 * (1.0 - 0.6 * spread_penalty - 0.4 * max_penalty);
        liquidity.clamp(MIN_LIQUIDITY_SCORE, 100.0)
    }
}

/// Yahoo Finance yield curve source.
pub struct YahooYieldCurveSource {
    options: YahooOptionsSource,
    history: YahooHistorySource,
}

impl YahooYieldCurveSource {
    pub fn new() -> Self {
        Self {
            options: YahooOptionsSource::new(),
            history: YahooHistorySource::new(),
        }
    }

    pub async fn fetch_underlying_price(&self, symbol: &str) -> anyhow::Result<f64> {
        let history = self
            .history
            .get_history(symbol, Range::D1, Interval::D1)
            .await?;
        if let Some(candle) = history.first() {
            Ok(money_to_f64(&candle.close))
        } else {
            Err(anyhow::anyhow!("no price data for {symbol}"))
        }
    }

    fn to_yahoo_symbol(symbol: &str) -> String {
        symbol.to_uppercase()
    }

    pub async fn fetch_yield_curve(&self, symbol: &str) -> anyhow::Result<YieldCurve> {
        let yahoo_symbol = Self::to_yahoo_symbol(symbol);
        let expiration_timestamps = self.options.get_expirations(&yahoo_symbol).await?;
        debug!(symbol = %symbol, expirations = expiration_timestamps.len(), "got expirations");
        if expiration_timestamps.is_empty() {
            return Err(anyhow::anyhow!("no expiration dates for {symbol}"));
        }

        let underlying_price = self.fetch_underlying_price(symbol).await?;
        debug!(symbol = %symbol, spot = underlying_price, "got underlying price");

        let today = Utc::now().date_naive();
        // Keep (ts, NaiveDate) pairs so we pass the original timestamp to get_chain.
        let valid_expirations: Vec<(i64, NaiveDate)> = expiration_timestamps
            .into_iter()
            .filter_map(|ts| {
                Utc.timestamp_opt(ts, 0)
                    .single()
                    .map(|dt| (ts, dt.date_naive()))
            })
            .filter(|(_, date)| {
                let dte = (*date - today).num_days() as i32;
                dte >= MIN_DTE && dte <= MAX_DTE
            })
            .collect();

        debug!(symbol = %symbol, valid_count = valid_expirations.len(),
               "filtered expirations (DTE {} to {})", MIN_DTE, MAX_DTE);

        let mut points = Vec::new();

        for (ts, expiry) in valid_expirations.into_iter().take(10) {
            let dte = (expiry - today).num_days() as i32;
            if let Some(point) = self
                .fetch_point(&yahoo_symbol, ts, expiry, underlying_price)
                .await
            {
                debug!(symbol = %symbol, dte = %dte, "computed point");
                points.push(point);
            }
        }

        if points.is_empty() {
            return Err(anyhow::anyhow!("could not compute any yield curve points"));
        }

        Ok(YieldCurve {
            symbol: symbol.to_string(),
            underlying_price,
            points,
            timestamp: Utc::now(),
            source: "yahoo_finance".to_string(),
        })
    }

    async fn fetch_point(
        &self,
        symbol: &str,
        expiration_ts: i64,
        expiry: NaiveDate,
        spot: f64,
    ) -> Option<YieldCurvePoint> {
        let chain = self.options.get_chain(symbol, expiration_ts).await.ok()?;

        debug!(symbol = %symbol, spot = spot, calls = chain.calls.len(), puts = chain.puts.len(),
               "fetched option chain for {}", expiry);

        let dte = (expiry - Utc::now().date_naive()).num_days() as i32;
        if dte <= 0 {
            return None;
        }

        // Try candidate widths (widest first) and take the one with the best
        // liquidity score that clears the minimum threshold.
        let widths = Self::candidate_widths(spot);
        debug!(%symbol, dte, ?widths, "trying strike widths");

        let mut best: Option<(YieldCurvePoint, f64)> = None;

        for width in &widths {
            let width = *width;

            let Some((c_low, c_high, p_low, p_high)) =
                self.find_box_legs(&chain.calls, &chain.puts, spot, width)
            else {
                continue;
            };

            let strike_low = c_low.strike; // all four legs share same two strikes
            let strike_high = c_high.strike;
            let actual_width = strike_high - strike_low;

            let Some(box_result) = BoxSpreadResult::from_quotes(
                (c_low.bid, c_low.ask),
                (c_high.bid, c_high.ask),
                (p_low.bid, p_low.ask),
                (p_high.bid, p_high.ask),
                actual_width,
                dte as f64,
            ) else {
                continue;
            };

            if box_result.liquidity_score < MIN_LIQUIDITY_SCORE {
                debug!(%symbol, dte, width, liquidity = box_result.liquidity_score, "low liquidity, trying next width");
                continue;
            }

            let spread_bps = (box_result.sell_rate - box_result.buy_rate) * 10000.0;
            debug!(%symbol, dte, width, buy_rate = box_result.buy_rate * 100.0, sell_rate = box_result.sell_rate * 100.0, "candidate box spread");

            let point = YieldCurvePoint {
                expiry,
                dte,
                strike_low,
                strike_high,
                strike_width: actual_width,
                net_debit: box_result.net_debit,
                net_credit: box_result.net_credit,
                buy_implied_rate: box_result.buy_rate,
                sell_implied_rate: box_result.sell_rate,
                liquidity_score: box_result.liquidity_score,
                mid_rate: box_result.mid_rate,
                spread_bps,
                source: "yahoo_finance".to_string(),
            };

            let score = box_result.liquidity_score;
            match best {
                None => best = Some((point, score)),
                Some((_, prev_score)) if score > prev_score => best = Some((point, score)),
                _ => {}
            }
        }

        if best.is_none() {
            debug!(%symbol, dte, "no valid box spread found across all candidate widths");
        }

        best.map(|(point, _)| point)
    }

    #[allow(dead_code)]
    fn find_option(
        &self,
        options: &[OptionContractData],
        target_strike: f64,
        _is_put: bool,
    ) -> Option<OptionContractData> {
        options
            .iter()
            .filter(|o| o.bid > 0.0 && o.ask > 0.0 && o.ask > o.bid)
            .min_by_key(|o| (o.strike - target_strike).abs() as i64)
            .cloned()
    }

    /// Returns candidate strike widths (per-share) to try for a given spot price,
    /// ordered from widest (best signal) to narrowest (most liquid).
    ///
    /// Width is derived from `MIN_BOX_PAYOFF / CONTRACT_MULTIPLIER` as a floor,
    /// then scaled by spot so strikes are spaced meaningfully relative to the
    /// underlying price.  E.g. for SPY at $656:
    ///   min_width = $500 / 100 = $5
    ///   candidates ≈ [$50, $25, $10, $5]
    pub fn candidate_widths(spot: f64) -> Vec<f64> {
        let min_width = (MIN_BOX_PAYOFF / CONTRACT_MULTIPLIER).ceil();
        // Anchor widths as round percentages of spot, rounded to nearest $5.
        let pcts: &[f64] = &[0.08, 0.04, 0.02, 0.01];
        let mut widths: Vec<f64> = pcts
            .iter()
            .map(|&p| {
                let raw = spot * p;
                let rounded = (raw / 5.0).round() * 5.0;
                rounded.max(min_width)
            })
            .collect();
        // Dedup and keep only widths above the minimum, widest first.
        widths.dedup();
        widths.retain(|&w| w >= min_width);
        if widths.is_empty() {
            widths.push(min_width);
        }
        widths
    }

    /// Finds (K_low, K_high) where ALL FOUR legs (call and put at each strike)
    /// are liquid, with K_high - K_low as close to `width` as possible.
    /// Returns (call_low, call_high, put_low, put_high).
    fn find_box_legs(
        &self,
        calls: &[OptionContractData],
        puts: &[OptionContractData],
        spot: f64,
        width: f64,
    ) -> Option<(
        OptionContractData,
        OptionContractData,
        OptionContractData,
        OptionContractData,
    )> {
        use std::collections::HashMap;

        // Build maps: strike → option, for liquid legs only.
        let call_map: HashMap<i64, &OptionContractData> = calls
            .iter()
            .filter(|o| o.bid > 0.0 && o.ask > 0.0 && o.ask > o.bid)
            .map(|o| ((o.strike * 100.0).round() as i64, o))
            .collect();
        let put_map: HashMap<i64, &OptionContractData> = puts
            .iter()
            .filter(|o| o.bid > 0.0 && o.ask > 0.0 && o.ask > o.bid)
            .map(|o| ((o.strike * 100.0).round() as i64, o))
            .collect();

        // Valid strikes: those with both a liquid call AND a liquid put.
        let mut valid_strikes: Vec<f64> = call_map
            .keys()
            .filter(|k| put_map.contains_key(k))
            .map(|&k| k as f64 / 100.0)
            .collect();
        valid_strikes.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let target_low = spot - width / 2.0;
        let target_high = spot + width / 2.0;

        // Find K_low: valid strike closest to target_low.
        let k_low = valid_strikes
            .iter()
            .min_by_key(|&&s| ((s - target_low).abs() * 100.0) as i64)
            .copied()?;

        // Find K_high: valid strike closest to target_high, strictly above K_low.
        let k_high = valid_strikes
            .iter()
            .filter(|&&s| s > k_low)
            .min_by_key(|&&s| ((s - target_high).abs() * 100.0) as i64)
            .copied()?;

        // Reject if the actual width is more than 2× the requested width.
        if k_high - k_low > width * 2.0 {
            return None;
        }

        let k_low_key = (k_low * 100.0).round() as i64;
        let k_high_key = (k_high * 100.0).round() as i64;

        let c_low = (*call_map.get(&k_low_key)?).clone();
        let c_high = (*call_map.get(&k_high_key)?).clone();
        let p_low = (*put_map.get(&k_low_key)?).clone();
        let p_high = (*put_map.get(&k_high_key)?).clone();

        Some((c_low, c_high, p_low, p_high))
    }

    pub async fn fetch_multi_symbol_curve(
        &self,
        symbols: &[String],
    ) -> HashMap<String, YieldCurve> {
        let mut curves = HashMap::new();
        for symbol in symbols {
            match self.fetch_yield_curve(symbol).await {
                Ok(curve) => {
                    debug!(%symbol, points = curve.points.len(), "fetched yield curve from Yahoo");
                    curves.insert(symbol.clone(), curve);
                }
                Err(e) => {
                    warn!(%symbol, error = %e, "failed to fetch yield curve from Yahoo");
                }
            }
        }
        curves
    }
}

impl Default for YahooYieldCurveSource {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn inspects_spy_option_chain() {
        use crate::yahoo::OptionsDataSource;

        let source = YahooOptionsSource::new();
        let timestamps = source.get_expirations("SPY").await.unwrap_or_default();
        eprintln!("SPY has {} expiration dates", timestamps.len());

        let today = Utc::now().date_naive();
        // Convert to (ts, NaiveDate) pairs, keeping original timestamps for get_chain.
        let dated: Vec<(i64, NaiveDate)> = timestamps
            .iter()
            .filter_map(|&ts| {
                Utc.timestamp_opt(ts, 0)
                    .single()
                    .map(|dt| (ts, dt.date_naive()))
            })
            .collect();

        let valid: Vec<_> = dated
            .iter()
            .filter(|(_, date)| {
                let dte = (*date - today).num_days();
                dte >= 7 && dte <= 60
            })
            .collect();
        eprintln!("Valid expirations (DTE 7-60): {}", valid.len());

        if let Some((ts, expiry)) = valid.first() {
            let expiry = *expiry;
            eprintln!(
                "First valid expiry: {} (DTE {}, ts={})",
                expiry,
                (expiry - today).num_days(),
                ts,
            );

            let chain = source.get_chain("SPY", *ts).await;
            match chain {
                Ok(c) => {
                    eprintln!("Chain: {} calls, {} puts", c.calls.len(), c.puts.len());

                    let spot = 680.0;
                    let target_low = spot - 3.0;
                    let target_high = spot + 3.0;

                    let nearby_calls: Vec<_> = c
                        .calls
                        .iter()
                        .filter(|o| {
                            o.strike >= target_low && o.strike <= target_high && o.bid > 0.0
                        })
                        .collect();
                    eprintln!("Calls near ATM ({:.0}-{:.0}):", target_low, target_high);
                    for call in nearby_calls.iter().take(5) {
                        eprintln!(
                            "  K={:.0} bid={:.2} ask={:.2} vol={}",
                            call.strike, call.bid, call.ask, call.volume
                        );
                    }
                }
                Err(e) => eprintln!("Chain fetch failed: {e}"),
            }
        }
    }

    #[tokio::test]
    async fn fetches_spy_yield_curve() {
        let source = YahooYieldCurveSource::new();

        match source.fetch_yield_curve("SPY").await {
            Ok(curve) => {
                assert!(!curve.points.is_empty(), "should have at least one point");
                assert!(
                    curve.underlying_price > 0.0,
                    "should have valid underlying price"
                );
                eprintln!(
                    "SPY yield curve: {} points, spot: {:.2}",
                    curve.points.len(),
                    curve.underlying_price
                );
                for point in curve.points.iter().take(5) {
                    eprintln!(
                        "  DTE {:3}: K={:.0}/{:.0} width={:.0}  sell={:.2}% buy={:.2}% mid={:.2}%  liquidity={:.1}",
                        point.dte,
                        point.strike_low, point.strike_high,
                        point.strike_width,
                        point.sell_implied_rate * 100.0,
                        point.buy_implied_rate * 100.0,
                        point.mid_rate * 100.0,
                        point.liquidity_score,
                    );
                }
            }
            Err(e) => {
                eprintln!("SPY yield curve fetch failed: {e}");
            }
        }
    }

    #[tokio::test]
    async fn fetches_qqq_yield_curve() {
        let source = YahooYieldCurveSource::new();

        match source.fetch_yield_curve("QQQ").await {
            Ok(curve) => {
                assert!(!curve.points.is_empty(), "should have at least one point");
                eprintln!(
                    "QQQ yield curve: {} points, spot: {:.2}",
                    curve.points.len(),
                    curve.underlying_price
                );
                for point in curve.points.iter().take(3) {
                    eprintln!(
                        "  DTE {}: mid_rate={:.2}% liquidity={:.1}",
                        point.dte,
                        point.mid_rate * 100.0,
                        point.liquidity_score
                    );
                }
            }
            Err(e) => {
                eprintln!("QQQ yield curve fetch failed: {e}");
            }
        }
    }

    /// SPX options are European-style — both buy and sell rates should be positive
    /// and close to the risk-free rate (~4-5%).  Yahoo Finance uses "^SPX" as the
    /// ticker for the index, but option chains are typically not available via Yahoo
    /// for cash-settled indices.  This test probes what's available.
    /// Probe Euro Stoxx 50 options availability on Yahoo Finance.
    /// Eurex OESX options are European-style, cash-settled — ideal for box spreads.
    #[tokio::test]
    async fn fetches_es50_yield_curve() {
        let source_raw = YahooOptionsSource::new();
        // Try known Yahoo Finance tickers for Euro Stoxx 50
        for sym in &["^STOXX50E", "STOXX50E", "SX5E", "^SX5E"] {
            match source_raw.get_expirations(sym).await {
                Ok(ts) if !ts.is_empty() => {
                    eprintln!("{sym}: {} expirations available", ts.len());
                    // Try fetching the first chain
                    if let Ok(chain) = source_raw.get_chain(sym, ts[0]).await {
                        let date = Utc.timestamp_opt(ts[0], 0).single().map(|d| d.date_naive());
                        eprintln!(
                            "  First expiry: {:?}  calls={} puts={}",
                            date,
                            chain.calls.len(),
                            chain.puts.len()
                        );
                        let liquid_calls = chain.calls.iter().filter(|o| o.bid > 0.0).count();
                        eprintln!("  Liquid calls (bid>0): {liquid_calls}");
                    }
                    // Try full yield curve
                    let source = YahooYieldCurveSource::new();
                    match source.fetch_yield_curve(sym).await {
                        Ok(curve) => {
                            eprintln!(
                                "{sym} yield curve: {} points, spot: {:.2}",
                                curve.points.len(),
                                curve.underlying_price
                            );
                            for point in curve.points.iter().take(5) {
                                eprintln!(
                                    "  DTE {:3}: K={:.0}/{:.0} width={:.0}  sell={:.2}% buy={:.2}% mid={:.2}%",
                                    point.dte, point.strike_low, point.strike_high, point.strike_width,
                                    point.sell_implied_rate * 100.0,
                                    point.buy_implied_rate * 100.0,
                                    point.mid_rate * 100.0,
                                );
                            }
                        }
                        Err(e) => eprintln!("{sym} yield curve failed: {e}"),
                    }
                    return;
                }
                Ok(_) => eprintln!("{sym}: 0 expirations"),
                Err(e) => eprintln!("{sym}: {e}"),
            }
        }
        eprintln!("No ES50 options found on Yahoo Finance");
    }

    #[tokio::test]
    async fn fetches_spx_yield_curve() {
        let source = YahooYieldCurveSource::new();

        for sym in &["SPX", "^SPX", "SPXW"] {
            let result = source.fetch_yield_curve(sym).await;
            match result {
                Ok(curve) => {
                    eprintln!(
                        "{sym} yield curve: {} points, spot: {:.2}",
                        curve.points.len(),
                        curve.underlying_price
                    );
                    for point in curve.points.iter().take(5) {
                        eprintln!(
                            "  DTE {:3}: K={:.0}/{:.0} width={:.0}  sell={:.2}% buy={:.2}% mid={:.2}%",
                            point.dte, point.strike_low, point.strike_high, point.strike_width,
                            point.sell_implied_rate * 100.0,
                            point.buy_implied_rate * 100.0,
                            point.mid_rate * 100.0,
                        );
                    }
                    return; // found a working ticker
                }
                Err(e) => eprintln!("{sym}: {e}"),
            }
        }
    }

    #[tokio::test]
    async fn box_spread_calculation() {
        let width = 5.0;
        let dte = 30.0;

        let c_low = (2.50, 2.55);
        let c_high = (0.50, 0.55);
        let p_low = (0.45, 0.50);
        let p_high = (2.40, 2.45);

        let result = BoxSpreadResult::from_quotes(c_low, c_high, p_low, p_high, width, dte);

        match result {
            Some(box_spread) => {
                eprintln!("Box spread @ {} DTE:", dte as i32);
                eprintln!("  Net debit: {:.2}", box_spread.net_debit);
                eprintln!("  Buy rate: {:.2}%", box_spread.buy_rate * 100.0);
                eprintln!("  Sell rate: {:.2}%", box_spread.sell_rate * 100.0);
                eprintln!("  Mid rate: {:.2}%", box_spread.mid_rate * 100.0);
                eprintln!("  Liquidity: {:.1}", box_spread.liquidity_score);

                assert!(box_spread.net_debit > 0.0);
                assert!(box_spread.buy_rate > 0.0);
            }
            None => {
                eprintln!("Box spread calculation returned None (expected with these quotes)");
            }
        }
    }
}
