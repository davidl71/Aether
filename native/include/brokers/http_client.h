// http_client.h - Simple HTTP client wrapper using libcurl
#pragma once

#include <string>
#include <map>
#include <memory>

namespace brokers {

/**
 * Simple HTTP client using libcurl
 * Used by broker adapters for REST API communication
 */
class HttpClient {
public:
    /**
     * Constructor
     */
    HttpClient();

    /**
     * Destructor
     */
    ~HttpClient();

    // Disable copy
    HttpClient(const HttpClient&) = delete;
    HttpClient& operator=(const HttpClient&) = delete;

    /**
     * Make HTTP request
     * @param method HTTP method (GET, POST, PUT, DELETE)
     * @param url Full URL
     * @param headers HTTP headers
     * @param body Request body (for POST/PUT)
     * @return Response body
     */
    std::string request(
        const std::string& method,
        const std::string& url,
        const std::map<std::string, std::string>& headers = {},
        const std::string& body = ""
    );

    /**
     * GET request
     */
    std::string get(const std::string& url, const std::map<std::string, std::string>& headers = {});

    /**
     * POST request
     */
    std::string post(const std::string& url, const std::string& body, const std::map<std::string, std::string>& headers = {});

    /**
     * PUT request
     */
    std::string put(const std::string& url, const std::string& body, const std::map<std::string, std::string>& headers = {});

    /**
     * DELETE request
     */
    std::string del(const std::string& url, const std::map<std::string, std::string>& headers = {});

    /**
     * Get last HTTP response code
     */
    long get_last_response_code() const { return last_response_code_; }

    /**
     * Get last error message
     */
    std::string get_last_error() const { return last_error_; }

private:
    void* curl_handle_;  // CURL* handle
    long last_response_code_;
    std::string last_error_;
};

} // namespace brokers
