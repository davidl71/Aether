# NATS Dependencies Added - Phase A Complete

**Date**: 2025-11-22
**Status**: ✅ Phase A Complete - Dependencies Added

---

## Dependencies Added

### ✅ 1. Python - nats-py
**File**: `requirements.in`
- Added: `nats-py>=2.6.0`
- Status: Ready to install
- Next: Run `pip-compile requirements.in` to update `requirements.txt`

### ✅ 2. TypeScript - nats.ws
**File**: `web/package.json`
- Added: `nats.ws: ^2.0.0` to dependencies
- Status: Ready to install
- Next: Run `npm install` in `web/` directory

### ✅ 3. C++ - nats.c (Optional)
**File**: `native/CMakeLists.txt`
- Added: Optional NATS integration via `ENABLE_NATS` CMake option
- Added: FetchContent declaration for nats.c library (v3.8.0)
- Added: Conditional linking to `ib_box_spread` target
- Status: Ready to build with `-DENABLE_NATS=ON`
- Next: Build with NATS enabled: `cmake -DENABLE_NATS=ON ...`

### ⏳ 4. Swift - SwiftNATS (Pending)
**File**: `ios/BoxSpreadIPad/` (Xcode project)
- Status: Need to add via Xcode Package Manager or Package.swift
- Note: iOS apps typically use Xcode project, not Package.swift
- Next: Add SwiftNATS package via Xcode → File → Add Package Dependencies

---

## Installation Commands

### Python
```bash
# Update requirements.txt
pip-compile requirements.in

# Install
pip install -r requirements.txt
```

### TypeScript
```bash
cd web
npm install
```

### C++
```bash
# Configure with NATS enabled
cmake -S . -B build -DENABLE_NATS=ON

# Build
cmake --build build
```

### Swift (iOS)
1. Open `ios/BoxSpreadIPad.xcodeproj` in Xcode
2. File → Add Package Dependencies
3. Add: `https://github.com/nats-io/swift-nats.git`
4. Select version: `0.1.0` or latest

---

## Next Steps (Phase B)

1. **C++**: Implement NATS connection and publishing in `tws_client.cpp`
2. **Python**: Create NATS client wrapper and integrate into `strategy_runner.py`
3. **TypeScript**: Create NATS service and React hooks
4. **Swift**: Create NATS manager class for iOS app

---

## Verification

After installation, verify dependencies:

```bash
# Python
python -c "import nats; print(nats.__version__)"

# TypeScript
cd web && npm list nats.ws

# C++
# Check CMake output for "NATS integration enabled"
```
