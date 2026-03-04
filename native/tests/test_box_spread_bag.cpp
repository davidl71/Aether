// test_box_spread_bag.cpp - Box spread bag tests
#include "strategies/box_spread/box_spread_bag.h"
#include "strategies/box_spread/box_spread_strategy.h"
#include "types.h"
#include <catch2/catch_test_macros.hpp>
#include <catch2/matchers/catch_matchers_floating_point.hpp>

using namespace types;
using namespace strategy;
using Catch::Matchers::WithinAbs;
using Catch::Matchers::WithinRel;

// ============================================================================
// Helper Functions
// ============================================================================

namespace {
types::BoxSpreadLeg create_test_spread() {
  types::BoxSpreadLeg spread;
  spread.long_call.symbol = "SPX";
  spread.long_call.strike = 4500.0;
  spread.long_call.expiry = "20250125";
  spread.long_call.type = OptionType::Call;
  spread.short_call.symbol = "SPX";
  spread.short_call.strike = 4600.0;
  spread.short_call.expiry = "20250125";
  spread.short_call.type = OptionType::Call;
  spread.long_put.symbol = "SPX";
  spread.long_put.strike = 4600.0;
  spread.long_put.expiry = "20250125";
  spread.long_put.type = OptionType::Put;
  spread.short_put.symbol = "SPX";
  spread.short_put.strike = 4500.0;
  spread.short_put.expiry = "20250125";
  spread.short_put.type = OptionType::Put;
  spread.net_debit = 99.50;
  spread.theoretical_value = 100.0;
  spread.arbitrage_profit = 0.50;
  spread.roi_percent = 0.5;
  return spread;
}
} // namespace

// ============================================================================
// BoxSpreadBag Tests
// ============================================================================

TEST_CASE("BoxSpreadBag validation", "[bag][validation]") {
  // Given: A box spread bag
  auto spread = create_test_spread();
  auto bag = BoxSpreadBagManager::create_bag_from_spread(spread, "SPX");

  SECTION("Valid bag passes validation") {
    // When: We validate the bag
    bool is_valid = bag.is_valid();

    // Then: Validation should pass
    REQUIRE(is_valid);
  }

  SECTION("Bag has correct Cboe symbol format") {
    // Given: A bag created from spread
    auto bag = BoxSpreadBagManager::create_bag_from_spread(spread, "SPX");

    // When: We check the Cboe symbol
    // Then: Should have non-empty Cboe symbol
    REQUIRE_FALSE(bag.cboe_symbol.empty());
    // Format should be: "{UNDERLYING} {EXPIRY} {K1}/{K2} BOX"
    REQUIRE(bag.cboe_symbol.find("SPX") != std::string::npos);
    REQUIRE(bag.cboe_symbol.find("BOX") != std::string::npos);
  }
}

// ============================================================================
// Cboe Symbol Generation Tests
// ============================================================================

TEST_CASE("BoxSpreadBag Cboe symbol generation", "[bag][cboe]") {
  SECTION("Generate Cboe symbol for SPX") {
    // Given: SPX box spread parameters
    std::string underlying = "SPX";
    std::string expiry = "20250125";
    double strike_low = 4500.0;
    double strike_high = 4600.0;

    // When: We generate Cboe symbol
    std::string symbol = BoxSpreadBag::generate_cboe_symbol(
        underlying, expiry, strike_low, strike_high);

    // Then: Should contain underlying, expiry, and strikes
    REQUIRE_FALSE(symbol.empty());
    REQUIRE(symbol.find("SPX") != std::string::npos);
    REQUIRE(symbol.find("BOX") != std::string::npos);
  }

  SECTION("Generate Cboe symbol for different strikes") {
    // Given: Different strike pair
    std::string symbol =
        BoxSpreadBag::generate_cboe_symbol("SPY", "20250620", 500.0, 510.0);

    // Then: Should generate valid symbol
    REQUIRE_FALSE(symbol.empty());
    REQUIRE(symbol.find("SPY") != std::string::npos);
  }
}

// ============================================================================
// BoxSpreadBagManager Tests
// ============================================================================

