"""BoxSpreadStrategy — NautilusTrader Strategy implementation.

Detects box spread arbitrage opportunities on European-style index options
(SPX, XSP, NDX) and executes them via IB BAG (combo) orders.

Box spread detection logic mirrors C++ BoxSpreadStrategy::find_box_spreads_in_chain()
in native/src/strategies/box_spread/box_spread_strategy.cpp lines 213-248.

NATS events published (via NatsBridge):
  market-data.tick.{symbol}     — on every QuoteTick
  strategy.signal.{symbol}      — on every QuoteTick (mid-price signal, mirrors C++ NatsClient)
  strategy.decision.{symbol}    — when a box spread opportunity is found
  orders.fill.{order_id}        — on OrderFilled
  positions.update.{symbol}     — on PositionOpened/PositionChanged
"""

from __future__ import annotations

import time
from dataclasses import dataclass, field
from typing import TYPE_CHECKING

import structlog
from google.protobuf import timestamp_pb2
from nautilus_trader.config import StrategyConfig
from nautilus_trader.model.data import QuoteTick
from nautilus_trader.model.enums import OrderSide, TimeInForce
from nautilus_trader.model.events import OrderFilled, PositionChanged, PositionOpened
from nautilus_trader.model.identifiers import InstrumentId
from nautilus_trader.trading.strategy import Strategy

from nautilus_agent.generated import messages_pb2 as pb
from nautilus_agent.types import (
    instrument_id_symbol,
    order_filled_to_box_spread_execution,
    position_to_proto,
    quote_tick_to_market_data_event,
)

if TYPE_CHECKING:
    from nautilus_agent.nats_bridge import NatsBridge

log = structlog.get_logger(__name__)


class BoxSpreadConfig(StrategyConfig, frozen=True):
    """Configuration for the box spread strategy."""

    symbols: tuple[str, ...] = ("SPX", "XSP", "NDX")
    min_dte: int = 30
    max_dte: int = 90
    min_arbitrage_profit: float = 0.10
    min_roi_percent: float = 0.5
    max_position_size: float = 10_000.0
    max_bid_ask_spread: float = 0.10
    min_volume: int = 100
    eval_debounce_seconds: float = 1.0
    max_contracts_per_symbol: int = 200


@dataclass
class _OptionQuote:
    """Cached bid/ask for a single option contract."""

    instrument_id: InstrumentId
    expiry: str  # YYYYMMDD
    strike: float
    option_type: str  # "C" or "P"
    bid: float = 0.0
    ask: float = 0.0
    ts: float = field(default_factory=time.time)

    @property
    def mid(self) -> float:
        return (self.bid + self.ask) / 2.0

    @property
    def spread_fraction(self) -> float:
        if self.mid == 0:
            return float("inf")
        return (self.ask - self.bid) / self.mid


@dataclass
class BoxSpreadOpportunity:
    """A detected box spread with pricing details."""

    symbol: str
    lower_strike: float
    upper_strike: float
    expiry: str
    long_call: _OptionQuote      # buy lower-strike call
    short_call: _OptionQuote     # sell upper-strike call
    long_put: _OptionQuote       # buy upper-strike put
    short_put: _OptionQuote      # sell lower-strike put

    @property
    def net_debit(self) -> float:
        return (
            self.long_call.ask
            - self.short_call.bid
            + self.long_put.ask
            - self.short_put.bid
        )

    @property
    def theoretical_value(self) -> float:
        return self.upper_strike - self.lower_strike

    @property
    def arbitrage_profit(self) -> float:
        return self.theoretical_value - self.net_debit

    @property
    def roi_percent(self) -> float:
        if self.net_debit == 0:
            return 0.0
        return (self.arbitrage_profit / self.net_debit) * 100.0


