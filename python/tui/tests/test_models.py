"""
Tests for TUI data models

These tests verify that the Python models match the TypeScript types
and can serialize/deserialize correctly.
"""

import json
import pytest

from ..models import (
    Candle, OptionStrike, OptionSeries, SymbolSnapshot,
    PositionSnapshot, TimelineEvent, AccountMetrics, SnapshotPayload,
    Severity, BoxSpreadScenario, BoxSpreadPayload, BoxSpreadSummary
)


def test_candle_serialization():
    """Test Candle serialization/deserialization"""
    candle = Candle(
        open=100.0,
        high=105.0,
        low=99.0,
        close=103.0,
        volume=1000.0,
        entry=100.0,
        updated="2024-01-01T12:00:00Z"
    )

    data = candle.to_dict()
    assert data["open"] == 100.0
    assert data["high"] == 105.0

    restored = Candle.from_dict(data)
    assert restored.open == candle.open
    assert restored.high == candle.high


def test_snapshot_payload_serialization():
    """Test SnapshotPayload serialization/deserialization"""
    snapshot = SnapshotPayload(
        generated_at="2024-01-01T12:00:00Z",
        mode="DRY-RUN",
        strategy="RUNNING",
        account_id="DU123456",
        metrics=AccountMetrics(
            net_liq=100000.0,
            portal_ok=True,
            tws_ok=True
        ),
        symbols=[
            SymbolSnapshot(
                symbol="SPX",
                last=4000.0,
                bid=3999.5,
                ask=4000.5,
                spread=1.0,
                roi=2.5
            )
        ],
        positions=[],
        historic=[],
        orders=[],
        alerts=[]
    )

    # Test JSON serialization
    json_str = snapshot.to_json()
    assert "SPX" in json_str
    assert "DRY-RUN" in json_str

    # Test deserialization
    restored = SnapshotPayload.from_json(json_str)
    assert restored.mode == snapshot.mode
    assert restored.account_id == snapshot.account_id
    assert len(restored.symbols) == 1
    assert restored.symbols[0].symbol == "SPX"


def test_timeline_event_severity():
    """Test TimelineEvent with different severity levels"""
    event = TimelineEvent(
        timestamp="2024-01-01T12:00:00Z",
        text="Test alert",
        severity=Severity.ERROR
    )

    data = event.to_dict()
    assert data["severity"] == "error"

    restored = TimelineEvent.from_dict(data)
    assert restored.severity == Severity.ERROR


def test_empty_snapshot():
    """Test empty snapshot creation"""
    snapshot = SnapshotPayload()
    assert snapshot.mode == "DRY-RUN"
    assert snapshot.strategy == "STOPPED"
    assert len(snapshot.symbols) == 0
    assert len(snapshot.positions) == 0


def test_symbol_snapshot_with_option_chains():
    """Test SymbolSnapshot with option chains"""
    symbol = SymbolSnapshot(
        symbol="SPX",
        last=4000.0,
        option_chains=[
            OptionSeries(
                expiration="2024-01-19",
                strikes=[
                    OptionStrike(
                        strike=4000.0,
                        call_bid=100.0,
                        call_ask=101.0,
                        put_bid=99.0,
                        put_ask=100.0
                    )
                ]
            )
        ]
    )

    data = symbol.to_dict()
    assert len(data["option_chains"]) == 1
    assert len(data["option_chains"][0]["strikes"]) == 1

    restored = SymbolSnapshot.from_dict(data)
    assert len(restored.option_chains) == 1
    assert restored.option_chains[0].expiration == "2024-01-19"


def test_symbol_snapshot_all_fields():
    """Test SymbolSnapshot with all fields populated"""
    candle = Candle(open=100.0, high=105.0, low=99.0, close=103.0, volume=1000.0)
    symbol = SymbolSnapshot(
        symbol="SPX",
        last=4000.0,
        bid=3999.5,
        ask=4000.5,
        spread=1.0,
        roi=2.5,
        maker_count=10,
        taker_count=5,
        volume=10000.0,
        candle=candle
    )

    data = symbol.to_dict()
    assert data["symbol"] == "SPX"
    assert data["last"] == 4000.0
    assert data["bid"] == 3999.5
    assert data["ask"] == 4000.5
    assert data["spread"] == 1.0
    assert data["roi"] == 2.5
    assert data["maker_count"] == 10
    assert data["taker_count"] == 5
    assert data["volume"] == 10000.0
    assert data["candle"]["open"] == 100.0

    restored = SymbolSnapshot.from_dict(data)
    assert restored.symbol == "SPX"
    assert restored.maker_count == 10
    assert restored.candle.open == 100.0


