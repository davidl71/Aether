# TWS Protobuf & Our Proto: Actionable Plan

**Purpose:** Turn learnings from the sibling tws-api repo and our own proto docs into a phased, actionable plan with clear outcomes and exarp tasks.

**Sources:** [TWS_API_PROTOBUF_LEARNINGS_FROM_SIBLING_REPO.md](../platform/TWS_API_PROTOBUF_LEARNINGS_FROM_SIBLING_REPO.md), [PROTO_OPPORTUNITIES_AND_BUF_CONFIG.md](../platform/PROTO_OPPORTUNITIES_AND_BUF_CONFIG.md), [PROTOBUF_DEDUP_OPPORTUNITIES.md](PROTOBUF_DEDUP_OPPORTUNITIES.md), [TWS_API_COMPARISON_AND_PROTOBUF.md](../TWS_API_COMPARISON_AND_PROTOBUF.md).

---

## Phase 0 — Documentation & alignment (low effort)

| # | Action | Outcome | Priority |
|---|--------|---------|----------|
| 0.1 | **Document TWS message IDs we care about** | Single reference (in TWS_API_PROTOBUF_LEARNINGS or a small addendum) listing message IDs and min server versions for: REQ_MKT_DATA, CANCEL_MKT_DATA, PLACE_ORDER, CANCEL_ORDER, REQ_POSITIONS, CANCEL_POSITIONS, REQ_CONTRACT_DATA, REQ_SEC_DEF_OPT_PARAMS, REQ_SCANNER_PARAMETERS, REQ_SCANNER_SUBSCRIPTION, CANCEL_SCANNER_SUBSCRIPTION. Enables future Rust wire module or C++ alignment. | Low |
| 0.2 | **Update PROTOBUF_DEDUP_OPPORTUNITIES.md** | Replace “Python expects messages not yet in the proto” with: “Proto already defines OptionContract, BoxSpreadLeg, BoxSpreadOpportunity, BankAccount, DiscountBankBalance, etc.; remaining work is wiring Python/TS to generated code and retiring mirrors.” Aligns with PROTO_OPPORTUNITIES_AND_BUF_CONFIG. | Low |

---

## Phase 1 — Our NATS/API proto (optional)

| # | Action | Outcome | Priority |
|---|--------|---------|----------|
| 1.1 | **Wire NATS loans to return proto binary** | When client requests proto (e.g. Accept header in request envelope or separate subject), `api_handlers.run_loans` returns `LoansResponse` binary; keep JSON as default. | Low |
| 1.2 | **Buf config tweaks (optional)** | Add `breaking.use: [WIRE_JSON]` or `breaking.ignore_unstable_packages: true` if we need stricter compatibility; document in PROTO_OPPORTUNITIES. | Low |

---

## Phase 2 — TWS wire protobuf (future / spike)

| # | Action | Outcome | Priority |
|---|--------|---------|----------|
| 2.1 | **Document decision: Rust TWS proto vs C++ client** | One-pager: when would we add a native Rust TWS client using protobuf wire (tws-api source/proto + prost)? Options: (A) never, keep C++ client; (B) spike only (wire format + one message type); (C) full Rust client later. | Low |
| 2.2 | **Minimal “TWS protobuf wire” module (spike)** | If 2.1 chooses (B) or (C): small Rust module that builds frame `[length: u32 BE][msgId: u32 BE][proto bytes]` and parses same; depend on tws-api `source/proto` for 1–2 message types (e.g. MarketDataRequest, Contract). No production use until decision. | Low |

---

## Task summary

- **Phase 0:** 2 tasks (doc message IDs, update dedup doc).
- **Phase 1:** 2 tasks (loans proto response, buf config).
- **Phase 2:** 2 tasks (decision doc, optional wire spike).

All priorities **low**; no blocking dependencies. Exarp tasks created from this plan are tagged so we can filter (e.g. `proto`, `tws`, `docs`, `nats`).

---

## References

- [TWS_API_PROTOBUF_LEARNINGS_FROM_SIBLING_REPO.md](../platform/TWS_API_PROTOBUF_LEARNINGS_FROM_SIBLING_REPO.md) — wire format, encoder/decoder, version gating.
- [PROTO_OPPORTUNITIES_AND_BUF_CONFIG.md](../platform/PROTO_OPPORTUNITIES_AND_BUF_CONFIG.md) — our NATS/API proto and buf.
- [PROTOBUF_DEDUP_OPPORTUNITIES.md](PROTOBUF_DEDUP_OPPORTUNITIES.md) — Python/TS mirrors and codegen.
- [TWS_API_COMPARISON_AND_PROTOBUF.md](../TWS_API_COMPARISON_AND_PROTOBUF.md) — our C++ proto generation from tws-api.
