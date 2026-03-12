# Dependencies Summary

## Quick Reference

| Language | Build System | Active |
|----------|--------------|--------|
| **Rust** | Cargo | ✅ Yes |
| **C++** | CMake | ✅ Yes |
| **Go** | — | ❌ Removed |
| **Python** | — | ❌ Removed |

---

## Rust Dependencies (agents/backend/)

| Crate | Version | Latest | Status |
|-------|---------|--------|--------|
| tokio | 1.50.0 | 1.50.0 | ✅ Current |
| axum | 0.7.9 | **0.8.8** | ⚠️ Upgrade available |
| reqwest | 0.13.2 | 0.13.2 | ✅ Current |
| async-nats | 0.46.0 | 0.46.0 | ✅ Current |
| sqlx | 0.8.6 | 0.9.0-alpha.1 | Alpha available |
| ratatui | 0.30.0 | 0.30.0 | ✅ Current |
| alloy | 1.7.3 | 1.7.3 | ✅ Current |
| serde | 1.0.228 | 1.0.239 | Minor upgrade |
| chrono | 0.4.44 | 0.4.47 | Minor upgrade |
| uuid | 1.22.0 | 1.11 | Minor upgrade |
| rust_decimal | 1.36 | 1.37 | Minor upgrade |
| prost | 0.13.5 | 0.13.5 | ✅ Current |
| anyhow | 1.0.102 | 1.0.97 | Minor upgrade |

### Direct Workspace Dependencies

```toml
[workspace.dependencies]
anyhow = "1"
async-trait = "0.1"
axum = { version = "0.7", features = ["json"] }
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["rt-multi-thread", "macros", "sync", "time"] }
tokio-stream = "0.1"
tower = "0.4"
rand = { version = "0.8", features = ["std"] }
futures = "0.3"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["fmt", "env-filter"] }
toml = "0.8"
thiserror = "1"
reqwest = { version = "0.13", default-features = false, features = ["cookies", "json", "rustls", "query"] }
rust_decimal = { version = "1.36", features = ["serde-with-str"] }
uuid = { version = "1.10", features = ["v4", "serde"] }
openssl = "0.10"
```

---

## C++ Dependencies (native/)

| Library | Version | Latest | Status |
|---------|---------|--------|--------|
| nlohmann/json | v3.11.3 | v3.9.1 | ⚠️ Behind (older is safer) |
| spdlog | v1.13.0 | v1.9.2 | Minor upgrade available |
| CLI11 | v2.4.1 | **v2.6.2** | Upgrade available |
| Eigen3 | master | master | ✅ Rolling |
| QuantLib | v1.41 | v1.41 | ✅ Current |
| NLopt | v2.9.1 | v2.9.1 | ✅ Current |
| Catch2 | (FetchContent) | v3.8.x | Recent |
| googlebenchmark | v1.6.0 | v1.10.x | Upgrade available |
| Boost | (system) | 1.87 | Via Homebrew |

### CMake FetchContent Versions

```cmake
# nlohmann/json
set(NLOHMANN_JSON_URL_VERSION "v3.11.3")

# spdlog
GIT_TAG v1.13.0

# CLI11
GIT_TAG v2.4.1

# Eigen3
GIT_TAG master

# QuantLib
GIT_TAG v1.41

# NLopt
GIT_TAG v2.9.1
```

---

## Vendored (native/third_party/)

| Library | Location | Notes |
|---------|----------|-------|
| TWS API | `tws-api/` | IBKR official |
| Intel Decimal | `IntelRDFPMathLib20U2/` | Bid128 math |

---

## System Dependencies

| Package | Source | Purpose |
|---------|--------|---------|
| Protobuf | brew | Proto code generation |
| Python3 | brew | Optional helper scripts and Python binding tests |
| Doxygen | optional | Documentation |
| CURL | system | HTTP client |

---

## Archived (Removed)

| Component | Removed | Notes |
|-----------|---------|-------|
| Go agents | ✅ | config-validator, supervisor |
| Python layer | ✅ | All deleted |
| npm/web | ✅ | Archived React app |
| Python services | ✅ | FastAPI services deleted |
