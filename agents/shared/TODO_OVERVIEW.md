# Cross-Agent TODO Overview

**Last Updated**: 2025-01-27
**Note**: Go-related tasks (1, 2, 3) removed - backend is implemented in Rust (`agents/backend/`). Market data ingestion and QuestDB integration are handled by the Rust backend service.

| TODO ID | Description | Owner Agent | Status |
|---------|-------------|-------------|--------|
| 4 | Add ANSI colorized output to C++ CLI | backend | pending |
| 5 | Integrate IBKR Client Portal Web API | backend | completed |
| 6 | Design iPad frontend architecture | ipad | in_progress |
| 7 | Implement backend endpoints for iPad app | backend | pending |
| 8 | Create SwiftUI iPad skeleton | ipad | pending |
| 9 | Build iPad dashboards | ipad | pending |
| 10 | Add iPad order controls/testing | ipad | pending |
| 11 | Design web SPA architecture/wireframes | web | pending |
| 12 | Implement REST API layer for web SPA | backend/web | in_progress |
| 13 | Scaffold React/TypeScript web app | web | pending |
| 14 | Build web dashboards | web | pending |
| 15 | Add web strategy controls/testing | web | pending |
| 16 | Design mock TWS API scenarios | backend-mock | in_progress |
| 17 | Implement mock TWS server/config toggle | backend-mock | in_progress |
| 18 | Integrate mock TWS into tests | backend-mock | pending |
| 19 | Prototype IB combo-market data requests | backend-data | pending |
| 20 | Integrate combo quotes into evaluation | backend-data | pending |
| 21 | Capture IB lastLiquidity info | backend-data | pending |
| 22 | Add rebate estimation/nightly reconciliation | backend-data | pending |
| 23 | Design TUI dashboard | tui | completed |
| 24 | Implement TUI front end with live data | tui | in_progress |
| 25 | Expose OHLCV candle data via API | backend-data | pending |
| 26 | Render candlestick charts in web SPA | web | pending |
| 27 | Render candlestick charts in iPad app | ipad | pending |
| 28 | Render candlestick charts in desktop client | desktop | pending |
| 29 | Design TWS TCP proxy for capture/replay | backend | pending |
| 30 | Implement proxy record/replay | backend | pending |
| 31 | Provide tooling to analyze PCAP sessions | backend | pending |
| 32 | Adopt Poetry for Python dependency management | backend | pending |
| 33 | Detect and integrate Livevol data when credentials available | backend | pending |
| 34 | Ensure Apple clients remain compatible with AnyLanguageModel; document low-priority integration hooks | ipad/desktop | pending |
| 35 | Fix day count convention in implied rate calculation (ACT/365, ACT/360, continuous compounding) | backend | pending |
| 36 | Add annualized ROI calculation to BoxSpreadCalculator | backend | pending |
| 37 | Implement portfolio VaR calculation for multiple box spread positions | backend | pending |
| 38 | Implement correlation analysis and covariance matrix calculation | backend | pending |
| 39 | Design and implement mean-variance portfolio optimization framework | backend | pending |
| 40 | Extend Kelly Criterion to multi-asset portfolio optimization | backend | pending |
| 41 | Add dividend-adjusted put-call parity violation calculation | backend | pending |
| 42 | Implement Conditional Value at Risk (CVaR) calculation | backend | pending |
| 43 | Implement Hierarchical Risk Parity (HRP) portfolio optimization | backend | pending |
| 44 | Add individual leg Greeks monitoring for box spreads | backend | pending |

**Mathematical Finance Improvements (T-201 to T-210):** Tasks 35-44 are part of the mathematical finance code improvements initiative. See `docs/analysis/code-improvements-mathematical-finance.md` and `docs/design/portfolio-optimization-framework.md` for detailed specifications.

**CI/CD and Parallel Agent Setup (CI-1 to CI-5):** CI/CD infrastructure tasks for parallel agent workflows. See `docs/TODO2_TASKS_CI_CD_SETUP.md` for detailed task descriptions.

| TODO ID | Description | Owner Agent | Status |
|---------|-------------|-------------|--------|
| CI-1 | Setup GitHub Actions runner on Ubuntu agent | ubuntu | pending |
| CI-2 | Setup GitHub Actions runner on macOS M4 agent | macos | pending |
| CI-3 | Configure enhanced CI/CD workflow for parallel agents | shared | pending |
| CI-4 | Document agent environment and system specifications | shared | pending |
| CI-5 | Test parallel agent CI/CD workflow | shared | pending |

**Parallel Execution Tasks (2025-11-29):** Security and investigation tasks running in parallel. See `docs/PARALLEL_EXECUTION_PLAN.md` for coordination details.

| TODO ID | Description | Owner Agent | Status |
|---------|-------------|-------------|--------|
| T-20251129155002 | Set up environment variable configuration | security/infrastructure | in_progress |
| T-20251129155003 | Write security tests | security/infrastructure | in_progress |
| T-20251129180920-1 | Investigate Exarp script discovery mechanism | investigation/automation | in_progress |

Update this table as tasks progress to keep all agents aligned.
