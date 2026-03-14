use crate::model::{BoxSpreadLeg, PortfolioRisk, PositionRisk};
use crate::quant::{Greeks, OptionKind, Position, QuantCalculator};

pub struct RiskCalculator;

impl RiskCalculator {
    pub fn new() -> Self {
        Self
    }

    pub fn calculate_box_spread_risk(
        &self,
        spread: &BoxSpreadLeg,
        _underlying_price: f64,
        _implied_volatility: f64,
    ) -> PositionRisk {
        let mut risk = PositionRisk::default();

        risk.position_size = spread.net_debit * 100.0;
        risk.max_loss = spread.net_debit * 100.0;
        risk.max_gain = (spread.get_strike_width() - spread.net_debit) * 100.0;
        risk.expected_value = risk.max_gain;

        risk.delta = 0.0;
        risk.gamma = 0.0;
        risk.theta = 0.0;
        risk.vega = 0.0;

        risk.leverage = 1.0;
        risk.probability_of_profit = 1.0;

        if risk.max_loss > 0.0 {
            risk.risk_reward_ratio = risk.max_gain / risk.max_loss;
        }

        risk
    }

    pub fn calculate_position_risk(
        &self,
        position: &Position,
        underlying_price: f64,
        implied_volatility: f64,
        risk_free_rate: f64,
    ) -> PositionRisk {
        let mut risk = PositionRisk::default();
        risk.position_size = (position.current_price * position.quantity).abs();

        if position.is_option {
            if let (Some(strike), Some(expiry), Some(opt_type)) =
                (position.strike, &position.expiry, position.option_type)
            {
                if let Ok(days) = parse_expiry_days(expiry) {
                    let t_years = days as f64 / 365.0;
                    if let Ok(greeks) = calculate_greeks_internal(
                        underlying_price,
                        strike,
                        t_years,
                        risk_free_rate,
                        implied_volatility,
                        opt_type,
                    ) {
                        risk.delta = greeks.delta * position.quantity;
                        risk.gamma = greeks.gamma * position.quantity;
                        risk.theta = greeks.theta * position.quantity;
                        risk.vega = greeks.vega * position.quantity;
                    }
                }
            }
        } else {
            risk.delta = position.quantity;
        }

        if position.quantity > 0.0 {
            risk.max_loss = position.current_price * position.quantity.abs();
            if let Some(strike) = position.strike {
                let qty: f64 = position.quantity.abs();
                risk.max_gain = (underlying_price - strike).max(0.0) * 100.0 * qty;
            } else {
                risk.max_gain = risk.position_size * 2.0;
            }
        } else {
            risk.max_gain = position.current_price * position.quantity.abs();
            if let Some(strike) = position.strike {
                risk.max_loss = strike * 100.0 * position.quantity.abs();
            }
        }

        risk.max_loss = risk.max_loss.max(0.0);
        risk.max_gain = risk.max_gain.max(0.0);

        risk
    }

    pub fn calculate_portfolio_risk(
        &self,
        positions: &[Position],
        _account_value: f64,
    ) -> PortfolioRisk {
        let mut portfolio_risk = PortfolioRisk::default();

        let total_exposure: f64 = positions
            .iter()
            .map(|p| (p.current_price * p.quantity).abs())
            .sum();
        portfolio_risk.total_exposure = total_exposure;

        portfolio_risk
    }
}

impl Default for RiskCalculator {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_expiry_days(expiry: &str) -> Result<i64, ()> {
    if expiry.len() != 8 {
        return Err(());
    }
    let year: i32 = expiry[0..4].parse().map_err(|_| ())?;
    let month: u32 = expiry[4..6].parse().map_err(|_| ())?;
    let day: u32 = expiry[6..8].parse().map_err(|_| ())?;

    let month: u8 = month.try_into().map_err(|_| ())?;
    let day: u8 = day.try_into().map_err(|_| ())?;

    let month = time::Month::try_from(month).map_err(|_| ())?;

    let today = time::OffsetDateTime::now_utc().date();
    let expiry_date = time::Date::from_calendar_date(year, month, day).map_err(|_| ())?;

    Ok((expiry_date - today).whole_days())
}

fn calculate_greeks_internal(
    s: f64,
    k: f64,
    t_years: f64,
    r: f64,
    sigma: f64,
    option_type: OptionKind,
) -> Result<Greeks, ()> {
    let calc = QuantCalculator::new();
    calc.calculate_greeks(s, k, t_years, r, sigma, option_type)
        .map_err(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_spread_risk() {
        let calc = RiskCalculator::new();
        let spread = BoxSpreadLeg::new(5.0, 10.0);

        let risk = calc.calculate_box_spread_risk(&spread, 100.0, 0.2);

        assert_eq!(risk.position_size, 500.0);
        assert_eq!(risk.max_loss, 500.0);
        assert_eq!(risk.max_gain, 500.0);
        assert_eq!(risk.delta, 0.0);
    }

    #[test]
    fn test_stock_position_risk() {
        let calc = RiskCalculator::new();
        let position = Position::stock("SPY", 100.0, 500.0);

        let risk = calc.calculate_position_risk(&position, 500.0, 0.2, 0.05);

        assert_eq!(risk.position_size, 50000.0);
        assert_eq!(risk.delta, 100.0);
    }
}
