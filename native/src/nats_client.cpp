// nats_client.cpp - NATS message queue client wrapper implementation
#include "nats_client.h"
#include <ctime>
#include <iomanip>
#include <spdlog/spdlog.h>
#include <sstream>
#include <uuid/uuid.h>

// NATS C client headers (only if ENABLE_NATS is defined)
#ifdef ENABLE_NATS
#include "nats.h"
#define NATS_AVAILABLE 1
#else
#define NATS_AVAILABLE 0
#endif

// Protobuf support for binary NATS messages (when both NATS and proto are enabled)
#if defined(ENABLE_NATS) && defined(ENABLE_PROTO)
#include "messages.pb.h"
#include <google/protobuf/timestamp.pb.h>
#define PROTO_NATS 1
#else
#define PROTO_NATS 0
#endif

namespace nats {

// ============================================================================
// Implementation Details
// ============================================================================

struct NatsClientImpl {
#ifdef ENABLE_NATS
  natsConnection *conn = nullptr;
#endif
  std::string url;
  bool connected = false;

  NatsClientImpl(const std::string &server_url) : url(server_url) {}

  ~NatsClientImpl() { disconnect_impl(); }

  bool connect_impl() {
#if NATS_AVAILABLE
    natsOptions *opts = nullptr;
    natsStatus status = natsOptions_Create(&opts);
    if (status != NATS_OK) {
      spdlog::warn("Failed to create NATS options: {}",
                   natsStatus_GetText(status));
      return false;
    }

    status = natsOptions_SetURL(opts, url.c_str());
    if (status != NATS_OK) {
      spdlog::warn("Failed to set NATS URL: {}", natsStatus_GetText(status));
      natsOptions_Destroy(opts);
      return false;
    }

    status = natsConnection_Connect(&conn, opts);
    natsOptions_Destroy(opts);

    if (status == NATS_OK) {
      connected = true;
      spdlog::info("Connected to NATS at {}", url);
      return true;
    } else {
      spdlog::warn("Failed to connect to NATS at {}: {}", url,
                   natsStatus_GetText(status));
      connected = false;
      return false;
    }
#else
    spdlog::debug("NATS integration disabled (ENABLE_NATS not set)");
    return false;
#endif
  }

  void disconnect_impl() {
#if NATS_AVAILABLE
    if (conn) {
      natsConnection_Close(conn);
      natsConnection_Destroy(conn);
      conn = nullptr;
    }
#endif
    connected = false;
  }

  bool publish_impl(const std::string &topic, const std::string &message) {
#if NATS_AVAILABLE
    if (!connected || !conn) {
      return false;
    }

    natsStatus status =
        natsConnection_PublishString(conn, topic.c_str(), message.c_str());
    if (status == NATS_OK) {
      spdlog::trace("Published to NATS topic {}: {}", topic, message);
      return true;
    } else {
      spdlog::warn("Failed to publish to NATS topic {}: {}", topic,
                   natsStatus_GetText(status));
      return false;
    }
#else
    (void)topic;
    (void)message;
    return false;
#endif
  }

