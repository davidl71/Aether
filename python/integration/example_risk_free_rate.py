#!/usr/bin/env python3
"""
example_risk_free_rate.py - Example usage of risk-free rate extraction

This script demonstrates how to:
1. Extract risk-free rates from box spreads
2. Build yield curves
3. Compare with SOFR/Treasury benchmarks
"""

from risk_free_rate_extractor import RiskFreeRateExtractor, RiskFreeRateCurve
from sofr_treasury_client import SOFRTreasuryClient, RateComparison

# Example box spread opportunities (from your strategy)
example_opportunities = [
    {
        "spread": {
            "symbol": "SPX",
            "expiry": "20250131",
            "days_to_expiry": 30,
            "strike_width": 50.0,
            "buy_implied_rate": 5.25,
            "sell_implied_rate": 5.15,
            "buy_net_debit": 49.50,
            "sell_net_credit": 49.60,
            "liquidity_score": 75.0,
            "spread_id": "SPX-20250131-50"
        }
    },
    {
        "spread": {
            "symbol": "SPX",
            "expiry": "20250228",
            "days_to_expiry": 60,
            "strike_width": 50.0,
            "buy_implied_rate": 5.30,
            "sell_implied_rate": 5.20,
            "buy_net_debit": 49.45,
            "sell_net_credit": 49.65,
            "liquidity_score": 80.0,
            "spread_id": "SPX-20250228-50"
        }
    },
    {
        "spread": {
            "symbol": "SPX",
            "expiry": "20250331",
            "days_to_expiry": 90,
            "strike_width": 50.0,
            "buy_implied_rate": 5.35,
            "sell_implied_rate": 5.25,
            "buy_net_debit": 49.40,
            "sell_net_credit": 49.70,
            "liquidity_score": 70.0,
            "spread_id": "SPX-20250331-50"
        }
    }
]


def main():
    print("Risk-Free Rate Extraction Example")
    print("=" * 50)
    print()

    # Initialize extractor
    extractor = RiskFreeRateExtractor(min_liquidity_score=50.0)

    # Build yield curve from opportunities
    print("1. Building yield curve from box spreads...")
    curve = extractor.build_curve_from_opportunities(example_opportunities, "SPX")
    print(f"   ✓ Curve built with {len(curve.points)} points")
    print()

    # Display curve points
    print("2. Yield Curve Points:")
    print(f"   {'DTE':<6} {'Mid Rate':<12} {'Buy Rate':<12} {'Sell Rate':<12} {'Liquidity':<10}")
    print("   " + "-" * 60)
    for point in curve.points:
        print(f"   {point.days_to_expiry:<6} {point.mid_rate:>10.2f}%  {point.buy_implied_rate:>10.2f}%  "
              f"{point.sell_implied_rate:>10.2f}%  {point.liquidity_score:>8.1f}")
    print()

    # Get rate at specific DTE
    print("3. Rate Lookup:")
    rate_30d = curve.get_rate_at_dte(30, tolerance=5)
    if rate_30d:
        print(f"   ✓ 30-day rate: {rate_30d:.2f}%")
    else:
        print("   ⚠ No rate found for 30-day maturity")
    print()

    # Compare with benchmarks
    print("4. Benchmark Comparison:")
    benchmark_client = SOFRTreasuryClient()
    sofr = benchmark_client.get_sofr_overnight()
    if sofr:
        print(f"   SOFR Overnight: {sofr.rate:.2f}%")
        if rate_30d:
            spread = RateComparison.calculate_spread(rate_30d, sofr.rate)
            print(f"   Box Spread (30d) vs SOFR: {spread:+.1f} bps")
    else:
        print("   ⚠ SOFR data not available (would require API integration)")
    print()

    # Filter by liquidity
    print("5. High-Liquidity Curve:")
    liquid_curve = curve.filter_by_liquidity(min_liquidity=70.0)
    print(f"   ✓ Filtered to {len(liquid_curve.points)} high-liquidity points")
    print()

    print("Example complete!")
    print()
    print("Next steps:")
    print("  - Integrate with live box spread opportunities")
    print("  - Connect to SOFR/Treasury APIs for real benchmarks")
    print("  - Build term structure visualization")
    print("  - Use rates for option pricing and discounting")


if __name__ == "__main__":
    main()
