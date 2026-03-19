# Windows 64-bit Setup Guide for TWS API in C++

This guide walks you through setting up the IBKR Box Spread Generator on Windows 64-bit with the Interactive Brokers TWS API.

## Prerequisites

### System Requirements

- **Windows 10/11** (64-bit)
- **Visual Studio 2019 or later** (with C++ Desktop Development workload)
  - Or **MinGW-w64** (alternative compiler)

- **CMake 3.21 or higher**
- **Git for Windows**
- **Interactive Brokers TWS or IB Gateway** installed

### Required Components

1. **Visual Studio Build Tools** or **Visual Studio Community**
   - Install "Desktop development with C++" workload
   - Ensure "Windows 10/11 SDK" is selected
   - Include "CMake tools for Windows"

2. **CMake**
   - Download from: <https://cmake.org/download/>
   - Or install via: `winget install Kitware.CMake`
   - Add to PATH during installation

3. **Git**
   - Download from: <https://git-scm.com/download/win>
   - Or install via: `winget install Git.Git`

## Step 1: Download TWS API

### 1.1 Get TWS API for Windows

1. Visit: <https://interactivebrokers.github.io/>
2. Navigate to **Downloads** section
3. Download **TWS API for Windows** (latest version, currently 10.40.01+)
4. File will be named: `twsapi_win_1040.01.zip` (or similar)

### 1.2 Extract TWS API

```powershell

# Create target directory

New-Item -ItemType Directory -Force -Path "native\third_party\tws-api"

# Extract to target (adjust path to your download location)

Expand-Archive -Path "$env:USERPROFILE\Downloads\twsapi_win_*.zip" -DestinationPath "native\third_party\tws-api" -Force
```

**Expected structure after extraction:**

```
native/third_party/tws-api/
└── IBJts/
    └── source/
        └── cppclient/
            ├── client/          <- Headers (.h files)
            │   ├── EClient.h
            │   ├── EWrapper.h
            │   ├── Contract.h
            │   └── ...
            └── src/             <- Source files (.cpp files)
```

## Step 2: Download Intel Decimal Math Library

The TWS API requires the Intel Decimal Math Library for precision decimal arithmetic.

### 2.1 Download from Netlib

```powershell

# Create cache directory

New-Item -ItemType Directory -Force -Path "native\third_party\cache"

# Download Intel Decimal Library (adjust URL for latest version)

$url = "https://netlib.org/misc/intel/IntelRDFPMathLib20U4.tar.gz"
$output = "native\third_party\cache\IntelRDFPMathLib20U4.tar.gz"
Invoke-WebRequest -Uri $url -OutFile $output

# Extract (requires 7-Zip or tar for Windows 10+)
# If you have 7-Zip:

& "C:\Program Files\7-Zip\7z.exe" x $output -o"native\third_party" -y

# Or use Windows 10+ built-in tar:

tar -xzf $output -C "native\third_party"
```

### 2.2 Verify Intel Library Location

```
native/third_party/IntelRDFPMathLib20U4/
└── LIBRARY/
    ├── libbid.a          <- Static library (will be built)
    └── src/              <- Source files
```

## Step 3: Install Protocol Buffers

TWS API 10.40+ requires Protocol Buffers.

### Option A: Using vcpkg (Recommended)

```powershell

# Install vcpkg if not already installed

git clone https://github.com/Microsoft/vcpkg.git
cd vcpkg
.\bootstrap-vcpkg.bat

# Install protobuf

.\vcpkg install protobuf:x64-windows
```

### Option B: Pre-built Binaries

1. Download from: <https://github.com/protocolbuffers/protobuf/releases>
2. Extract to `C:\protobuf` or similar
3. Add to PATH:
   - `C:\protobuf\bin` (for DLLs)
   - `C:\protobuf\lib` (for libraries)
   - `C:\protobuf\include` (for headers)

## Step 4: Configure Build Environment

### 4.1 Open Developer Command Prompt

**Option A: Visual Studio Developer Command Prompt**

- Open "x64 Native Tools Command Prompt for VS 2019/2022"
- Navigate to project directory

**Option B: PowerShell with VS Environment**

```powershell

# Load Visual Studio environment

& "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"
```

### 4.2 Set Environment Variables

