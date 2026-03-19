# TWS API Learnings from Sibling tws-api Repo

**Purpose:** Summarize what we can learn from the **sibling repository** `tws-api` (e.g. `../tws-api`) about option contracts, tick types, market hours, and the connection protocol. Use this when implementing or aligning TWS socket behaviour, or when mapping Client Portal fields to TWS concepts.

**Sibling path:** `../tws-api` (relative to this project root).

---

## 1. Option contracts

### 1.1 Contract fields (TWS API)

From **`tws-api/samples/Cpp/TestCppClient/ContractSamples.cpp`** (and `Contract.h` / Python `contract.py`):

| Field | Purpose | Example / note |
|-------|---------|----------------|
| **symbol** | Underlying symbol | `"GOOG"`, `"AAPL"` |
| **secType** | Security type | `"OPT"` (equity option), `"IOPT"` (e.g. Dutch warrant), `"FOP"` (futures option) |
| **exchange** | Exchange | `"SMART"`, `"BOX"`, `"ISE"`, `"EUREX"`, `"MEFFRV"` |
| **currency** | Currency | `"USD"`, `"EUR"` |
| **lastTradeDateOrContractMonth** | Expiry | `"20170120"` (YYYYMMDD) or `"202612"` (YYYYMM) for some |
| **strike** | Strike price | `615`, `7.5`, `0` (e.g. forecastx) |
| **right** | Put/Call | `"C"` (call), `"P"` (put) |
| **multiplier** | Contract multiplier | `"100"` (equity), `"1000"` (e.g. FOP), `"0.01"` (FX) |
| **tradingClass** | When multiple classes trade | e.g. `"SANEU"` (OptionWithTradingClass); required for some underlyings |
| **localSymbol** | Exchange’s own symbol | e.g. `"P BMW  20221216 72 M"` (OptionWithLocalSymbol); “watch out for spaces” |
| **conId** | Contract ID | Optional; used when server supports `MIN_SERVER_VER_REQ_MKT_DATA_CONID` |

**Takeaway:** Option contracts need **symbol, secType, exchange, currency, lastTradeDateOrContractMonth, strike, right, multiplier**. Use **tradingClass** or **localSymbol** when the exchange or product requires it. TWS’ symbol often maps to API **localSymbol** for futures/options; underlying symbol maps to API **symbol**.

### 1.2 Option samples in tws-api

- **USOptionContract** – GOOG, SMART, 20170120, strike 615, C, multiplier 100.
- **OptionAtBox** – GOOG, BOX, same expiry/strike/right.
- **OptionWithTradingClass** – SANT, MEFFRV, EUR, 20190621, strike 7.5, C, tradingClass SANEU.
- **OptionWithLocalSymbol** – localSymbol `"P BMW  20221216 72 M"`, EUREX, EUR.
- **OptionForQuery** – minimal fields for `reqContractDetails` (symbol, secType, exchange, currency; used to discover full chain/details).
- **FuturesOnOptions** – FOP example (underlying future + strike, right, multiplier).

### 1.3 reqMktData contract encoding (EClient.cpp)

For **reqMktData**, the contract is sent as: conId (if server supports), symbol, secType, lastTradeDateOrContractMonth, strike, right, multiplier, exchange, primaryExchange, currency, localSymbol, tradingClass (if server supports), then combo legs or delta-neutral if applicable. So option identification in market data requests matches the contract fields above.

---

## 2. Tick types

### 2.1 TickType enum (C++ EWrapper.h)

From **`tws-api/source/cppclient/client/EWrapper.h`** – full enum (abbreviated here by category):

**Core price/size:**

- `BID_SIZE`, `BID`, `ASK`, `ASK_SIZE`, `LAST`, `LAST_SIZE`
- `HIGH`, `LOW`, `VOLUME`, `CLOSE`, `OPEN`

**Option computations:**

