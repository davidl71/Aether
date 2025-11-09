// test_order_manager.cpp - Order manager tests
#include <catch2/catch_test_macros.hpp>
#include <catch2/matchers/catch_matchers_floating_point.hpp>
#include "order_manager.h"
#include "tws_client.h"
#include "config_manager.h"

using namespace order;

TEST_CASE("OrderValidator validates contracts", "[order]") {
    types::OptionContract contract;
    contract.symbol = "SPY";
    contract.strike = 500.0;
    contract.expiry = "20250620";
    contract.type = types::OptionType::Call;
    contract.exchange = "SMART";

    std::string error;

    SECTION("Valid contract passes validation") {
        REQUIRE(OrderValidator::validate_contract(contract, error));
    }

    SECTION("Empty symbol fails validation") {
        contract.symbol = "";
        REQUIRE_FALSE(OrderValidator::validate_contract(contract, error));
        REQUIRE_FALSE(error.empty());
    }

    SECTION("Zero strike fails validation") {
        contract.strike = 0.0;
        REQUIRE_FALSE(OrderValidator::validate_contract(contract, error));
    }

    SECTION("Empty expiry fails validation") {
        contract.expiry = "";
        REQUIRE_FALSE(OrderValidator::validate_contract(contract, error));
    }
}

TEST_CASE("OrderValidator validates quantity", "[order]") {
    std::string error;

    SECTION("Positive quantity is valid") {
        REQUIRE(OrderValidator::validate_quantity(10, error));
    }

    SECTION("Zero quantity fails validation") {
        REQUIRE_FALSE(OrderValidator::validate_quantity(0, error));
    }

    SECTION("Negative quantity fails validation") {
        REQUIRE_FALSE(OrderValidator::validate_quantity(-5, error));
    }

    SECTION("Excessive quantity fails validation") {
        REQUIRE_FALSE(OrderValidator::validate_quantity(10000, error));
    }
}

TEST_CASE("OrderValidator validates price", "[order]") {
    std::string error;

    SECTION("Positive price is valid") {
        REQUIRE(OrderValidator::validate_price(5.50, error));
    }

    SECTION("Zero price is valid (market order)") {
        REQUIRE(OrderValidator::validate_price(0.0, error));
    }

    SECTION("Negative price fails validation") {
        REQUIRE_FALSE(OrderValidator::validate_price(-1.0, error));
    }
}

TEST_CASE("OrderBuilder builds orders", "[order]") {
    types::OptionContract contract;
    contract.symbol = "SPY";
    contract.strike = 500.0;
    contract.expiry = "20250620";
    contract.type = types::OptionType::Call;
    contract.exchange = "SMART";

    SECTION("Build a complete order") {
        auto order = OrderBuilder()
            .contract(contract)
            .action(types::OrderAction::Buy)
            .quantity(10)
            .limit_price(5.50)
            .time_in_force(types::TimeInForce::Day)
            .build();

        REQUIRE(order.contract.symbol == "SPY");
        REQUIRE(order.quantity == 10);
        REQUIRE_THAT(order.limit_price, Catch::Matchers::WithinRel(5.50, 0.001));
        REQUIRE(order.action == types::OrderAction::Buy);
    }
}

TEST_CASE("OrderManager in dry-run mode", "[order]") {
    config::TWSConfig tws_config;
    tws::TWSClient client(tws_config);

    OrderManager manager(&client, true);  // Dry-run mode

    REQUIRE(manager.is_dry_run());

    types::OptionContract contract;
    contract.symbol = "SPY";
    contract.strike = 500.0;
    contract.expiry = "20250620";
    contract.type = types::OptionType::Call;
    contract.exchange = "SMART";

    SECTION("Place order in dry-run mode") {
        auto result = manager.place_order(
            contract,
            types::OrderAction::Buy,
            10,
            5.50
        );

        REQUIRE(result.success);
        REQUIRE_FALSE(result.order_ids.empty());
    }

    SECTION("Place box spread in dry-run mode") {
        types::BoxSpreadLeg spread;
        spread.long_call = contract;
        spread.short_call = contract;
        spread.short_call.strike = 510.0;
        spread.long_put = contract;
        spread.long_put.type = types::OptionType::Put;
        spread.long_put.strike = 510.0;
        spread.short_put = contract;
        spread.short_put.type = types::OptionType::Put;

        spread.long_call_price = 2.50;
        spread.short_call_price = 1.00;
        spread.long_put_price = 2.00;
        spread.short_put_price = 0.75;
        spread.net_debit = 2.75;

        auto result = manager.place_box_spread(spread);

        REQUIRE(result.success);
        REQUIRE(result.order_ids.size() == 4);  // 4 legs
    }

    SECTION("Cancel order in dry-run mode") {
        REQUIRE(manager.cancel_order(123));
    }
}

