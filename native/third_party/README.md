# Third-Party Assets

This tree aggregates all external vendor dependencies. Nothing under `native/third_party/`
should be committed unless the upstream license requires us to ship source code.
Populate the directories via `./scripts/fetch_third_party.sh` (which delegates to
`ansible/playbooks/fetch_third_party.yml`) to download or unwrap the following:

- **Protobuf v3.20.3** → unpacked into `native/third_party/protobuf-3.20.3/` from the official GitHub release (override with `PROTOBUF_URL`).
- **Intel Decimal Math Library** → extracted to `native/third_party/IntelRDFPMathLib20U2/` when `INTEL_DECIMAL_URL` is set or the archive already exists at `native/third_party/cache/IntelRDFPMathLib20U2.tar.gz`.
- **IBKR TWS API** → unzipped into `native/third_party/tws-api/` when `IB_API_ARCHIVE` points to a local or remote IBKR download. (IBKR requires you to fetch the archive yourself.)

## Manual TWS API Installation

1. Visit <https://interactivebrokers.github.io/> and download the latest TWS API zip.
2. Place the archive somewhere local and run:
   ```bash
   export IB_API_ARCHIVE="/path/to/twsapi_macunix.zip"
   ./scripts/fetch_third_party.sh
   ```
   Or unzip manually into `native/third_party/tws-api/` so that headers live at `native/third_party/tws-api/IBJts/source/cppclient/client/`.
3. Re-run CMake; it will detect the headers and the prebuilt `libtwsapi`.

## Cache Layout

Temporary archives are written to `native/third_party/cache/`. Feel free to seed that
folder with pre-downloaded tarballs before invoking the fetch script to avoid
re-downloading large files repeatedly.