  // Binary publish for protobuf messages
  bool publish_bytes_impl(const std::string &topic, const std::string &data) {
#if NATS_AVAILABLE
    if (!connected || !conn) {
      return false;
    }

    natsStatus status = natsConnection_Publish(
        conn, topic.c_str(),
        reinterpret_cast<const void *>(data.data()),
        static_cast<int>(data.size()));
    if (status == NATS_OK) {
      spdlog::trace("Published {} bytes to NATS topic {}", data.size(), topic);
      return true;
    } else {
      spdlog::warn("Failed to publish bytes to NATS topic {}: {}", topic,
                   natsStatus_GetText(status));
      return false;
    }
#else
    (void)topic;
    (void)data;
    return false;
#endif
  }
};

// ============================================================================
// Public Interface
// ============================================================================

NatsClient::NatsClient(const std::string &url)
    : pimpl_(std::make_unique<NatsClientImpl>(url)) {}

NatsClient::~NatsClient() = default;

NatsClient::NatsClient(NatsClient &&) noexcept = default;
NatsClient &NatsClient::operator=(NatsClient &&) noexcept = default;

bool NatsClient::connect() { return pimpl_->connect_impl(); }

void NatsClient::disconnect() { pimpl_->disconnect_impl(); }

bool NatsClient::is_connected() const { return pimpl_->connected; }

// Helper: Generate UUID string
static std::string generate_uuid() {
  uuid_t uuid;
  char uuid_str[37];
  uuid_generate(uuid);
  uuid_unparse_lower(uuid, uuid_str);
  return std::string(uuid_str);
}

// Helper: Get current time as protobuf Timestamp seconds + nanos
static std::pair<int64_t, int32_t> get_proto_timestamp() {
  auto now = std::chrono::system_clock::now();
  auto epoch = now.time_since_epoch();
  auto secs = std::chrono::duration_cast<std::chrono::seconds>(epoch);
  auto nanos = std::chrono::duration_cast<std::chrono::nanoseconds>(epoch) -
               std::chrono::duration_cast<std::chrono::nanoseconds>(secs);
  return {static_cast<int64_t>(secs.count()),
          static_cast<int32_t>(nanos.count())};
}

#if PROTO_NATS
// Helper: Build and serialize a NatsEnvelope wrapping an inner proto message.
// Returns the serialized bytes, or empty string on failure.
// Canonical pattern: inner.SerializeToString(&payload) -> envelope.set_payload(payload) -> envelope.SerializeToString(&out) -> publish_raw(out).
template <typename InnerMsg>
static std::string build_envelope(const std::string &message_type,
                                  const InnerMsg &inner) {
  std::string payload;
  if (!inner.SerializeToString(&payload)) {
    spdlog::warn("Failed to serialize inner protobuf message for type {}",
                 message_type);
    return {};
  }

  ::ib::platform::v1::NatsEnvelope envelope;
  envelope.set_id(generate_uuid());
  envelope.set_source("cpp-tws-client");
  envelope.set_message_type(message_type);
  envelope.set_payload(payload);

  auto [secs, nanos] = get_proto_timestamp();
  auto *ts = envelope.mutable_timestamp();
  ts->set_seconds(secs);
  ts->set_nanos(nanos);

  std::string out;
  if (!envelope.SerializeToString(&out)) {
    spdlog::warn("Failed to serialize NatsEnvelope for type {}", message_type);
    return {};
  }
  return out;
}
#endif  // PROTO_NATS

// Canonical NATS publish path (example): build MarketDataEvent, wrap in NatsEnvelope,
// SerializeToString, publish via publish_bytes_impl (binary; equivalent to publish_raw).
bool NatsClient::publish_market_data(const std::string &symbol, double bid,
                                     double ask, const std::string &timestamp) {
  if (!is_connected()) {
    return false;
  }

  std::string topic = "market-data.tick." + symbol;

#if PROTO_NATS
  ::ib::platform::v1::MarketDataEvent event;
  event.set_symbol(symbol);
  event.set_bid(bid);
  event.set_ask(ask);

  auto [secs, nanos] = get_proto_timestamp();
  auto *ts = event.mutable_timestamp();
  ts->set_seconds(secs);
  ts->set_nanos(nanos);

  std::string data = build_envelope("MarketDataEvent", event);
  if (data.empty()) {
    return false;
  }
  return pimpl_->publish_bytes_impl(topic, data);
#else
  // JSON fallback when protobuf not available
  std::ostringstream payload;
  payload << "{"
          << "\"symbol\":\"" << symbol << "\","
          << "\"bid\":" << bid << ","
          << "\"ask\":" << ask << ","
          << "\"timestamp\":\"" << timestamp << "\""
          << "}";

  std::ostringstream msg;
  msg << "{\"source\":\"cpp-tws-client\",\"type\":\"MarketDataTick\","
      << "\"payload\":" << payload.str() << "}";
  return pimpl_->publish_impl(topic, msg.str());
#endif
}

bool NatsClient::publish_strategy_signal(const std::string &symbol,
                                         double price,
                                         const std::string &signal_type) {
  if (!is_connected()) {
    return false;
  }

  std::string topic = "strategy.signal." + symbol;

#if PROTO_NATS
  ::ib::platform::v1::StrategySignal signal;
  signal.set_symbol(symbol);
  signal.set_price(price);

  auto [secs, nanos] = get_proto_timestamp();
  auto *ts = signal.mutable_timestamp();
  ts->set_seconds(secs);
  ts->set_nanos(nanos);

  std::string data = build_envelope("StrategySignal", signal);
  if (data.empty()) {
    return false;
  }
  return pimpl_->publish_bytes_impl(topic, data);
#else
  std::ostringstream payload;
  payload << "{"
          << "\"symbol\":\"" << symbol << "\","
          << "\"price\":" << price << ","
          << "\"signal_type\":\"" << signal_type << "\""
          << "}";

  std::ostringstream msg;
  msg << "{\"source\":\"cpp-tws-client\",\"type\":\"StrategySignal\","
      << "\"payload\":" << payload.str() << "}";
  return pimpl_->publish_impl(topic, msg.str());
#endif
}

bool NatsClient::publish_strategy_decision(const std::string &symbol,
                                           int quantity,
                                           const std::string &side, double mark,
                                           const std::string &decision_type) {
  if (!is_connected()) {
    return false;
  }

  std::string topic = "strategy.decision." + symbol;

#if PROTO_NATS
  ::ib::platform::v1::StrategyDecision decision;
  decision.set_symbol(symbol);
  decision.set_quantity(static_cast<int32_t>(quantity));
  decision.set_side(side);
  decision.set_mark(mark);

  auto [secs, nanos] = get_proto_timestamp();
  auto *ts = decision.mutable_created_at();
  ts->set_seconds(secs);
  ts->set_nanos(nanos);

  std::string data = build_envelope("StrategyDecision", decision);
  if (data.empty()) {
    return false;
  }
  return pimpl_->publish_bytes_impl(topic, data);
#else
  std::ostringstream payload;
  payload << "{"
          << "\"symbol\":\"" << symbol << "\","
          << "\"quantity\":" << quantity << ","
          << "\"side\":\"" << side << "\","
          << "\"mark\":" << mark << ","
          << "\"decision_type\":\"" << decision_type << "\""
          << "}";

  std::ostringstream msg;
  msg << "{\"source\":\"cpp-tws-client\",\"type\":\"StrategyDecision\","
      << "\"payload\":" << payload.str() << "}";
  return pimpl_->publish_impl(topic, msg.str());
#endif
}

} // namespace nats
