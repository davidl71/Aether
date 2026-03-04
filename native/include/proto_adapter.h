// proto_adapter.h - Convert between native types and protobuf DTOs at API boundaries.
#pragma once

#include "types.h"
#include <string>

// Forward declare proto types to avoid pulling in full generated header when not needed.
namespace ib {
namespace platform {
namespace v1 {
class OptionContract;
class BoxSpreadLeg;
class StrategyParams;
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

// Serialize types::BoxSpreadLeg to protobuf binary; return empty string on error.
std::string box_spread_leg_to_proto_bytes(const types::BoxSpreadLeg& leg);
// Deserialize protobuf binary to types::BoxSpreadLeg; return false on error.
bool proto_bytes_to_box_spread_leg(const std::string& bytes, types::BoxSpreadLeg* out);

}  // namespace proto_adapter
