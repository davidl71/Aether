// test_option_chain.cpp - Option chain tests
#include <catch2/catch_test_macros.hpp>
#include <catch2/matchers/catch_matchers_floating_point.hpp>
#include "option_chain.h"
#include "types.h"
#include <chrono>

using namespace option_chain;
using namespace types;
using Catch::Matchers::WithinRel;
using Catch::Matchers::WithinAbs;

// ============================================================================
// Helper Functions
// ============================================================================

namespace {
  OptionChainEntry create_test_entry(
      const std::string& symbol,
      const std::string& expiry,
      double strike,
      OptionType type,
      double bid = 1.0,
      double ask = 1.1,
      int volume = 100,
      int open_interest = 500) {
    OptionChainEntry entry;
    entry.contract.symbol = symbol;
    entry.contract.expiry = expiry;
    entry.contract.strike = strike;
    entry.contract.type = type;
    entry.market_data.bid = bid;
    entry.market_data.ask = ask;
    entry.market_data.last = (bid + ask) / 2.0;
    entry.volume = volume;
    entry.open_interest = open_interest;
    entry.liquidity_score = 75.0;
    return entry;
  }
} // namespace

// ============================================================================
// OptionChainEntry Tests
// ============================================================================

TEST_CASE("OptionChainEntry validation", "[option_chain][entry]") {
  // Given: A valid option chain entry
  auto entry = create_test_entry("SPY", "20250620", 500.0, OptionType::Call);

  SECTION("Valid entry passes validation") {
    // When: We validate the entry
    bool is_valid = entry.is_valid();

    // Then: Validation should pass
    REQUIRE(is_valid);
  }

  SECTION("Entry with zero bid/ask fails validation") {
    // Given: Entry with zero bid
    entry.market_data.bid = 0.0;
    entry.market_data.ask = 0.0;

    // When: We validate the entry
    bool is_valid = entry.is_valid();

    // Then: Validation should fail (no market data)
    REQUIRE_FALSE(is_valid);
  }

  SECTION("Entry meets liquidity requirements") {
    // Given: Entry with sufficient volume and open interest
    entry.volume = 200;
    entry.open_interest = 1000;

    // When: We check liquidity requirements
    bool meets_requirements = entry.meets_liquidity_requirements(100, 500);

    // Then: Should meet requirements
    REQUIRE(meets_requirements);
  }

  SECTION("Entry fails liquidity requirements") {
    // Given: Entry with insufficient volume
    entry.volume = 50;
    entry.open_interest = 200;

    // When: We check liquidity requirements
    bool meets_requirements = entry.meets_liquidity_requirements(100, 500);

    // Then: Should fail requirements
    REQUIRE_FALSE(meets_requirements);
  }
}

// ============================================================================
// StrikeChain Tests
// ============================================================================

TEST_CASE("StrikeChain operations", "[option_chain][strike]") {
  // Given: A strike chain with call and put
  StrikeChain chain;
  chain.strike = 500.0;
  chain.call = create_test_entry("SPY", "20250620", 500.0, OptionType::Call);
  chain.put = create_test_entry("SPY", "20250620", 500.0, OptionType::Put);

  SECTION("StrikeChain has both call and put") {
    // When: We check if both exist
    bool has_both = chain.has_both();

    // Then: Should return true
    REQUIRE(has_both);
  }

  SECTION("StrikeChain with missing call") {
    // Given: Strike chain without call
    chain.call = std::nullopt;

    // When: We check if both exist
    bool has_both = chain.has_both();

    // Then: Should return false
    REQUIRE_FALSE(has_both);
  }

  SECTION("StrikeChain IV skew calculation") {
    // Given: Strike chain with different IVs for call and put
    chain.call->market_data.implied_volatility = 0.20;
    chain.put->market_data.implied_volatility = 0.22;

    // When: We calculate IV skew
    double skew = chain.get_iv_skew();

    // Then: Skew should equal put IV - call IV = 0.02
    REQUIRE_THAT(skew, WithinAbs(0.02, 0.001));
  }
}

// ============================================================================
// ExpiryChain Tests
// ============================================================================

