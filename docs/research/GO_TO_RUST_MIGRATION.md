# Research: Migrate Go Supervisor and Config-Validator to Rust

## Overview

Evaluate migrating small Go utilities to Rust to reduce language sprawl.

## Current State

### supervisor (Go)
- 125 lines
- Manages child processes from JSON config
- Restarts on crash
- Used by: `scripts/run_supervisor.sh`

### config-validator (Go)
- 160 lines  
- Validates config JSON
- Validates API contract markdown
- Used by: `scripts/validate_api_contract.sh`

### Rust Backend
- Already owns config loading (`project_paths.rs`, `rest.rs`)
- Has CLI infrastructure via `cargo build -p tui_service`

## Migration Options

### Option 1: Keep Go (Status Quo)
- Pro: Works, small, not urgent
- Con: Another language to maintain

### Option 2: Migrate to Rust CLI
- Pro: Single language (Rust), reuse config loading
- Con: Rewriting working code

### Option 3: Delete Both
- Pro: Reduce complexity
- Con: Lose functionality (supervisor, config validation)

## Recommendation

**Keep Go for now** - both are small, working, and not a priority. Revisit if:
- Go runtime becomes a burden
- Need more integration with Rust config loading

## Tasks

- [ ] Re-evaluate after Rust backend stabilizes
- [ ] Consider deleting supervisor if `scripts/service_manager.sh` is sufficient
