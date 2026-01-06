"""
Tests for TUI models module.

Tests all dataclasses and their serialization/deserialization methods.
"""
import unittest
import json
from datetime import datetime

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent))

from tui.models import (
    Severity,
    Candle,
    OptionStrike,
    OptionSeries,
    SymbolSnapshot,
    PositionSnapshot,
    TimelineEvent,
    AccountMetrics,
    SnapshotPayload,
    BoxSpreadScenario,
    BoxSpreadPayload,
    BoxSpreadSummary,
)


class TestSeverity(unittest.TestCase):
    """Tests for Severity enum."""

    def test_severity_values(self):
        """Test Severity enum values."""
        assert Severity.INFO.value == "info"
        assert Severity.SUCCESS.value == "success"
        assert Severity.WARN.value == "warn"
        assert Severity.WARNING.value == "warning"
        assert Severity.ERROR.value == "error"
        assert Severity.CRITICAL.value == "critical"

    def test_severity_from_string(self):
        """Test creating Severity from string."""
        assert Severity("info") == Severity.INFO
        assert Severity("error") == Severity.ERROR


class TestCandle(unittest.TestCase):
    """Tests for Candle dataclass."""

    def test_candle_default(self):
        """Test Candle with default values."""
        candle = Candle()
        assert candle.open == 0.0
        assert candle.high == 0.0
        assert candle.low == 0.0
        assert candle.close == 0.0
        assert candle.volume == 0.0
        assert candle.entry == 0.0
        assert candle.updated == ""

    def test_candle_custom(self):
        """Test Candle with custom values."""
        candle = Candle(
            open=100.0,
            high=105.0,
            low=99.0,
            close=103.0,
            volume=1000000.0,
            entry=102.0,
            updated="2025-01-01T00:00:00Z"
        )
        assert candle.open == 100.0
        assert candle.high == 105.0
        assert candle.low == 99.0
        assert candle.close == 103.0

    def test_candle_to_dict(self):
        """Test Candle.to_dict() method."""
        candle = Candle(open=100.0, high=105.0, close=103.0)
        data = candle.to_dict()
        assert data["open"] == 100.0
        assert data["high"] == 105.0
        assert data["close"] == 103.0

    def test_candle_from_dict(self):
        """Test Candle.from_dict() method."""
        data = {
            "open": 100.0,
            "high": 105.0,
            "low": 99.0,
            "close": 103.0,
            "volume": 1000000.0,
            "entry": 102.0,
            "updated": "2025-01-01T00:00:00Z"
        }
        candle = Candle.from_dict(data)
        assert candle.open == 100.0
        assert candle.high == 105.0
        assert candle.close == 103.0


class TestOptionStrike(unittest.TestCase):
    """Tests for OptionStrike dataclass."""

    def test_option_strike_default(self):
        """Test OptionStrike with default values."""
        strike = OptionStrike()
        assert strike.strike == 0.0
        assert strike.call_bid == 0.0
        assert strike.call_ask == 0.0
        assert strike.put_bid == 0.0
        assert strike.put_ask == 0.0

    def test_option_strike_to_dict(self):
        """Test OptionStrike.to_dict() method."""
        strike = OptionStrike(strike=450.0, call_bid=5.0, call_ask=5.5)
        data = strike.to_dict()
        assert data["strike"] == 450.0
        assert data["call_bid"] == 5.0
        assert data["call_ask"] == 5.5

    def test_option_strike_from_dict(self):
        """Test OptionStrike.from_dict() method."""
        data = {
            "strike": 450.0,
            "call_bid": 5.0,
            "call_ask": 5.5,
            "put_bid": 3.0,
            "put_ask": 3.5
        }
        strike = OptionStrike.from_dict(data)
        assert strike.strike == 450.0
        assert strike.call_bid == 5.0
        assert strike.put_ask == 3.5


