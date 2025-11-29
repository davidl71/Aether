"""
Tests for security utilities.

Tests PathBoundaryEnforcer, RateLimiter, AccessControl, and related functionality.
"""
import unittest
import time
from pathlib import Path
from unittest.mock import Mock, patch, AsyncMock
from fastapi import Request, HTTPException

import sys
sys.path.insert(0, str(Path(__file__).parent.parent))

from services.security import (
    PathBoundaryEnforcer,
    RateLimiter,
    AccessControl,
    RateLimitMiddleware,
    require_api_key
)


class TestPathBoundaryEnforcer(unittest.TestCase):
    """Tests for PathBoundaryEnforcer."""
    
    def test_init(self):
        """Test PathBoundaryEnforcer initialization."""
        enforcer = PathBoundaryEnforcer([Path("/allowed/path")])
        assert len(enforcer.allowed_base_paths) == 1
        assert enforcer.allowed_base_paths[0] == Path("/allowed/path").resolve()
    
    def test_validate_path_within_boundary(self, tmp_path):
        """Test validating a path within allowed boundaries."""
        allowed = tmp_path / "allowed"
        allowed.mkdir()
        file_path = allowed / "test.txt"
        file_path.write_text("test")
        
        enforcer = PathBoundaryEnforcer([allowed])
        result = enforcer.validate_path(file_path)
        assert result == file_path.resolve()
    
    def test_validate_path_outside_boundary(self, tmp_path):
        """Test validating a path outside allowed boundaries."""
        allowed = tmp_path / "allowed"
        allowed.mkdir()
        outside = tmp_path / "outside"
        outside.mkdir()
        file_path = outside / "test.txt"
        file_path.write_text("test")
        
        enforcer = PathBoundaryEnforcer([allowed])
        with self.assertRaises(ValueError):
            enforcer.validate_path(file_path)
    
    def test_sanitize_path_relative(self, tmp_path):
        """Test sanitizing a relative path."""
        allowed = tmp_path / "allowed"
        allowed.mkdir()
        
        enforcer = PathBoundaryEnforcer([allowed])
        result = enforcer.sanitize_path("test.txt")
        assert result == (allowed / "test.txt").resolve()
    
    def test_sanitize_path_with_dotdot(self, tmp_path):
        """Test sanitizing a path with .. components."""
        allowed = tmp_path / "allowed"
        allowed.mkdir()
        subdir = allowed / "subdir"
        subdir.mkdir()
        
        enforcer = PathBoundaryEnforcer([allowed])
        # Path with .. should still resolve within boundary
        result = enforcer.sanitize_path("subdir/../test.txt")
        assert result == (allowed / "test.txt").resolve()
    
    def test_sanitize_path_absolute_within_boundary(self, tmp_path):
        """Test sanitizing an absolute path within boundary."""
        allowed = tmp_path / "allowed"
        allowed.mkdir()
        file_path = allowed / "test.txt"
        file_path.write_text("test")
        
        enforcer = PathBoundaryEnforcer([allowed])
        result = enforcer.sanitize_path(str(file_path))
        assert result == file_path.resolve()
    
    def test_sanitize_path_absolute_outside_boundary(self, tmp_path):
        """Test sanitizing an absolute path outside boundary."""
        allowed = tmp_path / "allowed"
        allowed.mkdir()
        outside = tmp_path / "outside"
        outside.mkdir()
        file_path = outside / "test.txt"
        file_path.write_text("test")
        
        enforcer = PathBoundaryEnforcer([allowed])
        with self.assertRaises(ValueError):
            enforcer.sanitize_path(str(file_path))


