# T-200: MCP Servers Extraction Plan

**Date**: 2025-11-22
**Task**: T-200
**Status**: In Progress
**Purpose**: Extract MCP trading server to standalone public repository

---

## Current State Analysis

### Files to Extract
- `mcp/trading_server/server.py` - Main MCP server
- `mcp/trading_server/bridge.py` - Trading bridge
- `mcp/trading_server/__init__.py` - Package init
- `mcp/trading_server/pyproject.toml` - Package config
- `mcp/trading_server/README.md` - Documentation
- `mcp/trading_server/CYTHON_BINDINGS_GUIDE.md` - Development guide

### Broker-Specific Code Found

**Issues to Fix**:
1. ✅ **TWS_HOST/TWS_PORT** - Already environment variables (GOOD)
2. ✅ **TWS references in docs** - Need to make generic
3. ⚠️ **bridge.py** - References TWS but uses REST API (configurable)
4. ⚠️ **CYTHON_BINDINGS_GUIDE.md** - Contains TWS-specific implementation details

**Action Required**:
- Make TWS references generic (broker-agnostic)
- Update documentation to be broker-agnostic
- Keep REST API approach (already generic)

---

## Extraction Steps

### Step 1: Create Repository Structure
```
trading-mcp-servers/
├── README.md
├── LICENSE (MIT)
├── pyproject.toml
├── .github/
│   ├── workflows/
│   │   └── publish.yml
│   └── ISSUE_TEMPLATE/
├── src/
│   └── trading_mcp_server/
│       ├── __init__.py
│       ├── server.py
│       └── bridge.py
├── docs/
│   └── CYTHON_BINDINGS_GUIDE.md (updated)
└── examples/
    └── config.example.json
```

### Step 2: Make Broker-Agnostic

**Changes Needed**:
1. Replace "TWS" references with "broker" or "trading backend"
2. Update environment variable names to be generic:
   - `TWS_HOST` → `BROKER_HOST` (or keep as is, document as generic)
   - `TWS_PORT` → `BROKER_PORT` (or keep as is, document as generic)
3. Update documentation to mention "broker" instead of "TWS"
4. Keep REST API approach (already generic)

### Step 3: Update Package Configuration

**pyproject.toml Updates**:
- Change name to `trading-mcp-servers`
- Update version to `1.0.0`
- Add proper metadata
- Add MIT license

### Step 4: Create GitHub Repository

**Repository Setup**:
- Name: `trading-mcp-servers`
- Description: "MCP servers for trading operations - broker-agnostic"
- License: MIT
- Public visibility

### Step 5: Update Documentation

**README.md Updates**:
- Remove TWS-specific language
- Make it broker-agnostic
- Add generic configuration examples
- Document REST API integration

---

## Files to Modify

### 1. server.py
- ✅ Already uses environment variables (GOOD)
- ⚠️ Update comments to be broker-agnostic
- ✅ API key validation is generic (GOOD)

### 2. bridge.py
- ✅ Uses REST API (generic) (GOOD)
- ⚠️ Update TWS_HOST/TWS_PORT comments to be generic
- ✅ No hardcoded broker code (GOOD)

### 3. README.md
- ⚠️ Replace "TWS" with "broker" or "trading backend"
- ⚠️ Update configuration examples
- ✅ Keep REST API architecture (generic)

### 4. CYTHON_BINDINGS_GUIDE.md
- ⚠️ Move TWS-specific details to "Advanced Integration" section
- ⚠️ Make main content broker-agnostic
- ✅ Keep as development guide (useful)

---

## Repository Creation Checklist

- [ ] Create GitHub repository `trading-mcp-servers`
- [ ] Add MIT LICENSE file
- [ ] Copy and modify files (make broker-agnostic)
- [ ] Update pyproject.toml
- [ ] Update README.md
- [ ] Create .github/workflows/publish.yml
- [ ] Create examples/config.example.json
- [ ] Test installation: `pip install -e .`
- [ ] Create initial release: v1.0.0
- [ ] Update main repo to reference new repo

---

## Next Steps

1. Create repository structure locally
2. Copy and modify files
3. Make broker-agnostic changes
4. Test locally
5. Create GitHub repository
6. Push code
7. Create release
8. Update main repo
