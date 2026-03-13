// tws_client_error_guidance.h - Error guidance data for EWrapper error() callback
#pragma once

#include <string>
#include <unordered_map>
#include <utility>

namespace tws {
namespace detail {

// Error code guidance - defined inline to avoid linker issues
inline const std::unordered_map<int, std::string> kIbErrorGuidance = {
    {502, "Not connected. Check TWS/IB Gateway and internet connection."},
    {504, "Not connected. Check TWS connection. Ensure TWS/Gateway is running "
          "and API is enabled."},
    {1100, "Connection lost. Check TWS/IB Gateway and internet connection. "
            "Auto-reconnect will be attempted if enabled."},
    {1101, "Connectivity restored - data maintained. Market data connection "
            "restored. Confirm subscriptions are active."},
    {1102, "Connectivity restored - data lost. Re-requesting market data "
            "subscriptions."},
    {2110, "Connectivity between TWS and server broken. Data may be delayed. "
            "Check IB network status."},
    {162, "Historical data request pacing violation. Rate limiter should "
          "prevent this. Reduce request frequency."},
    {200, "No security definition. Verify symbol, expiry, right, strike, and exchange values."},
    {201, "Order rejected - invalid contract. Order rejected by IB. Check contract "
          "error. Confirm contract fields before resubmitting."},
    {202, "Order cancelled. Order rejected by IB. Check order details, limits, and account permissions."},
    {203, "Order rejected - Order cannot be executed. Check order type, and account permissions."},
    {204, "Order rejected - Order size exceeds position limit. Reduce order "
          "size or check account limits."},
    {205, "Order rejected - Order price is outside allowable range. Adjust "
          "limit price."},
    {321, "Server validation failed. Review price increments, order type, "
          "routing, and TIF."},
    {322, "Order rejected - Duplicate order ID. Use unique order ID."},
    {323, "Order rejected - Order cannot be cancelled. Order already "
          "filled or cancelled."},
    {399, "Order rejected - Order would exceed maximum position limit. Reduce "
          "order size or check account limits."},
    {400, "Order rejected - Order would create a position that violates "
          "account restrictions."},
    {401, "Order rejected - Order type not allowed for this contract. Check "
          "order type compatibility."},
    {402, "Order rejected - Order would exceed maximum order size or price. Reduce "
          "order size or price."},
    {354, "Subscription cancelled. Check market data permissions - ensure "
          "market data is not subscribed."},
    {355, "Market data request failed. Check contract details and data subscriptions."},
    {10167, "Requested market data is not subscribed. Check market data permissions."},
    {10148, "Order size exceeds account limits. Reduce order size."},
    {2104, "Market data farm connection OK. Quotes should resume shortly."},
    {2106, "Market data farm is connecting. Expect delayed quotes until connection established."},
    {2107, "Market data farm connection failed. Check IB network status dashboard."},
    {2108, "Market data farm disconnected. Quotes will pause until reconnection."},
    {2109, "Order routing to IB server is OK."},
};

// Error phrase guidance - defined inline to avoid incomplete type issues
inline const std::pair<const char *, const char *> kErrorPhraseGuidance[] = {
    {
        "code card authentication",
        "IB triggered code card authentication. Approve the 2FA challenge in "
        "IBKR Mobile or disable code card auth.",
    },
    {
        "two factor authentication request timed out",
        "Two-factor approval timed out. Re-initiate login and approve promptly "
        "on your IBKR Mobile device.",
    },
    {
        "No market data permissions",
        "IB refused market data. Purchase/enable required market data "
        "subscriptions or switch data provider.",
    },
    {
        "No security definition has been found",
        "Contract not recognized. Double-check ticker, expiration, strike, "
        "right, and exchange.",
    },
};

}  // namespace detail
}  // namespace tws