TEST_CASE("Order cost calculations", "[order]") {
    types::Order order;
    order.quantity = 10;
    order.limit_price = 5.50;
    order.filled_quantity = 0;

    SECTION("Calculate order cost") {
        double cost = calculate_order_cost(order, 0.65);

        // Cost = (10 contracts * $5.50 * 100) + (10 * $0.65)
        // = $5500 + $6.50 = $5506.50
        REQUIRE_THAT(cost, Catch::Matchers::WithinRel(5506.50, 0.01));
    }

    SECTION("Calculate cost with different commission") {
        double cost = calculate_order_cost(order, 1.00);

        // Cost = $5500 + $10 = $5510
        REQUIRE_THAT(cost, Catch::Matchers::WithinRel(5510.00, 0.01));
    }
}

TEST_CASE("Slippage estimation", "[order]") {
    types::OptionContract contract;
    contract.symbol = "SPY";
    contract.strike = 500.0;

    SECTION("Estimate slippage from spread") {
        double slippage = estimate_slippage(
            contract,
            types::OrderAction::Buy,
            10,
            5.40,  // Bid
            5.60   // Ask
        );

        // Spread = $0.20, estimated slippage = $0.10 (half spread)
        REQUIRE(slippage > 0);
        REQUIRE_THAT(slippage, Catch::Matchers::WithinRel(0.10, 0.01));
    }
}

TEST_CASE("MultiLegOrder completion tracking", "[order]") {
    MultiLegOrder multi_leg;
    multi_leg.strategy_id = "test_strategy";

    // Create 4 legs
    for (int i = 0; i < 4; ++i) {
        types::Order order;
        order.order_id = i + 1;
        order.status = types::OrderStatus::Submitted;
        multi_leg.legs.push_back(order);
    }

    SECTION("Not complete initially") {
        multi_leg.legs_filled = 0;
        REQUIRE_FALSE(multi_leg.is_complete());
        REQUIRE_FALSE(multi_leg.is_partially_filled());
    }

    SECTION("Partially filled") {
        multi_leg.legs_filled = 2;
        REQUIRE_FALSE(multi_leg.is_complete());
        REQUIRE(multi_leg.is_partially_filled());
    }

    SECTION("Completely filled") {
        multi_leg.legs_filled = 4;
        REQUIRE(multi_leg.is_complete());
        REQUIRE_FALSE(multi_leg.is_partially_filled());
    }
}

TEST_CASE("Order formatting", "[order]") {
    types::Order order;
    order.order_id = 123;
    order.action = types::OrderAction::Buy;
    order.quantity = 10;
    order.limit_price = 5.50;

    order.contract.symbol = "SPY";
    order.contract.strike = 500.0;
    order.contract.expiry = "20250620";
    order.contract.type = types::OptionType::Call;

    SECTION("Format order string") {
        std::string formatted = format_order(order);

        REQUIRE_FALSE(formatted.empty());
        REQUIRE(formatted.find("123") != std::string::npos);
        REQUIRE(formatted.find("BUY") != std::string::npos);
        REQUIRE(formatted.find("10") != std::string::npos);
        REQUIRE(formatted.find("SPY") != std::string::npos);
    }
}

TEST_CASE("Order statistics tracking", "[order]") {
    config::TWSConfig tws_config;
    tws::TWSClient client(tws_config);
    OrderManager manager(&client, true);

    types::OptionContract contract;
    contract.symbol = "SPY";
    contract.strike = 500.0;
    contract.expiry = "20250620";
    contract.type = types::OptionType::Call;
    contract.exchange = "SMART";

    SECTION("Statistics updated after orders") {
        // Place some orders
        manager.place_order(contract, types::OrderAction::Buy, 10, 5.50);
        manager.place_order(contract, types::OrderAction::Sell, 5, 5.60);

        auto stats = manager.get_statistics();

        REQUIRE(stats.total_orders_placed == 2);
    }

    SECTION("Statistics can be reset") {
        manager.place_order(contract, types::OrderAction::Buy, 10, 5.50);

        auto stats_before = manager.get_statistics();
        REQUIRE(stats_before.total_orders_placed > 0);

        manager.reset_statistics();

        auto stats_after = manager.get_statistics();
        REQUIRE(stats_after.total_orders_placed == 0);
    }
}
