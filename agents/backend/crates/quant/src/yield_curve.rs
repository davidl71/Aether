#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct YieldCurvePoint {
    pub days_to_expiry: i32,
    pub implied_rate: f64,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct YieldCurve {
    pub points: Vec<YieldCurvePoint>,
    pub interpolated_rates: Vec<(f64, f64)>,
}

impl YieldCurve {
    pub fn new(points: Vec<YieldCurvePoint>) -> Self {
        let mut curve = Self {
            points: points.clone(),
            interpolated_rates: Vec::new(),
        };
        curve.interpolate();
        curve
    }

    fn interpolate(&mut self) {
        if self.points.len() < 2 {
            return;
        }

        self.points.sort_by_key(|p| p.days_to_expiry);

        for i in 0..self.points.len() - 1 {
            let x0 = self.points[i].days_to_expiry as f64;
            let x1 = self.points[i + 1].days_to_expiry as f64;
            let y0 = self.points[i].implied_rate;
            let y1 = self.points[i + 1].implied_rate;

            for j in 0..10 {
                let x = x0 + (x1 - x0) * (j as f64 / 10.0);
                let y = linear_interpolate(x, x0, x1, y0, y1);
                self.interpolated_rates.push((x, y));
            }
        }

        if let Some(last) = self.points.last() {
            self.interpolated_rates
                .push((last.days_to_expiry as f64, last.implied_rate));
        }
    }

    pub fn get_rate(&self, days: i32) -> f64 {
        let days_f = days as f64;

        if let Some(first) = self.points.first() {
            if days_f <= first.days_to_expiry as f64 {
                return first.implied_rate;
            }
        }

        if let Some(last) = self.points.last() {
            if days_f >= last.days_to_expiry as f64 {
                return last.implied_rate;
            }
        }

        for i in 0..self.points.len() - 1 {
            let x0 = self.points[i].days_to_expiry as f64;
            let x1 = self.points[i + 1].days_to_expiry as f64;

            if days_f >= x0 && days_f <= x1 {
                return linear_interpolate(
                    days_f,
                    x0,
                    x1,
                    self.points[i].implied_rate,
                    self.points[i + 1].implied_rate,
                );
            }
        }

        0.05
    }

    pub fn get_discount_factor(&self, days: i32, rate: f64) -> f64 {
        let t = days as f64 / 365.0;
        (-rate * t).exp()
    }

    pub fn bootstrap(&self) -> Vec<(i32, f64)> {
        let mut rates = Vec::new();

        for point in &self.points {
            rates.push((point.days_to_expiry, point.implied_rate));
        }

        rates
    }
}

fn linear_interpolate(x: f64, x0: f64, x1: f64, y0: f64, y1: f64) -> f64 {
    if (x1 - x0).abs() < 1e-10 {
        return y0;
    }
    y0 + (y1 - y0) * (x - x0) / (x1 - x0)
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct NelsonSiegelParams {
    pub beta0: f64,
    pub beta1: f64,
    pub beta2: f64,
    pub tau: f64,
}

pub struct NelsonSiegelFitter;

impl NelsonSiegelFitter {
    pub fn new() -> Self {
        Self
    }

    pub fn fit(&self, points: &[YieldCurvePoint]) -> Option<NelsonSiegelParams> {
        if points.len() < 3 {
            return None;
        }

        let avg_rate: f64 =
            points.iter().map(|p| p.implied_rate).sum::<f64>() / points.len() as f64;
        let min_rate = points
            .iter()
            .map(|p| p.implied_rate)
            .fold(f64::INFINITY, f64::min);

        Some(NelsonSiegelParams {
            beta0: avg_rate,
            beta1: 0.0,
            beta2: avg_rate - min_rate,
            tau: 1.0,
        })
    }

    pub fn yield_at(&self, params: &NelsonSiegelParams, maturity: f64) -> f64 {
        let tau = params.tau;
        let lambda = 1.0 / tau;

        let term1 = params.beta0;
        let term2 = params.beta1 * (1.0 - (-maturity * lambda).exp()) / (maturity * lambda);
        let term3 = params.beta2
            * ((1.0 - (-maturity * lambda).exp()) / (maturity * lambda)
                - (-maturity * lambda).exp());

        term1 + term2 + term3
    }
}

impl Default for NelsonSiegelFitter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yield_curve() {
        let points = vec![
            YieldCurvePoint {
                days_to_expiry: 30,
                implied_rate: 0.05,
            },
            YieldCurvePoint {
                days_to_expiry: 90,
                implied_rate: 0.052,
            },
            YieldCurvePoint {
                days_to_expiry: 180,
                implied_rate: 0.055,
            },
            YieldCurvePoint {
                days_to_expiry: 365,
                implied_rate: 0.06,
            },
        ];

        let curve = YieldCurve::new(points);
        assert!(curve.get_rate(60) > 0.0);
    }

    #[test]
    fn test_nelson_siegel() {
        let fitter = NelsonSiegelFitter::new();
        let points = vec![
            YieldCurvePoint {
                days_to_expiry: 30,
                implied_rate: 0.05,
            },
            YieldCurvePoint {
                days_to_expiry: 90,
                implied_rate: 0.052,
            },
            YieldCurvePoint {
                days_to_expiry: 180,
                implied_rate: 0.055,
            },
        ];

        let params = fitter.fit(&points);
        assert!(params.is_some());
    }

    #[test]
    fn test_discount_factor() {
        let points = vec![YieldCurvePoint {
            days_to_expiry: 365,
            implied_rate: 0.05,
        }];
        let curve = YieldCurve::new(points);
        let df = curve.get_discount_factor(365, 0.05);
        assert!((df - 0.9512).abs() < 0.01);
    }
}
