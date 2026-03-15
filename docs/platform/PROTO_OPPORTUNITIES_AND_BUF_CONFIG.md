# Protobuf Opportunities & Buf Config

**Last updated**: 2026-03-14  
**Purpose**: Where else to use protobuf in the repo, and optional buf config tweaks.

## Where protobuf is used today

- **NATS**: `NatsEnvelope` + inner messages (`SystemSnapshot`, `MarketDataEvent`, `StrategyDecision`, etc.). Rust backend publishes and consumes; TUI subscribes.
- **REST snapshot**: Backend serves `RuntimeSnapshotDto` (JSON). TUI and clients receive JSON; NATS path uses proto `SystemSnapshot` and converts to DTO for in-memory use.
- **Rust**: `nats_adapter` (prost) and `api` crate (conversion to/from proto for NATS and snapshot publisher). Internal state is Rust structs (`SystemSnapshot`, etc.); proto at the boundary only.
- **Go**: Generated from `proto/generate.sh`; used in Go agents if they consume NATS.
- **Python**: `python/generated/` from betterproto; optional helper/scripts.
- **TypeScript**: `web/src/generated/proto/` (ts-proto); web is archived.

## Where else protobuf could be used

| Area | Current | Opportunity | Notes |
|------|---------|-------------|--------|
| **Loans API** | REST JSON with `LoanRecord` (serde) in `api/src/loans.rs` | **Done:** `Loan`, `LoanType`, `LoanStatus`, `LoansResponse` in `messages.proto`; `GET /api/v1/loans` and `GET /api/v1/loans/:id` support `Accept: application/x-protobuf` and return proto binary. JSON remains default. | Unifies contract for proto-native clients. |
| **Bank accounts / Discount** | REST JSON (`FrontendBankAccountInput`, etc.) | Proto already has `BankAccount`, `DiscountBankBalance`, `DiscountBankTransaction`. Use them in REST responses or add an optional proto endpoint | Reduces duplication if multiple clients need the same shape. |
| **REST snapshot binary** | Snapshot only as JSON | Optional: `GET /api/v1/snapshot` with `Accept: application/x-protobuf` returning serialized `SystemSnapshot` | Smaller payload, one less conversion for proto-native clients. |
| **Config schema** | JSON Schema `config/schema.json` | Not a fit for protobuf (config is JSON/key-value); keep as-is. | — |
| **Internal Rust-only structs** | `RuntimeSnapshotDto`, `RuntimePositionState`, etc. | Keep as Rust types; proto only at NATS and optional REST boundaries. No need to replace every DTO with proto. | Clear boundary: proto = wire/contract; Rust structs = in-process. |

## Buf config updates (optional)

Current `proto/buf.yaml`:

- `lint: use: [STANDARD], except: [PACKAGE_DIRECTORY_MATCH], enum_zero_value_suffix: _UNSPECIFIED`
- `breaking: use: [FILE], except: [ENUM_VALUE_SAME_NAME]`

**Why we except ENUM_VALUE_SAME_NAME:** Allows enum value *renames* (e.g. for lint compliance) when the numeric value is unchanged. Wire format stays the same; only generated enum names change. Documented in `docs/message_schemas/README.md`.

Optional improvements:

| Change | Purpose |
|--------|---------|
| **`breaking.use: [WIRE_JSON]`** | Stricter: detect changes that break JSON mapping (e.g. field renames). Use if you care about JSON compatibility for the same proto. |
| **`lint.enum_zero_value_suffix: _UNSPECIFIED`** | Already satisfied by `OPTION_TYPE_ENUM_UNSPECIFIED`; set explicitly if you want to enforce for new enums. |
| **`breaking.ignore_unstable_packages: true`** | Ignore breaking changes in packages marked unstable (e.g. `buf.yaml` or package option). Useful if you version by directory. |
| **`deps:`** | Add BSR deps (e.g. `buf.build/googleapis/googleapis`) only if you start importing standard protos; not required for current single-file schema. |

No change required for current setup; apply when you want stricter compatibility or BSR deps.

## Stale planning doc

`docs/planning/PROTOBUF_DEDUP_OPPORTUNITIES.md` says Python expects "messages not yet in the proto" (OptionContract, BoxSpreadLeg, etc.). **Those messages are now in `proto/messages.proto`.** Update that doc to say "proto already defines OptionContract, BoxSpreadLeg, BoxSpreadOpportunity, BankAccount, DiscountBankBalance, etc.; remaining work is wiring Python/TS to generated code and retiring mirrors."

## References

- `proto/messages.proto` — canonical schema
- `docs/message_schemas/README.md` — codegen and lint/breaking
- `docs/planning/PROTOBUF_DEDUP_OPPORTUNITIES.md` — legacy opportunities (update as above)
- `scripts/buf_lint_and_breaking.sh` — run `just proto-validate`
