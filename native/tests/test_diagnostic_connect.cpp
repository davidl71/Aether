// test_diagnostic_connect.cpp - Deep diagnostic connection test with callback tracing
// Shows exactly what callbacks are being received and when

#include "tws_client.h"
#include <spdlog/spdlog.h>
#include <iostream>
#include <chrono>
#include <thread>
#include <atomic>
#include <mutex>

using namespace tws;

// Custom callback tracker
class DiagnosticTracker {
public:
    std::atomic<bool> connect_ack_received{false};
    std::atomic<bool> managed_accounts_received{false};
    std::atomic<bool> next_valid_id_received{false};
    std::atomic<bool> error_received{false};
    std::atomic<bool> connection_closed{false};
    
    std::mutex log_mutex;
    std::vector<std::string> callback_log;
    
    void log_callback(const std::string& msg) {
        std::lock_guard<std::mutex> lock(log_mutex);
        auto now = std::chrono::system_clock::now();
        auto ms = std::chrono::duration_cast<std::chrono::milliseconds>(
            now.time_since_epoch()
        ).count() % 1000;
        
        char timestamp[32];
        time_t t = std::chrono::system_clock::to_time_t(now);
        strftime(timestamp, sizeof(timestamp), "%H:%M:%S", localtime(&t));
        
        std::string full_msg = std::string(timestamp) + "." + 
                               std::to_string(ms) + " - " + msg;
        callback_log.push_back(full_msg);
        std::cout << "  [CALLBACK] " << full_msg << std::endl;
    }
    
    void print_summary() {
        std::lock_guard<std::mutex> lock(log_mutex);
        std::cout << "\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" << std::endl;
        std::cout << "  Callback Summary" << std::endl;
        std::cout << "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" << std::endl;
        std::cout << "Total callbacks received: " << callback_log.size() << std::endl;
        std::cout << std::endl;
        std::cout << "Status:" << std::endl;
        std::cout << "  connectAck:      " << (connect_ack_received ? "✓" : "✗") << std::endl;
        std::cout << "  managedAccounts: " << (managed_accounts_received ? "✓" : "✗") << std::endl;
        std::cout << "  nextValidId:     " << (next_valid_id_received ? "✓" : "✗") << std::endl;
        std::cout << "  errors:          " << (error_received ? "✓" : "✗") << std::endl;
        std::cout << "  disconnected:    " << (connection_closed ? "✓" : "✗") << std::endl;
        std::cout << std::endl;
        
        if (!callback_log.empty()) {
            std::cout << "Callback timeline:" << std::endl;
            for (const auto& entry : callback_log) {
                std::cout << "  " << entry << std::endl;
            }
        }
    }
};

