"""
config_adapter.py - Adapter for loading and converting configuration
Converts JSON config to format needed by NautilusTrader and ORATS clients
"""
import logging
import json
from typing import Any, Dict, List, Optional
from pathlib import Path

logger = logging.getLogger(__name__)


class ConfigAdapter:
    """
    Adapter for configuration management.
    Loads JSON config and converts to component-specific formats.
    """
    
    @staticmethod
    def load_config(config_path: str) -> Dict[str, Any]:
        """
        Load configuration from JSON file.
        
        Args:
            config_path: Path to config.json
            
        Returns:
            Configuration dictionary
        """
        config_file = Path(config_path)
        
        if not config_file.exists():
            raise FileNotFoundError(f"Configuration file not found: {config_path}")
        
        try:
            with open(config_file, 'r') as f:
                config = json.load(f)
            
            logger.info(f"Loaded configuration from {config_path}")
            return config
            
        except json.JSONDecodeError as e:
            logger.error(f"Failed to parse configuration: {e}")
            raise
        except Exception as e:
            logger.error(f"Failed to load configuration: {e}")
            raise
    
    @staticmethod
    def get_nautilus_data_config(config: Dict) -> Dict:
        """
        Extract NautilusTrader data client configuration.
        
        Args:
            config: Full configuration dict
            
        Returns:
            Data client config dict
        """
        nautilus_config = config.get("nautilus_trader", {})
        data_config = nautilus_config.get("data_client_config", {})
        
        return {
            "host": data_config.get("host", "127.0.0.1"),
            "port": data_config.get("port", 7497),
            "client_id": data_config.get("client_id", 1),
        }
    
    @staticmethod
    def get_nautilus_exec_config(config: Dict) -> Dict:
        """
        Extract NautilusTrader execution client configuration.
        
        Args:
            config: Full configuration dict
            
        Returns:
            Execution client config dict
        """
        nautilus_config = config.get("nautilus_trader", {})
        exec_config = nautilus_config.get("exec_client_config", {})
        
        return {
            "host": exec_config.get("host", "127.0.0.1"),
            "port": exec_config.get("port", 7497),
            "client_id": exec_config.get("client_id", 1),
        }
    
    @staticmethod
    def get_strategy_config(config: Dict) -> Dict:
        """
        Extract strategy configuration.
        
        Args:
            config: Full configuration dict
            
        Returns:
            Strategy config dict
        """
        return config.get("strategy", {})
    
    @staticmethod
    def get_risk_config(config: Dict) -> Dict:
        """
        Extract risk management configuration.
        
        Args:
            config: Full configuration dict
            
        Returns:
            Risk config dict
        """
        return config.get("risk", {})
    
    @staticmethod
    def get_orats_config(config: Dict) -> Optional[Dict]:
        """
        Extract ORATS configuration.
        
        Args:
            config: Full configuration dict
            
        Returns:
            ORATS config dict or None if not enabled
        """
        orats_config = config.get("orats", {})
        
        if not orats_config.get("enabled", False):
            return None
        
        return {
            "api_token": orats_config.get("api_token", ""),
            "base_url": orats_config.get("base_url", "https://api.orats.io"),
            "use_for_liquidity_scoring": orats_config.get("use_for_liquidity_scoring", True),
            "use_for_iv_data": orats_config.get("use_for_iv_data", True),
            "use_for_risk_events": orats_config.get("use_for_risk_events", True),
            "min_liquidity_score": orats_config.get("min_liquidity_score", 70.0),
            "max_iv_percentile": orats_config.get("max_iv_percentile", 80.0),
            "earnings_blackout_days": orats_config.get("earnings_blackout_days", 7),
            "dividend_blackout_days": orats_config.get("dividend_blackout_days", 2),
            "cache_duration_seconds": orats_config.get("cache_duration_seconds", 300),
            "rate_limit_per_second": orats_config.get("rate_limit_per_second", 10),
        }

    @staticmethod
    def get_connection_management_config(config: Dict) -> Dict:
        """Extract connection management configuration."""
        connection_cfg = config.get("connection_management", {})
        weekly = connection_cfg.get("weekly_reauth", {})

        return {
            "weekly_reauth": {
                "enabled": weekly.get("enabled", False),
                "day_of_week": weekly.get("day_of_week", "sunday"),
                "time_utc": weekly.get("time_utc", "21:00"),
                "reminder_minutes_before": weekly.get("reminder_minutes_before", 15),
                "reauth_window_minutes": weekly.get("reauth_window_minutes", 10),
                "auto_reconnect": weekly.get("auto_reconnect", True),
            }
        }

    @staticmethod
    def get_notifications_config(config: Dict) -> Dict:
        """Extract notification configuration."""
        notifications = config.get("notifications", {})
        return {
            "enabled": notifications.get("enabled", False),
            "default_severity": notifications.get("default_severity", "info"),
            "channels": notifications.get("channels", []),
        }

    @staticmethod
    def get_data_provider_config(config: Dict) -> Dict:
        """Extract data provider configuration."""
        providers = config.get("data_providers", {})
        return {
            "primary": providers.get("primary", "ib"),
            "fallbacks": providers.get("fallbacks", []),
        }

    @staticmethod
    def get_questdb_config(config: Dict) -> Dict:
        """Extract QuestDB configuration."""
        questdb = config.get("questdb", {})
        return {
            "enabled": questdb.get("enabled", False),
            "ilp_host": questdb.get("ilp_host", "127.0.0.1"),
            "ilp_port": questdb.get("ilp_port", 9009),
            "quote_table": questdb.get("quote_table", "quotes"),
            "trade_table": questdb.get("trade_table", "trades"),
            "max_retries": questdb.get("max_retries", 3),
        }

    @staticmethod
    def get_ibkr_portal_config(config: Dict) -> Dict:
        """Extract IBKR Client Portal API configuration."""
        portal = config.get("ibkr_portal", {})
        return {
            "enabled": portal.get("enabled", False),
            "base_url": portal.get("base_url", "https://localhost:5000/v1/portal"),
            "verify_ssl": portal.get("verify_ssl", False),
            "timeout_seconds": portal.get("timeout_seconds", 5),
            "preferred_accounts": portal.get("preferred_accounts", []),
        }
    
    @staticmethod
    def validate_config(config: Dict) -> tuple[bool, List[str]]:
        """
        Validate configuration.
        
        Args:
            config: Configuration dict
            
        Returns:
            Tuple of (is_valid, error_messages)
        """
        errors = []
        
        # Validate TWS config
        tws = config.get("tws", {})
        if not tws.get("host"):
            errors.append("TWS host not specified")
        if not tws.get("port"):
            errors.append("TWS port not specified")
        
        # Validate strategy config
        strategy = config.get("strategy", {})
        if not strategy.get("symbols"):
            errors.append("No symbols specified in strategy")
        
        # Validate ORATS config if enabled
        orats = config.get("orats", {})
        if orats.get("enabled", False):
            if not orats.get("api_token"):
                errors.append("ORATS enabled but no API token provided")
        
        return (len(errors) == 0, errors)
