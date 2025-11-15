# Trading MCP Server

Model Context Protocol (MCP) server for trading operations, inspired by [OpenAlgo](https://github.com/marketcalls/openalgo).

This MCP server enables AI assistants (Claude Desktop, Cursor, Windsurf) to execute trades, manage positions, and query market data through natural language commands.

## Features

- **Order Management**: Place, modify, cancel orders (market/limit/stop-loss)
- **Box Spread Operations**: Place and close box spread positions
- **Position Tracking**: Get open positions, position book, close positions
- **Market Data**: Real-time quotes, market depth, historical data
- **Account Info**: Funds, holdings, order book, trade book
- **Instrument Search**: Search instruments, get symbol info, expiry dates
- **Rate Limiting**: Built-in rate limiting to prevent abuse
- **Security**: API key authentication, encrypted token storage

## Installation

```bash
cd mcp/trading_server
pip install -e .
```

## Configuration

Add to `.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "trading": {
      "command": "python",
      "args": [
        "-m",
        "mcp.trading_server.server"
      ],
      "env": {
        "TRADING_API_KEY": "your-api-key-here",
        "TWS_HOST": "127.0.0.1",
        "TWS_PORT": "7497",
        "DRY_RUN": "true"
      }
    }
  }
}
```

## Available Tools

### Order Operations

- `place_order` - Place a single order
- `place_box_spread` - Place a 4-leg box spread order
- `modify_order` - Modify an existing order
- `cancel_order` - Cancel an order
- `cancel_all_orders` - Cancel all open orders

### Position Management

- `get_open_positions` - Get all open positions
- `get_position_book` - Get detailed position information
- `close_position` - Close a specific position
- `close_all_positions` - Close all positions

### Market Data

- `get_quote` - Get real-time quote for a symbol
- `get_market_depth` - Get order book depth
- `get_historical_data` - Get historical price data
- `search_instruments` - Search for instruments
- `get_symbol_info` - Get detailed symbol information
- `get_expiry_dates` - Get option expiry dates

### Account Information

- `get_funds` - Get account funds and buying power
- `get_holdings` - Get account holdings
- `get_order_book` - Get all orders (open, filled, cancelled)
- `get_trade_book` - Get trade history

## Usage Examples

### Natural Language Commands

```
"Buy 100 shares of SPY at market"
"Place a limit order for 10 SPX calls at strike 5000"
"Show me my open positions"
"Cancel order 12345"
"Get quote for SPY"
"What are my account funds?"
```

### Programmatic Usage

The MCP server exposes tools that can be called by AI assistants. Each tool has:
- **Name**: Tool identifier
- **Description**: What the tool does
- **Parameters**: Input schema (JSON Schema)
- **Returns**: Response format

## Security

- **API Key Authentication**: All requests require valid API key
- **Rate Limiting**: Per-IP and per-endpoint rate limits
- **Dry Run Mode**: Test operations without real execution
- **Token Encryption**: Sensitive tokens encrypted at rest

## Rate Limiting

Rate limits are configurable via environment variables:

- `ORDER_RATE_LIMIT`: Orders per second (default: 10)
- `API_RATE_LIMIT`: General API calls per second (default: 10)
- `MARKET_DATA_RATE_LIMIT`: Market data requests per second (default: 20)

## Error Handling

All tools return structured error responses:

```json
{
  "success": false,
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Too many requests. Please wait.",
    "retry_after": 1.5
  }
}
```

## Architecture

The MCP server uses a bridge pattern to integrate with native C++ code:

```
MCP Server (Python)
    ↓
Trading Bridge (bridge.py)
    ↓
[REST API] or [Future: Cython Bindings]
    ↓
Native C++ OrderManager
    ↓
TWS API
```

**Current Implementation**:
- Uses REST API to communicate with backend service
- Supports dry-run mode for testing
- Falls back to mock data if backend unavailable

**Future Enhancements**:
- Direct Cython bindings for OrderManager
- Shared library integration via ctypes
- WebSocket support for real-time updates

## Development

See `server.py` and `bridge.py` for implementation details. The server uses:
- FastMCP for MCP protocol implementation
- Trading bridge for native code integration
- REST API for backend communication
- TWS API for Interactive Brokers integration (via backend)

## References

- [OpenAlgo MCP Integration](https://github.com/marketcalls/openalgo/tree/main/mcp)
- [Model Context Protocol](https://modelcontextprotocol.io/)
- [TWS API Documentation](https://interactivebrokers.github.io/tws-api/)
