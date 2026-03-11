from __future__ import annotations

import logging
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timezone
from typing import Any, Dict, List, Optional

from .ibkr_portal_client import IBKRPortalClient, IBKRPortalError

logger = logging.getLogger(__name__)

try:
    from .combo_detector import detect_box_spreads
except ImportError:
    detect_box_spreads = None  # type: ignore[misc, assignment]


_prewarmed_symbol_keys: set = set()


def _now_iso() -> str:
    return datetime.now(timezone.utc).isoformat()


def _infer_ib_session_mode(account_id: Optional[str]) -> str:
    if not account_id or not str(account_id).strip():
        return "LIVE"
    aid = str(account_id).strip().upper()
    if aid.startswith("DU"):
        return "PAPER"
    return "LIVE"


def _ensure_conids_prewarmed(client: IBKRPortalClient, symbols: List[str]) -> None:
    key = tuple(sorted(symbols))
    if key in _prewarmed_symbol_keys:
        return
    client.prewarm_conids(symbols)
    _prewarmed_symbol_keys.add(key)


def _format_ibcid_display_name(
    raw_name: str,
    asset_class: str,
    conid: Optional[int],
    maturity_date_str: Optional[str],
) -> str:
    if not raw_name or conid is None:
        return raw_name or ""
    is_ibcid = raw_name.strip().upper().startswith("IBCID") or (
        raw_name.strip().isdigit() and str(conid) == raw_name.strip()
    )
    if not is_ibcid:
        return raw_name
    ac = (asset_class or "").strip().upper()
    if ac in ("BILL", "TBILL"):
        label = "T-Bill"
    elif ac == "BOND":
        label = "Bond"
    else:
        return raw_name
    maturity_part = ""
    if maturity_date_str and len(maturity_date_str) >= 10:
        maturity_part = maturity_date_str[:10] + " "
    return f"{label} {maturity_part}({conid})".strip()


def _extract_account_value(summary: Dict, key: str, default: float = 0.0) -> float:
    if not isinstance(summary, dict):
        return default

    keys_to_try = [key]
    if key == "TotalCashValue":
        keys_to_try = ["TotalCashValue", "totalCashValue", "CashBalance", "cashBalance", "AvailableFunds"]
    elif key == "NetLiquidation":
        keys_to_try = ["NetLiquidation", "netLiquidation", "NetLiquidationValue"]
    elif key == "BuyingPower":
        keys_to_try = ["BuyingPower", "buyingPower"]
    elif key == "ExcessLiquidity":
        keys_to_try = ["ExcessLiquidity", "excessLiquidity"]
    elif key == "MaintMarginReq":
        keys_to_try = ["MaintMarginReq", "maintMarginReq", "MaintMarginRequirement"]

    for k in keys_to_try:
        raw = summary.get(k)
        if raw is None:
            continue
        if isinstance(raw, list) and len(raw) > 0:
            first = raw[0]
            if isinstance(first, dict):
                value_str = first.get("value") or first.get("amount")
                if value_str is not None:
                    try:
                        return float(value_str)
                    except (ValueError, TypeError):
                        pass
            elif isinstance(first, (int, float)):
                return float(first)
        if isinstance(raw, (int, float)):
            return float(raw)
        if isinstance(raw, str) and raw.strip():
            try:
                return float(raw)
            except (ValueError, TypeError):
                pass
    return default


