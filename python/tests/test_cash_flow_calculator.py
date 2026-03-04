"""
test_cash_flow_calculator.py - Tests for the CashFlowCalculator.

Covers loan payments, option expiration (including box spreads), bond coupons/maturity,
and portfolio-level cash flow aggregation.
"""

import pytest
from datetime import datetime, timedelta

from python.integration.cash_flow_calculator import (
    CashFlowCalculator,
    CashFlowTimeline,
    CashFlowType,
    Loan,
    Position,
)


@pytest.fixture
def calculator():
    return CashFlowCalculator(fx_rates={"ILS_USD": 0.27})


@pytest.fixture
def now():
    return datetime.now()


class TestLoanCashFlows:
    def test_shir_loan_monthly_payments(self, calculator, now):
        loan = Loan(
            id="L1",
            loan_type="SHIR",
            principal=1_000_000.0,
            currency="ILS",
            shir_rate=0.045,
            spread=0.015,
        )
        end_date = now + timedelta(days=100)
        flows = calculator.calculate_loan_cash_flows(loan, end_date)

        assert len(flows) > 0
        assert all(f.cash_flow_type == CashFlowType.LOAN_PAYMENT for f in flows)
        assert all(f.amount < 0 for f in flows)  # outflows
        assert all(f.currency == "ILS" for f in flows)

    def test_shir_loan_payment_amount(self, calculator, now):
        loan = Loan(
            id="L1",
            loan_type="SHIR",
            principal=1_000_000.0,
            currency="ILS",
            shir_rate=0.045,
            spread=0.015,
        )
        end_date = now + timedelta(days=100)
        flows = calculator.calculate_loan_cash_flows(loan, end_date)

        # annual_rate = 0.045 + 0.015 = 0.06, monthly = 0.005
        # payment = 1_000_000 * 0.005 = 5000
        expected_payment = 5000.0
        assert abs(abs(flows[0].amount) - expected_payment) < 0.01

    def test_shir_loan_default_rate(self, calculator, now):
        loan = Loan(
            id="L2",
            loan_type="SHIR",
            principal=500_000.0,
            currency="ILS",
            shir_rate=None,
            spread=0.01,
        )
        end_date = now + timedelta(days=60)
        flows = calculator.calculate_loan_cash_flows(loan, end_date)

        assert len(flows) > 0
        # Default SHIR rate = 0.045, spread = 0.01, monthly = 0.055/12
        expected = 500_000.0 * (0.045 + 0.01) / 12.0
        assert abs(abs(flows[0].amount) - expected) < 0.01

    def test_cpi_linked_loan(self, calculator, now):
        loan = Loan(
            id="L3",
            loan_type="CPI_LINKED",
            principal=800_000.0,
            currency="ILS",
            fixed_rate=0.035,
            cpi_linked=True,
        )
        end_date = now + timedelta(days=60)
        flows = calculator.calculate_loan_cash_flows(loan, end_date)

        assert len(flows) > 0
        expected = 800_000.0 * 0.035 / 12.0
        assert abs(abs(flows[0].amount) - expected) < 0.01

    def test_cpi_linked_loan_default_rate(self, calculator, now):
        loan = Loan(
            id="L4",
            loan_type="CPI_LINKED",
            principal=600_000.0,
            currency="ILS",
            fixed_rate=None,
        )
        end_date = now + timedelta(days=60)
        flows = calculator.calculate_loan_cash_flows(loan, end_date)

        assert len(flows) > 0
        expected = 600_000.0 * 0.035 / 12.0
        assert abs(abs(flows[0].amount) - expected) < 0.01


