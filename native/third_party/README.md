# Third-Party Assets

**Only this README and `eula.txt` are tracked in git.** All other contents are populated by
`./scripts/fetch_third_party.sh` (Ansible playbook) and must not be committed.

Populate the directories via:

```bash
./scripts/fetch_third_party.sh
```

to download or unwrap the following:

- **Protobuf v3.20.3** → unpacked into `native/third_party/protobuf-3.20.3/` from the official GitHub release (override with `PROTOBUF_URL`).
- **Intel Decimal Math Library** → extracted to `native/third_party/IntelRDFPMathLib20U4/` when `INTEL_DECIMAL_URL` is set, the archive exists at `native/third_party/cache/IntelRDFPMathLib20U4.tar.gz`, or found in Downloads folder. Falls back to downloading from [Netlib](https://www.netlib.org/misc/intel/) if not found locally.
- **IBKR TWS API** → unzipped into `native/third_party/tws-api/` when `IB_API_ARCHIVE` points to a local or remote IBKR download. (IBKR requires you to fetch the archive yourself.)

## Manual Intel Decimal Library Installation

The Intel Decimal Floating-Point Math Library is available from [Netlib](https://www.netlib.org/misc/intel/).

**Automatic (recommended):**

```bash
# The fetch script will automatically download from Netlib if not found locally
./scripts/fetch_third_party.sh
```

**Manual options:**

1. **Place in Downloads folder** (any subdirectory) - will be auto-detected:

   ```bash
   # Download from https://www.netlib.org/misc/intel/IntelRDFPMathLib20U4.tar.gz
   # Place in ~/Downloads (or any subdirectory)
   ./scripts/fetch_third_party.sh
   ```

2. **Provide URL via environment variable:**

   ```bash
   export INTEL_DECIMAL_URL="https://www.netlib.org/misc/intel/IntelRDFPMathLib20U4.tar.gz"
   ./scripts/fetch_third_party.sh
   ```

3. **Place manually in cache:**
   ```bash
   # Download and place at:
   native/third_party/cache/IntelRDFPMathLib20U4.tar.gz
   ./scripts/fetch_third_party.sh
   ```

## Manual TWS API Installation

You can use either the **GitHub repo** (recommended) or the **IBKR zip**:

**Option A – GitHub repo (recommended)**
Clone the official [InteractiveBrokers/tws-api](https://github.com/InteractiveBrokers/tws-api) repo next to this project (e.g. `../tws-api`). CMake will auto-detect it and build the C++ client from source:

```bash
git clone https://github.com/InteractiveBrokers/tws-api.git ../tws-api
# From the repo root, configure; TWS_API_SOURCE_DIR is auto-detected when ../tws-api exists
cmake -S . -B build -G Ninja -DCMAKE_BUILD_TYPE=Debug
# Or point explicitly: -DTWS_API_SOURCE_DIR=/path/to/tws-api
```

**Option B – IBKR zip**
1. Visit <https://interactivebrokers.github.io/> and download the latest TWS API zip.
2. Place the archive somewhere local and run:
   ```bash
   export IB_API_ARCHIVE="/path/to/twsapi_macunix.zip"
   ./scripts/fetch_third_party.sh
   ```
   Or place in Downloads folder (any subdirectory) - will be auto-detected.
   Or unzip manually into `native/third_party/tws-api/` so that headers live at `native/third_party/tws-api/IBJts/source/cppclient/client/`.
3. Re-run CMake; it will detect the headers and the prebuilt `libtwsapi.so` (Linux) or `libtwsapi.dylib` (macOS).

**Linux:** Build requires system protobuf (`sudo apt install libprotobuf-dev`). The Intel Decimal library is built with `-fPIC` so it can be linked into the TWS API shared library. See TWS API `IBJts/source/cppclient/Intel_lib_build.txt` for official Intel library build notes.

## Cache Layout

Temporary archives are written to `native/third_party/cache/`. Feel free to seed that
folder with pre-downloaded tarballs before invoking the fetch script to avoid
re-downloading large files repeatedly.
