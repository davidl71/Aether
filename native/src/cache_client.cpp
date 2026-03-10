#include "cache_client.h"

#include <chrono>
#include <mutex>
#include <unordered_map>

namespace platform {

// ---------------------------------------------------------------------------
// InMemoryCache (always available, no deps)
// ---------------------------------------------------------------------------

struct InMemoryCache::Impl {
  struct Entry {
    std::string value;
    std::chrono::steady_clock::time_point expires;
    bool has_ttl;
  };

  std::mutex mu;
  std::unordered_map<std::string, Entry> store;
};

InMemoryCache::InMemoryCache() : impl_(std::make_unique<Impl>()) {}
InMemoryCache::~InMemoryCache() = default;
InMemoryCache::InMemoryCache(InMemoryCache &&) noexcept = default;
InMemoryCache &InMemoryCache::operator=(InMemoryCache &&) noexcept = default;

std::optional<std::string> InMemoryCache::get(const std::string &key) {
  std::lock_guard<std::mutex> lock(impl_->mu);
  auto it = impl_->store.find(key);
  if (it == impl_->store.end())
    return std::nullopt;
  if (it->second.has_ttl &&
      std::chrono::steady_clock::now() > it->second.expires) {
    impl_->store.erase(it);
    return std::nullopt;
  }
  return it->second.value;
}

void InMemoryCache::set(const std::string &key, const std::string &value,
                        int ttl_seconds) {
  std::lock_guard<std::mutex> lock(impl_->mu);
  Impl::Entry entry;
  entry.value = value;
  entry.has_ttl = (ttl_seconds > 0);
  if (entry.has_ttl)
    entry.expires =
        std::chrono::steady_clock::now() + std::chrono::seconds(ttl_seconds);
  impl_->store[key] = std::move(entry);
}

void InMemoryCache::del(const std::string &key) {
  std::lock_guard<std::mutex> lock(impl_->mu);
  impl_->store.erase(key);
}

bool InMemoryCache::is_healthy() { return true; }

} // namespace platform