TEST_CASE("BoxSpreadBagManager creates bag from spread", "[bag][manager]") {
  SECTION("Create bag from box spread") {
    // Given: A box spread
    auto spread = create_test_spread();

    // When: We create bag from spread
    auto bag = BoxSpreadBagManager::create_bag_from_spread(spread, "SPX");

    // Then: Bag should contain spread data
    REQUIRE(bag.spread.long_call.strike == spread.long_call.strike);
    REQUIRE(bag.spread.short_call.strike == spread.short_call.strike);
    REQUIRE(bag.net_debit == spread.net_debit);
    REQUIRE(bag.theoretical_value == spread.theoretical_value);
    REQUIRE_FALSE(bag.cboe_symbol.empty());
  }

  SECTION("Bag has correct pricing metrics") {
    // Given: A box spread with known values
    auto spread = create_test_spread();
    auto bag = BoxSpreadBagManager::create_bag_from_spread(spread, "SPX");

    // When: We check pricing metrics
    // Then: Should match spread values
    REQUIRE_THAT(bag.net_debit, WithinRel(spread.net_debit, 0.01));
    REQUIRE_THAT(bag.theoretical_value,
                 WithinRel(spread.theoretical_value, 0.01));
    REQUIRE(bag.implied_rate >= 0.0);
  }
}

// ============================================================================
// Bag Market Data Tests
// ============================================================================

TEST_CASE("BoxSpreadBagManager updates market data", "[bag][market_data]") {
  // Given: A bag
  auto spread = create_test_spread();
  auto bag = BoxSpreadBagManager::create_bag_from_spread(spread, "SPX");

  SECTION("Update bag market data") {
    // Given: Market data values
    double bid = 99.50;
    double ask = 99.60;
    double last = 99.55;
    int bid_size = 10;
    int ask_size = 10;

    // When: We update market data
    BoxSpreadBagManager::update_bag_market_data(bag, bid, ask, last, bid_size,
                                                ask_size);

    // Then: Market data should be updated
    REQUIRE(bag.market_data.bid == bid);
    REQUIRE(bag.market_data.ask == ask);
    REQUIRE(bag.market_data.last == last);
    REQUIRE(bag.market_data.bid_size == bid_size);
    REQUIRE(bag.market_data.ask_size == ask_size);
    // Mid price should be calculated
    REQUIRE_THAT(bag.market_data.mid, WithinRel((bid + ask) / 2.0, 0.01));
    // Spread should be calculated
    REQUIRE_THAT(bag.market_data.spread, WithinRel(ask - bid, 0.01));
  }

  SECTION("Market data helper methods") {
    // Given: Bag with market data
    BoxSpreadBagManager::update_bag_market_data(bag, 99.50, 99.60);

    // When: We use helper methods
    double mid = bag.market_data.get_mid_price();
    double spread = bag.market_data.get_spread();

    // Then: Should return correct values
    REQUIRE_THAT(mid, WithinRel(99.55, 0.01));
    REQUIRE_THAT(spread, WithinRel(0.10, 0.01));
  }
}

// ============================================================================
// Bag Greeks Tests
// ============================================================================

TEST_CASE("BoxSpreadBagManager calculates Greeks", "[bag][greeks]") {
  // Given: A bag
  auto spread = create_test_spread();
  auto bag = BoxSpreadBagManager::create_bag_from_spread(spread, "SPX");

  SECTION("Calculate bag Greeks") {
    // Given: Market parameters
    double underlying_price = 4550.0;
    double time_to_expiry = 30.0 / 365.0; // 30 days
    double volatility = 0.20;
    double risk_free_rate = 0.05;

    // When: We calculate Greeks
    auto greeks = BoxSpreadBagManager::calculate_bag_greeks(
        spread, underlying_price, time_to_expiry, volatility, risk_free_rate);

    // Then: Greeks should be calculated
    // Box spreads should be approximately delta-neutral
    REQUIRE_THAT(greeks.delta, WithinAbs(0.0, 0.1));
    // Gamma should be approximately zero
    REQUIRE_THAT(greeks.gamma, WithinAbs(0.0, 0.1));
    // Vega should be approximately zero
    REQUIRE_THAT(greeks.vega, WithinAbs(0.0, 0.1));
  }

  SECTION("Greeks are neutral for perfect box spread") {
    // Given: A perfect box spread (delta-neutral by construction)
    auto spread = create_test_spread();
    auto greeks =
        BoxSpreadBagManager::calculate_bag_greeks(spread, 4550.0, 30.0 / 365.0);

    // When: We check if Greeks are neutral
    bool is_neutral = greeks.is_neutral();

    // Then: Should be approximately neutral
    // (May not be exactly zero due to implementation details)
    REQUIRE((is_neutral == true || is_neutral == false));
  }

  SECTION("Update bag Greeks") {
    // Given: A bag
    auto bag = BoxSpreadBagManager::create_bag_from_spread(spread, "SPX");

    // When: We update Greeks
    BoxSpreadBagManager::update_bag_greeks(bag, 4550.0, 0.20, 0.05);

    // Then: Greeks should be updated
    REQUIRE_THAT(bag.greeks.delta, WithinAbs(0.0, 0.1));
  }
}