class TestOptionSeries(unittest.TestCase):
    """Tests for OptionSeries dataclass."""

    def test_option_series_default(self):
        """Test OptionSeries with default values."""
        series = OptionSeries()
        assert series.expiration == ""
        assert len(series.strikes) == 0

    def test_option_series_with_strikes(self):
        """Test OptionSeries with strikes."""
        strikes = [
            OptionStrike(strike=450.0, call_bid=5.0, call_ask=5.5),
            OptionStrike(strike=455.0, call_bid=2.0, call_ask=2.5),
        ]
        series = OptionSeries(expiration="2024-12-20", strikes=strikes)
        assert series.expiration == "2024-12-20"
        assert len(series.strikes) == 2

    def test_option_series_to_dict(self):
        """Test OptionSeries.to_dict() method."""
        strikes = [OptionStrike(strike=450.0)]
        series = OptionSeries(expiration="2024-12-20", strikes=strikes)
        data = series.to_dict()
        assert data["expiration"] == "2024-12-20"
        assert len(data["strikes"]) == 1

    def test_option_series_from_dict(self):
        """Test OptionSeries.from_dict() method."""
        data = {
            "expiration": "2024-12-20",
            "strikes": [
                {"strike": 450.0, "call_bid": 5.0, "call_ask": 5.5}
            ]
        }
        series = OptionSeries.from_dict(data)
        assert series.expiration == "2024-12-20"
        assert len(series.strikes) == 1
        assert series.strikes[0].strike == 450.0


class TestSymbolSnapshot(unittest.TestCase):
    """Tests for SymbolSnapshot dataclass."""

    def test_symbol_snapshot_default(self):
        """Test SymbolSnapshot with default values."""
        snapshot = SymbolSnapshot()
        assert snapshot.symbol == ""
        assert snapshot.last == 0.0
        assert snapshot.bid == 0.0
        assert snapshot.ask == 0.0
        assert isinstance(snapshot.candle, Candle)

    def test_symbol_snapshot_to_dict(self):
        """Test SymbolSnapshot.to_dict() method."""
        snapshot = SymbolSnapshot(symbol="SPY", last=450.0, bid=449.5, ask=450.5)
        data = snapshot.to_dict()
        assert data["symbol"] == "SPY"
        assert data["last"] == 450.0
        assert "candle" in data

    def test_symbol_snapshot_from_dict(self):
        """Test SymbolSnapshot.from_dict() method."""
        data = {
            "symbol": "SPY",
            "last": 450.0,
            "bid": 449.5,
            "ask": 450.5,
            "candle": {"open": 449.0, "close": 450.0}
        }
        snapshot = SymbolSnapshot.from_dict(data)
        assert snapshot.symbol == "SPY"
        assert snapshot.last == 450.0
        assert snapshot.candle.close == 450.0


class TestPositionSnapshot(unittest.TestCase):
    """Tests for PositionSnapshot dataclass."""

    def test_position_snapshot_default(self):
        """Test PositionSnapshot with default values."""
        position = PositionSnapshot()
        assert position.name == ""
        assert position.quantity == 0
        assert position.roi == 0.0
        assert position.instrument_type is None

    def test_position_snapshot_extended_fields(self):
        """Test PositionSnapshot with extended fields."""
        position = PositionSnapshot(
            name="Loan-001",
            instrument_type="bank_loan",
            rate=3.5,
            maturity_date="2030-01-01",
            cash_flow=-4500.0,
            currency="ILS"
        )
        assert position.instrument_type == "bank_loan"
        assert position.rate == 3.5
        assert position.currency == "ILS"

    def test_position_snapshot_to_dict(self):
        """Test PositionSnapshot.to_dict() method."""
        position = PositionSnapshot(
            name="SPY-Box",
            quantity=1,
            roi=5.0,
            instrument_type="box_spread"
        )
        data = position.to_dict()
        assert data["name"] == "SPY-Box"
        assert data["quantity"] == 1
        assert data["instrument_type"] == "box_spread"

    def test_position_snapshot_to_dict_omits_none(self):
        """Test PositionSnapshot.to_dict() omits None extended fields."""
        position = PositionSnapshot(name="Test")
        data = position.to_dict()
        assert "instrument_type" not in data
        assert "rate" not in data

    def test_position_snapshot_from_dict(self):
        """Test PositionSnapshot.from_dict() method."""
        data = {
            "name": "SPY-Box",
            "quantity": 1,
            "roi": 5.0,
            "instrument_type": "box_spread",
            "rate": 4.5,
            "candle": {"close": 100.0}
        }
        position = PositionSnapshot.from_dict(data)
        assert position.name == "SPY-Box"
        assert position.instrument_type == "box_spread"
        assert position.rate == 4.5


