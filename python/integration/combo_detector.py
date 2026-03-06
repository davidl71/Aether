"""
Detect combo/box-spread option positions from flat leg positions.

IB Client Portal returns options as individual legs. This module parses
contract descriptions and groups legs that form a box spread (same underlying,
same expiry, two strikes K1 < K2, with long K1 put, short K1 call, long K2 call,
short K2 put).
"""
from __future__ import annotations

import re
from dataclasses import dataclass, field
from typing import Any, Dict, List, Optional, Tuple


# contractDesc examples:
#   "SPX    MAR2027 6825 C [SPX   270319C06825000 100]"
#   "SPX    MAR2027 6825 P [SPX   270319P06825000 100]"
_OPT_DESC_RE = re.compile(
    r"^\s*(\S+)\s+"           # symbol
    r"(\w+\d{4})\s+"          # expiry (e.g. MAR2027)
    r"(\d+(?:\.\d+)?)\s+"     # strike
    r"([CP])\s+"              # C or P
    r"\[",
    re.IGNORECASE,
)


@dataclass
class OptionLeg:
    """Parsed option leg from a position."""
    conid: int
    symbol: str
    expiry: str
    strike: float
    right: str  # "C" or "P"
    position: float
    mktValue: float
    unrealizedPnl: float
    avgCost: float
    contractDesc: str
    raw: Dict[str, Any] = field(repr=False)

    @property
    def is_call(self) -> bool:
        return self.right.upper() == "C"

    @property
    def is_put(self) -> bool:
        return self.right.upper() == "P"


def parse_opt_contract_desc(contract_desc: str) -> Optional[Tuple[str, str, float, str]]:
    """
    Parse IB contractDesc for an option. Returns (symbol, expiry, strike, right) or None.
    """
    if not contract_desc or not isinstance(contract_desc, str):
        return None
    m = _OPT_DESC_RE.match(contract_desc.strip())
    if not m:
        return None
    symbol = (m.group(1) or "").strip()
    expiry = (m.group(2) or "").strip()
    try:
        strike = float(m.group(3) or 0)
    except (ValueError, TypeError):
        return None
    right = (m.group(4) or "C").strip().upper()
    if right not in ("C", "P"):
        return None
    return (symbol, expiry, strike, right)


def legs_to_option_legs(positions: List[Dict[str, Any]]) -> List[OptionLeg]:
    """Convert raw OPT positions to OptionLeg list (only those that parse)."""
    legs: List[OptionLeg] = []
    for p in positions:
        if p.get("assetClass") != "OPT":
            continue
        parsed = parse_opt_contract_desc(p.get("contractDesc") or "")
        if not parsed:
            continue
        symbol, expiry, strike, right = parsed
        legs.append(OptionLeg(
            conid=int(p.get("conid") or 0),
            symbol=symbol,
            expiry=expiry,
            strike=strike,
            right=right,
            position=float(p.get("position") or 0),
            mktValue=float(p.get("mktValue") or 0),
            unrealizedPnl=float(p.get("unrealizedPnl") or 0),
            avgCost=float(p.get("avgCost") or 0),
            contractDesc=str(p.get("contractDesc") or ""),
            raw=p,
        ))
    return legs


def detect_box_spreads(positions: List[Dict[str, Any]]) -> Tuple[List[Dict], List[Dict]]:
    """
    Group OPT positions into box spreads where possible.
    Returns (combo_groups, remaining_flat_positions).
    combo_groups: list of dicts with type="box_spread", underlying, expiry, k1, k2, legs, mktValue, unrealizedPnl.
    remaining_flat_positions: positions not part of a detected combo (stocks, bills, ungrouped options).
    """
    opt_legs = legs_to_option_legs(positions)
    non_opt = [p for p in positions if p.get("assetClass") != "OPT"]
    # Group by (symbol, expiry)
    by_key: Dict[Tuple[str, str], List[OptionLeg]] = {}
    for leg in opt_legs:
        key = (leg.symbol, leg.expiry)
        by_key.setdefault(key, []).append(leg)

    combos: List[Dict[str, Any]] = []
    used_conids: set = set()

    for (symbol, expiry), group in by_key.items():
        if len(group) != 4:
            continue
        strikes = sorted(set(leg.strike for leg in group))
        if len(strikes) != 2:
            continue
        k1, k2 = strikes[0], strikes[1]
        # Box spread: K1 put +1, K1 call -1, K2 call +1, K2 put -1
        def at(k: float, right: str, legs: List[OptionLeg]) -> Optional[OptionLeg]:
            for leg in legs:
                if leg.strike == k and leg.right == right:
                    return leg
            return None
        k1p = at(k1, "P", group)
        k1c = at(k1, "C", group)
        k2c = at(k2, "C", group)
        k2p = at(k2, "P", group)
        if not (k1p and k1c and k2c and k2p):
            continue
        # Check box pattern: K1 P +1, K1 C -1, K2 C +1, K2 P -1 (long) or all negated (short)
        p1, p2, p3, p4 = k1p.position, k1c.position, k2c.position, k2p.position
        if (p1, p2, p3, p4) == (1.0, -1.0, 1.0, -1.0) or (p1, p2, p3, p4) == (-1.0, 1.0, -1.0, 1.0):
            used_conids.update(leg.conid for leg in group)
            is_long = (p1, p2, p3, p4) == (1.0, -1.0, 1.0, -1.0)
            qty = int(abs(p1))
            # At expiry: long box pays (K2-K1)*mult, short box pays -(K2-K1)*mult. SPX/XSP mult=100.
            mult = 100
            payoff_per_unit = (float(k2) - float(k1)) * mult
            expected_cash = payoff_per_unit * qty if is_long else -payoff_per_unit * qty
            combos.append({
                "type": "box_spread",
                "underlying": symbol,
                "expiry": expiry,
                "k1": k1,
                "k2": k2,
                "quantity": qty,
                "side": "long" if is_long else "short",
                "expected_cash_at_expiry": expected_cash,
                "legs": [leg.raw for leg in group],
                "mktValue": sum(leg.mktValue for leg in group),
                "unrealizedPnl": sum(leg.unrealizedPnl for leg in group),
                "contractDesc": f"{symbol} {expiry} {int(k1)}/{int(k2)} box",
            })

    # Remaining positions: non-OPT plus OPT legs not in any combo
    remaining = list(non_opt)
    for p in positions:
        if p.get("assetClass") != "OPT":
            continue
        if (p.get("conid") or 0) in used_conids:
            continue
        remaining.append(p)

    return combos, remaining