class BoxSpreadStrategy(Strategy):
    """Box spread strategy using NautilusTrader lifecycle."""

    def __init__(self, config: BoxSpreadConfig, nats_bridge: NatsBridge) -> None:
        super().__init__(config)
        self._nats = nats_bridge
        self._cfg = config
        # chain_cache[symbol][instrument_id.value] → _OptionQuote
        self._chain_cache: dict[str, dict[str, _OptionQuote]] = {
            sym: {} for sym in config.symbols
        }
        # last evaluation timestamp per symbol (for debounce)
        self._last_eval: dict[str, float] = {sym: 0.0 for sym in config.symbols}

    # ------------------------------------------------------------------
    # NT Strategy lifecycle
    # ------------------------------------------------------------------

    def on_start(self) -> None:
        log.info("strategy.starting", symbols=list(self._cfg.symbols))
        for sym in self._cfg.symbols:
            self._subscribe_chain(sym)

    def on_stop(self) -> None:
        log.info("strategy.stopping")
        # Drain any in-flight NATS publishes before the event loop closes.
        asyncio.create_task(self._nats.drain())

    def on_quote_tick(self, tick: QuoteTick) -> None:
        symbol = instrument_id_symbol(tick.instrument_id)
        if symbol not in self._chain_cache:
            return

        # Update chain cache
        key = tick.instrument_id.value
        if key in self._chain_cache[symbol]:
            quote = self._chain_cache[symbol][key]
            quote.bid = float(tick.bid_price)
            quote.ask = float(tick.ask_price)
            quote.ts = time.time()
        else:
            # New instrument seen — parse from InstrumentId
            parsed = self._parse_instrument_id(tick.instrument_id)
            if parsed:
                parsed.bid = float(tick.bid_price)
                parsed.ask = float(tick.ask_price)
                self._chain_cache[symbol][key] = parsed

        # Publish market data to NATS — market-data.tick.{symbol}
        event = quote_tick_to_market_data_event(tick, symbol)
        self._nats.schedule(self._nats.publish_market_data(symbol, event))

        # Publish strategy signal — strategy.signal.{symbol} (mirrors C++ NatsClient)
        mid = (float(tick.bid_price) + float(tick.ask_price)) / 2.0
        _now = time.time()
        signal = pb.StrategySignal(
            symbol=symbol,
            price=mid,
            timestamp=timestamp_pb2.Timestamp(
                seconds=int(_now), nanos=int((_now % 1) * 1_000_000_000)
            ),
        )
        self._nats.schedule(self._nats.publish_strategy_signal(symbol, signal))

        # Debounced evaluation
        now = time.time()
        if now - self._last_eval[symbol] >= self._cfg.eval_debounce_seconds:
            self._last_eval[symbol] = now
            self._evaluate_symbol(symbol)

    def on_order_filled(self, event: OrderFilled) -> None:
        symbol = instrument_id_symbol(event.instrument_id)
        execution = order_filled_to_box_spread_execution(
            event, symbol, 0, 0, ""  # strike/expiry enriched in _execute_box_spread
        )
        self._nats.schedule(
            self._nats.publish_order_fill(str(event.trade_id), execution)
        )

    def on_position_opened(self, event: PositionOpened) -> None:
        symbol = instrument_id_symbol(event.instrument_id)
        proto_pos = position_to_proto(event, symbol)
        self._nats.schedule(self._nats.publish_position_update(symbol, proto_pos))

    def on_position_changed(self, event: PositionChanged) -> None:
        symbol = instrument_id_symbol(event.instrument_id)
        proto_pos = position_to_proto(event, symbol)
        self._nats.schedule(self._nats.publish_position_update(symbol, proto_pos))

    # ------------------------------------------------------------------
    # Box spread detection — mirrors C++ find_box_spreads_in_chain()
    # native/src/strategies/box_spread/box_spread_strategy.cpp:213-248
    # ------------------------------------------------------------------

    def _evaluate_symbol(self, symbol: str) -> None:
        chain = self._chain_cache.get(symbol, {})
        if len(chain) < 4:
            return

        opportunities = self._find_box_spreads(symbol, chain)
        for opp in opportunities:
            if self._is_actionable(opp):
                log.info(
                    "box_spread_opportunity",
                    symbol=symbol,
                    lower=opp.lower_strike,
                    upper=opp.upper_strike,
                    expiry=opp.expiry,
                    profit=round(opp.arbitrage_profit, 4),
                    roi=round(opp.roi_percent, 3),
                )
                decision = pb.StrategyDecision(
                    symbol=symbol,
                    quantity=1,
                    side="BUY",
                    mark=opp.net_debit,
                )
                self._nats.schedule(
                    self._nats.publish_strategy_decision(symbol, decision)
                )
                self._execute_box_spread(opp)
                break  # one trade per evaluation cycle

    def _find_box_spreads(
        self, symbol: str, chain: dict[str, _OptionQuote]
    ) -> list[BoxSpreadOpportunity]:
        """Iterate strike pairs and find valid box spreads.

        For each (lower_strike, upper_strike) pair at the same expiry:
          Buy  lower_strike call  (long_call)
          Sell upper_strike call  (short_call)
          Buy  upper_strike put   (long_put)
          Sell lower_strike put   (short_put)
        Theoretical value = upper_strike - lower_strike (at expiry, risk-free).
        """
        # Group by (expiry, strike, option_type)
        by_expiry: dict[str, dict[float, dict[str, _OptionQuote]]] = {}
        for q in chain.values():
            by_expiry.setdefault(q.expiry, {}).setdefault(q.strike, {})[q.option_type] = q

        opportunities: list[BoxSpreadOpportunity] = []

        for expiry, strikes in by_expiry.items():
            sorted_strikes = sorted(strikes.keys())
            for i, low in enumerate(sorted_strikes):
                for high in sorted_strikes[i + 1 :]:
                    low_legs = strikes[low]
                    high_legs = strikes[high]

                    long_call = low_legs.get("C")
                    short_call = high_legs.get("C")
                    long_put = high_legs.get("P")
                    short_put = low_legs.get("P")

                    if not (long_call and short_call and long_put and short_put):
                        continue

                    # Bid-ask spread quality filter
                    for leg in (long_call, short_call, long_put, short_put):
                        if leg.spread_fraction > self._cfg.max_bid_ask_spread:
                            break
                    else:
                        opp = BoxSpreadOpportunity(
                            symbol=symbol,
                            lower_strike=low,
                            upper_strike=high,
                            expiry=expiry,
                            long_call=long_call,
                            short_call=short_call,
                            long_put=long_put,
                            short_put=short_put,
                        )
                        if opp.arbitrage_profit > 0:
                            opportunities.append(opp)

        return sorted(opportunities, key=lambda o: o.arbitrage_profit, reverse=True)

    def _is_actionable(self, opp: BoxSpreadOpportunity) -> bool:
        """Apply profit and ROI filters before execution."""
        return (
            opp.arbitrage_profit >= self._cfg.min_arbitrage_profit
            and opp.roi_percent >= self._cfg.min_roi_percent
            and opp.net_debit <= self._cfg.max_position_size
        )

    # ------------------------------------------------------------------
    # Order execution — IB BAG combo order
    # ------------------------------------------------------------------

    def _execute_box_spread(self, opp: BoxSpreadOpportunity) -> None:
        """Submit 4-leg box spread as an IB BAG combo order via NT exec client.

        NOTE: NT 1.224.0 IB adapter BAG contract API must be verified against
        actual release before enabling. This is currently a logged stub.
        See: https://nautilustrader.io/docs/nightly/integrations/ib/
        C++ equivalent: IBroker::place_combo_order() in broker_interface.h:221-227
        """
        log.info(
            "execute_box_spread.stub",
            symbol=opp.symbol,
            lower=opp.lower_strike,
            upper=opp.upper_strike,
            expiry=opp.expiry,
            net_debit=round(opp.net_debit, 4),
            note="BAG combo order submission — verify NT 1.224.0 API before enabling",
        )
        # TODO: implement BAG order via NT IB exec client
        # Legs:
        #   BUY  1 opp.long_call.instrument_id   (lower_strike call)
        #   SELL 1 opp.short_call.instrument_id  (upper_strike call)
        #   BUY  1 opp.long_put.instrument_id    (upper_strike put)
        #   SELL 1 opp.short_put.instrument_id   (lower_strike put)

    # ------------------------------------------------------------------
    # Helpers
    # ------------------------------------------------------------------

    def _subscribe_chain(self, symbol: str) -> None:
        """Subscribe to all option quote ticks for the underlying symbol."""
        log.info("subscribing_chain", symbol=symbol)
        # NT IB adapter: subscribe_quote_ticks for each loaded instrument.
        # Instruments must be loaded via IBInstrumentProvider before subscribing.
        # Full chain subscription happens once instruments are resolved.
        # (See instrument_provider.py BoxSpreadInstrumentHelper)

    def _parse_instrument_id(self, instrument_id: InstrumentId) -> _OptionQuote | None:
        """Parse strike, expiry, option_type from NT IB simplified InstrumentId.

        IB simplified format: SPX240412C05100000.CBOE
          SPX     = underlying
          240412  = expiry YYMMDD
          C       = call/put
          05100000= strike * 1000 (zero-padded to 8 digits)
        """
        try:
            sym_val = instrument_id.symbol.value
            # strip venue suffix
            core = sym_val.split(".")[0]
            # find option type character
            for i, ch in enumerate(core):
                if ch in ("C", "P"):
                    underlying = core[:i]
                    if not underlying.startswith(instrument_id_symbol(instrument_id)):
                        return None
                    option_type = ch
                    expiry_str = core[len(underlying) : i]
                    # convert YYMMDD → YYYYMMDD
                    if len(expiry_str) == 6:
                        expiry = "20" + expiry_str
                    else:
                        expiry = expiry_str
                    strike_raw = core[i + 1 :]
                    strike = float(strike_raw) / 1000.0
                    return _OptionQuote(
                        instrument_id=instrument_id,
                        expiry=expiry,
                        strike=strike,
                        option_type=option_type,
                    )
        except Exception:
            log.debug("parse_instrument_id_failed", id=instrument_id.value)
        return None
