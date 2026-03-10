// cache_client.h - Abstract cache interface for market data and other cached values.
//
// Design: Small C++ cache abstraction for optional injected caches.
//
// Usage: Use get/set/del for string-serialized values. TTL in seconds.

#pragma once

#include <memory>
#include <optional>
#include <string>

namespace platform {

class CacheClient {
public:
  virtual ~CacheClient() = default;
  virtual std::optional<std::string> get(const std::string& key) = 0;
  virtual void set(const std::string& key, const std::string& value, int ttl_seconds = 300) = 0;
  virtual void del(const std::string& key) = 0;
  virtual bool is_healthy() = 0;
};

class InMemoryCache : public CacheClient {
public:
  InMemoryCache();
  ~InMemoryCache() override;
  InMemoryCache(InMemoryCache&&) noexcept;
  InMemoryCache& operator=(InMemoryCache&&) noexcept;
  std::optional<std::string> get(const std::string& key) override;
  void set(const std::string& key, const std::string& value, int ttl_seconds = 300) override;
  void del(const std::string& key) override;
  bool is_healthy() override;

private:
  struct Impl;
  std::unique_ptr<Impl> impl_;
};

} // namespace platform
