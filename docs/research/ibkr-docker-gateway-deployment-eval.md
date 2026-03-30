# Evaluation: ibkr-docker for IB Gateway deployment

**Task:** T-1774284379765481000  
**Date:** 2026-03-30  
**Reference:** [extrange/ibkr-docker](https://github.com/extrange/ibkr-docker)

## Summary conclusion

**Verdict:** Treat **ibkr-docker** as a **valid optional path** for headless IB Gateway (Linux servers, homelab, CI-style environments) where a native macOS/Windows install is undesirable. **Keep the current host-native workflow** (`scripts/install_ib_gateway.sh`, local TWS/Gateway) as the **primary developer default**, especially on macOS.

**Not a repo requirement:** No change to Aether code is needed to *evaluate* compatibility; the stack already connects via **host + port** (`TWS_HOST`, `TWS_PORT` — see `tws_yield_curve`, `tws_yield_curve_daemon`, backend broker wiring). Point those at the container-published API port after `docker compose up`.

---

## What ibkr-docker offers (per project scope)

The task targets **[extrange/ibkr-docker](https://github.com/extrange/ibkr-docker)** (~335 stars as of 2026-03). Public materials describe:

- Containerized **IB Gateway / TWS** with **IBC**-style automation for login/session handling  
- **noVNC** (browser) access to the desktop session for one-time setup or troubleshooting  
- **Docker Compose**-first deployment and env-driven configuration  

Comparable ecosystems exist (e.g. [gnzsnz/ib-gateway-docker](https://github.com/gnzsnz/ib-gateway-docker) — often cited for IBC + headless + port forwarding patterns). The evaluation below applies broadly; **implementation choice** is between these images, not Aether internals.

---

## Fit with Aether

| Area | Notes |
|------|--------|
| **API socket** | Rust `ib_adapter` / yield-curve path expects a standard TWS API TCP connection. Containers typically **publish** the gateway API port to the host (e.g. map container `4002`/`7497` to a host port). Set `TWS_HOST=127.0.0.1` (or remote host) and `TWS_PORT` to the **mapped** port. |
| **Default ports in tree** | `tws_yield_curve` probes paper/live pairs including **7497, 4002, 7496, 4001** (`crates/tws_yield_curve/src/lib.rs`). Match container mapping to the account mode you use (paper vs live). |
| **Client Portal** | Separate HTTP surface (`IB_PORTAL_URL`, default `https://localhost:5001/...`). If you rely on Client Portal from Gateway in Docker, ensure that port is also published and TLS/proxy expectations still hold — **extra** compose wiring beyond “API only.” |
| **Secrets** | Credentials belong in env files or secret stores **never committed**; same discipline as host install. |

---

## Pros (task list + validation)

- **No local IB installer** on Linux; repeatable compose files across machines  
- **Version pinning** via image tags / compose env  
- **Process isolation** from host JVM footprint and GUI clutter  
- **Auto-restart** patterns (supervisor/IBC) common in these images  

## Cons / risks

- **Docker operational tax**: volumes, upgrades, log inspection, resource limits  
- **2FA / login edge cases**: IBKR policy changes can break unattended flows; VNC helps but is another moving part  
- **macOS devs**: Docker Desktop + GUI/VNC path is often **more** friction than `install_ib_gateway.sh`  
- **Networking**: remote hosts need firewall rules; `host.docker.internal` vs explicit IP on non-Docker Desktop setups  
- **Trust**: third-party images — pin digests, read Dockerfiles, prefer building from source when security posture demands it  

---

## Recommendation

1. **Short term:** Document-only (this note). Continue recommending **host IB Gateway** for primary macOS development per existing scripts.  
2. **When to adopt ibkr-docker (or gnzsnz):** Linux **VPS/homelab** automation, **multi-environment** parity, or teams that already standardize on Compose for all services.  
3. **If adopted:** Add a short `docs/platform/` runbook (compose snippet, required env vars, `TWS_HOST`/`TWS_PORT` example, paper port **7497** for tests) — **separate task** if product wants first-class support.  

---

## References (verified URLs)

- [extrange/ibkr-docker](https://github.com/extrange/ibkr-docker) — primary reference for this task  
- [gnzsnz/ib-gateway-docker](https://github.com/gnzsnz/ib-gateway-docker) — widely used alternative for comparison  
- Interactive Brokers — official Gateway/TWS downloads (used by `scripts/install_ib_gateway.sh`)  