class TestOptionCashFlows:
    def test_box_spread_expiration(self, calculator, now):
        expiry = now + timedelta(days=30)
        position = Position(
            id="P1",
            symbol="SPX",
            instrument_type="option",
            quantity=1.0,
            currency="USD",
            expiration_date=expiry,
            is_box_spread=True,
            box_strike_width=50.0,
        )
        flows = calculator.calculate_option_cash_flows(position, now + timedelta(days=60))

        assert len(flows) == 1
        assert flows[0].cash_flow_type == CashFlowType.OPTION_EXPIRATION
        assert flows[0].is_box_spread is True
        # 50 * 100 * 1 = 5000
        assert abs(flows[0].amount - 5000.0) < 0.01

    def test_box_spread_multiple_contracts(self, calculator, now):
        expiry = now + timedelta(days=30)
        position = Position(
            id="P2",
            symbol="XSP",
            instrument_type="option",
            quantity=10.0,
            currency="USD",
            expiration_date=expiry,
            is_box_spread=True,
            box_strike_width=5.0,
        )
        flows = calculator.calculate_option_cash_flows(position, now + timedelta(days=60))

        assert len(flows) == 1
        # 5 * 100 * 10 = 5000
        assert abs(flows[0].amount - 5000.0) < 0.01

    def test_call_option_itm(self, calculator, now):
        expiry = now + timedelta(days=30)
        position = Position(
            id="P3",
            symbol="AAPL",
            instrument_type="option",
            quantity=1.0,
            currency="USD",
            current_price=180.0,
            expiration_date=expiry,
            strike=170.0,
            option_type="call",
        )
        flows = calculator.calculate_option_cash_flows(position, now + timedelta(days=60))

        assert len(flows) == 1
        # intrinsic = max(0, 180-170) = 10, * 100 * 1 = 1000
        assert abs(flows[0].amount - 1000.0) < 0.01

    def test_put_option_itm(self, calculator, now):
        expiry = now + timedelta(days=30)
        position = Position(
            id="P4",
            symbol="AAPL",
            instrument_type="option",
            quantity=1.0,
            currency="USD",
            current_price=160.0,
            expiration_date=expiry,
            strike=170.0,
            option_type="put",
        )
        flows = calculator.calculate_option_cash_flows(position, now + timedelta(days=60))

        assert len(flows) == 1
        # intrinsic = max(0, 170-160) = 10, * 100 * 1 = 1000
        assert abs(flows[0].amount - 1000.0) < 0.01

    def test_option_otm(self, calculator, now):
        expiry = now + timedelta(days=30)
        position = Position(
            id="P5",
            symbol="AAPL",
            instrument_type="option",
            quantity=1.0,
            currency="USD",
            current_price=160.0,
            expiration_date=expiry,
            strike=170.0,
            option_type="call",
        )
        flows = calculator.calculate_option_cash_flows(position, now + timedelta(days=60))

        assert len(flows) == 1
        assert abs(flows[0].amount) < 0.01

    def test_option_past_horizon(self, calculator, now):
        expiry = now + timedelta(days=400)
        position = Position(
            id="P6",
            symbol="AAPL",
            instrument_type="option",
            quantity=1.0,
            currency="USD",
            expiration_date=expiry,
            strike=170.0,
            option_type="call",
        )
        flows = calculator.calculate_option_cash_flows(position, now + timedelta(days=60))
        assert len(flows) == 0

    def test_option_no_expiry(self, calculator, now):
        position = Position(
            id="P7",
            symbol="AAPL",
            instrument_type="option",
            quantity=1.0,
            currency="USD",
            expiration_date=None,
        )
        flows = calculator.calculate_option_cash_flows(position, now + timedelta(days=60))
        assert len(flows) == 0