class TestTimelineEvent(unittest.TestCase):
    """Tests for TimelineEvent dataclass."""

    def test_timeline_event_default(self):
        """Test TimelineEvent with default values."""
        event = TimelineEvent()
        assert event.timestamp == ""
        assert event.text == ""
        assert event.severity == Severity.INFO

    def test_timeline_event_to_dict(self):
        """Test TimelineEvent.to_dict() method."""
        event = TimelineEvent(
            timestamp="2025-01-01T00:00:00Z",
            text="Order filled",
            severity=Severity.SUCCESS
        )
        data = event.to_dict()
        assert data["timestamp"] == "2025-01-01T00:00:00Z"
        assert data["text"] == "Order filled"
        assert data["severity"] == "success"

    def test_timeline_event_from_dict(self):
        """Test TimelineEvent.from_dict() method."""
        data = {
            "timestamp": "2025-01-01T00:00:00Z",
            "text": "Error occurred",
            "severity": "error"
        }
        event = TimelineEvent.from_dict(data)
        assert event.text == "Error occurred"
        assert event.severity == Severity.ERROR

    def test_timeline_event_from_dict_invalid_severity(self):
        """Test TimelineEvent.from_dict() with invalid severity."""
        data = {
            "timestamp": "2025-01-01T00:00:00Z",
            "text": "Test",
            "severity": "invalid"
        }
        event = TimelineEvent.from_dict(data)
        assert event.severity == Severity.INFO  # Defaults to INFO


class TestAccountMetrics(unittest.TestCase):
    """Tests for AccountMetrics dataclass."""

    def test_account_metrics_default(self):
        """Test AccountMetrics with default values."""
        metrics = AccountMetrics()
        assert metrics.net_liq == 0.0
        assert metrics.buying_power == 0.0
        assert metrics.portal_ok is False

    def test_account_metrics_to_dict(self):
        """Test AccountMetrics.to_dict() method."""
        metrics = AccountMetrics(net_liq=100000.0, buying_power=50000.0, portal_ok=True)
        data = metrics.to_dict()
        assert data["net_liq"] == 100000.0
        assert data["portal_ok"] is True

    def test_account_metrics_from_dict(self):
        """Test AccountMetrics.from_dict() method."""
        data = {
            "net_liq": 100000.0,
            "buying_power": 50000.0,
            "excess_liquidity": 30000.0,
            "portal_ok": True
        }
        metrics = AccountMetrics.from_dict(data)
        assert metrics.net_liq == 100000.0
        assert metrics.portal_ok is True


class TestSnapshotPayload(unittest.TestCase):
    """Tests for SnapshotPayload dataclass."""

    def test_snapshot_payload_default(self):
        """Test SnapshotPayload with default values."""
        payload = SnapshotPayload()
        assert payload.generated_at == ""
        assert payload.mode == "DRY-RUN"
        assert payload.strategy == "STOPPED"
        assert len(payload.symbols) == 0
        assert len(payload.positions) == 0

    def test_snapshot_payload_to_dict(self):
        """Test SnapshotPayload.to_dict() method."""
        payload = SnapshotPayload(
            generated_at="2025-01-01T00:00:00Z",
            mode="LIVE",
            strategy="RUNNING",
            account_id="DU123456"
        )
        data = payload.to_dict()
        assert data["mode"] == "LIVE"
        assert data["strategy"] == "RUNNING"
        assert "symbols" in data
        assert "positions" in data

    def test_snapshot_payload_to_json(self):
        """Test SnapshotPayload.to_json() method."""
        payload = SnapshotPayload(generated_at="2025-01-01T00:00:00Z")
        json_str = payload.to_json()
        assert isinstance(json_str, str)
        data = json.loads(json_str)
        assert data["generated_at"] == "2025-01-01T00:00:00Z"

    def test_snapshot_payload_from_dict(self):
        """Test SnapshotPayload.from_dict() method."""
        data = {
            "generated_at": "2025-01-01T00:00:00Z",
            "mode": "LIVE",
            "strategy": "RUNNING",
            "account_id": "DU123456",
            "symbols": [{"symbol": "SPY", "last": 450.0}],
            "positions": [{"name": "SPY-Box", "quantity": 1}],
            "metrics": {"net_liq": 100000.0}
        }
        payload = SnapshotPayload.from_dict(data)
        assert payload.mode == "LIVE"
        assert len(payload.symbols) == 1
        assert payload.symbols[0].symbol == "SPY"
        assert len(payload.positions) == 1

    def test_snapshot_payload_from_json(self):
        """Test SnapshotPayload.from_json() method."""
        json_str = json.dumps({
            "generated_at": "2025-01-01T00:00:00Z",
            "mode": "LIVE",
            "symbols": [],
            "positions": []
        })
        payload = SnapshotPayload.from_json(json_str)
        assert payload.mode == "LIVE"