```powershell

# TWS API paths

$env:TWS_API_INCLUDE_DIR = "$PWD\native\third_party\tws-api\IBJts\source\cppclient\client"
$env:TWS_API_LIB_DIR = "$PWD\native\third_party\tws-api\IBJts\source\cppclient\client\build\lib"

# Intel Decimal Library

$env:INTEL_DECIMAL_LIB = "$PWD\native\third_party\IntelRDFPMathLib20U4\LIBRARY\libbid.a"

# Protocol Buffers (if using vcpkg)

$env:Protobuf_ROOT = "C:\vcpkg\installed\x64-windows"
```

## Step 5: Build Intel Decimal Library

### 5.1 Build with CMake

```powershell
cd native\third_party\IntelRDFPMathLib20U4\LIBRARY

# Create build directory

New-Item -ItemType Directory -Force -Path "build"
cd build

# Configure

cmake .. -DCMAKE_BUILD_TYPE=Release -G "Visual Studio 17 2022" -A x64

# Build

cmake --build . --config Release
```

**Expected output:** `libbid.a` in `build\Release\` or `build\lib\`

## Step 6: Build TWS API Library

### 6.1 Build TWS API

```powershell
cd native\third_party\tws-api\IBJts\source\cppclient\client

# Create build directory

New-Item -ItemType Directory -Force -Path "build"
cd build

# Configure (adjust paths as needed)

cmake .. `
    -DCMAKE_BUILD_TYPE=Release `
    -G "Visual Studio 17 2022" `
    -A x64 `
    -DPROTOBUF_ROOT="C:\vcpkg\installed\x64-windows" `
    -DINTEL_BID_LIB="$PWD\..\..\..\..\..\..\IntelRDFPMathLib20U4\LIBRARY\libbid.a"

# Build

cmake --build . --config Release
```

**Expected output:** `TwsApiCpp.dll` and `TwsApiCpp.lib` in `build\Release\` or `build\lib\`

## Step 7: Build Main Project

### 7.1 Configure CMake

```powershell
cd <project_root>

# Create build directory

New-Item -ItemType Directory -Force -Path "build-windows"
cd build-windows

# Configure

cmake .. `
    -G "Visual Studio 17 2022" `
    -A x64 `
    -DCMAKE_BUILD_TYPE=Release `
    -DTWS_API_INCLUDE_DIR="$env:TWS_API_INCLUDE_DIR" `
    -DTWS_API_LIB="$env:TWS_API_LIB_DIR\TwsApiCpp.lib" `
    -DPROTOBUF_ROOT="C:\vcpkg\installed\x64-windows"
```

### 7.2 Build

```powershell

# Build all targets

cmake --build . --config Release

# Or build specific target

cmake --build . --config Release --target ib_box_spread
```

**Expected output:** `ib_box_spread.exe` in `build-windows\Release\` or `build-windows\bin\Release\`

## Step 8: Configure TWS/IB Gateway

### 8.1 Enable API in TWS

1. Open **TWS** or **IB Gateway**
2. Go to **Edit** → **Global Configuration** → **API** → **Settings**
3. Enable:
   - ✅ **Enable ActiveX and Socket Clients**
   - ✅ **Read-Only API** (for testing)
4. Set **Socket Port**:
   - **7497** for Paper Trading
   - **7496** for Live Trading
5. Add **Trusted IPs**: `127.0.0.1` (localhost)
6. Click **OK** and restart TWS/Gateway

### 8.2 Create Configuration File

Create `config\config.json`:

```json
{
  "tws": {
    "host": "127.0.0.1",
    "port": 7497,
    "client_id": 1,
    "connection_timeout_ms": 60000,
    "auto_reconnect": true,
    "max_reconnect_attempts": 10
  },
  "strategy": {
    "symbols": ["SPX"],
    "min_roi_percent": 0.5,
    "min_arbitrage_profit": 0.1
  },
  "risk": {
    "max_total_exposure": 50000.0,
    "max_positions": 10
  },
  "logging": {
    "log_level": "info",
    "log_to_console": true
  }
}
```

## Step 9: Run the Application

### 9.1 Start TWS/IB Gateway

1. Launch **TWS** or **IB Gateway**
2. Log in with your credentials
3. Wait for connection to establish

### 9.2 Run Application

```powershell