- `BID_OPTION_COMPUTATION`, `ASK_OPTION_COMPUTATION`, `LAST_OPTION_COMPUTATION`, `MODEL_OPTION`
- `OPTION_HISTORICAL_VOL`, `OPTION_IMPLIED_VOL`, `OPTION_BID_EXCH`, `OPTION_ASK_EXCH`
- `OPTION_CALL_OPEN_INTEREST`, `OPTION_PUT_OPEN_INTEREST`, `OPTION_CALL_VOLUME`, `OPTION_PUT_VOLUME`
- `CUST_OPTION_COMPUTATION`
- Delayed option: `DELAYED_BID_OPTION_COMPUTATION`, `DELAYED_ASK_OPTION_COMPUTATION`, `DELAYED_LAST_OPTION_COMPUTATION`, `DELAYED_MODEL_OPTION_COMPUTATION`

**Delayed (non–real-time):**

- `DELAYED_BID`, `DELAYED_ASK`, `DELAYED_LAST`, `DELAYED_BID_SIZE`, `DELAYED_ASK_SIZE`, `DELAYED_LAST_SIZE`
- `DELAYED_HIGH`, `DELAYED_LOW`, `DELAYED_VOLUME`, `DELAYED_CLOSE`, `DELAYED_OPEN`, `DELAYED_LAST_TIMESTAMP`, `DELAYED_HALTED`
- `DELAYED_YIELD_BID`, `DELAYED_YIELD_ASK`

**Session / RTH:**

- `LAST_RTH_TRADE` – last trade during regular trading hours.

**Other:**

- `BID_EXCH`, `ASK_EXCH`, `LAST_EXCH`, `LAST_TIMESTAMP`, `LAST_REG_TIME`
- `MARK_PRICE`, `AUCTION_*`, `HALTED`, `SHORTABLE`, `RT_VOLUME`, `TRADE_COUNT`, `TRADE_RATE`, `VOLUME_RATE`
- `AVG_VOLUME`, `OPEN_INTEREST`, `FUTURES_OPEN_INTEREST`, `AVG_OPT_VOLUME`
- EFP, yields, ETFs, IPO, etc.

**Helper:** `isPrice(TickType)` returns true for BID, ASK, LAST and their DELAYED_* variants (used for size tick pairing).

### 2.2 Client Portal snapshot “field 31”

Client Portal **marketdata snapshot** uses numeric field IDs. **Field 31** is **last price** (equivalent to TWS `LAST`). Other snapshot fields map to the same concepts (bid, ask, last, volume, etc.); see IBKR Client Portal docs for the full field list.

### 2.3 Tick-by-tick (reqTickByTickData)

