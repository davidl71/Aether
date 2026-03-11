"""Convert platform protobuf types (SystemSnapshot, BackendHealth) to TUI/API models.

Used by NATS subscribers that receive protobuf on snapshot.* and system.health.
"""

from __future__ import annotations

from typing import Any, Dict, Optional

from ..models import (
    AccountMetrics,
    Candle,
    SnapshotPayload,
    SymbolSnapshot,
    PositionSnapshot,
    TimelineEvent,
    Severity,
)


def _ts_to_iso(ts: Any) -> str:
    """Convert protobuf Timestamp or None to ISO 8601 string."""
    if ts is None:
        return ""
    if hasattr(ts, "ToDatetime"):
        dt = ts.ToDatetime()
    elif hasattr(ts, "to_datetime"):
        dt = ts.to_datetime()
    else:
        return ""
    return dt.isoformat() if hasattr(dt, "isoformat") else str(dt)


def backend_health_to_dict(health: Any) -> Dict[str, Any]:
    """Convert BackendHealth proto to dict for health dashboard / TUI."""
    backend = getattr(health, "backend", "") or ""
    status = getattr(health, "status", "") or ""
    error = getattr(health, "error", "") or ""
    hint = getattr(health, "hint", "") or ""
    extra = getattr(health, "extra", None) or {}
    updated_at = _ts_to_iso(getattr(health, "updated_at", None))
    out = {
        "backend": backend,
        "status": status,
        "updated_at": updated_at,
    }
    if error:
        out["error"] = error
    if hint:
        out["hint"] = hint
    if isinstance(extra, dict):
        out.update(extra)
    elif hasattr(extra, "items"):
        out.update(dict(extra))
    return out


def system_snapshot_to_payload(snap: Any) -> SnapshotPayload:
    """Convert SystemSnapshot proto to SnapshotPayload (TUI model)."""
    metrics = getattr(snap, "metrics", None)
    if metrics is not None:
        m = AccountMetrics(
            net_liq=getattr(metrics, "net_liq", 0.0) or 0.0,
            buying_power=getattr(metrics, "buying_power", 0.0) or 0.0,
            excess_liquidity=getattr(metrics, "excess_liquidity", 0.0) or 0.0,
            margin_requirement=getattr(metrics, "margin_requirement", 0.0) or 0.0,
            commissions=getattr(metrics, "commissions", 0.0) or 0.0,
            portal_ok=getattr(metrics, "portal_ok", False),
            tws_ok=getattr(metrics, "tws_ok", False),
            questdb_ok=getattr(metrics, "questdb_ok", False),
        )
    else:
        m = AccountMetrics()

    symbols: list = []
    for s in getattr(snap, "symbols", []) or []:
        c = getattr(s, "candle", None)
        if c is not None:
            candle = Candle(
                open=getattr(c, "open", 0.0) or 0.0,
                high=getattr(c, "high", 0.0) or 0.0,
                low=getattr(c, "low", 0.0) or 0.0,
                close=getattr(c, "close", 0.0) or 0.0,
                volume=getattr(c, "volume", 0) or 0,
                entry=getattr(c, "entry", 0.0) or 0.0,
                updated=_ts_to_iso(getattr(c, "updated", None)),
            )
        else:
            candle = Candle()
        symbols.append(
            SymbolSnapshot(
                symbol=getattr(s, "symbol", "") or "",
                last=getattr(s, "last", 0.0) or 0.0,
                bid=getattr(s, "bid", 0.0) or 0.0,
                ask=getattr(s, "ask", 0.0) or 0.0,
                spread=getattr(s, "spread", 0.0) or 0.0,
                roi=getattr(s, "roi", 0.0) or 0.0,
                maker_count=getattr(s, "maker_count", 0) or 0,
                taker_count=getattr(s, "taker_count", 0) or 0,
                volume=float(getattr(s, "volume", 0) or 0),
                candle=candle,
            )
        )

    positions: list = []
    for p in getattr(snap, "positions", []) or []:
        positions.append(
            PositionSnapshot(
                name=getattr(p, "symbol", "") or getattr(p, "id", "") or "",
                quantity=getattr(p, "quantity", 0) or 0,
                roi=0.0,
                price=getattr(p, "mark", 0.0) or 0.0,
            )
        )

    historic: list = []
    for p in getattr(snap, "historic", []) or []:
        historic.append(
            PositionSnapshot(
                name=getattr(p, "symbol", "") or getattr(p, "id", "") or "",
                quantity=getattr(p, "quantity", 0) or 0,
            )
        )

    orders: list = []
    for o in getattr(snap, "orders", []) or []:
        orders.append(
            TimelineEvent(
                timestamp=_ts_to_iso(getattr(o, "submitted_at", None)),
                text=f"{getattr(o, 'side', '')} {getattr(o, 'quantity', 0)} {getattr(o, 'symbol', '')} - {getattr(o, 'status', '')}",
                severity=Severity.INFO,
            )
        )

    alerts: list = []
    for a in getattr(snap, "alerts", []) or []:
        level = getattr(a, "level", None)
        severity = Severity.INFO
        if level is not None:
            name = getattr(level, "name", "") if hasattr(level, "name") else str(level)
            if "ERROR" in name or "WARN" in name:
                severity = Severity.WARNING
            if "ERROR" in name:
                severity = Severity.ERROR
        alerts.append(
            TimelineEvent(
                timestamp=_ts_to_iso(getattr(a, "timestamp", None)),
                text=getattr(a, "message", "") or "",
                severity=severity,
            )
        )

    risk_status = getattr(snap, "risk", None)
    risk_dict: Optional[Dict[str, Any]] = None
    if risk_status is not None:
        risk_dict = {
            "allowed": getattr(risk_status, "allowed", True),
            "reason": getattr(risk_status, "reason", "") or None,
            "updated_at": _ts_to_iso(getattr(risk_status, "updated_at", None)),
        }

    return SnapshotPayload(
        generated_at=_ts_to_iso(getattr(snap, "generated_at", None)),
        mode=getattr(snap, "mode", "DRY-RUN") or "DRY-RUN",
        strategy=getattr(snap, "strategy", "STOPPED") or "STOPPED",
        account_id=getattr(snap, "account_id", "") or "",
        metrics=m,
        symbols=symbols,
        positions=positions,
        historic=historic,
        orders=orders,
        alerts=alerts,
        decisions=[],
        risk=risk_dict,
    )
