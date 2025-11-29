# TWS API Quick Reference

**Source**: Interactive Brokers TWS API Documentation
**Version**: 10.40.01
**Last Updated**: 2025-01-27

This is a quick reference guide for the most commonly used TWS API methods. For detailed documentation, see:

- [TWS API Documentation](https://interactivebrokers.github.io/tws-api/)
- [EClient and EWrapper Architecture](../research/learnings/ECLIENT_EWRAPPER_ARCHITECTURE.md)
- [TWS Integration Status](../research/integration/TWS_INTEGRATION_STATUS.md)

---

## Connection Management

### EClientSocket Methods

```cpp
// Connect to TWS/Gateway
bool eConnect(const char* host, int port, int clientId);

// Disconnect
void eDisconnect();

// Check connection status
bool isConnected();
```

**Ports**:

- **TWS Production**: 7496
- **TWS Paper Trading**: 7497
- **IB Gateway Production**: 4001
- **IB Gateway Paper Trading**: 4002

---

## Market Data

### Request Market Data

```cpp
void reqMktData(TickerId tickerId, const Contract& contract,
                const std::string& genericTicks, bool snapshot,
                bool regulatorySnapshots, const TagValueListSPtr& mktDataOptions);
```

**Example**:

```cpp
Contract contract;
contract.symbol = "SPX";
contract.secType = "IND";
contract.exchange = "CBOE";
contract.currency = "USD";

client.reqMktData(1, contract, "", false, false, TagValueListSPtr());
```

### Cancel Market Data

```cpp
void cancelMktData(TickerId tickerId);
```

### Request Historical Data

```cpp
void reqHistoricalData(TickerId reqId, const Contract& contract,
                      const std::string& endDateTime, const std::string& durationStr,
                      const std::string& barSizeSetting, const std::string& whatToShow,
                      int useRTH, bool formatDate, const TagValueListSPtr& chartOptions);
```

**Bar Size Options**:

- `"1 sec"`, `"5 secs"`, `"10 secs"`, `"15 secs"`, `"30 secs"`
- `"1 min"`, `"2 mins"`, `"3 mins"`, `"5 mins"`, `"10 mins"`, `"15 mins"`, `"20 mins"`, `"30 mins"`
- `"1 hour"`, `"2 hours"`, `"3 hours"`, `"4 hours"`, `"8 hours"`
- `"1 day"`, `"1 week"`, `"1 month"`

**What to Show**:

- `"TRADES"`, `"MIDPOINT"`, `"BID"`, `"ASK"`, `"BID_ASK"`
- `"HISTORICAL_VOLATILITY"`, `"OPTION_IMPLIED_VOLATILITY"`
- `"YIELD_BID"`, `"YIELD_ASK"`, `"YIELD_BID_ASK"`, `"YIELD_LAST"`

---

## Order Management

### Place Order

```cpp
void placeOrder(OrderId id, const Contract& contract, const Order& order);
```

**Order Types**:

- `"MKT"` - Market order
- `"LMT"` - Limit order
- `"STP"` - Stop order
- `"STP LMT"` - Stop limit order
- `"TRAIL"` - Trailing stop
- `"TRAIL LIMIT"` - Trailing stop limit

**Time in Force**:

- `"DAY"` - Day order
- `"GTC"` - Good till cancelled
- `"IOC"` - Immediate or cancel
- `"FOK"` - Fill or kill
- `"GTD"` - Good till date (requires `goodTillDate`)

**Example**:

```cpp
Order order;
order.action = "BUY";
order.totalQuantity = 10;
order.orderType = "LMT";
order.lmtPrice = 4500.0;
order.tif = "DAY";

client.placeOrder(orderId, contract, order);
```

### Cancel Order

```cpp
void cancelOrder(OrderId orderId, const std::string& manualOrderTime);
```

### Request All Open Orders

```cpp
void reqAllOpenOrders();
```

### Request Open Orders

```cpp
void reqOpenOrders();
```

---

## Contract Information

### Request Contract Details

```cpp
void reqContractDetails(int reqId, const Contract& contract);
```

**EWrapper Callback**:

```cpp
void contractDetails(int reqId, const ContractDetails& contractDetails) override;
void contractDetailsEnd(int reqId) override;
```

### Request Market Depth (Level 2 Data)

```cpp
void reqMktDepth(TickerId tickerId, const Contract& contract, int numRows,
                 const TagValueListSPtr& mktDepthOptions);
```

**EWrapper Callbacks**:

```cpp
void updateMktDepth(TickerId tickerId, int position, int operation, int side,
                    double price, int size) override;
void updateMktDepthL2(TickerId tickerId, int position, const std::string& marketMaker,
                       int operation, int side, double price, int size) override;
```

---

## Account & Positions

### Request Account Updates

```cpp
void reqAccountUpdates(bool subscribe, const std::string& acctCode);
```

**EWrapper Callbacks**:

```cpp
void updateAccountValue(const std::string& key, const std::string& val,
                       const std::string& currency, const std::string& accountName) override;
void updateAccountTime(const std::string& timeStamp) override;
void accountDownloadEnd(const std::string& accountName) override;
```

**Common Account Values**:

- `"NetLiquidation"` - Total account value
- `"BuyingPower"` - Buying power
- `"CashBalance"` - Cash balance
- `"TotalCashValue"` - Total cash value
- `"GrossPositionValue"` - Gross position value
- `"UnrealizedPnL"` - Unrealized P&L
- `"RealizedPnL"` - Realized P&L

### Request Positions

```cpp
void reqPositions();
```

**EWrapper Callbacks**:

```cpp
void position(const std::string& account, const Contract& contract,
              Decimal position, double avgCost) override;
void positionEnd() override;
```

### Request Portfolio Updates

```cpp
void reqAccountUpdatesMulti(int requestId, const std::string& account,
                            const std::string& modelCode, bool ledgerAndNLV);
```

---

## Options Chain

### Request Security Definition Optional Parameters

```cpp
void reqSecDefOptParams(int reqId, const std::string& underlyingSymbol,
                        const std::string& futFopExchange, const std::string& underlyingSecType,
                        int underlyingConId);
```

**EWrapper Callback**:

```cpp
void securityDefinitionOptionalParameter(int reqId, const std::string& exchange,
                                         int underlyingConId, const std::string& tradingClass,
                                         const std::string& multiplier,
                                         const std::set<std::string>& expirations,
                                         const std::set<double>& strikes) override;
```

---

## EWrapper Callbacks (Most Common)

### Connection Callbacks

```cpp
void connectAck() override;
void nextValidId(OrderId orderId) override;
void connectionClosed() override;
```

### Market Data Callbacks

```cpp
void tickPrice(TickerId tickerId, TickType field, double price,
               const TickAttrib& attribs) override;

void tickSize(TickerId tickerId, TickType field, Decimal size) override;

void tickOptionComputation(TickerId tickerId, TickType tickType,
                            double impliedVol, double delta, double optPrice,
                            double pvDividend, double gamma, double vega,
                            double theta, double undPrice) override;
```

**TickType Values**:

- `BID` (1), `ASK` (2), `LAST` (4)
- `HIGH` (6), `LOW` (7), `CLOSE` (9)
- `VOLUME` (5), `BID_SIZE` (0), `ASK_SIZE` (3)
- `LAST_SIZE` (8), `OPEN` (14)

### Order Callbacks

```cpp
void orderStatus(OrderId orderId, const std::string& status,
                 Decimal filled, Decimal remaining, double avgFillPrice,
                 int permId, int parentId, double lastFillPrice,
                 int clientId, const std::string& whyHeld,
                 double mktCapPrice) override;

void openOrder(OrderId orderId, const Contract& contract,
               const Order& order, const OrderState& orderState) override;

void execDetails(int reqId, const Contract& contract,
                 const Execution& execution) override;
```

**Order Status Values**:

- `"ApiPending"` - Order submitted but not yet acknowledged
- `"PendingSubmit"` - Order submitted to broker
- `"PreSubmitted"` - Order validated but not yet submitted
- `"Submitted"` - Order submitted to exchange
- `"Filled"` - Order completely filled
- `"Cancelled"` - Order cancelled
- `"Inactive"` - Order inactive

### Error Callback

```cpp
void error(int id, int errorCode, const std::string& errorString,
           const std::string& advancedOrderRejectJson) override;
```

**Error Code Ranges**:

- **< 1100**: Error messages (connection issues, invalid requests)
- **1100-1999**: System messages (warnings, info)
- **2000-2999**: Informational messages (market data subscriptions, etc.)

---

## Common Patterns

### 1. Connect and Wait for nextValidId

```cpp
client.eConnect("127.0.0.1", 7497, 1);

// Wait for connection acknowledgment
std::unique_lock<std::mutex> lock(mutex_);
cv_.wait(lock, [this] { return connected_ && next_valid_id_ > 0; });
```

### 2. Request Market Data with Error Handling

```cpp
void request_market_data(TickerId id, const Contract& contract) {
  if (!client_.isConnected()) {
    logger_->error("Not connected to TWS");
    return;
  }

  try {
    client_.reqMktData(id, contract, "", false, false, TagValueListSPtr());
  } catch (const std::exception& e) {
    logger_->error("Failed to request market data: {}", e.what());
  }
}
```

### 3. Place Order with Tracking

```cpp
void place_order(OrderId id, const Contract& contract, const Order& order) {
  std::lock_guard<std::mutex> lock(orders_mutex_);
  pending_orders_[id] = OrderStatus{id, "Pending", 0, order.totalQuantity};

  client_.placeOrder(id, contract, order);
}
```

---

## Thread Safety

**Important**: All EWrapper callbacks are called from the EReader thread. Always use mutexes when accessing shared data:

```cpp
class MyWrapper : public DefaultEWrapper {
  std::mutex data_mutex_;
  std::map<TickerId, double> prices_;

  void tickPrice(TickerId tickerId, TickType field, double price,
                 const TickAttrib& attribs) override {
    std::lock_guard<std::mutex> lock(data_mutex_);
    if (field == TickType::LAST) {
      prices_[tickerId] = price;
    }
  }
};
```

---

## References

- [TWS API Documentation](https://interactivebrokers.github.io/tws-api/)
- [EClient and EWrapper Architecture](../research/learnings/ECLIENT_EWRAPPER_ARCHITECTURE.md)
- [TWS Integration Status](../research/integration/TWS_INTEGRATION_STATUS.md)
- [IBKR Campus: EClient and EWrapper](https://www.interactivebrokers.com/campus/ibkr-quant-news/the-eclient-and-ewrapper-api-classes/)