- **tickType** values (string): `"Last"`, `"AllLast"`, `"BidAsk"`, `"MidPoint"` (from C#/Python client and docs).
- Historical tick types: `HISTORICAL_TICKS_LAST` (98), `HISTORICAL_TICKS_BID_ASK` (97).

---

## 3. Market hours

### 3.1 ContractDetails: tradingHours and liquidHours

From **ContractDetails** (and proto `ContractDetails.proto`):

- **tradingHours** – String describing regular trading hours for the contract.
- **liquidHours** – String describing liquid trading hours.

Decoded in **EDecoder** (contract details message) and **EDecoderUtils** (protobuf); printed in **TestCppClient.cpp** and VB/Java samples. Server version **47** and above can send **timeZoneId**, **tradingHours**, **liquidHours** in contract details (see EClient.java comment).

### 3.2 useRTH (Regular Trading Hours) in requests

Used in historical and real-time bar/tick requests to restrict to regular trading hours:

- **reqHistoricalData**: `useRTH`: `0` = all data in the span, `1` = only data within RTH.
- **reqRealTimeBars**, **reqHeadTimeStamp**, **reqHistogramData**, **reqHistoricalTicks**: same `useRTH` flag.

So “market hours” for **data** are controlled by **useRTH**; for **contract metadata** they are in **tradingHours** / **liquidHours**.

### 3.3 Orders: outsideRth and conditionsIgnoreRth

From **order.py** and **decoder_utils**:

- **outsideRth** – Allow execution outside regular trading hours (e.g. extended hours).
- **conditionsIgnoreRth** – Whether order conditions ignore RTH.

So **market hours** for **execution** are influenced by **outsideRth** and condition handling.

### 3.4 Bond trading hours

**MIN_SERVER_VER_BOND_TRADING_HOURS** (188) gates decoding of bond-specific trading hours in contract details (EDecoder.cpp).

---

## 4. Connection protocol

### 4.1 Sequence (socket)

1. **connect(host, port, clientId)**  
   TCP connect to TWS/Gateway (e.g. 7496 live, 7497 paper). **clientId** is an integer; multiple clients can use different IDs (0, 1, 2, …).

2. **connectAck()**  
   First server message after connect. EDecoder **processConnectAck** decodes:
   - **serverVersion** (int) – TWS/Gateway API version.
   - **twsTime** (string) – if serverVersion >= 20.

   After that, the client can send **startApi**.

3. **startApi()**  
   Must be called **after** connectAck when using **asynchronous** connect. Sends:
   - **clientId** (int).
   - **optionalCapabilities** (string) – if serverVersion >= MIN_SERVER_VER_OPTIONAL_CAPABILITIES.

   In samples: **connectAck** callback calls **startApi()** when `asynchronous`/`AsyncEConnect` is true (Python Testbed, VB EWrapperImpl, Java EWrapperImpl).

4. **nextValidId(orderId)** (or nextValidIdProtoBuf)  
   Server sends the next valid order ID. Samples use this as “session ready” and store the value for **placeOrder**; order IDs must be incremented after each order when multiple clients share an account.

### 4.2 Client identity and version

- **setClientId(clientId)** – Set before connect (default often -1; then set to 0, 1, etc.).
- **m_serverVersion** – Stored in EDecoder from connectAck; used everywhere for MIN_SERVER_VER_* checks (e.g. conId, tradingClass, protobuf, bond trading hours).
- **setConnectOptions** / **optionalCapabilities** – Optional connection string/capabilities sent in startApi.

### 4.3 ProtoBuf path

If **useProtoBuf(START_API)** is true, **startApiProtoBuf(StartApiRequest)** is used: message contains **clientId** and **optionalCapabilities**. Same pattern for other requests (e.g. reqSecDefOptParams, reqMktData).

### 4.4 Summary table

| Step | Client | Server |
|------|--------|--------|
| 1 | connect(host, port, clientId) | TCP accept |
| 2 | — | connectAck: serverVersion [, twsTime] |
| 3 | startApi(clientId [, optionalCapabilities]) | — |
| 4 | — | nextValidId(orderId) |
| 5 | Use orderId for placeOrder, etc. | — |

---

## 5. Where to look in tws-api

| Topic | Key paths (under `tws-api/`) |
|-------|------------------------------|
| Option contract samples | `samples/Cpp/TestCppClient/ContractSamples.cpp`, `ContractSamples.h` |
| Contract / ContractDetails | `source/cppclient/client/Contract.h`, proto `ContractDetails.proto` |
| TickType enum | `source/cppclient/client/EWrapper.h` |
| Tick decoding / attributes | `source/cppclient/client/EDecoder.cpp`, `TickAttribLast.h`, `TickAttribBidAsk.h` |
| Market data request (contract encoding) | `source/cppclient/client/EClient.cpp` (reqMktData, reqTickByTickData) |
| tradingHours / liquidHours | `Contract.h`, `EDecoder.cpp`, `EDecoderUtils.cpp`, `ContractDetails.proto` |
| useRTH | `source/pythonclient/ibapi/client.py` (reqHistoricalData, etc.), client_utils (create*RequestProto) |
| outsideRth / conditionsIgnoreRth | `source/pythonclient/ibapi/order.py`, decoder_utils |
| connectAck / serverVersion | `source/cppclient/client/EDecoder.cpp` (processConnectAck) |
| startApi / clientId | `source/cppclient/client/EClient.cpp`, `EClientUtils.cpp` (createStartApiRequestProto) |
| connectAck → startApi in samples | `samples/Python/Testbed/Program.py`, `samples/VB/Testbed/EWrapperImpl.vb`, `samples/Java/.../EWrapperImpl.java` |
| Scanner subscription | See §6 below |

---

## 6. Scanner subscription

The TWS API provides **market scanners**: pre-defined or parameterized screens (e.g. “hot by volume”, “top % gainers”, “high option volume put/call ratio”) that return a list of contracts. Useful for discovery (e.g. which underlyings to then request market data for).

### 6.1 Flow

1. **reqScannerParameters()** – Request an XML string that describes all available scanner queries (instruments, location codes, scan codes). No arguments.
2. **scannerParameters(xml)** (EWrapper) – Callback with the XML. Can be saved or parsed to discover valid `instrument`, `locationCode`, `scanCode` and filter names.
3. **reqScannerSubscription(reqId, subscription, scannerSubscriptionOptions, scannerSubscriptionFilterOptions)** – Start a scanner. **subscription** is a **ScannerSubscription** (see below). Options/filterOptions are TagValue lists (e.g. generic filters; filter options require TWS v973+ / MIN_SERVER_VER_SCANNER_GENERIC_OPTS).
4. **scannerData(reqId, rank, contractDetails, distance, benchmark, projection, legsStr)** – One callback per result row (contract + rank and scan-specific fields). **No market data** (bid/ask/last) is included; use **reqMktData** separately if needed.
5. **scannerDataEnd(reqId)** – Signals end of this delivery for that reqId. The subscription stays open and can receive periodic updates until cancelled.
6. **cancelScannerSubscription(reqId)** – Stop updates for that reqId.

### 6.2 ScannerSubscription fields (C++ ScannerSubscription.h / Python scanner.py)

| Field | Type | Purpose |
|-------|------|---------|
| **numberOfRows** | int | Max results to return (-1 = default, often 50 max) |
| **instrument** | string | Instrument filter, e.g. `"STK"`, `"STOCK.EU"`, `"FUT.EU"`, `"IND.US"`, `"NATCOMB"` |
| **locationCode** | string | Location/exchange, e.g. `"STK.US.MAJOR"`, `"STK.EU.IBIS"`, `"FUT.EU.EUREX"`, `"IND.US"`, `"NATCOMB.OPT.US"` |
| **scanCode** | string | Scan type, e.g. `"HOT_BY_VOLUME"`, `"MOST_ACTIVE"`, `"TOP_PERC_GAIN"`, `"HIGH_OPT_VOLUME_PUT_CALL_RATIO"`, `"COMBO_LATEST_TRADE"` |
| **abovePrice** / **belowPrice** | double | Price filter |
| **aboveVolume** | int | Minimum volume |
| **marketCapAbove** / **marketCapBelow** | double | Market cap filter |
| **moodyRatingAbove/Below**, **spRatingAbove/Below** | string | Credit ratings (bonds) |
| **maturityDateAbove/Below**, **couponRateAbove/Below** | string / double | Bond filters |
| **excludeConvertible** | bool/int | Exclude convertibles |
| **averageOptionVolumeAbove** | int | Option volume filter |
| **scannerSettingPairs** | string | Additional key=value pairs |
| **stockTypeFilter** | string | Stock type filter |

### 6.3 Sample subscriptions (samples/Cpp/TestCppClient/ScannerSubscriptionSamples.cpp)

- **HotUSStkByVolume** – instrument `STK`, locationCode `STK.US.MAJOR`, scanCode `HOT_BY_VOLUME`.
- **TopPercentGainersIbis** – `STOCK.EU`, `STK.EU.IBIS`, `TOP_PERC_GAIN`.
- **MostActiveFutEurex** – `FUT.EU`, `FUT.EU.EUREX`, `MOST_ACTIVE`.
- **HighOptVolumePCRatioUSIndexes** – `IND.US`, `IND.US`, `HIGH_OPT_VOLUME_PUT_CALL_RATIO`.
- **ComplexOrdersAndTrades** – `NATCOMB`, `NATCOMB.OPT.US`, `COMBO_LATEST_TRADE` (combo/options scan).

### 6.4 Limits (from docs/content/scanners/scanners.txt)

- **Max 50 results** per scan code per subscription.
- **Max 10** API scanner subscriptions active at once.

### 6.5 Example files in tws-api

| Path (under `tws-api/`) | Description |
|--------------------------|-------------|
| **Extended Samples/Python/Scanner/Scanner.py** | Minimal: after nextValidId, builds ScannerSubscription (STK, STK.US.MAJOR, MOST_ACTIVE), adds filter_options (e.g. marketCapAbove/Below), calls reqScannerSubscription; scannerData prints rank+contract; scannerDataEnd cancels and disconnects. |
| **Extended Samples/Python/Scanner/ScannerParameters.py** | reqScannerParameters; scannerParameters saves XML. |
| **samples/Python/Testbed/Program.py** | reqScannerParameters; reqScannerSubscription(7001, HighOptVolumePCRatioUSIndexes(), [], []), (7002, HotUSStkByVolume(), [], tagvalues), (7003, ComplexOrdersAndTrades(), [], AAPLConIDTag); cancelScannerSubscription(7001/7002/7003); scannerParameters/scannerData/scannerDataEnd handlers. |
| **samples/Cpp/TestCppClient/ScannerSubscriptionSamples.cpp** | HotUSStkByVolume, TopPercentGainersIbis, MostActiveFutEurex, HighOptVolumePCRatioUSIndexes, ComplexOrdersAndTrades. |
| **samples/VB/VB_API_Sample/dlgScanner.vb** | Scanner dialog UI. |
| **samples/Java/.../ScannerSubscriptionSamples.java** | Java scanner samples. |
| **source/pythonclient/ibapi/scanner.py** | ScannerSubscription and ScanData classes. |
| **source/cppclient/client/ScannerSubscription.h** | C++ struct. |
| **docs/content/scanners/scanners.txt** | Market Scanners doc: limits, no market data in results, cancel to stop updates. |

---

## 7. Relation to this project

- **Client Portal** (REST) uses different endpoints and field IDs (e.g. snapshot field 31 = last); mapping to TWS tick types is by **meaning** (last, bid, ask, etc.).
- **TWS socket** (if used from this repo) should follow the same connection sequence (connect → connectAck → startApi → nextValidId) and use the same contract fields and tick-type semantics when requesting market data or option chains.
- **Options chain:** TWS uses **reqSecDefOptParams** for expirations/strikes; Client Portal uses **search → strikes → info**. See [CLIENT_PORTAL_OPTIONS_CHAIN.md](CLIENT_PORTAL_OPTIONS_CHAIN.md) and §7 there for the tws-api examples.
- **What we're subscribed to and market hours:** See [MARKET_DATA_SUBSCRIPTIONS_AND_HOURS.md](MARKET_DATA_SUBSCRIPTIONS_AND_HOURS.md) for which symbols we subscribe to, which tick types we consume (Bid, Ask, Last), and how market hours (useRTH, tradingHours) apply today and how they could be configured.
- **Scanner:** We do not use the TWS scanner in this repo today. To add scanner-based discovery (e.g. “hot by volume” or “high option volume P/C ratio”), use **reqScannerParameters** then **reqScannerSubscription** with a **ScannerSubscription** (instrument, locationCode, scanCode, filters); handle **scannerData** / **scannerDataEnd** and **cancelScannerSubscription**. See §6 and [TWS API Market Scanners](https://interactivebrokers.github.io/tws-api/market_scanners.html).
- **Protobuf (TWS wire):** The tws-api Python/C++ clients can send and receive protobuf on the TWS socket (wire format: length + msgId + proto bytes; server version ≥ 201). See [TWS_API_PROTOBUF_LEARNINGS_FROM_SIBLING_REPO.md](TWS_API_PROTOBUF_LEARNINGS_FROM_SIBLING_REPO.md) for encoder/decoder patterns, version gating, and per-message proto types.
