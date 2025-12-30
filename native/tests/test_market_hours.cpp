// test_market_hours.cpp - Market hours and DST tests
#include <catch2/catch_test_macros.hpp>
#include "market_hours.h"
#include <chrono>
#include <ctime>

using namespace market_hours;

// ============================================================================
// Market Hours Tests
// ============================================================================

TEST_CASE("MarketHours - Holiday detection", "[market_hours][holiday]") {
    MarketHours mh;

    // Test New Year's Day 2025 (January 1)
    std::tm tm = {};
    tm.tm_year = 2025 - 1900;
    tm.tm_mon = 0;  // January
    tm.tm_mday = 1;
    tm.tm_hour = 12;
    tm.tm_min = 0;
    tm.tm_sec = 0;
    auto time = std::chrono::system_clock::from_time_t(std::mktime(&tm));

    REQUIRE(mh.is_holiday(time) == true);

    // Test regular weekday (not a holiday)
    tm.tm_mday = 2;  // January 2, 2025 (Thursday)
    time = std::chrono::system_clock::from_time_t(std::mktime(&tm));

    REQUIRE(mh.is_holiday(time) == false);
}

TEST_CASE("MarketHours - Early close detection", "[market_hours][early_close]") {
    MarketHours mh;

    // Test July 3, 2025 (early close - day before Independence Day)
    std::tm tm = {};
    tm.tm_year = 2025 - 1900;
    tm.tm_mon = 6;  // July
    tm.tm_mday = 3;
    tm.tm_hour = 12;
    tm.tm_min = 0;
    tm.tm_sec = 0;
    auto time = std::chrono::system_clock::from_time_t(std::mktime(&tm));

    REQUIRE(mh.is_early_close(time) == true);
    REQUIRE(mh.get_close_hour_et(time) == 13);  // 1:00 PM ET

    // Test regular trading day
    tm.tm_mday = 2;  // July 2, 2025
    time = std::chrono::system_clock::from_time_t(std::mktime(&tm));

    REQUIRE(mh.is_early_close(time) == false);
    REQUIRE(mh.get_close_hour_et(time) == 16);  // 4:00 PM ET
}

TEST_CASE("MarketHours - Market status during regular hours", "[market_hours][status]") {
    MarketHours mh;

    // Test regular trading hours (Wednesday, January 2, 2025 at 2:00 PM ET)
    std::tm tm = {};
    tm.tm_year = 2025 - 1900;
    tm.tm_mon = 0;  // January
    tm.tm_mday = 2;
    tm.tm_hour = 19;  // 2:00 PM ET = 7:00 PM UTC (EST, UTC-5)
    tm.tm_min = 0;
    tm.tm_sec = 0;
    auto time = std::chrono::system_clock::from_time_t(std::mktime(&tm));

    auto status = mh.get_market_status_at(time);
    REQUIRE(status.is_open == true);
    REQUIRE(status.current_session == MarketSession::Regular);
    REQUIRE(status.is_holiday == false);
}

TEST_CASE("MarketHours - Market status on weekend", "[market_hours][weekend]") {
    MarketHours mh;

    // Test Saturday, January 4, 2025
    std::tm tm = {};
    tm.tm_year = 2025 - 1900;
    tm.tm_mon = 0;  // January
    tm.tm_mday = 4;  // Saturday
    tm.tm_hour = 14;
    tm.tm_min = 0;
    tm.tm_sec = 0;
    auto time = std::chrono::system_clock::from_time_t(std::mktime(&tm));

    auto status = mh.get_market_status_at(time);
    REQUIRE(status.is_open == false);
    REQUIRE(status.current_session == MarketSession::Closed);
    REQUIRE(status.reason == "weekend");
}

TEST_CASE("MarketHours - Market status on holiday", "[market_hours][holiday_status]") {
    MarketHours mh;

    // Test New Year's Day 2025 (January 1, Wednesday)
    std::tm tm = {};
    tm.tm_year = 2025 - 1900;
    tm.tm_mon = 0;  // January
    tm.tm_mday = 1;
    tm.tm_hour = 14;
    tm.tm_min = 0;
    tm.tm_sec = 0;
    auto time = std::chrono::system_clock::from_time_t(std::mktime(&tm));

    auto status = mh.get_market_status_at(time);
    REQUIRE(status.is_open == false);
    REQUIRE(status.current_session == MarketSession::Closed);
    REQUIRE(status.is_holiday == true);
    REQUIRE(status.reason == "holiday");
}

TEST_CASE("MarketHours - DST timezone conversion", "[market_hours][dst]") {
    MarketHours mh;

    // Test during DST period (July 15, 2025 - should be EDT, UTC-4)
    // Note: get_et_time is private, so we test via get_market_status_at
    std::tm tm_dst = {};
    tm_dst.tm_year = 2025 - 1900;
    tm_dst.tm_mon = 6;  // July
    tm_dst.tm_mday = 15;
    tm_dst.tm_hour = 16;  // 4:00 PM UTC
    tm_dst.tm_min = 0;
    tm_dst.tm_sec = 0;
    auto time_dst = std::chrono::system_clock::from_time_t(std::mktime(&tm_dst));

    // Test market status during DST (should correctly convert UTC to ET)
    auto status_dst = mh.get_market_status_at(time_dst);
    // Market should be open during regular hours (2:00 PM ET = 6:00 PM UTC during DST)
    // Actually: 4:00 PM UTC - 4 hours (EDT) = 12:00 PM ET (regular hours)
    REQUIRE(status_dst.is_open == true);
    REQUIRE(status_dst.current_session == MarketSession::Regular);

    // Test during EST period (January 15, 2025 - should be EST, UTC-5)
    std::tm tm_est = {};
    tm_est.tm_year = 2025 - 1900;
    tm_est.tm_mon = 0;  // January
    tm_est.tm_mday = 15;
    tm_est.tm_hour = 18;  // 6:00 PM UTC
    tm_est.tm_min = 0;
    tm_est.tm_sec = 0;
    auto time_est = std::chrono::system_clock::from_time_t(std::mktime(&tm_est));

    // Test market status during EST
    auto status_est = mh.get_market_status_at(time_est);
    // 6:00 PM UTC - 5 hours (EST) = 1:00 PM ET (regular hours)
    REQUIRE(status_est.is_open == true);
    REQUIRE(status_est.current_session == MarketSession::Regular);
}
