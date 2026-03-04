// test_config_manager.cpp - Configuration manager tests
#include "config_manager.h"
#include <catch2/catch_test_macros.hpp>
#include <catch2/matchers/catch_matchers_floating_point.hpp>
#include <filesystem>
#include <fstream>

using namespace config;

TEST_CASE("ConfigManager loads default configuration", "[config]") {
  Config config = ConfigManager::get_default();

  REQUIRE(config.tws.host == "127.0.0.1");
  REQUIRE(config.tws.port == 7497);
  REQUIRE(config.tws.client_id == 1);
  REQUIRE(config.tws.connect_options == "+PACEAPI");
}

TEST_CASE("ConfigManager validates TWS configuration", "[config]") {
  SECTION("Valid TWS config passes validation") {
    TWSConfig tws;
    tws.host = "127.0.0.1";
    tws.port = 7497;
    tws.client_id = 1;

    REQUIRE_NOTHROW(ConfigManager::validate_tws_config(tws));
  }

  SECTION("Valid TWS config with connect_options passes validation") {
    TWSConfig tws;
    tws.host = "127.0.0.1";
    tws.port = 7497;
    tws.client_id = 1;
    tws.connect_options = "+PACEAPI";

    REQUIRE_NOTHROW(ConfigManager::validate_tws_config(tws));
  }

  SECTION("Non-ASCII connect_options fails validation") {
    TWSConfig tws;
    tws.host = "127.0.0.1";
    tws.port = 7497;
    tws.client_id = 1;
    tws.connect_options = "+PACEAPI\x01"; // non-printable

    REQUIRE_THROWS_AS(ConfigManager::validate_tws_config(tws),
                      std::invalid_argument);
  }

  SECTION("Non-ASCII optional_capabilities fails validation") {
    TWSConfig tws;
    tws.host = "127.0.0.1";
    tws.port = 7497;
    tws.client_id = 1;
    tws.optional_capabilities = "cap\x7f";

    REQUIRE_THROWS_AS(ConfigManager::validate_tws_config(tws),
                      std::invalid_argument);
  }

  SECTION("Empty host fails validation") {
    TWSConfig tws;
    tws.host = "";
    tws.port = 7497;

    REQUIRE_THROWS_AS(ConfigManager::validate_tws_config(tws),
                      std::invalid_argument);
  }

  SECTION("Invalid port fails validation") {
    TWSConfig tws;
    tws.host = "127.0.0.1";
    tws.port = 100; // Too low

    REQUIRE_THROWS_AS(ConfigManager::validate_tws_config(tws),
                      std::invalid_argument);
  }
}

TEST_CASE("ConfigManager validates strategy parameters", "[config]") {
  SECTION("Valid strategy params pass validation") {
    StrategyParams params;
    params.symbols = {"SPY", "QQQ"};
    params.min_arbitrage_profit = 0.10;
    params.min_roi_percent = 0.5;
    params.max_position_size = 10000.0;

    REQUIRE_NOTHROW(ConfigManager::validate_strategy_params(params));
  }

  SECTION("Empty symbols fail validation") {
    StrategyParams params;
    params.symbols = {};

    REQUIRE_THROWS_AS(ConfigManager::validate_strategy_params(params),
                      std::invalid_argument);
  }

  SECTION("Invalid symbol format fails validation") {
    StrategyParams params;
    params.symbols = {"invalid_symbol_123"};

    REQUIRE_THROWS_AS(ConfigManager::validate_strategy_params(params),
                      std::invalid_argument);
  }

  SECTION("Negative profit threshold fails validation") {
    StrategyParams params;
    params.symbols = {"SPY"};
    params.min_arbitrage_profit = -1.0;

    REQUIRE_THROWS_AS(ConfigManager::validate_strategy_params(params),
                      std::invalid_argument);
  }
}

TEST_CASE("ConfigManager validates risk configuration", "[config]") {
  SECTION("Valid risk config passes validation") {
    RiskConfig risk;
    risk.max_total_exposure = 50000.0;
    risk.max_positions = 10;
    risk.position_size_percent = 0.1;

    REQUIRE_NOTHROW(ConfigManager::validate_risk_config(risk));
  }

  SECTION("Zero max exposure fails validation") {
    RiskConfig risk;
    risk.max_total_exposure = 0.0;

    REQUIRE_THROWS_AS(ConfigManager::validate_risk_config(risk),
                      std::invalid_argument);
  }

  SECTION("Invalid position size percent fails validation") {
    RiskConfig risk;
    risk.position_size_percent = 1.5; // > 1.0

    REQUIRE_THROWS_AS(ConfigManager::validate_risk_config(risk),
                      std::invalid_argument);
  }
}

