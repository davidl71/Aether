# Emscripten Setup Guide

Emscripten is required to compile C++ code to WebAssembly (WASM) for the web app.

## Installation

Emscripten has been installed in the project directory:

```
ib_box_spread_full_universal/
  emsdk/          ← Emscripten SDK
```

## Activation

### For Current Shell Session

```bash
source /Users/davidlowes/ib_box_spread_full_universal/emsdk/emsdk_env.sh
```

### For Persistent Activation

The setup script has automatically added Emscripten to your `~/.zshrc` file. It will be available in new terminal sessions.

To manually add it:

```bash
echo 'source "/Users/davidlowes/ib_box_spread_full_universal/emsdk/emsdk_env.sh"' >> ~/.zshrc
```

## Verification

Check that Emscripten is working:

```bash
emcc --version
```

You should see:

```
emcc (Emscripten gcc/clang-like replacement + linker emulating GNU ld) 4.0.19
```

## Building WASM

Once Emscripten is activated, you can build the WASM module:

```bash
./scripts/build_wasm.sh
```

The build script will automatically detect and use Emscripten if it's in the PATH or in the project's `emsdk/` directory.

## Troubleshooting

### Emscripten Not Found

If you get "Emscripten not found" errors:

1. **Check if emsdk exists:**

   ```bash
   ls -la emsdk/
   ```

2. **Source the environment:**

   ```bash
   source emsdk/emsdk_env.sh
   ```

3. **Verify installation:**

   ```bash
   emcc --version
   ```

### Build Script Can't Find Emscripten

The build script automatically looks for Emscripten in:

- `${PROJECT_ROOT}/emsdk` (project directory)
- `${HOME}/emsdk` (home directory)
- `/usr/local/emsdk` (system-wide)

If Emscripten is elsewhere, source it manually before running the build script:

```bash
source /path/to/emsdk/emsdk_env.sh
./scripts/build_wasm.sh
```

## Updating Emscripten

To update to the latest version:

```bash
cd emsdk
./emsdk install latest
./emsdk activate latest
source ./emsdk_env.sh
```

## Version

Current installed version: **4.0.19**

## See Also

- [WASM Integration Plan](./WASM_INTEGRATION_PLAN.md)
- [WASM Quick Start](./WASM_QUICK_START.md)
- [Emscripten Documentation](https://emscripten.org/docs/getting_started/index.html)
