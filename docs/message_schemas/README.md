# NATS Message Schemas

**DEPRECATED** -- JSON schema files have been replaced by
`proto/messages.proto` (protobuf), which is the canonical cross-language
contract.

## Where to find schemas

| Language   | Source                                           |
|------------|--------------------------------------------------|
| Protobuf   | `proto/messages.proto`                           |
| C++        | Generated at build by CMake (`native/generated/`); or run `./proto/generate.sh` |
| Rust       | `nats_adapter::proto::v1` (prost codegen)        |
| Python     | `python/proto_types.py` (manual mirror)          |
| TypeScript | `web/src/types/proto.ts` (manual mirror)         |
| Go         | Run `proto/generate.sh` for `agents/go/proto/v1` |

## Codegen

Run `./proto/generate.sh` from the repo root to regenerate **Python** (betterproto), **Go**, **TypeScript** (ts-proto), and **C++** (optional; C++ is also generated automatically at build by CMake).

Rust types are generated automatically by prost via
`nats_adapter/build.rs` during `cargo build`.

## Further reading

- [awesome-grpc](https://github.com/grpc-ecosystem/awesome-grpc) — Curated gRPC/protobuf ecosystem (libraries, tools, language support). Useful for codegen, proxies, and patterns when using protobuf (e.g. NATS + proto).
