# NATS Message Schemas

**DEPRECATED** -- JSON schema files have been replaced by
`proto/messages.proto` (protobuf), which is the canonical cross-language
contract.

## Where to find schemas

| Language   | Source                                           |
|------------|--------------------------------------------------|
| Protobuf   | `proto/messages.proto`                           |
| Rust       | `nats_adapter::proto::v1` (prost codegen)        |
| Python     | `python/proto_types.py` (manual mirror)          |
| TypeScript | `web/src/types/proto.ts` (manual mirror)         |
| Go         | Run `proto/generate.sh` for `agents/go/proto/v1` |

## Codegen

Run `./proto/generate.sh` from the repo root to regenerate Python
(betterproto), Go, and TypeScript (ts-proto) bindings.

Rust types are generated automatically by prost via
`nats_adapter/build.rs` during `cargo build`.