def _extract_cash_by_currency_from_summary(summary: Optional[Dict]) -> List[Dict[str, Any]]:
    if not isinstance(summary, dict):
        return []

    out: List[Dict[str, Any]] = []
    seen_currencies: set = set()

    for key in ("cash", "balanceByCurrency", "cashBalanceByCurrency", "ledger"):
        raw = summary.get(key)
        if isinstance(raw, dict):
            for curr, val in raw.items():
                if not isinstance(curr, str) or curr.upper() in ("TIMESTAMP", "ACCOUNTID", "LEDGER"):
                    continue
                curr = curr.strip().upper()
                if not curr or len(curr) != 3:
                    continue
                try:
                    bal = float(val) if val is not None else 0.0
                except (ValueError, TypeError):
                    continue
                if curr not in seen_currencies:
                    out.append({"currency": curr, "balance": bal})
                    seen_currencies.add(curr)
            if out:
                return out
        elif isinstance(raw, list):
            for item in raw:
                if not isinstance(item, dict):
                    continue
                curr = (item.get("currency") or item.get("currencyCode") or "").strip().upper()
                if not curr or len(curr) != 3:
                    continue
                val = item.get("value") or item.get("balance") or item.get("amount") or item.get("cashbalance")
                try:
                    bal = float(val) if val is not None else 0.0
                except (ValueError, TypeError):
                    continue
                if curr not in seen_currencies:
                    out.append({"currency": curr, "balance": bal})
                    seen_currencies.add(curr)
            if out:
                return out

    common_prefixes = ("TotalCashValue", "totalCashValue", "CashBalance", "cashBalance", "AvailableFunds")
    for k, v in summary.items():
        if not isinstance(k, str) or v is None:
            continue
        for prefix in common_prefixes:
            if k == prefix:
                try:
                    val = float(v) if isinstance(v, (int, float)) else float(v[0].get("value", 0)) if isinstance(v, list) and v and isinstance(v[0], dict) else 0.0
                except (ValueError, TypeError, KeyError):
                    continue
                if "USD" not in seen_currencies and val != 0:
                    out.append({"currency": "USD", "balance": val})
                    seen_currencies.add("USD")
                break
            if k.startswith(prefix):
                rest = k[len(prefix):].lstrip("|.-_")
                if len(rest) >= 3 and rest.isalpha():
                    curr = rest[:3].upper()
                    try:
                        val = float(v) if isinstance(v, (int, float)) else float(v[0].get("value", 0)) if isinstance(v, list) and v and isinstance(v[0], dict) else 0.0
                    except (ValueError, TypeError, KeyError):
                        continue
                    if curr not in seen_currencies:
                        out.append({"currency": curr, "balance": val})
                        seen_currencies.add(curr)
                break

    if "USD" not in seen_currencies:
        usd = _extract_account_value(summary, "TotalCashValue")
        if usd != 0.0:
            out.append({"currency": "USD", "balance": usd})
            seen_currencies.add("USD")

    return out


def _float_or_none(val: Any) -> Optional[float]:
    if val is None:
        return None
    try:
        return float(val)
    except (ValueError, TypeError):
        return None


def _expiry_str_to_date(expiry: str) -> str:
    if not expiry or not isinstance(expiry, str):
        return ""
    import calendar

    s = expiry.strip().upper()
    months = {"JAN": 1, "FEB": 2, "MAR": 3, "APR": 4, "MAY": 5, "JUN": 6,
              "JUL": 7, "AUG": 8, "SEP": 9, "OCT": 10, "NOV": 11, "DEC": 12}
    for mon_name, mon_num in months.items():
        if s.startswith(mon_name):
            try:
                year = int(s[len(mon_name):])
                if year < 100:
                    year += 2000
                cal = calendar.Calendar(calendar.FRIDAY)
                fridays = [d for d in cal.itermonthdates(year, mon_num) if d.month == mon_num]
                if len(fridays) >= 3:
                    d = fridays[2]
                    return f"{d.year:04d}-{d.month:02d}-{d.day:02d}"
                _, last = calendar.monthrange(year, mon_num)
                return f"{year:04d}-{mon_num:02d}-{last:02d}"
            except (ValueError, TypeError):
                pass
            break
    return ""


def _build_cash_flow_timeline(
    positions: List[Dict[str, Any]],
    reported_future_events: Optional[List[Dict[str, Any]]] = None,
) -> Optional[Dict[str, Any]]:
    try:
        from .cash_flow_timeline import calculate_cash_flow_timeline
    except ImportError:
        return None

    if not positions and not (reported_future_events and len(reported_future_events) > 0):
        return None

    try:
        result = calculate_cash_flow_timeline(
            positions=positions,
            bank_accounts=[],
            projection_months=12,
            reported_future_events=reported_future_events or [],
        )
        return {
            "events": [
                {"date": e.date, "amount": e.amount, "description": e.description,
                 "position_name": e.position_name, "type": e.type}
                for e in result.events
            ],
            "monthly_flows": {
                month: {"month": m.month, "inflows": m.inflows, "outflows": m.outflows,
                        "net": m.net, "events": [
                            {"date": ev.date, "amount": ev.amount, "description": ev.description,
                             "position_name": ev.position_name, "type": ev.type}
                            for ev in m.events
                        ]}
                for month, m in result.monthly_flows.items()
            },
            "total_inflows": result.total_inflows,
            "total_outflows": result.total_outflows,
            "net_cash_flow": result.net_cash_flow,
        }
    except Exception:
        logger.warning("Failed to generate cash flow timeline", exc_info=True)
        return None


