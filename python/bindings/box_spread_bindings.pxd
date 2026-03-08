# box_spread_bindings.pxd - Cython declarations for C++ types and functions
# distutils: language = c++

from libcpp.string cimport string
from libcpp.vector cimport vector
from libcpp.memory cimport unique_ptr, shared_ptr
from libcpp.optional cimport optional
from libcpp.map cimport map
cimport numpy as np

# Forward declarations
cdef extern from "types.h" namespace "types":
    cdef enum OptionType:
        Call
        Put
    
    cdef enum OrderAction:
        Buy
        Sell
    
    cdef enum OrderStatus:
        Pending
        Submitted
        Filled
        PartiallyFilled
        Cancelled
        Rejected
        Error
    
    cdef enum TimeInForce:
        Day
        GTC
        IOC
        FOK
    
    cdef cppclass OptionContract:
        string symbol
        string expiry
        double strike
        OptionType type
        string exchange
        string local_symbol
        bint is_valid()
    
    cdef cppclass BoxSpreadLeg:
        OptionContract long_call
        OptionContract short_call
        OptionContract long_put
        OptionContract short_put
        double net_debit
        double theoretical_value
        double arbitrage_profit
        double roi_percent
        double long_call_price
        double short_call_price
        double long_put_price
        double short_put_price
        double long_call_bid_ask_spread
        double short_call_bid_ask_spread
        double long_put_bid_ask_spread
        double short_put_bid_ask_spread
        bint is_valid()
        double get_strike_width()
        int get_days_to_expiry()
    
    cdef cppclass MarketData:
        string symbol
        double bid
        double ask
        double last
        int bid_size
        int ask_size
        int last_size
        double volume
        double high
        double low
        double close
        double open
        optional[double] implied_volatility
        optional[double] delta
        optional[double] gamma
        optional[double] theta
        optional[double] vega
        double get_mid_price()
        double get_spread()
        double get_spread_percent()
    
    cdef cppclass Position:
        OptionContract contract
        int quantity
        double avg_price
        double current_price
        double unrealized_pnl
        bint is_long()
        bint is_short()
        double get_market_value()
        double get_cost_basis()
    
    cdef cppclass Order:
        int order_id
        OptionContract contract
        OrderAction action
        int quantity
        double limit_price
        TimeInForce tif
        OrderStatus status
        int filled_quantity
        double avg_fill_price
        string status_message
        bint is_active()
        bint is_complete()
        double get_total_cost()

cdef extern from "strategies/box_spread/box_spread_strategy.h" namespace "strategy":
    cdef cppclass BoxSpreadStrategy:
        cppclass BoxSpreadOpportunity:
            BoxSpreadLeg spread
            double confidence_score
            double expected_profit
            double risk_adjusted_return
            double liquidity_score
            double execution_probability
            bint is_actionable()
        
        BoxSpreadStrategy(...)  # Will be wrapped with Python-friendly constructor
        void evaluate_opportunities()
        void evaluate_symbol(const string& symbol)
        vector[BoxSpreadOpportunity] find_box_spreads(const string& symbol)
        optional[BoxSpreadOpportunity] evaluate_box_spread(
            const OptionContract& long_call,
            const OptionContract& short_call,
            const OptionContract& long_put,
            const OptionContract& short_put,
            ...  # OptionChain parameter
        )
        bint is_profitable(const BoxSpreadLeg& spread)
        double calculate_roi(const BoxSpreadLeg& spread)
        double calculate_confidence_score(const BoxSpreadLeg& spread, ...)

cdef extern from "strategies/box_spread/box_spread_strategy.h" namespace "strategy":
    cdef cppclass BoxSpreadCalculator "strategy::BoxSpreadCalculator":
        @staticmethod
        double calculate_max_profit(const BoxSpreadLeg& spread)

cdef extern from "risk_calculator.h" namespace "risk":
    cdef cppclass RiskCalculator:
        RiskCalculator(...)  # Will be wrapped with Python-friendly constructor
        # Note: Methods will be exposed through Python wrapper

cdef extern from "config_manager.h" namespace "config":
    cdef cppclass StrategyParams:
        vector[string] symbols
        double min_arbitrage_profit
        double min_roi_percent
        double max_position_size
        int min_days_to_expiry
        int max_days_to_expiry
    
    cdef cppclass RiskConfig:
        double max_total_exposure
        int max_positions
        double max_loss_per_position
        double max_daily_loss



