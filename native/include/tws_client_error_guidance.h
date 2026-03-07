// tws_client_error_guidance.h - Error guidance data for EWrapper error() callback
#pragma once

#include <string>
#include <unordered_map>
#include <utility>

namespace tws {
namespace detail {

extern const std::unordered_map<int, std::string> kIbErrorGuidance;
extern const std::pair<const char *, const char *> kErrorPhraseGuidance[];

}  // namespace detail
}  // namespace tws
