# Box Spread C++ Library Integration

**Date**: 2025-11-20
**Status**: Submodule added, integration ready

## Overview

The broker-agnostic C++ engine has been extracted to a separate repository and is now available as a git submodule.

**Repository**: https://github.com/davidl71/box-spread-cpp
**Location**: `libs/box-spread-cpp/`

## Current Status

✅ **Submodule Added**: Repository cloned to `libs/box-spread-cpp/`
✅ **CMake Integration**: Option added to use the extracted library
⚠️ **Migration Pending**: Main repo still uses local source files

## Using the Extracted Library

### Option 1: Enable Library (Future Migration)

```bash
# Configure CMake to use the extracted library
cmake -B build -DUSE_BOX_SPREAD_CPP_LIB=ON

# Build
cmake --build build
```

**Note**: This requires migrating the main executable to use the `box_spread::` namespace and `IBroker` interface instead of direct `TWSClient` usage.

### Option 2: Continue Using Local Implementation (Current)

The main repo continues to use local source files in `native/src/` and `native/include/`. This is the default behavior.

```bash
# Build with local implementation (default)
cmake -B build
cmake --build build
```

## Migration Path

To fully migrate to the extracted library:

1. **Update includes**: Change `#include "box_spread_strategy.h"` to `#include <box_spread/box_spread_strategy.h>`
2. **Update namespaces**: Change `strategy::` to `box_spread::strategy::`
3. **Replace TWSClient**: Change `tws::TWSClient*` to `box_spread::brokers::IBroker*`
4. **Implement IBroker**: Create a TWS adapter that implements the `IBroker` interface
5. **Update CMakeLists.txt**: Remove local source files, link to `box_spread_cpp` library

## Submodule Management

### Initialize Submodule

```bash
# Initialize and clone the submodule
git submodule update --init --recursive
```

### Update Submodule

```bash
# Update to latest commit
cd libs/box-spread-cpp
git pull origin main
cd ../..

# Commit the submodule update
git add libs/box-spread-cpp
git commit -m "Update box-spread-cpp submodule"
```

### Remove Submodule (if needed)

```bash
# Remove submodule
git submodule deinit libs/box-spread-cpp
git rm libs/box-spread-cpp
rm -rf .git/modules/libs/box-spread-cpp
```

## Benefits of Using the Extracted Library

1. **Broker-Agnostic**: Works with any broker via `IBroker` interface
2. **Reusable**: Can be used in other projects
3. **Maintainable**: Clear separation of concerns
4. **Testable**: Library can be tested independently
5. **Versioned**: Library has its own versioning and release cycle

## Next Steps

1. **Create TWS Adapter**: Implement `IBroker` interface for TWS
2. **Migrate Main Executable**: Update to use extracted library
3. **Update Tests**: Migrate tests to use library
4. **Remove Local Files**: Clean up after migration complete

## Related Documentation

- [Project Split Strategy](PROJECT_SPLIT_STRATEGY.md)
- [Extracted Repositories](EXTRACTED_REPOSITORIES.md)
- [Box Spread C++ Library README](platform/README.md)
