# Cross-Agent TODO Overview

**Last Updated**: 2025-11-18
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
| 12 | Implement REST API layer for web SPA | backend/web | pending |
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

Update this table as tasks progress to keep all agents aligned.
