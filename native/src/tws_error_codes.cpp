// tws_error_codes.cpp - IB TWS error code guidance lookup tables
#include "tws_error_codes.h"

namespace tws {

const std::unordered_map<int, std::string> kIbErrorGuidance = {
    // Connection errors (500-599)
    {502, "Connection rejected. Enable 'ActiveX and Socket Clients' in TWS Settings > API > Settings. Verify IP is trusted (127.0.0.1) and port is correct."},
    {504, "Not connected. Check TWS connection. Ensure TWS/Gateway is running and API is enabled."},

    // System messages (1100-1999)
    {1100, "Connection lost. Check TWS/IB Gateway and internet connection. Auto-reconnect will be attempted if enabled."},
    {1101, "Connectivity restored - data maintained. Market data connection restored. Confirm subscriptions are active."},
    {1102, "Connectivity restored - data lost. Re-requesting market data subscriptions."},
    {2110, "Connectivity between TWS and server broken. Data may be delayed. Check IB network status."},

    // Contract/Order errors (100-299)
    {162, "Historical data request pacing violation. Rate limiter should prevent this. Reduce request frequency."},
    {200, "No security definition found for request. Invalid contract definition. Verify symbol, expiry, right, strike, and exchange values."},
    {201, "Order rejected - invalid contract. Order rejected due to contract error. Confirm contract fields before resubmitting."},
    {202, "Order cancelled. Order rejected by IB. Check order parameters, size limits, and account permissions."},
    {203, "Order rejected - Order cannot be executed. Check market hours, order type, and account permissions."},
    {204, "Order rejected - Order size exceeds position limit. Reduce order size or check account limits."},
    {205, "Order rejected - Order price is outside acceptable range. Adjust limit price."},

    // Validation errors (300-399)
    {321, "Server validation failed. Review price increments, exchange routing, and TIF."},
    {322, "Order rejected - Duplicate order ID. Use unique order IDs for each order."},
    {323, "Order rejected - Order cannot be cancelled. Order may already be filled or cancelled."},
    {399, "Order rejected - Order would exceed maximum position size. Check account limits."},
    {400, "Order rejected - Order would create a position that violates account restrictions."},
    {401, "Order rejected - Order type not allowed for this contract. Check order type compatibility."},
    {402, "Order rejected - Order would exceed maximum order value. Reduce order size or price."},

    // Market data errors (350-399, 10000+)
    {354, "Subscription cancelled. Check market data permissions. Requested market data is not subscribed."},
    {355, "Market data request failed. Check contract details and market data subscriptions."},
    {10167, "Requested market data is not subscribed. Check data permissions. Ensure your IB account has the required data subscriptions."},

    // Order errors (10000+)
    {10148, "Order size exceeds account limits. Reduce order size or check account limits."},

    // Market data farm messages (2100-2199)
    {2104, "Market data farm connection restored. Quotes should resume normally."},
    {2106, "Market data farm is connecting. Expect delayed quotes until established."},
    {2107, "Market data farm connection failed. Check IB network status dashboard."},
    {2108, "Market data farm disconnected. Quotes will pause until reconnection."},
    {2109, "Order routing to IB server is OK."},
};

const std::pair<const char*, const char*> kErrorPhraseGuidance[] = {
    {
        "code card authentication",
        "IB triggered code card authentication. Approve the 2FA challenge in IBKR Mobile or disable code card auth.",
    },
    {
        "two factor authentication request timed out",
        "Two-factor approval timed out. Re-initiate login and approve promptly on your IBKR Mobile device.",
    },
    {
        "No market data permissions",
        "IB refused market data. Purchase/enable required market data subscriptions or switch data provider.",
    },
    {
        "No security definition has been found",
        "Contract not recognized. Double-check ticker, expiration, strike, right, and exchange.",
    },
};

const size_t kErrorPhraseGuidanceCount = sizeof(kErrorPhraseGuidance) / sizeof(kErrorPhraseGuidance[0]);

} // namespace tws
