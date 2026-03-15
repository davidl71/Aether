use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OptionContract {
    pub symbol: String,
    pub expiry: String,
    pub strike: f64,
    pub is_call: bool,
}

impl OptionContract {
    pub fn new(symbol: &str, expiry: &str, strike: f64, is_call: bool) -> Self {
        Self {
            symbol: symbol.to_string(),
            expiry: expiry.to_string(),
            strike,
            is_call,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.symbol.is_empty() && self.expiry.len() == 8 && self.strike > 0.0
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MarketData {
    pub bid: f64,
    pub ask: f64,
    pub last: f64,
    pub volume: i64,
    pub open_interest: i64,
    pub implied_volatility: Option<f64>,
}

impl MarketData {
    pub fn new(bid: f64, ask: f64) -> Self {
        Self {
            bid,
            ask,
            last: (bid + ask) / 2.0,
            volume: 0,
            open_interest: 0,
            implied_volatility: None,
        }
    }

    pub fn mid(&self) -> f64 {
        (self.bid + self.ask) / 2.0
    }

    pub fn spread(&self) -> f64 {
        self.ask - self.bid
    }
}

#[derive(Clone, Debug)]
pub struct OptionChainEntry {
    pub contract: OptionContract,
    pub market_data: MarketData,
    pub theoretical_price: f64,
    pub intrinsic_value: f64,
    pub extrinsic_value: f64,
    pub moneyness: f64,
}

impl OptionChainEntry {
    pub fn new(contract: OptionContract, market_data: MarketData, underlying_price: f64) -> Self {
        let moneyness = if underlying_price > 0.0 {
            contract.strike / underlying_price
        } else {
            1.0
        };

        let intrinsic_value = if contract.is_call {
            (underlying_price - contract.strike).max(0.0)
        } else {
            (contract.strike - underlying_price).max(0.0)
        };

        let mid = market_data.mid();
        let extrinsic_value = (mid - intrinsic_value).max(0.0);

        Self {
            contract,
            market_data,
            theoretical_price: mid,
            intrinsic_value,
            extrinsic_value,
            moneyness,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.contract.is_valid() && self.market_data.bid > 0.0 && self.market_data.ask > 0.0
    }
}

#[derive(Clone, Debug)]
pub struct StrikeChain {
    pub strike: f64,
    pub call: Option<OptionChainEntry>,
    pub put: Option<OptionChainEntry>,
}

impl StrikeChain {
    pub fn new(strike: f64) -> Self {
        Self {
            strike,
            call: None,
            put: None,
        }
    }

    pub fn has_both(&self) -> bool {
        self.call.is_some() && self.put.is_some()
    }

    pub fn call_iv(&self) -> f64 {
        self.call
            .as_ref()
            .and_then(|c| c.market_data.implied_volatility)
            .unwrap_or(0.0)
    }

    pub fn put_iv(&self) -> f64 {
        self.put
            .as_ref()
            .and_then(|p| p.market_data.implied_volatility)
            .unwrap_or(0.0)
    }

    pub fn iv_skew(&self) -> f64 {
        self.put_iv() - self.call_iv()
    }
}

#[derive(Clone, Debug)]
pub struct ExpiryChain {
    symbol: String,
    expiry: String,
    strikes: BTreeMap<i64, StrikeChain>,
}

impl ExpiryChain {
    pub fn new(symbol: &str, expiry: &str) -> Self {
        Self {
            symbol: symbol.to_string(),
            expiry: expiry.to_string(),
            strikes: BTreeMap::new(),
        }
    }

    pub fn add_option(&mut self, entry: OptionChainEntry) {
        let strike_key = (entry.contract.strike * 1000.0) as i64;
        let chain = self
            .strikes
            .entry(strike_key)
            .or_insert_with(|| StrikeChain::new(entry.contract.strike));

        if entry.contract.is_call {
            chain.call = Some(entry);
        } else {
            chain.put = Some(entry);
        }
    }

    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    pub fn expiry(&self) -> &str {
        &self.expiry
    }

    pub fn days_to_expiry(&self) -> i32 {
        let (y, m, d) = match common::expiry::parse_expiry_yyyy_mm_dd(&self.expiry) {
            Ok(ymd) => ymd,
            Err(_) => return 0,
        };
        let year = y as i32;
        let month = time::Month::try_from(m).unwrap_or(time::Month::January);
        let expiry_date = match time::Date::from_calendar_date(year, month, d) {
            Ok(exp) => exp,
            Err(_) => return 0,
        };
        let today = time::OffsetDateTime::now_utc().date();
        (expiry_date - today).whole_days() as i32
    }

    pub fn get_strikes(&self) -> Vec<f64> {
        self.strikes.values().map(|s| s.strike).collect()
    }

    pub fn get_strike_chain(&self, strike: f64) -> Option<&StrikeChain> {
        let strike_key = (strike * 1000.0) as i64;
        self.strikes.get(&strike_key)
    }

    pub fn get_option(&self, strike: f64, is_call: bool) -> Option<&OptionChainEntry> {
        let strike_key = (strike * 1000.0) as i64;
        self.strikes.get(&strike_key).and_then(|chain| {
            if is_call {
                chain.call.as_ref()
            } else {
                chain.put.as_ref()
            }
        })
    }

    pub fn get_strikes_in_range(&self, min_strike: f64, max_strike: f64) -> Vec<f64> {
        self.get_strikes()
            .into_iter()
            .filter(|&s| s >= min_strike && s <= max_strike)
            .collect()
    }

    pub fn find_atm_strike(&self, underlying_price: f64) -> Option<f64> {
        self.get_strikes().into_iter().min_by(|a, b| {
            let dist_a = (a - underlying_price).abs();
            let dist_b = (b - underlying_price).abs();
            dist_a.partial_cmp(&dist_b).unwrap()
        })
    }

    pub fn get_all_options(&self) -> Vec<&OptionChainEntry> {
        self.strikes
            .values()
            .flat_map(|chain| chain.call.as_ref().into_iter().chain(chain.put.as_ref()))
            .collect()
    }

    pub fn get_calls(&self) -> Vec<&OptionChainEntry> {
        self.strikes
            .values()
            .filter_map(|chain| chain.call.as_ref())
            .collect()
    }

    pub fn get_puts(&self) -> Vec<&OptionChainEntry> {
        self.strikes
            .values()
            .filter_map(|chain| chain.put.as_ref())
            .collect()
    }
}

#[derive(Clone, Debug)]
pub struct OptionChain {
    symbol: String,
    underlying_price: f64,
    expiries: BTreeMap<String, ExpiryChain>,
}

impl OptionChain {
    pub fn new(symbol: &str) -> Self {
        Self {
            symbol: symbol.to_string(),
            underlying_price: 0.0,
            expiries: BTreeMap::new(),
        }
    }

    pub fn add_option(&mut self, entry: OptionChainEntry) {
        let expiry = entry.contract.expiry.clone();
        let chain = self
            .expiries
            .entry(expiry)
            .or_insert_with(|| ExpiryChain::new(&self.symbol, &entry.contract.expiry));
        chain.add_option(entry);
    }

    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    pub fn get_expiries(&self) -> Vec<String> {
        self.expiries.keys().cloned().collect()
    }

    pub fn get_expiry_chain(&self, expiry: &str) -> Option<&ExpiryChain> {
        self.expiries.get(expiry)
    }

    pub fn get_expiries_in_dte_range(&self, min_dte: i32, max_dte: i32) -> Vec<String> {
        self.expiries
            .iter()
            .filter(|(_, chain)| {
                let dte = chain.days_to_expiry();
                dte >= min_dte && dte <= max_dte
            })
            .map(|(expiry, _)| expiry.clone())
            .collect()
    }

    pub fn get_all_options(&self) -> Vec<&OptionChainEntry> {
        self.expiries
            .values()
            .flat_map(|chain| chain.get_all_options())
            .collect()
    }

    pub fn set_underlying_price(&mut self, price: f64) {
        self.underlying_price = price;
    }

    pub fn underlying_price(&self) -> f64 {
        self.underlying_price
    }

    pub fn total_option_count(&self) -> usize {
        self.expiries
            .values()
            .map(|c| c.get_all_options().len())
            .sum()
    }

    pub fn expiry_count(&self) -> usize {
        self.expiries.len()
    }
}
