pub mod amortization;
pub mod convexity;
pub mod margin;
pub mod option_chain;
pub mod yield_curve;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::Date;
use RustQuant::instruments::options::{implied_volatility, BlackScholesMerton, TypeFlag};
use RustQuant_stochastics::geometric_brownian_motion::GeometricBrownianMotion;
use RustQuant_stochastics::process::{StochasticProcess, StochasticProcessConfig};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OptionKind {
    Call,
    Put,
}

impl From<OptionKind> for TypeFlag {
    fn from(kind: OptionKind) -> Self {
        match kind {
            OptionKind::Call => TypeFlag::Call,
            OptionKind::Put => TypeFlag::Put,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Greeks {
    pub delta: f64,
    pub gamma: f64,
    pub theta: f64,
    pub vega: f64,
    pub rho: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct BondGreeks {
    pub delta: f64,
    pub gamma: f64,
    pub theta: f64,
    pub vega: f64,
    pub rho: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Position {
    pub symbol: String,
    pub quantity: f64,
    pub current_price: f64,
    pub is_option: bool,
    pub strike: Option<f64>,
    pub expiry: Option<String>,
    pub option_type: Option<OptionKind>,
}

impl Position {
    pub fn stock(symbol: &str, quantity: f64, price: f64) -> Self {
        Self {
            symbol: symbol.to_string(),
            quantity,
            current_price: price,
            is_option: false,
            strike: None,
            expiry: None,
            option_type: None,
        }
    }

    pub fn option(
        symbol: &str,
        quantity: f64,
        price: f64,
        strike: f64,
        expiry: &str,
        opt_type: OptionKind,
    ) -> Self {
        Self {
            symbol: symbol.to_string(),
            quantity,
            current_price: price,
            is_option: true,
            strike: Some(strike),
            expiry: Some(expiry.to_string()),
            option_type: Some(opt_type),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HistoricalVolatilityResult {
    pub hv: f64,
    pub sample_std_dev: f64,
    pub variance: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RiskMetrics {
    pub var_95: f64,
    pub cvar_95: f64,
    pub max_loss: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyResult {
    pub name: String,
    pub payoff_at_expiry: f64,
    pub max_profit: f64,
    pub max_loss: f64,
    pub breakeven: Vec<f64>,
    pub cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoxSpreadResult {
    pub synthetic_leg_cost: f64,
    pub actual_leg_cost: f64,
    pub net_cost: f64,
    pub annualized_rate: f64,
    pub is_arbitrage: bool,
    pub legs: Vec<BoxSpreadLeg>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoxSpreadLeg {
    pub instrument: String,
    pub strike: f64,
    pub expiry: f64,
    pub side: String,
    pub price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComboResult {
    pub name: String,
    pub net_debit: f64,
    pub max_profit: f64,
    pub max_loss: f64,
    pub breakeven: Vec<f64>,
    pub legs: Vec<ComboLeg>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComboLeg {
    pub instrument: String,
    pub strike: f64,
    pub option_type: String,
    pub side: String,
    pub quantity: f64,
    pub price: f64,
}

#[derive(Debug, Error)]
pub enum QuantError {
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    #[error("Calculation failed: {0}")]
    CalculationFailed(String),
    #[error("Implied volatility not found")]
    ImpliedVolatilityNotFound,
    #[error("Insufficient data: {0}")]
    InsufficientData(String),
    #[error("ML error: {0}")]
    MlError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonteCarloResult {
    pub price: f64,
    pub standard_error: f64,
    pub confidence_lower: f64,
    pub confidence_upper: f64,
    pub simulations: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearRegressionResult {
    pub coefficients: Vec<f64>,
    pub intercept: f64,
    pub r_squared: f64,
    pub predictions: Vec<f64>,
}

pub struct QuantCalculator;

impl QuantCalculator {
    pub fn new() -> Self {
        Self
    }

    fn parse_expiry_to_days(expiry: &str) -> Result<i64, QuantError> {
        if expiry.len() != 8 {
            return Err(QuantError::InvalidParameter(
                "Expiry must be YYYYMMDD format".to_string(),
            ));
        }

        let year: i32 = expiry[0..4]
            .parse()
            .map_err(|_| QuantError::InvalidParameter("Invalid year in expiry".to_string()))?;
        let month: u32 = expiry[4..6]
            .parse()
            .map_err(|_| QuantError::InvalidParameter("Invalid month in expiry".to_string()))?;
        let day: u32 = expiry[6..8]
            .parse()
            .map_err(|_| QuantError::InvalidParameter("Invalid day in expiry".to_string()))?;

        let month: u8 = month
            .try_into()
            .map_err(|_| QuantError::InvalidParameter("Invalid month".to_string()))?;
        let day: u8 = day
            .try_into()
            .map_err(|_| QuantError::InvalidParameter("Invalid day".to_string()))?;

        let month = time::Month::try_from(month)
            .map_err(|_| QuantError::InvalidParameter("Invalid month".to_string()))?;

        let expiry_date = Date::from_calendar_date(year, month, day)
            .map_err(|_| QuantError::InvalidParameter("Invalid date".to_string()))?;

        let today = time::OffsetDateTime::now_utc().date();
        let days = (expiry_date - today).whole_days();

        if days < 0 {
            return Err(QuantError::InvalidParameter(
                "Expiry date is in the past".to_string(),
            ));
        }

        Ok(days)
    }

    fn days_to_expiry(&self, days: i64) -> Date {
        let today = time::OffsetDateTime::now_utc().date();
        today + time::Duration::days(days)
    }

    pub fn calculate_option_price(
        &self,
        s: f64,
        k: f64,
        t_years: f64,
        r: f64,
        sigma: f64,
        option_type: OptionKind,
    ) -> Result<f64, QuantError> {
        if s <= 0.0 || k <= 0.0 || t_years <= 0.0 || sigma < 0.0 {
            return Err(QuantError::InvalidParameter(
                "S, K, T must be positive, sigma must be non-negative".to_string(),
            ));
        }

        let days = (t_years * 365.0).round() as i64;
        let expiry = self.days_to_expiry(days);

        let bsm = BlackScholesMerton::new(r, s, k, sigma, r, None, expiry, option_type.into());

        Ok(bsm.price())
    }

    pub fn calculate_greeks(
        &self,
        s: f64,
        k: f64,
        t_years: f64,
        r: f64,
        sigma: f64,
        option_type: OptionKind,
    ) -> Result<Greeks, QuantError> {
        if s <= 0.0 || k <= 0.0 || t_years <= 0.0 || sigma < 0.0 {
            return Err(QuantError::InvalidParameter(
                "S, K, T must be positive, sigma must be non-negative".to_string(),
            ));
        }

        let days = (t_years * 365.0).round() as i64;
        let expiry = self.days_to_expiry(days);

        let bsm = BlackScholesMerton::new(r, s, k, sigma, r, None, expiry, option_type.into());

        Ok(Greeks {
            delta: bsm.delta(),
            gamma: bsm.gamma(),
            theta: bsm.theta() / 365.0,
            vega: bsm.vega() / 100.0,
            rho: bsm.rho() / 100.0,
        })
    }

    pub fn calculate_implied_volatility(
        &self,
        market_price: f64,
        s: f64,
        k: f64,
        t_years: f64,
        r: f64,
        option_type: OptionKind,
    ) -> Result<f64, QuantError> {
        if market_price <= 0.0 || s <= 0.0 || k <= 0.0 || t_years <= 0.0 {
            return Err(QuantError::InvalidParameter(
                "All prices and parameters must be positive".to_string(),
            ));
        }

        let iv = implied_volatility(market_price, s, k, t_years, r, option_type.into());

        if iv.is_finite() {
            Ok(iv)
        } else {
            Err(QuantError::ImpliedVolatilityNotFound)
        }
    }

    pub fn calculate_historical_volatility(
        &self,
        prices: &[f64],
        annualization_factor: f64,
    ) -> Result<HistoricalVolatilityResult, QuantError> {
        if prices.len() < 2 {
            return Err(QuantError::InsufficientData(
                "Need at least 2 prices".to_string(),
            ));
        }

        let returns: Vec<f64> = prices.windows(2).map(|w| (w[1] - w[0]) / w[0]).collect();

        if returns.is_empty() {
            return Err(QuantError::InsufficientData(
                "Could not compute returns".to_string(),
            ));
        }

        let n = returns.len() as f64;
        let mean = returns.iter().sum::<f64>() / n;
        let variance = returns.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
        let std_dev = variance.sqrt();
        let hv = std_dev * annualization_factor;

        Ok(HistoricalVolatilityResult {
            hv,
            sample_std_dev: std_dev,
            variance,
        })
    }

    pub fn calculate_var_cvar(
        &self,
        returns: &[f64],
        confidence: f64,
    ) -> Result<RiskMetrics, QuantError> {
        if returns.is_empty() {
            return Err(QuantError::InsufficientData(
                "Need at least 1 return".to_string(),
            ));
        }

        if confidence <= 0.0 || confidence >= 1.0 {
            return Err(QuantError::InvalidParameter(
                "Confidence must be between 0 and 1".to_string(),
            ));
        }

        let mut sorted_returns = returns.to_vec();
        sorted_returns.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let index = ((1.0 - confidence) * sorted_returns.len() as f64).floor() as usize;
        let var = -sorted_returns.get(index).copied().unwrap_or(0.0);

        let cvar = if index > 0 {
            -sorted_returns[..index].iter().sum::<f64>() / index as f64
        } else {
            var
        };

        let max_loss = -sorted_returns.iter().copied().fold(f64::INFINITY, f64::min);

        Ok(RiskMetrics {
            var_95: var,
            cvar_95: cvar,
            max_loss,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn calculate_binomial_option_price(
        &self,
        s: f64,
        k: f64,
        t_years: f64,
        r: f64,
        sigma: f64,
        option_type: OptionKind,
        steps: u32,
    ) -> Result<f64, QuantError> {
        if s <= 0.0 || k <= 0.0 || t_years <= 0.0 || sigma < 0.0 || steps < 1 {
            return Err(QuantError::InvalidParameter(
                "Invalid parameters for binomial pricing".to_string(),
            ));
        }

        let dt = t_years / steps as f64;
        let u = (sigma * dt.sqrt()).exp();
        let d = 1.0 / u;
        let p = ((r - 0.5 * sigma * sigma) * dt + 1.0 - d) / (u - d);
        let df = (-r * dt).exp();

        let mut prices: Vec<f64> = vec![0.0; (steps + 1) as usize];
        for i in 0..=steps {
            prices[i as usize] = s * u.powi(i as i32) * d.powi((steps - i) as i32);
        }

        let payoff = |price: f64| match option_type {
            OptionKind::Call => (price - k).max(0.0),
            OptionKind::Put => (k - price).max(0.0),
        };

        let mut option_values: Vec<f64> = (0..=steps).map(|i| payoff(prices[i as usize])).collect();

        for j in (0..steps).rev() {
            for i in 0..=j {
                option_values[i as usize] = df
                    * (p * option_values[(i + 1) as usize] + (1.0 - p) * option_values[i as usize]);
            }
        }

        Ok(option_values[0])
    }

    pub fn calculate_straddle(
        &self,
        s: f64,
        k: f64,
        t_years: f64,
        r: f64,
        sigma: f64,
    ) -> Result<StrategyResult, QuantError> {
        let call_price = self.calculate_option_price(s, k, t_years, r, sigma, OptionKind::Call)?;
        let put_price = self.calculate_option_price(s, k, t_years, r, sigma, OptionKind::Put)?;

        let cost = call_price + put_price;
        let max_profit = f64::INFINITY;
        let max_loss = cost;
        let breakeven = vec![k - cost, k + cost];

        Ok(StrategyResult {
            name: "Straddle".to_string(),
            payoff_at_expiry: cost,
            max_profit,
            max_loss,
            breakeven,
            cost,
        })
    }

    pub fn calculate_strangle(
        &self,
        s: f64,
        k_call: f64,
        k_put: f64,
        t_years: f64,
        r: f64,
        sigma: f64,
    ) -> Result<StrategyResult, QuantError> {
        if k_put >= k_call {
            return Err(QuantError::InvalidParameter(
                "k_put must be less than k_call".to_string(),
            ));
        }

        let call_price =
            self.calculate_option_price(s, k_call, t_years, r, sigma, OptionKind::Call)?;
        let put_price =
            self.calculate_option_price(s, k_put, t_years, r, sigma, OptionKind::Put)?;

        let cost = call_price + put_price;
        let max_profit = f64::INFINITY;
        let max_loss = cost;
        let breakeven = vec![k_put - cost, k_call + cost];

        Ok(StrategyResult {
            name: "Strangle".to_string(),
            payoff_at_expiry: cost,
            max_profit,
            max_loss,
            breakeven,
            cost,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn calculate_butterfly_spread(
        &self,
        s: f64,
        k_low: f64,
        k_mid: f64,
        k_high: f64,
        t_years: f64,
        r: f64,
        sigma: f64,
    ) -> Result<StrategyResult, QuantError> {
        if k_mid != (k_low + k_high) / 2.0 {
            return Err(QuantError::InvalidParameter(
                "k_mid must be midpoint of k_low and k_high".to_string(),
            ));
        }

        let long_call_low =
            self.calculate_option_price(s, k_low, t_years, r, sigma, OptionKind::Call)?;
        let short_calls_mid =
            self.calculate_option_price(s, k_mid, t_years, r, sigma, OptionKind::Call)?;
        let long_call_high =
            self.calculate_option_price(s, k_high, t_years, r, sigma, OptionKind::Call)?;

        let cost = long_call_low - 2.0 * short_calls_mid + long_call_high;
        let max_profit = k_mid - k_low - cost;
        let max_loss = cost;
        let breakeven = vec![k_low + cost, k_high - cost];

        Ok(StrategyResult {
            name: "Butterfly Spread".to_string(),
            payoff_at_expiry: cost,
            max_profit,
            max_loss,
            breakeven,
            cost,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn calculate_iron_condor(
        &self,
        s: f64,
        k_put_low: f64,
        k_put_high: f64,
        k_call_low: f64,
        k_call_high: f64,
        t_years: f64,
        r: f64,
        sigma: f64,
    ) -> Result<StrategyResult, QuantError> {
        if k_put_low >= k_put_high || k_call_low >= k_call_high || k_put_high >= k_call_low {
            return Err(QuantError::InvalidParameter(
                "Strike ordering invalid".to_string(),
            ));
        }

        let put_long =
            self.calculate_option_price(s, k_put_low, t_years, r, sigma, OptionKind::Put)?;
        let put_short =
            self.calculate_option_price(s, k_put_high, t_years, r, sigma, OptionKind::Put)?;
        let call_short =
            self.calculate_option_price(s, k_call_low, t_years, r, sigma, OptionKind::Call)?;
        let call_long =
            self.calculate_option_price(s, k_call_high, t_years, r, sigma, OptionKind::Call)?;

        let cost = put_long - put_short + call_short - call_long;
        let max_profit = cost;
        let max_loss = (k_put_high - k_put_low).min(k_call_high - k_call_low) - cost;
        let breakeven = vec![k_put_high - cost, k_call_low + cost];

        Ok(StrategyResult {
            name: "Iron Condor".to_string(),
            payoff_at_expiry: cost,
            max_profit,
            max_loss,
            breakeven,
            cost,
        })
    }

    pub fn calculate_box_spread(
        &self,
        s: f64,
        k_low: f64,
        k_high: f64,
        t_years: f64,
        r: f64,
        sigma: f64,
    ) -> Result<BoxSpreadResult, QuantError> {
        if k_low >= k_high {
            return Err(QuantError::InvalidParameter(
                "k_low must be less than k_high".to_string(),
            ));
        }

        let synthetic_long_call =
            self.calculate_option_price(s, k_low, t_years, r, sigma, OptionKind::Call)?;
        let synthetic_long_put =
            self.calculate_option_price(s, k_low, t_years, r, sigma, OptionKind::Put)?;
        let synthetic_short_call =
            self.calculate_option_price(s, k_high, t_years, r, sigma, OptionKind::Call)?;
        let synthetic_short_put =
            self.calculate_option_price(s, k_high, t_years, r, sigma, OptionKind::Put)?;

        let synthetic_leg_cost = (synthetic_long_call - synthetic_long_put).abs();
        let actual_leg_cost = s - k_high;

        let net_cost = synthetic_leg_cost - actual_leg_cost;

        let days = (t_years * 365.0).round() as i64;
        let annualized_rate = if net_cost > 0.0 {
            (net_cost / actual_leg_cost) * (365.0 / days as f64)
        } else {
            0.0
        };

        let is_arbitrage = annualized_rate > r * 1.5;

        let legs = vec![
            BoxSpreadLeg {
                instrument: "Synthetic Long".to_string(),
                strike: k_low,
                expiry: t_years,
                side: "Long Call".to_string(),
                price: synthetic_long_call,
            },
            BoxSpreadLeg {
                instrument: "Synthetic Long".to_string(),
                strike: k_low,
                expiry: t_years,
                side: "Short Put".to_string(),
                price: synthetic_long_put,
            },
            BoxSpreadLeg {
                instrument: "Synthetic Short".to_string(),
                strike: k_high,
                expiry: t_years,
                side: "Short Call".to_string(),
                price: synthetic_short_call,
            },
            BoxSpreadLeg {
                instrument: "Synthetic Short".to_string(),
                strike: k_high,
                expiry: t_years,
                side: "Long Put".to_string(),
                price: synthetic_short_put,
            },
        ];

        Ok(BoxSpreadResult {
            synthetic_leg_cost,
            actual_leg_cost,
            net_cost,
            annualized_rate,
            is_arbitrage,
            legs,
        })
    }

    pub fn calculate_jelly_roll(
        &self,
        s: f64,
        k: f64,
        t_years_short: f64,
        t_years_long: f64,
        r: f64,
        sigma: f64,
    ) -> Result<ComboResult, QuantError> {
        if t_years_short >= t_years_long {
            return Err(QuantError::InvalidParameter(
                "Short expiry must be less than long expiry".to_string(),
            ));
        }

        let call_short =
            self.calculate_option_price(s, k, t_years_short, r, sigma, OptionKind::Call)?;
        let put_short =
            self.calculate_option_price(s, k, t_years_short, r, sigma, OptionKind::Put)?;
        let call_long =
            self.calculate_option_price(s, k, t_years_long, r, sigma, OptionKind::Call)?;
        let put_long =
            self.calculate_option_price(s, k, t_years_long, r, sigma, OptionKind::Put)?;

        let net_debit = (call_short + put_short) - (call_long + put_long);

        let legs = vec![
            ComboLeg {
                instrument: "Call".to_string(),
                strike: k,
                option_type: "Call".to_string(),
                side: "Long".to_string(),
                quantity: 1.0,
                price: call_short,
            },
            ComboLeg {
                instrument: "Put".to_string(),
                strike: k,
                option_type: "Put".to_string(),
                side: "Long".to_string(),
                quantity: 1.0,
                price: put_short,
            },
            ComboLeg {
                instrument: "Call".to_string(),
                strike: k,
                option_type: "Call".to_string(),
                side: "Short".to_string(),
                quantity: 1.0,
                price: call_long,
            },
            ComboLeg {
                instrument: "Put".to_string(),
                strike: k,
                option_type: "Put".to_string(),
                side: "Short".to_string(),
                quantity: 1.0,
                price: put_long,
            },
        ];

        Ok(ComboResult {
            name: "Jelly Roll".to_string(),
            net_debit,
            max_profit: f64::INFINITY,
            max_loss: net_debit.abs(),
            breakeven: vec![],
            legs,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn calculate_ratio_spread(
        &self,
        s: f64,
        k_call: f64,
        k_put: f64,
        t_years: f64,
        r: f64,
        sigma: f64,
        ratio: i32,
    ) -> Result<ComboResult, QuantError> {
        let call_price =
            self.calculate_option_price(s, k_call, t_years, r, sigma, OptionKind::Call)?;
        let put_price =
            self.calculate_option_price(s, k_put, t_years, r, sigma, OptionKind::Put)?;

        let net_debit = call_price * ratio as f64 - put_price;

        let legs = vec![
            ComboLeg {
                instrument: "Call".to_string(),
                strike: k_call,
                option_type: "Call".to_string(),
                side: "Long".to_string(),
                quantity: ratio as f64,
                price: call_price,
            },
            ComboLeg {
                instrument: "Put".to_string(),
                strike: k_put,
                option_type: "Put".to_string(),
                side: "Short".to_string(),
                quantity: 1.0,
                price: put_price,
            },
        ];

        Ok(ComboResult {
            name: "Ratio Spread".to_string(),
            net_debit,
            max_profit: f64::INFINITY,
            max_loss: net_debit.abs(),
            breakeven: vec![],
            legs,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn calculate_monte_carlo_option(
        &self,
        s: f64,
        k: f64,
        t_years: f64,
        r: f64,
        sigma: f64,
        option_type: OptionKind,
        simulations: usize,
    ) -> Result<MonteCarloResult, QuantError> {
        if s <= 0.0 || k <= 0.0 || t_years <= 0.0 || sigma < 0.0 || simulations < 1 {
            return Err(QuantError::InvalidParameter(
                "Invalid parameters for Monte Carlo pricing".to_string(),
            ));
        }

        let gbm = GeometricBrownianMotion::new(r, sigma);

        let config = StochasticProcessConfig::new(s, 0.0, t_years, 100, simulations, false);

        let output = gbm.euler_maruyama(&config);

        let final_prices: Vec<f64> = output
            .paths
            .iter()
            .filter_map(|path| path.last().copied())
            .collect();

        if final_prices.len() != simulations {
            return Err(QuantError::CalculationFailed(
                "Monte Carlo simulation failed".to_string(),
            ));
        }

        let payoffs: Vec<f64> = final_prices
            .iter()
            .map(|&spot| match option_type {
                OptionKind::Call => (spot - k).max(0.0),
                OptionKind::Put => (k - spot).max(0.0),
            })
            .collect();

        let discount = (-r * t_years).exp();
        let payoff_mean = payoffs.iter().sum::<f64>() / simulations as f64;
        let price = payoff_mean * discount;

        let variance: f64 = payoffs
            .iter()
            .map(|x| (x - payoff_mean).powi(2))
            .sum::<f64>()
            / simulations as f64;
        let standard_error = variance.sqrt() / (simulations as f64).sqrt();

        let confidence_lower = price - 1.96 * standard_error;
        let confidence_upper = price + 1.96 * standard_error;

        Ok(MonteCarloResult {
            price,
            standard_error,
            confidence_lower,
            confidence_upper,
            simulations,
        })
    }

    pub fn calculate_linear_regression(
        &self,
        x: &[f64],
        y: &[f64],
    ) -> Result<LinearRegressionResult, QuantError> {
        if x.len() != y.len() {
            return Err(QuantError::InvalidParameter(
                "x and y must have same length".to_string(),
            ));
        }

        if x.len() < 2 {
            return Err(QuantError::InsufficientData(
                "Need at least 2 data points".to_string(),
            ));
        }

        let n = x.len() as f64;
        let x_mean = x.iter().sum::<f64>() / n;
        let y_mean = y.iter().sum::<f64>() / n;

        let mut numerator = 0.0;
        let mut denominator = 0.0;
        for i in 0..x.len() {
            numerator += (x[i] - x_mean) * (y[i] - y_mean);
            denominator += (x[i] - x_mean).powi(2);
        }

        let slope = if denominator != 0.0 {
            numerator / denominator
        } else {
            0.0
        };
        let intercept = y_mean - slope * x_mean;

        let mut ss_res = 0.0;
        let mut ss_tot = 0.0;
        for i in 0..x.len() {
            let predicted = slope * x[i] + intercept;
            ss_res += (y[i] - predicted).powi(2);
            ss_tot += (y[i] - y_mean).powi(2);
        }

        let r_squared = if ss_tot != 0.0 {
            1.0 - ss_res / ss_tot
        } else {
            0.0
        };

        let predictions: Vec<f64> = x.iter().map(|&xi| slope * xi + intercept).collect();

        Ok(LinearRegressionResult {
            coefficients: vec![slope],
            intercept,
            r_squared,
            predictions,
        })
    }

    pub fn calculate_stock_greeks(&self, quantity: f64) -> Greeks {
        Greeks {
            delta: quantity,
            gamma: 0.0,
            theta: 0.0,
            vega: 0.0,
            rho: 0.0,
        }
    }

    pub fn calculate_bond_greeks(
        &self,
        price: f64,
        quantity: f64,
        duration: f64,
        convexity: f64,
    ) -> BondGreeks {
        BondGreeks {
            delta: -duration * price / 100.0 * quantity,
            gamma: convexity * price * 0.0001 * quantity,
            theta: 0.0,
            vega: 0.0,
            rho: -duration * price * quantity,
        }
    }

    pub fn calculate_currency_greeks(&self, position_value_local: f64) -> Greeks {
        Greeks {
            delta: position_value_local,
            gamma: 0.0,
            vega: 0.0,
            theta: 0.0,
            rho: 0.0,
        }
    }

    pub fn aggregate_greeks(
        &self,
        positions: &[Position],
        underlying_price: f64,
        risk_free_rate: f64,
        implied_volatility: f64,
        _dividend_yield: f64,
    ) -> Result<Greeks, QuantError> {
        let mut aggregate = Greeks::default();

        for pos in positions {
            if pos.is_option {
                if let (Some(strike), Some(expiry_str), Some(opt_type)) =
                    (pos.strike, &pos.expiry, pos.option_type)
                {
                    let days = Self::parse_expiry_to_days(expiry_str)?;
                    let t_years = days as f64 / 365.0;

                    let greeks = self.calculate_greeks(
                        underlying_price,
                        strike,
                        t_years,
                        risk_free_rate,
                        implied_volatility,
                        opt_type,
                    )?;

                    aggregate.delta += greeks.delta * pos.quantity;
                    aggregate.gamma += greeks.gamma * pos.quantity;
                    aggregate.theta += greeks.theta * pos.quantity;
                    aggregate.vega += greeks.vega * pos.quantity;
                    aggregate.rho += greeks.rho * pos.quantity;
                }
            } else {
                let stock_greeks = self.calculate_stock_greeks(pos.quantity);
                aggregate.delta += stock_greeks.delta;
            }
        }

        Ok(aggregate)
    }
}

impl Default for QuantCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_black_scholes_call_price() {
        let calc = QuantCalculator::new();
        let s = 100.0;
        let k = 100.0;
        let t = 1.0;
        let r = 0.05;
        let sigma = 0.2;

        let price = calc.calculate_option_price(s, k, t, r, sigma, OptionKind::Call);
        assert!(price.is_ok());
        let price = price.unwrap();

        assert!((price - 10.4506).abs() < 0.1);
    }

    #[test]
    fn test_black_scholes_put_price() {
        let calc = QuantCalculator::new();
        let s = 100.0;
        let k = 100.0;
        let t = 1.0;
        let r = 0.05;
        let sigma = 0.2;

        let price = calc.calculate_option_price(s, k, t, r, sigma, OptionKind::Put);
        assert!(price.is_ok());
        let price = price.unwrap();

        assert!((price - 5.5735).abs() < 0.1);
    }

    #[test]
    fn test_greeks_call() {
        let calc = QuantCalculator::new();
        let s = 100.0;
        let k = 100.0;
        let t = 1.0;
        let r = 0.05;
        let sigma = 0.2;

        let greeks = calc.calculate_greeks(s, k, t, r, sigma, OptionKind::Call);
        assert!(greeks.is_ok());
        let g = greeks.unwrap();

        assert!((g.delta - 0.60).abs() < 0.1);
        assert!((g.gamma - 0.0179).abs() < 0.001);
        assert!(g.theta < 0.0);
        assert!(g.vega > 0.0);
    }

    #[test]
    fn test_implied_volatility() {
        let calc = QuantCalculator::new();

        let s = 100.0;
        let k = 100.0;
        let t = 1.0;
        let r = 0.05;
        let sigma = 0.2;

        let call_price = calc
            .calculate_option_price(s, k, t, r, sigma, OptionKind::Call)
            .unwrap();

        let iv = calc.calculate_implied_volatility(call_price, s, k, t, r, OptionKind::Call);
        assert!(iv.is_ok());
        let iv = iv.unwrap();

        assert!((iv - sigma).abs() < 0.01);
    }

    #[test]
    fn test_historical_volatility() {
        let calc = QuantCalculator::new();
        let prices = [
            100.0, 102.0, 101.0, 103.0, 105.0, 104.0, 106.0, 108.0, 107.0, 109.0,
        ];

        let hv = calc.calculate_historical_volatility(&prices, 252.0_f64.sqrt());
        assert!(hv.is_ok());
    }

    #[test]
    fn test_var_cvar() {
        let calc = QuantCalculator::new();
        let returns = [-0.05, -0.02, -0.01, 0.0, 0.01, 0.02, 0.03, 0.05];

        let risk = calc.calculate_var_cvar(&returns, 0.95);
        assert!(risk.is_ok());
        let r = risk.unwrap();
        assert!(r.var_95 >= 0.0);
    }

    #[test]
    fn test_binomial() {
        let calc = QuantCalculator::new();

        let price = calc.calculate_binomial_option_price(
            100.0,
            100.0,
            1.0,
            0.05,
            0.2,
            OptionKind::Call,
            100,
        );
        assert!(price.is_ok());
        let price = price.unwrap();

        assert!((price - 10.4506).abs() < 2.0);
    }

    #[test]
    fn test_straddle() {
        let calc = QuantCalculator::new();

        let result = calc.calculate_straddle(100.0, 100.0, 1.0, 0.05, 0.2);
        assert!(result.is_ok());
        let r = result.unwrap();

        assert_eq!(r.name, "Straddle");
        assert!(r.cost > 0.0);
    }

    #[test]
    fn test_iron_condor() {
        let calc = QuantCalculator::new();

        let result = calc.calculate_iron_condor(100.0, 85.0, 90.0, 110.0, 115.0, 1.0, 0.05, 0.2);
        assert!(result.is_ok());
        let r = result.unwrap();

        assert_eq!(r.name, "Iron Condor");
    }

    #[test]
    fn test_box_spread() {
        let calc = QuantCalculator::new();

        let result = calc.calculate_box_spread(100.0, 95.0, 105.0, 1.0, 0.05, 0.2);
        assert!(result.is_ok());
        let r = result.unwrap();

        assert!(r.synthetic_leg_cost > 0.0);
        assert!(r.legs.len() == 4);
    }

    #[test]
    fn test_jelly_roll() {
        let calc = QuantCalculator::new();

        let result = calc.calculate_jelly_roll(100.0, 100.0, 0.25, 1.0, 0.05, 0.2);
        assert!(result.is_ok());
        let r = result.unwrap();

        assert_eq!(r.name, "Jelly Roll");
        assert!(r.legs.len() == 4);
    }

    #[test]
    fn test_ratio_spread() {
        let calc = QuantCalculator::new();

        let result = calc.calculate_ratio_spread(100.0, 100.0, 100.0, 1.0, 0.05, 0.2, 2);
        assert!(result.is_ok());
        let r = result.unwrap();

        assert_eq!(r.name, "Ratio Spread");
    }

    #[test]
    fn test_invalid_parameters() {
        let calc = QuantCalculator::new();

        let result = calc.calculate_option_price(0.0, 100.0, 1.0, 0.05, 0.2, OptionKind::Call);
        assert!(result.is_err());

        let result = calc.calculate_option_price(100.0, 100.0, 0.0, 0.05, 0.2, OptionKind::Call);
        assert!(result.is_err());

        let result = calc.calculate_option_price(100.0, 100.0, 1.0, 0.05, -0.1, OptionKind::Call);
        assert!(result.is_err());
    }

    #[test]
    fn test_stock_greeks() {
        let calc = QuantCalculator::new();
        let greeks = calc.calculate_stock_greeks(100.0);
        assert_eq!(greeks.delta, 100.0);
        assert_eq!(greeks.gamma, 0.0);
        assert_eq!(greeks.theta, 0.0);
    }

    #[test]
    fn test_bond_greeks() {
        let calc = QuantCalculator::new();
        let greeks = calc.calculate_bond_greeks(100.0, 10.0, 7.5, 70.0);
        assert!(greeks.delta < 0.0);
        assert!(greeks.gamma > 0.0);
        assert_eq!(greeks.vega, 0.0);
    }

    #[test]
    fn test_currency_greeks() {
        let calc = QuantCalculator::new();
        let greeks = calc.calculate_currency_greeks(10000.0);
        assert_eq!(greeks.delta, 10000.0);
    }

    #[test]
    fn test_aggregate_greeks() {
        let calc = QuantCalculator::new();

        let positions = vec![
            Position {
                symbol: "SPY".to_string(),
                quantity: 100.0,
                current_price: 500.0,
                is_option: false,
                strike: None,
                expiry: None,
                option_type: None,
            },
            Position {
                symbol: "SPY".to_string(),
                quantity: 1.0,
                current_price: 5.0,
                is_option: true,
                strike: Some(500.0),
                expiry: Some("20261218".to_string()),
                option_type: Some(OptionKind::Call),
            },
        ];

        let result = calc.aggregate_greeks(&positions, 500.0, 0.05, 0.2, 0.0);
        assert!(result.is_ok());
        let agg = result.unwrap();
        assert!(agg.delta > 100.0);
    }
}
