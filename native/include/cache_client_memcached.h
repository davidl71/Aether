// cache_client_memcached.h - Memcached backend factory (only when ENABLE_MEMCACHED)
#pragma once

#if defined(ENABLE_MEMCACHED) && ENABLE_MEMCACHED

#include "cache_client.h"
#include <memory>
#include <string>

namespace platform {

std::unique_ptr<CacheClient> create_memcached_cache(const std::string& host,
                                                    int port,
                                                    const std::string& prefix,
                                                    int default_ttl);
}

#endif
