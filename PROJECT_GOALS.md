# Project Goals - IBKR Box Spread Trading System

**Project**: Interactive Brokers Box Spread Arbitrage Generator
**Type**: C++ Trading Application / Multi-platform Agents
**Updated**: 2025-11-27

---

## Vision

A comprehensive trading system for box spread arbitrage on Interactive Brokers. Implements efficient pricing calculations, multi-platform display (CLI/TUI/Web/iOS/iPad), and agent-based architecture for scalable trading operations.

---

## Strategic Phases

### Phase 1: Core Pricing Engine
**Timeline**: Foundation
**Priority**: Critical

**Goals**:
- C++ pricing calculations for box spreads
- APR/yield computation with contract multiplier
- Strike width optimization
- XSP/SPX specific handling

**Keywords**: pricing, calculation, apr, yield, box spread, strike, xsp, spx, native, cpp, core

---

### Phase 2: IB API Integration
**Timeline**: Integration
**Priority**: Critical

**Goals**:
- TWS API client implementation
- Market data streaming
- Order management
- Position tracking
- Risk calculation

**Keywords**: tws, ibkr, api, market data, order, position, risk, client, connection, gateway

---

### Phase 3: Multi-Platform Agents
**Timeline**: Expansion
**Priority**: High

**Goals**:
- Backend agent (Rust) for order execution
- Textual TUI for terminal interface
- Web agent for browser access
- Desktop agent (Swift) for macOS
- iPad/iOS agents for mobile

**Keywords**: agent, backend, tui, web, desktop, ios, ipad, rust, swift, typescript, multi-platform

---

### Phase 4: Risk & Strategy
**Timeline**: Enhancement
**Priority**: High

**Goals**:
- Risk management calculations
- Strategy optimization
- Position sizing
- P&L tracking
- Margin requirements

**Keywords**: risk, strategy, margin, pnl, profit, loss, sizing, optimization, management

---

### Phase 5: Infrastructure & DevOps
**Timeline**: Stabilization
**Priority**: Medium

**Goals**:
- CI/CD pipeline (GitHub Actions)
- Universal binary builds
- Ansible deployment
- ib-gateway Docker setup
- Testing framework

**Keywords**: ci, cd, github actions, build, deploy, ansible, docker, test, infrastructure, devops

---

### Phase 6: Documentation & Polish
**Timeline**: Documentation
**Priority**: Medium

**Goals**:
- API documentation
- User guides
- Architecture docs
- Trading workflow guides
- Best practices

**Keywords**: documentation, docs, readme, guide, api, architecture, workflow

---

## Design Constraints

### Code Quality Standards
- **C++ Standard**: C++20
- **Style**: 2-space indentation, Allman braces
- **Testing**: All pricing/risk calculations must have tests
- **Security**: Never commit credentials or API keys

### Trading Safety
- Always use paper trading port (7497) for development
- Gate live trading behind configuration flags
- Validate all orders before submission
- Implement position limits

---

## Success Metrics

| Metric | Target | Current |
|--------|--------|---------|
| Pricing Accuracy | 99.9% | ✅ |
| Test Coverage | 80% | TBD |
| Build Success | 100% | ✅ |
| Doc Coverage | High | TBD |

---

## Infrastructure Keywords

Tasks matching these keywords are considered infrastructure/support work (not misaligned):

- research, analysis, review, investigation
- config, configuration, setup, infrastructure
- testing, test, unittest, catch2
- documentation, docs, readme
- build, cmake, ninja, universal
- refactor, cleanup, optimization
- ci, cd, github actions, deploy

---

## Task Categories

### CODE-TODO-*
Tasks for resolving TODO comments in specific modules. These are intentionally similar in format but target different code areas.

### SHARED-*
Cross-platform features that need implementation across multiple agents (web, iPad, desktop).

### CI-*
CI/CD setup tasks for different agents/runners.

---

## File Format

This file is parsed by the Todo2 alignment analyzer. Structure:

```markdown
### Phase N: Name
**Keywords**: comma, separated, keywords
```

Keywords are case-insensitive and matched against task content, description, and tags.
