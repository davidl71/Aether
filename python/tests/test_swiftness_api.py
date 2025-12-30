"""
Tests for Swiftness API service.

Tests FastAPI endpoints, Pydantic models, and Swiftness integration.
"""
import unittest
from pathlib import Path
from unittest.mock import Mock, patch
from datetime import datetime
import pytest
from fastapi.testclient import TestClient

import sys
sys.path.insert(0, str(Path(__file__).parent.parent))

from services.swiftness_api import (
    app,
    ExchangeRateUpdate,
    CashFlowRequest,
    PositionSnapshotResponse,
    CashFlowEventResponse,
    ValidationReportResponse,
    PortfolioValueResponse,
)


class TestPydanticModels(unittest.TestCase):
    """Tests for Pydantic model classes."""

    def test_exchange_rate_update_valid(self):
        """Test ExchangeRateUpdate with valid rate."""
        update = ExchangeRateUpdate(rate=0.27)
        assert update.rate == 0.27

    def test_exchange_rate_update_invalid(self):
        """Test ExchangeRateUpdate with invalid rate."""
        with pytest.raises(ValueError):
            ExchangeRateUpdate(rate=-1.0)

        with pytest.raises(ValueError):
            ExchangeRateUpdate(rate=0.0)

    def test_cash_flow_request(self):
        """Test CashFlowRequest model."""
        start = datetime(2025, 1, 1)
        end = datetime(2025, 12, 31)
        request = CashFlowRequest(
            start_date=start,
            end_date=end,
            check_validity=True,
            max_age_days=90
        )
        assert request.start_date == start
        assert request.end_date == end
        assert request.check_validity is True
        assert request.max_age_days == 90

    def test_position_snapshot_response_from_snapshot(self):
        """Test PositionSnapshotResponse.from_snapshot()."""
        # Create a mock PositionSnapshot
        mock_snapshot = Mock()
        mock_snapshot.id = "test-id"
        mock_snapshot.symbol = "AAPL"
        mock_snapshot.quantity = 100
        mock_snapshot.cost_basis = 150.0
        mock_snapshot.mark = 155.0
        mock_snapshot.unrealized_pnl = 500.0

        response = PositionSnapshotResponse.from_snapshot(mock_snapshot)
        assert response.id == "test-id"
        assert response.symbol == "AAPL"
        assert response.quantity == 100
        assert response.cost_basis == 150.0
        assert response.mark == 155.0
        assert response.unrealized_pnl == 500.0

    def test_cash_flow_event_response_from_event(self):
        """Test CashFlowEventResponse.from_event()."""
        # Create a mock CashFlowEvent
        mock_event = Mock()
        mock_event.date = datetime(2025, 6, 15)
        mock_event.amount = 1000.0
        mock_event.currency = "USD"
        mock_event.description = "Dividend payment"
        mock_event.source = "AAPL"

        response = CashFlowEventResponse.from_event(mock_event)
        assert response.date == datetime(2025, 6, 15)
        assert response.amount == 1000.0
        assert response.currency == "USD"
        assert response.description == "Dividend payment"
        assert response.source == "AAPL"

    def test_validation_report_response(self):
        """Test ValidationReportResponse model."""
        report = ValidationReportResponse(
            total_products=10,
            valid_products=["AAPL", "MSFT"],
            stale_products=["GOOGL"],
            last_updated=datetime(2025, 1, 1)
        )
        assert report.total_products == 10
        assert len(report.valid_products) == 2
        assert len(report.stale_products) == 1
        assert report.last_updated == datetime(2025, 1, 1)

    def test_portfolio_value_response(self):
        """Test PortfolioValueResponse model."""
        response = PortfolioValueResponse(total_value_usd=100000.0)
        assert response.total_value_usd == 100000.0
        assert response.currency == "USD"


