// broker_interface.h - Unified broker interface for multi-broker support
#pragma once

#include "../types.h"
#include <functional>
#include <optional>
#include <vector>
#include <string>
#include <memory>

namespace brokers {

// ============================================================================
// Broker Types and Enumerations
// ============================================================================

enum class BrokerType {
  TWS,
  IB_CLIENT_PORTAL,
  ALPACA
};

enum class ConnectionState {
  Disconnected,
  Connecting,
  Connected,
  Error
};

struct BrokerCapabilities {
  bool supports_options;
  bool supports_multi_leg_orders;
  bool supports_real_time_data;
  bool supports_historical_data;
  int max_orders_per_second;
  int rate_limit_per_minute;

  BrokerCapabilities()
    : supports_options(false),
      supports_multi_leg_orders(false),
      supports_real_time_data(false),
      supports_historical_data(false),
      max_orders_per_second(0),
      rate_limit_per_minute(0) {}
};

// ============================================================================
// IBroker Interface
// ============================================================================

/**
 * Unified broker interface for multi-broker support.
 *
 * All broker adapters (TWS, IB Client Portal, Alpaca) implement this interface
 * to provide a consistent API for trading operations.
 */
class IBroker {
public:
  virtual ~IBroker() = default;

  // ========================================================================
  // Connection Management
  // ========================================================================

  /**
   * Connect to the broker.
   * @return true if connection successful, false otherwise
   */
  [[nodiscard]] virtual bool connect() = 0;

  /**
   * Disconnect from the broker.
   */
  virtual void disconnect() = 0;

  /**
   * Check if currently connected.
   * @return true if connected, false otherwise
   */
  [[nodiscard]] virtual bool is_connected() const = 0;

  /**
   * Get current connection state.
   * @return ConnectionState enum value
   */
  [[nodiscard]] virtual ConnectionState get_connection_state() const = 0;

  /**
   * Get broker type.
   * @return BrokerType enum value
   */
  [[nodiscard]] virtual BrokerType get_broker_type() const = 0;

  /**
   * Get broker capabilities.
   * @return BrokerCapabilities struct
   */
  [[nodiscard]] virtual BrokerCapabilities get_capabilities() const = 0;

  // ========================================================================
  // Market Data
  // ========================================================================

  /**
   * Request market data for an option contract (async).
   * @param contract Option contract to get data for
   * @param callback Function called when data is received
   * @return Request ID (positive) or error code (negative)
   */
  virtual int request_market_data(
    const types::OptionContract& contract,
    std::function<void(const types::MarketData&)> callback
  ) = 0;

  /**
   * Cancel market data subscription.
   * @param request_id Request ID returned from request_market_data
   */
  virtual void cancel_market_data(int request_id) = 0;

  /**
   * Request market data synchronously (blocks until data received or timeout).
   * @param contract Option contract to get data for
   * @param timeout_ms Timeout in milliseconds (default: 5000)
   * @return MarketData if successful, empty optional if timeout/error
   */
  [[nodiscard]] virtual std::optional<types::MarketData> request_market_data_sync(
    const types::OptionContract& contract,
    int timeout_ms = 5000
  ) = 0;

  // ========================================================================
  // Options Chain
  // ========================================================================

  /**
   * Request options chain for a symbol.
   * @param symbol Underlying symbol (e.g., "SPY", "XSP")
   * @param expiry Expiration date in YYYYMMDD format (empty for all expirations)
   * @return Vector of OptionContract objects
   */
  [[nodiscard]] virtual std::vector<types::OptionContract> request_option_chain(
    const std::string& symbol,
    const std::string& expiry = ""
  ) = 0;

  // ========================================================================
  // Contract Details
  // ========================================================================

  /**
   * Request contract details (async).
   * @param contract Option contract
   * @param callback Function called with contract ID when details received
   * @return Request ID (positive) or error code (negative)
   */
  virtual int request_contract_details(
    const types::OptionContract& contract,
    std::function<void(long conId)> callback
  ) = 0;

  /**
   * Request contract details synchronously.
   * @param contract Option contract
   * @param timeout_ms Timeout in milliseconds (default: 5000)
   * @return Contract ID if successful, 0 if timeout/error
   */
  [[nodiscard]] virtual long request_contract_details_sync(
    const types::OptionContract& contract,
    int timeout_ms = 5000
  ) = 0;

  // ========================================================================
  // Order Management
  // ========================================================================

  /**
   * Place a single-leg order.
   * @param contract Option contract
   * @param action Buy or Sell
   * @param quantity Number of contracts
   * @param limit_price Limit price (0.0 for market orders)
   * @param tif Time in force (default: Day)
   * @return Order ID (positive) or error code (negative)
   */
  virtual int place_order(
    const types::OptionContract& contract,
    types::OrderAction action,
    int quantity,
    double limit_price = 0.0,
    types::TimeInForce tif = types::TimeInForce::Day
  ) = 0;

  /**
   * Cancel an order.
   * @param order_id Order ID returned from place_order
   * @return true if cancellation successful, false otherwise
   */
  virtual bool cancel_order(int order_id) = 0;

  /**
   * Get order status.
   * @param order_id Order ID
   * @return Order object if found, empty optional otherwise
   */
  [[nodiscard]] virtual std::optional<types::Order> get_order_status(int order_id) const = 0;

  // ========================================================================
  // Multi-Leg Orders (Box Spreads)
  // ========================================================================

  /**
   * Place a multi-leg combo order (e.g., box spread).
   * @param contracts Vector of option contracts (4 legs for box spread)
   * @param actions Vector of order actions (Buy/Sell for each leg)
   * @param quantities Vector of quantities for each leg
   * @param contract_ids Vector of contract IDs (from request_contract_details)
   * @param limit_prices Vector of limit prices (0.0 for market)
   * @return Order ID (positive) or error code (negative)
   */
  virtual int place_combo_order(
    const std::vector<types::OptionContract>& contracts,
    const std::vector<types::OrderAction>& actions,
    const std::vector<int>& quantities,
    const std::vector<long>& contract_ids,
    const std::vector<double>& limit_prices
  ) = 0;

  // ========================================================================
  // Positions
  // ========================================================================

  /**
   * Get all positions.
   * @return Vector of Position objects
   */
  [[nodiscard]] virtual std::vector<types::Position> get_positions() = 0;

  /**
   * Get position for a specific contract.
   * @param contract Option contract
   * @return Position if found, empty optional otherwise
   */
  [[nodiscard]] virtual std::optional<types::Position> get_position(
    const types::OptionContract& contract
  ) = 0;

  // ========================================================================
  // Account Information
  // ========================================================================

  /**
   * Get account information.
   * @return AccountInfo if successful, empty optional otherwise
   */
  [[nodiscard]] virtual std::optional<types::AccountInfo> get_account_info() = 0;

  /**
   * Get buying power.
   * @return Buying power amount
   */
  [[nodiscard]] virtual double get_buying_power() = 0;

  /**
   * Get net liquidation value.
   * @return Net liquidation value
   */
  [[nodiscard]] virtual double get_net_liquidation_value() = 0;

  // ========================================================================
  // Error Handling
  // ========================================================================

  /**
   * Set error callback function.
   * @param callback Function called when errors occur
   */
  virtual void set_error_callback(
    std::function<void(int code, const std::string& msg)> callback
  ) = 0;
};

} // namespace brokers