TEST_CASE("ConfigManager JSON serialization", "[config]") {
  SECTION("Config can be serialized to JSON") {
    Config config = ConfigManager::get_default();
    config.tws.host = "localhost";
    config.tws.port = 7496;
    config.strategy.symbols = {"SPY", "QQQ"};

    nlohmann::json j = ConfigManager::to_json(config);

    REQUIRE(j["tws"]["host"] == "localhost");
    REQUIRE(j["tws"]["port"] == 7496);
    REQUIRE(j["strategy"]["symbols"].size() == 2);
  }

  SECTION("JSON can be deserialized to Config") {
    nlohmann::json j = {
        {"tws", {{"host", "127.0.0.1"}, {"port", 7497}, {"client_id", 1}}},
        {"strategy",
         {{"symbols", {"SPY"}},
          {"min_arbitrage_profit", 0.10},
          {"min_roi_percent", 0.5},
          {"max_position_size", 10000.0},
          {"min_days_to_expiry", 30},
          {"max_days_to_expiry", 90},
          {"max_bid_ask_spread", 0.10},
          {"min_volume", 100},
          {"min_open_interest", 500}}}};

    Config config = ConfigManager::from_json(j);

    REQUIRE(config.tws.host == "127.0.0.1");
    REQUIRE(config.tws.port == 7497);
    REQUIRE(config.tws.connect_options ==
            "+PACEAPI"); // default when key omitted
    REQUIRE(config.strategy.symbols[0] == "SPY");
    REQUIRE_THAT(config.strategy.min_arbitrage_profit,
                 Catch::Matchers::WithinRel(0.10, 0.001));
  }

  SECTION("connect_options and optional_capabilities round-trip") {
    Config config = ConfigManager::get_default();
    config.tws.connect_options = "+PACEAPI";
    config.tws.optional_capabilities = "cap123";

    nlohmann::json j = ConfigManager::to_json(config);
    REQUIRE(j["tws"]["connect_options"] == "+PACEAPI");
    REQUIRE(j["tws"]["optional_capabilities"] == "cap123");

    Config loaded = ConfigManager::from_json(j);
    REQUIRE(loaded.tws.connect_options == "+PACEAPI");
    REQUIRE(loaded.tws.optional_capabilities == "cap123");
  }

  SECTION("empty connect_options omits from JSON and loads as empty") {
    nlohmann::json j = {{"tws",
                         {{"host", "127.0.0.1"},
                          {"port", 7497},
                          {"client_id", 1},
                          {"connect_options", ""}}},
                        {"strategy",
                         {{"symbols", {"SPY"}},
                          {"min_arbitrage_profit", 0.10},
                          {"min_roi_percent", 0.5},
                          {"max_position_size", 10000.0},
                          {"min_days_to_expiry", 30},
                          {"max_days_to_expiry", 90},
                          {"max_bid_ask_spread", 0.10},
                          {"min_volume", 100},
                          {"min_open_interest", 500}}}};
    Config config = ConfigManager::from_json(j);
    REQUIRE(config.tws.connect_options.empty());
  }
}

TEST_CASE("ConfigManager file operations", "[config]") {
  const std::string test_config_file = "test_config.json";

  // Cleanup
  if (std::filesystem::exists(test_config_file)) {
    std::filesystem::remove(test_config_file);
  }

  SECTION("Save and load configuration") {
    Config original = ConfigManager::get_default();
    original.tws.host = "test.example.com";
    original.tws.port = 8080;
    original.strategy.symbols = {"TEST"};

    // Save
    REQUIRE_NOTHROW(ConfigManager::save(original, test_config_file));
    REQUIRE(std::filesystem::exists(test_config_file));

    // Load
    Config loaded = ConfigManager::load(test_config_file);

    REQUIRE(loaded.tws.host == original.tws.host);
    REQUIRE(loaded.tws.port == original.tws.port);
    REQUIRE(loaded.tws.connect_options == original.tws.connect_options);
    REQUIRE(loaded.strategy.symbols == original.strategy.symbols);
  }

  // Cleanup
  if (std::filesystem::exists(test_config_file)) {
    std::filesystem::remove(test_config_file);
  }
}