class TestRateLimiter(unittest.TestCase):
    """Tests for RateLimiter."""
    
    def test_init(self):
        """Test RateLimiter initialization."""
        limiter = RateLimiter(requests_per_minute=60, requests_per_second=10)
        assert limiter.requests_per_minute == 60
        assert limiter.requests_per_minute == 60
        assert limiter.requests_per_second == 10
    
    def test_rate_limit_allows_requests(self):
        """Test that rate limiter allows requests within limits."""
        limiter = RateLimiter(requests_per_minute=60, requests_per_second=10)
        
        # Should allow requests
        for i in range(5):
            assert limiter.check_rate_limit("127.0.0.1") is True
    
    def test_rate_limit_blocks_per_second(self):
        """Test that rate limiter blocks requests exceeding per-second limit."""
        limiter = RateLimiter(requests_per_minute=60, requests_per_second=2)
        
        # First 2 requests should be allowed
        assert limiter.check_rate_limit("127.0.0.1") is True
        assert limiter.check_rate_limit("127.0.0.1") is True
        
        # Third request should be blocked
        assert limiter.check_rate_limit("127.0.0.1") is False
    
    def test_rate_limit_blocks_per_minute(self):
        """Test that rate limiter blocks requests exceeding per-minute limit."""
        limiter = RateLimiter(requests_per_minute=5, requests_per_second=10)
        
        # First 5 requests should be allowed
        for i in range(5):
            assert limiter.check_rate_limit("127.0.0.1") is True
        
        # Sixth request should be blocked
        assert limiter.check_rate_limit("127.0.0.1") is False
    
    def test_rate_limit_different_ips(self):
        """Test that rate limits are per IP address."""
        limiter = RateLimiter(requests_per_minute=2, requests_per_second=2)
        
        # Each IP should have its own limit
        assert limiter.check_rate_limit("127.0.0.1") is True
        assert limiter.check_rate_limit("127.0.0.1") is True
        assert limiter.check_rate_limit("127.0.0.1") is False
        
        # Different IP should have separate limit
        assert limiter.check_rate_limit("192.168.1.1") is True
        assert limiter.check_rate_limit("192.168.1.1") is True
        assert limiter.check_rate_limit("192.168.1.1") is False
    
    def test_get_remaining_requests(self):
        """Test getting remaining request counts."""
        limiter = RateLimiter(requests_per_minute=60, requests_per_second=10)
        
        # Make some requests
        limiter.check_rate_limit("127.0.0.1")
        limiter.check_rate_limit("127.0.0.1")
        
        remaining = limiter.get_remaining_requests("127.0.0.1")
        assert remaining["per_second"] == 8  # 10 - 2
        assert remaining["per_minute"] == 58  # 60 - 2
    
    def test_cleanup_old_requests(self):
        """Test that old requests are cleaned up."""
        limiter = RateLimiter(requests_per_minute=60, requests_per_second=10)
        
        # Make requests
        with patch('time.time', return_value=1000.0):
            limiter.check_rate_limit("127.0.0.1")
        
        # Advance time by more than 60 seconds
        with patch('time.time', return_value=1070.0):
            remaining = limiter.get_remaining_requests("127.0.0.1")
            # Old request should be cleaned up
            assert remaining["per_minute"] == 60


class TestAccessControl(unittest.TestCase):
    """Tests for AccessControl."""
    
    def test_init_without_auth(self):
        """Test AccessControl initialization without authentication."""
        access = AccessControl(api_key=None, require_auth=False)
        assert access.api_key is None
        assert access.require_auth is False
    
    def test_init_with_auth(self):
        """Test AccessControl initialization with authentication."""
        access = AccessControl(api_key="test-key", require_auth=True)
        assert access.api_key == "test-key"
        assert access.require_auth is True
    
    def test_validate_api_key_no_auth_required(self):
        """Test API key validation when auth not required."""
        access = AccessControl(api_key=None, require_auth=False)
        assert access.validate_api_key(None) is True
        assert access.validate_api_key("any-key") is True
    
    def test_validate_api_key_auth_required_valid(self):
        """Test API key validation with valid key when auth required."""
        access = AccessControl(api_key="test-key", require_auth=True)
        assert access.validate_api_key("test-key") is True
    
    def test_validate_api_key_auth_required_invalid(self):
        """Test API key validation with invalid key when auth required."""
        access = AccessControl(api_key="test-key", require_auth=True)
        assert access.validate_api_key("wrong-key") is False
        assert access.validate_api_key(None) is False
    
    def test_validate_api_key_auth_required_no_key_set(self):
        """Test API key validation when auth required but no key set."""
        access = AccessControl(api_key=None, require_auth=True)
        assert access.validate_api_key("any-key") is False
        assert access.validate_api_key(None) is False
    
    def test_get_client_ip_direct(self):
        """Test getting client IP from direct connection."""
        access = AccessControl()
        request = Mock()
        request.headers = {}
        request.client.host = "192.168.1.1"
        
        ip = access.get_client_ip(request)
        assert ip == "192.168.1.1"
    
    def test_get_client_ip_forwarded(self):
        """Test getting client IP from X-Forwarded-For header."""
        access = AccessControl()
        request = Mock()
        request.headers = {"X-Forwarded-For": "192.168.1.1, 10.0.0.1"}
        request.client.host = "127.0.0.1"
        
        ip = access.get_client_ip(request)
        assert ip == "192.168.1.1"  # Should use first IP
    
    def test_get_client_ip_real_ip(self):
        """Test getting client IP from X-Real-IP header."""
        access = AccessControl()
        request = Mock()
        request.headers = {"X-Real-IP": "192.168.1.1"}
        request.client.host = "127.0.0.1"
        
        ip = access.get_client_ip(request)
        assert ip == "192.168.1.1"
    
    def test_get_client_ip_unknown(self):
        """Test getting client IP when client is None."""
        access = AccessControl()
        request = Mock()
        request.headers = {}
        request.client = None
        
        ip = access.get_client_ip(request)
        assert ip == "unknown"