def test_position_snapshot_basic():
    """Test PositionSnapshot with basic fields"""
    candle = Candle(open=150.0, high=155.0, low=149.0, close=153.0)
    position = PositionSnapshot(
        name="AAPL",
        quantity=100,
        roi=2.0,
        maker_count=5,
        taker_count=3,
        rebate_estimate=10.0,
        vega=0.5,
        theta=-0.1,
        fair_diff=0.5,
        candle=candle
    )

    data = position.to_dict()
    assert data["name"] == "AAPL"
    assert data["quantity"] == 100
    assert data["roi"] == 2.0
    assert data["vega"] == 0.5
    assert data["theta"] == -0.1

    restored = PositionSnapshot.from_dict(data)
    assert restored.name == "AAPL"
    assert restored.quantity == 100
    assert restored.candle.open == 150.0


def test_position_snapshot_extended_fields():
    """Test PositionSnapshot with extended fields (instrument_type, rate, etc.)"""
    position = PositionSnapshot(
        name="Bond ETF",
        quantity=1000,
        instrument_type="bond",
        rate=4.5,
        maturity_date="2026-12-31",
        cash_flow=45000.0,
        collateral_value=100000.0,
        currency="USD"
    )

    data = position.to_dict()
    assert data["name"] == "Bond ETF"
    assert data["instrument_type"] == "bond"
    assert data["rate"] == 4.5
    assert data["maturity_date"] == "2026-12-31"
    assert data["cash_flow"] == 45000.0
    assert data["collateral_value"] == 100000.0
    assert data["currency"] == "USD"

    restored = PositionSnapshot.from_dict(data)
    assert restored.instrument_type == "bond"
    assert restored.rate == 4.5
    assert restored.maturity_date == "2026-12-31"


def test_position_snapshot_optional_fields_none():
    """Test PositionSnapshot with None optional fields (should not be in dict)"""
    position = PositionSnapshot(
        name="AAPL",
        quantity=100,
        instrument_type=None,
        rate=None
    )

    data = position.to_dict()
    assert "name" in data
    assert "instrument_type" not in data  # None fields should be omitted
    assert "rate" not in data


def test_account_metrics_all_fields():
    """Test AccountMetrics with all fields"""
    metrics = AccountMetrics(
        net_liq=100000.0,
        buying_power=50000.0,
        excess_liquidity=25000.0,
        margin_requirement=5000.0,
        commissions=100.0,
        portal_ok=True,
        tws_ok=True,
        questdb_ok=True
    )

    data = metrics.to_dict()
    assert data["net_liq"] == 100000.0
    assert data["buying_power"] == 50000.0
    assert data["excess_liquidity"] == 25000.0
    assert data["margin_requirement"] == 5000.0
    assert data["commissions"] == 100.0
    assert data["portal_ok"] is True
    assert data["tws_ok"] is True
    assert data["questdb_ok"] is True

    restored = AccountMetrics.from_dict(data)
    assert restored.net_liq == 100000.0
    assert restored.portal_ok is True
    assert restored.questdb_ok is True


def test_timeline_event_all_severities():
    """Test TimelineEvent with all severity levels"""
    severities = [Severity.INFO, Severity.SUCCESS, Severity.WARN, Severity.WARNING, Severity.ERROR, Severity.CRITICAL]

    for severity in severities:
        event = TimelineEvent(
            timestamp="2024-01-01T12:00:00Z",
            text=f"Test {severity.value}",
            severity=severity
        )

        data = event.to_dict()
        assert data["severity"] == severity.value

        restored = TimelineEvent.from_dict(data)
        assert restored.severity == severity


def test_timeline_event_invalid_severity():
    """Test TimelineEvent with invalid severity (should default to INFO)"""
    data = {"timestamp": "2024-01-01T12:00:00Z", "text": "Test", "severity": "invalid"}
    event = TimelineEvent.from_dict(data)
    assert event.severity == Severity.INFO


def test_option_strike_serialization():
    """Test OptionStrike serialization/deserialization"""
    strike = OptionStrike(
        strike=4000.0,
        call_bid=100.0,
        call_ask=101.0,
        put_bid=99.0,
        put_ask=100.0
    )

    data = strike.to_dict()
    assert data["strike"] == 4000.0
    assert data["call_bid"] == 100.0
    assert data["put_ask"] == 100.0

    restored = OptionStrike.from_dict(data)
    assert restored.strike == 4000.0
    assert restored.call_bid == 100.0


