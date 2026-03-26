//! Portfolio summary calculations for dashboard and reporting.

use std::collections::HashMap;

/// Portfolio-level summary statistics.
#[derive(Debug, Clone, Default)]
pub struct PortfolioSummary {
    /// Total portfolio value (sum of all position market values)
    pub total_value: f64,
    /// Total cost basis (sum of all position cost bases)
    pub total_cost: f64,
    /// Total unrealized PnL (total_value - total_cost)
    pub total_pnl: f64,
    /// PnL as percentage of cost basis
    pub pnl_percent: f64,
    /// Number of positions
    pub position_count: usize,
    /// Per-strategy breakdown
    pub by_strategy: HashMap<String, StrategySummary>,
}

/// Summary statistics for a single strategy.
#[derive(Debug, Clone, Default)]
pub struct StrategySummary {
    /// Strategy name
    pub name: String,
    /// Total value for this strategy
    pub value: f64,
    /// Total cost basis for this strategy
    pub cost: f64,
    /// Unrealized PnL for this strategy
    pub pnl: f64,
    /// PnL percentage
    pub pnl_percent: f64,
    /// Number of positions
    pub position_count: usize,
}

impl PortfolioSummary {
    /// Calculate summary from positions.
    pub fn from_positions(positions: &[api::RuntimePositionDto]) -> Self {
        let mut summary = Self::default();
        summary.position_count = positions.len();

        for pos in positions {
            let value = pos.mark * pos.quantity as f64;
            let cost = pos.cost_basis;
            let pnl = pos.unrealized_pnl;

            summary.total_value += value;
            summary.total_cost += cost;
            summary.total_pnl += pnl;

            // Track per-strategy
            let strategy = pos.strategy.clone().unwrap_or_else(|| "None".to_string());
            let strat_summary = summary
                .by_strategy
                .entry(strategy.clone())
                .or_insert_with(|| StrategySummary {
                    name: strategy,
                    ..Default::default()
                });

            strat_summary.value += value;
            strat_summary.cost += cost;
            strat_summary.pnl += pnl;
            strat_summary.position_count += 1;
        }

        // Calculate percentages
        if summary.total_cost > 0.0 {
            summary.pnl_percent = (summary.total_pnl / summary.total_cost) * 100.0;
        }

        for strat in summary.by_strategy.values_mut() {
            if strat.cost > 0.0 {
                strat.pnl_percent = (strat.pnl / strat.cost) * 100.0;
            }
        }

        summary
    }

    /// Format total value for display.
    pub fn format_value(&self) -> String {
        format!("${:.2}", self.total_value)
    }

    /// Format total PnL with sign.
    pub fn format_pnl(&self) -> String {
        format!("{:+.2}", self.total_pnl)
    }

    /// Format PnL percentage with sign.
    pub fn format_pnl_percent(&self) -> String {
        format!("{:+.2}%", self.pnl_percent)
    }

    /// Get color for PnL (green for positive, red for negative).
    pub fn pnl_color(&self) -> ratatui::style::Color {
        if self.total_pnl >= 0.0 {
            ratatui::style::Color::Green
        } else {
            ratatui::style::Color::Red
        }
    }
}

impl StrategySummary {
    /// Format value for display.
    pub fn format_value(&self) -> String {
        format!("${:.2}", self.value)
    }

    /// Format PnL with sign.
    pub fn format_pnl(&self) -> String {
        format!("{:+.2}", self.pnl)
    }

    /// Format PnL percentage with sign.
    pub fn format_pnl_percent(&self) -> String {
        format!("{:+.2}%", self.pnl_percent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_portfolio_summary_empty() {
        let summary = PortfolioSummary::from_positions(&[]);
        assert_eq!(summary.total_value, 0.0);
        assert_eq!(summary.total_pnl, 0.0);
        assert_eq!(summary.position_count, 0);
    }

    #[test]
    fn test_portfolio_summary_calculations() {
        let positions = vec![
            api::RuntimePositionDto {
                symbol: "AAPL".to_string(),
                quantity: 100,
                cost_basis: 15000.0,
                mark: 160.0,
                unrealized_pnl: 1000.0,
                strategy: Some("Long".to_string()),
                ..Default::default()
            },
            api::RuntimePositionDto {
                symbol: "TSLA".to_string(),
                quantity: 50,
                cost_basis: 10000.0,
                mark: 180.0,
                unrealized_pnl: -1000.0,
                strategy: Some("Long".to_string()),
                ..Default::default()
            },
        ];

        let summary = PortfolioSummary::from_positions(&positions);
        assert_eq!(summary.total_value, 16000.0 + 9000.0); // AAPL value + TSLA value
        assert_eq!(summary.total_cost, 25000.0);
        assert_eq!(summary.total_pnl, 0.0); // +1000 -1000
        assert_eq!(summary.position_count, 2);
    }
}
