// market_hours.h - Market hours and holiday calendar
#pragma once

#include <chrono>
#include <map>
#include <string>
#include <set>
#include <vector>

namespace market_hours {

// ============================================================================
// Market Session Types
// ============================================================================

enum class MarketSession {
    PreMarket,      // Pre-market hours (4:00 AM - 9:30 AM ET)
    Regular,        // Regular trading hours (9:30 AM - 4:00 PM ET)
    AfterHours,     // After-hours (4:00 PM - 8:00 PM ET)
    Closed          // Market is closed
};

// ============================================================================
// Market Status
// ============================================================================

struct MarketStatus {
    bool is_open;
    MarketSession current_session;
    std::string next_open_time;      // ISO 8601 format
    std::string next_close_time;     // ISO 8601 format
    bool is_holiday;
    bool is_early_close;
    std::string reason;              // "holiday", "early_close", "regular_hours", etc.
};

// ============================================================================
// MarketHours Class
// ============================================================================

class MarketHours {
public:
    MarketHours();
    ~MarketHours() = default;

    // Check if market is currently open
    MarketStatus get_market_status() const;

    // Check if market is open at a specific time
    MarketStatus get_market_status_at(std::chrono::system_clock::time_point time) const;

    // Check if a specific date is a market holiday
    bool is_holiday(const std::chrono::system_clock::time_point& date) const;

    // Check if a specific date is an early close day
    bool is_early_close(const std::chrono::system_clock::time_point& date) const;

    // Get early close time for a specific date (returns 13:00 ET if early close, 16:00 ET otherwise)
    int get_close_hour_et(const std::chrono::system_clock::time_point& date) const;

    // Update holiday calendar (call annually)
    void update_holiday_calendar(int year);

private:
    // US Market Holidays 2025
    std::set<std::string> holidays_2025_;
    std::set<std::string> early_closes_2025_;

    // Dynamic year calendars
    std::map<int, std::set<std::string>> holidays_by_year_;
    std::map<int, std::set<std::string>> early_closes_by_year_;

    std::string date_to_string(const std::chrono::system_clock::time_point& date) const;
    std::tuple<int, int, int> get_et_time(const std::chrono::system_clock::time_point& time) const;
    bool is_regular_hours(int hour_et, int minute_et) const;
    bool is_pre_market_hours(int hour_et, int minute_et) const;
    bool is_after_hours(int hour_et, int minute_et) const;
    bool is_holiday_date(const std::chrono::system_clock::time_point& date) const;
    bool is_early_close_date(const std::chrono::system_clock::time_point& date) const;

    // Next open/close calculation
    std::string calculate_next_open(const std::chrono::system_clock::time_point& from) const;
    std::string calculate_next_close(const std::chrono::system_clock::time_point& from) const;

    // Holiday generation for arbitrary years
    std::set<std::string> generate_holidays_for_year(int year) const;
    std::set<std::string> generate_early_closes_for_year(int year) const;
};

} // namespace market_hours
