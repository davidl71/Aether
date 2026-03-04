// http_client.cpp - Simple HTTP client wrapper using libcurl
#include "brokers/http_client.h"
#include <curl/curl.h>
#include <spdlog/spdlog.h>
#include <sstream>

namespace brokers {

// ============================================================================
// CURL Write Callback
// ============================================================================

static size_t WriteCallback(void *contents, size_t size, size_t nmemb,
                            void *userp) {
  size_t realsize = size * nmemb;
  std::string *data = static_cast<std::string *>(userp);
  data->append(static_cast<char *>(contents), realsize);
  return realsize;
}

// ============================================================================
// HttpClient Implementation
// ============================================================================

HttpClient::HttpClient() : curl_handle_(nullptr), last_response_code_(0) {
  curl_handle_ = curl_easy_init();
  if (!curl_handle_) {
    spdlog::error("Failed to initialize curl");
    last_error_ = "Failed to initialize curl";
  }
}

HttpClient::~HttpClient() {
  if (curl_handle_) {
    curl_easy_cleanup(curl_handle_);
  }
}

std::string
HttpClient::request(const std::string &method, const std::string &url,
                    const std::map<std::string, std::string> &headers,
                    const std::string &body) {
  if (!curl_handle_) {
    last_error_ = "CURL not initialized";
    return "";
  }

  // Reset curl handle
  curl_easy_reset(curl_handle_);

  // Set URL
  curl_easy_setopt(curl_handle_, CURLOPT_URL, url.c_str());

  // Set write callback
  std::string response_data;
  curl_easy_setopt(curl_handle_, CURLOPT_WRITEFUNCTION, WriteCallback);
  curl_easy_setopt(curl_handle_, CURLOPT_WRITEDATA, &response_data);

  // Build header list
  struct curl_slist *header_list = nullptr;
  for (const auto &[key, value] : headers) {
    std::string header = key + ": " + value;
    header_list = curl_slist_append(header_list, header.c_str());
  }
  if (header_list) {
    curl_easy_setopt(curl_handle_, CURLOPT_HTTPHEADER, header_list);
  }

  // Set method and body
  if (method == "POST") {
    curl_easy_setopt(curl_handle_, CURLOPT_POSTFIELDS, body.c_str());
    curl_easy_setopt(curl_handle_, CURLOPT_POSTFIELDSIZE, body.length());
  } else if (method == "PUT") {
    curl_easy_setopt(curl_handle_, CURLOPT_CUSTOMREQUEST, "PUT");
    curl_easy_setopt(curl_handle_, CURLOPT_POSTFIELDS, body.c_str());
    curl_easy_setopt(curl_handle_, CURLOPT_POSTFIELDSIZE, body.length());
  } else if (method == "DELETE") {
    curl_easy_setopt(curl_handle_, CURLOPT_CUSTOMREQUEST, "DELETE");
  }

  // Set timeouts
  curl_easy_setopt(curl_handle_, CURLOPT_TIMEOUT, 30L);
  curl_easy_setopt(curl_handle_, CURLOPT_CONNECTTIMEOUT, 10L);

  // Follow redirects
  curl_easy_setopt(curl_handle_, CURLOPT_FOLLOWLOCATION, 1L);

  // Perform request
  CURLcode res = curl_easy_perform(curl_handle_);

  // Clean up headers
  if (header_list) {
    curl_slist_free_all(header_list);
  }

  // Get response code
  curl_easy_getinfo(curl_handle_, CURLINFO_RESPONSE_CODE, &last_response_code_);

  if (res != CURLE_OK) {
    last_error_ = curl_easy_strerror(res);
    spdlog::error("CURL error: {}", last_error_);
    return "";
  }

  return response_data;
}

std::string HttpClient::get(const std::string &url,
                            const std::map<std::string, std::string> &headers) {
  return request("GET", url, headers);
}

std::string
HttpClient::post(const std::string &url, const std::string &body,
                 const std::map<std::string, std::string> &headers) {
  return request("POST", url, headers, body);
}

std::string HttpClient::put(const std::string &url, const std::string &body,
                            const std::map<std::string, std::string> &headers) {
  return request("PUT", url, headers, body);
}

std::string HttpClient::del(const std::string &url,
                            const std::map<std::string, std::string> &headers) {
  return request("DELETE", url, headers);
}

} // namespace brokers
