use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BondData {
    pub duration: f64,
    pub convexity: f64,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct OptimizationResult {
    pub short_term_weight: f64,
    pub long_term_weight: f64,
    pub portfolio_convexity: f64,
    pub portfolio_duration: f64,
    pub success: bool,
    pub error_message: String,
}

pub struct ConvexityCalculator;

impl ConvexityCalculator {
    pub fn new() -> Self {
        Self
    }

    pub fn calculate_portfolio_convexity(
        &self,
        short_term_weight: f64,
        short_term_convexity: f64,
        long_term_weight: f64,
        long_term_convexity_value: f64,
    ) -> f64 {
        short_term_weight * short_term_convexity + long_term_weight * long_term_convexity_value
    }

    pub fn calculate_current_convexity(
        &self,
        short_term_weight: f64,
        short_term_bond: &BondData,
        long_term_weight: f64,
        long_term_bond: &BondData,
    ) -> f64 {
        self.calculate_portfolio_convexity(
            short_term_weight,
            short_term_bond.convexity,
            long_term_weight,
            long_term_bond.convexity,
        )
    }

    pub fn calculate_weighted_duration(
        &self,
        short_weight: f64,
        short_duration: f64,
        long_weight: f64,
        long_duration: f64,
    ) -> f64 {
        short_weight * short_duration + long_weight * long_duration
    }

    pub fn should_rebalance(
        &self,
        current_convexity: f64,
        target_convexity: f64,
        threshold_percent: f64,
    ) -> bool {
        if target_convexity == 0.0 {
            return false;
        }

        let deviation_percent =
            ((current_convexity - target_convexity) / target_convexity).abs() * 100.0;
        deviation_percent >= threshold_percent
    }

    pub fn optimize_barbell_allocation(
        &self,
        short_term_bond: &BondData,
        long_term_bond: &BondData,
        target_duration: f64,
        target_convexity: f64,
    ) -> OptimizationResult {
        let mut result = OptimizationResult::default();
        result.success = false;

        let short_weight = (target_duration - long_term_bond.duration)
            / (short_term_bond.duration - long_term_bond.duration);
        let long_weight = 1.0 - short_weight;

        if short_weight >= 0.0 && short_weight <= 1.0 && long_weight >= 0.0 && long_weight <= 1.0 {
            result.short_term_weight = short_weight;
            result.long_term_weight = long_weight;
            result.portfolio_convexity = self.calculate_portfolio_convexity(
                short_weight,
                short_term_bond.convexity,
                long_weight,
                long_term_bond.convexity,
            );
            result.portfolio_duration = self.calculate_weighted_duration(
                short_weight,
                short_term_bond.duration,
                long_weight,
                long_term_bond.duration,
            );
            result.success = true;
        } else {
            result.error_message = "Target duration outside achievable range".to_string();
            result.short_term_weight = 0.5;
            result.long_term_weight = 0.5;
            result.portfolio_convexity = self.calculate_portfolio_convexity(
                0.5,
                short_term_bond.convexity,
                0.5,
                long_term_bond.convexity,
            );
            result.portfolio_duration = self.calculate_weighted_duration(
                0.5,
                short_term_bond.duration,
                0.5,
                long_term_bond.duration,
            );
        }

        result
    }
}

impl Default for ConvexityCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn short_bond() -> BondData {
        BondData {
            duration: 2.0,
            convexity: 5.0,
            name: "SHY".to_string(),
        }
    }

    fn long_bond() -> BondData {
        BondData {
            duration: 20.0,
            convexity: 150.0,
            name: "TLT".to_string(),
        }
    }

    #[test]
    fn test_portfolio_convexity() {
        let calc = ConvexityCalculator::new();
        let cv = calc.calculate_portfolio_convexity(0.5, 5.0, 0.5, 150.0);
        assert!((cv - 77.5).abs() < 0.01);
    }

    #[test]
    fn test_weighted_duration() {
        let calc = ConvexityCalculator::new();
        let d = calc.calculate_weighted_duration(0.5, 2.0, 0.5, 20.0);
        assert!((d - 11.0).abs() < 0.01);
    }

    #[test]
    fn test_should_rebalance() {
        let calc = ConvexityCalculator::new();
        assert!(calc.should_rebalance(160.0, 150.0, 5.0));
        assert!(!calc.should_rebalance(155.0, 150.0, 5.0));
    }

    #[test]
    fn test_optimize_barbell() {
        let calc = ConvexityCalculator::new();
        let result = calc.optimize_barbell_allocation(&short_bond(), &long_bond(), 10.0, 150.0);
        assert!(result.success);
        assert!((result.short_term_weight - 0.5556).abs() < 0.01);
    }
}