class TestBondCashFlows:
    def test_bond_coupon_semi_annual(self, calculator, now):
        position = Position(
            id="B1",
            symbol="US10Y",
            instrument_type="bond",
            quantity=10.0,
            currency="USD",
            current_price=100.0,
            coupon_rate=0.04,
            payment_frequency=2,
            maturity_date=now + timedelta(days=500),
        )
        flows = calculator.calculate_bond_cash_flows(position, now + timedelta(days=365))

        coupon_flows = [f for f in flows if f.cash_flow_type == CashFlowType.BOND_COUPON]
        assert len(coupon_flows) >= 1
        assert all(f.amount > 0 for f in coupon_flows)

    def test_bond_maturity(self, calculator, now):
        maturity = now + timedelta(days=180)
        position = Position(
            id="B2",
            symbol="US5Y",
            instrument_type="bond",
            quantity=10.0,
            currency="USD",
            current_price=100.0,
            coupon_rate=0.03,
            payment_frequency=2,
            maturity_date=maturity,
        )
        flows = calculator.calculate_bond_cash_flows(position, now + timedelta(days=365))

        maturity_flows = [f for f in flows if f.cash_flow_type == CashFlowType.BOND_MATURITY]
        assert len(maturity_flows) == 1
        assert maturity_flows[0].amount > 0

    def test_bond_no_coupon(self, calculator, now):
        position = Position(
            id="B3",
            symbol="TBILL",
            instrument_type="bond",
            quantity=10.0,
            currency="USD",
            current_price=99.5,
            coupon_rate=None,
            maturity_date=now + timedelta(days=90),
        )
        flows = calculator.calculate_bond_cash_flows(position, now + timedelta(days=365))

        coupon_flows = [f for f in flows if f.cash_flow_type == CashFlowType.BOND_COUPON]
        maturity_flows = [f for f in flows if f.cash_flow_type == CashFlowType.BOND_MATURITY]
        assert len(coupon_flows) == 0
        assert len(maturity_flows) == 1

    def test_bond_maturity_beyond_horizon(self, calculator, now):
        position = Position(
            id="B4",
            symbol="US30Y",
            instrument_type="bond",
            quantity=10.0,
            currency="USD",
            current_price=100.0,
            coupon_rate=0.04,
            maturity_date=now + timedelta(days=5000),
        )
        flows = calculator.calculate_bond_cash_flows(position, now + timedelta(days=365))

        maturity_flows = [f for f in flows if f.cash_flow_type == CashFlowType.BOND_MATURITY]
        assert len(maturity_flows) == 0


class TestCPILinkedLoanAdjustment:
    def test_cpi_adjustment_increases_payment(self, now):
        calc = CashFlowCalculator(
            cpi_index_current=120.0,
            cpi_index_base=100.0,
        )
        loan = Loan(
            id="CPI1",
            loan_type="CPI_LINKED",
            principal=800_000.0,
            currency="ILS",
            fixed_rate=0.036,
            cpi_linked=True,
        )
        end_date = now + timedelta(days=60)
        flows = calc.calculate_loan_cash_flows(loan, end_date)
        assert len(flows) > 0
        adjusted_principal = 800_000.0 * (120.0 / 100.0)
        expected_payment = adjusted_principal * 0.036 / 12.0
        assert abs(abs(flows[0].amount) - expected_payment) < 0.01

    def test_no_cpi_data_uses_original_principal(self, calculator, now):
        loan = Loan(
            id="CPI2",
            loan_type="CPI_LINKED",
            principal=800_000.0,
            currency="ILS",
            fixed_rate=0.036,
            cpi_linked=True,
        )
        end_date = now + timedelta(days=60)
        flows = calculator.calculate_loan_cash_flows(loan, end_date)
        expected = 800_000.0 * 0.036 / 12.0
        assert abs(abs(flows[0].amount) - expected) < 0.01

    def test_cpi_not_linked_ignores_index(self, now):
        calc = CashFlowCalculator(
            cpi_index_current=150.0,
            cpi_index_base=100.0,
        )
        loan = Loan(
            id="CPI3",
            loan_type="CPI_LINKED",
            principal=500_000.0,
            currency="ILS",
            fixed_rate=0.04,
            cpi_linked=False,
        )
        flows = calc.calculate_loan_cash_flows(loan, now + timedelta(days=60))
        expected = 500_000.0 * 0.04 / 12.0
        assert abs(abs(flows[0].amount) - expected) < 0.01

    def test_cpi_base_zero_skips_adjustment(self, now):
        calc = CashFlowCalculator(
            cpi_index_current=120.0,
            cpi_index_base=0.0,
        )
        loan = Loan(
            id="CPI4",
            loan_type="CPI_LINKED",
            principal=500_000.0,
            currency="ILS",
            fixed_rate=0.04,
            cpi_linked=True,
        )
        flows = calc.calculate_loan_cash_flows(loan, now + timedelta(days=60))
        expected = 500_000.0 * 0.04 / 12.0
        assert abs(abs(flows[0].amount) - expected) < 0.01