class TestAPIEndpoints(unittest.TestCase):
    """Tests for FastAPI endpoints."""

    def setUp(self):
        """Set up test client with mocked dependencies."""
        # Mock the SwiftnessIntegration and SwiftnessStorage
        self.mock_storage = Mock()
        self.mock_integration = Mock()

        # Patch the integration instance in the app
        with patch('services.swiftness_api.integration', self.mock_integration):
            self.client = TestClient(app)

    def test_health_endpoint(self):
        """Test health check endpoint."""
        response = self.client.get("/health")
        assert response.status_code == 200
        data = response.json()
        assert data["status"] == "ok"
        assert data["service"] == "swiftness-api"

    def test_get_positions_success(self):
        """Test GET /positions endpoint with successful response."""
        # Mock position snapshots
        mock_snapshot = Mock()
        mock_snapshot.id = "test-id"
        mock_snapshot.symbol = "AAPL"
        mock_snapshot.quantity = 100
        mock_snapshot.cost_basis = 150.0
        mock_snapshot.mark = 155.0
        mock_snapshot.unrealized_pnl = 500.0

        self.mock_integration.get_positions.return_value = [mock_snapshot]

        with patch('services.swiftness_api.integration', self.mock_integration):
            client = TestClient(app)
            response = client.get("/positions?check_validity=true&max_age_days=90")

            assert response.status_code == 200
            data = response.json()
            assert len(data) == 1
            assert data[0]["id"] == "test-id"
            assert data[0]["symbol"] == "AAPL"

    def test_get_positions_error(self):
        """Test GET /positions endpoint with error."""
        self.mock_integration.get_positions.side_effect = Exception("Database error")

        with patch('services.swiftness_api.integration', self.mock_integration):
            client = TestClient(app)
            response = client.get("/positions")

            assert response.status_code == 500
            assert "Failed to fetch positions" in response.json()["detail"]

    def test_get_portfolio_value_success(self):
        """Test GET /portfolio-value endpoint."""
        self.mock_integration.get_portfolio_value.return_value = 100000.0

        with patch('services.swiftness_api.integration', self.mock_integration):
            client = TestClient(app)
            response = client.get("/portfolio-value")

            assert response.status_code == 200
            data = response.json()
            assert data["total_value_usd"] == 100000.0
            assert data["currency"] == "USD"

    def test_get_exchange_rate(self):
        """Test GET /exchange-rate endpoint."""
        self.mock_integration.ils_to_usd_rate = 0.27

        with patch('services.swiftness_api.integration', self.mock_integration):
            client = TestClient(app)
            response = client.get("/exchange-rate")

            assert response.status_code == 200
            data = response.json()
            assert data["rate"] == 0.27
            assert data["currency"] == "ILS/USD"

    def test_update_exchange_rate_success(self):
        """Test PUT /exchange-rate endpoint."""
        self.mock_integration.ils_to_usd_rate = 0.27

        with patch('services.swiftness_api.integration', self.mock_integration):
            client = TestClient(app)
            response = client.put("/exchange-rate", json={"rate": 0.28})

            assert response.status_code == 200
            data = response.json()
            assert data["status"] == "ok"
            # Verify update_exchange_rate was called
            self.mock_integration.update_exchange_rate.assert_called_once_with(0.28)

    def test_get_cash_flows_success(self):
        """Test GET /cash-flows endpoint."""
        mock_event = Mock()
        mock_event.date = datetime(2025, 6, 15)
        mock_event.amount = 1000.0
        mock_event.currency = "USD"
        mock_event.description = "Dividend"
        mock_event.source = "AAPL"

        self.mock_integration.get_cash_flows.return_value = [mock_event]

        start_date = datetime(2025, 1, 1)
        end_date = datetime(2025, 12, 31)

        with patch('services.swiftness_api.integration', self.mock_integration):
            client = TestClient(app)
            response = client.get(
                f"/cash-flows?start_date={start_date.isoformat()}&end_date={end_date.isoformat()}"
            )

            assert response.status_code == 200
            data = response.json()
            assert len(data) == 1

    def test_get_cash_flows_invalid_dates(self):
        """Test GET /cash-flows endpoint with invalid date range."""
        start_date = datetime(2025, 12, 31)
        end_date = datetime(2025, 1, 1)  # End before start

        with patch('services.swiftness_api.integration', self.mock_integration):
            client = TestClient(app)
            response = client.get(
                f"/cash-flows?start_date={start_date.isoformat()}&end_date={end_date.isoformat()}"
            )

            assert response.status_code == 400
            assert "end_date must be after start_date" in response.json()["detail"]

    def test_validate_positions_success(self):
        """Test GET /validate endpoint."""
        mock_report = {
            "total_products": 10,
            "valid_products": ["AAPL", "MSFT"],
            "stale_products": ["GOOGL"],
            "last_updated": datetime(2025, 1, 1)
        }

        self.mock_integration.validate_positions.return_value = mock_report

        with patch('services.swiftness_api.integration', self.mock_integration):
            client = TestClient(app)
            response = client.get("/validate")

            assert response.status_code == 200
            data = response.json()
            assert data["total_products"] == 10
            assert len(data["valid_products"]) == 2
            assert len(data["stale_products"]) == 1


if __name__ == "__main__":
    unittest.main()
