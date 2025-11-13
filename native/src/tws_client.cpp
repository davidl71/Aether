// tws_client.cpp - TWS API Client implementation with full EWrapper integration
#include "tws_client.h"
#include "rate_limiter.h"
#include <spdlog/spdlog.h>

// TWS API headers
#include "EWrapper.h"
#include "DefaultEWrapper.h"
#include "EClientSocket.h"
#include "EReaderOSSignal.h"
#include "EReader.h"
#include "Contract.h"
#include "Order.h"
#include "OrderState.h"
#include "Execution.h"
#include "OrderCancel.h"

#include <algorithm>
#include <thread>
#include <mutex>
#include <condition_variable>
#include <atomic>
#include <chrono>
#include <cfloat>
#include <ctime>
#include <unordered_map>
#include <vector>
#include <string>
#include <cstring>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <unistd.h>
#include <fcntl.h>
#include <errno.h>

// NOTE FOR AUTOMATION AGENTS:
// `TWSClient` is the primary integration point with Interactive Brokers' TWS API.
// It wraps the asynchronous `EClientSocket`/`EWrapper` interface, translating IB
// callbacks into thread-safe data structures consumed by higher layers. When
// extending behaviour, focus on the `Impl` class and prefer reusing the helper
// converters and callback registration patterns already established below.

namespace tws {

namespace {

// ============================================================================
// Port Checking Helper
// ============================================================================

/**
 * Check if a port is open/listening on the given host
 * @param host Hostname or IP address
 * @param port Port number to check
 * @param timeout_ms Timeout in milliseconds
 * @return true if port is open, false otherwise
 */
bool is_port_open(const std::string& host, int port, int timeout_ms = 1000) {
    int sock = socket(AF_INET, SOCK_STREAM, 0);
    if (sock < 0) {
        return false;
    }

    // Set socket to non-blocking
    int flags = fcntl(sock, F_GETFL, 0);
    fcntl(sock, F_SETFL, flags | O_NONBLOCK);

    struct sockaddr_in server_addr;
    memset(&server_addr, 0, sizeof(server_addr));
    server_addr.sin_family = AF_INET;
    server_addr.sin_port = htons(port);

    if (inet_pton(AF_INET, host.c_str(), &server_addr.sin_addr) <= 0) {
        close(sock);
        return false;
    }

    // Attempt connection
    int result = connect(sock, (struct sockaddr*)&server_addr, sizeof(server_addr));

    if (result == 0) {
        // Connected immediately
        close(sock);
        return true;
    }

    if (errno == EINPROGRESS) {
        // Connection in progress, wait with select
        fd_set write_fds;
        FD_ZERO(&write_fds);
        FD_SET(sock, &write_fds);

        struct timeval timeout;
        timeout.tv_sec = timeout_ms / 1000;
        timeout.tv_usec = (timeout_ms % 1000) * 1000;

        int select_result = select(sock + 1, nullptr, &write_fds, nullptr, &timeout);
        if (select_result > 0) {
            int so_error;
            socklen_t len = sizeof(so_error);
            if (getsockopt(sock, SOL_SOCKET, SO_ERROR, &so_error, &len) == 0 && so_error == 0) {
                close(sock);
                return true;
            }
        }
    }

    close(sock);
    return false;
}

/**
 * Get default ports to try based on configured port
 * Returns configured port first, then all standard ports (TWS and IB Gateway, paper and live)
 * This ensures we find the correct port even if user configured wrong type (paper vs live)
 */
std::vector<int> get_port_candidates(int configured_port) {
    std::vector<int> candidates;

    // Standard ports in priority order:
    // 1. Configured port (if it's a standard port, it will be tried first anyway)
    // 2. TWS Paper (7497) - most common for testing
    // 3. TWS Live (7496)
    // 4. IB Gateway Paper (4002)
    // 5. IB Gateway Live (4001)

    // Start with configured port
    candidates.push_back(configured_port);

    // Add all standard ports, avoiding duplicates
    std::vector<int> standard_ports = {7497, 7496, 4002, 4001};
    for (int port : standard_ports) {
        if (port != configured_port) {
            candidates.push_back(port);
        }
    }

    return candidates;
}

const std::unordered_map<int, std::string> kIbErrorGuidance = {
    // Connection errors (500-599)
    {502, "Connection rejected. Enable 'ActiveX and Socket Clients' in TWS Settings > API > Settings. Verify IP is trusted (127.0.0.1) and port is correct."},

    // System messages (1100-1999)
    {1100, "Connection lost. Check TWS/IB Gateway and internet connection. Auto-reconnect will be attempted if enabled."},
    {1101, "Market data connection restored. Confirm subscriptions are active."},
    {1102, "Order routing connection restored."},

    // Contract/Order errors (100-299)
    {162, "Order rejected - Invalid order ticket. Check order parameters, contract ID, and trading permissions."},
    {200, "Invalid contract definition. Verify symbol, expiry, right, strike, and exchange values."},
    {201, "Order rejected due to contract error. Confirm contract fields before resubmitting."},
    {202, "Order rejected by IB. Check order parameters, size limits, and account permissions."},
    {203, "Order rejected - Order cannot be executed. Check market hours, order type, and account permissions."},
    {204, "Order rejected - Order size exceeds position limit. Reduce order size or check account limits."},
    {205, "Order rejected - Order price is outside acceptable range. Adjust limit price."},

    // Validation errors (300-399)
    {321, "Server validation failed. Review price increments, exchange routing, and TIF."},
    {322, "Order rejected - Duplicate order ID. Use unique order IDs for each order."},
    {323, "Order rejected - Order cannot be cancelled. Order may already be filled or cancelled."},

    // Market data errors (350-399)
    {354, "No market data permissions. Ensure your IB account has the required data subscriptions."},
    {355, "Market data request failed. Check contract details and market data subscriptions."},

    // Market data farm messages (2100-2199)
    {2104, "Market data farm connection restored."},
    {2106, "Market data farm is connecting. Expect delayed quotes until established."},
    {2107, "Market data farm connection failed. Check IB network status dashboard."},
    {2108, "Market data farm disconnected. Quotes will pause until reconnection."},
    {2109, "Order routing to IB server is OK."},

    // Additional common errors
    {399, "Order rejected - Order would exceed maximum position size. Check account limits."},
    {400, "Order rejected - Order would create a position that violates account restrictions."},
    {401, "Order rejected - Order type not allowed for this contract. Check order type compatibility."},
    {402, "Order rejected - Order would exceed maximum order value. Reduce order size or price."},
};

const std::pair<const char*, const char*> kErrorPhraseGuidance[] = {
    {
        "code card authentication",
        "IB triggered code card authentication. Approve the 2FA challenge in IBKR Mobile or disable code card auth.",
    },
    {
        "two factor authentication request timed out",
        "Two-factor approval timed out. Re-initiate login and approve promptly on your IBKR Mobile device.",
    },
    {
        "No market data permissions",
        "IB refused market data. Purchase/enable required market data subscriptions or switch data provider.",
    },
    {
        "No security definition has been found",
        "Contract not recognized. Double-check ticker, expiration, strike, right, and exchange.",
    },
};

}  // namespace

// ============================================================================
// TWSClient::Impl - Full TWS API Implementation with DefaultEWrapper
// ============================================================================

class TWSClient::Impl : public DefaultEWrapper {
public:
    explicit Impl(const config::TWSConfig& config)
        : config_(config)
        , signal_(2000) // 2 second timeout
        , client_(this, &signal_)
        , next_order_id_(0)
        , connected_(false)
        , next_request_id_(1000)
        , state_(ConnectionState::Disconnected)
        , reconnect_attempts_(0)
        , last_heartbeat_(std::chrono::steady_clock::now())
        , rate_limiter_(RateLimiterConfig{}) {
        // Enable async connection mode for non-blocking connections
        client_.asyncEConnect(true);

        // Initialize callback tracking
        connection_callbacks_received_.connectAck = false;
        connection_callbacks_received_.managedAccounts = false;
    }

    ~Impl() {
        disconnect();
    }

    // ========================================================================
    // Connection Management
    // ========================================================================

