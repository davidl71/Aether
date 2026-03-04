# Skill: Before commit

**When:** User is about to commit or push.

**Checklist:**

1. **Lint:** Run `lint:run` or `./scripts/run_linters.sh`.
2. **Tests:** Run `test:run` or `ctest --test-dir build --output-on-failure`.
3. **Build:** Ensure build succeeds (`build:debug` or `ninja -C build`). For quiet + JSON result: `build:ai-friendly` or `build:ai-friendly-json`.
4. **Secrets:** Confirm no credentials, API keys, or secrets are in the commit (manual check).
5. **Docs:** If you changed behavior or APIs, update docs and optionally run docs health (exarp `check_documentation_health_tool` with workingDirectory = project root).

**Proto:** C++ is generated at build by CMake; run `./proto/generate.sh` only if you need Python/Go/TypeScript codegen.

**Reference:** .cursorrules "Before Committing", AGENTS.md "Commit & Pull Request Guidelines".
