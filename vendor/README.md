# Vendor Directory

This directory contains third-party dependencies that must be manually downloaded.

## TWS API (Required)

The Interactive Brokers TWS C++ API must be downloaded separately.

### Download Instructions

1. **Visit**: https://interactivebrokers.github.io/
2. **Navigate to**: Downloads → TWS API
3. **Download**: TWS API for your platform (Mac/Linux/Windows)
4. **Extract here**: `vendor/tws-api/`

### Expected Directory Structure

After extraction, you should have:

```
vendor/
└── tws-api/
    ├── IBJts/
    │   ├── source/
    │   │   └── cppclient/
    │   │       ├── client/          ← Headers (.h files)
    │   │       │   ├── EClient.h
    │   │       │   ├── EWrapper.h
    │   │       │   ├── Contract.h
    │   │       │   ├── Order.h
    │   │       │   └── ... (many more)
    │   │       └── src/             ← Implementation (.cpp files)
    │   └── Guides/                  ← Documentation (PDFs)
    └── samples/                     ← Example code
```

### Verification

After extracting, rebuild the project:

```bash
cd /Users/davidlowes/.claude-squad/worktrees/claude_1873e0c42c155fb0
rm -rf build
./scripts/build_universal.sh
```

Look for this message in the build output:
```
CMake Status: TWS API found: /path/to/vendor/tws-api/source/cppclient/client
```

If you see a warning instead, the API is not correctly installed.

### Why Manual Download?

The TWS API has licensing terms that require you to download it directly from Interactive Brokers. We cannot bundle it with this software.

### Version Compatibility

This application is designed for TWS API v10.19 or later. Check the API documentation for compatibility with your TWS/IB Gateway version.

### Support

- **API Documentation**: https://interactivebrokers.github.io/tws-api/
- **IBKR Support**: https://www.interactivebrokers.com/en/support/
- **API Forums**: https://groups.io/g/twsapi

---

**Note**: This directory is in `.gitignore` - the TWS API will not be committed to version control.
