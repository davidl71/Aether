# Abstractions, DSLs & Frameworks Beyond Protobuf

**Purpose:** A short index of abstractions and frameworks that pair well with Protobuf in this repo. Protobuf remains the single source for **wire shapes and cross-language DTOs**; these fill other roles (expressions, config, API docs, codegen, trading infra).

---

## 1. Expression / rule DSLs

| Tool | Role | When to use | Where in this repo |
|------|------|-------------|--------------------|
| **CEL (Common Expression Language)** | Safe, side-effect-free expressions over a context (e.g. `roi_percent >= 4.0 && spread_bps < 20`). | Strategy filters, alert/risk rules, config-driven conditions without code changes. | Config `strategy.filter_expr` or a proto `StrategyRule.expression`; evaluate in C++ (cel-cpp), Rust (cel-interpreter), or Python (cel-python / google/cel-py). |
| **Tiny custom DSL** | Minimal expression grammar (e.g. `symbol in ["SPX","XSP"] && dte >= 30`). | When you want full control and no extra deps; more work to maintain. | Same place as CEL; implement a small parser in the language that runs the strategy. |

**Reference:** `docs/design/DSL_AND_PROTO_OPPORTUNITIES.md` — recommends “more protobuf for data; optional CEL (or small DSL) for expressions.”

---

## 2. Schema & config

| Tool | Role | When to use | Where in this repo |
|------|------|-------------|--------------------|
| **JSON Schema** | Validate JSON config (TUI/PWA/standalone) and REST payloads. | Single config format across clients; fail-fast on bad config. | `config/config.*.json`; Python `jsonschema`, TypeScript `ajv`; C++ optional (e.g. nlohmann/json + json-schema-validator). |
| **OpenAPI (Swagger)** | Describe REST API (paths, request/response shapes). | API docs, client generation, contract tests. | REST layer (e.g. FastAPI auto-OpenAPI); keep proto for NATS and internal contracts. |
| **AsyncAPI** | Describe async/message APIs (NATS topics, payloads). | Docs and codegen for NATS publishers/subscribers. | Optional: describe NATS subjects and reference `proto/messages.proto` for payload schema. |

**Reference:** `docs/research/architecture/SHARED_CONFIGURATION_SCHEMA.md` — JSON Schema for config.

---

## 3. Protobuf ecosystem (beyond `protoc`)

| Tool | Role | When to use | Where in this repo |
|------|------|-------------|--------------------|
| **Buf (bufbuild)** | Lint, format, break detection, and optional schema registry for `.proto` files. | Consistent style; detect breaking changes before they hit consumers. | `buf lint` / `buf breaking` in CI; optional Buf Schema Registry for versioned proto. |
| **grpc-gateway** | Generate REST ↔ gRPC bridge from proto + annotations. | If you add gRPC and want REST in front of it. | Not used today (NATS + REST only); consider only if you introduce gRPC. |
| **Connect (Buf)** | RPC framework on top of HTTP/2 and proto; optional gRPC compatibility. | If you want typed RPC with proto without full gRPC stack. | Alternative to “REST + proto payloads” if you standardise on RPC. |

**Current:** `proto/generate.sh` + CMake for C++; betterproto for Python; prost for Rust; ts-proto for TypeScript; protoc-gen-go for Go.

---

## 4. Codegen & single-source patterns

| Tool | Role | When to use | Where in this repo |
|------|------|-------------|--------------------|
| **Protobuf** | Single schema → C++ / Python / Rust / Go / TS. | All cross-language DTOs and wire format (already in use). | `proto/messages.proto`; `./proto/generate.sh`; CMake for C++. |
| **pybind11 / Litgen** | C++ ↔ Python bindings. | Not currently in use — Python layer is archived. If a Python consumer is reintroduced, pybind11 via `pybind11_add_module` is the preferred approach. | N/A (source and CMake targets removed). |

---

## 5. Trading / finance frameworks (reference only)

| Tool | Role | When to use | Where in this repo |
|------|------|-------------|--------------------|
| **NautilusTrader** | Python-first trading framework (data, execution, backtest). | Strategy research, execution adapter, backtesting. | Referenced in integration; C++ remains canonical for box spread math. |
| **LEAN (QuantConnect)** | Cloud backtesting and live trading. | Alternative backtest/live stack; REST wrapper exists. | `python/lean_integration/`; proto/types can feed into LEAN where useful. |
| **SmartQuant / FLOX** | C++ trading frameworks. | Reference for patterns; not a direct dependency. | Docs only (`docs/API_DOCUMENTATION_INDEX.md`, research). |

---

## 6. State machines & workflow

| Tool | Role | When to use | Where in this repo |
|------|------|-------------|--------------------|
| **XState / statecharts** | Declarative state machines (e.g. order/strategy lifecycle). | Clear order/strategy states and transitions in UI or services. | Optional: TUI/PWA or Rust/Python services if you formalise lifecycle. |
| **Temporal / Cadence** | Durable workflows (retries, sagas). | Multi-step, long-running flows (e.g. multi-leg execution, reconciliation). | Only if you need durable workflow engine; otherwise keep logic in existing runners. |

---

## Recommended split (summary)

| Need | Use | Notes |
|------|-----|--------|
| Message shapes, API contracts, cross-language DTOs | **Protobuf** | `proto/messages.proto`; already canonical. |
| Config validation, REST payload validation | **JSON Schema** | Optional; aligns with shared config design. |
| Strategy/alert conditions without code changes | **CEL or small DSL** | Optional; see DSL_AND_PROTO_OPPORTUNITIES.md. |
| Proto linting and breaking-change checks | **Buf** | Optional; improves proto hygiene. |
| Python ↔ C++ (box spread, risk) | **pybind11** | Default bindings backend; single CMake build. |
| REST API docs and client generation | **OpenAPI** | If you want generated REST clients; proto stays for NATS. |

---

## References

- `docs/design/DSL_AND_PROTO_OPPORTUNITIES.md` — Where DSL vs proto fits; CEL for expressions.
- `docs/message_schemas/README.md` — Single proto story; codegen per language.
- `docs/planning/CROSS_LANGUAGE_DEDUP_PLAN.md` — Protobuf + single-source strategy (implemented).
- `proto/generate.sh` — Current codegen (C++/Python/Go/TS); Rust via prost in `nats_adapter`.
