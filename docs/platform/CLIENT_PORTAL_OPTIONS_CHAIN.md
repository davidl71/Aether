# Client Portal API: Options Chain Flow

**Purpose:** Document the mandatory sequence and endpoints for retrieving options chain data via the IB Client Portal Web API (REST). This flow is required when building contract libraries or resolving option strikes/expiries from the Gateway.

**Source:** Based on IBKR article "Handling Options Chains" (Andrew Wise). The Client Portal API requires calling three endpoints in order; there is no single-call options chain endpoint.

**Related:** [IB_REAL_POSITIONS.md](../runbooks/IB_REAL_POSITIONS.md) (Client Portal setup), [IB_SNAPSHOT_ARCHITECTURE_DEEP_DIVE.md](../IB_SNAPSHOT_ARCHITECTURE_DEEP_DIVE.md) (secdef/search usage), [TWS_ORATS_PORTAL_QUESTDB.md](../TWS_ORATS_PORTAL_QUESTDB.md) (Client Portal vs TWS).

---

## 1. Mandatory sequence

The Client Portal API **requires** calling these endpoints **in this order**:

1. **`/iserver/secdef/search`** – Resolve symbol to underlying `conid` and option expiration months.
2. **`/iserver/secdef/strikes`** – Get strike prices for a given underlying and month.
3. **`/iserver/secdef/info`** – Get contract details (conid, symbol, strike, maturityDate, etc.) per strike (and optionally per expiry within the month).

There is no way to skip steps or call them out of order for options chain data.

---

## 2. Base URL and auth

- **Base URL:** `https://localhost:5001/v1/api` (local Client Portal Gateway). Port may differ; use `IB_PORTAL_URL` if the project config exposes it.
- **Auth:** Session cookie after logging in via the Gateway UI. Requests typically use `verify=False` when using the local Gateway (self-signed cert).
- **Use case:** Suited for building contract libraries once (e.g. start of day or start of month), not for high-frequency intraday chains.

---

## 3. Endpoint details

### 3.1 Search – resolve underlying and option months

**Request:**

- **Path:** `GET /iserver/secdef/search`
- **Query:** `symbol` (required), `listingExchange` (optional but recommended to filter exact listing).

**Response:** Array of contracts. For each contract:

- `conid` – Contract ID (underlying conid for step 2).
- `description` – Often matches exchange name (e.g. `"NASDAQ"`); use to select the desired listing.
- `sections` – Array of objects with:
  - `secType` – e.g. `"OPT"` for options.
  - `months` – Semicolon-separated list of option expiration months (e.g. `"202505;202506"`).

**Typical logic:**

- Filter contract where `description == listingExchange` (e.g. `"NASDAQ"`).
- From that contract, find the `secType` with `secType == "OPT"` and take `months = secType["months"].split(';')`.
- Use `conid` as `underConid` and one of `months` (e.g. first = front month) for the next calls.

---

### 3.2 Snapshot – last price (for filtering strikes)

Used to decide which strikes are “around the money” (e.g. for building a focused chain).

**Request:**

- **Path:** `GET /iserver/marketdata/snapshot`
- **Query:** `conids={underConid}&fields=31`  
  (Field `31` = last price.)

**Note:** The snapshot endpoint may require a preflight request (first call returns nothing; second returns data). Call twice if needed.

**Response:** Array of objects; index by `conid` and field key, e.g. `response[0]["31"]` for last price.

---

### 3.3 Strikes – list strike prices

**Request:**

- **Path:** `GET /iserver/secdef/strikes`
- **Query:**
  - `conid` – Underlying conid from search.
  - `secType=OPT` (equity options; use another type for futures options).
  - `month` – One of the month strings from search (e.g. first element of `months`).

**Response:** Object with keys such as `"put"` and `"call"`, each an array of strike prices. Put and call strikes are often but not always identical; choose the side(s) you need.

**Typical logic:** Filter strikes to a range (e.g. within ±$10 of snapshot price, or ±5% of spot) to get “in the money” or “near the money” strikes for the next step.

---

### 3.4 Info – contract details per strike

**Request:**

- **Path:** `GET /iserver/secdef/info`
- **Query:**
  - `conid` – Underlying conid.
  - `month` – Same month string as in strikes.
  - `strike` – One strike from the strikes response.
  - `secType=OPT`
  - `right=P` or `right=C` (put vs call).

**Response:** Array of contracts (multiple expiries within the month, e.g. weeklies). Each object can include:

- `conid` – Option contract ID.
- `symbol`
- `strike`
- `maturityDate`

**Typical logic:** For each strike in the filtered strike list, call info once per side (P/C). Either take all expiries in the month or filter by `maturityDate` to a single expiry.

---

## 4. End-to-end flow (summary)

| Step | Action | Output |
|------|--------|--------|
| 1 | `GET /iserver/secdef/search?symbol=AAPL` (optional: filter by `listingExchange`) | `underConid`, `months[]` |
| 2 | `GET /iserver/marketdata/snapshot?conids={underConid}&fields=31` (×2 if preflight) | Last price for underlying |
| 3 | `GET /iserver/secdef/strikes?conid={underConid}&secType=OPT&month={month}` | Put/call strike lists |
| 4 | Filter strikes (e.g. ±$10 or ±5% of snapshot) | `itmStrikes[]` |
| 5 | For each strike: `GET /iserver/secdef/info?conid=...&month=...&strike=...&secType=OPT&right=P` (and/or `right=C`) | Contract details (conid, symbol, strike, maturityDate) per expiry |
| 6 | Optionally write to CSV/DB for the session or month | Contract library |

---

## 5. Project context