TEST_CASE("ExpiryChain strike management", "[option_chain][expiry]") {
  // Given: An expiry chain for SPY
  ExpiryChain chain("SPY", "20250620");

  SECTION("Add options to chain") {
    // Given: Multiple option entries
    auto call_500 = create_test_entry("SPY", "20250620", 500.0, OptionType::Call);
    auto put_500 = create_test_entry("SPY", "20250620", 500.0, OptionType::Put);
    auto call_510 = create_test_entry("SPY", "20250620", 510.0, OptionType::Call);
    auto put_510 = create_test_entry("SPY", "20250620", 510.0, OptionType::Put);

    // When: We add options to the chain
    chain.add_option(call_500);
    chain.add_option(put_500);
    chain.add_option(call_510);
    chain.add_option(put_510);

    // Then: Should have 2 strikes
    auto strikes = chain.get_strikes();
    REQUIRE(strikes.size() == 2);
    REQUIRE(strikes[0] == 500.0);
    REQUIRE(strikes[1] == 510.0);
  }

  SECTION("Get strike chain") {
    // Given: Chain with options at strike 500
    auto call_500 = create_test_entry("SPY", "20250620", 500.0, OptionType::Call);
    auto put_500 = create_test_entry("SPY", "20250620", 500.0, OptionType::Put);
    chain.add_option(call_500);
    chain.add_option(put_500);

    // When: We get strike chain for 500
    auto strike_chain = chain.get_strike_chain(500.0);

    // Then: Should return valid strike chain
    REQUIRE(strike_chain.has_value());
    REQUIRE(strike_chain->strike == 500.0);
    REQUIRE(strike_chain->has_both());
  }

  SECTION("Get option by strike and type") {
    // Given: Chain with call at strike 500
    auto call_500 = create_test_entry("SPY", "20250620", 500.0, OptionType::Call);
    chain.add_option(call_500);

    // When: We get call option at strike 500
    auto option = chain.get_option(500.0, OptionType::Call);

    // Then: Should return the call option
    REQUIRE(option.has_value());
    REQUIRE(option->contract.strike == 500.0);
    REQUIRE(option->contract.type == OptionType::Call);
  }

  SECTION("Find strikes in range") {
    // Given: Chain with strikes at 500, 510, 520, 530
    for (double strike = 500.0; strike <= 530.0; strike += 10.0) {
      chain.add_option(create_test_entry("SPY", "20250620", strike, OptionType::Call));
    }

    // When: We find strikes in range 505-525
    auto strikes = chain.get_strikes_in_range(505.0, 525.0);

    // Then: Should return 510, 520
    REQUIRE(strikes.size() == 2);
    REQUIRE(strikes[0] == 510.0);
    REQUIRE(strikes[1] == 520.0);
  }

  SECTION("Find ATM strike") {
    // Given: Chain with strikes at 490, 500, 510, 520
    for (double strike = 490.0; strike <= 520.0; strike += 10.0) {
      chain.add_option(create_test_entry("SPY", "20250620", strike, OptionType::Call));
    }

    // When: We find ATM strike for underlying price 505
    auto atm_strike = chain.find_atm_strike(505.0);

    // Then: Should return 500 (closest strike below)
    REQUIRE(atm_strike.has_value());
    REQUIRE(atm_strike.value() == 500.0);
  }

  SECTION("Filter by liquidity") {
    // Given: Chain with options of varying liquidity
    auto liquid = create_test_entry("SPY", "20250620", 500.0, OptionType::Call, 1.0, 1.1, 200, 1000);
    auto illiquid = create_test_entry("SPY", "20250620", 510.0, OptionType::Call, 1.0, 1.1, 50, 200);
    chain.add_option(liquid);
    chain.add_option(illiquid);

    // When: We filter by liquidity (min_volume=100, min_oi=500)
    auto filtered = chain.filter_by_liquidity(100, 500);

    // Then: Should return only liquid option
    REQUIRE(filtered.size() == 1);
    REQUIRE(filtered[0].contract.strike == 500.0);
  }

  SECTION("Get calls and puts separately") {
    // Given: Chain with both calls and puts
    chain.add_option(create_test_entry("SPY", "20250620", 500.0, OptionType::Call));
    chain.add_option(create_test_entry("SPY", "20250620", 500.0, OptionType::Put));
    chain.add_option(create_test_entry("SPY", "20250620", 510.0, OptionType::Call));

    // When: We get calls only
    auto calls = chain.get_calls();

    // Then: Should return 2 calls
    REQUIRE(calls.size() == 2);

    // When: We get puts only
    auto puts = chain.get_puts();

    // Then: Should return 1 put
    REQUIRE(puts.size() == 1);
  }
}

// ============================================================================
// OptionChain Tests
// ============================================================================