int main(int argc, char* argv[]) {
    // Enable maximum verbosity
    spdlog::set_level(spdlog::level::trace);
    spdlog::set_pattern("[%H:%M:%S.%e] [%^%l%$] [%s:%#] %v");

    std::string host = "127.0.0.1";
    int port = 7496;
    int client_id = 0; // Match Python sample
    
    if (argc >= 2) {
        client_id = std::stoi(argv[1]);
    }
    
    std::cout << "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" << std::endl;
    std::cout << "  IBKR Deep Diagnostic Connection Test" << std::endl;
    std::cout << "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" << std::endl;
    std::cout << std::endl;
    std::cout << "Configuration:" << std::endl;
    std::cout << "  Host:      " << host << std::endl;
    std::cout << "  Port:      " << port << " (Live Gateway)" << std::endl;
    std::cout << "  Client ID: " << client_id << std::endl;
    std::cout << std::endl;
    
    DiagnosticTracker tracker;
    
    std::cout << "Creating TWS client with callback tracking..." << std::endl;
    
    config::TWSConfig cfg;
    cfg.host = host;
    cfg.port = port;
    cfg.client_id = client_id;
    
    TWSClient client(cfg);
    
    std::cout << "\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" << std::endl;
    std::cout << "  Phase 1: Socket Connection" << std::endl;
    std::cout << "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" << std::endl;
    
    auto start_time = std::chrono::steady_clock::now();
    
    std::cout << "\nAttempting connection..." << std::endl;
    bool connected = client.connect();
    
    auto connect_time = std::chrono::steady_clock::now();
    auto connect_duration = std::chrono::duration_cast<std::chrono::milliseconds>(
        connect_time - start_time
    ).count();
    
    if (!connected) {
        std::cout << "\n✗ Connection failed after " << connect_duration << "ms" << std::endl;
        std::cout << "\nPossible causes:" << std::endl;
        std::cout << "  1. IB Gateway not running" << std::endl;
        std::cout << "  2. Wrong port (check 4001 vs 4002 vs 7496 vs 7497)" << std::endl;
        std::cout << "  3. API not enabled in Gateway settings" << std::endl;
        return 1;
    }
    
    std::cout << "✓ Socket connected in " << connect_duration << "ms" << std::endl;
    
    std::cout << "\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" << std::endl;
    std::cout << "  Phase 2: Handshake (30 second observation)" << std::endl;
    std::cout << "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" << std::endl;
    std::cout << "\nWatching for callbacks..." << std::endl;
    std::cout << "(Looking for: connectAck → managedAccounts → nextValidId)" << std::endl;
    std::cout << std::endl;
    
    // Monitor callbacks for 30 seconds with detailed progress
    const int total_iterations = 300; // 30 seconds at 100ms each
    int last_callback_count = 0;
    
    for (int i = 0; i < total_iterations; ++i) {
        client.process_messages(100);
        std::this_thread::sleep_for(std::chrono::milliseconds(100));
        
        // Print progress every 5 seconds
        if ((i + 1) % 50 == 0) {
            int elapsed_sec = (i + 1) / 10;
            std::cout << "[" << elapsed_sec << "s] Still watching... ";
            
            if (tracker.callback_log.size() > last_callback_count) {
                std::cout << "(callbacks detected)" << std::endl;
                last_callback_count = tracker.callback_log.size();
            } else {
                std::cout << "(no new callbacks)" << std::endl;
            }
        }
        
        // Check if we got all required callbacks
        if (tracker.connect_ack_received && 
            tracker.managed_accounts_received && 
            tracker.next_valid_id_received) {
            std::cout << "\n✓ All callbacks received! Connection fully established." << std::endl;
            break;
        }
        
        // Stop early if disconnected
        if (tracker.connection_closed) {
            std::cout << "\n✗ Connection closed by Gateway" << std::endl;
            break;
        }
    }
    
    std::cout << "\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" << std::endl;
    std::cout << "  Phase 3: Analysis" << std::endl;
    std::cout << "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" << std::endl;
    
    tracker.print_summary();
    
    std::cout << "\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" << std::endl;
    std::cout << "  Diagnosis" << std::endl;
    std::cout << "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" << std::endl;
    std::cout << std::endl;
    
    if (!tracker.connect_ack_received) {
        std::cout << "❌ CRITICAL: No connectAck received" << std::endl;
        std::cout << std::endl;
        std::cout << "This means the TWS API handshake never started." << std::endl;
        std::cout << "Possible causes:" << std::endl;
        std::cout << "  1. Connected to wrong port (non-API service)" << std::endl;
        std::cout << "  2. API version mismatch" << std::endl;
        std::cout << "  3. Gateway rejected connection before handshake" << std::endl;
        
    } else if (!tracker.managed_accounts_received && !tracker.connection_closed) {
        std::cout << "⚠️  WARNING: Got connectAck but no managedAccounts (still connected)" << std::endl;
        std::cout << std::endl;
        std::cout << "Gateway is waiting for something. This usually means:" << std::endl;
        std::cout << "  1. Connection approval dialog waiting (not visible?)" << std::endl;
        std::cout << "  2. 'Accept incoming requests automatically' is OFF" << std::endl;
        std::cout << "  3. Security restrictions blocking the connection" << std::endl;
        std::cout << std::endl;
        std::cout << "Check IB Gateway window for any prompts or notifications!" << std::endl;
        
    } else if (!tracker.managed_accounts_received && tracker.connection_closed) {
        std::cout << "❌ REJECTED: Gateway closed connection after connectAck" << std::endl;
        std::cout << std::endl;
        std::cout << "The connection was explicitly rejected by IB Gateway." << std::endl;
        std::cout << "Most common cause:" << std::endl;
        std::cout << "  → 'Accept incoming connection requests automatically' is DISABLED" << std::endl;
        std::cout << std::endl;
        std::cout << "Solution:" << std::endl;
        std::cout << "  1. Open IB Gateway" << std::endl;
        std::cout << "  2. Configure → Settings → API → Settings" << std::endl;
        std::cout << "  3. Check: ☑ Accept incoming connection requests automatically" << std::endl;
        std::cout << "  4. Add '127.0.0.1' to Trusted IPs" << std::endl;
        std::cout << "  5. Click OK and restart Gateway" << std::endl;
        
    } else if (tracker.managed_accounts_received && !tracker.next_valid_id_received) {
        std::cout << "⚠️  PARTIAL: Got managedAccounts but no nextValidId" << std::endl;
        std::cout << std::endl;
        std::cout << "Connection partially established but not ready for operations." << std::endl;
        std::cout << "This is unusual - may indicate:" << std::endl;
        std::cout << "  1. Gateway is still initializing" << std::endl;
        std::cout << "  2. Read-only API mode preventing order ID assignment" << std::endl;
        std::cout << "  3. Account/permission issues" << std::endl;
        
    } else if (tracker.next_valid_id_received) {
        std::cout << "✅ SUCCESS: Connection fully established!" << std::endl;
        std::cout << std::endl;
        std::cout << "All required callbacks received. You can now:" << std::endl;
        std::cout << "  • Retrieve positions" << std::endl;
        std::cout << "  • Get account data" << std::endl;
        std::cout << "  • Request market data" << std::endl;
        std::cout << "  • Place orders (if not read-only)" << std::endl;
        std::cout << std::endl;
        std::cout << "Run the full test:" << std::endl;
        std::cout << "  ./native/build_native/bin/test_positions_live" << std::endl;
    }
    
    std::cout << std::endl;
    
    // Try to get positions if connection successful
    if (tracker.next_valid_id_received) {
        std::cout << "\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" << std::endl;
        std::cout << "  Phase 4: Position Test" << std::endl;
        std::cout << "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" << std::endl;
        std::cout << "\nAttempting to retrieve positions..." << std::endl;
        
        auto positions = client.request_positions_sync(5000);
        
        std::cout << "\nResult: Found " << positions.size() << " position(s)" << std::endl;
        
        if (!positions.empty()) {
            std::cout << "\nSample positions:" << std::endl;
            for (size_t i = 0; i < std::min(size_t(5), positions.size()); ++i) {
                const auto& pos = positions[i];
                std::cout << "  " << (i+1) << ". " << pos.contract.symbol;
                if (!pos.contract.expiry.empty()) {
                    std::cout << " " << pos.contract.expiry 
                             << " $" << pos.contract.strike;
                }
                std::cout << " × " << pos.quantity << std::endl;
            }
        }
    }
    
    std::cout << "\nDisconnecting..." << std::endl;
    client.disconnect();
    
    std::cout << "\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" << std::endl;
    std::cout << "  Test Complete" << std::endl;
    std::cout << "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" << std::endl;
    
    return tracker.next_valid_id_received ? 0 : 1;
}
