// market_hours.cpp - Market hours and holiday calendar implementation
#include "market_hours.h"
#include <spdlog/spdlog.h>
#include <ctime>
#include <iomanip>
#include <sstream>
#include <algorithm>

namespace market_hours {

MarketHours::MarketHours() {
    // Initialize US Market Holidays 2025
    holidays_2025_ = {
        "20250101",  // New Year's Day
        "20250120",  // Martin Luther King Jr. Day
        "20250217",  // Presidents' Day
        "20250418",  // Good Friday
        "20250526",  // Memorial Day
        "20250619",  // Juneteenth
        "20250704",  // Independence Day
        "20250901",  // Labor Day
        "20251127",  // Thanksgiving
        "20251225"   // Christmas
    };

    // Early close dates 2025 (1:00 PM ET close)
    early_closes_2025_ = {
        "20250703",  // Day before Independence Day
        "20251128",  // Day after Thanksgiving
        "20251224"   // Christmas Eve
    };

    spdlog::debug("MarketHours initialized with {} holidays and {} early closes for 2025",
                  holidays_2025_.size(), early_closes_2025_.size());
}

MarketStatus MarketHours::get_market_status() const {
    return get_market_status_at(std::chrono::system_clock::now());
}

MarketStatus MarketHours::get_market_status_at(std::chrono::system_clock::time_point time) const {
    MarketStatus status{};

    // Get ET time components
    auto [hour_et, minute_et, wday] = get_et_time(time);
    std::string date_str = date_to_string(time);

    // Check if holiday
    bool holiday = is_holiday(time);
    bool is_early_close_day = is_early_close(time);

    if (holiday) {
        status.is_open = false;
        status.current_session = MarketSession::Closed;
        status.is_holiday = true;
        status.is_early_close = false;
        status.reason = "holiday";
        return status;
    }

    // Check if weekend
    if (wday == 0 || wday == 6) {  // Sunday = 0, Saturday = 6
        status.is_open = false;
        status.current_session = MarketSession::Closed;
        status.is_holiday = false;
        status.is_early_close = false;
        status.reason = "weekend";
        return status;
    }

    // Determine close hour (1:00 PM for early closes, 4:00 PM otherwise)
    int close_hour = get_close_hour_et(time);

    // Check market session
    if (is_pre_market_hours(hour_et, minute_et)) {
        status.is_open = true;
        status.current_session = MarketSession::PreMarket;
        status.reason = "pre_market";
    } else if (is_regular_hours(hour_et, minute_et) && hour_et < close_hour) {
        status.is_open = true;
        status.current_session = MarketSession::Regular;
        status.reason = is_early_close_day ? "regular_hours_early_close" : "regular_hours";
    } else if (is_after_hours(hour_et, minute_et) && hour_et < 20) {  // After-hours until 8:00 PM
        status.is_open = true;
        status.current_session = MarketSession::AfterHours;
        status.reason = "after_hours";
    } else {
        status.is_open = false;
        status.current_session = MarketSession::Closed;
        status.reason = "closed";
    }

    status.is_holiday = holiday;
    status.is_early_close = is_early_close_day;

    // Calculate next open and close times
    status.next_open_time = calculate_next_open(time);
    status.next_close_time = calculate_next_close(time);

    return status;
}

bool MarketHours::is_holiday(const std::chrono::system_clock::time_point& date) const {
    return is_holiday_date(date);
}

bool MarketHours::is_early_close(const std::chrono::system_clock::time_point& date) const {
    return is_early_close_date(date);
}

int MarketHours::get_close_hour_et(const std::chrono::system_clock::time_point& date) const {
    std::string date_str = date_to_string(date);
    if (early_closes_2025_.count(date_str) > 0) {
        return 13;  // 1:00 PM ET
    }
    return 16;  // 4:00 PM ET
}

std::string MarketHours::date_to_string(const std::chrono::system_clock::time_point& date) const {
    auto time_t = std::chrono::system_clock::to_time_t(date);
    std::tm tm = *std::gmtime(&time_t);

    // Format as YYYYMMDD (no timezone conversion needed for date string)
    std::ostringstream oss;
    oss << std::setfill('0') << std::setw(4) << (tm.tm_year + 1900)
        << std::setw(2) << (tm.tm_mon + 1)
        << std::setw(2) << tm.tm_mday;
    return oss.str();
}

std::tuple<int, int, int> MarketHours::get_et_time(const std::chrono::system_clock::time_point& time) const {
    auto time_t = std::chrono::system_clock::to_time_t(time);
    std::tm tm_utc = *std::gmtime(&time_t);

    // DST in US Eastern Time:
    // Starts: Second Sunday in March at 2:00 AM (spring forward)
    // Ends: First Sunday in November at 2:00 AM (fall back)
    // During DST: EDT = UTC-4, Otherwise: EST = UTC-5

    // Determine if DST is in effect
    int year = tm_utc.tm_year + 1900;
    int month = tm_utc.tm_mon + 1;
    int day = tm_utc.tm_mday;
    int hour_utc = tm_utc.tm_hour;

    bool is_dst = false;

    // DST starts: Second Sunday in March
    if (month == 3) {
        // Find second Sunday in March
        std::tm march_first = {};
        march_first.tm_year = year - 1900;
        march_first.tm_mon = 2;  // March (0-indexed)
        march_first.tm_mday = 1;
        march_first.tm_hour = 2;  // 2:00 AM
        march_first.tm_min = 0;
        march_first.tm_sec = 0;
        std::mktime(&march_first);

        // Find first Sunday
        int first_sunday = 1 + (7 - march_first.tm_wday) % 7;
        if (first_sunday == 7) first_sunday = 0;  // If March 1 is Sunday, first Sunday is March 1
        int second_sunday = first_sunday + 7;

        // DST starts at 2:00 AM on second Sunday
        if (day > second_sunday || (day == second_sunday && hour_utc >= 7)) {  // 7 UTC = 2 AM EST (before DST)
            is_dst = true;
        }
    }
    // DST ends: First Sunday in November
    else if (month == 11) {
        // Find first Sunday in November
        std::tm nov_first = {};
        nov_first.tm_year = year - 1900;
        nov_first.tm_mon = 10;  // November (0-indexed)
        nov_first.tm_mday = 1;
        nov_first.tm_hour = 2;  // 2:00 AM
        nov_first.tm_min = 0;
        nov_first.tm_sec = 0;
        std::mktime(&nov_first);

        int first_sunday = 1 + (7 - nov_first.tm_wday) % 7;
        if (first_sunday == 7) first_sunday = 0;

        // DST ends at 2:00 AM on first Sunday
        if (day < first_sunday || (day == first_sunday && hour_utc < 6)) {  // 6 UTC = 1 AM EST (after DST ends)
            is_dst = true;
        }
    }
    // DST is active: April through October
    else if (month > 3 && month < 11) {
        is_dst = true;
    }

    // Convert UTC to ET: UTC-4 (EDT) or UTC-5 (EST)
    int offset_hours = is_dst ? 4 : 5;
    int hour_et = hour_utc - offset_hours;
    if (hour_et < 0) {
        hour_et += 24;
    }

    return std::make_tuple(hour_et, tm_utc.tm_min, tm_utc.tm_wday);
}

bool MarketHours::is_regular_hours(int hour_et, int minute_et) const {
    // Regular hours: 9:30 AM - 4:00 PM ET
    if (hour_et == 9) {
        return minute_et >= 30;
    }
    return hour_et >= 10 && hour_et < 16;
}

bool MarketHours::is_pre_market_hours(int hour_et, int minute_et) const {
    // Pre-market: 4:00 AM - 9:30 AM ET
    if (hour_et == 9) {
        return minute_et < 30;
    }
    return hour_et >= 4 && hour_et < 9;
}

bool MarketHours::is_after_hours(int hour_et, int minute_et) const {
    // After-hours: 4:00 PM - 8:00 PM ET
    return hour_et >= 16 && hour_et < 20;
}

bool MarketHours::is_holiday_date(const std::chrono::system_clock::time_point& date) const {
    std::string date_str = date_to_string(date);
    return holidays_2025_.count(date_str) > 0;
}

bool MarketHours::is_early_close_date(const std::chrono::system_clock::time_point& date) const {
    std::string date_str = date_to_string(date);
    return early_closes_2025_.count(date_str) > 0;
}

void MarketHours::update_holiday_calendar(int year) {
    if (year == 2025) {
        return;
    }

    // NYSE/NASDAQ fixed-date and rule-based US market holidays.
    // Each entry is generated from the federal holiday rules.
    holidays_by_year_[year] = generate_holidays_for_year(year);
    early_closes_by_year_[year] = generate_early_closes_for_year(year);

    spdlog::info("Holiday calendar updated for year {} ({} holidays, {} early closes)",
                 year,
                 holidays_by_year_[year].size(),
                 early_closes_by_year_[year].size());
}

std::string MarketHours::calculate_next_open(
    const std::chrono::system_clock::time_point& from) const {

    auto candidate = from;
    for (int i = 0; i < 10; ++i) {
        candidate += std::chrono::hours(24);

        auto [hour_et, minute_et, wday] = get_et_time(candidate);
        if (wday == 0 || wday == 6) continue;
        if (is_holiday_date(candidate)) continue;

        // Next regular open is 9:30 AM ET on this day
        auto t = std::chrono::system_clock::to_time_t(candidate);
        std::tm tm = *std::gmtime(&t);
        std::ostringstream oss;
        oss << std::setfill('0')
            << std::setw(4) << (tm.tm_year + 1900) << "-"
            << std::setw(2) << (tm.tm_mon + 1) << "-"
            << std::setw(2) << tm.tm_mday
            << "T09:30:00-05:00";
        return oss.str();
    }
    return "";
}

std::string MarketHours::calculate_next_close(
    const std::chrono::system_clock::time_point& from) const {

    auto [hour_et, minute_et, wday] = get_et_time(from);
    std::string date_str = date_to_string(from);

    // If currently in a trading session, close is today
    if (wday >= 1 && wday <= 5 && !is_holiday_date(from)) {
        int close_hour = get_close_hour_et(from);
        if (hour_et < close_hour || (hour_et == close_hour && minute_et == 0)) {
            auto t = std::chrono::system_clock::to_time_t(from);
            std::tm tm = *std::gmtime(&t);
            std::ostringstream oss;
            oss << std::setfill('0')
                << std::setw(4) << (tm.tm_year + 1900) << "-"
                << std::setw(2) << (tm.tm_mon + 1) << "-"
                << std::setw(2) << tm.tm_mday
                << "T" << std::setw(2) << close_hour << ":00:00-05:00";
            return oss.str();
        }
    }

    // Otherwise find the next trading day's close
    auto candidate = from;
    for (int i = 0; i < 10; ++i) {
        candidate += std::chrono::hours(24);
        auto [h, m, w] = get_et_time(candidate);
        if (w == 0 || w == 6) continue;
        if (is_holiday_date(candidate)) continue;

        int close_h = get_close_hour_et(candidate);
        auto t = std::chrono::system_clock::to_time_t(candidate);
        std::tm tm = *std::gmtime(&t);
        std::ostringstream oss;
        oss << std::setfill('0')
            << std::setw(4) << (tm.tm_year + 1900) << "-"
            << std::setw(2) << (tm.tm_mon + 1) << "-"
            << std::setw(2) << tm.tm_mday
            << "T" << std::setw(2) << close_h << ":00:00-05:00";
        return oss.str();
    }
    return "";
}

std::set<std::string> MarketHours::generate_holidays_for_year(int year) const {
    std::set<std::string> holidays;
    auto fmt = [](int y, int m, int d) {
        std::ostringstream oss;
        oss << std::setfill('0') << std::setw(4) << y
            << std::setw(2) << m << std::setw(2) << d;
        return oss.str();
    };

    // New Year's Day (Jan 1, or observed if falls on weekend)
    holidays.insert(fmt(year, 1, 1));

    // MLK Day - third Monday in January
    {
        std::tm jan1{}; jan1.tm_year = year - 1900; jan1.tm_mon = 0; jan1.tm_mday = 1;
        std::mktime(&jan1);
        int first_monday = (8 - jan1.tm_wday) % 7;
        if (first_monday == 0) first_monday = 7;
        int third_monday = first_monday + 14;
        holidays.insert(fmt(year, 1, third_monday));
    }

    // Presidents' Day - third Monday in February
    {
        std::tm feb1{}; feb1.tm_year = year - 1900; feb1.tm_mon = 1; feb1.tm_mday = 1;
        std::mktime(&feb1);
        int first_monday = (8 - feb1.tm_wday) % 7;
        if (first_monday == 0) first_monday = 7;
        int third_monday = first_monday + 14;
        holidays.insert(fmt(year, 2, third_monday));
    }

    // Good Friday (approximation: March/April)
    // Simplified: use known dates for common years
    // Full Easter calculation is complex; cover 2025-2030
    static const std::map<int, std::pair<int,int>> good_fridays = {
        {2025, {4, 18}}, {2026, {4, 3}}, {2027, {3, 26}},
        {2028, {4, 14}}, {2029, {3, 30}}, {2030, {4, 19}},
    };
    if (auto it = good_fridays.find(year); it != good_fridays.end()) {
        holidays.insert(fmt(year, it->second.first, it->second.second));
    }

    // Memorial Day - last Monday in May
    {
        std::tm may31{}; may31.tm_year = year - 1900; may31.tm_mon = 4; may31.tm_mday = 31;
        std::mktime(&may31);
        int last_monday = 31 - ((may31.tm_wday + 6) % 7);
        holidays.insert(fmt(year, 5, last_monday));
    }

    // Juneteenth (June 19)
    holidays.insert(fmt(year, 6, 19));

    // Independence Day (July 4)
    holidays.insert(fmt(year, 7, 4));

    // Labor Day - first Monday in September
    {
        std::tm sep1{}; sep1.tm_year = year - 1900; sep1.tm_mon = 8; sep1.tm_mday = 1;
        std::mktime(&sep1);
        int first_monday = (8 - sep1.tm_wday) % 7;
        if (first_monday == 0) first_monday = 7;
        holidays.insert(fmt(year, 9, first_monday));
    }

    // Thanksgiving - fourth Thursday in November
    {
        std::tm nov1{}; nov1.tm_year = year - 1900; nov1.tm_mon = 10; nov1.tm_mday = 1;
        std::mktime(&nov1);
        int first_thursday = (11 - nov1.tm_wday) % 7;
        if (first_thursday == 0) first_thursday = 7;
        int fourth_thursday = first_thursday + 21;
        holidays.insert(fmt(year, 11, fourth_thursday));
    }

    // Christmas (December 25)
    holidays.insert(fmt(year, 12, 25));

    return holidays;
}

std::set<std::string> MarketHours::generate_early_closes_for_year(int year) const {
    std::set<std::string> early;
    auto fmt = [](int y, int m, int d) {
        std::ostringstream oss;
        oss << std::setfill('0') << std::setw(4) << y
            << std::setw(2) << m << std::setw(2) << d;
        return oss.str();
    };

    // Day before Independence Day (July 3)
    early.insert(fmt(year, 7, 3));

    // Day after Thanksgiving
    auto holidays = generate_holidays_for_year(year);
    for (const auto& h : holidays) {
        if (h.substr(4, 2) == "11" && std::stoi(h.substr(6, 2)) >= 22) {
            int thanksgiving_day = std::stoi(h.substr(6, 2));
            early.insert(fmt(year, 11, thanksgiving_day + 1));
            break;
        }
    }

    // Christmas Eve (December 24)
    early.insert(fmt(year, 12, 24));

    return early;
}

} // namespace market_hours
