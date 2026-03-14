use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MarginResult {
    pub initial_margin: f64,
    pub maintenance_margin: f64,
    pub reg_t_margin: f64,
    pub span_margin: f64,
    pub uses_portfolio_margin: bool,
    pub portfolio_margin_benefit: f64,
    pub margin_call_risk: bool,
    pub margin_utilization: f64,
    pub calculated_at: i64,
}

impl MarginResult {
    pub fn get_effective_margin(&self) -> f64 {
        if self.uses_portfolio_margin && self.span_margin > 0.0 {
            return self.span_margin;
        }
        if self.initial_margin > 0.0 {
            return self.initial_margin;
        }
        if self.reg_t_margin > 0.0 {
            return self.reg_t_margin;
        }
        0.0
    }

    pub fn is_valid(&self) -> bool {
        self.initial_margin >= 0.0 && self.maintenance_margin >= 0.0
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BoxSpreadMarginInput {
    pub short_call_strike: f64,
    pub short_call_price: f64,
    pub short_put_strike: f64,
    pub short_put_price: f64,
    pub long_call_price: f64,
    pub long_put_price: f64,
    pub net_debit: f64,
    pub days_to_expiry: i32,
}

pub struct MarginCalculator;

impl MarginCalculator {
    pub fn new() -> Self {
        Self
    }

    pub fn calculate_reg_t_margin(
        &self,
        spread: &BoxSpreadMarginInput,
        underlying_price: f64,
        _implied_volatility: f64,
    ) -> MarginResult {
        let mut result = MarginResult::default();
        result.calculated_at = OffsetDateTime::now_utc().unix_timestamp();
        result.uses_portfolio_margin = false;

        let short_call_margin = self.calculate_short_option_margin(
            underlying_price,
            spread.short_call_strike,
            true,
            spread.short_call_price,
            spread.days_to_expiry,
        );

        let short_put_margin = self.calculate_short_option_margin(
            underlying_price,
            spread.short_put_strike,
            false,
            spread.short_put_price,
            spread.days_to_expiry,
        );

        let long_call_premium = spread.long_call_price * 100.0;
        let long_put_premium = spread.long_put_price * 100.0;

        let net_margin =
            short_call_margin + short_put_margin - long_call_premium - long_put_premium;

        result.reg_t_margin = net_margin.max(spread.net_debit * 100.0);
        result.initial_margin = result.reg_t_margin;
        result.maintenance_margin = result.initial_margin * 0.75;

        result
    }

    pub fn calculate_portfolio_margin(
        &self,
        spread: &BoxSpreadMarginInput,
        underlying_price: f64,
        implied_volatility: f64,
        portfolio_multiplier: f64,
    ) -> MarginResult {
        let mut result = MarginResult::default();
        result.calculated_at = OffsetDateTime::now_utc().unix_timestamp();
        result.uses_portfolio_margin = true;

        let span_result = self.calculate_span_margin(spread, underlying_price, implied_volatility);
        result.span_margin = span_result.span_margin;

        let reg_t_result =
            self.calculate_reg_t_margin(spread, underlying_price, implied_volatility);
        result.reg_t_margin = reg_t_result.reg_t_margin;

        result.initial_margin = result
            .span_margin
            .max(reg_t_result.initial_margin * portfolio_multiplier);
        result.initial_margin = result.initial_margin.max(spread.net_debit * 100.0);

        result.maintenance_margin = result.initial_margin * 0.75;
        result.portfolio_margin_benefit = reg_t_result.initial_margin - result.initial_margin;

        result
    }

    pub fn calculate_span_margin(
        &self,
        spread: &BoxSpreadMarginInput,
        underlying_price: f64,
        _implied_volatility: f64,
    ) -> MarginResult {
        let mut result = MarginResult::default();
        result.calculated_at = OffsetDateTime::now_utc().unix_timestamp();
        result.uses_portfolio_margin = true;

        let scenarios = self.calculate_span_scenarios(underlying_price, spread.days_to_expiry);
        let mut max_loss: f64 = 0.0;

        for _scenario_price in scenarios {
            let scenario_loss = spread.net_debit * 100.0;
            max_loss = max_loss.max(scenario_loss);
        }

        result.span_margin = max_loss.max(spread.net_debit * 100.0);
        result.initial_margin = result.span_margin;
        result.maintenance_margin = result.initial_margin * 0.75;

        result
    }

    pub fn calculate_margin(
        &self,
        spread: &BoxSpreadMarginInput,
        underlying_price: f64,
        implied_volatility: f64,
        prefer_portfolio_margin: bool,
    ) -> MarginResult {
        if prefer_portfolio_margin {
            self.calculate_portfolio_margin(spread, underlying_price, implied_volatility, 0.25)
        } else {
            self.calculate_reg_t_margin(spread, underlying_price, implied_volatility)
        }
    }

    pub fn calculate_portfolio_margin_benefit(
        &self,
        spreads: &[BoxSpreadMarginInput],
        underlying_price: f64,
    ) -> f64 {
        if spreads.is_empty() {
            return 0.0;
        }

        let mut total_reg_t = 0.0;
        for spread in spreads {
            let reg_t = self.calculate_reg_t_margin(spread, underlying_price, 0.0);
            total_reg_t += reg_t.initial_margin;
        }

        let portfolio = self.calculate_portfolio_margin(&spreads[0], underlying_price, 0.20, 0.25);
        let total_portfolio = portfolio.initial_margin * spreads.len() as f64;

        (total_reg_t - total_portfolio).max(0.0)
    }

    pub fn is_margin_call_risk(
        &self,
        current_margin: f64,
        maintenance_margin: f64,
        account_value: f64,
        buffer_percent: f64,
    ) -> bool {
        let maintenance_threshold = maintenance_margin * (1.0 + buffer_percent / 100.0);
        let available_margin = account_value - current_margin;

        current_margin >= maintenance_threshold || available_margin < maintenance_margin
    }

    pub fn calculate_margin_utilization(&self, margin_used: f64, available_margin: f64) -> f64 {
        if available_margin <= 0.0 {
            return 100.0;
        }

        let total_margin = margin_used + available_margin;
        if total_margin <= 0.0 {
            return 0.0;
        }

        (margin_used / total_margin) * 100.0
    }

    pub fn calculate_remaining_margin_capacity(
        &self,
        account_value: f64,
        _initial_margin_used: f64,
        maintenance_margin_used: f64,
    ) -> f64 {
        (account_value - maintenance_margin_used).max(0.0)
    }

    fn calculate_short_option_margin(
        &self,
        underlying_price: f64,
        strike: f64,
        is_call: bool,
        option_price: f64,
        _days_to_expiry: i32,
    ) -> f64 {
        let premium = option_price * 100.0;
        let otm_amount = if is_call {
            (strike - underlying_price).max(0.0)
        } else {
            (underlying_price - strike).max(0.0)
        };

        let margin = if is_call {
            (0.20 * underlying_price - otm_amount).max(0.10 * underlying_price) * 100.0
        } else {
            (0.20 * underlying_price - otm_amount).max(0.10 * strike) * 100.0
        };

        margin + premium
    }

    fn calculate_span_scenarios(&self, underlying_price: f64, _days_to_expiry: i32) -> Vec<f64> {
        vec![
            underlying_price * 0.70,
            underlying_price * 0.85,
            underlying_price * 0.95,
            underlying_price,
            underlying_price * 1.05,
            underlying_price * 1.15,
            underlying_price * 1.30,
        ]
    }
}

impl Default for MarginCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn box_spread() -> BoxSpreadMarginInput {
        BoxSpreadMarginInput {
            short_call_strike: 100.0,
            short_call_price: 3.0,
            short_put_strike: 95.0,
            short_put_price: 2.5,
            long_call_price: 1.0,
            long_put_price: 0.5,
            net_debit: 4.0,
            days_to_expiry: 30,
        }
    }

    #[test]
    fn test_reg_t_margin() {
        let calc = MarginCalculator::new();
        let result = calc.calculate_reg_t_margin(&box_spread(), 100.0, 0.2);
        assert!(result.initial_margin > 0.0);
    }

    #[test]
    fn test_margin_call_risk() {
        let calc = MarginCalculator::new();
        let result = calc.is_margin_call_risk(8000.0, 10000.0, 50000.0, 10.0);
        assert!(!result);
    }

    #[test]
    fn test_margin_utilization() {
        let calc = MarginCalculator::new();
        let util = calc.calculate_margin_utilization(3000.0, 7000.0);
        assert!((util - 30.0).abs() < 0.01);
    }
}