- **TWS vs Client Portal:** This repo uses **TWS API** (socket) for live trading and market data in the native path, and **Client Portal** (REST) for positions and some snapshot/secdef usage. Options chain discovery via Client Portal is optional and typically used for contract library build or one-off resolution; real-time chains may use TWS reqSecDefOptParams / contract details instead.
- **Rust backend:** Any Client Portal options chain implementation would call the same three endpoints in order from the backend (e.g. via `reqwest` or existing HTTP client). See `api::ib_positions` and snapshot/secdef usage in the codebase for existing Client Portal patterns.
- **Caching:** Underlying `conid` and option months can be cached per symbol/listing to avoid repeated search calls when building chains for the same underlyings.

---

## 6. References

- IBKR Campus – Client Portal API: [ibkrcampus.com](https://ibkrcampus.com/ibkr-api-page/cpapi-v1/)
- Article: “Handling Options Chains” (Andrew Wise), describing the search → snapshot → strikes → info flow and CSV export (Python).
- Project: [IB_REAL_POSITIONS.md](../runbooks/IB_REAL_POSITIONS.md), [TWS_ORATS_PORTAL_QUESTDB.md](../TWS_ORATS_PORTAL_QUESTDB.md).

---

## 7. TWS API (sibling tws-api repo) examples

The **sibling repository** `tws-api` (e.g. `../tws-api` relative to this project) contains the official IBKR TWS API. It provides **one-call** options chain parameters over the socket API, which contrasts with the Client Portal’s **three-call** REST sequence.

### 7.1 reqSecDefOptParams – chain parameters in one request

**Request (TWS socket):**

- **Method:** `reqSecDefOptParams(reqId, underlyingSymbol, futFopExchange, underlyingSecType, underlyingConId)`
- **Parameters:** Same logical inputs as Client Portal search + month: underlying symbol (e.g. `"AAPL"`, `"IBM"`), optional futures exchange, underlying type (e.g. `"STK"`), and underlying conid (e.g. `265598` for AAPL, `8314` for IBM).

**Response (EWrapper callbacks):**

- **`securityDefinitionOptionParameter(reqId, exchange, underlyingConId, tradingClass, multiplier, expirations, strikes)`** – Called one or more times per request. Delivers:
  - `expirations` – Set of expiry strings (analogous to Client Portal “months” / expiries).
  - `strikes` – Set of strike prices (analogous to Client Portal strikes response).
  - Plus exchange, tradingClass, multiplier.
- **`securityDefinitionOptionParameterEnd(reqId)`** – Signals end of the option chain parameter stream for that reqId.

So in one TWS request you receive **expirations + strikes** (and exchange/tradingClass/multiplier); no separate “search” then “strikes” then “info” as in Client Portal.

### 7.2 Example files in sibling tws-api

| Path (under `tws-api/`) | Description |
|-------------------------|-------------|
| **Extended Samples/Python/Contracts/reqSecDefOptParams.py** | Minimal Python sample: connects, calls `reqSecDefOptParams(123, "AAPL", "", "STK", 265598)` after `nextValidId`, prints `securityDefinitionOptionParameter` and `securityDefinitionOptionParameterEnd`. |
| **samples/Python/Testbed/Program.py** | Full testbed; includes `reqSecDefOptParams(0, "IBM", "", "STK", 8314)` and handlers for `securityDefinitionOptionParameter` / `securityDefinitionOptionParameterEnd` (around lines 1172–1216). |
| **samples/VB/Testbed/MainModule.vb** | VB example: `client.reqSecDefOptParams(0, "IBM", "", "STK", 8314)`. |
| **samples/VB/VB_API_Sample/MainForm.vb** | UI button “Req Sec Def Opt Params” and dialog `dlgSecDefOptParamsReq` (reqId, symbol, exchange, secType, conId); displays expirations and strikes in callback. |
| **source/pythonclient/ibapi/client.py** | `reqSecDefOptParams` / `reqSecDefOptParamsProtoBuf` (around 6757–6835). |
| **source/cppclient/client/EClient.cpp** | `reqSecDefOptParams` / `reqSecDefOptParamsProtoBuf`. |
| **source/cppclient/client/EWrapper_prototypes.h** | `securityDefinitionOptionalParameter(..., expirations, strikes)` and ProtoBuf variants `secDefOptParameterProtoBuf` / `secDefOptParameterEndProtoBuf`. |
| **source/cppclient/client/EDecoder.cpp** | Decoding of option parameter message into expirations and strikes sets; invokes EWrapper callbacks. |

### 7.3 Getting full contract details (TWS)

After you have **expirations** and **strikes** from `reqSecDefOptParams`, full option contract details in TWS are obtained via **`reqContractDetails(reqId, contract)`** with an **Option** contract (symbol, strike, expiry, right, exchange, etc.). The **ContractDetails.py** sample shows `reqContractDetails` for a stock; for options you would build an Option contract from the secDef opt parameter data and request details per contract (or batch as needed).

### 7.4 Client Portal vs TWS (options chain)

| Aspect | Client Portal (REST) | TWS API (socket, tws-api repo) |
|--------|----------------------|---------------------------------|
| **Chain parameters** | Three steps: search → strikes → info | One request: `reqSecDefOptParams` → `securityDefinitionOptionParameter` (+ End) |
| **Expirations / strikes** | From search “months” and `/secdef/strikes` | From `securityDefinitionOptionParameter(…, expirations, strikes)` |
| **Full contract details** | `/iserver/secdef/info` per strike/right | `reqContractDetails` with Option contract(s) |
| **Transport** | HTTPS, session cookie | TWS/Gateway socket (e.g. 7496/7497) |

Use the sibling **tws-api** repo for runnable examples of the TWS options chain flow when implementing or comparing against the Client Portal three-step flow documented above.
