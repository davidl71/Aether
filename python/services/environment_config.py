"""
Environment variable configuration management.

Provides centralized configuration loading from environment.json and environment variables.
Environment variables take precedence over config file values.
"""
import os
import json
from pathlib import Path
from typing import Any, Optional, Dict


class EnvironmentConfig:
    """Manages environment variable configuration with file-based defaults."""
    
    def __init__(self, config_file: Optional[Path] = None):
        """
        Initialize environment configuration.
        
        Args:
            config_file: Path to environment.json file (default: config/environment.json)
        """
        if config_file is None:
            # Find project root (assume this file is in python/services/)
            current_file = Path(__file__)
            project_root = current_file.parent.parent.parent
            config_file = project_root / "config" / "environment.json"
        
        self.config_file = Path(config_file)
        self._config: Dict[str, Any] = {}
        self._load_config()
    
    def _load_config(self):
        """Load configuration from file."""
        if self.config_file.exists():
            try:
                with open(self.config_file, 'r') as f:
                    self._config = json.load(f)
            except (json.JSONDecodeError, IOError):
                # If config file is invalid, use empty config
                self._config = {}
        else:
            self._config = {}
    
    def get(self, key: str, default: Any = None, env_var: Optional[str] = None) -> Any:
        """
        Get configuration value.
        
        Priority:
        1. Environment variable (if env_var specified)
        2. Config file value (nested keys supported with dot notation)
        3. Default value
        
        Args:
            key: Config key (supports dot notation, e.g., "security.rate_limit_per_minute")
            default: Default value if not found
            env_var: Environment variable name to check first (optional)
            
        Returns:
            Configuration value
        """
        # First check environment variable if specified
        if env_var:
            env_value = os.getenv(env_var)
            if env_value is not None:
                # Try to convert to appropriate type
                return self._convert_type(env_value, default)
        
        # Then check config file
        value = self._get_nested(key, self._config)
        if value is not None:
            return value
        
        # Fall back to default
        return default
    
    def _get_nested(self, key: str, config: Dict) -> Any:
        """Get nested config value using dot notation."""
        keys = key.split('.')
        current = config
        
        for k in keys:
            if isinstance(current, dict) and k in current:
                current = current[k]
            else:
                return None
        
        return current
    
    def _convert_type(self, value: str, default: Any) -> Any:
        """Convert string environment variable to appropriate type based on default."""
        if default is None:
            # Try to infer type
            if value.lower() in ('true', 'false'):
                return value.lower() == 'true'
            try:
                return int(value)
            except ValueError:
                try:
                    return float(value)
                except ValueError:
                    return value
        
        # Convert based on default type
        if isinstance(default, bool):
            return value.lower() in ('true', '1', 'yes', 'on')
        elif isinstance(default, int):
            try:
                return int(value)
            except ValueError:
                return default
        elif isinstance(default, float):
            try:
                return float(value)
            except ValueError:
                return default
        elif isinstance(default, list):
            # Assume comma-separated list
            return [item.strip() for item in value.split(',') if item.strip()]
        
        return value
    
    def get_security_config(self) -> Dict[str, Any]:
        """Get security configuration."""
        return {
            'rate_limit_per_minute': self.get(
                'security.rate_limit_per_minute',
                default=60,
                env_var='RATE_LIMIT_PER_MINUTE'
            ),
            'rate_limit_per_second': self.get(
                'security.rate_limit_per_second',
                default=10,
                env_var='RATE_LIMIT_PER_SECOND'
            ),
            'api_key': self.get(
                'security.api_key',
                default=None,
                env_var='API_KEY'
            ),
            'require_auth': self.get(
                'security.require_auth',
                default=False,
                env_var='REQUIRE_AUTH'
            ),
            'allowed_origins': self.get(
                'security.allowed_origins',
                default=[
                    "http://localhost:3000",
                    "http://localhost:5173",
                    "http://127.0.0.1:3000",
                    "http://127.0.0.1:5173",
                ]
            ),
        }
    
    def get_service_port(self, service_name: str, default: int) -> int:
        """Get port for a service."""
        key = f"services.{service_name}_port"
        env_var = f"{service_name.upper()}_PORT"
        return self.get(key, default=default, env_var=env_var)
    
    def reload(self):
        """Reload configuration from file."""
        self._load_config()


# Global instance
_config_instance: Optional[EnvironmentConfig] = None


def get_config(config_file: Optional[Path] = None) -> EnvironmentConfig:
    """Get global configuration instance."""
    global _config_instance
    if _config_instance is None:
        _config_instance = EnvironmentConfig(config_file)
    return _config_instance


def reload_config():
    """Reload global configuration."""
    global _config_instance
    if _config_instance:
        _config_instance.reload()
