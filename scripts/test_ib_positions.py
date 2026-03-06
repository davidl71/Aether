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
            print(f"Total positions: {len(positions)}")
            if not positions:
                print("No positions. This is normal for an empty or paper account.")
            else:
                for i, pos in enumerate(positions[:20]):  # cap at 20
                    conid = pos.get("conid") or pos.get("contractId")
                    symbol = pos.get("ticker") or pos.get("symbol") or pos.get("assetClass", "")
                    position = pos.get("position", pos.get("quantity", 0))
                    avg_cost = pos.get("avgCost") or pos.get("avgPrice") or ""
                    print(f"  [{i+1}] conid={conid} symbol={symbol} position={position} avgCost={avg_cost}")
                if len(positions) > 20:
                    print(f"  ... and {len(positions) - 20} more")
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
