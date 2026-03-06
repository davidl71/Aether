# When to use a proper web server (or proxy / process manager)

**TL;DR**  
- **Reverse proxy (nginx / Traefik / Caddy):** as soon as you have **more than one** HTTP service and want a single hostname, TLS, or production hardening—so **now** is reasonable.  
- **Process manager (systemd / supervisord / PM2):** as soon as you care about **restarts, logging, or running in the background**—also **now** if you run multiple services.  
- **API gateway / load balancer:** when you have **many backends (e.g. 10+)** or need **routing, rate limits, or auth** in one place.  
- **Consolidating apps (monolith or fewer processes):** when **operational overhead** (ports, configs, deploys) hurts more than keeping services separate.

---

## 1. How many services do you have?

Rough cutoffs:

| Situation | Typical approach |
|-----------|-------------------|
| **1–2 services** | Run uvicorn/npm directly; optional reverse proxy for TLS. |
| **3–6 services** | **Reverse proxy** (nginx/Traefik/Caddy) in front: one hostname, one TLS cert, path or subdomain routing. **Process manager** (systemd units or supervisor) to start/restart/log. |
| **7–15 services** | Same as above; consider **one config file** (e.g. one nginx include or one Traefik labels file) so adding a service is one block. |
| **15+ services** | **API gateway** (Kong, Traefik with more features, or cloud LB) can pay off: central auth, rate limits, routing. Optionally **consolidate** some backends (e.g. one FastAPI app with routers per “service”) to reduce ports and processes. |

You’re in the **“3–6+”** range (IB, Alpaca, TradeStation, Tastytrade, Discount Bank, Risk-free rate, Rust backend, Web, etc.), so **reverse proxy + process manager** is already justified; gateway/consolidation is optional.

---

## 2. Reverse proxy (nginx / Traefik / Caddy)

**Use when:**

- You have **multiple HTTP services** (different ports).
- You want **one hostname** (e.g. `app.example.com`) and **one TLS certificate**.
- You want to **hide internal ports** and add **security headers** or **rate limiting** in one place.

**Cutoff:** **2+ services** that need to be reached over HTTP/HTTPS. No hard number—even with 2, a proxy simplifies TLS and URLs.

**This repo:** Production deployment already suggests nginx/traefik (e.g. `web/IB_INTEGRATION.md`). The Debian packaging script generates nginx config for the web app.

---

## 3. Process manager (systemd / supervisord / PM2)

**Use when:**

- You want services to **restart on crash** and **start on boot**.
- You want **centralized logs** (stdout/stderr) and **resource limits**.
- You’re tired of managing many terminal sessions or `nohup`/screen.

**Cutoff:** **2+ long‑lived processes** you care about keeping up. With many uvicorn + npm + NATS + Rust processes, a process manager is appropriate.

**This repo:** `scripts/service_manager.sh` and `scripts/run_supervisor.sh` (Go supervisor) are the current levers. Moving to **systemd units** (one unit per service) or **supervisord** is the “proper” next step for a single machine.

---

## 4. API gateway / load balancer

**Use when:**

- You have **many backends** (e.g. **10+**) and want **one place** for routing, auth, or rate limits.
- You need **canary** or **A/B routing**, or **central API keys**.
- You’re in the cloud and use a **managed LB/gateway** (ALB, API Gateway, etc.).

**Cutoff:** Usually **~10+ services** or when **cross‑cutting concerns** (auth, limits, routing) become painful to duplicate in each app. Not required before that if a simple reverse proxy is enough.

---

## 5. Consolidating services (fewer processes)

**Use when:**

- **Operational overhead** is high: too many ports, configs, and deploy steps.
- Services are **small** and **low traffic**; many could be **routers** in one FastAPI app (e.g. `/api/ib/*`, `/api/alpaca/*`) or one Node app.
- You want **one process** to scale (e.g. one uvicorn with many workers) instead of many small processes.

**Cutoff:** Subjective. Often considered when you have **~8+** separate HTTP services that could share one runtime and one port behind a proxy. Trade-off: simpler ops vs. coupling and shared deploys.

---

## 6. Practical recommendation for this project

- **Now (or next production deploy):**  
  - Put a **reverse proxy** (nginx or Traefik) in front of all HTTP services; single hostname and TLS.  
  - Use a **process manager** (systemd or supervisord) to run and restart each service.

- **Later, if the list grows or ops hurt:**  
  - Add an **API gateway** or use Traefik’s more advanced features.  
  - Optionally **merge** some Python services into one FastAPI app with multiple routers and route by path.

- **Not required yet:**  
  - A heavy API gateway when a simple proxy + process manager solves the problem.  
  - Consolidating all services into one app unless you explicitly want fewer processes and are okay with the coupling.

Summary cutoff: **multiple web services + production** → use a **proper web server (reverse proxy)** and a **process manager**; add a **gateway** or **consolidation** when you have many backends or central auth/limits needs.

---

## 7. Implementation in this repo

The following files implement reverse proxy and process manager options.

### Reverse proxy (nginx)

| File | Purpose |
|------|--------|
| `config/nginx/backend-services.conf` | Single nginx server block (port 8080) with path-based routing to all backends. `/api/` → IB (default), `/api/ib/`, `/api/alpaca/`, etc. → respective service. Root `/` proxies to web dev server (5173). |

**Install (Linux):**
```bash
sudo cp config/nginx/backend-services.conf /etc/nginx/sites-available/
sudo ln -s /etc/nginx/sites-available/backend-services.conf /etc/nginx/sites-enabled/
sudo nginx -t && sudo systemctl reload nginx
```
**Frontend:** Set `VITE_API_URL=http://localhost:8080/api` to use the proxy (default backend = IB). Or use path-prefixed URLs: `/api/ib/snapshot`, `/api/alpaca/snapshot`.

### Process manager – supervisord (Python)

| File | Purpose |
|------|--------|
| `config/supervisord.conf` | All backends (ib, alpaca, tradestation, tastytrade, discount_bank, risk_free_rate, web, nats, rust_backend) with autorestart and logs. |
| `scripts/run_supervisord.sh` | Wrapper that sets `PROJECT_ROOT` and runs supervisord. |

**Run:**
```bash
export PROJECT_ROOT="$(pwd)"   # or your repo path
./scripts/run_supervisord.sh
# Or: supervisord -c config/supervisord.conf
supervisorctl -c config/supervisord.conf status
```
**Requires:** `pip install supervisord` or `apt install supervisor`.

### Process manager – systemd user units

| File | Purpose |
|------|--------|
| `config/systemd/user/ib-box-spread.env` | Env file with `PROJECT_ROOT` (replaced on install). |
| `config/systemd/user/ib-box-spread-*.service` | One unit per service (ib, alpaca, tradestation, tastytrade, discount_bank, risk_free_rate, web, nats). |
| `scripts/install_systemd_user_units.sh` | Copies units to `~/.config/systemd/user/` and sets `PROJECT_ROOT`. |

**Install and run:**
```bash
./scripts/install_systemd_user_units.sh
systemctl --user daemon-reload
systemctl --user start ib-box-spread-ib ib-box-spread-alpaca
systemctl --user enable ib-box-spread-ib   # optional: start at login
```

### Existing alternatives

- **`scripts/service_manager.sh`** – Start/stop services manually by port (no auto-restart).
- **`scripts/run_supervisor.sh`** – Go-based supervisor using `config/services.supervisor.json` (requires Go).
