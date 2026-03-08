// box_spread_pybind.cpp - pybind11 bindings for box spread C++ (CMake-built, no TWS)
#include <pybind11/pybind11.h>
#include <pybind11/stl.h>
#include "types.h"
#include "risk_calculator.h"
#include "tws_conversions.h"
#include "strategies/box_spread/box_spread_strategy.h"
#include <string>

namespace py = pybind11;

PYBIND11_MODULE(box_spread_bindings, m)
{
  m.doc() = "Box spread C++ calculations (pybind11); same API as Cython bindings.";

  // ---------------------------------------------------------------------------
  // Enums (match Cython: OptionType, OptionTypePy alias, etc.)
  // ---------------------------------------------------------------------------
  py::enum_<types::OptionType>(m, "OptionType")
    .value("Call", types::OptionType::Call)
    .value("Put", types::OptionType::Put)
    .export_values();
  m.attr("OptionTypePy") = m.attr("OptionType");

  py::enum_<types::OrderAction>(m, "OrderActionPy")
    .value("Buy", types::OrderAction::Buy)
    .value("Sell", types::OrderAction::Sell)
    .export_values();

  py::enum_<types::OrderStatus>(m, "OrderStatusPy")
    .value("Pending", types::OrderStatus::Pending)
    .value("Submitted", types::OrderStatus::Submitted)
    .value("Filled", types::OrderStatus::Filled)
    .value("PartiallyFilled", types::OrderStatus::PartiallyFilled)
    .value("Cancelled", types::OrderStatus::Cancelled)
    .value("Rejected", types::OrderStatus::Rejected)
    .value("Error", types::OrderStatus::Error)
    .export_values();

  py::enum_<types::TimeInForce>(m, "TimeInForcePy")
    .value("Day", types::TimeInForce::Day)
    .value("GTC", types::TimeInForce::GTC)
    .value("IOC", types::TimeInForce::IOC)
    .value("FOK", types::TimeInForce::FOK)
    .export_values();

  py::enum_<types::OptionStyle>(m, "OptionStyle")
    .value("European", types::OptionStyle::European)
    .value("American", types::OptionStyle::American)
    .export_values();

  // ---------------------------------------------------------------------------
  // OptionContract (exposed as PyOptionContract for API compat)
  // ---------------------------------------------------------------------------
  py::class_<types::OptionContract>(m, "PyOptionContract")
    .def(py::init<>())
    .def(py::init([](const std::string& symbol, const std::string& expiry, double strike,
                     int option_type, const std::string& exchange, const std::string& local_symbol) {
           types::OptionContract c;
           c.symbol = symbol;
           c.expiry = expiry;
           c.strike = strike;
           c.type = static_cast<types::OptionType>(option_type);
           c.style = types::OptionStyle::American;
           c.exchange = exchange.empty() ? "SMART" : exchange;
           c.local_symbol = local_symbol;
           return c;
         }),
         py::arg("symbol"), py::arg("expiry"), py::arg("strike"), py::arg("option_type"),
         py::arg("exchange") = "SMART", py::arg("local_symbol") = "")
    .def_readwrite("symbol", &types::OptionContract::symbol)
    .def_readwrite("expiry", &types::OptionContract::expiry)
    .def_readwrite("strike", &types::OptionContract::strike)
    .def_property("type",
                  [](const types::OptionContract& c) { return static_cast<int>(c.type); },
                  [](types::OptionContract& c, int v) { c.type = static_cast<types::OptionType>(v); })
    .def_readwrite("exchange", &types::OptionContract::exchange)
    .def_readwrite("local_symbol", &types::OptionContract::local_symbol)
    .def("is_valid", &types::OptionContract::is_valid)
    .def("__repr__", [](const types::OptionContract& c) { return c.to_string(); });

  // ---------------------------------------------------------------------------
  // BoxSpreadLeg (exposed as PyBoxSpreadLeg)
  // ---------------------------------------------------------------------------
  py::class_<types::BoxSpreadLeg>(m, "PyBoxSpreadLeg")
    .def(py::init<>())
    .def(py::init([](const types::OptionContract& long_call, const types::OptionContract& short_call,
                     const types::OptionContract& long_put, const types::OptionContract& short_put) {
           types::BoxSpreadLeg leg;
           leg.long_call = long_call;
           leg.short_call = short_call;
           leg.long_put = long_put;
           leg.short_put = short_put;
           return leg;
         }),
         py::arg("long_call"), py::arg("short_call"), py::arg("long_put"), py::arg("short_put"))
    .def_readwrite("net_debit", &types::BoxSpreadLeg::net_debit)
    .def_readwrite("theoretical_value", &types::BoxSpreadLeg::theoretical_value)
    .def_readwrite("arbitrage_profit", &types::BoxSpreadLeg::arbitrage_profit)
    .def_readwrite("roi_percent", &types::BoxSpreadLeg::roi_percent)
    .def("get_strike_width", &types::BoxSpreadLeg::get_strike_width)
    .def("get_days_to_expiry", &types::BoxSpreadLeg::get_days_to_expiry)
    .def("is_valid", &types::BoxSpreadLeg::is_valid)
    .def("__repr__", [](const types::BoxSpreadLeg& leg) {
      return "BoxSpreadLeg(net_debit=" + std::to_string(leg.net_debit) +
             ", profit=" + std::to_string(leg.arbitrage_profit) +
             ", roi=" + std::to_string(leg.roi_percent) + "%)";
    });

  // ---------------------------------------------------------------------------
  // MarketData (exposed as PyMarketData)
  // ---------------------------------------------------------------------------
  py::class_<types::MarketData>(m, "PyMarketData")
    .def(py::init<>())
    .def_readwrite("symbol", &types::MarketData::symbol)
    .def_readwrite("bid", &types::MarketData::bid)
    .def_readwrite("ask", &types::MarketData::ask)
    .def_readwrite("last", &types::MarketData::last)
    .def("get_mid_price", &types::MarketData::get_mid_price)
    .def("get_spread", &types::MarketData::get_spread)
    .def("get_spread_percent", &types::MarketData::get_spread_percent);

  // ---------------------------------------------------------------------------
  // BoxSpreadCalculator static methods
  // ---------------------------------------------------------------------------
  py::class_<strategy::BoxSpreadCalculator>(m, "BoxSpreadCalculator")
    .def_static("calculate_max_profit", &strategy::BoxSpreadCalculator::calculate_max_profit)
    .def_static("calculate_roi", &strategy::BoxSpreadCalculator::calculate_roi)
    .def_static("calculate_implied_interest_rate",
                &strategy::BoxSpreadCalculator::calculate_implied_interest_rate)
    .def_static("calculate_theoretical_value",
                &strategy::BoxSpreadCalculator::calculate_theoretical_value)
    .def_static("calculate_net_debit", &strategy::BoxSpreadCalculator::calculate_net_debit);

  // ---------------------------------------------------------------------------
  // Standalone functions (same names as Cython for drop-in replacement)
  // ---------------------------------------------------------------------------
  m.def("calculate_arbitrage_profit",
        [](const types::BoxSpreadLeg& spread) {
          return strategy::BoxSpreadCalculator::calculate_max_profit(spread);
        },
        py::arg("spread"),
        "Compute max profit for a box spread (BoxSpreadCalculator::calculate_max_profit).");

  m.def("calculate_roi",
        [](const types::BoxSpreadLeg& spread) {
          return strategy::BoxSpreadCalculator::calculate_roi(spread);
        },
        py::arg("spread"),
        "Calculate ROI for a box spread.");

  m.def("calculate_implied_interest_rate",
        [](const types::BoxSpreadLeg& spread) {
          return strategy::BoxSpreadCalculator::calculate_implied_interest_rate(spread);
        },
        py::arg("spread"),
        "Calculate implied annual interest rate (%) for the box spread.");

  m.def("validate_box_spread",
        [](const types::BoxSpreadLeg& spread) {
          return strategy::BoxSpreadValidator::validate_structure(spread);
        },
        py::arg("spread"),
        "Validate box spread structure.");

  // ---------------------------------------------------------------------------
  // Statistical helpers (risk_calculator.h free functions; single source of truth)
  // ---------------------------------------------------------------------------
  m.def("calculate_mean",
        &risk::calculate_mean,
        py::arg("values"),
        "Mean of values (C++ implementation).");
  m.def("calculate_percentile",
        &risk::calculate_percentile,
        py::arg("values"),
        py::arg("percentile"),
        "Percentile (0..1) of sorted values (C++ implementation).");
  m.def("calculate_correlation",
        &risk::calculate_correlation,
        py::arg("x"),
        py::arg("y"),
        "Pearson correlation of x and y (C++ implementation).");

  // ---------------------------------------------------------------------------
  // DTE (days to expiry) - trading-day aware
  // ---------------------------------------------------------------------------
  m.def("calculate_dte",
        &tws::calculate_dte,
        py::arg("expiry"),
        "Days to expiry (trading days) from YYYYMMDD string. Uses MarketHours.");
}
