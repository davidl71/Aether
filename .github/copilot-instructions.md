# AI Agent Instructions for IBKR Box Spread Generator

This is a C++ project that generates box spread options strategies for Interactive Brokers. These instructions will help AI agents understand the key aspects of the codebase.

## Project Structure

- `ib_box_spread.cpp` - Main source file containing the box spread generation logic
- `CMakeLists.txt` - CMake build configuration
- `build_universal.sh` - Build script for creating universal binaries

## Build System

The project uses CMake as its build system. Key build workflows:

1. Default build:
```bash
mkdir build
cd build
cmake ..
make
```

2. Universal binary build (macOS):
```bash
./build_universal.sh
```

## Development Patterns

### Code Organization
- Single-file architecture keeping all related functionality together
- CMake-based build system with versioning support

### Integration Points
- Interactive Brokers API integration - Requires proper setup and authentication
- Box spread options calculation - Follows standard options pricing models

## Key Notes for AI Agents

1. When modifying options calculations, ensure:
   - Proper handling of option contract specifications
   - Price calculation accuracy for all legs of the box spread
   - Risk management checks are maintained

2. Build considerations:
   - Always test both regular and universal binary builds
   - Verify CMake configuration when adding new dependencies

3. Project conventions:
   - Maintain single-file architecture unless complexity requires splitting
   - Follow existing error handling patterns
   - Keep builds compatible with target platforms

## References
- Main logic: See `ib_box_spread.cpp`
- Build configuration: `CMakeLists.txt`
- Build scripts: `build_universal.sh`

Please request clarification if you need more specific details about any of these aspects.