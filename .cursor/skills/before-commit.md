# Skill: Before commit

**When:** User is about to commit or push.

**Checklist:**

1. **Lint:** Run `lint:run` or `./scripts/run_linters.sh`.
2. **Tests:** Run `just test` or `cargo test` in `agents/backend/`.
3. **Build:** Ensure the active Rust build succeeds (`just build-rust` or `cargo build` in `agents/backend/`).
4. **Secrets:** Confirm no credentials, API keys, or secrets are in the commit (manual check).
5. **Docs:** If you changed behavior or APIs, update docs and optionally run docs health via the repo wrapper (for example `./scripts/run_exarp_go.sh -tool health -args '{\"action\":\"docs\"}'` or `./scripts/run_exarp_go_tool.sh -tool health -args '{\"action\":\"docs\"}'`) so `PROJECT_ROOT` stays correct.

**Proto:** Rust/protobuf generation is handled by the active Rust build where applicable; run `./proto/generate.sh` only if you explicitly need non-Rust codegen.

**Reference:** .cursorrules "Before Committing", AGENTS.md "Commit & Pull Request Guidelines".
