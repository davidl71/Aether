// proto_adapter.h - Convert between native types and protobuf DTOs at API boundaries.
#pragma once

#include "config_manager.h"
#include "types.h"
#include <string>

// Forward declare proto types to avoid pulling in full generated header when not needed.
namespace ib {
namespace platform {
namespace v1 {
class OptionContract;
class BoxSpreadLeg;
class StrategyParams;
class RiskConfig;
}  // namespace v1
}  // namespace platform
}  // namespace ib

namespace proto_adapter {

// types::OptionContract <-> ib::platform::v1::OptionContract
void to_proto(const types::OptionContract& from, ::ib::platform::v1::OptionContract* out);
void from_proto(const ::ib::platform::v1::OptionContract& from, types::OptionContract* out);

// types::BoxSpreadLeg <-> ib::platform::v1::BoxSpreadLeg
void to_proto(const types::BoxSpreadLeg& from, ::ib::platform::v1::BoxSpreadLeg* out);
void from_proto(const ::ib::platform::v1::BoxSpreadLeg& from, types::BoxSpreadLeg* out);

// config::StrategyParams <-> ib::platform::v1::StrategyParams (commissions not in proto)
void to_proto(const config::StrategyParams& from, ::ib::platform::v1::StrategyParams* out);
void from_proto(const ::ib::platform::v1::StrategyParams& from, config::StrategyParams* out);

// config::RiskConfig <-> ib::platform::v1::RiskConfig
void to_proto(const config::RiskConfig& from, ::ib::platform::v1::RiskConfig* out);
void from_proto(const ::ib::platform::v1::RiskConfig& from, config::RiskConfig* out);

// Serialize types::BoxSpreadLeg to protobuf binary; return empty string on error.
std::string box_spread_leg_to_proto_bytes(const types::BoxSpreadLeg& leg);
// Deserialize protobuf binary to types::BoxSpreadLeg; return false on error.
bool proto_bytes_to_box_spread_leg(const std::string& bytes, types::BoxSpreadLeg* out);

}  // namespace proto_adapter