class TestBoxSpreadScenario(unittest.TestCase):
    """Tests for BoxSpreadScenario dataclass."""

    def test_box_spread_scenario_default(self):
        """Test BoxSpreadScenario with default values."""
        scenario = BoxSpreadScenario()
        assert scenario.width == 0.0
        assert scenario.annualized_return == 0.0
        assert scenario.option_style == "European"

    def test_box_spread_scenario_optional_fields(self):
        """Test BoxSpreadScenario with optional fields."""
        scenario = BoxSpreadScenario(
            width=5.0,
            buy_profit=10.0,
            sell_profit=8.0,
            expiration_date="2024-12-20"
        )
        assert scenario.buy_profit == 10.0
        assert scenario.expiration_date == "2024-12-20"

    def test_box_spread_scenario_to_dict(self):
        """Test BoxSpreadScenario.to_dict() method."""
        scenario = BoxSpreadScenario(width=5.0, annualized_return=4.5)
        data = scenario.to_dict()
        assert data["width"] == 5.0
        assert data["annualized_return"] == 4.5

    def test_box_spread_scenario_from_dict(self):
        """Test BoxSpreadScenario.from_dict() method."""
        data = {
            "width": 5.0,
            "annualized_return": 4.5,
            "fill_probability": 0.8,
            "buy_profit": 10.0
        }
        scenario = BoxSpreadScenario.from_dict(data)
        assert scenario.width == 5.0
        assert scenario.buy_profit == 10.0


class TestBoxSpreadPayload(unittest.TestCase):
    """Tests for BoxSpreadPayload dataclass."""

    def test_box_spread_payload_default(self):
        """Test BoxSpreadPayload with default values."""
        payload = BoxSpreadPayload()
        assert payload.as_of == ""
        assert payload.underlying == ""
        assert len(payload.scenarios) == 0

    def test_box_spread_payload_to_dict(self):
        """Test BoxSpreadPayload.to_dict() method."""
        scenarios = [BoxSpreadScenario(width=5.0), BoxSpreadScenario(width=10.0)]
        payload = BoxSpreadPayload(
            as_of="2025-01-01T00:00:00Z",
            underlying="SPY",
            scenarios=scenarios
        )
        data = payload.to_dict()
        assert data["underlying"] == "SPY"
        assert len(data["scenarios"]) == 2

    def test_box_spread_payload_from_dict(self):
        """Test BoxSpreadPayload.from_dict() method."""
        data = {
            "as_of": "2025-01-01T00:00:00Z",
            "underlying": "SPY",
            "scenarios": [
                {"width": 5.0, "annualized_return": 4.5},
                {"width": 10.0, "annualized_return": 5.0}
            ]
        }
        payload = BoxSpreadPayload.from_dict(data)
        assert payload.underlying == "SPY"
        assert len(payload.scenarios) == 2
        assert payload.scenarios[0].width == 5.0

    def test_box_spread_payload_from_json(self):
        """Test BoxSpreadPayload.from_json() method."""
        json_str = json.dumps({
            "as_of": "2025-01-01T00:00:00Z",
            "underlying": "SPY",
            "scenarios": [{"width": 5.0}]
        })
        payload = BoxSpreadPayload.from_json(json_str)
        assert payload.underlying == "SPY"
        assert len(payload.scenarios) == 1


