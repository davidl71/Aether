//! Orchestration: compare box-spread points to benchmarks and multi-curve yield comparison.

use crate::finance_rates::benchmarks::{
    all_benchmarks, fetch_sofr_overnight, fetch_sofr_term_rates, fetch_treasury_rates,
};
use crate::finance_rates::curve::{
    aggregate_opportunities, split_compare_request, DEFAULT_MIN_LIQUIDITY_SCORE,
};
use crate::finance_rates::types::{
    BenchmarkCurvePointResponse, BenchmarkRate, CompareRequest, ComparisonResponse,
    CurvePointResponse, CurveQuery, SpreadPointResponse, YieldCurveComparisonRequest,
    YieldCurveComparisonResponse,
};
use chrono::Utc;
use reqwest::Client;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone)]
struct CurvePoint {
    dte: i32,
    rate: f64,
    source: String,
    tenor: String,
    liquidity: f64,
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