class TestRateLimitMiddleware(unittest.TestCase):
    """Tests for RateLimitMiddleware."""
    
    async def test_middleware_allows_request(self):
        """Test that middleware allows requests within rate limit."""
        limiter = RateLimiter(requests_per_minute=60, requests_per_second=10)
        app = Mock()
        middleware = RateLimitMiddleware(app, limiter)
        
        request = Mock()
        request.client.host = "127.0.0.1"
        
        async def call_next(req):
            return Mock(status_code=200)
        
        response = await middleware.dispatch(request, call_next)
        self.assertEqual(response.status_code, 200)
    
    async def test_middleware_blocks_request(self):
        """Test that middleware blocks requests exceeding rate limit."""
        limiter = RateLimiter(requests_per_minute=60, requests_per_second=1)
        app = Mock()
        middleware = RateLimitMiddleware(app, limiter)
        
        request = Mock()
        request.client.host = "127.0.0.1"
        
        async def call_next(req):
            return Mock(status_code=200)
        
        # First request should succeed
        response = await middleware.dispatch(request, call_next)
        self.assertEqual(response.status_code, 200)
        
        # Second request should be blocked
        with self.assertRaises(HTTPException) as context:
            await middleware.dispatch(request, call_next)
        
        self.assertEqual(context.exception.status_code, 429)
        self.assertIn("Rate limit exceeded", str(context.exception.detail))


class TestRequireApiKey(unittest.TestCase):
    """Tests for require_api_key decorator."""
    
    async def test_decorator_with_valid_key(self):
        """Test decorator with valid API key."""
        access = AccessControl(api_key="test-key", require_auth=True)
        
        @require_api_key(access)
        async def test_func(request: Request):
            return {"status": "ok"}
        
        request = Mock()
        request.headers = {"X-API-Key": "test-key"}
        
        result = await test_func(request=request)
        self.assertEqual(result, {"status": "ok"})
    
    async def test_decorator_with_invalid_key(self):
        """Test decorator with invalid API key."""
        access = AccessControl(api_key="test-key", require_auth=True)
        
        @require_api_key(access)
        async def test_func(request: Request):
            return {"status": "ok"}
        
        request = Mock()
        request.headers = {"X-API-Key": "wrong-key"}
        
        with self.assertRaises(HTTPException) as context:
            await test_func(request=request)
        
        self.assertEqual(context.exception.status_code, 401)
    
    async def test_decorator_with_bearer_token(self):
        """Test decorator with Bearer token."""
        access = AccessControl(api_key="test-key", require_auth=True)
        
        @require_api_key(access)
        async def test_func(request: Request):
            return {"status": "ok"}
        
        request = Mock()
        request.headers = {"Authorization": "Bearer test-key"}
        
        result = await test_func(request=request)
        self.assertEqual(result, {"status": "ok"})
    
    async def test_decorator_no_auth_required(self):
        """Test decorator when auth not required."""
        access = AccessControl(api_key=None, require_auth=False)
        
        @require_api_key(access)
        async def test_func(request: Request):
            return {"status": "ok"}
        
        request = Mock()
        request.headers = {}
        
        result = await test_func(request=request)
        self.assertEqual(result, {"status": "ok"})