class TestBoxSpreadSummary(unittest.TestCase):
    """Tests for BoxSpreadSummary dataclass."""

    def test_box_spread_summary_default(self):
        """Test BoxSpreadSummary with default values."""
        summary = BoxSpreadSummary()
        assert summary.total_scenarios == 0
        assert summary.avg_apr == 0.0
        assert summary.probable_count == 0
        assert summary.max_apr_scenario is None

    def test_box_spread_summary_calculate_empty(self):
        """Test BoxSpreadSummary.calculate() with empty payload."""
        payload = BoxSpreadPayload()
        summary = BoxSpreadSummary.calculate(payload)
        assert summary.total_scenarios == 0
        assert summary.avg_apr == 0.0

    def test_box_spread_summary_calculate_european(self):
        """Test BoxSpreadSummary.calculate() with European scenarios."""
        scenarios = [
            BoxSpreadScenario(width=5.0, annualized_return=4.0, fill_probability=0.8, option_style="European"),
            BoxSpreadScenario(width=10.0, annualized_return=5.0, fill_probability=0.9, option_style="European"),
            BoxSpreadScenario(width=15.0, annualized_return=6.0, fill_probability=0.0, option_style="European"),
        ]
        payload = BoxSpreadPayload(scenarios=scenarios)
        summary = BoxSpreadSummary.calculate(payload)

        assert summary.total_scenarios == 3
        assert summary.avg_apr == 5.0  # (4.0 + 5.0 + 6.0) / 3
        assert summary.probable_count == 2  # 2 with fill_probability > 0
        assert summary.max_apr_scenario is not None
        assert summary.max_apr_scenario.annualized_return == 6.0

    def test_box_spread_summary_calculate_mixed_styles(self):
        """Test BoxSpreadSummary.calculate() with mixed European/American scenarios."""
        scenarios = [
            BoxSpreadScenario(width=5.0, annualized_return=4.0, option_style="European"),
            BoxSpreadScenario(width=10.0, annualized_return=5.0, option_style="American"),
        ]
        payload = BoxSpreadPayload(scenarios=scenarios)
        summary = BoxSpreadSummary.calculate(payload)

        # Should only count European scenarios
        assert summary.total_scenarios == 1
        assert summary.avg_apr == 4.0

    def test_box_spread_summary_calculate_no_european(self):
        """Test BoxSpreadSummary.calculate() with no European scenarios."""
        scenarios = [
            BoxSpreadScenario(width=5.0, annualized_return=4.0, option_style="American"),
            BoxSpreadScenario(width=10.0, annualized_return=5.0, option_style="American"),
        ]
        payload = BoxSpreadPayload(scenarios=scenarios)
        summary = BoxSpreadSummary.calculate(payload)

        # Should fall back to all scenarios if no European
        assert summary.total_scenarios == 2
        assert summary.avg_apr == 4.5

    def test_box_spread_summary_to_dict(self):
        """Test BoxSpreadSummary.to_dict() method."""
        scenario = BoxSpreadScenario(width=5.0, annualized_return=4.5)
        summary = BoxSpreadSummary(
            total_scenarios=1,
            avg_apr=4.5,
            probable_count=1,
            max_apr_scenario=scenario
        )
        data = summary.to_dict()
        assert data["total_scenarios"] == 1
        assert data["max_apr_scenario"] is not None

    def test_box_spread_summary_to_dict_no_max_scenario(self):
        """Test BoxSpreadSummary.to_dict() with no max scenario."""
        summary = BoxSpreadSummary(total_scenarios=0)
        data = summary.to_dict()
        assert data["max_apr_scenario"] is None

    def test_box_spread_summary_from_dict(self):
        """Test BoxSpreadSummary.from_dict() method."""
        data = {
            "total_scenarios": 2,
            "avg_apr": 4.5,
            "probable_count": 1,
            "max_apr_scenario": {"width": 5.0, "annualized_return": 5.0}
        }
        summary = BoxSpreadSummary.from_dict(data)
        assert summary.total_scenarios == 2
        assert summary.max_apr_scenario is not None
        assert summary.max_apr_scenario.width == 5.0


if __name__ == "__main__":
    unittest.main()
