# TWS API Protobuf Learnings from Sibling tws-api Repo

**Purpose:** What we can learn from the **sibling tws-api** repository about using Protocol Buffers for the TWS socket API—wire format, encoder/decoder patterns, version gating, and per-message proto types. Use this when implementing or aligning a Rust (or other) TWS client that may send or receive protobuf messages to TWS/IB Gateway.

**Sibling path:** `../tws-api` (relative to this project root).  
**References:** `docs/TWS_API_COMPARISON_AND_PROTOBUF.md` (our C++ build and proto generation); `docs/platform/PROTO_OPPORTUNITIES_AND_BUF_CONFIG.md` (our NATS/API proto usage).

---

## 1. Proto source layout and generation

| What | Where |
|------|--------|
| **Proto sources** | `tws-api/source/proto/*.proto` — 200+ files (Contract, Order, MarketDataRequest, ScannerSubscription, SecDefOptParamsRequest, PlaceOrderRequest, etc.) |
| **Syntax** | `syntax = "proto3";` with `optional` fields |
| **Package** | `package protobuf;` (Java: `com.ib.client.protobuf`, C#: `IBApi.protobuf`) |
| **Python output** | `protoc --proto_path=./proto --python_out=./pythonclient/ibapi/protobuf proto/*.proto` → `ibapi/protobuf/*_pb2.py` |
| **C++ (Linux)** | `--experimental_allow_proto3_optional --cpp_out=...` (proto 3.12.4) |
| **C++ (Windows)** | protobuf 5.29.5 (vcpkg) |

Our project: we generate from `tws-api/source/proto` at build time into a single output dir; see `docs/TWS_API_COMPARISON_AND_PROTOBUF.md`. We do **not** use the Python/C# protobuf paths for the Rust backend; this doc is for **patterns and wire format** if we ever add TWS wire-level protobuf in Rust.

---

## 2. Wire format (client → TWS)

From **`tws-api/source/pythonclient/ibapi/comm.py`**:

```python
def make_msg_proto(msgId: int, protobufData: bytes) -> bytes:
    """adds the length prefix"""
    byteArray = msgId.to_bytes(4, 'big') + protobufData
    msg = struct.pack(f"!I{len(byteArray)}s", len(byteArray), byteArray)
    return msg
```

**Layout:**

1. **4-byte big-endian** total length of the following payload.
2. **Payload:** 4-byte big-endian **message ID** + **raw protobuf bytes** (no extra wrapper).

So: `[length: u32 BE][msgId: u32 BE][proto bytes]`. The server uses `msgId` to dispatch to the correct parser; proto schema is per message type (e.g. PlaceOrderRequest, MarketDataRequest).

**Text path (fallback):** When protobuf is not used, `make_msg(msgId, useRawIntMsgId, text)` sends length + (optionally 4-byte msgId +) null-terminated string. Protobuf is chosen only when server version supports it (see §4).

---

## 3. Encoder / decoder split (Python)

| Role | File | Purpose |
|------|------|---------|
| **Encode (domain → proto)** | `client_utils.py` | `createContractProto(contract)`, `createPlaceOrderRequestProto(orderId, contract, order)`, etc. Build proto from domain objects; then `SerializeToString()`. |
| **Decode (proto → domain)** | `decoder_utils.py` | `decodeContract(contractProto)`, `decodeOrder(...)`, `decodeContractDetails(...)`. Use `HasField()` for optional fields; populate domain Contract, Order, Execution, etc. |
| **Send** | `client.py` | `sendMsgProtoBuf(OUT.PLACE_ORDER + PROTOBUF_MSG_ID, serializedString)`; `conn.sendMsg(full_msg)` where `full_msg = make_msg_proto(msgId, msg)`. |
| **Receive** | `client.py` + `decoder` | When `msgId > PROTOBUF_MSG_ID`, subtract `PROTOBUF_MSG_ID` and call `decoder.processProtoBuf(text, msgId)` to parse and dispatch to EWrapper callbacks. |

**Takeaway:** One-to-one mapping: each API request/response type has a dedicated proto (e.g. PlaceOrderRequest, Contract, Order). No single “envelope” type; message ID identifies the schema.

---

## 4. Server version gating

From **`tws-api/source/pythonclient/ibapi/server_versions.py`** and **`client.py`**:

- **`MIN_SERVER_VER_PROTOBUF = 201`** — Base: when `serverVersion() >= 201`, the client may send protobuf for supported message types.
- **Per-message minimums** (examples): Place Order 203, Contract Data 205, Market Data 206, Accounts/Positions 207, Historical Data 208, News 209, Scan 210, etc.
- **`useProtoBuf(msgId)`** — Returns true iff a “unified” protobuf version for that msgId is defined and `unifiedVersion <= self.serverVersion()`.
- **Sending:** For each request the client tries proto first (if `useProtoBuf`); on exception or unsupported, falls back to text encoding.

So: **always check server version** before sending protobuf; have a text fallback for older TWS/Gateway or for messages not yet proto-enabled.

---

## 4.1 Message IDs we care about (reference)

Single reference for TWS message IDs and minimum server version for protobuf. Enables future Rust wire module or C++ alignment. Source: tws-api `server_versions.py` and TWS API message codes.

| Message / area | Purpose | Min server ver (protobuf) |
|----------------|---------|---------------------------|
| **Base** | Protobuf allowed at all | 201 (`MIN_SERVER_VER_PROTOBUF`) |
| **REQ_MKT_DATA / CANCEL_MKT_DATA** | Streaming market data | 206 |
| **PLACE_ORDER / CANCEL_ORDER** | Order submission and cancel | 203 |
| **REQ_POSITIONS / CANCEL_POSITIONS** | Positions request | 207 |
| **REQ_CONTRACT_DATA** | Contract details | 205 |
| **REQ_SEC_DEF_OPT_PARAMS** | Options chain (strikes, expiries) | (see tws-api) |
| **REQ_SCANNER_PARAMETERS / REQ_SCANNER_SUBSCRIPTION / CANCEL_SCANNER_SUBSCRIPTION** | Scanner | 210 |
| **REQ_HISTORICAL_DATA** | Historical bars | 208 |

When implementing a Rust TWS protobuf client or aligning with C++: check `serverVersion() >= min_ver` for the message type before sending proto; otherwise use text encoding.

---

## 5. Request flow (example: place order)

1. Build domain **Contract** and **Order**.
2. **Encode:** `placeOrderRequestProto = createPlaceOrderRequestProto(orderId, contract, order)` (fills PlaceOrderRequest.contract and PlaceOrderRequest.order).
3. **Serialize:** `serializedString = placeOrderRequestProto.SerializeToString()`.
4. **Send:** `sendMsgProtoBuf(OUT.PLACE_ORDER + PROTOBUF_MSG_ID, serializedString)` → `make_msg_proto(OUT.PLACE_ORDER + PROTOBUF_MSG_ID, serializedString)` → socket write.

**Incoming responses** (e.g. openOrder, orderStatus) are decoded in the reader loop: first 4 bytes = msgId; if `msgId > PROTOBUF_MSG_ID`, strip offset and pass payload to `processProtoBuf` for the corresponding response proto (e.g. OpenOrder, OrderStatus).

---

## 6. Proto message types (examples)

| Domain | Request proto (client → TWS) | Response/callback proto (TWS → client) |
|--------|------------------------------|----------------------------------------|
| Market data | MarketDataRequest, CancelMarketData | TickPrice, TickSize, TickString, etc. (decoder_utils maps to domain) |
| Orders | PlaceOrderRequest (Contract + Order), CancelOrderRequest | OpenOrder, OrderStatus, OrderState (decoder_utils) |
| Positions | PositionsRequest, CancelPositions | Position (decoder) |
| Contract details | ContractDataRequest | ContractDetails (Contract + ContractDetails) |
| Options chain | SecDefOptParamsRequest | SecDefOptParameter, SecDefOptParameterEnd |
| Scanner | ScannerParametersRequest, ScannerSubscriptionRequest | ScannerData, ScannerDataEnd |

All live under `source/proto/` with matching `*_pb2.py` (Python) or `*.pb.h`/`*.pb.cc` (C++) after codegen.

---

## 7. Relation to this project

- **Our NATS/proto:** We use **our own** `proto/messages.proto` (NatsEnvelope, SystemSnapshot, MarketDataEvent, etc.) for NATS and optional REST snapshot. That is **independent** of TWS wire format.
- **Our TWS stack:** Rust `ib_adapter` / `tws_yield_curve` and backend TWS market data and positions use the **TWS API C++** client (or a Rust wrapper) that today uses **text** encoding for the socket (or whatever the vendored C++ client uses). We do not currently send or receive TWS protobuf in Rust.
- **If we add TWS protobuf in Rust:** We would need to:
  - Depend on the same `.proto` sources from tws-api (or a vendored copy) and generate Rust with `prost`/`tonic`.
  - Implement the same wire format: 4-byte length + 4-byte msgId + proto bytes.
  - Gate on server version (e.g. 201+) and per-message minimums; fall back to text when unavailable.
  - Mirror the encoder/decoder split: Rust domain types ↔ generated proto types, and a single place that builds the wire message (e.g. `make_msg_proto` equivalent).

**Optional follow-up:** Document or implement a minimal Rust “TWS protobuf wire” module (length + msgId + bytes) and list which message IDs we care about (e.g. REQ_MKT_DATA, PLACE_ORDER, REQ_POSITIONS, REQ_SEC_DEF_OPT_PARAMS) for future use.

---

## 8. Quick reference

| Topic | Location in tws-api |
|-------|---------------------|
| Wire format | `source/pythonclient/ibapi/comm.py` → `make_msg_proto` |
| Send protobuf | `source/pythonclient/ibapi/client.py` → `sendMsgProtoBuf`, `useProtoBuf` |
| Encode (domain → proto) | `source/pythonclient/ibapi/client_utils.py` → `create*Proto` |
| Decode (proto → domain) | `source/pythonclient/ibapi/decoder_utils.py` → `decode*` |
| Server versions | `source/pythonclient/ibapi/server_versions.py` → `MIN_SERVER_VER_PROTOBUF*` |
| Proto definitions | `source/proto/*.proto` |
| TWS API Protobuf readme | `source/ProtoBuf_readme.txt` (if present) or build docs |

---

**See also:** [TWS_API_LEARNINGS_FROM_SIBLING_REPO.md](TWS_API_LEARNINGS_FROM_SIBLING_REPO.md) (contracts, ticks, connection), [TWS_API_COMPARISON_AND_PROTOBUF.md](../TWS_API_COMPARISON_AND_PROTOBUF.md) (our C++ proto generation), [PROTO_OPPORTUNITIES_AND_BUF_CONFIG.md](PROTO_OPPORTUNITIES_AND_BUF_CONFIG.md) (our NATS/API proto), [TWS_PROTOBUF_ACTIONABLE_PLAN.md](../planning/TWS_PROTOBUF_ACTIONABLE_PLAN.md) (phased plan and exarp tasks).
