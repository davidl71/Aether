use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};

const DEFAULT_MIN_LIQUIDITY_SCORE: f64 = 50.0;
const FRED_BASE_URL: &str = "https://api.stlouisfed.org/fred";
const NEW_YORK_FED_BASE_URL: &str = "https://markets.newyorkfed.org/api";

#[derive(Debug, Clone, Deserialize)]
pub struct BoxSpreadInput {
    pub symbol: String,
    pub expiry: String,
    pub days_to_expiry: i32,
    pub strike_width: f64,
    pub buy_implied_rate: f64,
    pub sell_implied_rate: f64,
    pub net_debit: f64,
    pub net_credit: f64,
    pub liquidity_score: f64,
    pub spread_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatePointResponse {
    pub symbol: String,
    pub expiry: String,
    pub days_to_expiry: i32,
    pub strike_width: f64,
    pub buy_implied_rate: f64,
    pub sell_implied_rate: f64,
    pub mid_rate: f64,
    pub net_debit: f64,
    pub net_credit: f64,
    pub liquidity_score: f64,
    pub timestamp: String,
    pub spread_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CurveResponse {
    pub symbol: String,
    pub points: Vec<RatePointResponse>,
    pub timestamp: String,
    pub strike_width: Option<f64>,
    pub point_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComparisonResponse {
    pub dte: i32,
    pub box_spread_rate: f64,
    pub benchmark_rate: f64,
    pub benchmark_type: String,
    pub spread_bps: f64,
    pub liquidity_score: f64,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkRateResponse {
    pub tenor: String,
    pub rate: f64,
    pub days_to_expiry: Option<i32>,
    pub timestamp: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SofrBenchmarksResponse {
    pub overnight: SofrOvernightResponse,
    pub term_rates: Vec<BenchmarkRateResponse>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SofrOvernightResponse {
    pub rate: Option<f64>,
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TreasuryBenchmarksResponse {
    pub rates: Vec<BenchmarkRateResponse>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CurveQuery {
    pub symbol: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum CurveRequest {
    Opportunities(Vec<Value>),
    Named {
        opportunities: Vec<Value>,
        symbol: Option<String>,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum CompareRequest {
    Opportunities(Vec<Value>),
    Named {
        opportunities: Vec<Value>,
        symbol: Option<String>,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct YieldCurveComparisonRequest {
    pub box_spread_rates: HashMap<String, HashMap<String, f64>>,
    pub treasury_rates: Option<HashMap<String, f64>>,
    pub sofr_rates: Option<HashMap<String, f64>>,
    #[serde(default)]
    pub fetch_live: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct YieldCurveComparisonResponse {
    pub symbols: Vec<String>,
    pub tenor_count: usize,
    pub treasury_points: usize,
    pub sofr_points: usize,
    pub spread_points: usize,
    pub box_spread_wins: usize,
    pub benchmark_wins: usize,
    pub ties: usize,
    pub generated: String,
    pub box_curves: HashMap<String, Vec<CurvePointResponse>>,
    pub treasury_curve: Vec<BenchmarkCurvePointResponse>,
    pub sofr_curve: Vec<BenchmarkCurvePointResponse>,
    pub spreads: Vec<SpreadPointResponse>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CurvePointResponse {
    pub dte: i32,
    pub rate: f64,
    pub liquidity: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkCurvePointResponse {
    pub dte: i32,
    pub rate: f64,
    pub tenor: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpreadPointResponse {
    pub dte: i32,
    pub tenor: String,
    pub box_rate: f64,
    pub benchmark_rate: f64,
    pub benchmark_source: String,
    pub spread_bps: f64,
    pub symbol: String,
}

#[derive(Debug, Clone)]
pub struct BenchmarkRate {
    pub rate_type: String,
    pub tenor: String,
    pub days_to_expiry: Option<i32>,
    pub rate: f64,
    pub timestamp: String,
    pub source: String,
}

#[derive(Debug, Clone)]
struct CurvePoint {
    dte: i32,
    rate: f64,
    source: String,
    tenor: String,
    liquidity: f64,
}

pub fn extract_rate(input: BoxSpreadInput) -> Result<RatePointResponse, String> {
    build_rate_point(input, DEFAULT_MIN_LIQUIDITY_SCORE)
        .ok_or_else(|| "Invalid box spread data or below minimum liquidity threshold".to_string())
}

pub fn build_curve(
    request: CurveRequest,
    query: Option<CurveQuery>,
) -> Result<CurveResponse, String> {
    let (opportunities, symbol) = split_curve_request(request, query)?;
    let points = aggregate_opportunities(&opportunities, &symbol, DEFAULT_MIN_LIQUIDITY_SCORE);
    Ok(CurveResponse {
        symbol,
        strike_width: points.first().map(|point| point.strike_width),
        point_count: points.len(),
        points,
        timestamp: Utc::now().to_rfc3339(),
    })
}

pub async fn compare_rates(
    request: CompareRequest,
    query: Option<CurveQuery>,
    client: &Client,
) -> Result<Vec<ComparisonResponse>, String> {
    let (opportunities, symbol) = split_compare_request(request, query)?;
    let curve_points =
        aggregate_opportunities(&opportunities, &symbol, DEFAULT_MIN_LIQUIDITY_SCORE);
    let benchmarks = all_benchmarks(client).await;

    let mut comparisons = Vec::new();
    for point in curve_points {
        if let Some(benchmark) = closest_benchmark(point.days_to_expiry, &benchmarks, 10) {
            comparisons.push(ComparisonResponse {
                dte: point.days_to_expiry,
                box_spread_rate: point.mid_rate,
                benchmark_rate: benchmark.rate,
                benchmark_type: benchmark.rate_type.clone(),
                spread_bps: (point.mid_rate - benchmark.rate) * 100.0,
                liquidity_score: point.liquidity_score,
                timestamp: point.timestamp.clone(),
            });
        }
    }

    Ok(comparisons)
}

pub async fn get_sofr_rates(client: &Client) -> SofrBenchmarksResponse {
    let overnight = fetch_sofr_overnight(client).await;
    let term_rates = fetch_sofr_term_rates(client).await;

    SofrBenchmarksResponse {
        overnight: SofrOvernightResponse {
            rate: overnight.as_ref().map(|rate| rate.rate),
            timestamp: overnight.as_ref().map(|rate| rate.timestamp.clone()),
        },
        term_rates: term_rates
            .into_iter()
            .map(BenchmarkRateResponse::from)
            .collect(),
        timestamp: Utc::now().to_rfc3339(),
    }
}

pub async fn get_treasury_rates(client: &Client) -> TreasuryBenchmarksResponse {
    let rates = fetch_treasury_rates(client).await;
    TreasuryBenchmarksResponse {
        rates: rates.into_iter().map(BenchmarkRateResponse::from).collect(),
        timestamp: Utc::now().to_rfc3339(),
    }
}

pub async fn yield_curve_comparison(
    request: YieldCurveComparisonRequest,
    client: &Client,
) -> YieldCurveComparisonResponse {
    let treasury_curve = if let Some(rates) = request.treasury_rates {
        manual_curve_points(rates, "manual")
    } else if request.fetch_live {
        fetch_treasury_rates(client)
            .await
            .into_iter()
            .filter_map(|rate| {
                rate.days_to_expiry.map(|dte| CurvePoint {
                    dte,
                    rate: rate.rate,
                    source: rate.source,
                    tenor: rate.tenor,
                    liquidity: 0.0,
                })
            })
            .collect()
    } else {
        Vec::new()
    };

    let sofr_curve = if let Some(rates) = request.sofr_rates {
        manual_curve_points(rates, "manual")
    } else if request.fetch_live {
        let mut points = Vec::new();
        if let Some(rate) = fetch_sofr_overnight(client).await {
            if let Some(dte) = rate.days_to_expiry {
                points.push(CurvePoint {
                    dte,
                    rate: rate.rate,
                    source: rate.source,
                    tenor: "O/N".to_string(),
                    liquidity: 0.0,
                });
            }
        }
        points.extend(
            fetch_sofr_term_rates(client)
                .await
                .into_iter()
                .filter_map(|rate| {
                    rate.days_to_expiry.map(|dte| CurvePoint {
                        dte,
                        rate: rate.rate,
                        source: rate.source,
                        tenor: rate.tenor,
                        liquidity: 0.0,
                    })
                }),
        );
        points
    } else {
        Vec::new()
    };

    let mut box_curves = HashMap::new();
    for (symbol, rates) in request.box_spread_rates {
        let mut points = rates
            .into_iter()
            .filter_map(|(dte, rate)| {
                dte.parse::<i32>().ok().map(|parsed| CurvePoint {
                    dte: parsed,
                    rate,
                    source: format!("box_spread:{symbol}"),
                    tenor: tenor_label(parsed),
                    liquidity: 0.0,
                })
            })
            .collect::<Vec<_>>();
        points.sort_by_key(|point| point.dte);
        box_curves.insert(symbol, points);
    }

    let mut spreads = Vec::new();
    let benchmarks = treasury_curve
        .iter()
        .chain(sofr_curve.iter())
        .cloned()
        .collect::<Vec<_>>();
    for (symbol, points) in &box_curves {
        for point in points {
            if let Some(benchmark) = closest_curve_point(point.dte, &benchmarks, 20) {
                spreads.push(SpreadPointResponse {
                    dte: point.dte,
                    tenor: point.tenor.clone(),
                    box_rate: point.rate,
                    benchmark_rate: benchmark.rate,
                    benchmark_source: benchmark.source.clone(),
                    spread_bps: (point.rate - benchmark.rate) * 100.0,
                    symbol: symbol.clone(),
                });
            }
        }
    }
    spreads.sort_by(|a, b| a.symbol.cmp(&b.symbol).then(a.dte.cmp(&b.dte)));

    let symbols = {
        let mut values = box_curves.keys().cloned().collect::<Vec<_>>();
        values.sort();
        values
    };
    let tenors = {
        let mut set = BTreeMap::new();
        for points in box_curves.values() {
            for point in points {
                set.insert(point.dte, ());
            }
        }
        for point in &treasury_curve {
            set.insert(point.dte, ());
        }
        for point in &sofr_curve {
            set.insert(point.dte, ());
        }
        set.len()
    };
    let box_spread_wins = spreads.iter().filter(|item| item.spread_bps > 5.0).count();
    let benchmark_wins = spreads.iter().filter(|item| item.spread_bps < -5.0).count();
    let ties = spreads
        .len()
        .saturating_sub(box_spread_wins + benchmark_wins);

    YieldCurveComparisonResponse {
        symbols,
        tenor_count: tenors,
        treasury_points: treasury_curve.len(),
        sofr_points: sofr_curve.len(),
        spread_points: spreads.len(),
        box_spread_wins,
        benchmark_wins,
        ties,
        generated: Utc::now().to_rfc3339(),
        box_curves: box_curves
            .into_iter()
            .map(|(symbol, points)| {
                (
                    symbol,
                    points
                        .into_iter()
                        .map(|point| CurvePointResponse {
                            dte: point.dte,
                            rate: point.rate,
                            liquidity: point.liquidity,
                        })
                        .collect(),
                )
            })
            .collect(),
        treasury_curve: treasury_curve
            .into_iter()
            .map(|point| BenchmarkCurvePointResponse {
                dte: point.dte,
                rate: point.rate,
                tenor: point.tenor,
                source: point.source,
            })
            .collect(),
        sofr_curve: sofr_curve
            .into_iter()
            .map(|point| BenchmarkCurvePointResponse {
                dte: point.dte,
                rate: point.rate,
                tenor: point.tenor,
                source: point.source,
            })
            .collect(),
        spreads,
    }
}

fn split_curve_request(
    request: CurveRequest,
    query: Option<CurveQuery>,
) -> Result<(Vec<Value>, String), String> {
    match request {
        CurveRequest::Opportunities(opportunities) => Ok((
            opportunities,
            query
                .and_then(|item| item.symbol)
                .ok_or_else(|| "symbol is required".to_string())?,
        )),
        CurveRequest::Named {
            opportunities,
            symbol,
        } => Ok((
            opportunities,
            symbol
                .or_else(|| query.and_then(|item| item.symbol))
                .ok_or_else(|| "symbol is required".to_string())?,
        )),
    }
}

fn split_compare_request(
    request: CompareRequest,
    query: Option<CurveQuery>,
) -> Result<(Vec<Value>, String), String> {
    match request {
        CompareRequest::Opportunities(opportunities) => Ok((
            opportunities,
            query
                .and_then(|item| item.symbol)
                .ok_or_else(|| "symbol is required".to_string())?,
        )),
        CompareRequest::Named {
            opportunities,
            symbol,
        } => Ok((
            opportunities,
            symbol
                .or_else(|| query.and_then(|item| item.symbol))
                .ok_or_else(|| "symbol is required".to_string())?,
        )),
    }
}

fn aggregate_opportunities(
    opportunities: &[Value],
    symbol: &str,
    min_liquidity_score: f64,
) -> Vec<RatePointResponse> {
    let mut grouped: BTreeMap<i32, Vec<RatePointResponse>> = BTreeMap::new();

    for opportunity in opportunities {
        let Some(spread) = opportunity.get("spread") else {
            continue;
        };
        let Ok(input) = serde_json::from_value::<BoxSpreadInput>(spread.clone()) else {
            continue;
        };
        if let Some(point) = build_rate_point(input, min_liquidity_score) {
            grouped.entry(point.days_to_expiry).or_default().push(point);
        }
    }

    grouped
        .into_iter()
        .filter_map(|(_dte, mut points)| {
            if points.len() == 1 {
                return points.pop();
            }
            let total_weight: f64 = points.iter().map(|point| point.liquidity_score).sum();
            let divisor = if total_weight > 0.0 {
                total_weight
            } else {
                points.len() as f64
            };
            let weighted = |accessor: fn(&RatePointResponse) -> f64| -> f64 {
                if total_weight > 0.0 {
                    points
                        .iter()
                        .map(|point| accessor(point) * point.liquidity_score)
                        .sum::<f64>()
                        / divisor
                } else {
                    points.iter().map(accessor).sum::<f64>() / divisor
                }
            };
            let template = points.first()?;
            Some(RatePointResponse {
                symbol: symbol.to_string(),
                expiry: template.expiry.clone(),
                days_to_expiry: template.days_to_expiry,
                strike_width: template.strike_width,
                buy_implied_rate: weighted(|point| point.buy_implied_rate),
                sell_implied_rate: weighted(|point| point.sell_implied_rate),
                mid_rate: weighted(|point| point.mid_rate),
                net_debit: points.iter().map(|point| point.net_debit).sum::<f64>()
                    / points.len() as f64,
                net_credit: points.iter().map(|point| point.net_credit).sum::<f64>()
                    / points.len() as f64,
                liquidity_score: points
                    .iter()
                    .map(|point| point.liquidity_score)
                    .fold(0.0_f64, f64::max),
                timestamp: Utc::now().to_rfc3339(),
                spread_id: None,
            })
        })
        .collect()
}

fn build_rate_point(input: BoxSpreadInput, min_liquidity_score: f64) -> Option<RatePointResponse> {
    let mid_rate = if input.buy_implied_rate > 0.0 && input.sell_implied_rate > 0.0 {
        (input.buy_implied_rate + input.sell_implied_rate) / 2.0
    } else if input.buy_implied_rate > 0.0 {
        input.buy_implied_rate
    } else if input.sell_implied_rate > 0.0 {
        input.sell_implied_rate
    } else {
        return None;
    };

    if input.days_to_expiry <= 0
        || input.strike_width <= 0.0
        || input.liquidity_score < min_liquidity_score
    {
        return None;
    }

    Some(RatePointResponse {
        symbol: input.symbol,
        expiry: input.expiry,
        days_to_expiry: input.days_to_expiry,
        strike_width: input.strike_width,
        buy_implied_rate: input.buy_implied_rate,
        sell_implied_rate: input.sell_implied_rate,
        mid_rate,
        net_debit: input.net_debit,
        net_credit: input.net_credit,
        liquidity_score: input.liquidity_score,
        timestamp: Utc::now().to_rfc3339(),
        spread_id: input.spread_id,
    })
}

async fn all_benchmarks(client: &Client) -> Vec<BenchmarkRate> {
    let mut benchmarks = Vec::new();
    if let Some(overnight) = fetch_sofr_overnight(client).await {
        benchmarks.push(overnight);
    }
    benchmarks.extend(fetch_sofr_term_rates(client).await);
    benchmarks.extend(fetch_treasury_rates(client).await);
    benchmarks
}

async fn fetch_sofr_overnight(client: &Client) -> Option<BenchmarkRate> {
    if let Some(api_key) = fred_api_key() {
        if let Some(rate) = fetch_fred_latest_series(client, "SOFR", Some(&api_key), 1).await {
            return Some(BenchmarkRate {
                rate_type: "SOFR".to_string(),
                tenor: "Overnight".to_string(),
                days_to_expiry: Some(1),
                rate: rate.0,
                timestamp: Utc::now().to_rfc3339(),
                source: "FRED (St. Louis Fed)".to_string(),
            });
        }
    }

    let url = format!("{NEW_YORK_FED_BASE_URL}/rates/all");
    let Ok(response) = client.get(url).send().await else {
        return None;
    };
    let Ok(json) = response.json::<Value>().await else {
        return None;
    };
    let sofr = json
        .get("sofr")
        .or_else(|| json.get("SOFR"))
        .and_then(Value::as_object)?;
    let rate = sofr
        .get("rate")
        .or_else(|| sofr.get("value"))
        .and_then(Value::as_f64)?;
    if rate <= 0.0 {
        return None;
    }
    Some(BenchmarkRate {
        rate_type: "SOFR".to_string(),
        tenor: "Overnight".to_string(),
        days_to_expiry: Some(1),
        rate,
        timestamp: Utc::now().to_rfc3339(),
        source: "New York Fed".to_string(),
    })
}

async fn fetch_sofr_term_rates(client: &Client) -> Vec<BenchmarkRate> {
    let Some(api_key) = fred_api_key() else {
        return Vec::new();
    };

    let mut rates = Vec::new();
    for (tenor, series_id, dte) in [
        ("1M", "SOFR30DAYAVG", 30),
        ("3M", "SOFR90DAYAVG", 90),
        ("6M", "SOFR180DAYAVG", 180),
    ] {
        if let Some((rate, _date)) =
            fetch_fred_latest_series(client, series_id, Some(&api_key), 1).await
        {
            rates.push(BenchmarkRate {
                rate_type: "SOFR".to_string(),
                tenor: tenor.to_string(),
                days_to_expiry: Some(dte),
                rate,
                timestamp: Utc::now().to_rfc3339(),
                source: "FRED (St. Louis Fed)".to_string(),
            });
        }
    }

    if let Some(rate) = fetch_sofr_one_year(client, &api_key).await {
        rates.push(rate);
    }

    rates
}

async fn fetch_sofr_one_year(client: &Client, api_key: &str) -> Option<BenchmarkRate> {
    let observations = fetch_fred_series_observations(client, "SOFRINDEX", api_key, 400).await?;
    if observations.len() <= 252 {
        return None;
    }
    let latest = &observations[0];
    let year_ago = &observations[252];
    let latest_value = latest.1?;
    let year_ago_value = year_ago.1?;
    if latest_value <= 0.0 || year_ago_value <= 0.0 {
        return None;
    }
    let rate = (latest_value / year_ago_value - 1.0) * 100.0;
    if !(0.1..25.0).contains(&rate) {
        return None;
    }
    Some(BenchmarkRate {
        rate_type: "SOFR".to_string(),
        tenor: "1Y".to_string(),
        days_to_expiry: Some(365),
        rate,
        timestamp: Utc::now().to_rfc3339(),
        source: "FRED (St. Louis Fed)".to_string(),
    })
}

async fn fetch_treasury_rates(client: &Client) -> Vec<BenchmarkRate> {
    let Some(api_key) = fred_api_key() else {
        return Vec::new();
    };

    let mut rates = Vec::new();
    for (tenor, series_id, dte) in [
        ("1M", "DGS1MO", 30),
        ("3M", "DGS3MO", 90),
        ("6M", "DGS6MO", 180),
        ("1Y", "DGS1", 365),
        ("2Y", "DGS2", 730),
        ("5Y", "DGS5", 1825),
        ("10Y", "DGS10", 3650),
        ("30Y", "DGS30", 10950),
    ] {
        if let Some((rate, _date)) =
            fetch_fred_latest_series(client, series_id, Some(&api_key), 1).await
        {
            rates.push(BenchmarkRate {
                rate_type: "Treasury".to_string(),
                tenor: tenor.to_string(),
                days_to_expiry: Some(dte),
                rate,
                timestamp: Utc::now().to_rfc3339(),
                source: "FRED (St. Louis Fed)".to_string(),
            });
        }
    }
    rates
}

async fn fetch_fred_latest_series(
    client: &Client,
    series_id: &str,
    api_key: Option<&str>,
    limit: usize,
) -> Option<(f64, String)> {
    let api_key = api_key?;
    let observations = fetch_fred_series_observations(client, series_id, api_key, limit).await?;
    observations
        .into_iter()
        .find_map(|(date, value)| value.map(|rate| (rate, date)))
}

async fn fetch_fred_series_observations(
    client: &Client,
    series_id: &str,
    api_key: &str,
    limit: usize,
) -> Option<Vec<(String, Option<f64>)>> {
    let url = format!("{FRED_BASE_URL}/series/observations");
    let response = client
        .get(url)
        .query(&[
            ("series_id", series_id),
            ("api_key", api_key),
            ("file_type", "json"),
            ("limit", &limit.to_string()),
            ("sort_order", "desc"),
        ])
        .send()
        .await
        .ok()?;
    let json = response.json::<Value>().await.ok()?;
    let observations = json.get("observations")?.as_array()?;
    Some(
        observations
            .iter()
            .filter_map(|item| {
                let date = item.get("date")?.as_str()?.to_string();
                let value = match item.get("value")?.as_str()? {
                    "." => None,
                    raw => raw.parse::<f64>().ok(),
                };
                Some((date, value))
            })
            .collect(),
    )
}

fn fred_api_key() -> Option<String> {
    std::env::var("FRED_API_KEY")
        .ok()
        .filter(|value| !value.trim().is_empty())
}

fn closest_benchmark(
    dte: i32,
    benchmarks: &[BenchmarkRate],
    tolerance: i32,
) -> Option<&BenchmarkRate> {
    benchmarks
        .iter()
        .filter_map(|benchmark| {
            benchmark
                .days_to_expiry
                .map(|days| (benchmark, (days - dte).abs()))
        })
        .filter(|(_, diff)| *diff <= tolerance)
        .min_by_key(|(_, diff)| *diff)
        .map(|(benchmark, _)| benchmark)
}

fn manual_curve_points(rates: HashMap<String, f64>, source: &str) -> Vec<CurvePoint> {
    let mut points = rates
        .into_iter()
        .filter_map(|(dte, rate)| {
            dte.parse::<i32>().ok().map(|parsed| CurvePoint {
                dte: parsed,
                rate,
                source: source.to_string(),
                tenor: tenor_label(parsed),
                liquidity: 0.0,
            })
        })
        .collect::<Vec<_>>();
    points.sort_by_key(|point| point.dte);
    points
}

fn closest_curve_point(dte: i32, points: &[CurvePoint], tolerance: i32) -> Option<CurvePoint> {
    points
        .iter()
        .min_by_key(|point| (point.dte - dte).abs())
        .filter(|point| (point.dte - dte).abs() <= tolerance)
        .cloned()
}

fn tenor_label(dte: i32) -> String {
    match dte {
        1 => "O/N".to_string(),
        30 => "1M".to_string(),
        60 => "2M".to_string(),
        90 => "3M".to_string(),
        120 => "4M".to_string(),
        180 => "6M".to_string(),
        270 => "9M".to_string(),
        365 => "1Y".to_string(),
        730 => "2Y".to_string(),
        1095 => "3Y".to_string(),
        1825 => "5Y".to_string(),
        2555 => "7Y".to_string(),
        3650 => "10Y".to_string(),
        7300 => "20Y".to_string(),
        10950 => "30Y".to_string(),
        value if value < 30 => format!("{value}d"),
        value => {
            let months = (value as f64 / 30.0).round() as i32;
            if months <= 12 {
                format!("{months}M")
            } else {
                let years = value as f64 / 365.0;
                format!("{years:.1}Y")
            }
        }
    }
}

impl From<BenchmarkRate> for BenchmarkRateResponse {
    fn from(value: BenchmarkRate) -> Self {
        Self {
            tenor: value.tenor,
            rate: value.rate,
            days_to_expiry: value.days_to_expiry,
            timestamp: value.timestamp,
            source: value.source,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn extract_rate_rejects_low_liquidity() {
        let result = extract_rate(BoxSpreadInput {
            symbol: "XSP".into(),
            expiry: "20261218".into(),
            days_to_expiry: 100,
            strike_width: 100.0,
            buy_implied_rate: 4.9,
            sell_implied_rate: 5.1,
            net_debit: 95.0,
            net_credit: 105.0,
            liquidity_score: 10.0,
            spread_id: None,
        });

        assert!(result.is_err());
    }

    #[test]
    fn build_curve_aggregates_duplicate_dte_points() {
        let request = CurveRequest::Named {
            symbol: Some("XSP".into()),
            opportunities: vec![
                json!({"spread": {
                    "symbol": "XSP",
                    "expiry": "20261218",
                    "days_to_expiry": 30,
                    "strike_width": 100.0,
                    "buy_implied_rate": 4.8,
                    "sell_implied_rate": 5.0,
                    "net_debit": 95.0,
                    "net_credit": 105.0,
                    "liquidity_score": 70.0
                }}),
                json!({"spread": {
                    "symbol": "XSP",
                    "expiry": "20261218",
                    "days_to_expiry": 30,
                    "strike_width": 100.0,
                    "buy_implied_rate": 5.0,
                    "sell_implied_rate": 5.2,
                    "net_debit": 96.0,
                    "net_credit": 106.0,
                    "liquidity_score": 90.0
                }}),
            ],
        };

        let curve = build_curve(request, None).expect("curve");
        assert_eq!(curve.point_count, 1);
        assert_eq!(curve.points[0].days_to_expiry, 30);
        assert!(curve.points[0].mid_rate > 4.9);
    }

    #[test]
    fn yield_curve_comparison_builds_spreads() {
        let response = futures::executor::block_on(yield_curve_comparison(
            YieldCurveComparisonRequest {
                box_spread_rates: HashMap::from([(
                    "XSP".into(),
                    HashMap::from([("30".into(), 5.0), ("90".into(), 5.1)]),
                )]),
                treasury_rates: Some(HashMap::from([("30".into(), 4.8), ("90".into(), 4.9)])),
                sofr_rates: None,
                fetch_live: false,
            },
            &Client::new(),
        ));

        assert_eq!(response.symbols, vec!["XSP".to_string()]);
        assert_eq!(response.spread_points, 2);
        assert_eq!(response.box_spread_wins, 2);
    }
}
