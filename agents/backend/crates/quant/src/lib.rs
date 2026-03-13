use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::Date;
use RustQuant::instruments::options::{implied_volatility, BlackScholesMerton, TypeFlag};
use RustQuant::time::today;

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

#[derive(Debug, Error)]
pub enum QuantError {
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    #[error("Calculation failed: {0}")]
    CalculationFailed(String),
    #[error("Implied volatility not found")]
    ImpliedVolatilityNotFound,
}

pub struct QuantCalculator;

impl QuantCalculator {
    pub fn new() -> Self {
        Self
    }

    fn days_to_expiry(&self, days: i64) -> Date {
        today() + time::Duration::days(days)
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

        let bsm = BlackScholesMerton::new(
            r,      // cost_of_carry
            s,      // underlying_price
            k,      // strike_price
            sigma,  // volatility
            r,      // risk_free_rate
            None,   // evaluation_date
            expiry, // expiration_date
            option_type.into(),
        );

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

        let bsm = BlackScholesMerton::new(
            r,      // cost_of_carry
            s,      // underlying_price
            k,      // strike_price
            sigma,  // volatility
            r,      // risk_free_rate
            None,   // evaluation_date
            expiry, // expiration_date
            option_type.into(),
        );

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
    fn test_invalid_parameters() {
        let calc = QuantCalculator::new();

        let result = calc.calculate_option_price(0.0, 100.0, 1.0, 0.05, 0.2, OptionKind::Call);
        assert!(result.is_err());

        let result = calc.calculate_option_price(100.0, 100.0, 0.0, 0.05, 0.2, OptionKind::Call);
        assert!(result.is_err());

        let result = calc.calculate_option_price(100.0, 100.0, 1.0, 0.05, -0.1, OptionKind::Call);
        assert!(result.is_err());
    }
}
