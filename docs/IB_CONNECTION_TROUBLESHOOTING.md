# Why can't I connect to IB?

This app talks to Interactive Brokers in **two different ways**. Only **one** can be active at a time.

---

## 1. Client Portal (REST) – used by the Rust backend and TUI

**Used for:** Rust-owned IB snapshot, positions, and account-facing routes used by the TUI.

| Requirement | What to check |
|-------------|----------------|
| **Gateway running** | Run the **IB Client Portal Gateway** (not classic TWS/Gateway). Example: `./ib-gateway/run-gateway.sh` or the Gateway package’s `bin/run.sh`. |
| **Logged in** | Open **https://localhost:5001** in a browser and log in with your IB credentials. The app uses this session. |
| **Port** | Default is **5001**. Override with `IB_PORTAL_URL` or config `ibkr_portal.base_url` (e.g. `https://localhost:5001/v1/portal`). |
| **SSL** | Gateway uses a self-signed cert. Use `verify_ssl: false` in config (or `ibkr_portal.verify_ssl: false`). |
| **Exclusive mode** | If **TWS or IB Gateway (socket)** is logged in on 7496/7497, log out of it first. Client Portal and socket API cannot both be connected at once. |

**Quick test:**

```bash
# Ensure Gateway is running and you're logged in at https://localhost:5001, then:
IB_PORTAL_URL=https://localhost:5001/v1/portal uv run python scripts/test_ib_positions.py
```

If you see "No accounts" or connection errors, the Gateway is not running, not logged in, or not reachable on 5001.

---

## 2. TWS API (socket) – used by C++ client

**Used for:** Native TWS API (orders, streaming market data) via `native/src/tws_client.cpp`.

| Requirement | What to check |
|-------------|----------------|
| **TWS or IB Gateway (socket) running** | Start TWS or the **classic** IB Gateway (socket mode), not the Client Portal Gateway. |
| **Port** | **7497** = paper, **7496** = live. Config: `port` in TWS connection section or `tcp_backend_ports.tws`. |
| **API enabled** | In TWS/Gateway: **Configure → API → Settings** → enable "Enable ActiveX and Socket Clients", add **127.0.0.1** to trusted IPs. |
| **Exclusive mode** | If the **Client Portal Gateway** is logged in on 5001, log out of it first. Only one of (Client Portal, TWS socket) can be connected at a time. |

**Note:** The C++ TWS client in this repo may still be a stub; see README and `docs/API_DOCUMENTATION_INDEX.md` for TWS API status.

---

## Checklist when "can't connect to IB"

1. **Using TUI or Rust IB routes (REST)?**  
   → Start **Client Portal Gateway**, open https://localhost:5001 and log in.  
   → Ensure nothing else is using the socket (7496/7497) with the same account.

2. **Using C++/TWS socket?**  
   → Start **TWS** or **classic IB Gateway** (socket).  
   → Enable API in Configure → API → Settings, allow 127.0.0.1.  
   → Ensure you are not logged into Client Portal on 5001 with the same account.

3. **Wrong port or URL?**  
   → REST: `https://localhost:5001/v1/portal` (or your `IB_PORTAL_URL`).  
   → Socket: 7497 (paper) or 7496 (live).

4. **SSL errors?**  
   → Set `ibkr_portal.verify_ssl: false` (or `verify_ssl: false`) for the self-signed Gateway cert.

5. **Firewall / VPN?**  
   → Ensure localhost (127.0.0.1) is not blocked and no VPN is breaking localhost.

---

## References

- **Client Portal vs TWS:** `docs/TWS_ORATS_PORTAL_QUESTDB.md`
- **Config:** `README.md` (§ IBKR Client Portal API), `config/config.example.json` (§ `ibkr_portal`, `tui`)
- **Test script:** `scripts/test_ib_positions.py` – tests Client Portal connectivity
