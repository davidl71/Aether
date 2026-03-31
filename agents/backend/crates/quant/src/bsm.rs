//! Minimal Black–Scholes–Merton option pricing and Greeks (no RustQuant/polars).
//! Standard formulas with inline normal CDF approximation.

use super::{OptionKind, QuantError};

/// Standard normal CDF Φ(x). Abramowitz & Stegun 26.2.17 approximation.
fn norm_cdf(x: f64) -> f64 {
    const A: [f64; 5] = [
        0.31938153,
        -0.356563782,
        1.781477937,
        -1.821255978,
        1.330274429,
    ];
    let k = 1.0 / (1.0 + 0.2316419 * x.abs());
    let v = A[0] + k * (A[1] + k * (A[2] + k * (A[3] + k * A[4])));
    let p = 1.0 - (1.0 / (2.0 * std::f64::consts::PI).sqrt()) * (-x * x / 2.0).exp() * v * k;
    if x >= 0.0 {
        p
    } else {
        1.0 - norm_cdf(-x)
    }
}

/// Standard normal PDF φ(x).
fn norm_pdf(x: f64) -> f64 {
    (-x * x / 2.0).exp() / (2.0 * std::f64::consts::PI).sqrt()
}

/// Black–Scholes–Merton price for European option.
/// s: spot, k: strike, t: time to expiry (years), r: risk-free rate, sigma: vol, option_type: Call/Put.
pub fn bsm_price(s: f64, k: f64, t: f64, r: f64, sigma: f64, option_type: OptionKind) -> f64 {
    if t <= 0.0 {
        return (s - k).max(0.0)
            * if matches!(option_type, OptionKind::Call) {
                1.0
            } else {
                -1.0
            }
            + k
            - s * if matches!(option_type, OptionKind::Call) {
                0.0
            } else {
                1.0
            };
    }
    let sqrt_t = t.sqrt();
    let d1 = (s.ln() - k.ln() + (r + 0.5 * sigma * sigma) * t) / (sigma * sqrt_t);
    let d2 = d1 - sigma * sqrt_t;
    let df = (-r * t).exp();
    match option_type {
        OptionKind::Call => s * norm_cdf(d1) - k * df * norm_cdf(d2),
        OptionKind::Put => k * df * norm_cdf(-d2) - s * norm_cdf(-d1),
    }
}

/// BSM delta (per 1 unit spot).
pub fn bsm_delta(s: f64, k: f64, t: f64, r: f64, sigma: f64, option_type: OptionKind) -> f64 {
    if t <= 0.0 {
        return match option_type {
            OptionKind::Call => {
                if s > k {
                    1.0
                } else {
                    0.5
                }
            }
            OptionKind::Put => {
                if s < k {
                    -1.0
                } else {
                    -0.5
                }
            }
        };
    }
    let sqrt_t = t.sqrt();
    let d1 = (s.ln() - k.ln() + (r + 0.5 * sigma * sigma) * t) / (sigma * sqrt_t);
    match option_type {
        OptionKind::Call => norm_cdf(d1),
        OptionKind::Put => norm_cdf(d1) - 1.0,
    }
}

/// BSM gamma.
pub fn bsm_gamma(s: f64, k: f64, t: f64, r: f64, sigma: f64) -> f64 {
    if t <= 0.0 || sigma <= 0.0 {
        return 0.0;
    }
    let sqrt_t = t.sqrt();
    let d1 = (s.ln() - k.ln() + (r + 0.5 * sigma * sigma) * t) / (sigma * sqrt_t);
    norm_pdf(d1) / (s * sigma * sqrt_t)
}

/// BSM theta (per calendar day; divide by 365 for per-year).
pub fn bsm_theta(s: f64, k: f64, t: f64, r: f64, sigma: f64, option_type: OptionKind) -> f64 {
    if t <= 0.0 {
        return 0.0;
    }
    let sqrt_t = t.sqrt();
    let d1 = (s.ln() - k.ln() + (r + 0.5 * sigma * sigma) * t) / (sigma * sqrt_t);
    let d2 = d1 - sigma * sqrt_t;
    let df = (-r * t).exp();
    let term1 = -s * norm_pdf(d1) * sigma / (2.0 * sqrt_t);
    match option_type {
        OptionKind::Call => (term1 - r * k * df * norm_cdf(d2) + r * s * norm_cdf(d1)) / 365.0,
        OptionKind::Put => (term1 + r * k * df * norm_cdf(-d2) - r * s * norm_cdf(-d1)) / 365.0,
    }
}

/// BSM vega (per 1% move in vol).
pub fn bsm_vega(s: f64, k: f64, t: f64, r: f64, sigma: f64) -> f64 {
    if t <= 0.0 {
        return 0.0;
    }
    let sqrt_t = t.sqrt();
    let d1 = (s.ln() - k.ln() + (r + 0.5 * sigma * sigma) * t) / (sigma * sqrt_t);
    s * norm_pdf(d1) * sqrt_t / 100.0
}

/// BSM rho (per 1% move in rate).
pub fn bsm_rho(s: f64, k: f64, t: f64, r: f64, sigma: f64, option_type: OptionKind) -> f64 {
    if t <= 0.0 {
        return 0.0;
    }
    let sqrt_t = t.sqrt();
    let d2 = (s.ln() - k.ln() + (r - 0.5 * sigma * sigma) * t) / (sigma * sqrt_t);
    let df = (-r * t).exp();
    match option_type {
        OptionKind::Call => -k * t * df * norm_cdf(d2) / 100.0,
        OptionKind::Put => k * t * df * norm_cdf(-d2) / 100.0,
    }
}

/// Implied volatility by bisection (target price, s, k, t, r, option_type).
pub fn implied_volatility(
    market_price: f64,
    s: f64,
    k: f64,
    t: f64,
    r: f64,
    option_type: OptionKind,
) -> Result<f64, QuantError> {
    if market_price <= 0.0 || s <= 0.0 || k <= 0.0 || t <= 0.0 {
        return Err(QuantError::InvalidParameter(
            "All of market_price, s, k, t must be positive".to_string(),
        ));
    }
    let mut lo = 1e-6;
    let mut hi = 5.0;
    for _ in 0..100 {
        let mid = (lo + hi) / 2.0;
        let p = bsm_price(s, k, t, r, mid, option_type);
        if (p - market_price).abs() < 1e-10 {
            return Ok(mid);
        }
        if p > market_price {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    let sigma = (lo + hi) / 2.0;
    if (bsm_price(s, k, t, r, sigma, option_type) - market_price).abs() < 1e-6 {
        Ok(sigma)
    } else {
        Err(QuantError::ImpliedVolatilityNotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bsm_call_price() {
        let p = bsm_price(100.0, 100.0, 1.0, 0.05, 0.2, OptionKind::Call);
        assert!((p - 10.45).abs() < 0.1);
    }

    #[test]
    fn test_bsm_put_price() {
        let p = bsm_price(100.0, 100.0, 1.0, 0.05, 0.2, OptionKind::Put);
        assert!((p - 5.57).abs() < 0.1);
    }

    #[test]
    fn test_implied_vol() {
        let s = 100.0;
        let k = 100.0;
        let t = 1.0;
        let r = 0.05;
        let sigma = 0.2;
        let call_price = bsm_price(s, k, t, r, sigma, OptionKind::Call);
        let iv = implied_volatility(call_price, s, k, t, r, OptionKind::Call).unwrap();
        assert!((iv - sigma).abs() < 0.01);
    }
}