def test_option_series_serialization():
    """Test OptionSeries serialization/deserialization"""
    series = OptionSeries(
        expiration="2024-01-19",
        strikes=[
            OptionStrike(strike=4000.0, call_bid=100.0, call_ask=101.0, put_bid=99.0, put_ask=100.0),
            OptionStrike(strike=4050.0, call_bid=50.0, call_ask=51.0, put_bid=49.0, put_ask=50.0)
        ]
    )

    data = series.to_dict()
    assert data["expiration"] == "2024-01-19"
    assert len(data["strikes"]) == 2

    restored = OptionSeries.from_dict(data)
    assert restored.expiration == "2024-01-19"
    assert len(restored.strikes) == 2
    assert restored.strikes[0].strike == 4000.0
    assert restored.strikes[1].strike == 4050.0


def test_box_spread_scenario_basic():
    """Test BoxSpreadScenario with basic fields"""
    scenario = BoxSpreadScenario(
        width=50.0,
        put_bid=25.0,
        call_ask=25.5,
        synthetic_bid=50.0,
        synthetic_ask=50.5,
        mid_price=50.25,
        annualized_return=5.0,
        fill_probability=0.8,
        option_style="European"
    )

    data = scenario.to_dict()
    assert data["width"] == 50.0
    assert data["annualized_return"] == 5.0
    assert data["fill_probability"] == 0.8
    assert data["option_style"] == "European"

    restored = BoxSpreadScenario.from_dict(data)
    assert restored.width == 50.0
    assert restored.option_style == "European"


def test_box_spread_scenario_optional_fields():
    """Test BoxSpreadScenario with optional fields"""
    scenario = BoxSpreadScenario(
        width=50.0,
        put_bid=25.0,
        call_ask=25.5,
        synthetic_bid=50.0,
        synthetic_ask=50.5,
        mid_price=50.25,
        annualized_return=5.0,
        fill_probability=0.8,
        buy_profit=100.0,
        buy_implied_rate=4.5,
        sell_profit=150.0,
        sell_implied_rate=5.5,
        buy_sell_disparity=50.0,
        put_call_parity_violation=0.1,
        expiration_date="2024-12-20",
        days_to_expiry=30
    )

    data = scenario.to_dict()
    assert data["buy_profit"] == 100.0
    assert data["buy_implied_rate"] == 4.5
    assert data["expiration_date"] == "2024-12-20"
    assert data["days_to_expiry"] == 30

    restored = BoxSpreadScenario.from_dict(data)
    assert restored.buy_profit == 100.0
    assert restored.days_to_expiry == 30


def test_box_spread_payload():
    """Test BoxSpreadPayload serialization/deserialization"""
    scenarios = [
        BoxSpreadScenario(width=50.0, annualized_return=5.0, fill_probability=0.8, option_style="European"),
        BoxSpreadScenario(width=100.0, annualized_return=4.5, fill_probability=0.7, option_style="European")
    ]

    payload = BoxSpreadPayload(
        as_of="2024-01-01T12:00:00Z",
        underlying="SPX",
        scenarios=scenarios
    )

    data = payload.to_dict()
    assert data["as_of"] == "2024-01-01T12:00:00Z"
    assert data["underlying"] == "SPX"
    assert len(data["scenarios"]) == 2

    restored = BoxSpreadPayload.from_dict(data)
    assert restored.as_of == "2024-01-01T12:00:00Z"
    assert restored.underlying == "SPX"
    assert len(restored.scenarios) == 2

    # Test JSON serialization
    json_str = json.dumps(payload.to_dict())
    assert "SPX" in json_str
    assert "2024-01-01T12:00:00Z" in json_str

    restored_from_json = BoxSpreadPayload.from_json(json_str)
    assert restored_from_json.underlying == "SPX"


def test_box_spread_summary_calculate():
    """Test BoxSpreadSummary.calculate() method"""
    scenarios = [
        BoxSpreadScenario(width=50.0, annualized_return=5.0, fill_probability=0.8, option_style="European"),
        BoxSpreadScenario(width=100.0, annualized_return=4.0, fill_probability=0.7, option_style="European"),
        BoxSpreadScenario(width=150.0, annualized_return=6.0, fill_probability=0.9, option_style="European")
    ]

    payload = BoxSpreadPayload(
        as_of="2024-01-01T12:00:00Z",
        underlying="SPX",
        scenarios=scenarios
    )

    summary = BoxSpreadSummary.calculate(payload)
    assert summary.total_scenarios == 3
    assert summary.avg_apr == pytest.approx(5.0, rel=0.01)  # (5.0 + 4.0 + 6.0) / 3 = 5.0
    assert summary.probable_count == 3  # All have fill_probability > 0
    assert summary.max_apr_scenario is not None
    assert summary.max_apr_scenario.annualized_return == 6.0