    bool connect() {
        state_ = ConnectionState::Connecting;

        // Get port candidates (TWS first, then IB Gateway)
        std::vector<int> port_candidates = get_port_candidates(config_.port);

        // Check which ports are open in parallel
        std::vector<int> open_ports;
        std::vector<int> closed_ports;
        std::vector<std::thread> check_threads;
        std::mutex ports_mutex;

        // Build port list string for debug logging
        std::string port_list_str;
        for (size_t i = 0; i < port_candidates.size(); ++i) {
            if (i > 0) port_list_str += ", ";
            port_list_str += std::to_string(port_candidates[i]);
        }
        spdlog::debug("Checking {} ports in parallel: {}", port_candidates.size(), port_list_str);

        auto check_start = std::chrono::steady_clock::now();

        // Launch parallel port checks for all candidates
        for (int port : port_candidates) {
            check_threads.emplace_back([&, port]() {
                bool is_open = is_port_open(config_.host, port, 1000);
                std::lock_guard<std::mutex> lock(ports_mutex);
                if (is_open) {
                    open_ports.push_back(port);
                    spdlog::info("Port {} is open on {}", port, config_.host);
                } else {
                    closed_ports.push_back(port);
                }
            });
        }

        // Wait for all checks to complete
        for (auto& thread : check_threads) {
            thread.join();
        }

        auto check_duration = std::chrono::duration_cast<std::chrono::milliseconds>(
            std::chrono::steady_clock::now() - check_start).count();

        // Log summary of port check results
        if (!open_ports.empty()) {
            std::string open_str;
            for (size_t i = 0; i < open_ports.size(); ++i) {
                if (i > 0) open_str += ", ";
                open_str += std::to_string(open_ports[i]);
            }
            spdlog::info("Port check complete ({}ms): {} open port(s) found: {}",
                        check_duration, open_ports.size(), open_str);
        } else {
            spdlog::warn("Port check complete ({}ms): No open ports found", check_duration);
        }

        // Helper to determine if a port is for paper trading
        auto is_paper_port = [](int port) -> bool {
            return port == 7497 || port == 4002;
        };

        // Helper to determine if a port is for live trading
        auto is_live_port = [](int port) -> bool {
            return port == 7496 || port == 4001;
        };

        // Helper to get port type string
        auto get_port_type_string = [](int port) -> std::string {
            if (port == 7497) return "TWS Paper Trading";
            if (port == 7496) return "TWS Live Trading";
            if (port == 4002) return "IB Gateway Paper Trading";
            if (port == 4001) return "IB Gateway Live Trading";
            return "Custom";
        };

        // Determine which port to use based on priority
        // Priority: configured port > TWS paper > TWS live > IB Gateway paper > IB Gateway live
        int port_to_use = config_.port;
        bool found_open_port = false;
        bool paper_live_mismatch = false;

        // Check if configured port is in open_ports
        if (std::find(open_ports.begin(), open_ports.end(), config_.port) != open_ports.end()) {
            port_to_use = config_.port;
            found_open_port = true;
        } else if (!open_ports.empty()) {
            // Check for paper/live mismatch
            bool configured_is_paper = is_paper_port(config_.port);
            bool configured_is_live = is_live_port(config_.port);

            // Find what type of ports are actually open
            bool has_paper_port = false;
            bool has_live_port = false;
            for (int port : open_ports) {
                if (is_paper_port(port)) has_paper_port = true;
                if (is_live_port(port)) has_live_port = true;
            }

            // Detect mismatch
            if (configured_is_paper && !has_paper_port && has_live_port) {
                paper_live_mismatch = true;
                spdlog::warn("⚠️  Paper/Live Trading Mismatch Detected:");
                spdlog::warn("   Configured: Paper Trading (port {})", config_.port);
                spdlog::warn("   Available: Only Live Trading ports are open");
                spdlog::warn("   Action: Will use available Live Trading port");
            } else if (configured_is_live && !has_live_port && has_paper_port) {
                paper_live_mismatch = true;
                spdlog::warn("⚠️  Paper/Live Trading Mismatch Detected:");
                spdlog::warn("   Configured: Live Trading (port {})", config_.port);
                spdlog::warn("   Available: Only Paper Trading ports are open");
                spdlog::warn("   Action: Will use available Paper Trading port");
                spdlog::warn("   ⚠️  WARNING: This will use PAPER TRADING instead of LIVE!");
            }

            // Use first open port from priority list
            for (int priority_port : port_candidates) {
                if (std::find(open_ports.begin(), open_ports.end(), priority_port) != open_ports.end()) {
                    port_to_use = priority_port;
                    found_open_port = true;
                    break;
                }
            }
        }

        std::string port_type = get_port_type_string(port_to_use);
        std::string configured_type = get_port_type_string(config_.port);

        if (found_open_port) {
            if (port_to_use != config_.port) {
                if (paper_live_mismatch) {
                    spdlog::warn("Switching from {} (port {}) to {} (port {})",
                                configured_type, config_.port, port_type, port_to_use);
                } else {
                    spdlog::warn("Configured port {} is not open, using {} port {} instead",
                                config_.port, port_type, port_to_use);
                }
            } else {
                spdlog::info("Using configured port {} ({})", port_to_use, port_type);
            }
        }

        if (!found_open_port) {
            std::string ports_str;
            for (size_t i = 0; i < port_candidates.size(); ++i) {
                if (i > 0) ports_str += ", ";
                ports_str += std::to_string(port_candidates[i]);
            }
            spdlog::error("No open ports found on {}. Checked ports: {}",
                         config_.host, ports_str);
            spdlog::error("Please ensure TWS or IB Gateway is running and API is enabled");
            state_ = ConnectionState::Error;
            return false;
        }

        spdlog::info("Connecting to {}:{}...", config_.host, port_to_use);

        // Try connecting to each open port until one succeeds
        bool connected = false;
        int successful_port = port_to_use;

        for (int port : open_ports) {
            // Disconnect any previous attempt
            if (client_.isConnected()) {
                client_.eDisconnect();
                std::this_thread::sleep_for(std::chrono::milliseconds(100));
            }

            spdlog::info("Attempting connection to {}:{}...", config_.host, port);

            // Connect to TWS/IB Gateway
            spdlog::info("Calling eConnect() for {}:{} (client_id={})...", config_.host, port, config_.client_id);

            // Reset connection state before attempting
            connected_ = false;
        state_ = ConnectionState::Connecting;
            last_error_code_ = 0;
            last_error_message_.clear();

            // Reset callback tracking for this connection attempt
            connection_callbacks_received_.connectAck = false;
            connection_callbacks_received_.managedAccounts = false;

        bool success = client_.eConnect(
            config_.host.c_str(),
                port,
            config_.client_id
        );

            spdlog::info("eConnect() returned: {}", success ? "true" : "false");

        if (!success) {
                spdlog::warn("eConnect() returned false for {}:{}, trying next port...", config_.host, port);
                continue;
            }

            // Give a brief moment for any immediate errors to come through
            std::this_thread::sleep_for(std::chrono::milliseconds(200));

            // Check for immediate connection errors (like 502)
            {
                std::lock_guard<std::mutex> lock(error_mutex_);
                if (last_error_code_ == 502) {
                    spdlog::warn("Connection rejected by TWS/Gateway (error 502) for {}:{}", config_.host, port);
                    spdlog::warn("Error: {}", last_error_message_);
            client_.eDisconnect();
                    continue;
                }
            }

            // Check if socket is actually connected
            bool is_connected = client_.isConnected();
            spdlog::info("isConnected() returned: {}", is_connected ? "true" : "false");

            if (!is_connected) {
                spdlog::warn("Socket not connected after eConnect() for {}:{}, trying next port...", config_.host, port);
                spdlog::warn("This may indicate the port is open but TWS/Gateway rejected the connection");
                spdlog::warn("Check TWS/Gateway for error messages or authentication prompts");
                client_.eDisconnect();
                continue;
            }

            spdlog::info("Socket connected, starting message reader thread...");

            // CRITICAL: Start reader thread BEFORE waiting for acknowledgment
            // The reader thread processes messages from TWS, including nextValidId
            // which sets connected_ = true. Without it, we'll wait forever.
            start_reader_thread();

            spdlog::info("Waiting for TWS acknowledgment (timeout: {}ms)...",
                         config_.connection_timeout_ms);

            // Wait for connection acknowledgment with progress logging
            auto wait_start = std::chrono::steady_clock::now();
            bool connection_acknowledged = wait_for_connection_with_progress(config_.connection_timeout_ms);

            if (connection_acknowledged) {
                connected = true;
                successful_port = port;

                // Determine port type for logging
                std::string port_type = "";
                if (port == 7497) port_type = "TWS Paper Trading";
                else if (port == 7496) port_type = "TWS Live Trading";
                else if (port == 4002) port_type = "IB Gateway Paper Trading";
                else if (port == 4001) port_type = "IB Gateway Live Trading";
                else port_type = "Custom";

                if (port != config_.port) {
                    spdlog::warn("Configured port {} failed, successfully connected to {} port {}",
                                config_.port, port_type, port);
                } else {
                    spdlog::info("Successfully connected to {} port {}", port_type, port);
                }
                break;
            } else {
                spdlog::warn("Connection timeout on {}:{}, trying next port...", config_.host, port);
                // Stop reader thread if it was started
                if (reader_thread_ && reader_thread_->joinable()) {
                    connected_ = false;
                    signal_.issueSignal();
                    reader_thread_->join();
                    reader_thread_.reset();
                }
                client_.eDisconnect();
            }
        }

        if (!connected) {
            spdlog::error("Failed to connect to any open port on {}", config_.host);
            if (!open_ports.empty()) {
                std::string tried_ports;
                for (size_t i = 0; i < open_ports.size(); ++i) {
                    if (i > 0) tried_ports += ", ";
                    tried_ports += std::to_string(open_ports[i]);
                }
                spdlog::error("Attempted connection to open port(s): {}", tried_ports);
                spdlog::error("Possible causes:");
                spdlog::error("  - API not enabled in TWS/Gateway settings");
                spdlog::error("  - Authentication required (check TWS/Gateway for prompts)");
                spdlog::error("  - Client ID conflict (try a different client_id)");
                spdlog::error("  - Firewall blocking connection");
                spdlog::error("  - Wrong port type (paper vs live trading mismatch)");
            }
            spdlog::error("Port reference: TWS Paper=7497, TWS Live=7496, IB Gateway Paper=4002, IB Gateway Live=4001");
            state_ = ConnectionState::Error;
            return false;
        }

        port_to_use = successful_port;

        state_ = ConnectionState::Connected;
        spdlog::info("✓ Connected to {}:{}", config_.host, port_to_use);
        return true;
    }

    void disconnect() {
        if (connected_) {
            spdlog::info("Disconnecting from TWS...");

            // Stop health monitoring
            stop_health_monitoring();

            // Stop reader thread
            if (reader_thread_ && reader_thread_->joinable()) {
                connected_ = false;
                signal_.issueSignal();  // Wake up the thread
                reader_thread_->join();
            }

            client_.eDisconnect();
            state_ = ConnectionState::Disconnected;
            spdlog::info("✓ Disconnected");
        }
    }

    bool is_connected() const {
        return connected_ && client_.isConnected();
    }

    ConnectionState get_connection_state() const {
        return state_;
    }

    void process_messages(int timeout_ms) {
        // Messages are processed by the reader thread
        // This method can be used for additional processing if needed
        std::this_thread::sleep_for(std::chrono::milliseconds(timeout_ms));
    }

    // ========================================================================
    // EWrapper Callbacks - Connection
    // ========================================================================

    void connectAck() override {
        try {
            spdlog::info("✓ connectAck received - Socket connection established, server version received");
            spdlog::info("Connection sequence: connectAck → managedAccounts → nextValidId");
            spdlog::info("Status: Waiting for managedAccounts and nextValidId...");

            // Track that we received connectAck (for diagnostics)
            connection_callbacks_received_.connectAck = true;
            connection_callbacks_received_.connectAck_time = std::chrono::steady_clock::now();

            // In async mode, connectAck is called immediately after socket connection
            // We still need to wait for nextValidId to confirm full connection

            // Reset reconnection attempts on successful connection
            reconnect_attempts_ = 0;
            last_message_time_ = std::chrono::steady_clock::now();

            // Note: Don't start health monitoring yet - wait for nextValidId
            // Request next valid order ID (this will trigger nextValidId callback)
            // According to IB API Quick Reference, reqIds(-1) requests the next valid order ID
            client_.reqIds(-1);
            spdlog::debug("Requested next valid order ID via reqIds(-1), waiting for nextValidId callback...");
        } catch (const std::exception& e) {
            spdlog::error("Exception in connectAck: {}", e.what());
        } catch (...) {
            spdlog::error("Unknown exception in connectAck");
        }
    }

    void connectionClosed() override {
        try {
            spdlog::warn("Connection closed by TWS");
            connected_ = false;
            state_ = ConnectionState::Disconnected;

            // Call error callback
            if (error_callback_) {
                error_callback_(1100, "Connection closed by TWS");
            }

            // Auto-reconnect if enabled with exponential backoff
            if (config_.auto_reconnect) {
                attempt_reconnect_with_backoff();
            }
        } catch (const std::exception& e) {
            spdlog::error("Exception in connectionClosed: {}", e.what());
        } catch (...) {
            spdlog::error("Unknown exception in connectionClosed");
        }
    }

