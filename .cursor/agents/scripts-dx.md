# Agent B: Scripts & Developer Experience

## Role

Consolidate duplicated shell scripts into parameterized versions and add corresponding Justfile recipes.

## Tasks

1. **Consolidate 12 service scripts into parameterized set** (`T-1772135701719552000`)
   - Current state: 18 individual scripts in `scripts/`:
     - `start_alpaca_service.sh`, `stop_alpaca_service.sh`
     - `start_discount_bank_service.sh`, `stop_discount_bank_service.sh`
     - `start_ib_service.sh`, `stop_ib_service.sh`
     - `start_nats.sh`, `stop_nats.sh`
     - `start_risk_free_rate_service.sh`, `stop_risk_free_rate_service.sh`
     - `start_rust_backend.sh`, `stop_rust_backend.sh`
     - `start_tastytrade_service.sh`, `stop_tastytrade_service.sh`
     - `start_tradestation_service.sh`, `stop_tradestation_service.sh`
     - `start_web_dev.sh`, `stop_web_dev.sh`
     - `start_all_services.sh`, `stop_all_services.sh`
   - Replace with: `scripts/start_service.sh <name>` and `scripts/stop_service.sh <name>`
   - Keep `start_all_services.sh` and `stop_all_services.sh` as wrappers
   - Add Justfile recipes: `just start <service>`, `just stop <service>`
   - Update existing `services-start` and `services-stop` recipes to use the new scripts

2. **Consolidate build variant scripts** (`T-1772135701754405000`)
   - Current state: 4 build variant scripts with duplicated CMake invocation patterns:
     - `scripts/build_distributed.sh`
     - `scripts/build_fast.sh`
     - `scripts/build_ramdisk.sh`
     - `scripts/build_with_logging.sh`
   - Keep `scripts/build_universal.sh` (already referenced from Justfile)
   - Consolidate the 4 variants into Justfile recipes:
     - `just build-fast` (fast incremental build)
     - `just build-ramdisk` (ramdisk-backed build)
     - `just build-distributed` (distributed compilation)
     - `just build-logging` (build with verbose logging)
   - Add these under the existing `# --- Build ---` section in Justfile

## Files You Own (exclusive)

- `scripts/start_*.sh` and `scripts/stop_*.sh`
- `scripts/build_distributed.sh`, `scripts/build_fast.sh`, `scripts/build_ramdisk.sh`, `scripts/build_with_logging.sh`
- `Justfile` -- ONLY the service recipes (`start`, `stop`) and build variant recipes (`build-fast`, `build-ramdisk`, `build-distributed`, `build-logging`)

## Files You Must NOT Touch

- `native/` (owned by Agent A and Agent E)
- `ansible/` (owned by Agent C)
- `proto/` (owned by Agent D)
- `python/` (owned by Agent C)
- `Justfile` proto recipes (owned by Agent D)
- `Justfile` `build-intel-decimal` recipe (owned by Agent A)

## Completion Criteria

- [ ] `just start nats` and `just stop nats` work
- [ ] `just start ib` and `just stop ib` work
- [ ] `just build-fast` works (or prints usage if deps missing)
- [ ] Old individual scripts removed or replaced with thin wrappers
- [ ] Both exarp tasks marked Done
