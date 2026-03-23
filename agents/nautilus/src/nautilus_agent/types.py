"""Type conversion utilities: NautilusTrader types → project protobuf types.

All functions are pure (no side effects). NT prices use Intel Decimal (MDF)
via Rust — convert to float only at the serialization boundary here.
"""

from __future__ import annotations

import time

from google.protobuf import timestamp_pb2

# NautilusTrader types
from nautilus_trader.model.data import QuoteTick, TradeTick
from nautilus_trader.model.events import OrderFilled, PositionChanged, PositionOpened
from nautilus_trader.model.enums import OrderSide
from nautilus_trader.model.identifiers import InstrumentId

from nautilus_agent.generated import messages_pb2 as pb


def _now_ts() -> timestamp_pb2.Timestamp:
    now = time.time()
    return timestamp_pb2.Timestamp(
        seconds=int(now),
        nanos=int((now % 1) * 1_000_000_000),
    )


def _ns_to_ts(ns: int) -> timestamp_pb2.Timestamp:
    """Convert nanosecond epoch timestamp to protobuf Timestamp."""
    seconds = ns // 1_000_000_000
    nanos = ns % 1_000_000_000
    return timestamp_pb2.Timestamp(seconds=seconds, nanos=nanos)


NAUTILUS_SOURCE = "nautilus"
NAUTILUS_PRIORITY = 100


def quote_tick_to_market_data_event(tick: QuoteTick, symbol: str) -> pb.MarketDataEvent:
    """Convert a NT QuoteTick to MarketDataEvent proto for NATS publishing."""
    return pb.MarketDataEvent(
        symbol=symbol,
        bid=float(tick.bid_price),
        ask=float(tick.ask_price),
        last=float(tick.bid_price),  # no last-trade in QuoteTick; use bid as proxy
        volume=int(tick.bid_size) + int(tick.ask_size),
        timestamp=_ns_to_ts(tick.ts_event),
        source=NAUTILUS_SOURCE,
        source_priority=NAUTILUS_PRIORITY,
    )


def trade_tick_to_market_data_event(tick: TradeTick, symbol: str) -> pb.MarketDataEvent:
    """Convert a NT TradeTick to MarketDataEvent proto."""
    return pb.MarketDataEvent(
        symbol=symbol,
        bid=float(tick.price),
        ask=float(tick.price),
        last=float(tick.price),
        volume=int(tick.size),
        timestamp=_ns_to_ts(tick.ts_event),
        source=NAUTILUS_SOURCE,
        source_priority=NAUTILUS_PRIORITY,
    )


def order_filled_to_box_spread_execution(
    event: OrderFilled,
    symbol: str,
    lower_strike: int,
    upper_strike: int,
    expiry: str,
) -> pb.BoxSpreadExecution:
    """Convert an NT OrderFilled event to BoxSpreadExecution proto."""
    return pb.BoxSpreadExecution(
        symbol=symbol,
        lower_strike=lower_strike,
        upper_strike=upper_strike,
        expiry=expiry,
        net_debit=float(event.last_px),
        trade_id=str(event.order_side.value) + "-" + str(event.trade_id),
        executed_at=_ns_to_ts(event.ts_event),
    )


def position_to_proto(
    event: PositionOpened | PositionChanged,
    symbol: str,
) -> pb.Position:
    """Convert an NT position event to Position proto."""
    pos = event.position
    return pb.Position(
        id=str(pos.id),
        symbol=symbol,
        quantity=int(pos.quantity),
        cost_basis=float(pos.avg_px_open) if pos.avg_px_open else 0.0,
        mark=float(pos.unrealized_pnl) + float(pos.avg_px_open)
        if pos.avg_px_open
        else 0.0,
        unrealized_pnl=float(pos.unrealized_pnl) if pos.unrealized_pnl else 0.0,
    )


def instrument_id_symbol(instrument_id: InstrumentId) -> str:
    """Extract the underlying symbol from a NT InstrumentId.

    NT IB simplified format examples:
      SPX240412C05100000.CBOE → "SPX"
      SPX.CBOE               → "SPX"
    """
    raw = instrument_id.symbol.value
    # Strip trailing option strike/expiry/type suffix
    for sym in ("SPX", "XSP", "NDX", "SPXW"):
        if raw.upper().startswith(sym):
            return sym
    return raw.split(".")[0]
