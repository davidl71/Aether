# Wishlist

## High Priority

- [ ] Evaluate Lean CLI integration for optional QuantConnect/LEAN workflows
  - Track lessons learned in `docs/LEAN_LEARNINGS.md`
  - Reference Lean CLI documentation: https://www.lean.io/docs/v2/lean-cli
  - Gate any implementation on clear benefit for our IBKR box-spread flow

## Low Priority

- [ ] Universal binary support (x86_64 + arm64)
  - Currently builds x86_64 only due to TWS API library architecture limitations
  - Would require rebuilding TWS API library and all dependencies as universal binaries
  - Priority: Low - x86_64 works fine on Intel Macs and via Rosetta on Apple Silicon
  - Note: Deprecated universal presets in `CMakePresets.json` still exist but build x86_64 only
