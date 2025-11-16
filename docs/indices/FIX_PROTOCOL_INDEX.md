# FIX Protocol & FIX API Index

<!--
@index: fix-protocol
@category: fix-protocol
@tags: fix-protocol, fix-api, trading-protocol, standards, c++
@last-updated: 2025-01-27
-->

**Purpose**: Focused index of all FIX protocol related resources for quick reference.

**Full Documentation**: See `API_DOCUMENTATION_INDEX.md` for complete details.

---

## Quick Reference

### FIX Protocol Standards

- **FIX Trading Community**: <https://www.fixtrading.org/>
- **FIX Online Specification**: <https://www.fixtrading.org/online-specification/introduction/>
- **FIXimate**: <https://fiximate.fixtrading.org/> - Interactive FIX protocol reference tool
- **FIX Trading Community GitHub**: <https://github.com/FIXTradingCommunity>

### FIX Development Tools

| Tool | Type | Language | Purpose |
|------|------|----------|---------|
| **QuickFIX** | Library | C++, Java, Python, Ruby, .NET | FIX protocol engine |
| **QuickFIX++** | Library | C++ | C++ FIX protocol implementation |
| **fix8.org** | Library | C++ | FIX protocol library and tools |

### FIX Simulators

| Simulator | Type | Purpose |
|-----------|------|---------|
| **FIXSim.com** | General | FIX protocol testing |
| **Esprow FIX Exchange Simulator** | Professional | Professional FIX testing |
| **B2Bits FIX Client Simulator** | Client | FIX client testing |
| **FIX Trading Simulator** | Open-source | Complete broker-exchange simulation |
| **FIXPusher** | Open-source | FIX message pusher |

### FIX API Providers

| Provider | Focus | Latency | Options Support | Best For |
|----------|-------|---------|-----------------|----------|
| **Tools for Brokers (TFB)** | Platform | Ultra-low | ✅ Verify | Direct CBOE, multi-venue |
| **4T** | Institutional | Ultra-low (LD4) | ✅ Verify | LD4 proximity, PrimeXM |
| **B2PRIME** | Prime of Prime | Low | ⚠️ FOREX/CFD | FOREX/CFD strategies |
| **ATFX** | Broker | Low | ⚠️ Verify | Custom integration |
| **Kraken** | Crypto Derivatives | Ultra-low | ⚠️ Crypto only | Crypto derivatives |
| **OnixS directConnect** | SDK | Ultra-low | ✅ Full | Direct exchange SDK |
| **dxFeed** | Market Data | Low | ✅ Full | Market data via FIX |

---

## Decision Tree

### Which FIX Tool Should I Use?

```
Need FIX protocol library?
  → C++ → QuickFIX++ or fix8.org
  → Java → QuickFIX/J
  → Python → QuickFIX/Python
  → .NET → QuickFIX/N

Need FIX simulator?
  → Professional testing → Esprow FIX Exchange Simulator
  → Open-source → FIX Trading Simulator
  → Client testing → B2Bits FIX Client Simulator
  → General testing → FIXSim.com

Need FIX API provider?
  → Direct CBOE access → Tools for Brokers (TFB) or OnixS directConnect
  → Institutional LD4 → 4T
  → FOREX/CFD → B2PRIME
  → Crypto derivatives → Kraken
  → Market data → dxFeed
```

---

## Key Resources

### Documentation
- **Full Index**: `../API_DOCUMENTATION_INDEX.md#fix-protocol`
- **FIX Protocol Development Tools**: `../API_DOCUMENTATION_INDEX.md#fix-protocol-development-tools`
- **FIX API Providers**: `../API_DOCUMENTATION_INDEX.md#fix-api-providers`

### External Links
- **FIX Trading Community**: <https://www.fixtrading.org/>
- **FIX Online Specification**: <https://www.fixtrading.org/online-specification/introduction/>
- **FIXimate**: <https://fiximate.fixtrading.org/>
- **QuickFIX Engine**: <https://quickfixengine.org/>

---

## See Also

- **Full Documentation**: `../API_DOCUMENTATION_INDEX.md`
- **Summary**: `../API_DOCUMENTATION_SUMMARY.md`
- **NotebookLM Suggestions**: `../NOTEBOOKLM_API_DOCUMENTATION_SUGGESTIONS.md`