TEST_CASE("OptionChain expiry management", "[option_chain][chain]") {
  // Given: An option chain for SPY
  OptionChain chain("SPY");

  SECTION("Add options across multiple expiries") {
    // Given: Options with different expiries
    auto entry1 = create_test_entry("SPY", "20250620", 500.0, OptionType::Call);
    auto entry2 = create_test_entry("SPY", "20250627", 500.0, OptionType::Call);

    // When: We add options to the chain
    chain.add_option(entry1);
    chain.add_option(entry2);

    // Then: Should have 2 expiries
    auto expiries = chain.get_expiries();
    REQUIRE(expiries.size() == 2);
    REQUIRE(chain.get_expiry_count() == 2);
  }

  SECTION("Get expiry chain") {
    // Given: Chain with options at expiry 20250620
    auto entry = create_test_entry("SPY", "20250620", 500.0, OptionType::Call);
    chain.add_option(entry);

    // When: We get expiry chain for 20250620
    auto expiry_chain = chain.get_expiry_chain("20250620");

    // Then: Should return valid expiry chain
    REQUIRE(expiry_chain.has_value());
    REQUIRE(expiry_chain->get_expiry() == "20250620");
    REQUIRE(expiry_chain->get_symbol() == "SPY");
  }

  SECTION("Get all options across expiries") {
    // Given: Options at multiple expiries
    chain.add_option(create_test_entry("SPY", "20250620", 500.0, OptionType::Call));
    chain.add_option(create_test_entry("SPY", "20250627", 500.0, OptionType::Call));
    chain.add_option(create_test_entry("SPY", "20250620", 510.0, OptionType::Call));

    // When: We get all options
    auto all_options = chain.get_all_options();

    // Then: Should return 3 options
    REQUIRE(all_options.size() == 3);
  }

  SECTION("Set underlying price") {
    // Given: Chain with underlying price
    double underlying_price = 505.0;

    // When: We set underlying price
    chain.set_underlying_price(underlying_price);

    // Then: Should return the set price
    REQUIRE(chain.get_underlying_price() == underlying_price);
  }

  SECTION("Get total option count") {
    // Given: Chain with multiple options
    chain.add_option(create_test_entry("SPY", "20250620", 500.0, OptionType::Call));
    chain.add_option(create_test_entry("SPY", "20250620", 510.0, OptionType::Call));
    chain.add_option(create_test_entry("SPY", "20250627", 500.0, OptionType::Call));

    // When: We get total option count
    int count = chain.get_total_option_count();

    // Then: Should return 3
    REQUIRE(count == 3);
  }
}

// ============================================================================
// OptionChainBuilder Tests
// ============================================================================

TEST_CASE("OptionChainBuilder builds chain from market data", "[option_chain][builder]") {
  // Given: Market data for options
  std::vector<OptionContract> contracts;
  std::map<std::string, MarketData> market_data;

  OptionContract contract1;
  contract1.symbol = "SPY";
  contract1.expiry = "20250620";
  contract1.strike = 500.0;
  contract1.type = OptionType::Call;
  contracts.push_back(contract1);

  MarketData data1;
  data1.bid = 1.0;
  data1.ask = 1.1;
  market_data["SPY_20250620_500_C"] = data1;

  SECTION("Build chain from contracts and market data") {
    // When: We build chain from market data
    auto chain = OptionChainBuilder::build_from_market_data(
        "SPY", contracts, market_data);

    // Then: Chain should be created
    REQUIRE(chain.get_symbol() == "SPY");
    REQUIRE(chain.get_total_option_count() > 0);
  }

  SECTION("Calculate metrics for entry") {
    // Given: An option chain entry
    OptionChainEntry entry;
    entry.contract.symbol = "SPY";
    entry.contract.strike = 500.0;
    entry.contract.type = OptionType::Call;
    entry.market_data.bid = 1.0;
    entry.market_data.ask = 1.1;
    double underlying_price = 505.0;

    // When: We calculate metrics
    OptionChainBuilder::calculate_metrics(entry, underlying_price);

    // Then: Entry should have calculated metrics
    // (Exact values depend on implementation, but should be non-zero)
    REQUIRE(entry.moneyness > 0.0);
  }
}

// ============================================================================
// Edge Cases and Error Conditions
// ============================================================================

TEST_CASE("OptionChain edge cases", "[option_chain][edge]") {
  SECTION("Empty expiry chain") {
    // Given: An empty expiry chain
    ExpiryChain chain("SPY", "20250620");

    // When: We get strikes
    auto strikes = chain.get_strikes();

    // Then: Should return empty vector
    REQUIRE(strikes.empty());
  }

  SECTION("Get non-existent strike chain") {
    // Given: Chain without strike 500
    ExpiryChain chain("SPY", "20250620");
    chain.add_option(create_test_entry("SPY", "20250620", 510.0, OptionType::Call));

    // When: We get strike chain for 500
    auto strike_chain = chain.get_strike_chain(500.0);

    // Then: Should return nullopt
    REQUIRE_FALSE(strike_chain.has_value());
  }

  SECTION("Get non-existent option") {
    // Given: Chain without call at strike 500
    ExpiryChain chain("SPY", "20250620");
    chain.add_option(create_test_entry("SPY", "20250620", 500.0, OptionType::Put));

    // When: We get call option at strike 500
    auto option = chain.get_option(500.0, OptionType::Call);

    // Then: Should return nullopt
    REQUIRE_FALSE(option.has_value());
  }

  SECTION("Empty option chain") {
    // Given: An empty option chain
    OptionChain chain("SPY");

    // When: We get expiries
    auto expiries = chain.get_expiries();

    // Then: Should return empty vector
    REQUIRE(expiries.empty());
    REQUIRE(chain.get_total_option_count() == 0);
  }
}
