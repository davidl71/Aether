use quant::{Greeks, OptionKind, QuantCalculator};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GreeksRequest {
    pub underlying_price: f64,
    pub strike_price: f64,
    pub time_to_expiry: f64,
    pub risk_free_rate: f64,
    pub volatility: f64,
    pub option_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GreeksResponse {
    pub price: f64,
    pub greeks: Greeks,
    pub request: GreeksRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IvRequest {
    pub market_price: f64,
    pub underlying_price: f64,
    pub strike_price: f64,
    pub time_to_expiry: f64,
    pub risk_free_rate: f64,
    pub option_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IvResponse {
    pub implied_volatility: f64,
    pub request: IvRequest,
}

pub fn calculate_greeks(request: &GreeksRequest) -> Result<GreeksResponse, String> {
    let calc = QuantCalculator::new();

    let option_type = match request.option_type.to_lowercase().as_str() {
        "call" => OptionKind::Call,
        "put" => OptionKind::Put,
        _ => return Err("option_type must be 'call' or 'put'".to_string()),
    };

    let price = calc
        .calculate_option_price(
            request.underlying_price,
            request.strike_price,
            request.time_to_expiry,
            request.risk_free_rate,
            request.volatility,
            option_type,
        )
        .map_err(|e| e.to_string())?;

    let greeks = calc
        .calculate_greeks(
            request.underlying_price,
            request.strike_price,
            request.time_to_expiry,
            request.risk_free_rate,
            request.volatility,
            option_type,
        )
        .map_err(|e| e.to_string())?;

    Ok(GreeksResponse {
        price,
        greeks,
        request: request.clone(),
    })
}

pub fn calculate_iv(request: &IvRequest) -> Result<IvResponse, String> {
    let calc = QuantCalculator::new();

    let option_type = match request.option_type.to_lowercase().as_str() {
        "call" => OptionKind::Call,
        "put" => OptionKind::Put,
        _ => return Err("option_type must be 'call' or 'put'".to_string()),
    };

    let iv = calc
        .calculate_implied_volatility(
            request.market_price,
            request.underlying_price,
            request.strike_price,
            request.time_to_expiry,
            request.risk_free_rate,
            option_type,
        )
        .map_err(|e| e.to_string())?;

    Ok(IvResponse {
        implied_volatility: iv,
        request: request.clone(),
    })
}
