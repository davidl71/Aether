// cache_client_memcached.cpp - Memcached backend for CacheClient (optional,
// requires libmemcached)
#if defined(ENABLE_MEMCACHED) && ENABLE_MEMCACHED

#include "cache_client.h"

#include <libmemcached/memcached.h>

#include <chrono>
#include <mutex>
#include <sstream>
#include <stdexcept>

namespace platform {

class MemcachedClient : public CacheClient {
public:
  MemcachedClient(const std::string &host, int port, const std::string &prefix,
                  int default_ttl);
  ~MemcachedClient() override;

  std::optional<std::string> get(const std::string &key) override;
  void set(const std::string &key, const std::string &value,
           int ttl_seconds = 300) override;
  void del(const std::string &key) override;
  bool is_healthy() override;

private:
  std::string make_key(const std::string &key) const;
  memcached_st *memcached_;
  std::string prefix_;
  int default_ttl_;
  mutable std::mutex mutex_;
};

MemcachedClient::MemcachedClient(const std::string &host, int port,
                                 const std::string &prefix, int default_ttl)
    : prefix_(prefix), default_ttl_(default_ttl_ > 0 ? default_ttl_ : 300) {
  memcached_ = memcached_create(nullptr);
  if (!memcached_)
    throw std::runtime_error("memcached_create failed");

  memcached_server_st *servers =
      memcached_server_list_append(nullptr, host.c_str(), port, nullptr);
  if (!servers) {
    memcached_free(memcached_);
    memcached_ = nullptr;
    throw std::runtime_error("memcached_server_list_append failed");
  }
  memcached_return_t ret = memcached_server_push(memcached_, servers);
  memcached_server_list_free(servers);
  if (ret != MEMCACHED_SUCCESS) {
    memcached_free(memcached_);
    memcached_ = nullptr;
    throw std::runtime_error("memcached_server_push failed");
  }
}

MemcachedClient::~MemcachedClient() {
  std::lock_guard<std::mutex> lock(mutex_);
  if (memcached_) {
    memcached_free(memcached_);
    memcached_ = nullptr;
  }
}

std::string MemcachedClient::make_key(const std::string &key) const {
  return prefix_ + key;
}

std::optional<std::string> MemcachedClient::get(const std::string &key) {
  std::lock_guard<std::mutex> lock(mutex_);
  if (!memcached_)
    return std::nullopt;

  std::string full_key = make_key(key);
  size_t value_len = 0;
  uint32_t flags = 0;
  memcached_return_t error = MEMCACHED_SUCCESS;
  char *value = memcached_get(memcached_, full_key.c_str(), full_key.size(),
                              &value_len, &flags, &error);
  if (error != MEMCACHED_SUCCESS || !value)
    return std::nullopt;

  std::string result(value, value_len);
  free(value);
  return result;
}

void MemcachedClient::set(const std::string &key, const std::string &value,
                          int ttl_seconds) {
  std::lock_guard<std::mutex> lock(mutex_);
  if (!memcached_)
    return;

  int ttl = ttl_seconds > 0 ? ttl_seconds : default_ttl_;
  std::string full_key = make_key(key);
  memcached_set(memcached_, full_key.c_str(), full_key.size(), value.data(),
                value.size(), static_cast<time_t>(ttl), 0);
}

void MemcachedClient::del(const std::string &key) {
  std::lock_guard<std::mutex> lock(mutex_);
  if (!memcached_)
    return;

  std::string full_key = make_key(key);
  memcached_delete(memcached_, full_key.c_str(), full_key.size(), 0);
}

bool MemcachedClient::is_healthy() {
  std::lock_guard<std::mutex> lock(mutex_);
  if (!memcached_)
    return false;
  memcached_return_t ret = memcached_version(memcached_);
  return (ret == MEMCACHED_SUCCESS);
}

std::unique_ptr<CacheClient> create_memcached_cache(const std::string &host,
                                                    int port,
                                                    const std::string &prefix,
                                                    int default_ttl) {
  return std::make_unique<MemcachedClient>(host, port, prefix, default_ttl);
}

} // namespace platform

#endif // ENABLE_MEMCACHED
