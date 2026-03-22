# Decimal Library Migration Planning Guide

## Purpose

Provide future maintainers with a clear roadmap for transitioning from Intel's IntelRDFPMathLib (libbid) dependency to the C++ Alliance Boost.Decimal library while preserving trading accuracy, performance, and build reproducibility.

## Background Summary

- Current builds link `libbid.a` directly via `native/CMakeLists.txt`, `native/ibapi_cmake/CMakeLists.txt`, and `native/tests/CMakeLists.txt`.
- Intel's library exposes a C ABI and Binary Integer Decimal (BID) encoding required by the vendored TWS API sources.
- Boost.Decimal (C++ Alliance) targets modern C++14+ with templated decimal types (`decimal32_t`, `decimal64_t`, `decimal128_t`) and supports BID/DPD encoding but lacks a drop-in C wrapper.
- Prior internal analysis (see `docs/TWS_BUILD_PROGRESS.md` and `docs/TWS_BUILD_COMPLETE.md`) documents how Intel dependencies were previously integrated.

## Migration Goals

- Remove the Intel proprietary dependency while retaining IEEE 754-2008 decimal semantics.
- Keep the TWS API client operable across x86_64 and arm64 universal builds.
- Maintain numerical parity for pricing, risk metrics, and TSV output.
- Minimize long-term maintenance overhead for future TWS API updates.

## Workstream Overview

1. Feasibility & Spike (T3/T4 scope)
   - Inventory Intel-specific symbols referenced by the TWS API sources (`___bid64_*`, `___bid128_*`).
   - Prototype an adapter that maps Intel-style calls to Boost.Decimal operations or evaluate direct refactor requirements.
   - Benchmark representative option calculations to gauge performance differences.
2. Build & Tooling Updates
   - Replace `INTEL_DECIMAL_LIB` variables in all CMake files with Boost.Decimal components (FetchContent, vendored submodule, or system Boost ≥1.86 once released).
   - Update helper scripts (`scripts/setup_worktree.sh`, Ansible roles) to download/build Boost.Decimal instead of IntelRDFPMathLib.
   - Ensure universal builds still succeed and export `compile_commands.json`.
3. Code Integration
   - Option A: Implement C-compatible shim exposing the Intel ABI while internally delegating to Boost.Decimal types.
   - Option B: Refactor TWS API decimal utilities (`Decimal.cpp`, parsing helpers) to use Boost.Decimal directly.
   - Validate serialization/deserialization paths for market data payloads and order submission structs.
4. Validation & Rollout
   - Construct side-by-side regression harness comparing Intel vs. Boost decimal outputs for pricing scenarios.
   - Run full test suite (`ctest --preset macos-universal-debug`) and capture benchmarks.
   - Document findings, update READMEs, and communicate rollout steps to stakeholders.

## Detailed Phased Plan

| Phase | Duration (est.) | Deliverables |
|-------|-----------------|--------------|
| Discovery & Spike | 3 days | Symbol inventory, adapter feasibility notes, decision on integration approach |
| Implementation | 5 days | Updated build scripts, shim/refactored decimal usage, compiling TWS client |
| Validation | 3 days | Regression comparison report, performance benchmarks, updated automated tests |
| Documentation & Handoff | 1 day | Revised docs (`docs/TWS_*`, `docs/API_DOCUMENTATION_INDEX.md`), change log, rollout checklist |

## Key Tasks Checklist

- [ ] Enumerate Intel-specific headers and macros relied upon (`bid_functions.h`, etc.).
- [ ] Decide on shim vs. refactor strategy and document rationale.
- [ ] Integrate Boost.Decimal dependency management into CMake presets.
- [ ] Update CI/worktree scripts to fetch new dependency.
- [ ] Implement compatibility layer or refactor TWS decimal code.
- [ ] Add regression tests comparing decimal outputs across both libraries.
- [ ] Re-run full regression suite and capture performance metrics.
- [ ] Finalize documentation updates and communicate migration plan.

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| API mismatch between Intel C ABI and Boost.Decimal | Build failures or runtime errors in TWS API | Prototype adapter early; maintain Intel symbol compatibility if TWS updates lag |
| Numerical drift in pricing outputs | Incorrect arbitrage assessments | Build differential harness using archived market scenarios; define acceptable tolerances |
| Performance regressions | Slower CLI runs or TWS callbacks | Benchmark on arm64 + x86_64; profile hotspots; consider specialization for common precisions |
| Upstream TWS updates reintroduce Intel-specific code | Rework required after releases | Automate adapter generation; document patch process |

## Validation Strategy

- Extend existing Catch2 tests with fixtures that compute spreads using both libraries and assert equality within defined ULP tolerances.
- Capture CLI TSV outputs before/after migration for three standard strike widths and store in `tests/golden/` fixtures.
- Smoke-test on both architectures and ensure fat binary still packages correctly.

## Dependencies & References

- Intel library build notes: `native/third_party/tws-api/IBJts/source/cppclient/Intel_lib_build.txt`
- Internal progress logs: `docs/TWS_BUILD_PROGRESS.md`, `docs/TWS_BUILD_COMPLETE.md`
- Boost.Decimal updates (2025): <https://cppalliance.org/matt/2025/10/06/Matts2025Q3Update.html?utm_source=openai>
- Intel Decimal library overview: <https://www.intel.com/content/www/us/en/developer/articles/tool/intel-decimal-floating-point-math-library.html?utm_source=openai>

## Handoff Notes for Future Agents

- Maintain this document alongside the migration issue tracker entry; update checkboxes and timing estimates as tasks complete.
- Capture empirical test outputs in version control to simplify future audits.
- Prefer incremental rollout via feature branch with toggled dependency in CMake cache to allow quick rollback if regressions surface.
