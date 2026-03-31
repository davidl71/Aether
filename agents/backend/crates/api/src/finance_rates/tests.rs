use super::*;
use market_data::yield_curve::synthetic_yield_curve as market_data_synthetic_yield_curve;
use serde_json::json;
use std::collections::HashMap;

/// Distinct from `SYNTHETIC_BASE_RATE` in `curve.rs` so synthetic “live base” vs default is observable.
const LIVE_BASE_RATE_FOR_TEST: f64 = 0.06;

#[test]
fn extract_rate_rejects_low_liquidity() {
    let result = extract_rate(BoxSpreadInput {
        symbol: "XSP".into(),
        expiry: "20261218".into(),
        days_to_expiry: 100,
        strike_width: 100.0,
        strike_low: None,
        strike_high: None,
        buy_implied_rate: 4.9,
        sell_implied_rate: 5.1,
        net_debit: 95.0,
        net_credit: 105.0,
        liquidity_score: 10.0,
        spread_id: None,
        convenience_yield: None,
        delayed: None,
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
fn build_curve_preserves_source_label() {
    let request = CurveRequest::Named {
        symbol: Some("XSP".into()),
        opportunities: vec![json!({
            "spread": {
                "symbol": "XSP",
                "expiry": "20261218",
                "days_to_expiry": 30,
                "strike_width": 100.0,
                "buy_implied_rate": 4.8,
                "sell_implied_rate": 5.0,
                "net_debit": 95.0,
                "net_credit": 105.0,
                "liquidity_score": 70.0
            },
            "data_source": "yahoo"
        })],
    };

    let curve = build_curve(request, None).expect("curve");
    assert_eq!(curve.point_count, 1);
    assert_eq!(curve.points[0].data_source.as_deref(), Some("yahoo"));
}

#[test]
fn build_curve_preserves_strike_bounds_when_present() {
    let request = CurveRequest::Named {
        symbol: Some("XSP".into()),
        opportunities: vec![json!({
            "spread": {
                "symbol": "XSP",
                "expiry": "20261218",
                "days_to_expiry": 30,
                "strike_width": 4.0,
                "strike_low": 5998.0,
                "strike_high": 6002.0,
                "buy_implied_rate": 0.048,
                "sell_implied_rate": 0.050,
                "net_debit": 3.98,
                "net_credit": 3.96,
                "liquidity_score": 70.0
            },
            "data_source": "tws"
        })],
    };

    let curve = build_curve(request, None).expect("curve");
    assert_eq!(curve.point_count, 1);
    assert_eq!(curve.points[0].data_source.as_deref(), Some("tws"));
    assert_eq!(curve.points[0].strike_low, Some(5998.0));
    assert_eq!(curve.points[0].strike_high, Some(6002.0));
}

#[test]
fn build_curve_mixed_sources_collapses_label_to_mixed() {
    let request = CurveRequest::Named {
        symbol: Some("XSP".into()),
        opportunities: vec![
            json!({
                "spread": {
                    "symbol": "XSP",
                    "expiry": "20261218",
                    "days_to_expiry": 30,
                    "strike_width": 4.0,
                    "buy_implied_rate": 0.048,
                    "sell_implied_rate": 0.050,
                    "net_debit": 3.98,
                    "net_credit": 3.96,
                    "liquidity_score": 70.0
                },
                "data_source": "tws",
                "comparison": { "bag_ms": 12, "legs_ms": 90 }
            }),
            json!({
                "spread": {
                    "symbol": "XSP",
                    "expiry": "20261218",
                    "days_to_expiry": 30,
                    "strike_width": 4.0,
                    "buy_implied_rate": 0.049,
                    "sell_implied_rate": 0.051,
                    "net_debit": 3.97,
                    "net_credit": 3.95,
                    "liquidity_score": 80.0
                },
                "data_source": "yahoo"
            }),
        ],
    };

    let curve = build_curve(request, None).expect("curve");
    assert_eq!(curve.point_count, 1);
    assert_eq!(curve.points[0].days_to_expiry, 30);
    assert_eq!(curve.points[0].data_source.as_deref(), Some("mixed"));
}

#[test]
fn build_synthetic_curve_is_deterministic_shape_for_tests() {
    let curve = build_synthetic_curve("MOCK_SPX", Some(0.05));
    assert_eq!(curve.symbol, "MOCK_SPX");
    assert_eq!(curve.point_count, 6);
    assert_eq!(curve.strike_width, Some(4.0));
    assert_eq!(curve.underlying_price, Some(6000.0));
    let dtes: Vec<i32> = curve.points.iter().map(|p| p.days_to_expiry).collect();
    assert_eq!(dtes, vec![30, 60, 90, 120, 180, 365]);
    for p in &curve.points {
        assert_eq!(p.data_source.as_deref(), Some("synthetic"));
        assert!(p.buy_implied_rate > 0.0 && p.sell_implied_rate >= p.buy_implied_rate);
        assert!(p.liquidity_score >= 50.0);
    }
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
        &reqwest::Client::new(),
    ));

    assert_eq!(response.symbols, vec!["XSP".to_string()]);
    assert_eq!(response.spread_points, 2);
    assert_eq!(response.box_spread_wins, 2);
}

/// With `fetch_live: false` and no injected maps, benchmarks are empty: no spread rows (safe CI path).
#[test]
fn yield_curve_comparison_fallback_no_benchmark_source_no_spreads() {
    let response = futures::executor::block_on(yield_curve_comparison(
        YieldCurveComparisonRequest {
            box_spread_rates: HashMap::from([("XSP".into(), HashMap::from([("30".into(), 5.0)]))]),
            treasury_rates: None,
            sofr_rates: None,
            fetch_live: false,
        },
        &reqwest::Client::new(),
    ));

    assert_eq!(response.treasury_points, 0);
    assert_eq!(response.sofr_points, 0);
    assert_eq!(response.spread_points, 0);
    assert!(response.spreads.is_empty());
}

/// Injected SOFR/Treasury maps mimic a live read without HTTP (paper-safe).
#[test]
fn yield_curve_comparison_manual_sofr_and_treasury_reflected_in_curves() {
    let response = futures::executor::block_on(yield_curve_comparison(
        YieldCurveComparisonRequest {
            box_spread_rates: HashMap::from([(
                "XSP".into(),
                HashMap::from([("30".into(), 5.05), ("90".into(), 5.15)]),
            )]),
            treasury_rates: Some(HashMap::from([("30".into(), 4.90), ("90".into(), 5.00)])),
            sofr_rates: Some(HashMap::from([("1".into(), 4.88)])),
            fetch_live: false,
        },
        &reqwest::Client::new(),
    ));

    assert_eq!(response.treasury_points, 2);
    assert_eq!(response.sofr_points, 1);
    assert_eq!(response.treasury_curve.len(), 2);
    assert_eq!(response.sofr_curve.len(), 1);
    assert_eq!(response.sofr_curve[0].rate, 4.88);
    assert_eq!(response.sofr_curve[0].dte, 1);
    assert_eq!(response.spread_points, 2);

    let s30 = response
        .spreads
        .iter()
        .find(|s| s.dte == 30)
        .expect("30d spread");
    assert!((s30.box_rate - 5.05).abs() < 1e-9);
    assert!((s30.benchmark_rate - 4.90).abs() < 1e-9);
    assert!((s30.spread_bps - (5.05 - 4.90) * 100.0).abs() < 1e-6);
    assert_eq!(s30.benchmark_source, "manual");
}

/// Synthetic curve uses `SYNTHETIC_BASE_RATE` when no live base; optional rate shifts mids (integration hook).
#[test]
fn build_synthetic_curve_live_base_rate_changes_shape_vs_default() {
    let default_curve = build_synthetic_curve("SYM", None);
    let liveish = build_synthetic_curve("SYM", Some(LIVE_BASE_RATE_FOR_TEST));
    assert_ne!(
        default_curve.points[0].mid_rate, liveish.points[0].mid_rate,
        "live_base_rate should alter synthetic mids"
    );
    assert!((liveish.points[0].mid_rate - LIVE_BASE_RATE_FOR_TEST).abs() < 1e-9);
}

/// Ensure the `market_data` synthetic yield curve can feed the finance_rates curve builder.
///
/// This exercises the “synthetic curve via market_data path” without any network calls.
#[test]
fn market_data_synthetic_curve_can_build_finance_rates_curve() {
    let curve = market_data_synthetic_yield_curve("XSP", 6000.0, 0.05);
    assert_eq!(curve.source, "synthetic");
    assert!(!curve.points.is_empty());

    let opportunities = curve
        .points
        .iter()
        .map(|p| {
            let expiry = p.expiry.format("%Y%m%d").to_string();
            json!({
                "spread": {
                    "symbol": curve.symbol,
                    "expiry": expiry,
                    "days_to_expiry": p.dte,
                    "strike_width": p.strike_width,
                    "strike_low": p.strike_low,
                    "strike_high": p.strike_high,
                    "buy_implied_rate": p.buy_implied_rate,
                    "sell_implied_rate": p.sell_implied_rate,
                    "net_debit": p.net_debit,
                    "net_credit": p.net_credit,
                    "liquidity_score": p.liquidity_score
                },
                "data_source": "synthetic"
            })
        })
        .collect::<Vec<_>>();

    let request = CurveRequest::Named {
        symbol: Some(curve.symbol),
        opportunities,
    };
    let response = build_curve(request, None).expect("curve");
    assert!(response.point_count > 0);
    assert!(response.points.iter().all(|p| p.data_source.as_deref() == Some("synthetic")));
    assert!(response.points.iter().all(|p| p.mid_rate > 0.0 && p.mid_rate < 1.0));
}

/// When the only benchmarks are farther than the 20 DTE matching window, box points produce no spreads.
#[test]
fn yield_curve_comparison_benchmark_far_tenor_skips_spread() {
    let response = futures::executor::block_on(yield_curve_comparison(
        YieldCurveComparisonRequest {
            box_spread_rates: HashMap::from([("XSP".into(), HashMap::from([("30".into(), 5.0)]))]),
            treasury_rates: Some(HashMap::from([("400".into(), 4.85)])),
            sofr_rates: None,
            fetch_live: false,
        },
        &reqwest::Client::new(),
    ));

    assert_eq!(response.treasury_points, 1);
    assert_eq!(response.spread_points, 0);
}

/// Live benchmark fetch smoke test (network + optional API keys).
///
/// This is intentionally ignored in CI; run it manually when validating the
/// "live rates" path end-to-end.
#[test]
#[ignore]
fn yield_curve_comparison_fetch_live_smoke() {
    let response = futures::executor::block_on(yield_curve_comparison(
        YieldCurveComparisonRequest {
            // Any box curve points are fine; the goal is to confirm we can
            // fetch at least one benchmark curve point and not panic.
            box_spread_rates: HashMap::from([("XSP".into(), HashMap::from([("30".into(), 5.0)]))]),
            treasury_rates: None,
            sofr_rates: None,
            fetch_live: true,
        },
        &reqwest::Client::new(),
    ));

    assert!(
        response.treasury_points + response.sofr_points > 0,
        "expected at least one live benchmark point; check network and optional FRED/FMP credentials"
    );
    // We may or may not match tenors closely enough to create spreads; just sanity check shape.
    assert_eq!(response.symbols, vec!["XSP".to_string()]);
    assert!(response.tenor_count >= response.treasury_points + response.sofr_points);
}
