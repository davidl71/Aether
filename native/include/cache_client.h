// cache_client.h - Abstract cache interface for market data and other cached values.
//
// Design: Small C++ cache abstraction with in-memory and optional memcached backends.
//
// Usage: Use get/set/del for JSON-serialized values. TTL in seconds.
// To add Memcached implementation: link libmemcached, implement CacheClient interface.

#pragma once

#include <memory>
#include <optional>
#include <string>

namespace platform {

enum class CacheBackend {
  kInMemory,
  kMemcached,
};

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

std::unique_ptr<CacheClient> create_cache(
    CacheBackend backend,
    const std::string& host,
    int port,
    const std::string& prefix,
    int default_ttl);

} // namespace platform
