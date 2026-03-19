# Protobuf Opportunities & Buf Config

**Last updated**: 2026-03-15  
**Purpose**: Where else to use protobuf in the repo, and optional buf config tweaks.

## Loan proto and REST snapshot status (optional task)

- **Loan proto:** `Loan`, `LoanType`, `LoanStatus`, `LoansResponse` are already in `proto/messages.proto` (§ Loans). No proto change required. The `api` crate has `loan_record_to_proto` and `LoansResponse` for binary responses.
- **REST snapshot proto:** Deferred. See table row "REST snapshot binary" — when a gateway exposes `GET /api/v1/snapshot`, add `Accept: application/x-protobuf` support using `snapshot_to_proto` from `snapshot_publisher.rs`. No implementation in this repo (NATS-only).
- **Follow-up (optional):** Wire NATS loans handlers to return proto binary when the client requests it (e.g. request envelope or separate subject), in addition to current JSON.

## api.* request/reply and proto

Backend api.* request/reply uses **queue subscriptions** (queue group from `NATS_API_QUEUE_GROUP`, default `api`) so multiple backends can scale out; see `docs/platform/NATS_API.md`. Current api.* handlers use **JSON**; **new** api.* endpoints may use protobuf via `nats_adapter::rpc` (subscribe_proto / request_proto) and messages in `proto/messages.proto`. No change to existing JSON handlers.

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
| **Loans API** | NATS-only (`api.loans.list`, `api.loans.get`) with JSON in `api_handlers.rs`; `api/src/loans.rs` has `LoanRecord` (serde) and `loan_record_to_proto` / `LoansResponse` (proto). | **Proto:** `Loan`, `LoanType`, `LoanStatus`, `LoansResponse` in `messages.proto`; conversion in `loans.rs`. **REST:** Not implemented (backend is NATS-only per NATS_API.md). NATS handlers currently return JSON; proto response path exists in code but is not wired. | To support proto over NATS, have `api_handlers.run_loans` accept a request flag or separate subject and return `LoansResponse` / `Loan` binary. |
| **Bank accounts / Discount** | REST JSON (`FrontendBankAccountInput`, etc.) | Proto already has `BankAccount`, `DiscountBankBalance`, `DiscountBankTransaction`. Use them in REST responses or add an optional proto endpoint | Reduces duplication if multiple clients need the same shape. |
| **REST snapshot binary** | Snapshot only as JSON (or NATS-only in this repo) | **Deferred:** `GET /api/v1/snapshot` with `Accept: application/x-protobuf` returning serialized `SystemSnapshot`. No REST snapshot endpoint exists in this repo; snapshot is published via NATS only (`snapshot.{backend_id}`). When a gateway or service exposes `GET /api/v1/snapshot`, add content negotiation: if `Accept` includes `application/x-protobuf`, return `snapshot_to_proto(&snapshot)` (see `backend_service/src/snapshot_publisher.rs`) with `Content-Type: application/x-protobuf`; otherwise JSON. | Smaller payload, one less conversion for proto-native clients. |
| **Config schema** | JSON Schema `config/schema.json` | Not a fit for protobuf (config is JSON/key-value); keep as-is. | — |
| **Internal Rust-only structs** | `RuntimeSnapshotDto`, `RuntimePositionState`, etc. | Keep as Rust types; proto only at NATS and optional REST boundaries. No need to replace every DTO with proto. | Clear boundary: proto = wire/contract; Rust structs = in-process. |

## Buf config updates (optional)

**For CI/build:** Run `buf lint` and `buf breaking` from `proto/` (e.g. `just proto-validate` or `scripts/buf_lint_and_breaking.sh`). Optional tweaks (WIRE_JSON, `ignore_unstable_packages`) are in `proto/buf.yaml` and the table below.

Current `proto/buf.yaml`:

- `lint: use: [STANDARD], except: [PACKAGE_DIRECTORY_MATCH], enum_zero_value_suffix: _UNSPECIFIED`
- `breaking: use: [FILE], except: [ENUM_VALUE_SAME_NAME]`

**Why we except ENUM_VALUE_SAME_NAME:** Allows enum value *renames* (e.g. for lint compliance) when the numeric value is unchanged. Wire format stays the same; only generated enum names change. Documented in `docs/message_schemas/README.md`.

**Applied (Phase 1.2):** `proto/buf.yaml` now sets `breaking.ignore_unstable_packages: true` and documents switching `breaking.use` to `WIRE_JSON` for stricter JSON compatibility when needed. See comments in `proto/buf.yaml`.

Optional improvements (remaining):

| Change | Purpose |
|--------|---------|
| **`breaking.use: [WIRE_JSON]`** | Stricter: detect changes that break JSON mapping (e.g. field renames). Use if you care about JSON compatibility for the same proto. |
| **`lint.enum_zero_value_suffix: _UNSPECIFIED`** | Already satisfied by `OPTION_TYPE_ENUM_UNSPECIFIED`; set explicitly if you want to enforce for new enums. |
| **`breaking.ignore_unstable_packages: true`** | Ignore breaking changes in packages marked unstable (e.g. `buf.yaml` or package option). Useful if you version by directory. |
| **`deps:`** | Add BSR deps (e.g. `buf.build/googleapis/googleapis`) only if you start importing standard protos; not required for current single-file schema. |

No change required for current setup; apply when you want stricter compatibility or BSR deps.

## GET /api/v1/snapshot Accept: application/x-protobuf (optional)

**Status:** Implemented (optional, 2026-03-15).

**Behaviour:** When the backend service is started with `REST_SNAPSHOT_PORT` set (e.g. `REST_SNAPSHOT_PORT=8080`), it serves `GET /api/v1/snapshot`:

1. Read current snapshot from shared state.
2. If `Accept` header includes `application/x-protobuf`, serialize with the same proto used for NATS: `snapshot_to_proto(&snapshot)` from `snapshot_publisher.rs`, encode with `prost::Message::encode_to_vec`, set `Content-Type: application/x-protobuf`, and return the bytes.
3. Otherwise return JSON (`RuntimeSnapshotDto`) with `Content-Type: application/json`.

**Default:** If `REST_SNAPSHOT_PORT` is not set, no HTTP server is started; snapshot remains NATS-only (`snapshot.{backend_id}`).

## Stale planning doc

`docs/planning/PROTOBUF_DEDUP_OPPORTUNITIES.md` says Python expects "messages not yet in the proto" (OptionContract, BoxSpreadLeg, etc.). **Those messages are now in `proto/messages.proto`.** Update that doc to say "proto already defines OptionContract, BoxSpreadLeg, BoxSpreadOpportunity, BankAccount, DiscountBankBalance, etc.; remaining work is wiring Python/TS to generated code and retiring mirrors."

## References

- `proto/messages.proto` — canonical schema
- `docs/message_schemas/README.md` — codegen and lint/breaking
- `docs/planning/PROTOBUF_DEDUP_OPPORTUNITIES.md` — legacy opportunities (update as above)
- `scripts/buf_lint_and_breaking.sh` — run `just proto-validate`
- [NATS_API.md](NATS_API.md) — NATS-only API; §2 contract says prefer protobuf for new endpoints; current `api.*` handlers use JSON; proto response paths (e.g. loans) documented here.
