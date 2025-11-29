"""
Security integration helper for FastAPI services.
Provides reusable functions to add security to any FastAPI app.
"""
import os
from pathlib import Path
from typing import List, Optional

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from .security import (
    PathBoundaryEnforcer,
    RateLimiter,
    RateLimitMiddleware,
    AccessControl
)
from .environment_config import get_config


def add_security_to_app(
    app: FastAPI,
    project_root: Optional[Path] = None,
    allowed_origins: Optional[List[str]] = None,
    enable_api_key: bool = False
) -> dict:
    """
    Add security components to a FastAPI application.
    
    Args:
        app: FastAPI application instance
        project_root: Project root path (for path boundary enforcement)
        allowed_origins: List of allowed CORS origins (default: localhost only)
        enable_api_key: Whether to enable API key authentication
        
    Returns:
        Dictionary with security components (rate_limiter, path_enforcer, access_control)
    """
    # Default CORS origins (secure by default, can be overridden by config)
    config = get_config()
    security_config = config.get_security_config()
    if allowed_origins is None:
        allowed_origins = security_config.get('allowed_origins', [
            "http://localhost:3000",
            "http://localhost:5173",
            "http://127.0.0.1:3000",
            "http://127.0.0.1:5173",
        ])
    
    # Update CORS middleware with secure defaults
    # Remove existing CORS middleware if present
    app.user_middleware = [m for m in app.user_middleware if m.cls != CORSMiddleware]
    
    app.add_middleware(
        CORSMiddleware,
        allow_origins=allowed_origins,
        allow_credentials=True,
        allow_methods=["GET", "POST", "PUT", "DELETE", "OPTIONS"],
        allow_headers=["*"],
        expose_headers=["X-RateLimit-Remaining-PerMinute", "X-RateLimit-Remaining-PerSecond"],
    )
    
    # Initialize rate limiter (using environment config)
    config = get_config()
    security_config = config.get_security_config()
    rate_limiter = RateLimiter(
        requests_per_minute=security_config['rate_limit_per_minute'],
        requests_per_second=security_config['rate_limit_per_second']
    )
    app.add_middleware(RateLimitMiddleware, rate_limiter=rate_limiter)
    
    # Initialize path boundary enforcer
    if project_root is None:
        # Try to detect project root
        current_file = Path(__file__)
        project_root = current_file.parent.parent.parent
    
    path_enforcer = PathBoundaryEnforcer(
        allowed_base_paths=[
            Path.home() / ".config" / "ib_box_spread",
            project_root / "data",
            project_root / "storage",
            project_root / "logs",
        ]
    )
    
    # Initialize access control (using environment config)
    access_control = AccessControl(
        api_key=security_config['api_key'],
        require_auth=enable_api_key or security_config['require_auth']
    )
    
    return {
        "rate_limiter": rate_limiter,
        "path_enforcer": path_enforcer,
        "access_control": access_control
    }


def add_security_headers_middleware(app: FastAPI):
    """
    Add security headers middleware to FastAPI app.
    """
    from starlette.middleware.base import BaseHTTPMiddleware
    from starlette.responses import Response
    
    class SecurityHeadersMiddleware(BaseHTTPMiddleware):
        async def dispatch(self, request, call_next):
            response = await call_next(request)
            response.headers["X-Content-Type-Options"] = "nosniff"
            response.headers["X-Frame-Options"] = "DENY"
            response.headers["X-XSS-Protection"] = "1; mode=block"
            response.headers["Referrer-Policy"] = "strict-origin-when-cross-origin"
            
            # Add CSP header (can be customized)
            csp = (
                "default-src 'self'; "
                "script-src 'self' 'unsafe-inline' 'unsafe-eval'; "
                "style-src 'self' 'unsafe-inline'; "
                "img-src 'self' data: https:; "
                "font-src 'self' data:; "
                "connect-src 'self' https:;"
            )
            response.headers["Content-Security-Policy"] = csp
            
            return response
    
    app.add_middleware(SecurityHeadersMiddleware)
