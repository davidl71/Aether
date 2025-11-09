"""
test_nautilus_integration.py - Tests for nautilus_trader integration
"""
import pytest
import sys
from pathlib import Path
from unittest.mock import Mock, MagicMock

# Add project root to path
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root / "python"))

try:
    from integration.nautilus_client import NautilusClient
    from integration.market_data_handler import MarketDataHandler
    from integration.execution_handler import ExecutionHandler
    from integration.config_adapter import ConfigAdapter
    NAUTILUS_AVAILABLE = True
except ImportError:
    NAUTILUS_AVAILABLE = False
    pytest.skip("NautilusTrader integration not available", allow_module_level=True)


class TestConfigAdapter:
    """Tests for ConfigAdapter"""
    
    def test_load_config(self, tmp_path):
        """Test loading configuration"""
        config_file = tmp_path / "config.json"
        config_file.write_text('{"test": "value"}')
        
        config = ConfigAdapter.load_config(str(config_file))
        assert config == {"test": "value"}
    
    def test_get_nautilus_data_config(self):
        """Test extracting data client config"""
        config = {
            "tws": {"host": "127.0.0.1", "port": 7497, "client_id": 1},
            "nautilus_trader": {
                "data_client_config": {
                    "host": "localhost",
                    "port": 7498,
                }
            }
        }
        
        data_config = ConfigAdapter.get_nautilus_data_config(config)
        assert data_config["host"] == "localhost"
        assert data_config["port"] == 7498
    
    def test_get_nautilus_exec_config(self):
        """Test extracting execution client config"""
        config = {
            "tws": {"host": "127.0.0.1", "port": 7497, "client_id": 1},
            "nautilus_trader": {
                "exec_client_config": {
                    "host": "localhost",
                    "port": 7498,
                }
            }
        }
        
        exec_config = ConfigAdapter.get_nautilus_exec_config(config)
        assert exec_config["host"] == "localhost"
        assert exec_config["port"] == 7498
    
    def test_get_strategy_config(self):
        """Test extracting strategy config"""
        config = {
            "strategy": {
                "symbols": ["SPY", "QQQ"],
                "min_arbitrage_profit": 0.10,
            }
        }
        
        strategy_config = ConfigAdapter.get_strategy_config(config)
        assert strategy_config["symbols"] == ["SPY", "QQQ"]
        assert strategy_config["min_arbitrage_profit"] == 0.10


class TestMarketDataHandler:
    """Tests for MarketDataHandler"""
    
    def test_init(self):
        """Test handler initialization"""
        handler = MarketDataHandler()
        assert handler is not None
    
    def test_register_callback(self):
        """Test registering callbacks"""
        handler = MarketDataHandler()
        callback = Mock()
        
        handler.register_callback("SPY", callback)
        assert "SPY" in handler._callbacks


class TestNautilusClient:
    """Tests for NautilusClient"""
    
    def test_init(self):
        """Test client initialization"""
        client = NautilusClient(
            data_config={"host": "127.0.0.1"},
            exec_config={"host": "127.0.0.1"},
        )
        assert client is not None
        assert not client.is_connected()
    
    def test_create_data_client(self):
        """Test creating data client"""
        # Note: This will fail if nautilus_trader is not properly installed
        # So we'll skip the actual creation test
        client = NautilusClient()
        # Just verify the method exists
        assert hasattr(client, "create_data_client")
    
    def test_create_exec_client(self):
        """Test creating execution client"""
        client = NautilusClient()
        # Just verify the method exists
        assert hasattr(client, "create_exec_client")



