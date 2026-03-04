"""
api_converter.py - Convert LEAN data format to API contract format

This module converts LEAN internal data structures to the API contract format
defined in agents/shared/API_CONTRACT.md.
"""

from typing import Dict, List
from datetime import datetime, timezone
import logging

from .api_models import (
    SnapshotResponse,
    Metrics,
    SymbolSnapshot,
    PositionSnapshot,
    OrderSnapshot,
    RiskStatus,
    CandleData
)

logger = logging.getLogger(__name__)


class ApiConverter:
    """Convert LEAN format to API contract format."""

    def build_snapshot(
        self,
        portfolio: Dict,
        positions: List[Dict],
        orders: List[Dict],
        metrics: Dict,
        symbols: List[Dict],
        algorithm
    ) -> SnapshotResponse:
        """
        Build snapshot response matching API contract.

        Args:
            portfolio: Portfolio data from LEAN
            positions: List of position dictionaries
            orders: List of order dictionaries
            metrics: Metrics dictionary
            symbols: List of symbol dictionaries
            algorithm: LEAN algorithm instance (for additional data)

        Returns:
            SnapshotResponse matching API contract
        """
        try:
            # Convert positions
            position_snapshots = [
                self._convert_position(pos) for pos in positions
            ]

            # Convert orders
            order_snapshots = [
                self._convert_order(order) for order in orders
            ]

            # Convert symbols
            symbol_snapshots = self._convert_symbols(symbols, algorithm)

            # Build metrics
            metrics_obj = Metrics(
                net_liq=metrics.get("net_liq", 0.0),
                buying_power=metrics.get("buying_power", 0.0),
                excess_liquidity=metrics.get("excess_liquidity", 0.0),
                margin_requirement=metrics.get("margin_requirement", 0.0),
                commissions=metrics.get("commissions", 0.0),
                portal_ok=metrics.get("portal_ok", True),
                tws_ok=metrics.get("tws_ok", True),
                orats_ok=metrics.get("orats_ok", True),
                questdb_ok=metrics.get("questdb_ok", True)
            )

            # Build risk status
            risk_status = RiskStatus(
                allowed=True,  # From LEAN risk checks (would be enhanced)
                reason=None,
                updated_at=datetime.now(timezone.utc)
            )

            # Get account ID and mode
            account_id = self._get_account_id(algorithm)
            mode = self._get_mode(algorithm)
            strategy_status = "RUNNING" if algorithm else "IDLE"

            return SnapshotResponse(
                generated_at=datetime.now(timezone.utc),
                mode=mode,
                strategy=strategy_status,
                account_id=account_id,
                metrics=metrics_obj,
                symbols=symbol_snapshots,
                positions=position_snapshots,
                historic=[],  # From QuestDB or LEAN history (future enhancement)
                orders=order_snapshots,
                decisions=[],  # From LEAN strategy decisions (future enhancement)
                alerts=[],     # From LEAN alerts/logs (future enhancement)
                risk=risk_status
            )
        except Exception as e:
            logger.error(f"Error building snapshot: {e}")
            raise ValueError(f"Failed to build snapshot: {str(e)}")

    def _convert_position(self, pos: Dict) -> PositionSnapshot:
        """Convert LEAN position to PositionSnapshot."""
        symbol = pos.get("symbol", "UNKNOWN")
        quantity = pos.get("quantity", 0)
        cost_basis = pos.get("average_price", 0.0)
        current_price = pos.get("current_price", cost_basis)
        unrealized_pnl = pos.get("unrealized_profit", 0.0)

        # Calculate unrealized PnL if not provided
        if unrealized_pnl == 0.0 and quantity != 0:
            unrealized_pnl = (current_price - cost_basis) * quantity

        return PositionSnapshot(
            id=f"POS-{symbol}",
            symbol=symbol,
            quantity=quantity,
            cost_basis=cost_basis,
            mark=current_price,
            unrealized_pnl=unrealized_pnl
        )

    def _convert_order(self, order: Dict) -> OrderSnapshot:
        """Convert LEAN order to OrderSnapshot."""
        order_id = order.get("id", "UNKNOWN")
        symbol = order.get("symbol", "UNKNOWN")
        status = order.get("status", "UNKNOWN")
        timestamp = order.get("timestamp")

        # Determine side from quantities (if available)
        quantities = order.get("quantities", [])
        side = "BUY"
        if quantities:
            # If first quantity is negative, it's a sell
            if quantities[0] < 0:
                side = "SELL"

        # Default quantity
        quantity = abs(quantities[0]) if quantities else 1

        # Convert timestamp
        if timestamp is None:
            submitted_at = datetime.now(timezone.utc)
        elif isinstance(timestamp, datetime):
            submitted_at = timestamp
        else:
            submitted_at = datetime.now(timezone.utc)

        return OrderSnapshot(
            id=str(order_id),
            symbol=symbol,
            side=side,
            quantity=quantity,
            status=status,
            submitted_at=submitted_at
        )

    def _convert_symbols(self, symbols: List[Dict], algorithm) -> List[SymbolSnapshot]:
        """Convert symbol data to SymbolSnapshot list."""
        symbol_snapshots = []

        for symbol_data in symbols:
            try:
                symbol = symbol_data.get("symbol", "")
                last = symbol_data.get("last", 0.0)
                bid = symbol_data.get("bid", last)
                ask = symbol_data.get("ask", last)
                spread = symbol_data.get("spread", ask - bid if ask > bid else 0.0)
                volume = symbol_data.get("volume", 0)

                # Build candle data (simplified - would be enhanced with historical data)
                candle = None
                if algorithm and hasattr(algorithm, 'Securities'):
                    try:
                        security = algorithm.Securities.get(symbol)
                        if security and hasattr(security, 'Price'):
                            # Build simple candle from current price
                            price = float(security.Price)
                            candle = CandleData(
                                open=price,
                                high=price,
                                low=price,
                                close=price,
                                volume=volume,
                                entry=price,
                                updated=datetime.now(timezone.utc)
                            )
                    except Exception:
                        pass  # Skip candle if not available

                symbol_snapshots.append(SymbolSnapshot(
                    symbol=symbol,
                    last=last,
                    bid=bid,
                    ask=ask,
                    spread=spread,
                    roi=0.0,  # Calculate from box spread opportunities (future)
                    maker_count=0,  # From order history (future)
                    taker_count=0,  # From order history (future)
                    volume=volume,
                    candle=candle
                ))
            except Exception as e:
                logger.warning(f"Error converting symbol {symbol_data.get('symbol', 'UNKNOWN')}: {e}")
                continue

        return symbol_snapshots

    def _get_account_id(self, algorithm) -> str:
        """Get account ID from LEAN."""
        if algorithm and hasattr(algorithm, 'AccountId'):
            try:
                return str(algorithm.AccountId)
            except Exception:
                pass

        # Default account ID
        return "DU123456"

    def _get_mode(self, algorithm) -> str:
        """Get trading mode from LEAN."""
        if algorithm:
            # Check LEAN configuration or algorithm state
            # This would typically come from LEAN config
            # For now, default to DRY-RUN
            pass

        return "DRY-RUN"  # Default - should be determined from LEAN config
