"""
Examples of using the Box Spread DSL

Demonstrates various DSL usage patterns for box spread scenarios,
financing strategies, and cash flow modeling.
"""

from box_spread_dsl import (
    BoxSpread, Direction, Benchmark,
    FinancingStrategy, bank_loan, box_spread, margin_for, invest_in, minimize_total_cost,
    CashFlowModel, box_spread_lending, bank_loan as bank_loan_pos, pension_loan
)


def example_basic_box_spread():
    """Basic box spread scenario"""
    scenario = BoxSpread("SPX") \
        .strike_width(50) \
        .expiration("2025-12-19")

    print(f"Scenario: {scenario}")
    result = scenario.evaluate()
    print(f"Valid: {result.is_valid()}")
    return scenario


def example_financing_constraints():
    """Box spread with financing constraints"""
    scenario = BoxSpread("SPX") \
        .strike_width(50) \
        .expiration("2025-12-19") \
        .direction(Direction.LENDING) \
        .min_implied_rate(4.5) \
        .benchmark(Benchmark.SOFR) \
        .min_advantage_bps(50) \
        .liquidity(min_volume=100, min_open_interest=500, max_spread=0.1)

    print(f"Scenario: {scenario}")
    return scenario


def example_multi_asset_strategy():
    """Multi-asset financing strategy"""
    strategy = FinancingStrategy("optimize_loan_to_box_spread") \
        .source(bank_loan(rate=5.5, amount=100000)) \
        .use_as(margin_for(
            box_spread(symbol="SPX", min_rate=4.0)
        )) \
        .then(invest_in(fund="providence", rate=3.0)) \
        .optimize(minimize_total_cost())

    print(f"Strategy: {strategy}")
    return strategy


def example_cash_flow_model():
    """Cash flow modeling"""
    cash_flow = CashFlowModel() \
        .add_position(
            box_spread_lending(
                amount=50000,
                rate=4.8,
                maturity="2025-12-19"
            )
        ) \
        .add_position(
            bank_loan_pos(
                amount=100000,
                rate=5.5,
                payments="monthly"
            )
        ) \
        .add_position(
            pension_loan(
                amount=50000,
                rate=3.2,
                collateral="pension_fund"
            )
        ) \
        .project(months=12) \
        .optimize("net_cash_flow")

    print(f"Cash Flow Model: {cash_flow}")
    result = cash_flow.simulate()
    print(f"Total Net Cash Flow: {result.total_net_cash_flow}")
    return cash_flow


def example_code_generation():
    """Generate C++ code from DSL"""
    scenario = BoxSpread("SPX") \
        .strike_width(50) \
        .expiration("2025-12-19") \
        .min_implied_rate(4.5) \
        .benchmark(Benchmark.SOFR)

    cpp_code = scenario.to_cpp()
    print("Generated C++ Code:")
    print(cpp_code)
    return cpp_code


if __name__ == "__main__":
    print("=== Basic Box Spread ===")
    example_basic_box_spread()

    print("\n=== Financing Constraints ===")
    example_financing_constraints()

    print("\n=== Multi-Asset Strategy ===")
    example_multi_asset_strategy()

    print("\n=== Cash Flow Model ===")
    example_cash_flow_model()

    print("\n=== Code Generation ===")
    example_code_generation()
