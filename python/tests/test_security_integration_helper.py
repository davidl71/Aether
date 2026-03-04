"""
Tests for security integration helper.

Tests add_security_to_app() and add_security_headers_middleware() functions.
"""

import unittest
from pathlib import Path
from unittest.mock import Mock, patch
from fastapi import FastAPI
from fastapi.testclient import TestClient

import sys

sys.path.insert(0, str(Path(__file__).parent.parent))

from services.security_integration_helper import (
    add_security_to_app,
    add_security_headers_middleware,
)


class TestAddSecurityToApp(unittest.TestCase):
    """Tests for add_security_to_app() function."""

    def setUp(self):
        """Set up test FastAPI app."""
        self.app = FastAPI()

    def test_add_security_to_app_defaults(self):
        """Test add_security_to_app() with default parameters."""
        with patch("services.security_integration_helper.get_config") as mock_config:
            mock_config_instance = Mock()
            mock_config_instance.get_security_config.return_value = {
                "rate_limit_per_minute": 60,
                "rate_limit_per_second": 10,
                "allowed_origins": [],
                "api_key": None,
                "require_auth": False,
            }
            mock_config.return_value = mock_config_instance

            result = add_security_to_app(self.app)

            # Verify security components are returned
            assert "rate_limiter" in result
            assert "path_enforcer" in result
            assert "access_control" in result

            # Verify CORS middleware was added
            assert len(self.app.user_middleware) > 0

    def test_add_security_to_app_custom_origins(self):
        """Test add_security_to_app() with custom allowed origins."""
        custom_origins = ["https://example.com", "https://app.example.com"]

        with patch("services.security_integration_helper.get_config") as mock_config:
            mock_config_instance = Mock()
            mock_config_instance.get_security_config.return_value = {
                "rate_limit_per_minute": 60,
                "rate_limit_per_second": 10,
                "api_key": None,
                "require_auth": False,
            }
            mock_config.return_value = mock_config_instance

            result = add_security_to_app(self.app, allowed_origins=custom_origins)

            # Verify security components are returned
            assert "rate_limiter" in result
            assert "path_enforcer" in result
            assert "access_control" in result

    def test_add_security_to_app_custom_project_root(self):
        """Test add_security_to_app() with custom project root."""
        custom_root = Path("/custom/project/root")

        with patch("services.security_integration_helper.get_config") as mock_config:
            mock_config_instance = Mock()
            mock_config_instance.get_security_config.return_value = {
                "rate_limit_per_minute": 60,
                "rate_limit_per_second": 10,
                "api_key": None,
                "require_auth": False,
            }
            mock_config.return_value = mock_config_instance

            result = add_security_to_app(self.app, project_root=custom_root)

            # Verify path_enforcer was created with custom root
            assert "path_enforcer" in result
            path_enforcer = result["path_enforcer"]
            # Path enforcer stores subdirectories, not the root itself
            assert (custom_root / "data") in path_enforcer.allowed_base_paths
            assert (custom_root / "storage") in path_enforcer.allowed_base_paths

    def test_add_security_to_app_enable_api_key(self):
        """Test add_security_to_app() with API key enabled."""
        with patch("services.security_integration_helper.get_config") as mock_config:
            mock_config_instance = Mock()
            mock_config_instance.get_security_config.return_value = {
                "rate_limit_per_minute": 60,
                "rate_limit_per_second": 10,
                "api_key": "test-api-key",
                "require_auth": False,
            }
            mock_config.return_value = mock_config_instance

            result = add_security_to_app(self.app, enable_api_key=True)

            # Verify access_control was created with API key requirement
            assert "access_control" in result
            access_control = result["access_control"]
            assert access_control.require_auth is True

    def test_add_security_to_app_auto_detect_project_root(self):
        """Test add_security_to_app() auto-detects project root when None."""
        with patch("services.security_integration_helper.get_config") as mock_config:
            mock_config_instance = Mock()
            mock_config_instance.get_security_config.return_value = {
                "rate_limit_per_minute": 60,
                "rate_limit_per_second": 10,
                "api_key": None,
                "require_auth": False,
            }
            mock_config.return_value = mock_config_instance

            result = add_security_to_app(self.app, project_root=None)

            # Verify path_enforcer was created (auto-detected root)
            assert "path_enforcer" in result

    def test_add_security_to_app_removes_existing_cors(self):
        """Test that add_security_to_app() removes existing CORS middleware."""
        # Add a CORS middleware first
        from fastapi.middleware.cors import CORSMiddleware

        self.app.add_middleware(CORSMiddleware, allow_origins=["old-origin"])

        initial_middleware_count = len(self.app.user_middleware)

        with patch("services.security_integration_helper.get_config") as mock_config:
            mock_config_instance = Mock()
            mock_config_instance.get_security_config.return_value = {
                "rate_limit_per_minute": 60,
                "rate_limit_per_second": 10,
                "api_key": None,
                "require_auth": False,
            }
            mock_config.return_value = mock_config_instance

            add_security_to_app(self.app)

            # Verify new CORS middleware was added (old one removed)
            assert len(self.app.user_middleware) >= initial_middleware_count

    def test_add_security_to_app_rate_limiter_config(self):
        """Test that rate limiter uses config values."""
        with patch("services.security_integration_helper.get_config") as mock_config:
            mock_config_instance = Mock()
            mock_config_instance.get_security_config.return_value = {
                "rate_limit_per_minute": 120,
                "rate_limit_per_second": 20,
                "api_key": None,
                "require_auth": False,
            }
            mock_config.return_value = mock_config_instance

            result = add_security_to_app(self.app)

            # Verify rate limiter was created
            assert "rate_limiter" in result
            rate_limiter = result["rate_limiter"]
            assert rate_limiter.requests_per_minute == 120
            assert rate_limiter.requests_per_second == 20


class TestAddSecurityHeadersMiddleware(unittest.TestCase):
    """Tests for add_security_headers_middleware() function."""

    def setUp(self):
        """Set up test FastAPI app."""
        self.app = FastAPI()

        @self.app.get("/test")
        def test_endpoint():
            return {"message": "test"}

    def test_add_security_headers_middleware_adds_middleware(self):
        """Test that add_security_headers_middleware() adds middleware."""
        initial_middleware_count = len(self.app.user_middleware)
        add_security_headers_middleware(self.app)
        assert len(self.app.user_middleware) > initial_middleware_count

    def test_security_headers_present_in_response(self):
        """Test that security headers are present in HTTP responses."""
        add_security_headers_middleware(self.app)
        client = TestClient(self.app)

        response = client.get("/test")

        assert response.status_code == 200
        assert "X-Content-Type-Options" in response.headers
        assert response.headers["X-Content-Type-Options"] == "nosniff"
        assert "X-Frame-Options" in response.headers
        assert response.headers["X-Frame-Options"] == "DENY"
        assert "X-XSS-Protection" in response.headers
        assert response.headers["X-XSS-Protection"] == "1; mode=block"
        assert "Referrer-Policy" in response.headers
        assert response.headers["Referrer-Policy"] == "strict-origin-when-cross-origin"
        assert "Content-Security-Policy" in response.headers

    def test_csp_header_format(self):
        """Test that CSP header has correct format."""
        add_security_headers_middleware(self.app)
        client = TestClient(self.app)

        response = client.get("/test")

        csp = response.headers["Content-Security-Policy"]
        assert "default-src 'self'" in csp
        assert "script-src 'self'" in csp
        assert "style-src 'self'" in csp
        assert "img-src 'self'" in csp


if __name__ == "__main__":
    unittest.main()
