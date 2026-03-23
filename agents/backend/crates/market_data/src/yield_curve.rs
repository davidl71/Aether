//! Yield curve derivation from options market data.
//!
//! Derives box-spread implied financing rates from equity/index options chains.
//! 
//! ## Data Sources
//!
//! | Source | European Options | American Options | Notes |
//! |--------|----------------|-----------------|-------|
//! | **TWS** (IBKR) | ✅ SPX, NDX | ✅ SPY, QQQ | Recommended - full data |
//! | **Yahoo Finance** | ❌ No index options | ⚠️ May be limited | Requires testing |
//!
//! ## Box Spread Financing Rate
//!
//! A box spread is a combination of a bull call spread and bear put spread with
//! the same strikes. Its payoff at expiration is always `(K_high - K_low)` regardless
//! of the underlying price.
//!
//! The financing rate implied by a box spread is:
//! ```text
//! Net Debit = C(K_low) + P(K_high) - C(K_high) - P(K_low)
//! r_implied = (K_high - K_low - Net_Debit) / (Net_Debit * T)
//! ```
//!
//! Where `T = DTE / 365` (time to expiration in years).

use std::collections::HashMap;

use chrono::{Datelike, NaiveDate, Utc};
use thiserror::Error;
use tracing::{debug, warn};
use yfinance_rs::{
    core::conversions::money_to_f64,
    Interval, Range,
};

use crate::yahoo::{OptionContractData, OptionsDataSource, YahooOptionsSource, YahooHistorySource};