def build_snapshot_payload(
    symbols: List[str], client: IBKRPortalClient, account_id: Optional[str] = None
) -> Dict[str, Any]:
    try:
        accounts = client.get_accounts()
        effective_account_id = account_id or (accounts[0] if accounts else None)
    except IBKRPortalError:
        effective_account_id = account_id
    display_account_id = effective_account_id if effective_account_id is not None else "IBKR"

    client.ensure_session()
    client.set_session_ensured_for_request(True)
    try:
        _ensure_conids_prewarmed(client, symbols)
        ledger_rows: List[Dict] = []

        def fetch_market_data() -> List[Dict]:
            try:
                return client.get_snapshots_batch(symbols)
            except IBKRPortalError:
                return [{}] * len(symbols)

        def fetch_summary() -> Optional[Dict]:
            try:
                return client.get_account_summary(effective_account_id)
            except IBKRPortalError as e:
                logger.warning("Failed to get account summary: %s", e)
                return None

        def fetch_positions() -> List[Dict]:
            try:
                return client.get_portfolio_positions(effective_account_id)
            except IBKRPortalError as e:
                logger.warning("Failed to get positions: %s", e)
                return []

        def fetch_ledger() -> List[Dict]:
            try:
                return client.get_account_ledger(effective_account_id)
            except IBKRPortalError as e:
                logger.debug("Ledger not available: %s", e)
                return []

        with ThreadPoolExecutor(max_workers=4) as pool:
            fut_snap = pool.submit(fetch_market_data)
            fut_summary = pool.submit(fetch_summary)
            fut_pos = pool.submit(fetch_positions)
            fut_ledger = pool.submit(fetch_ledger)
            snapshots_list = fut_snap.result()
            account_summary = fut_summary.result()
            positions_raw = fut_pos.result()
            ledger_rows = fut_ledger.result()
    finally:
        client.set_session_ensured_for_request(False)

    symbol_snapshots = []
    for i, sym in enumerate(symbols):
        try:
            s = snapshots_list[i] if i < len(snapshots_list) else {}
            spread = s.get("ask", 0.0) - s.get("bid", 0.0) if s.get("ask") and s.get("bid") else 0.0
            last_price = s.get("last", 0.0) or s.get("close", 0.0)
            symbol_snapshots.append(
                {
                    "symbol": sym,
                    "last": float(last_price),
                    "bid": float(s.get("bid", 0.0)),
                    "ask": float(s.get("ask", 0.0)),
                    "spread": float(spread),
                    "roi": 0.0,
                    "maker_count": 0,
                    "taker_count": 0,
                    "volume": int(s.get("volume", 0.0)),
                    "candle": {
                        "open": float(s.get("close", 0.0)),
                        "high": float(s.get("last", 0.0) or s.get("close", 0.0)),
                        "low": float(s.get("last", 0.0) or s.get("close", 0.0)),
                        "close": float(s.get("close", 0.0) or s.get("last", 0.0)),
                        "volume": int(s.get("volume", 0.0)),
                        "entry": float(s.get("last", 0.0) or s.get("close", 0.0)),
                        "updated": _now_iso(),
                    },
                    "option_chains": [],
                }
            )
        except (KeyError, TypeError, ValueError) as e:
            logger.warning("Failed to format snapshot for %s: %s", sym, e)
            symbol_snapshots.append(
                {
                    "symbol": sym,
                    "last": 0.0,
                    "bid": 0.0,
                    "ask": 0.0,
                    "spread": 0.0,
                    "roi": 0.0,
                    "maker_count": 0,
                    "taker_count": 0,
                    "volume": 0,
                    "candle": {
                        "open": 0.0,
                        "high": 0.0,
                        "low": 0.0,
                        "close": 0.0,
                        "volume": 0,
                        "entry": 0.0,
                        "updated": _now_iso(),
                    },
                    "option_chains": [],
                }
            )

    metrics = {
        "net_liq": _extract_account_value(account_summary, "NetLiquidation") if account_summary else 0.0,
        "buying_power": _extract_account_value(account_summary, "BuyingPower") if account_summary else 0.0,
        "excess_liquidity": _extract_account_value(account_summary, "ExcessLiquidity") if account_summary else 0.0,
        "margin_requirement": _extract_account_value(account_summary, "MaintMarginReq") if account_summary else 0.0,
        "commissions": 0.0,
        "portal_ok": account_summary is not None,
        "tws_ok": False,
        "questdb_ok": False,
    }

    positions_data = []
    future_events: List[Dict[str, Any]] = []
    if detect_box_spreads:
        try:
            combos, remaining = detect_box_spreads(positions_raw)
            for c in combos:
                if c.get("type") == "box_spread":
                    name = f"Box: {c.get('underlying', '')} {c.get('expiry', '')} {c.get('k1')}/{c.get('k2')} box"
                    qty = int(c.get("quantity", 0))
                    mkt = float(c.get("mktValue", 0) or 0)
                    pnl = float(c.get("unrealizedPnl", 0) or 0)
                    side = c.get("side", "long" if qty > 0 else "short")
                    exp_cash = c.get("expected_cash_at_expiry")
                    positions_data.append({
                        "name": name,
                        "symbol": name,
                        "quantity": qty,
                        "avg_price": 0.0,
                        "current_price": mkt / qty if qty else 0.0,
                        "market_value": mkt,
                        "unrealized_pl": pnl,
                        "roi": (pnl / mkt * 100.0) if mkt else 0.0,
                        "instrument_type": "box_spread",
                        "side": side,
                        "expected_cash_at_expiry": float(exp_cash) if exp_cash is not None else None,
                        "currency": "USD",
                        "dividend": None,
                    })
                    if exp_cash is not None and qty != 0:
                        exp_date = _expiry_str_to_date(c.get("expiry") or "")
                        future_events.append({
                            "event_type": "expiry",
                            "date": exp_date,
                            "amount": float(exp_cash),
                            "currency": "USD",
                            "position_name": name,
                            "description": "Box spread expiry",
                        })
            positions_raw = remaining
        except Exception as e:
            logger.debug("Combo detection skipped: %s", e)

    for pos in positions_raw:
        if not isinstance(pos, dict):
            continue
        name = (pos.get("ticker") or pos.get("symbol") or pos.get("contractDesc") or "").strip()
        if not name:
            name = str(pos.get("conid", ""))
        conid_val = pos.get("conid")
        if conid_val is not None:
            try:
                conid_val = int(conid_val)
            except (ValueError, TypeError):
                conid_val = None
        asset_class = (pos.get("assetClass") or "").strip().upper()
        maturity_date_str = (pos.get("maturityDate") or pos.get("maturity_date") or "").strip()
        if isinstance(maturity_date_str, (int, float)):
            maturity_date_str = str(int(maturity_date_str))
        name = _format_ibcid_display_name(name, asset_class, conid_val, maturity_date_str or None)
        qty = int(float(pos.get("position", 0.0)))
        avg_cost = float(pos.get("avgCost", 0.0) or pos.get("averageCost", 0.0))
        mkt_val = float(pos.get("mktValue", 0.0) or pos.get("markValue", 0.0))
        cur = float(pos.get("markPrice", 0.0) or pos.get("lastPrice", 0.0))
        if not cur and qty:
            cur = mkt_val / qty
        pnl = float(pos.get("unrealizedPnl", 0.0))
        roi = (pnl / mkt_val * 100.0) if mkt_val else 0.0
        bid = _float_or_none(pos.get("bidPrice") or pos.get("bid"))
        ask = _float_or_none(pos.get("askPrice") or pos.get("ask"))
        last = _float_or_none(pos.get("lastPrice") or pos.get("last") or pos.get("markPrice"))
        spread = (ask - bid) if (bid is not None and ask is not None) else None
        price = last or cur
        side = "long" if qty > 0 else "short" if qty < 0 else None
        curr = (pos.get("currency") or "USD").strip() or "USD"
        exp_cash = pos.get("expected_cash_at_expiry")
        div = _float_or_none(pos.get("dividend") or pos.get("dividendAmount") or pos.get("nextDividendAmount"))
        if div is None and pos.get("dividendPerShare") is not None and qty:
            try:
                dps = float(pos.get("dividendPerShare", 0) or 0)
                if dps:
                    div = dps * abs(qty)
            except (ValueError, TypeError):
                pass
        inst_type = "bond" if asset_class == "BOND" else "t_bill" if asset_class in ("BILL", "TBILL") else None
        positions_data.append({
            "name": name,
            "symbol": name,
            "conid": conid_val,
            "quantity": qty,
            "avg_price": avg_cost,
            "current_price": cur,
            "market_value": mkt_val,
            "unrealized_pl": pnl,
            "roi": roi,
            "bid": bid,
            "ask": ask,
            "last": last,
            "spread": spread,
            "price": price,
            "side": side,
            "currency": curr,
            "expected_cash_at_expiry": float(exp_cash) if exp_cash is not None else None,
            "dividend": div,
            "instrument_type": inst_type,
            "maturity_date": maturity_date_str[:10] if maturity_date_str else None,
        })
        if div is not None and div != 0:
            future_events.append({
                "event_type": "dividend",
                "date": (pos.get("exDate") or pos.get("nextDividendDate") or "").strip()[:10] or "",
                "amount": float(div),
                "currency": curr,
                "position_name": name,
                "description": "Dividend",
            })
        if maturity_date_str and (exp_cash is not None or mkt_val):
            amt = float(exp_cash) if exp_cash is not None else mkt_val
            if amt != 0:
                future_events.append({
                    "event_type": "principal_repayment",
                    "date": maturity_date_str[:10] if len(maturity_date_str) >= 10 else maturity_date_str,
                    "amount": amt,
                    "currency": curr,
                    "position_name": name,
                    "description": "Principal at maturity",
                })

    if ledger_rows and isinstance(ledger_rows, list):
        for row in ledger_rows:
            if not isinstance(row, dict):
                continue
            curr = (row.get("currency") or "").strip().upper()
            bal = row.get("balance")
            if curr is None:
                continue
            if curr == "":
                curr = "USD"
            try:
                balance = float(bal) if bal is not None else 0.0
            except (ValueError, TypeError):
                balance = 0.0
            positions_data.append({
                "name": f"Cash ({curr})",
                "symbol": "Cash",
                "quantity": 0,
                "avg_price": 0.0,
                "current_price": balance,
                "market_value": balance,
                "unrealized_pl": 0.0,
                "roi": 0.0,
                "instrument_type": "cash",
                "currency": curr,
                "dividend": None,
            })
    else:
        summary_cash = _extract_cash_by_currency_from_summary(account_summary) if account_summary else []
        if summary_cash:
            for row in summary_cash:
                curr = (row.get("currency") or "USD").strip().upper() or "USD"
                balance = float(row.get("balance", 0.0))
                positions_data.append({
                    "name": f"Cash ({curr})",
                    "symbol": "Cash",
                    "quantity": 0,
                    "avg_price": 0.0,
                    "current_price": balance,
                    "market_value": balance,
                    "unrealized_pl": 0.0,
                    "roi": 0.0,
                    "instrument_type": "cash",
                    "currency": curr,
                    "dividend": None,
                })
        else:
            total_cash = _extract_account_value(account_summary, "TotalCashValue") if account_summary else 0.0
            if account_summary and total_cash == 0.0:
                logger.debug(
                    "Account summary present but TotalCashValue is 0; summary keys: %s. "
                    "Ledger was empty (use /v1/api/portfolio/{id}/ledger if available).",
                    list(account_summary.keys()) if isinstance(account_summary, dict) else type(account_summary),
                )
            positions_data.append({
                "name": "Cash (USD)",
                "symbol": "Cash",
                "quantity": 0,
                "avg_price": 0.0,
                "current_price": total_cash,
                "market_value": total_cash,
                "unrealized_pl": 0.0,
                "roi": 0.0,
                "instrument_type": "cash",
                "currency": "USD",
                "dividend": None,
            })

    cash_flow_timeline = _build_cash_flow_timeline(positions_data, future_events)
    payload: Dict[str, Any] = {
        "generated_at": _now_iso(),
        "mode": _infer_ib_session_mode(display_account_id),
        "strategy": "box_spread",
        "account_id": display_account_id,
        "metrics": metrics,
        "symbols": symbol_snapshots,
        "positions": positions_data,
        "historic": [],
        "orders": [],
        "decisions": [],
        "alerts": [],
        "future_events": future_events,
        "risk": {
            "allowed": True,
            "reason": None,
            "updated_at": _now_iso(),
        },
    }
    if cash_flow_timeline:
        payload["cash_flow_timeline"] = cash_flow_timeline
    return payload
