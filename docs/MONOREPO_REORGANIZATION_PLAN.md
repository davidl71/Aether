# Monorepo Reorganization Plan

**Date**: 2025-11-20
**Task**: T-207
**Status**: Planning Phase

## Objective

Reorganize the main monorepo to use extracted libraries (`box-spread-cpp` and `box-spread-python`) instead of local source files.

## Current State

- ✅ `box-spread-cpp` extracted and available as submodule
- ✅ `box-spread-python` extracted and available
- ⚠️ Main repo still uses local source files
- ⚠️ Duplicate code exists in both locations

## Reorganization Strategy

### Phase 1: Update CMake Configuration

**Files to Update:**

- `native/CMakeLists.txt` - Already has `USE_BOX_SPREAD_CPP_LIB` option
- Update to use library by default (or make it easy to switch)

**Changes:**

1. Set `USE_BOX_SPREAD_CPP_LIB=ON` as default
2. Remove local source files from build when using library
3. Update include paths to use library headers

### Phase 2: Create TWS Adapter

**New Component:**

- `native/src/brokers/tws_adapter.cpp` - Implements `IBroker` interface for TWS
- `native/include/brokers/tws_adapter.h` - Adapter header

**Purpose:**

- Bridge between TWS API and abstract `IBroker` interface
- Enables using extracted library with TWS

### Phase 3: Update Python Imports

**Files to Update:**

- Python files that import from `python.dsl`, `python.tools`, etc.
- Update to use `box_spread` package (from PyPI or submodule)

**Changes:**

1. Install `box-spread-python` package
2. Update imports: `from python.dsl` → `from box_spread.dsl`
3. Remove local Python modules that are now in package

### Phase 4: Remove Duplicate Code

**After Migration Complete:**

1. Remove local copies of extracted files
2. Update all references
3. Verify builds and tests still pass

## Migration Checklist

### CMake Updates

- [ ] Enable `USE_BOX_SPREAD_CPP_LIB` by default
- [ ] Update include directories
- [ ] Update library linking
- [ ] Remove local source files from build

### TWS Adapter

- [ ] Create `tws_adapter.h`
- [ ] Implement `IBroker` interface
- [ ] Map TWS API calls to interface methods
- [ ] Test adapter with extracted library

### Python Updates

- [ ] Add `box-spread-python` as dependency
- [ ] Update all imports
- [ ] Remove local Python modules
- [ ] Test Python code still works

### Verification

- [ ] All builds succeed
- [ ] All tests pass
- [ ] No duplicate code remains
- [ ] Documentation updated

## Dependencies

- T-204: ✅ Complete (C++ library extracted)
- T-205: ✅ Complete (Python package extracted)
- T-206: ⚠️ Required (TWS adapter implementation)

## Risks

1. **Breaking Changes**: Migration may break existing code
   - Mitigation: Keep old code until new code proven
   - Use feature flags to switch between old/new

2. **Missing Functionality**: Library may not have all features
   - Mitigation: Audit feature parity before migration
   - Add missing features to library if needed

3. **Build Complexity**: Submodule management
   - Mitigation: Document submodule workflow
   - Consider package managers for production

## Next Steps

1. **Create TWS Adapter** (T-206) - Required before migration
2. **Test Library Integration** - Verify library works with adapter
3. **Gradual Migration** - Migrate one component at a time
4. **Remove Old Code** - After migration verified