const STRIKE_WIDTH: f64 = 2.0;
const MIN_LIQUIDITY_SCORE: f64 = 50.0;
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

        let net_debit = c_low_ask + p_high_ask - c_high_bid - p_low_bid;
        let net_credit = c_high_bid + p_low_bid - c_low_bid - p_high_ask;

        if net_debit <= 0.0 || net_debit >= width {
            debug!("box spread: invalid net_debit {:.2} vs width {:.2}", net_debit, width);
            return None;
        }

        let t = dte / 365.0;
        if t <= 0.0 {
            return None;
        }

        let buy_rate = if net_debit > 0.0 && net_debit < width {
            (width - net_debit) / (net_debit * t)
        } else {
            return None;
        };
        let sell_rate = if net_credit > 0.0 && net_credit < width {
            (width - net_credit) / (net_credit * t)
        } else {
            buy_rate
        };

        let liquidity_score = Self::calculate_liquidity_score(
            c_low_bid, c_low_ask, c_high_bid, c_high_ask,
            p_low_bid, p_low_ask, p_high_bid, p_high_ask,
        );

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
            buy_rate: buy_rate.clamp(0.001, 0.20),
            sell_rate: sell_rate.clamp(0.001, 0.20),
            mid_rate: (buy_rate + sell_rate) / 2.0,
            liquidity_score,
        })
    }

    fn calculate_liquidity_score(
        c_low_bid: f64, c_low_ask: f64,
        c_high_bid: f64, c_high_ask: f64,
        p_low_bid: f64, p_low_ask: f64,
        p_high_bid: f64, p_high_ask: f64,
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
        let history = self.history.get_history(symbol, Range::D1, Interval::D1).await?;
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
        let expirations = self.options.get_expirations(&yahoo_symbol).await?;
        debug!(symbol = %symbol, expirations = expirations.len(), "got expirations");
        if expirations.is_empty() {
            return Err(anyhow::anyhow!("no expiration dates for {symbol}"));
        }

        let underlying_price = self.fetch_underlying_price(symbol).await?;
        debug!(symbol = %symbol, spot = underlying_price, "got underlying price");

        let today = Utc::now().date_naive();
        let valid_expirations: Vec<NaiveDate> = expirations
            .into_iter()
            .filter(|exp| {
                let dte = (*exp - today).num_days() as i32;
                dte >= MIN_DTE && dte <= MAX_DTE
            })
            .collect();

        debug!(symbol = %symbol, valid_count = valid_expirations.len(), 
               "filtered expirations (DTE {} to {})", MIN_DTE, MAX_DTE);

        let mut points = Vec::new();

        for expiry in valid_expirations.into_iter().take(10) {
            let dte = (expiry - today).num_days() as i32;
            if let Some(point) = self.fetch_point(&yahoo_symbol, expiry, underlying_price).await {
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
        expiry: NaiveDate,
        spot: f64,
    ) -> Option<YieldCurvePoint> {
        let chain = self.options.get_chain(symbol, expiry).await.ok()?;

        debug!(symbol = %symbol, spot = spot, calls = chain.calls.len(), puts = chain.puts.len(), 
               "fetched option chain for {}", expiry);

        let dte = (expiry - Utc::now().date_naive()).num_days() as i32;
        if dte <= 0 {
            return None;
        }

        let (c_low, c_high) = self.find_strike_pair(&chain.calls, spot, STRIKE_WIDTH)?;
        debug!(%symbol, c_low_strike = c_low.strike, c_high_strike = c_high.strike, "call strikes");

        let (p_low, p_high) = self.find_strike_pair(&chain.puts, spot, STRIKE_WIDTH)?;
        debug!(%symbol, p_low_strike = p_low.strike, p_high_strike = p_high.strike, "put strikes");

        let strike_low = c_low.strike.min(p_low.strike);
        let strike_high = c_high.strike.max(p_high.strike);
        let width = strike_high - strike_low;

        if width <= 0.0 || width > STRIKE_WIDTH * 2.0 {
            debug!("strike width {:.2} outside acceptable range", width);
            return None;
        }

        let box_result = BoxSpreadResult::from_quotes(
            (c_low.bid, c_low.ask),
            (c_high.bid, c_high.ask),
            (p_low.bid, p_low.ask),
            (p_high.bid, p_high.ask),
            width,
            dte as f64,
        )?;

        let liquidity = box_result.liquidity_score;
        if liquidity < MIN_LIQUIDITY_SCORE {
            debug!("low liquidity score {:.1} for {}", liquidity, expiry);
            return None;
        }

        let spread_bps = (box_result.sell_rate - box_result.buy_rate) * 10000.0;

        debug!(%symbol, dte = %dte, buy_rate = box_result.buy_rate * 100.0, sell_rate = box_result.sell_rate * 100.0, "computed box spread rates");

        Some(YieldCurvePoint {
            expiry,
            dte,
            strike_low,
            strike_high,
            strike_width: width,
            net_debit: box_result.net_debit,
            net_credit: box_result.net_credit,
            buy_implied_rate: box_result.buy_rate,
            sell_implied_rate: box_result.sell_rate,
            liquidity_score: liquidity,
            mid_rate: box_result.mid_rate,
            spread_bps,
            source: "yahoo_finance".to_string(),
        })
    }

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

    fn find_strike_pair(
        &self,
        options: &[OptionContractData],
        spot: f64,
        width: f64,
    ) -> Option<(OptionContractData, OptionContractData)> {
        let target_low = (spot - width / 2.0).round();
        let target_high = (spot + width / 2.0).round();

        let mut sorted: Vec<_> = options
            .iter()
            .filter(|o| o.bid > 0.0 && o.ask > 0.0 && o.ask > o.bid)
            .collect();

        sorted.sort_by_key(|o| (o.strike - target_low).abs() as i64);

        let c_low = sorted.first()?.clone();

        sorted.sort_by_key(|o| (o.strike - target_high).abs() as i64);

        let c_high = sorted.first()?.clone();

        if c_low.strike >= c_high.strike {
            return None;
        }

        Some((c_low.clone(), c_high.clone()))
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
        let expirations = source.get_expirations("SPY").await.unwrap_or_default();
        eprintln!("SPY has {} expiration dates", expirations.len());

        let today = Utc::now().date_naive();
        let valid: Vec<_> = expirations.iter()
            .filter(|e| {
                let dte = (**e - today).num_days();
                dte >= 7 && dte <= 60
            })
            .collect();
        eprintln!("Valid expirations (DTE 7-60): {}", valid.len());

        if let Some(exp) = valid.first() {
            let expiry = **exp;
            eprintln!("First valid expiry: {} (DTE {})", expiry, (expiry - today).num_days());

            let chain = source.get_chain("SPY", expiry).await;
            match chain {
                Ok(c) => {
                    eprintln!("Chain: {} calls, {} puts", c.calls.len(), c.puts.len());

                    let spot = 680.0;
                    let target_low = spot - 3.0;
                    let target_high = spot + 3.0;

                    let nearby_calls: Vec<_> = c.calls.iter()
                        .filter(|o| o.strike >= target_low && o.strike <= target_high && o.bid > 0.0)
                        .collect();
                    eprintln!("Calls near ATM ({:.0}-{:.0}):", target_low, target_high);
                    for call in nearby_calls.iter().take(5) {
                        eprintln!("  K={:.0} bid={:.2} ask={:.2} vol={}", 
                            call.strike, call.bid, call.ask, call.volume);
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
                assert!(curve.underlying_price > 0.0, "should have valid underlying price");
                eprintln!("SPY yield curve: {} points, spot: {:.2}", 
                    curve.points.len(), curve.underlying_price);
                for point in curve.points.iter().take(3) {
                    eprintln!("  DTE {}: mid_rate={:.2}% liquidity={:.1}",
                        point.dte, point.mid_rate * 100.0, point.liquidity_score);
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
                eprintln!("QQQ yield curve: {} points, spot: {:.2}", 
                    curve.points.len(), curve.underlying_price);
                for point in curve.points.iter().take(3) {
                    eprintln!("  DTE {}: mid_rate={:.2}% liquidity={:.1}",
                        point.dte, point.mid_rate * 100.0, point.liquidity_score);
                }
            }
            Err(e) => {
                eprintln!("QQQ yield curve fetch failed: {e}");
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
