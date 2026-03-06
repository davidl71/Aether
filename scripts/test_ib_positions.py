#!/usr/bin/env python3
"""
Test pulling positions from IBKR via the Client Portal API.

Uses the same path as the TUI/IB service: Client Portal Gateway on port 5001.
Ensure the IB Client Portal Gateway is running and you are logged in at https://localhost:5001
(Client Portal and TWS are exclusive — only one can be logged in at a time.)

Usage:
  uv run python scripts/test_ib_positions.py
  uv run python scripts/test_ib_positions.py --base-url https://localhost:5001/v1/portal
  IB_PORTAL_URL=https://localhost:5001/v1/portal uv run python scripts/test_ib_positions.py
"""

from __future__ import annotations

import argparse
import os
import sys
from pathlib import Path

# Suppress InsecureRequestWarning for localhost Gateway (verify_ssl=False)
try:
    import urllib3
    urllib3.disable_warnings(urllib3.exceptions.InsecureRequestWarning)
except Exception:
    pass

# Allow importing python.integration from repo root
_repo_root = Path(__file__).resolve().parent.parent
if str(_repo_root) not in sys.path:
    sys.path.insert(0, str(_repo_root))

from python.integration.ibkr_portal_client import IBKRPortalClient, IBKRPortalError
from python.integration.combo_detector import detect_box_spreads


def main() -> int:
    parser = argparse.ArgumentParser(description="Test pulling positions from IBKR Client Portal")
    parser.add_argument(
        "--base-url",
        default=os.getenv("IB_PORTAL_URL", "https://localhost:5001/v1/portal"),
        help="Client Portal base URL (default: IB_PORTAL_URL or https://localhost:5001/v1/portal)",
    )
    parser.add_argument(
        "--account",
        default=None,
        help="Account ID to use (default: first account)",
    )
    parser.add_argument(
        "--timeout",
        type=int,
        default=15,
        help="Request timeout in seconds (default: 15)",
    )
    args = parser.parse_args()

    print("=== IBKR positions test (Client Portal) ===")
    print()
    print(f"Gateway: {args.base_url}")
    print(f"Timeout: {args.timeout}s")
    print()
    print("Connecting and ensuring session...")
    client = IBKRPortalClient(
        base_url=args.base_url.rstrip("/"),
        verify_ssl=False,
        timeout_seconds=args.timeout,
    )

    try:
        accounts = client.get_accounts()
        if not accounts:
            print("No accounts returned. Is the Gateway logged in? Open https://localhost:5001 and log in.")
            return 1
        print(f"Accounts: {', '.join(accounts)}")
        account_id = args.account or accounts[0]
        print(f"Using account: {account_id}")
        print()

        # Account summary
        print("=== Account summary ===")
        try:
            summary = client.get_account_summary(account_id)
            if isinstance(summary, dict):
                for key in ("NetLiquidation", "TotalCashValue", "BuyingPower", "GrossPositionValue"):
                    val = summary.get(key) or summary.get(key.replace("_", ""))
                    if val is not None:
                        print(f"  {key}: {val}")
            else:
                print("  (summary format unexpected)")
        except IBKRPortalError as e:
            print(f"  Summary error: {e}")
        print()

        # Positions
        print("=== Positions ===")
        try:
            positions = client.get_portfolio_positions(account_id)
            print(f"Total positions (raw): {len(positions)}")
            if not positions:
                print("No positions. This is normal for an empty or paper account.")
            else:
                combos, remaining = detect_box_spreads(positions)
                if combos:
                    print()
                    print("--- Combo / box spreads (detected) ---")
                    for c in combos:
                        desc = c.get("contractDesc") or f"{c.get('underlying')} {c.get('expiry')} {c.get('k1')}/{c.get('k2')} box"
                        qty = c.get("quantity", 1)
                        mkt = c.get("mktValue", 0)
                        pnl = c.get("unrealizedPnl", 0)
                        print(f"  Box: {desc}  qty={qty}  mktValue={mkt:,.2f}  PnL={pnl:,.2f}")
                    print()
                if remaining:
                    print("--- Other positions ---")
                    for i, pos in enumerate(remaining[:25]):
                        desc = pos.get("contractDesc") or pos.get("ticker") or pos.get("symbol") or pos.get("assetClass", "")
                        position = pos.get("position", pos.get("quantity", 0))
                        mkt = pos.get("mktValue", 0)
                        pnl = pos.get("unrealizedPnl", 0)
                        asset = pos.get("assetClass", "")
                        print(f"  [{i+1}] {desc[:50]:<50} pos={position:>8.1f}  mktValue={mkt:>12,.2f}  PnL={pnl:>10,.2f}  ({asset})")
                    if len(remaining) > 25:
                        print(f"  ... and {len(remaining) - 25} more")
        except IBKRPortalError as e:
            print(f"Positions error: {e}")
            return 1

        print()
        print("Done.")
        return 0

    except IBKRPortalError as e:
        print(f"Error: {e}")
        print()
        print("Troubleshooting:")
        print("  1. Start the IB Client Portal Gateway and log in at https://localhost:5001")
        print("  2. Client Portal and TWS are exclusive — only one can be logged in at a time")
        print("  3. If you use TWS socket (7496/7497), log out of Client Portal first")
        return 1


if __name__ == "__main__":
    sys.exit(main())