    // managedAccounts is called with the list of accounts after connection
    // This is an early indicator of successful connection (happens before nextValidId)
    // According to IB API Quick Reference, this is called after connectAck
    void managedAccounts(const std::string& accountsList) override {
        try {
            spdlog::info("✓ managedAccounts received: {} - Connection progressing", accountsList);

            // Track that we received managedAccounts (for diagnostics)
            connection_callbacks_received_.managedAccounts = true;
            connection_callbacks_received_.managedAccounts_time = std::chrono::steady_clock::now();

            auto elapsed = std::chrono::duration_cast<std::chrono::milliseconds>(
                connection_callbacks_received_.managedAccounts_time -
                connection_callbacks_received_.connectAck_time).count();

            // Show progress indicator
            spdlog::info("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            spdlog::info("Connection Progress: [████████░░░░░░░░░░] 50%");
            spdlog::info("  ✓ Step 1/3: connectAck received (socket connected)");
            spdlog::info("  ✓ Step 2/3: managedAccounts received ({}ms after connectAck)", elapsed);
            spdlog::info("  ⏳ Step 3/3: Waiting for nextValidId... (this may take a few seconds)");
            spdlog::info("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

            // Parse account list if needed
            if (!accountsList.empty()) {
                spdlog::info("Account(s) available: {}", accountsList);
                // Store account info for later use
                // Note: This happens before nextValidId, so connection is progressing
            } else {
                spdlog::warn("⚠️  managedAccounts received but account list is empty");
            }
        } catch (const std::exception& e) {
            spdlog::error("Exception in managedAccounts: {} (accountsList: {})", e.what(), accountsList);
        } catch (...) {
            spdlog::error("Unknown exception in managedAccounts");
        }
    }

    void nextValidId(OrderId orderId) override {
        try {
            // Calculate total connection time
            auto total_elapsed = std::chrono::duration_cast<std::chrono::milliseconds>(
                std::chrono::steady_clock::now() - connection_callbacks_received_.connectAck_time).count();

            spdlog::info("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            spdlog::info("Connection Progress: [████████████████████████████████████████████████] 100%");
            spdlog::info("  ✓ Step 1/3: connectAck received");
            spdlog::info("  ✓ Step 2/3: managedAccounts received");
            spdlog::info("  ✓ Step 3/3: nextValidId received: {} (connection complete!)", orderId);
            spdlog::info("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            spdlog::info("✓ Connection fully established in {}ms", total_elapsed);

            next_order_id_ = orderId;

            std::lock_guard<std::mutex> lock(connection_mutex_);
            connected_ = true;

            // Check if this is a reconnection
            bool is_reconnection = (reconnect_attempts_.load() > 0);

            if (is_reconnection) {
                spdlog::info("Reconnection detected ({} attempts). Synchronizing state with TWS...",
                           reconnect_attempts_.load());
            }

            // Start health monitoring now that we're fully connected
            start_health_monitoring();

            // According to EWrapper best practices, request open orders to sync state
            // This is important for order recovery after reconnection
            // TWS will call openOrder() for each open order, then openOrderEnd()
            spdlog::debug("Requesting open orders to sync state...");
            client_.reqAllOpenOrders();

            // Sync positions and account after reconnection
            if (is_reconnection) {
                spdlog::debug("Synchronizing positions and account data after reconnection...");
                client_.reqPositions();
                client_.reqAccountUpdates(true, "");
                reconnect_attempts_ = 0; // Reset after successful sync
                spdlog::info("State synchronization complete after reconnection");
            }

            connection_cv_.notify_all();
            spdlog::info("✓ Connection fully established and ready");
        } catch (const std::exception& e) {
            spdlog::error("Exception in nextValidId: {} (orderId: {})", e.what(), orderId);
        } catch (...) {
            spdlog::error("Unknown exception in nextValidId (orderId: {})", orderId);
        }
    }

    // ========================================================================
    // EWrapper Callbacks - Market Data
    // ========================================================================

    void tickPrice(TickerId tickerId, TickType field,
                   double price, const TickAttrib& attribs) override {
        try {
        spdlog::trace("tickPrice: id={}, field={}, price={}", tickerId, field, price);

        std::lock_guard<std::mutex> lock(data_mutex_);
        auto& market_data = market_data_[tickerId];

        switch (field) {
            case BID:
                market_data.bid = price;
                break;
            case ASK:
                market_data.ask = price;
                break;
            case LAST:
                market_data.last = price;
                break;
            case HIGH:
                market_data.high = price;
                break;
            case LOW:
                market_data.low = price;
                break;
            case CLOSE:
                market_data.close = price;
                break;
            case OPEN:
                market_data.open = price;
                break;
            default:
                break;
        }

        market_data.timestamp = std::chrono::system_clock::now();

        // Notify callback if registered
        if (market_data_callbacks_.count(tickerId)) {
            market_data_callbacks_[tickerId](market_data);
            }

            // Fulfill promise if waiting for synchronous request
            if (market_data_promises_.count(tickerId)) {
                market_data_promises_[tickerId]->set_value(market_data);
                market_data_promises_.erase(tickerId);
            }
        } catch (const std::exception& e) {
            spdlog::error("Exception in tickPrice(tickerId={}, field={}): {}", tickerId, field, e.what());
        } catch (...) {
            spdlog::error("Unknown exception in tickPrice(tickerId={}, field={})", tickerId, field);
        }
    }

    void tickSize(TickerId tickerId, TickType field, Decimal size) override {
        spdlog::trace("tickSize: id={}, field={}, size={}", tickerId, field, size);

        std::lock_guard<std::mutex> lock(data_mutex_);
        auto& market_data = market_data_[tickerId];

        switch (field) {
            case BID_SIZE:
                market_data.bid_size = static_cast<int>(size);
                break;
            case ASK_SIZE:
                market_data.ask_size = static_cast<int>(size);
                break;
            case LAST_SIZE:
                market_data.last_size = static_cast<int>(size);
                break;
            case VOLUME:
                market_data.volume = static_cast<double>(size);
                break;
            default:
                break;
        }
    }

    void tickOptionComputation(TickerId tickerId, TickType tickType,
                               int tickAttrib, double impliedVol,
                               double delta, double optPrice,
                               double pvDividend, double gamma,
                               double vega, double theta,
                               double undPrice) override {
        spdlog::trace("tickOptionComputation: id={}, IV={}, delta={}",
                     tickerId, impliedVol, delta);

        std::lock_guard<std::mutex> lock(data_mutex_);
        auto& market_data = market_data_[tickerId];

        if (impliedVol >= 0 && impliedVol != DBL_MAX) {
            market_data.implied_volatility = impliedVol;
        }
        if (delta != DBL_MAX) {
            market_data.delta = delta;
        }
        if (gamma != DBL_MAX) {
            market_data.gamma = gamma;
        }
        if (vega != DBL_MAX) {
            market_data.vega = vega;
        }
        if (theta != DBL_MAX) {
            market_data.theta = theta;
        }
    }

    // ========================================================================
    // EWrapper Callbacks - Orders
    // ========================================================================

    void orderStatus(OrderId orderId, const std::string& status,
                    Decimal filled, Decimal remaining,
                    double avgFillPrice, long long permId, int parentId,
                    double lastFillPrice, int clientId,
                    const std::string& whyHeld, double mktCapPrice) override {
        try {
        spdlog::info("Order #{} status: {}, filled={}, remaining={}, avgPrice={}",
                    orderId, status, filled, remaining, avgFillPrice);

        std::lock_guard<std::mutex> lock(order_mutex_);

        if (orders_.count(orderId)) {
            auto& order = orders_[orderId];

            // Update status
            if (status == "PreSubmitted" || status == "Submitted") {
                order.status = types::OrderStatus::Submitted;
            } else if (status == "Filled") {
                order.status = types::OrderStatus::Filled;
            } else if (status == "Cancelled") {
                order.status = types::OrderStatus::Cancelled;
            } else if (status == "Inactive" || status == "ApiCancelled") {
                order.status = types::OrderStatus::Rejected;
            } else if (filled > 0 && remaining > 0) {
                order.status = types::OrderStatus::PartiallyFilled;
            }

            order.filled_quantity = static_cast<int>(filled);
            order.avg_fill_price = avgFillPrice;
            order.last_update = std::chrono::system_clock::now();

            // Notify callback
            if (order_status_callback_) {
                order_status_callback_(order);
            }
            }
        } catch (const std::exception& e) {
            spdlog::error("Exception in orderStatus(orderId={}, status={}): {}", orderId, status, e.what());
        } catch (...) {
            spdlog::error("Unknown exception in orderStatus(orderId={}, status={})", orderId, status);
        }
    }

    void openOrder(OrderId orderId, const Contract& contract,
                   const Order& order, const OrderState& orderState) override {
        try {
            spdlog::debug("Open order: #{}, {}, {}, status={}",
                         orderId, contract.symbol, order.action, orderState.status);

        std::lock_guard<std::mutex> lock(order_mutex_);

            // openOrder is called for ALL open orders, including ones we didn't place
            // This is important for order recovery and syncing with TWS state

            // Update order if it exists in our tracking
        if (orders_.count(orderId)) {
            auto& our_order = orders_[orderId];

                // Update status based on orderState
                // Note: OrderState contains status string, but filled quantity and avg price
                // come from orderStatus() callback, not openOrder()
                if (orderState.status == "PreSubmitted" || orderState.status == "Submitted") {
                    our_order.status = types::OrderStatus::Submitted;
                } else if (orderState.status == "Filled") {
                our_order.status = types::OrderStatus::Filled;
                    // Filled quantity and price come from orderStatus() callback
            } else if (orderState.status == "Cancelled") {
                our_order.status = types::OrderStatus::Cancelled;
                } else if (orderState.status == "PartiallyFilled") {
                    our_order.status = types::OrderStatus::PartiallyFilled;
                    // Filled quantity and price come from orderStatus() callback
                }

                our_order.last_update = std::chrono::system_clock::now();

                // Notify callback
                if (order_status_callback_) {
                    order_status_callback_(our_order);
                }
            } else {
                // Order not in our tracking - might be from previous session or another client
                spdlog::debug("Received openOrder for order #{} not in our tracking (may be from previous session)", orderId);
            }
        } catch (const std::exception& e) {
            spdlog::error("Exception in openOrder(orderId={}, symbol={}): {}", orderId, contract.symbol, e.what());
        } catch (...) {
            spdlog::error("Unknown exception in openOrder(orderId={}, symbol={})", orderId, contract.symbol);
        }
    }

    void openOrderEnd() override {
        try {
            spdlog::debug("Open orders sync complete");
            // All open orders have been sent via openOrder() callbacks
            // This is useful for order recovery after reconnection
        } catch (const std::exception& e) {
            spdlog::error("Exception in openOrderEnd: {}", e.what());
        } catch (...) {
            spdlog::error("Unknown exception in openOrderEnd");
        }
    }

    void execDetails(int reqId, const Contract& contract,
                    const Execution& execution) override {
        try {
            spdlog::info("Execution: order={}, shares={}, price={}, time={}",
                        execution.orderId, execution.shares, execution.price, execution.time);

            std::lock_guard<std::mutex> lock(order_mutex_);

            if (orders_.count(execution.orderId)) {
                auto& order = orders_[execution.orderId];
                order.filled_quantity += static_cast<int>(execution.shares);
                order.avg_fill_price = execution.price;
                order.last_update = std::chrono::system_clock::now();
            }
        } catch (const std::exception& e) {
            spdlog::error("Exception in execDetails: {} (reqId: {}, orderId: {})",
                         e.what(), reqId, execution.orderId);
        } catch (...) {
            spdlog::error("Unknown exception in execDetails (reqId: {}, orderId: {})",
                         reqId, execution.orderId);
        }
    }

    void execDetailsEnd(int reqId) override {
        try {
            spdlog::debug("Execution details end for reqId={}", reqId);
        } catch (const std::exception& e) {
            spdlog::error("Exception in execDetailsEnd: {} (reqId: {})", e.what(), reqId);
        } catch (...) {
            spdlog::error("Unknown exception in execDetailsEnd (reqId: {})", reqId);
        }
    }

    // ========================================================================
    // EWrapper Callbacks - Account & Positions
    // ========================================================================

    void position(const std::string& account, const Contract& contract,
                 Decimal position, double avgCost) override {
        try {
            spdlog::debug("Position: {} {} @ {} (account={})",
                         position, contract.symbol, avgCost, account);

            std::lock_guard<std::mutex> lock(position_mutex_);

            types::Position pos;
            pos.contract = convert_from_tws_contract(contract);
            pos.quantity = static_cast<int>(position);
            pos.avg_price = avgCost;
            pos.current_price = avgCost;  // Will be updated by market data

            // Update or add position
            auto existing = std::find_if(
                positions_.begin(),
                positions_.end(),
                [&pos](const types::Position& candidate) {
                    return candidate.contract.symbol == pos.contract.symbol &&
                           candidate.contract.expiry == pos.contract.expiry &&
                           candidate.contract.strike == pos.contract.strike &&
                           candidate.contract.type == pos.contract.type;
                }
            );

            if (existing != positions_.end()) {
                *existing = pos;
            } else if (position != 0) {
                positions_.push_back(pos);
            }
        } catch (const std::exception& e) {
            spdlog::error("Exception in position: {} (account: {}, symbol: {})",
                         e.what(), account, contract.symbol);
        } catch (...) {
            spdlog::error("Unknown exception in position (account: {}, symbol: {})",
                         account, contract.symbol);
        }
    }

    void positionEnd() override {
        try {
            spdlog::debug("Position updates complete");

            // Fulfill promise if waiting for synchronous request
            if (positions_request_pending_.load()) {
                std::lock_guard<std::mutex> lock(position_mutex_);
                if (positions_promise_) {
                    positions_promise_->set_value(positions_);
                    positions_promise_.reset();
                    positions_request_pending_ = false;
                }
            }
        } catch (const std::exception& e) {
            spdlog::error("Exception in positionEnd: {}", e.what());
        } catch (...) {
            spdlog::error("Unknown exception in positionEnd");
        }
    }

    void updateAccountValue(const std::string& key, const std::string& val,
                           const std::string& currency,
                           const std::string& accountName) override {
        try {
            spdlog::trace("Account update: {}={} ({}, {})", key, val, currency, accountName);

            std::lock_guard<std::mutex> lock(account_mutex_);

            try {
                if (key == "NetLiquidation" && currency == "USD") {
                    account_info_.net_liquidation = std::stod(val);
                } else if (key == "TotalCashBalance" && currency == "USD") {
                    account_info_.cash_balance = std::stod(val);
                } else if (key == "BuyingPower") {
                    account_info_.buying_power = std::stod(val);
                } else if (key == "GrossPositionValue" && currency == "USD") {
                    account_info_.gross_position_value = std::stod(val);
                } else if (key == "UnrealizedPnL" && currency == "USD") {
                    account_info_.unrealized_pnl = std::stod(val);
                } else if (key == "RealizedPnL" && currency == "USD") {
                    account_info_.realized_pnl = std::stod(val);
                }
            } catch (const std::exception& e) {
                spdlog::warn("Failed to parse account value: {}={} ({})", key, val, e.what());
            }

            account_info_.account_id = accountName;
            account_info_.timestamp = std::chrono::system_clock::now();
        } catch (const std::exception& e) {
            spdlog::error("Exception in updateAccountValue: {} (key: {}, account: {})",
                         e.what(), key, accountName);
        } catch (...) {
            spdlog::error("Unknown exception in updateAccountValue (key: {}, account: {})",
                         key, accountName);
        }
    }

    void updateAccountTime(const std::string& timeStamp) override {
        spdlog::trace("Account time: {}", timeStamp);
    }

    void accountDownloadEnd(const std::string& accountName) override {
        spdlog::debug("Account download complete: {}", accountName);

        // Fulfill promise if waiting for synchronous request
        if (account_request_pending_.load()) {
            std::lock_guard<std::mutex> lock(account_mutex_);
            if (account_promise_) {
                account_promise_->set_value(account_info_);
                account_promise_.reset();
                account_request_pending_ = false;
            }
        }
    }

    void updatePortfolio(const Contract& contract, Decimal position,
                        double marketPrice, double marketValue,
                        double averageCost, double unrealizedPNL,
                        double realizedPNL, const std::string& accountName) override {
        try {
            spdlog::debug("Portfolio update: {} position={}, value={}, PnL={}, avgCost={}",
                         contract.symbol, position, marketValue, unrealizedPNL, averageCost);

            // updatePortfolio is called for each position in the portfolio
            // This is important for tracking current positions and P&L
            // Per EWrapper best practices, this provides real-time position updates

            std::lock_guard<std::mutex> lock(position_mutex_);

        auto it = std::find_if(
            positions_.begin(),
            positions_.end(),
            [&contract](const types::Position& candidate) {
                return candidate.contract.symbol == contract.symbol &&
                       candidate.contract.expiry ==
                           contract.lastTradeDateOrContractMonth &&
                       std::abs(candidate.contract.strike - contract.strike) < 0.01;
            }
        );

            if (it != positions_.end()) {
                // Update existing position with latest data
                it->current_price = marketPrice;
                it->quantity = static_cast<int>(position);
                it->avg_price = averageCost;
                // Note: unrealizedPNL and realizedPNL could be stored if needed
            } else if (position != 0) {
                // New position - add it (might be from previous session)
                types::Position pos;
                pos.contract = convert_from_tws_contract(contract);
                pos.quantity = static_cast<int>(position);
                pos.avg_price = averageCost;
                pos.current_price = marketPrice;
                positions_.push_back(pos);
                spdlog::debug("Added new position from updatePortfolio: {}", contract.symbol);
            }
        } catch (const std::exception& e) {
            spdlog::error("Exception in updatePortfolio: {} (symbol: {}, account: {})",
                         e.what(), contract.symbol, accountName);
        } catch (...) {
            spdlog::error("Unknown exception in updatePortfolio (symbol: {}, account: {})",
                         contract.symbol, accountName);
        }
    }

    // ========================================================================
    // EWrapper Callbacks - Error Handling
    // ========================================================================

    void error(int id, time_t errorTime, int errorCode, const std::string& errorString,
              const std::string& advancedOrderRejectJson) override {
        try {
            // Store error for connection attempt checking
            {
                std::lock_guard<std::mutex> lock(error_mutex_);
                last_error_code_ = errorCode;
                last_error_message_ = errorString;
            }

            std::vector<std::string> guidance_notes;

            // Check for connection failure errors (502 and other connection errors)
            if (errorCode == 502 || (errorCode >= 500 && errorCode < 600)) {
                spdlog::error("TWS connection error {}: {}", errorCode, errorString);
                connected_ = false;
                state_ = ConnectionState::Error;
                connection_cv_.notify_all(); // Wake up waiting connection attempt
            }

            // Check for authentication/authorization errors
            if (errorCode == 162 || errorCode == 200) {
                spdlog::error("TWS authentication/authorization error {}: {}", errorCode, errorString);
                spdlog::error("This usually means:");
                spdlog::error("  - TWS/Gateway is waiting for you to accept the connection");
                spdlog::error("  - Check TWS/Gateway window for a connection prompt");
                spdlog::error("  - Ensure 'Enable ActiveX and Socket Clients' is enabled in API settings");
                connected_ = false;
                state_ = ConnectionState::Error;
                connection_cv_.notify_all();
            }

            // Enhanced logging with context and guidance
            std::string guidance = "";
            if (auto it = kIbErrorGuidance.find(errorCode); it != kIbErrorGuidance.end()) {
                guidance = it->second;
            }

            // Build context string with order/request details if available
            std::string context = "";
            {
                // Check if ID matches an order ID
                std::lock_guard<std::mutex> lock(order_mutex_);
                if (orders_.count(id) > 0) {
                    const auto& order = orders_[id];
                    context = "Order #" + std::to_string(id) + ": " +
                             types::order_action_to_string(order.action) + " " +
                             std::to_string(order.quantity) + " " +
                             order.contract.to_string() + " @ " +
                             (order.limit_price > 0 ? std::to_string(order.limit_price) : "MKT");
                }
            }

            // Check if ID matches a market data request ID
            if (context.empty()) {
                std::lock_guard<std::mutex> lock(data_mutex_);
                if (market_data_.count(id) > 0 || market_data_callbacks_.count(id) > 0) {
                    context = "Market data request #" + std::to_string(id);
                }
            }

            // Add advanced order reject JSON context if available
            if (!advancedOrderRejectJson.empty() && advancedOrderRejectJson != "{}") {
                if (!context.empty()) {
                    context += " | ";
                }
                context += "Reject JSON: " + advancedOrderRejectJson;
            }

            // Log with appropriate level and context
            if (errorCode >= 2100 && errorCode < 3000) {
                // Informational messages
                if (!context.empty()) {
                    spdlog::info("[IB Info {}] ID: {} | {} | Context: {}", errorCode, id, errorString, context);
                } else {
                    spdlog::info("[IB Info {}] ID: {} | {}", errorCode, id, errorString);
                }
                if (!guidance.empty()) {
                    spdlog::info("  → {}", guidance);
                }
            } else if (errorCode >= 1100 && errorCode < 2000) {
                // System messages
                if (!context.empty()) {
                    spdlog::warn("[IB System {}] ID: {} | {} | Context: {}", errorCode, id, errorString, context);
                } else {
                    spdlog::warn("[IB System {}] ID: {} | {}", errorCode, id, errorString);
                }
                if (!guidance.empty()) {
                    spdlog::warn("  → {}", guidance);
                }

                // Connection lost - trigger reconnection if enabled
                if (errorCode == 1100) {
                    connected_ = false;
                    state_ = ConnectionState::Error;
                    spdlog::warn("Connection lost (error 1100). Auto-reconnect: {}",
                                config_.auto_reconnect ? "enabled" : "disabled");
                    if (config_.auto_reconnect) {
                        attempt_reconnect_with_backoff();
                    }
                }
                // Connection restored
                else if (errorCode == 1101 || errorCode == 1102) {
                    connected_ = true;
                    state_ = ConnectionState::Connected;
                    spdlog::info("Connection restored (error {}). Resuming operations.", errorCode);
                }
            } else {
                // Errors (< 1100)
                if (!context.empty()) {
                    spdlog::error("[IB Error {}] ID: {} | {} | Context: {}", errorCode, id, errorString, context);
                } else {
                    spdlog::error("[IB Error {}] ID: {} | {}", errorCode, id, errorString);
                }
                if (!guidance.empty()) {
                    spdlog::error("  → {}", guidance);
                }
            }

            // Add guidance to notes (already logged above, but keep for callback)
            if (!guidance.empty()) {
                guidance_notes.emplace_back(guidance);
            }

            for (const auto& phrase : kErrorPhraseGuidance) {
                if (errorString.find(phrase.first) != std::string::npos) {
                    spdlog::warn("Guidance: {}", phrase.second);
                    guidance_notes.emplace_back(phrase.second);
                }
            }

            if (error_callback_) {
                if (guidance_notes.empty()) {
                    error_callback_(errorCode, errorString);
                } else {
                    std::string enriched_message = errorString + " | Guidance: ";
                    for (size_t i = 0; i < guidance_notes.size(); ++i) {
                        if (i > 0) {
                            enriched_message += " | ";
                        }
                        enriched_message += guidance_notes[i];
                    }
                    error_callback_(errorCode, enriched_message);
                }
            }
        } catch (const std::exception& e) {
            spdlog::error("Exception in error callback: {} (errorCode: {}, id: {})",
                         e.what(), errorCode, id);
        } catch (...) {
            spdlog::error("Unknown exception in error callback (errorCode: {}, id: {})",
                         errorCode, id);
        }
    }

    // ========================================================================
    // Market Data Operations (Public Interface)
    // ========================================================================

    int request_market_data(const types::OptionContract& contract,
                           MarketDataCallback callback) {
        // Check rate limits
        if (!rate_limiter_.check_message_rate()) {
            spdlog::error("Rate limit exceeded: Cannot request market data for {}",
                         contract.to_string());
            return -1;  // Invalid request ID
        }

        int request_id = next_request_id_++;

        // Check market data line limit
        if (!rate_limiter_.can_start_market_data(request_id)) {
            spdlog::error("Market data line limit exceeded: Cannot subscribe to {}",
                         contract.to_string());
            return -1;  // Invalid request ID
        }

        // Convert to TWS Contract
        Contract tws_contract = convert_to_tws_contract(contract);

        // Register callback
        {
            std::lock_guard<std::mutex> lock(data_mutex_);
            market_data_callbacks_[request_id] = callback;
        }

        // Record message and start tracking
        rate_limiter_.record_message();
        rate_limiter_.start_market_data(request_id);

        // Request market data
        client_.reqMktData(
            request_id,           // Request ID
            tws_contract,         // Contract
            "",                   // Generic tick list
            false,                // Snapshot
            false,                // Regulatory snapshot
            TagValueListSPtr()    // Options
        );

        spdlog::debug("Requested market data for {} (id={})",
                     contract.to_string(), request_id);

        return request_id;
    }

    void cancel_market_data(int request_id) {
        // Check rate limits for cancel message
        if (rate_limiter_.is_enabled() && !rate_limiter_.check_message_rate()) {
            spdlog::warn("Rate limit exceeded: Delaying cancel_market_data for request {}", request_id);
            // Still proceed with cancel, but log the rate limit issue
        }

        client_.cancelMktData(request_id);

        // End market data tracking
        rate_limiter_.end_market_data(request_id);
        if (rate_limiter_.is_enabled()) {
            rate_limiter_.record_message();
        }

        std::lock_guard<std::mutex> lock(data_mutex_);
        market_data_callbacks_.erase(request_id);
        market_data_.erase(request_id);
        // Cancel any pending promise
        if (market_data_promises_.count(request_id)) {
            // Set promise to indicate cancellation (empty MarketData)
            types::MarketData empty_data;
            market_data_promises_[request_id]->set_value(empty_data);
            market_data_promises_.erase(request_id);
        }

        spdlog::debug("Cancelled market data request {}", request_id);
    }

    // Synchronous wrapper for market data request
    std::optional<types::MarketData> request_market_data_sync(
        const types::OptionContract& contract,
        int timeout_ms) {
        int request_id = next_request_id_++;

        // Create promise for synchronous wait
        auto promise = std::make_shared<std::promise<types::MarketData>>();
        auto future = promise->get_future();

        // Register promise
        {
            std::lock_guard<std::mutex> lock(data_mutex_);
            market_data_promises_[request_id] = promise;
        }

        // Convert to TWS Contract
        Contract tws_contract = convert_to_tws_contract(contract);

        // Check rate limits
        if (!rate_limiter_.check_message_rate()) {
            spdlog::error("Rate limit exceeded: Cannot request market data synchronously for {}",
                         contract.to_string());
            return std::nullopt;
        }

        // Check market data line limit
        if (!rate_limiter_.can_start_market_data(request_id)) {
            spdlog::error("Market data line limit exceeded: Cannot subscribe to {}",
                         contract.to_string());
            return std::nullopt;
        }

        // Record message and start tracking
        rate_limiter_.record_message();
        rate_limiter_.start_market_data(request_id);

        // Request market data directly (don't use request_market_data to avoid double request_id)
        client_.reqMktData(
            request_id,           // Request ID
            tws_contract,         // Contract
            "",                   // Generic tick list
            false,                // Snapshot
            false,                // Regulatory snapshot
            TagValueListSPtr()    // Options
        );

        spdlog::debug("Requested market data synchronously for {} (id={})",
                     contract.to_string(), request_id);

        // Wait for response with timeout
        if (future.wait_for(std::chrono::milliseconds(timeout_ms)) == std::future_status::ready) {
            auto data = future.get();
            // Check if we got valid data (not empty/cancelled)
            if (data.bid > 0 || data.ask > 0 || data.last > 0) {
                return data;
            }
        } else {
            // Timeout - cancel request and clean up
            spdlog::warn("Market data request {} timed out after {}ms", request_id, timeout_ms);
            cancel_market_data(request_id);
        }

        // Clean up promise if still pending
        {
            std::lock_guard<std::mutex> lock(data_mutex_);
            market_data_promises_.erase(request_id);
        }

        return std::nullopt;
    }

    std::vector<types::OptionContract> request_option_chain(
        const std::string& symbol,
        const std::string& expiry) {

        spdlog::debug("Requesting option chain for {} (expiry={})",
                     symbol, expiry.empty() ? "all" : expiry);

        // TODO: Implement option chain request using reqSecDefOptParams
        // This is complex and requires handling multiple callbacks
        return {};
    }

    // ========================================================================
    // Order Operations (Public Interface)
    // ========================================================================

    int place_order(const types::OptionContract& contract,
                   types::OrderAction action,
                   int quantity,
                   double limit_price,
                   types::TimeInForce tif) {
        // Check rate limits
        if (!rate_limiter_.check_message_rate()) {
            spdlog::error("Rate limit exceeded: Cannot place order for {}",
                         contract.to_string());
            return -1;  // Invalid order ID
        }

        int order_id = next_order_id_++;

        // Convert to TWS types
        Contract tws_contract = convert_to_tws_contract(contract);
        Order tws_order = create_tws_order(action, quantity, limit_price, tif);

        // Record message before placing order
        rate_limiter_.record_message();

        // Place order
        client_.placeOrder(order_id, tws_contract, tws_order);

        spdlog::info("Placed order #{}: {} {} {} @ {}",
                    order_id,
                    types::order_action_to_string(action),
                    quantity,
                    contract.to_string(),
                    limit_price > 0 ? std::to_string(limit_price) : "MKT");

        // Store order
        {
            std::lock_guard<std::mutex> lock(order_mutex_);
            types::Order our_order;
            our_order.order_id = order_id;
            our_order.contract = contract;
            our_order.action = action;
            our_order.quantity = quantity;
            our_order.limit_price = limit_price;
            our_order.tif = tif;
            our_order.status = types::OrderStatus::Submitted;
            our_order.submitted_time = std::chrono::system_clock::now();
            orders_[order_id] = our_order;
        }

        return order_id;
    }

    int place_combo_order(
        const std::vector<types::OptionContract>& contracts,
        const std::vector<types::OrderAction>& actions,
        const std::vector<int>& quantities,
        const std::vector<long>& contract_ids,
        const std::vector<double>& limit_prices,
        types::TimeInForce tif) {

        // Validate inputs
        if (contracts.size() != actions.size() || contracts.size() != quantities.size() ||
            contracts.size() != contract_ids.size() || contracts.size() != limit_prices.size()) {
            spdlog::error("Combo order: Mismatched vector sizes (contracts: {}, actions: {}, quantities: {}, contract_ids: {}, prices: {})",
                         contracts.size(), actions.size(), quantities.size(), contract_ids.size(), limit_prices.size());
            return -1;
        }

        if (contracts.empty()) {
            spdlog::error("Combo order: Cannot place empty combo");
            return -1;
        }

        // Check rate limits
        if (!rate_limiter_.check_message_rate()) {
            spdlog::error("Rate limit exceeded: Cannot place combo order");
            return -1;
        }

        int order_id = next_order_id_++;

        // Create BAG (combo) contract
        Contract combo_contract;
        combo_contract.secType = "BAG";
        combo_contract.symbol = contracts[0].symbol;  // Use underlying symbol
        combo_contract.currency = "USD";
        combo_contract.exchange = "SMART";

        // Create combo legs
        combo_contract.comboLegs = std::make_shared<Contract::ComboLegList>();
        for (size_t i = 0; i < contracts.size(); ++i) {
            auto leg = std::make_shared<ComboLeg>();
            leg->conId = contract_ids[i];
            leg->ratio = quantities[i];
            leg->action = (actions[i] == types::OrderAction::Buy) ? "BUY" : "SELL";
            leg->exchange = contracts[i].exchange.empty() ? "SMART" : contracts[i].exchange;
            leg->openClose = 0;  // SAME_POS - same as combo
            leg->shortSaleSlot = 0;
            leg->exemptCode = -1;

            combo_contract.comboLegs->push_back(leg);
        }

        // Create order with combo leg prices
        Order combo_order = create_tws_order(actions[0], 1, 0.0, tif);  // Base order
        combo_order.orderType = "LMT";  // Limit order for combo
        combo_order.allOrNone = true;   // All-or-nothing execution

        // Set order combo legs with prices
        combo_order.orderComboLegs = std::make_shared<Order::OrderComboLegList>();
        for (size_t i = 0; i < limit_prices.size(); ++i) {
            auto order_leg = std::make_shared<OrderComboLeg>();
            order_leg->price = limit_prices[i];
            combo_order.orderComboLegs->push_back(order_leg);
        }

        // Calculate total limit price (sum of all legs)
        double total_limit = 0.0;
        for (size_t i = 0; i < limit_prices.size(); ++i) {
            if (actions[i] == types::OrderAction::Buy) {
                total_limit += limit_prices[i] * quantities[i];
            } else {
                total_limit -= limit_prices[i] * quantities[i];
            }
        }
        combo_order.lmtPrice = total_limit;

        // Record message before placing order
        rate_limiter_.record_message();

        // Place combo order
        client_.placeOrder(order_id, combo_contract, combo_order);

        spdlog::info("Placed combo order #{}: {} legs, total limit price: {:.2f}",
                    order_id, contracts.size(), total_limit);

        // Store order (mark as combo)
        {
            std::lock_guard<std::mutex> lock(order_mutex_);
            types::Order our_order;
            our_order.order_id = order_id;
            our_order.contract = contracts[0];  // Store first contract as representative
            our_order.action = actions[0];
            our_order.quantity = 1;  // Combo order quantity is 1 (legs have individual quantities)
            our_order.limit_price = total_limit;
            our_order.tif = tif;
            our_order.status = types::OrderStatus::Submitted;
            our_order.submitted_time = std::chrono::system_clock::now();
            orders_[order_id] = our_order;
        }

        return order_id;
    }

    void cancel_order(int order_id) {
        // Check rate limits
        if (rate_limiter_.is_enabled() && !rate_limiter_.check_message_rate()) {
            spdlog::warn("Rate limit exceeded: Delaying cancel_order for order #{}", order_id);
            // Still proceed with cancel, but log the rate limit issue
        }

        OrderCancel orderCancel;
        client_.cancelOrder(order_id, orderCancel);

        if (rate_limiter_.is_enabled()) {
            rate_limiter_.record_message();
        }

        spdlog::info("Cancelling order #{}", order_id);

        std::lock_guard<std::mutex> lock(order_mutex_);
        if (orders_.count(order_id)) {
            orders_[order_id].status = types::OrderStatus::Cancelled;
            orders_[order_id].last_update = std::chrono::system_clock::now();
        }
    }

    void cancel_all_orders() {
        spdlog::info("Cancelling all orders");
        OrderCancel orderCancel;
        client_.reqGlobalCancel(orderCancel);

        std::lock_guard<std::mutex> lock(order_mutex_);
        for (auto& [id, order] : orders_) {
            if (order.is_active()) {
                order.status = types::OrderStatus::Cancelled;
                order.last_update = std::chrono::system_clock::now();
            }
        }
    }

    std::optional<types::Order> get_order(int order_id) const {
        std::lock_guard<std::mutex> lock(order_mutex_);
        auto it = orders_.find(order_id);
        if (it != orders_.end()) {
            return it->second;
        }
        return std::nullopt;
    }

    std::vector<types::Order> get_active_orders() const {
        std::lock_guard<std::mutex> lock(order_mutex_);
        std::vector<types::Order> active_orders;
        for (const auto& [id, order] : orders_) {
            if (order.is_active()) {
                active_orders.push_back(order);
            }
        }
        return active_orders;
    }

    // ========================================================================
    // Position Operations (Public Interface)
    // ========================================================================

    void request_positions(PositionCallback callback) {
        // Check rate limits
        if (!rate_limiter_.check_message_rate()) {
            spdlog::error("Rate limit exceeded: Cannot request positions");
            return;
        }

        spdlog::debug("Requesting positions");
        position_callback_ = callback;

        rate_limiter_.record_message();
        client_.reqPositions();
    }

    // Synchronous wrapper for positions request
    std::vector<types::Position> request_positions_sync(int timeout_ms) {
        // Create promise for synchronous wait
        auto promise = std::make_shared<std::promise<std::vector<types::Position>>>();
        auto future = promise->get_future();

        // Register promise
        {
            std::lock_guard<std::mutex> lock(position_mutex_);
            positions_promise_ = promise;
            positions_request_pending_ = true;
        }

        // Check rate limits
        if (!rate_limiter_.check_message_rate()) {
            spdlog::error("Rate limit exceeded: Cannot request positions synchronously");
            return {};
        }

        rate_limiter_.record_message();

        // Request positions (async)
        client_.reqPositions();

        // Wait for positionEnd() callback with timeout
        if (future.wait_for(std::chrono::milliseconds(timeout_ms)) == std::future_status::ready) {
            return future.get();
        } else {
            // Timeout
            spdlog::warn("Positions request timed out after {}ms", timeout_ms);
            {
                std::lock_guard<std::mutex> lock(position_mutex_);
                positions_promise_.reset();
                positions_request_pending_ = false;
            }
            // Return cached positions if available
            return get_positions();
        }
    }

    std::vector<types::Position> get_positions() const {
        std::lock_guard<std::mutex> lock(position_mutex_);
        return positions_;
    }

    std::optional<types::Position> get_position(
        const types::OptionContract& contract) const {

        std::lock_guard<std::mutex> lock(position_mutex_);
        auto it = std::find_if(
            positions_.begin(),
            positions_.end(),
            [&contract](const types::Position& pos) {
                return pos.contract.symbol == contract.symbol &&
                       pos.contract.expiry == contract.expiry &&
                       std::abs(pos.contract.strike - contract.strike) < 0.01 &&
                       pos.contract.type == contract.type;
            }
        );
        if (it != positions_.end()) {
            return *it;
        }
        return std::nullopt;
    }

    // ========================================================================
    // Account Operations (Public Interface)
    // ========================================================================

    void request_account_updates(AccountCallback callback) {
        // Check rate limits
        if (!rate_limiter_.check_message_rate()) {
            spdlog::error("Rate limit exceeded: Cannot request account updates");
            return;
        }

        spdlog::debug("Requesting account updates");
        account_callback_ = callback;

        rate_limiter_.record_message();
        client_.reqAccountUpdates(true, "");
    }

    // Synchronous wrapper for account info request
    std::optional<types::AccountInfo> request_account_info_sync(int timeout_ms) {
        // Create promise for synchronous wait
        auto promise = std::make_shared<std::promise<types::AccountInfo>>();
        auto future = promise->get_future();

        // Register promise
        {
            std::lock_guard<std::mutex> lock(account_mutex_);
            account_promise_ = promise;
            account_request_pending_ = true;
        }

        // Check rate limits
        if (!rate_limiter_.check_message_rate()) {
            spdlog::error("Rate limit exceeded: Cannot request account info synchronously");
            return std::nullopt;
        }

        rate_limiter_.record_message();

        // Request account updates (async)
        client_.reqAccountUpdates(true, "");

        // Wait for accountDownloadEnd() callback with timeout
        if (future.wait_for(std::chrono::milliseconds(timeout_ms)) == std::future_status::ready) {
            auto info = future.get();
            if (!info.account_id.empty()) {
                return info;
            }
        } else {
            // Timeout
            spdlog::warn("Account info request timed out after {}ms", timeout_ms);
            {
                std::lock_guard<std::mutex> lock(account_mutex_);
                account_promise_.reset();
                account_request_pending_ = false;
            }
        }

        // Return cached account info if available
        return get_account_info();
    }

    std::optional<types::AccountInfo> get_account_info() const {
        std::lock_guard<std::mutex> lock(account_mutex_);
        if (account_info_.account_id.empty()) {
            return std::nullopt;
        }
        return account_info_;
    }

    // ========================================================================
    // Callbacks
    // ========================================================================

    void set_order_status_callback(OrderStatusCallback callback) {
        order_status_callback_ = callback;
    }

    void set_error_callback(ErrorCallback callback) {
        error_callback_ = callback;
    }

    // ========================================================================
    // Helper Methods
    // ========================================================================

    int get_next_order_id() const {
        return next_order_id_;
    }

    bool is_market_open() const {
        // TODO: Implement proper market hours check
        auto now = std::chrono::system_clock::now();
        auto time_t_now = std::chrono::system_clock::to_time_t(now);
        auto tm_now = *std::localtime(&time_t_now);

        // Simple check: weekday and between 9:30 AM and 4:00 PM ET
        // This is approximate and doesn't account for holidays
        int hour = tm_now.tm_hour;
        int wday = tm_now.tm_wday;

        return (wday >= 1 && wday <= 5) && (hour >= 9 && hour < 16);
    }

    std::chrono::system_clock::time_point get_server_time() const {
        // TODO: Request and return actual TWS server time
        return std::chrono::system_clock::now();
    }

    // ========================================================================
    // Rate Limiting (Public Interface)
    // ========================================================================

    void enable_rate_limiting() {
        RateLimiterConfig config;
        config.enabled = true;
        rate_limiter_.configure(config);
    }

    void configure_rate_limiter(const RateLimiterConfig& config) {
        rate_limiter_.configure(config);
    }

    std::optional<RateLimiterStatus> get_rate_limiter_status() const {
        if (!rate_limiter_.is_enabled()) {
            return std::nullopt;
        }
        return rate_limiter_.get_status();
    }

    void cleanup_stale_rate_limiter_requests(std::chrono::seconds max_age) {
        rate_limiter_.cleanup_stale_requests(max_age);
    }

private:
    // ========================================================================
    // Helper Methods - Private
    // ========================================================================

    void start_reader_thread() {
        auto reader = std::make_unique<EReader>(&client_, &signal_);
        reader->start();

        reader_thread_ = std::make_unique<std::thread>([this, r = std::move(reader)]() mutable {
            while (connected_) {
                signal_.waitForSignal();
                if (!connected_) break;

                try {
                    r->processMsgs();
                    // Update last message time for health monitoring
                    last_message_time_ = std::chrono::steady_clock::now();
                } catch (const std::exception& e) {
                    spdlog::error("Error processing messages: {}", e.what());
                }
            }
        });
    }

    void attempt_reconnect_with_backoff() {
        std::lock_guard<std::mutex> lock(reconnect_mutex_);

        int attempts = reconnect_attempts_.load();
        const int max_retries = config_.max_reconnect_attempts > 0
            ? config_.max_reconnect_attempts
            : 10; // Default max retries

        if (attempts >= max_retries) {
            spdlog::error("Max reconnection attempts ({}) reached. Stopping auto-reconnect.", max_retries);
            state_ = ConnectionState::Error;
            return;
        }

        // Exponential backoff: 1s, 2s, 4s, 8s, 16s, max 30s
        int delay_ms = std::min(
            1000 * (1 << attempts),  // Exponential: 1000, 2000, 4000, 8000, 16000...
            30000                     // Cap at 30 seconds
        );

        reconnect_attempts_++;
        last_reconnect_attempt_ = std::chrono::steady_clock::now();

        spdlog::info("Attempting reconnection (attempt {}/{}, delay: {}ms)...",
                    attempts + 1, max_retries, delay_ms);

        // Wait with backoff
        std::this_thread::sleep_for(std::chrono::milliseconds(delay_ms));

        // Attempt reconnection in background thread to avoid blocking
        std::thread([this]() {
            if (connect()) {
                spdlog::info("✓ Reconnected successfully after {} attempts",
                           reconnect_attempts_.load());
            } else {
                spdlog::warn("Reconnection attempt {} failed", reconnect_attempts_.load());
                // Will retry on next connectionClosed() or error 1100
            }
        }).detach();
    }

    void start_health_monitoring() {
        if (health_check_enabled_.load()) {
            return; // Already running
        }

        health_check_enabled_ = true;
        last_message_time_ = std::chrono::steady_clock::now();

        health_check_thread_ = std::make_unique<std::thread>([this]() {
            const auto health_check_interval = std::chrono::seconds(30); // Check every 30 seconds
            const auto stale_threshold = std::chrono::minutes(2); // Consider stale after 2 minutes

            while (health_check_enabled_.load() && connected_.load()) {
                std::this_thread::sleep_for(health_check_interval);

                if (!connected_.load()) {
                    break;
                }

                auto now = std::chrono::steady_clock::now();
                auto time_since_last_message = now - last_message_time_;

                // Check if connection appears stale
                if (time_since_last_message > stale_threshold) {
                    spdlog::warn("Connection appears stale (no messages for {}s). Checking connection...",
                                std::chrono::duration_cast<std::chrono::seconds>(time_since_last_message).count());

                    // Verify socket is still connected
                    if (!client_.isConnected()) {
                        spdlog::error("Socket disconnected. Triggering reconnection...");
                        connected_ = false;
                        state_ = ConnectionState::Error;
                        connectionClosed(); // This will trigger reconnection if enabled
                        break;
                    } else {
                        // Socket is connected but no messages - might be idle
                        // Request server time as a heartbeat
                        spdlog::debug("Connection alive but idle. Requesting server time as heartbeat...");
                        // Note: reqCurrentTime() would be ideal but may not be available
                        // Instead, we'll just log and continue monitoring
                    }
                }

                last_heartbeat_ = now;
            }
        });

        spdlog::debug("Connection health monitoring started");
    }

    void stop_health_monitoring() {
        if (!health_check_enabled_.load()) {
            return;
        }

        health_check_enabled_ = false;

        if (health_check_thread_ && health_check_thread_->joinable()) {
            health_check_thread_->join();
            health_check_thread_.reset();
        }

        spdlog::debug("Connection health monitoring stopped");
    }

    bool wait_for_connection(int timeout_ms) {
        std::unique_lock<std::mutex> lock(connection_mutex_);
        return connection_cv_.wait_for(
            lock,
            std::chrono::milliseconds(timeout_ms),
            [this] { return connected_.load(); }
        );
    }

    bool wait_for_connection_with_progress(int timeout_ms) {
        std::unique_lock<std::mutex> lock(connection_mutex_);

        auto start = std::chrono::steady_clock::now();
        auto last_log = start;
        const auto log_interval = std::chrono::seconds(2); // Log every 2 seconds

        while (true) {
            auto now = std::chrono::steady_clock::now();
            auto elapsed = std::chrono::duration_cast<std::chrono::milliseconds>(now - start).count();
            auto time_since_last_log = now - last_log;

            // Check if we've timed out
            if (elapsed >= timeout_ms) {
                spdlog::warn("Connection timeout after {}ms (waiting for nextValidId from TWS)", elapsed);
                spdlog::warn("Possible reasons:");
                spdlog::warn("  1. TWS/Gateway is waiting for you to accept the connection");
                spdlog::warn("     → Check TWS/Gateway window for a connection prompt");
                spdlog::warn("  2. API not fully enabled in TWS/Gateway settings");
                spdlog::warn("     → Go to Edit → Global Configuration → API → Settings");
                spdlog::warn("     → Enable 'Enable ActiveX and Socket Clients'");
                spdlog::warn("  3. Client ID conflict (another application using same ID)");
                spdlog::warn("     → Try a different client_id in your config");
                spdlog::warn("  4. TWS/Gateway requires authentication");
                spdlog::warn("     → Check TWS/Gateway for authentication prompts");
                return false;
            }

            // Check for connection errors during wait
            {
                std::lock_guard<std::mutex> error_lock(error_mutex_);
                if (last_error_code_ == 502) {
                    spdlog::warn("Connection error 502 detected during wait: {}", last_error_message_);
                    return false;
                }
                // Check for authentication/authorization errors (162, 200)
                if (last_error_code_ == 162 || last_error_code_ == 200) {
                    spdlog::error("TWS authentication error {} detected: {}", last_error_code_, last_error_message_);
                    spdlog::error("TWS is likely waiting for you to accept the connection in the TWS/Gateway window");
                    return false;
                }
                // Check for other connection-related errors (500-599)
                if (last_error_code_ >= 500 && last_error_code_ < 600) {
                    spdlog::warn("Connection error {} detected during wait: {}", last_error_code_, last_error_message_);
                    return false;
                }
            }

            // Check if connected
            if (connected_.load()) {
                auto total_elapsed = std::chrono::duration_cast<std::chrono::milliseconds>(now - start).count();
                spdlog::info("Connection acknowledged after {}ms", total_elapsed);
                return true;
            }

            // Log progress every 2 seconds with diagnostic info
            if (time_since_last_log >= log_interval) {
                auto remaining = timeout_ms - elapsed;

                // Calculate progress percentage based on callbacks received
                int progress_percent = 0;
                std::string progress_bar = "";
                if (connection_callbacks_received_.connectAck) {
                    progress_percent += 33;
                }
                if (connection_callbacks_received_.managedAccounts) {
                    progress_percent += 33;
                }
                // nextValidId would add the final 34%, but we're still waiting

                // Create visual progress bar (50 characters wide)
                // Use string literals for Unicode box-drawing characters (multi-byte UTF-8)
                int filled = (progress_percent * 50) / 100;
                std::string filled_str = std::string(filled, '#');
                std::string empty_str = std::string(50 - filled, '-');
                progress_bar = filled_str + empty_str;

                // Include diagnostic info about which callbacks were received
                std::string callback_status = "Callbacks received: ";
                if (connection_callbacks_received_.connectAck) {
                    callback_status += "connectAck ✓";
                } else {
                    callback_status += "connectAck ✗";
                }
                if (connection_callbacks_received_.managedAccounts) {
                    callback_status += ", managedAccounts ✓";
                } else {
                    callback_status += ", managedAccounts ✗";
                }
                callback_status += ", nextValidId ✗ (waiting)";

                spdlog::info("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                spdlog::info("Connection Progress: [{}] {}% ({}ms remaining)", progress_bar, progress_percent, remaining);
                spdlog::info("  {}", callback_status);

                // Show step-by-step progress
                if (connection_callbacks_received_.connectAck) {
                    spdlog::info("  ✓ Step 1/3: connectAck received");
                } else {
                    spdlog::warn("  ✗ Step 1/3: connectAck not received");
                }
                if (connection_callbacks_received_.managedAccounts) {
                    auto managed_elapsed = std::chrono::duration_cast<std::chrono::milliseconds>(
                        now - connection_callbacks_received_.managedAccounts_time).count();
                    spdlog::info("  ✓ Step 2/3: managedAccounts received ({}ms ago)", managed_elapsed);
                } else {
                    spdlog::warn("  ✗ Step 2/3: managedAccounts not received");
                }
                spdlog::info("  ⏳ Step 3/3: Waiting for nextValidId...");
                spdlog::info("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

                // If we received connectAck but not managedAccounts, provide specific guidance
                if (connection_callbacks_received_.connectAck && !connection_callbacks_received_.managedAccounts) {
                    spdlog::warn("  ⚠️  Received connectAck but not managedAccounts - TWS may be waiting for connection approval");
                    spdlog::warn("  → Check TWS/Gateway window for a connection prompt and click 'Accept' or 'OK'");
                }
                // If we received both but not nextValidId, provide different guidance
                else if (connection_callbacks_received_.connectAck && connection_callbacks_received_.managedAccounts) {
                    spdlog::warn("  ⚠️  Received connectAck and managedAccounts but not nextValidId");
                    spdlog::warn("  → This may indicate TWS is processing the connection or there's a delay");
                    spdlog::warn("  → Check TWS/Gateway for any error messages or warnings");
                }

                last_log = now;
            }

            // Wait with a shorter timeout to allow periodic checks
            auto wait_time = std::min(
                std::chrono::milliseconds(timeout_ms - elapsed),
                std::chrono::milliseconds(500) // Check every 500ms
            );

            if (connection_cv_.wait_for(lock, wait_time, [this] { return connected_.load(); })) {
                auto total_elapsed = std::chrono::duration_cast<std::chrono::milliseconds>(
                    std::chrono::steady_clock::now() - start).count();
                spdlog::info("Connection acknowledged after {}ms", total_elapsed);
                return true;
            }
        }
    }

    Contract convert_to_tws_contract(const types::OptionContract& contract) {
        Contract c;
        c.symbol = contract.symbol;
        c.secType = "OPT";
        c.currency = "USD";
        c.exchange = contract.exchange;
        c.lastTradeDateOrContractMonth = contract.expiry;
        c.strike = contract.strike;
        c.right = (contract.type == types::OptionType::Call) ? "C" : "P";
        c.multiplier = "100";
        return c;
    }

    types::OptionContract convert_from_tws_contract(const Contract& contract) {
        types::OptionContract c;
        c.symbol = contract.symbol;
        c.exchange = contract.exchange;
        c.expiry = contract.lastTradeDateOrContractMonth;
        c.strike = contract.strike;
        c.type = (contract.right == "C") ? types::OptionType::Call : types::OptionType::Put;
        return c;
    }

    Order create_tws_order(types::OrderAction action, int quantity,
                          double limit_price, types::TimeInForce tif) {
        Order o;
        o.action = (action == types::OrderAction::Buy) ? "BUY" : "SELL";
        o.totalQuantity = quantity;
        o.orderType = (limit_price > 0) ? "LMT" : "MKT";

        if (limit_price > 0) {
            o.lmtPrice = limit_price;
        }

        // Set time in force
        switch (tif) {
            case types::TimeInForce::Day:
                o.tif = "DAY";
                break;
            case types::TimeInForce::GTC:
                o.tif = "GTC";
                break;
            case types::TimeInForce::IOC:
                o.tif = "IOC";
                break;
            case types::TimeInForce::FOK:
                o.tif = "FOK";
                break;
        }

        return o;
    }

    // ========================================================================
    // Remaining EWrapper callbacks (stubs - implement as needed)
    // Note: Most callbacks have default implementations from DefaultEWrapper
    // ========================================================================

    void tickString(TickerId tickerId, TickType tickType,
                   const std::string& value) override {}
    void tickEFP(TickerId tickerId, TickType tickType, double basisPoints,
                const std::string& formattedBasisPoints, double totalDividends,
                int holdDays, const std::string& futureLastTradeDate,
                double dividendImpact, double dividendsToLastTradeDate) override {}
    void tickGeneric(TickerId tickerId, TickType tickType, double value) override {}
    void tickSnapshotEnd(int reqId) override {}
    void marketDataType(TickerId reqId, int marketDataType) override {}
    void realtimeBar(TickerId reqId, long time, double open, double high,
                    double low, double close, Decimal volume, Decimal wap,
                    int count) override {}
    void historicalData(TickerId reqId, const Bar& bar) override {}
    void historicalDataEnd(int reqId, const std::string& startDateStr,
                          const std::string& endDateStr) override {}
    void scannerParameters(const std::string& xml) override {}
    void scannerData(int reqId, int rank, const ContractDetails& contractDetails,
                    const std::string& distance, const std::string& benchmark,
                    const std::string& projection, const std::string& legsStr) override {}
    void scannerDataEnd(int reqId) override {}
    void receiveFA(faDataType pFaDataType, const std::string& cxml) override {}
    void bondContractDetails(int reqId, const ContractDetails& contractDetails) override {}
    void contractDetails(int reqId, const ContractDetails& contractDetails) override {}
    void contractDetailsEnd(int reqId) override {}
    void accountSummary(int reqId, const std::string& account, const std::string& tag,
                       const std::string& value, const std::string& currency) override {}
    void accountSummaryEnd(int reqId) override {}
    void verifyMessageAPI(const std::string& apiData) override {}
    void verifyCompleted(bool isSuccessful, const std::string& errorText) override {}
    void verifyAndAuthMessageAPI(const std::string& apiData,
                                const std::string& xyzChallange) override {}
    void verifyAndAuthCompleted(bool isSuccessful, const std::string& errorText) override {}
    void displayGroupList(int reqId, const std::string& groups) override {}
    void displayGroupUpdated(int reqId, const std::string& contractInfo) override {}
    void positionMulti(int reqId, const std::string& account,
                      const std::string& modelCode, const Contract& contract,
                      Decimal pos, double avgCost) override {}
    void positionMultiEnd(int reqId) override {}
    void accountUpdateMulti(int reqId, const std::string& account,
                          const std::string& modelCode, const std::string& key,
                          const std::string& value, const std::string& currency) override {}
    void accountUpdateMultiEnd(int reqId) override {}
    void securityDefinitionOptionalParameter(int reqId, const std::string& exchange,
                                            int underlyingConId, const std::string& tradingClass,
                                            const std::string& multiplier, const std::set<std::string>& expirations,
                                            const std::set<double>& strikes) override {}
    void securityDefinitionOptionalParameterEnd(int reqId) override {}
    void softDollarTiers(int reqId, const std::vector<SoftDollarTier>& tiers) override {}
    void familyCodes(const std::vector<FamilyCode>& familyCodes) override {}
    void symbolSamples(int reqId, const std::vector<ContractDescription>& contractDescriptions) override {}
    void mktDepthExchanges(const std::vector<DepthMktDataDescription>& depthMktDataDescriptions) override {}
    void tickNews(int tickerId, time_t timeStamp, const std::string& providerCode,
                 const std::string& articleId, const std::string& headline,
                 const std::string& extraData) override {}
    void smartComponents(int reqId, const SmartComponentsMap& theMap) override {}
    void tickReqParams(int tickerId, double minTick, const std::string& bboExchange,
                      int snapshotPermissions) override {}
    void newsProviders(const std::vector<NewsProvider>& newsProviders) override {}
    void newsArticle(int requestId, int articleType, const std::string& articleText) override {}
    void historicalNews(int requestId, const std::string& time, const std::string& providerCode,
                       const std::string& articleId, const std::string& headline) override {}
    void historicalNewsEnd(int requestId, bool hasMore) override {}
    void headTimestamp(int reqId, const std::string& headTimestamp) override {}
    void histogramData(int reqId, const HistogramDataVector& data) override {}
    void historicalDataUpdate(TickerId reqId, const Bar& bar) override {}
    void rerouteMktDataReq(int reqId, int conid, const std::string& exchange) override {}
    void rerouteMktDepthReq(int reqId, int conid, const std::string& exchange) override {}
    void marketRule(int marketRuleId, const std::vector<PriceIncrement>& priceIncrements) override {}
    void pnl(int reqId, double dailyPnL, double unrealizedPnL, double realizedPnL) override {}
    void pnlSingle(int reqId, Decimal pos, double dailyPnL, double unrealizedPnL,
                  double realizedPnL, double value) override {}
    void historicalTicks(int reqId, const std::vector<HistoricalTick>& ticks, bool done) override {}
    void historicalTicksBidAsk(int reqId, const std::vector<HistoricalTickBidAsk>& ticks,
                              bool done) override {}
    void historicalTicksLast(int reqId, const std::vector<HistoricalTickLast>& ticks,
                           bool done) override {}
    void tickByTickAllLast(int reqId, int tickType, time_t time, double price,
                          Decimal size, const TickAttribLast& tickAttribLast,
                          const std::string& exchange, const std::string& specialConditions) override {}
    void tickByTickBidAsk(int reqId, time_t time, double bidPrice, double askPrice,
                         Decimal bidSize, Decimal askSize, const TickAttribBidAsk& tickAttribBidAsk) override {}
    void tickByTickMidPoint(int reqId, time_t time, double midPoint) override {}
    void orderBound(long long orderId, int apiClientId, int apiOrderId) override {}
    void completedOrder(const Contract& contract, const Order& order,
                       const OrderState& orderState) override {}
    void completedOrdersEnd() override {}
    void replaceFAEnd(int reqId, const std::string& text) override {}
    void wshMetaData(int reqId, const std::string& dataJson) override {}
    void wshEventData(int reqId, const std::string& dataJson) override {}
    void historicalSchedule(int reqId, const std::string& startDateTime,
                          const std::string& endDateTime, const std::string& timeZone,
                          const std::vector<HistoricalSession>& sessions) override {}
    void userInfo(int reqId, const std::string& whiteBrandingId) override {}

    // ========================================================================
    // Member Variables
    // ========================================================================

    config::TWSConfig config_;
    EReaderOSSignal signal_;
    EClientSocket client_;

    std::atomic<bool> connected_;
    std::atomic<int> next_order_id_;
    std::atomic<int> next_request_id_;
    ConnectionState state_;
    std::atomic<int> last_error_code_{0};
    std::string last_error_message_;
    std::mutex error_mutex_;

    // Reconnection state
    std::atomic<int> reconnect_attempts_{0};
    std::chrono::steady_clock::time_point last_reconnect_attempt_;
    std::mutex reconnect_mutex_;

    // Connection health monitoring
    std::chrono::steady_clock::time_point last_heartbeat_;
    std::chrono::steady_clock::time_point last_message_time_;
    std::atomic<bool> health_check_enabled_{false};
    std::unique_ptr<std::thread> health_check_thread_;

    // Connection callback tracking (for diagnostics)
    struct ConnectionCallbacks {
        bool connectAck = false;
        bool managedAccounts = false;
        std::chrono::steady_clock::time_point connectAck_time;
        std::chrono::steady_clock::time_point managedAccounts_time;
    } connection_callbacks_received_;

    std::unique_ptr<std::thread> reader_thread_;

    // Connection synchronization
    mutable std::mutex connection_mutex_;
    std::condition_variable connection_cv_;

    // Market data
    mutable std::mutex data_mutex_;
    std::map<int, types::MarketData> market_data_;
    std::map<int, MarketDataCallback> market_data_callbacks_;
    // Synchronous request tracking
    std::map<int, std::shared_ptr<std::promise<types::MarketData>>> market_data_promises_;

    // Orders
    mutable std::mutex order_mutex_;
    std::map<int, types::Order> orders_;
    OrderStatusCallback order_status_callback_;

    // Positions
    mutable std::mutex position_mutex_;
    std::vector<types::Position> positions_;
    PositionCallback position_callback_;
    std::shared_ptr<std::promise<std::vector<types::Position>>> positions_promise_;
    std::atomic<bool> positions_request_pending_{false};

    // Account
    mutable std::mutex account_mutex_;
    types::AccountInfo account_info_;
    AccountCallback account_callback_;
    std::shared_ptr<std::promise<types::AccountInfo>> account_promise_;
    std::atomic<bool> account_request_pending_{false};

    // Callbacks
    ErrorCallback error_callback_;

    // Rate limiting
    RateLimiter rate_limiter_;
};

// ============================================================================
// TWSClient Public Interface (delegates to Impl)
// ============================================================================

TWSClient::TWSClient(const config::TWSConfig& config)
    : pimpl_(std::make_unique<Impl>(config)) {
    spdlog::debug("TWSClient created with real TWS API integration");
}

TWSClient::~TWSClient() {
    if (is_connected()) {
        disconnect();
    }
    spdlog::debug("TWSClient destroyed");
}

bool TWSClient::connect() {
    return pimpl_->connect();
}

void TWSClient::disconnect() {
    pimpl_->disconnect();
}

bool TWSClient::is_connected() const {
    return pimpl_->is_connected();
}

ConnectionState TWSClient::get_connection_state() const {
    return pimpl_->get_connection_state();
}

void TWSClient::process_messages(int timeout_ms) {
    pimpl_->process_messages(timeout_ms);
}

// ============================================================================
// Market Data Operations
// ============================================================================

int TWSClient::request_market_data(const types::OptionContract& contract,
                                   MarketDataCallback callback) {
    return pimpl_->request_market_data(contract, callback);
}

std::optional<types::MarketData> TWSClient::request_market_data_sync(
    const types::OptionContract& contract,
    int timeout_ms) {
    return pimpl_->request_market_data_sync(contract, timeout_ms);
}

void TWSClient::cancel_market_data(int request_id) {
    pimpl_->cancel_market_data(request_id);
}

std::vector<types::OptionContract> TWSClient::request_option_chain(
    const std::string& symbol,
    const std::string& expiry) {
    return pimpl_->request_option_chain(symbol, expiry);
}

// ============================================================================
// Order Operations
// ============================================================================

int TWSClient::place_order(const types::OptionContract& contract,
                          types::OrderAction action,
                          int quantity,
                          double limit_price,
                          types::TimeInForce tif) {
    return pimpl_->place_order(contract, action, quantity, limit_price, tif);
}

int TWSClient::place_combo_order(
        const std::vector<types::OptionContract>& contracts,
        const std::vector<types::OrderAction>& actions,
        const std::vector<int>& quantities,
        const std::vector<long>& contract_ids,
        const std::vector<double>& limit_prices,
        types::TimeInForce tif) {
    return pimpl_->place_combo_order(contracts, actions, quantities, contract_ids, limit_prices, tif);
}

void TWSClient::cancel_order(int order_id) {
    pimpl_->cancel_order(order_id);
}

void TWSClient::cancel_all_orders() {
    pimpl_->cancel_all_orders();
}

std::optional<types::Order> TWSClient::get_order(int order_id) const {
    return pimpl_->get_order(order_id);
}

std::vector<types::Order> TWSClient::get_active_orders() const {
    return pimpl_->get_active_orders();
}

// ============================================================================
// Position Operations
// ============================================================================

void TWSClient::request_positions(PositionCallback callback) {
    pimpl_->request_positions(callback);
}

std::vector<types::Position> TWSClient::request_positions_sync(int timeout_ms) {
    return pimpl_->request_positions_sync(timeout_ms);
}

std::vector<types::Position> TWSClient::get_positions() const {
    return pimpl_->get_positions();
}

std::optional<types::Position> TWSClient::get_position(
    const types::OptionContract& contract) const {
    return pimpl_->get_position(contract);
}

// ============================================================================
// Account Operations
// ============================================================================

void TWSClient::request_account_updates(AccountCallback callback) {
    pimpl_->request_account_updates(callback);
}

std::optional<types::AccountInfo> TWSClient::request_account_info_sync(int timeout_ms) {
    return pimpl_->request_account_info_sync(timeout_ms);
}

std::optional<types::AccountInfo> TWSClient::get_account_info() const {
    return pimpl_->get_account_info();
}

// ============================================================================
// Callbacks
// ============================================================================

void TWSClient::set_order_status_callback(OrderStatusCallback callback) {
    pimpl_->set_order_status_callback(callback);
}

void TWSClient::set_error_callback(ErrorCallback callback) {
    pimpl_->set_error_callback(callback);
}

// ============================================================================
// Helper Methods
// ============================================================================

int TWSClient::get_next_order_id() const {
    return pimpl_->get_next_order_id();
}

bool TWSClient::is_market_open() const {
    return pimpl_->is_market_open();
}

std::chrono::system_clock::time_point TWSClient::get_server_time() const {
    return pimpl_->get_server_time();
}

// ============================================================================
// Rate Limiting (IBKR Compliance)
// ============================================================================

void TWSClient::enable_rate_limiting() {
    pimpl_->enable_rate_limiting();
}

void TWSClient::configure_rate_limiter(const RateLimiterConfig& config) {
    pimpl_->configure_rate_limiter(config);
}

std::optional<RateLimiterStatus> TWSClient::get_rate_limiter_status() const {
    return pimpl_->get_rate_limiter_status();
}

void TWSClient::cleanup_stale_rate_limiter_requests(std::chrono::seconds max_age) {
    pimpl_->cleanup_stale_rate_limiter_requests(max_age);
}

} // namespace tws

// ============================================================================
// types.h implementations
// ============================================================================

namespace types {

std::string OptionContract::to_string() const {
    return symbol + " " + expiry + " " +
           std::to_string(strike) + " " +
           option_type_to_string(type);
}

bool OptionContract::is_valid() const {
    return !symbol.empty() &&
           !expiry.empty() &&
           strike > 0 &&
           !exchange.empty();
}

bool BoxSpreadLeg::is_valid() const {
    return long_call.is_valid() &&
           short_call.is_valid() &&
           long_put.is_valid() &&
           short_put.is_valid();
}

double BoxSpreadLeg::get_strike_width() const {
    return short_call.strike - long_call.strike;
}

int BoxSpreadLeg::get_days_to_expiry() const {
    // TODO: Implement proper DTE calculation
    return 30;  // Stub
}

double Position::get_market_value() const {
    return static_cast<double>(quantity) * current_price * 100.0;
}

double Position::get_cost_basis() const {
    return static_cast<double>(quantity) * avg_price * 100.0;
}

bool Order::is_active() const {
    return status == OrderStatus::Pending ||
           status == OrderStatus::Submitted ||
           status == OrderStatus::PartiallyFilled;
}

bool Order::is_complete() const {
    return status == OrderStatus::Filled ||
           status == OrderStatus::Cancelled ||
           status == OrderStatus::Rejected;
}

double Order::get_total_cost() const {
    if (filled_quantity > 0) {
        return static_cast<double>(filled_quantity) * avg_fill_price * 100.0;
    }
    return static_cast<double>(quantity) * limit_price * 100.0;
}

} // namespace types