# From build directory

.\Release\ib_box_spread.exe

# Or with config file

.\Release\ib_box_spread.exe -c config\config.json

# Dry run mode (safe testing)

.\Release\ib_box_spread.exe --dry-run
```

## Windows-Specific Considerations

### DLL Dependencies

The application requires several DLLs to be in the PATH or same directory:

1. **TwsApiCpp.dll** - TWS API library
2. **protobuf.dll** - Protocol Buffers runtime
3. **MSVC Runtime** - Visual C++ Redistributable

**Solution:** Copy DLLs to executable directory or add to PATH:

```powershell

# Copy DLLs to executable directory

Copy-Item "native\third_party\tws-api\IBJts\source\cppclient\client\build\Release\TwsApiCpp.dll" -Destination "build-windows\Release\"
Copy-Item "C:\vcpkg\installed\x64-windows\bin\protobuf.dll" -Destination "build-windows\Release\"
```

### Windows Socket Library

The TWS API on Windows uses **Winsock 2** (`ws2_32.lib`). This is automatically linked by CMake when `WIN32` is detected.

### Path Separators

- Use backslashes `\` in Windows paths
- Or use forward slashes `/` (CMake handles both)
- Use `$env:VARIABLE` syntax in PowerShell for environment variables

### Build Generators

**Visual Studio (Recommended):**

```powershell
cmake .. -G "Visual Studio 17 2022" -A x64
```

**MinGW Makefiles (Alternative):**

```powershell
cmake .. -G "MinGW Makefiles" -DCMAKE_BUILD_TYPE=Release
```

**Ninja (Fast builds):**

```powershell
cmake .. -G "Ninja" -DCMAKE_BUILD_TYPE=Release
cmake --build . -j8
```

## Troubleshooting

### Error: "Cannot find TwsApiCpp.dll"

**Solution:** Ensure DLL is in executable directory or PATH:

```powershell
$env:PATH += ";native\third_party\tws-api\IBJts\source\cppclient\client\build\Release"
```

### Error: "Protocol Buffers not found"

**Solution:** Install Protocol Buffers via vcpkg or set `Protobuf_ROOT`:

```powershell
$env:Protobuf_ROOT = "C:\vcpkg\installed\x64-windows"
```

### Error: "Intel Decimal Library not found"

**Solution:** Build Intel library first (Step 5) and verify path:

```powershell
Test-Path "native\third_party\IntelRDFPMathLib20U4\LIBRARY\libbid.a"
```

### Connection Timeout

**Solution:**

1. Verify TWS/Gateway is running
2. Check API is enabled in TWS settings
3. Verify port matches config (7497 for paper, 7496 for live)
4. Check Windows Firewall isn't blocking connection

### Build Errors with Visual Studio

**Solution:** Ensure you're using the correct architecture:

- Use **x64 Native Tools Command Prompt** (not x86)
- Specify `-A x64` in CMake configuration

## Next Steps

1. **Test Connection**: Run with `--dry-run` flag first
2. **Paper Trading**: Always test in paper trading mode (port 7497)
3. **Review Logs**: Check `logs\ib_box_spread.log` for connection status
4. **Read Documentation**: See `docs\TWS_INTEGRATION_STATUS.md` for API details

## References

- [TWS API Documentation](https://interactivebrokers.github.io/tws-api/)
- [TWS API Quick Reference](https://www.interactivebrokers.com/download/C++APIQuickReference.pdf)
- [CMake Windows Guide](https://cmake.org/cmake/help/latest/guide/user-interaction/index.html#windows)
- [Visual Studio C++ Documentation](https://docs.microsoft.com/en-us/cpp/)

## Platform-Specific Notes

Our codebase currently focuses on macOS/Linux, but the TWS API itself supports Windows. Key differences:

- **Library extensions**: `.dll` / `.lib` on Windows vs `.dylib` / `.a` on macOS/Linux
- **Threading**: Windows uses `CreateThread` vs `pthread_create` on POSIX
- **Socket library**: Winsock 2 on Windows vs POSIX sockets on Linux/macOS
- **Build system**: Visual Studio solution files vs Makefiles/Ninja

The TWS API handles these differences internally via platform-specific headers (`platformspecific.h`, `EReaderOSSignal.h`).
