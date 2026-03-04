// tws_error_codes.h - IB TWS error code guidance lookup tables
#pragma once

#include <string>
#include <unordered_map>
#include <utility>

namespace tws {

extern const std::unordered_map<int, std::string> kIbErrorGuidance;

extern const std::pair<const char*, const char*> kErrorPhraseGuidance[];
extern const size_t kErrorPhraseGuidanceCount;

} // namespace tws
