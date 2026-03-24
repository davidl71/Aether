#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct CashFlow {
    pub period: i32,
    pub date: String,
    pub payment: f64,
    pub principal: f64,
    pub interest: f64,
    pub balance: f64,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct AmortizationSchedule {
    pub face_amount: f64,
    pub coupon_rate: f64,
    pub maturity_years: f64,
    pub frequency: i32,
    pub cash_flows: Vec<CashFlow>,
}

impl AmortizationSchedule {
    pub fn new(face_amount: f64, coupon_rate: f64, maturity_years: f64, frequency: i32) -> Self {
        let mut schedule = Self {
            face_amount,
            coupon_rate,
            maturity_years,
            frequency,
            cash_flows: Vec::new(),
        };
        schedule.generate();
        schedule
    }

    pub fn generate(&mut self) {
        let periods = (self.maturity_years * self.frequency as f64) as i32;
        let periodic_rate = self.coupon_rate / self.frequency as f64;
        let periodic_payment = self.calculate_payment(periodic_rate, periods);

        for i in 1..=periods {
            let interest = self.face_amount * periodic_rate;
            let principal = periodic_payment - interest;
            let balance = self.face_amount - principal;

            self.cash_flows.push(CashFlow {
                period: i,
                date: format!("day_{}", (i as f64 * 365.0 / self.frequency as f64) as i32),
                payment: periodic_payment,
                principal,
                interest,
                balance: balance.max(0.0),
            });

            self.face_amount = balance.max(0.0);
        }
    }

    fn calculate_payment(&self, periodic_rate: f64, periods: i32) -> f64 {
        if periodic_rate.abs() < 1e-10 {
            return self.face_amount / periods as f64;
        }

        let factor = (1.0 + periodic_rate).powi(periods);
        self.face_amount * (periodic_rate * factor) / (factor - 1.0)
    }

    pub fn total_interest(&self) -> f64 {
        self.cash_flows.iter().map(|cf| cf.interest).sum()
    }

    pub fn total_payments(&self) -> f64 {
        self.cash_flows.iter().map(|cf| cf.payment).sum()
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct BondPriceResult {
    pub price: f64,
    pub duration: f64,
    pub convexity: f64,
}

pub struct BondCalculator;

impl BondCalculator {
    pub fn new() -> Self {
        Self
    }

    pub fn calculate_price(
        &self,
        face_value: f64,
        coupon_rate: f64,
        years_to_maturity: f64,
        yield_rate: f64,
        frequency: i32,
    ) -> f64 {
        let periods = years_to_maturity * frequency as f64;
        let periodic_coupon = coupon_rate / frequency as f64;
        let periodic_yield = yield_rate / frequency as f64;

        let mut price = 0.0;

        for t in 1..=periods as i32 {
            let discount = (1.0 + periodic_yield).powi(-t);
            price += periodic_coupon * face_value * discount;
        }

        price += face_value * (1.0 + periodic_yield).powi(-periods as i32);

        price
    }

    pub fn calculate_yield(
        &self,
        face_value: f64,
        coupon_rate: f64,
        years_to_maturity: f64,
        price: f64,
        frequency: i32,
    ) -> f64 {
        let mut rate = 0.05;

        for _ in 0..100 {
            let estimated_price =
                self.calculate_price(face_value, coupon_rate, years_to_maturity, rate, frequency);
            let diff = price - estimated_price;

            if diff.abs() < 0.01 {
                break;
            }

            rate += diff / 10000.0;
        }

        rate.clamp(0.0, 1.0)
    }

    pub fn calculate_duration(
        &self,
        face_value: f64,
        coupon_rate: f64,
        years_to_maturity: f64,
        yield_rate: f64,
        frequency: i32,
    ) -> f64 {
        let periods = years_to_maturity * frequency as f64;
        let periodic_coupon = coupon_rate / frequency as f64;
        let periodic_yield = yield_rate / frequency as f64;

        let price = self.calculate_price(
            face_value,
            coupon_rate,
            years_to_maturity,
            yield_rate,
            frequency,
        );

        let mut weighted_sum = 0.0;

        for t in 1..=periods as i32 {
            let discount = (1.0 + periodic_yield).powi(-t);
            let pv = if t == periods as i32 {
                (periodic_coupon * face_value + face_value) * discount
            } else {
                periodic_coupon * face_value * discount
            };

            weighted_sum += (t as f64 / frequency as f64) * pv;
        }

        weighted_sum / price
    }

    pub fn calculate_convexity(
        &self,
        face_value: f64,
        coupon_rate: f64,
        years_to_maturity: f64,
        yield_rate: f64,
        frequency: i32,
    ) -> f64 {
        let periods = years_to_maturity * frequency as f64;
        let periodic_coupon = coupon_rate / frequency as f64;
        let periodic_yield = yield_rate / frequency as f64;

        let price = self.calculate_price(
            face_value,
            coupon_rate,
            years_to_maturity,
            yield_rate,
            frequency,
        );

        let mut convexity_sum = 0.0;

        for t in 1..=periods as i32 {
            let t_years = t as f64 / frequency as f64;
            let discount = (1.0 + periodic_yield).powi(-t);
            let pv = if t == periods as i32 {
                (periodic_coupon * face_value + face_value) * discount
            } else {
                periodic_coupon * face_value * discount
            };

            convexity_sum += t_years * (t_years + 1.0) * pv;
        }

        convexity_sum / (price * (1.0 + periodic_yield).powi(2))
    }
}

impl Default for BondCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amortization_schedule() {
        let schedule = AmortizationSchedule::new(10000.0, 0.05, 1.0, 2);
        assert!(!schedule.cash_flows.is_empty());
        assert!(schedule.total_interest() > 0.0);
    }

    #[test]
    fn test_bond_price() {
        let calc = BondCalculator::new();
        let price = calc.calculate_price(100.0, 0.05, 1.0, 0.05, 2);
        assert!((price - 100.0).abs() < 1.0);
    }

    #[test]
    fn test_bond_duration() {
        let calc = BondCalculator::new();
        let duration = calc.calculate_duration(100.0, 0.05, 1.0, 0.05, 2);
        assert!((duration - 0.99).abs() < 0.1);
    }
}
