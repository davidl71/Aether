// tui_converter.h - Convert between native trading types and TUI display types
#pragma once

#include "types.h"
#include "tui_data.h"
#include <vector>
#include <string>

namespace tui {

// ============================================================================
// Conversion Functions: Native Types -> TUI Display Types
// ============================================================================

// Convert types::MarketData OHLC to tui::Candle
Candle ConvertCandle(const types::MarketData& market_data);

// Convert types::Position to tui::Position (display format)
// Note: tui::Position is simplified for display, may need additional data
Position ConvertPosition(const types::Position& pos, const std::string& name = "");

// Convert types::Order to tui::Order (display format)
// Note: tui::Order is just a display event, not the full order structure
Order ConvertOrderEvent(const types::Order& order);

// Convert types::AccountInfo to tui::AccountMetrics
AccountMetrics ConvertAccountMetrics(const types::AccountInfo& account);

// Convert multiple native positions to TUI positions
std::vector<Position> ConvertPositions(const std::vector<types::Position>& positions);

// Convert multiple native orders to TUI order events
std::vector<Order> ConvertOrderEvents(const std::vector<types::Order>& orders);

// ============================================================================
// Helper: Format order status as display text
// ============================================================================

std::string FormatOrderText(const types::Order& order);

} // namespace tui