def test_box_spread_summary_calculate_empty():
    """Test BoxSpreadSummary.calculate() with empty payload"""
    payload = BoxSpreadPayload(as_of="2024-01-01T12:00:00Z", underlying="SPX", scenarios=[])
    summary = BoxSpreadSummary.calculate(payload)

    assert summary.total_scenarios == 0
    assert summary.avg_apr == 0.0
    assert summary.probable_count == 0
    assert summary.max_apr_scenario is None


def test_box_spread_summary_calculate_mixed_styles():
    """Test BoxSpreadSummary.calculate() with mixed European/American styles"""
    scenarios = [
        BoxSpreadScenario(width=50.0, annualized_return=5.0, fill_probability=0.8, option_style="European"),
        BoxSpreadScenario(width=100.0, annualized_return=4.0, fill_probability=0.7, option_style="American"),
        BoxSpreadScenario(width=150.0, annualized_return=6.0, fill_probability=0.9, option_style="European")
    ]

    payload = BoxSpreadPayload(
        as_of="2024-01-01T12:00:00Z",
        underlying="SPX",
        scenarios=scenarios
    )

    summary = BoxSpreadSummary.calculate(payload)
    # Should only count European scenarios
    assert summary.total_scenarios == 2
    assert summary.avg_apr == pytest.approx(5.5, rel=0.01)  # (5.0 + 6.0) / 2 = 5.5


def test_box_spread_summary_serialization():
    """Test BoxSpreadSummary serialization/deserialization"""
    scenario = BoxSpreadScenario(width=50.0, annualized_return=5.0, fill_probability=0.8)
    summary = BoxSpreadSummary(
        total_scenarios=10,
        avg_apr=4.5,
        probable_count=8,
        max_apr_scenario=scenario
    )

    data = summary.to_dict()
    assert data["total_scenarios"] == 10
    assert data["avg_apr"] == 4.5
    assert data["probable_count"] == 8
    assert data["max_apr_scenario"] is not None
    assert data["max_apr_scenario"]["annualized_return"] == 5.0

    restored = BoxSpreadSummary.from_dict(data)
    assert restored.total_scenarios == 10
    assert restored.max_apr_scenario is not None
    assert restored.max_apr_scenario.annualized_return == 5.0


def test_box_spread_summary_no_max_scenario():
    """Test BoxSpreadSummary with None max_apr_scenario"""
    summary = BoxSpreadSummary(
        total_scenarios=0,
        avg_apr=0.0,
        probable_count=0,
        max_apr_scenario=None
    )

    data = summary.to_dict()
    assert data["max_apr_scenario"] is None

    restored = BoxSpreadSummary.from_dict(data)
    assert restored.max_apr_scenario is None


def test_snapshot_payload_with_positions():
    """Test SnapshotPayload with positions"""
    position = PositionSnapshot(name="AAPL", quantity=100, roi=2.0)
    snapshot = SnapshotPayload(
        generated_at="2024-01-01T12:00:00Z",
        mode="LIVE",
        strategy="RUNNING",
        account_id="DU123456",
        positions=[position]
    )

    data = snapshot.to_dict()
    assert len(data["positions"]) == 1
    assert data["positions"][0]["name"] == "AAPL"

    restored = SnapshotPayload.from_dict(data)
    assert len(restored.positions) == 1
    assert restored.positions[0].name == "AAPL"


def test_snapshot_payload_with_orders_and_alerts():
    """Test SnapshotPayload with orders and alerts"""
    order = TimelineEvent(timestamp="2024-01-01T12:00:00Z", text="Order filled", severity=Severity.SUCCESS)
    alert = TimelineEvent(timestamp="2024-01-01T12:01:00Z", text="Warning", severity=Severity.WARN)

    snapshot = SnapshotPayload(
        generated_at="2024-01-01T12:00:00Z",
        orders=[order],
        alerts=[alert]
    )

    data = snapshot.to_dict()
    assert len(data["orders"]) == 1
    assert len(data["alerts"]) == 1
    assert data["orders"][0]["text"] == "Order filled"
    assert data["alerts"][0]["severity"] == "warn"

    restored = SnapshotPayload.from_dict(data)
    assert len(restored.orders) == 1
    assert len(restored.alerts) == 1
    assert restored.orders[0].severity == Severity.SUCCESS
    assert restored.alerts[0].severity == Severity.WARN


def test_candle_edge_cases():
    """Test Candle with edge case values"""
    # Zero values
    candle = Candle(open=0.0, high=0.0, low=0.0, close=0.0, volume=0.0)
    data = candle.to_dict()
    assert data["open"] == 0.0
    assert data["volume"] == 0.0

    # Negative values (allowed for some fields)
    candle = Candle(open=100.0, high=105.0, low=95.0, close=98.0, entry=100.0)
    data = candle.to_dict()
    restored = Candle.from_dict(data)
    assert restored.close == 98.0
