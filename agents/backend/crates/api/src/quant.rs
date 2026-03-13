use quant::{
    Greeks, HistoricalVolatilityResult, OptionKind, QuantCalculator, RiskMetrics, StrategyResult,
};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalVolRequest {
    pub prices: Vec<f64>,
    pub annualization_factor: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalVolResponse {
    pub hv: f64,
    pub sample_std_dev: f64,
    pub variance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMetricsRequest {
    pub returns: Vec<f64>,
    pub confidence: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMetricsResponse {
    pub var_95: f64,
    pub cvar_95: f64,
    pub max_loss: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyRequest {
    pub underlying_price: f64,
    pub strike_price: f64,
    pub time_to_expiry: f64,
    pub risk_free_rate: f64,
    pub volatility: f64,
    pub strategy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyResponse {
    pub strategy: StrategyResult,
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

pub fn calculate_historical_volatility(
    request: &HistoricalVolRequest,
) -> Result<HistoricalVolResponse, String> {
    let calc = QuantCalculator::new();
    let annualization = request.annualization_factor.unwrap_or(252.0_f64.sqrt());

    let result = calc
        .calculate_historical_volatility(&request.prices, annualization)
        .map_err(|e| e.to_string())?;

    Ok(HistoricalVolResponse {
        hv: result.hv,
        sample_std_dev: result.sample_std_dev,
        variance: result.variance,
    })
}

pub fn calculate_risk_metrics(request: &RiskMetricsRequest) -> Result<RiskMetricsResponse, String> {
    let calc = QuantCalculator::new();
    let confidence = request.confidence.unwrap_or(0.95);

    let result = calc
        .calculate_var_cvar(&request.returns, confidence)
        .map_err(|e| e.to_string())?;

    Ok(RiskMetricsResponse {
        var_95: result.var_95,
        cvar_95: result.cvar_95,
        max_loss: result.max_loss,
    })
}

pub fn calculate_strategy(request: &StrategyRequest) -> Result<StrategyResponse, String> {
    let calc = QuantCalculator::new();

    let result = match request.strategy.to_lowercase().as_str() {
        "straddle" => calc.calculate_straddle(
            request.underlying_price,
            request.strike_price,
            request.time_to_expiry,
            request.risk_free_rate,
            request.volatility,
        ),
        "strangle" => calc.calculate_strangle(
            request.underlying_price,
            request.strike_price,
            request.strike_price * 0.9,
            request.time_to_expiry,
            request.risk_free_rate,
            request.volatility,
        ),
        "butterfly" => calc.calculate_butterfly_spread(
            request.underlying_price,
            request.strike_price * 0.9,
            request.strike_price,
            request.strike_price * 1.1,
            request.time_to_expiry,
            request.risk_free_rate,
            request.volatility,
        ),
        "iron_condor" => calc.calculate_iron_condor(
            request.underlying_price,
            request.strike_price * 0.85,
            request.strike_price * 0.90,
            request.strike_price * 1.10,
            request.strike_price * 1.15,
            request.time_to_expiry,
            request.risk_free_rate,
            request.volatility,
        ),
        _ => {
            return Err(format!(
                "Unknown strategy: {}. Valid: straddle, strangle, butterfly, iron_condor",
                request.strategy
            ))
        }
    }
    .map_err(|e| e.to_string())?;

    Ok(StrategyResponse { strategy: result })
}
