#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct FfiGreeks {
    pub delta: f64,
    pub gamma: f64,
    pub theta: f64,
    pub vega: f64,
    pub rho: f64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct FfiOptionData {
    pub bid: f64,
    pub ask: f64,
    pub last: f64,
    pub volume: i64,
    pub open_interest: i64,
    pub implied_volatility: f64,
}

#[no_mangle]
pub extern "C" fn calculate_greeks(
    s: f64,
    k: f64,
    t_years: f64,
    r: f64,
    sigma: f64,
    is_call: bool,
) -> FfiGreeks {
    use crate::OptionKind;
    let calc = crate::QuantCalculator::new();
    let option_type = if is_call {
        OptionKind::Call
    } else {
        OptionKind::Put
    };

    match calc.calculate_greeks(s, k, t_years, r, sigma, option_type) {
        Ok(g) => FfiGreeks {
            delta: g.delta,
            gamma: g.gamma,
            theta: g.theta,
            vega: g.vega,
            rho: g.rho,
        },
        Err(_) => FfiGreeks::default(),
    }
}

#[no_mangle]
pub extern "C" fn calculate_implied_volatility(
    market_price: f64,
    s: f64,
    k: f64,
    t_years: f64,
    r: f64,
    is_call: bool,
) -> f64 {
    use crate::OptionKind;
    let calc = crate::QuantCalculator::new();
    let option_type = if is_call {
        OptionKind::Call
    } else {
        OptionKind::Put
    };

    calc.calculate_implied_volatility(market_price, s, k, t_years, r, option_type)
        .unwrap_or(0.0)
}

#[no_mangle]
pub extern "C" fn calculate_option_price(
    s: f64,
    k: f64,
    t_years: f64,
    r: f64,
    sigma: f64,
    is_call: bool,
) -> f64 {
    use crate::OptionKind;
    let calc = crate::QuantCalculator::new();
    let option_type = if is_call {
        OptionKind::Call
    } else {
        OptionKind::Put
    };

    calc.calculate_option_price(s, k, t_years, r, sigma, option_type)
        .unwrap_or(0.0)
}
