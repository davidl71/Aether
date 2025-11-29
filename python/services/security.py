"""
Security utilities for API endpoints: path boundary enforcement, rate limiting, and access control.
"""
import os
from pathlib import Path
from typing import Optional
from functools import wraps
from datetime import datetime, timedelta
from collections import defaultdict
import time

from fastapi import HTTPException, Request, status
from starlette.middleware.base import BaseHTTPMiddleware


class PathBoundaryEnforcer:
    """Enforces path boundaries to prevent directory traversal attacks."""
    
    def __init__(self, allowed_base_paths: list[Path | str]):
        """
        Initialize path boundary enforcer.
        
        Args:
            allowed_base_paths: List of allowed base paths. All resolved paths must be within these.
        """
        self.allowed_base_paths = [Path(p).resolve() for p in allowed_base_paths]
    
    def validate_path(self, file_path: str | Path) -> Path:
        """
        Validate that a path is within allowed boundaries.
        
        Args:
            file_path: Path to validate
            
        Returns:
            Resolved Path object if valid
            
        Raises:
            ValueError: If path is outside allowed boundaries
        """
        resolved_path = Path(file_path).resolve()
        
        # Check if path is within any allowed base path
        for base_path in self.allowed_base_paths:
            try:
                resolved_path.relative_to(base_path)
                return resolved_path
            except ValueError:
                continue
        
        raise ValueError(f"Path {resolved_path} is outside allowed boundaries")
    
    def sanitize_path(self, file_path: str | Path) -> Path:
        """
        Sanitize and validate a path, removing any dangerous components.
        
        Args:
            file_path: Path to sanitize
            
        Returns:
            Sanitized and validated Path object
            
        Raises:
            ValueError: If path cannot be sanitized or is invalid
        """
        # Remove any '..' components and resolve
        path = Path(file_path)
        
        # Prevent absolute paths from being used directly
        if path.is_absolute():
            # Only allow if it's within an allowed base path
            return self.validate_path(path)
        
        # For relative paths, resolve against allowed base paths
        for base_path in self.allowed_base_paths:
            candidate = (base_path / path).resolve()
            try:
                candidate.relative_to(base_path)
                return candidate
            except ValueError:
                continue
        
        raise ValueError(f"Cannot sanitize path: {file_path}")


class RateLimiter:
    """Simple in-memory rate limiter for API endpoints."""
    
    def __init__(self, requests_per_minute: int = 60, requests_per_second: int = 10):
        """
        Initialize rate limiter.
        
        Args:
            requests_per_minute: Maximum requests per minute per IP
            requests_per_second: Maximum requests per second per IP
        """
        self.requests_per_minute = requests_per_minute
        self.requests_per_second = requests_per_second
        self._requests: dict[str, list[float]] = defaultdict(list)
        self._lock = {}
    
    def _cleanup_old_requests(self, ip: str, now: float):
        """Remove requests older than 1 minute."""
        cutoff = now - 60.0
        self._requests[ip] = [t for t in self._requests[ip] if t > cutoff]
    
    def check_rate_limit(self, ip: str) -> bool:
        """
        Check if request should be allowed based on rate limits.
        
        Args:
            ip: Client IP address
            
        Returns:
            True if request should be allowed, False if rate limited
        """
        now = time.time()
        
        # Clean up old requests
        self._cleanup_old_requests(ip, now)
        
        # Check per-second limit
        recent_second = [t for t in self._requests[ip] if t > now - 1.0]
        if len(recent_second) >= self.requests_per_second:
            return False
        
        # Check per-minute limit
        if len(self._requests[ip]) >= self.requests_per_minute:
            return False
        
        # Record this request
        self._requests[ip].append(now)
        return True
    
    def get_remaining_requests(self, ip: str) -> dict[str, int]:
        """Get remaining request counts for an IP."""
        now = time.time()
        self._cleanup_old_requests(ip, now)
        
        recent_second = [t for t in self._requests[ip] if t > now - 1.0]
        
        return {
            "per_second": max(0, self.requests_per_second - len(recent_second)),
            "per_minute": max(0, self.requests_per_minute - len(self._requests[ip]))
        }


class RateLimitMiddleware(BaseHTTPMiddleware):
    """FastAPI middleware for rate limiting."""
    
    def __init__(self, app, rate_limiter: RateLimiter):
        super().__init__(app)
        self.rate_limiter = rate_limiter
    
    async def dispatch(self, request: Request, call_next):
        # Get client IP
        client_ip = request.client.host if request.client else "unknown"
        
        # Check rate limit
        if not self.rate_limiter.check_rate_limit(client_ip):
            remaining = self.rate_limiter.get_remaining_requests(client_ip)
            raise HTTPException(
                status_code=status.HTTP_429_TOO_MANY_REQUESTS,
                detail={
                    "error": "Rate limit exceeded",
                    "remaining": remaining,
                    "retry_after": 60
                },
                headers={
                    "Retry-After": "60",
                    "X-RateLimit-Limit-PerMinute": str(self.rate_limiter.requests_per_minute),
                    "X-RateLimit-Limit-PerSecond": str(self.rate_limiter.requests_per_second),
                }
            )
        
        response = await call_next(request)
        
        # Add rate limit headers
        remaining = self.rate_limiter.get_remaining_requests(client_ip)
        response.headers["X-RateLimit-Remaining-PerMinute"] = str(remaining["per_minute"])
        response.headers["X-RateLimit-Remaining-PerSecond"] = str(remaining["per_second"])
        
        return response


class AccessControl:
    """Simple access control for API endpoints."""
    
    def __init__(self, api_key: Optional[str] = None, require_auth: bool = False):
        """
        Initialize access control.
        
        Args:
            api_key: Optional API key for authentication
            require_auth: Whether authentication is required
        """
        self.api_key = api_key
        self.require_auth = require_auth
    
    def validate_api_key(self, provided_key: Optional[str]) -> bool:
        """
        Validate API key.
        
        Args:
            provided_key: API key from request
            
        Returns:
            True if valid, False otherwise
        """
        if not self.require_auth:
            return True
        
        if not self.api_key:
            return False
        
        return provided_key == self.api_key
    
    def get_client_ip(self, request: Request) -> str:
        """Extract client IP from request."""
        # Check for forwarded IP (from proxy)
        forwarded = request.headers.get("X-Forwarded-For")
        if forwarded:
            return forwarded.split(",")[0].strip()
        
        # Check for real IP
        real_ip = request.headers.get("X-Real-IP")
        if real_ip:
            return real_ip
        
        # Fall back to direct client
        return request.client.host if request.client else "unknown"


def require_api_key(access_control: AccessControl):
    """Decorator to require API key authentication."""
    def decorator(func):
        @wraps(func)
        async def wrapper(*args, request: Request, **kwargs):
            # Get API key from header
            api_key = request.headers.get("X-API-Key") or request.headers.get("Authorization")
            
            # Remove "Bearer " prefix if present
            if api_key and api_key.startswith("Bearer "):
                api_key = api_key[7:]
            
            if not access_control.validate_api_key(api_key):
                raise HTTPException(
                    status_code=status.HTTP_401_UNAUTHORIZED,
                    detail="Invalid or missing API key",
                    headers={"WWW-Authenticate": "Bearer"},
                )
            
            return await func(*args, request=request, **kwargs)
        return wrapper
    return decorator