class TestDividendCashFlows:
    def test_no_orats_client(self, calculator, now):
        position = Position(
            id="S1",
            symbol="AAPL",
            instrument_type="stock",
            quantity=100.0,
            currency="USD",
        )
        flows = calculator.calculate_dividend_cash_flows(position, now + timedelta(days=365))
        assert len(flows) == 0

    def test_orats_dividend_schedule(self, now):
        next_ex = (now + timedelta(days=30)).strftime("%Y-%m-%d")

        class MockORATS:
            def get_dividend_schedule(self, symbol):
                return {
                    "ticker": symbol,
                    "next_div_ex_date": next_ex,
                    "next_div_amount": 0.24,
                    "div_frequency": 4,
                    "div_yield": 0.005,
                }

        calc = CashFlowCalculator(orats_client=MockORATS())
        position = Position(
            id="S2",
            symbol="AAPL",
            instrument_type="stock",
            quantity=100.0,
            currency="USD",
        )
        flows = calc.calculate_dividend_cash_flows(position, now + timedelta(days=365))

        assert len(flows) >= 3
        assert all(f.cash_flow_type == CashFlowType.DIVIDEND for f in flows)
        assert all(f.amount == 0.24 * 100 for f in flows)

    def test_orats_no_dividend_amount(self, now):
        class MockORATS:
            def get_dividend_schedule(self, symbol):
                return {"ticker": symbol, "next_div_amount": 0}

        calc = CashFlowCalculator(orats_client=MockORATS())
        position = Position(
            id="S3",
            symbol="GOOG",
            instrument_type="stock",
            quantity=50.0,
            currency="USD",
        )
        flows = calc.calculate_dividend_cash_flows(position, now + timedelta(days=365))
        assert len(flows) == 0

    def test_orats_api_failure_returns_empty(self, now):
        class FailingORATS:
            def get_dividend_schedule(self, symbol):
                raise ConnectionError("API down")

        calc = CashFlowCalculator(orats_client=FailingORATS())
        position = Position(
            id="S4",
            symbol="MSFT",
            instrument_type="stock",
            quantity=50.0,
            currency="USD",
        )
        flows = calc.calculate_dividend_cash_flows(position, now + timedelta(days=365))
        assert len(flows) == 0

    def test_orats_none_schedule(self, now):
        class NoneORATS:
            def get_dividend_schedule(self, symbol):
                return None

        calc = CashFlowCalculator(orats_client=NoneORATS())
        position = Position(
            id="S5",
            symbol="TSLA",
            instrument_type="stock",
            quantity=10.0,
            currency="USD",
        )
        flows = calc.calculate_dividend_cash_flows(position, now + timedelta(days=365))
        assert len(flows) == 0


