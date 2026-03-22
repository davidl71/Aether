# Lean CLI Learnings (Interactive Brokers)

Source: QuantConnect Lean CLI – Interactive Brokers documentation[^1]

## Deployment Workflow Principles

- **Structured deployment wizard** – Lean guides operators through a fixed sequence (push code, select brokerage, configure notifications,
  set restart policy). We mirrored this with a Python pre-flight checklist (`python/integration/preflight.py`) that validates configuration,
  network reachability, notification setup, and summarizes trading mode before the strategy starts.
- **Weekly re-authentication window** – Lean forces a weekly IB Key re-auth to keep sessions alive.
  We added a configurable re-auth scheduler (`connection_management.weekly_reauth`) that pauses the strategy, disconnects IB,
  and reconnects after the operator approves 2FA.
- **Automatic restart controls** – Lean exposes a knob to auto-restart on failure.
  Our `connection_management` module now exposes `auto_reconnect`, allowing unattended reconnect attempts before escalation to notifications.

## Monitoring & Notifications

- **Multi-channel alerts** – Lean lets users subscribe to email/webhook/SMS/Telegram for order and insight events.
  The new `NotificationCenter` supports the same channels, with per-event subscriptions so operators can route critical (`reauth_failure`,
  `order_rejected`) and informational (`strategy_start`) events appropriately.
- **Meaningful error guidance** – Lean explains common IB errors (missing data permissions, code-card authentication, etc.).
  We enriched the C++ `tws_client` error handler with a guidance catalog so logs surface actionable advice alongside raw IB error codes.

## Data & Session Hygiene

- **Explicit data provider selection** – Lean asks whether to use IB or QuantConnect feeds.
  We added a `data_providers` block that defines a primary provider (IB) and fallbacks (ORATS).
  The data router logs which provider supplies each quote, making post-mortems easier.
- **Pre-flight host checks** – Lean verifies credentials and node availability before deployment.
  Our checklist now probes the IB host/port and warns when notifications or re-auth workflows are disabled.

## Implementation Summary

| Lean Pattern | Implementation | Outcome |
| --- | --- | --- |
| Weekly IB Key reminder | `ReauthScheduler` + `ConnectionSupervisor` | Scheduled pause/reconnect cycle with notifications |
| Deployment wizard prompts | `PreflightChecklist` | Prevents runtime start when config/network prerequisites fail |
| Notification routing | `NotificationCenter` | Email/Webhook/SMS/Telegram hooks with event filters |
| Error catalog | `tws_client.cpp` guidance map | Logs include remediation hints for common IB API failures |
| Data feed choice | `DataProviderRouter` | IB first, ORATS as fallback when primary quotes fail |

[^1]: QuantConnect LEAN CLI – Interactive Brokers Brokerage Integration,
<https://www.quantconnect.com/docs/v2/lean-cli/live-trading/brokerages/interactive-brokers>