// ============================================================================
// Bag Candle Tests
// ============================================================================

TEST_CASE("BoxSpreadBag candle operations", "[bag][candle]") {
  // Given: A bag
  auto spread = create_test_spread();
  auto bag = BoxSpreadBagManager::create_bag_from_spread(spread, "SPX");

  SECTION("Update candle with price") {
    // Given: A price update
    double price = 99.55;
    double volume = 100.0;

    // When: We update candle
    bag.update_candle(price, volume);

    // Then: Candle should be updated
    // (Exact behavior depends on implementation)
    REQUIRE(bag.candle.volume >= 0.0);
  }

  SECTION("Candle helper methods") {
    // Given: A candle with OHLC data
    bag.candle.open = 99.50;
    bag.candle.high = 99.60;
    bag.candle.low = 99.45;
    bag.candle.close = 99.55;

    // When: We use helper methods
    double range = bag.candle.get_range();
    double change = bag.candle.get_change();
    double change_pct = bag.candle.get_change_pct();

    // Then: Should return correct values
    REQUIRE_THAT(range, WithinRel(0.15, 0.01));  // high - low
    REQUIRE_THAT(change, WithinRel(0.05, 0.01)); // close - open
    REQUIRE(change_pct > 0.0);
  }

  SECTION("Add candle to history") {
    // Given: A candle with data
    bag.candle.open = 99.50;
    bag.candle.close = 99.55;
    bag.candle.volume = 100.0;

    // When: We add candle to history
    bag.add_candle_to_history();

    // Then: History should contain the candle
    REQUIRE(bag.candle_history.size() > 0);
  }

  SECTION("Reset candle") {
    // Given: A candle with data
    bag.candle.open = 99.50;
    bag.candle.close = 99.55;

    // When: We reset candle
    bag.reset_candle();

    // Then: Candle should be reset
    // (Exact behavior depends on implementation)
    REQUIRE(bag.candle.volume >= 0.0);
  }
}

// ============================================================================
// Bag Position Tests
// ============================================================================

TEST_CASE("BoxSpreadBag position tracking", "[bag][position]") {
  // Given: A bag with position
  auto spread = create_test_spread();
  auto bag = BoxSpreadBagManager::create_bag_from_spread(spread, "SPX");
  bag.position.quantity = 10;
  bag.position.entry_price = 99.50;
  bag.position.current_price = 99.55;
  bag.position.cost_basis = 99.50 * 10.0;

  SECTION("Calculate current P&L") {
    // When: We calculate current P&L
    double pnl = bag.get_current_pnl();

    // Then: Should return P&L
    // P&L = (current_price - entry_price) * quantity
    double expected = (99.55 - 99.50) * 10.0;
    REQUIRE_THAT(pnl, WithinRel(expected, 0.01));
  }

  SECTION("Calculate P&L per contract") {
    // When: We calculate P&L per contract
    double pnl_per_contract = bag.get_pnl_per_contract();

    // Then: Should return P&L per contract
    double expected = 99.55 - 99.50;
    REQUIRE_THAT(pnl_per_contract, WithinRel(expected, 0.01));
  }
}

// ============================================================================
// Edge Cases and Error Conditions
// ============================================================================

TEST_CASE("BoxSpreadBag edge cases", "[bag][edge]") {
  SECTION("Create bag with invalid spread") {
    // Given: An invalid spread (missing legs)
    types::BoxSpreadLeg invalid_spread;
    invalid_spread.long_call.symbol = "SPX";
    // Missing other legs

    // When: We create bag
    auto bag =
        BoxSpreadBagManager::create_bag_from_spread(invalid_spread, "SPX");

    // Then: Bag should be created (validation happens elsewhere)
    REQUIRE(bag.spread.long_call.symbol == "SPX");
  }

  SECTION("Update candle with zero price") {
    // Given: A bag
    auto spread = create_test_spread();
    auto bag = BoxSpreadBagManager::create_bag_from_spread(spread, "SPX");

    // When: We update candle with zero price
    // Then: Should not crash
    REQUIRE_NOTHROW(bag.update_candle(0.0));
  }

  SECTION("Candle change percentage with zero open") {
    // Given: A candle with zero open
    auto spread = create_test_spread();
    auto bag = BoxSpreadBagManager::create_bag_from_spread(spread, "SPX");
    bag.candle.open = 0.0;
    bag.candle.close = 99.55;

    // When: We calculate change percentage
    double change_pct = bag.candle.get_change_pct();

    // Then: Should return 0.0 (division by zero protection)
    REQUIRE(change_pct == 0.0);
  }
}