class TestPortfolioCashFlows:
    def test_full_portfolio(self, calculator, now):
        positions = [
            Position(
                id="P1",
                symbol="SPX",
                instrument_type="option",
                quantity=1.0,
                currency="USD",
                expiration_date=now + timedelta(days=30),
                is_box_spread=True,
                box_strike_width=50.0,
            ),
            Position(
                id="B1",
                symbol="US5Y",
                instrument_type="bond",
                quantity=10.0,
                currency="USD",
                current_price=100.0,
                coupon_rate=0.04,
                payment_frequency=2,
                maturity_date=now + timedelta(days=200),
            ),
        ]
        loans = [
            Loan(
                id="L1",
                loan_type="SHIR",
                principal=500_000.0,
                currency="ILS",
                shir_rate=0.045,
                spread=0.01,
            ),
        ]

        timeline = calculator.calculate_portfolio_cash_flows(
            positions=positions,
            loans=loans,
            forecast_horizon_days=365,
        )

        assert isinstance(timeline, CashFlowTimeline)
        assert len(timeline.cash_flows) > 0
        assert timeline.total_inflows > 0
        assert timeline.total_outflows > 0
        assert timeline.currency == "USD"

    def test_currency_conversion(self, calculator, now):
        positions = []
        loans = [
            Loan(
                id="L1",
                loan_type="SHIR",
                principal=1_000_000.0,
                currency="ILS",
                shir_rate=0.045,
                spread=0.015,
            ),
        ]

        timeline = calculator.calculate_portfolio_cash_flows(
            positions=positions,
            loans=loans,
            forecast_horizon_days=60,
        )

        assert all(cf.currency == "USD" for cf in timeline.cash_flows)

    def test_sorted_by_date(self, calculator, now):
        positions = [
            Position(
                id="P1",
                symbol="SPX",
                instrument_type="option",
                quantity=1.0,
                currency="USD",
                expiration_date=now + timedelta(days=90),
                is_box_spread=True,
                box_strike_width=50.0,
            ),
        ]
        loans = [
            Loan(
                id="L1",
                loan_type="SHIR",
                principal=500_000.0,
                currency="ILS",
                shir_rate=0.045,
                spread=0.01,
            ),
        ]

        timeline = calculator.calculate_portfolio_cash_flows(
            positions=positions,
            loans=loans,
            forecast_horizon_days=365,
        )

        dates = [cf.date for cf in timeline.cash_flows]
        assert dates == sorted(dates)

    def test_cumulative_balance(self, calculator, now):
        positions = [
            Position(
                id="P1",
                symbol="SPX",
                instrument_type="option",
                quantity=1.0,
                currency="USD",
                expiration_date=now + timedelta(days=30),
                is_box_spread=True,
                box_strike_width=50.0,
            ),
        ]
        timeline = calculator.calculate_portfolio_cash_flows(
            positions=positions,
            loans=[],
            forecast_horizon_days=60,
        )

        assert len(timeline.cumulative_balance) > 0

    def test_empty_portfolio(self, calculator, now):
        timeline = calculator.calculate_portfolio_cash_flows(
            positions=[],
            loans=[],
            forecast_horizon_days=365,
        )
        assert len(timeline.cash_flows) == 0
        assert timeline.total_inflows == 0
        assert timeline.total_outflows == 0
        assert timeline.net_cash_flow == 0


class TestHelperMethods:
    def test_option_intrinsic_call_itm(self, calculator):
        pos = Position(
            id="X", symbol="X", instrument_type="option",
            quantity=1, currency="USD", strike=100.0, option_type="call",
        )
        assert calculator._calculate_option_intrinsic_value(pos, 110.0) == 10.0

    def test_option_intrinsic_call_otm(self, calculator):
        pos = Position(
            id="X", symbol="X", instrument_type="option",
            quantity=1, currency="USD", strike=100.0, option_type="call",
        )
        assert calculator._calculate_option_intrinsic_value(pos, 90.0) == 0.0

    def test_option_intrinsic_put_itm(self, calculator):
        pos = Position(
            id="X", symbol="X", instrument_type="option",
            quantity=1, currency="USD", strike=100.0, option_type="put",
        )
        assert calculator._calculate_option_intrinsic_value(pos, 90.0) == 10.0

    def test_option_intrinsic_put_otm(self, calculator):
        pos = Position(
            id="X", symbol="X", instrument_type="option",
            quantity=1, currency="USD", strike=100.0, option_type="put",
        )
        assert calculator._calculate_option_intrinsic_value(pos, 110.0) == 0.0

    def test_add_months(self, calculator, now):
        result = calculator._add_months(now, 1)
        assert (result - now).days == 30

        result_3m = calculator._add_months(now, 3)
        assert (result_3m - now).days == 90
