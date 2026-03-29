use super::*;
use serde_json::json;
use std::collections::HashMap;

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
