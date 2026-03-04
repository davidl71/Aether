// connection_utils.cpp - TWS connection utilities (port checking, mock detection)
#include "connection_utils.h"

#include <algorithm>
#include <chrono>
#include <cstring>
#include <cstdlib>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <unistd.h>
#include <fcntl.h>
#include <errno.h>

namespace tws {

bool is_port_open(const std::string& host, int port, int timeout_ms) {
    int sock = socket(AF_INET, SOCK_STREAM, 0);
    if (sock < 0) {
        return false;
    }

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

    int result = connect(sock, (struct sockaddr*)&server_addr, sizeof(server_addr));

    if (result == 0) {
        close(sock);
        return true;
    }

    if (errno == EINPROGRESS) {
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

std::vector<int> get_port_candidates(int configured_port) {
    std::vector<int> candidates;
    candidates.push_back(configured_port);

    std::vector<int> standard_ports = {7497, 7496, 4002, 4001};
    for (int port : standard_ports) {
        if (port != configured_port) {
            candidates.push_back(port);
        }
    }

    return candidates;
}

bool env_flag_enabled(const char* name) {
    const char* value = std::getenv(name);
    if (!value) {
        return false;
    }
    std::string flag(value);
    std::transform(flag.begin(), flag.end(), flag.begin(), [](unsigned char c) {
        return static_cast<char>(std::tolower(c));
    });
    return flag == "1" || flag == "true" || flag == "yes" || flag == "on";
}

bool should_use_mock_client(const config::TWSConfig& config) {
    return config.use_mock || env_flag_enabled("TWS_MOCK");
}

types::OptionContract make_mock_contract(const std::string& symbol,
                                         const std::string& expiry,
                                         double strike,
                                         types::OptionType type) {
    types::OptionContract contract;
    contract.symbol = symbol.empty() ? "SPY" : symbol;
    contract.expiry = expiry.empty() ? "20251219" : expiry;
    contract.strike = strike;
    contract.type = type;
    contract.exchange = "SMART";
    contract.local_symbol = contract.symbol + contract.expiry +
                            (type == types::OptionType::Call ? "C" : "P") +
                            std::to_string(static_cast<int>(strike * 1000));
    return contract;
}

types::MarketData generate_mock_market_data(const types::OptionContract& contract) {
    types::MarketData data;
    data.symbol = contract.symbol.empty() ? "SPY" : contract.symbol;
    data.timestamp = std::chrono::system_clock::now();

    double base_price = contract.strike > 0 ? contract.strike / 10.0 : 100.0;
    if (!contract.symbol.empty()) {
        base_price += static_cast<double>((contract.symbol[0] % 10));
    }

    data.bid = std::max(0.1, base_price - 0.1);
    data.ask = data.bid + 0.2;
    data.last = (data.bid + data.ask) / 2.0;
    data.bid_size = 10;
    data.ask_size = 10;
    data.last_size = 1;
    data.volume = 5000;
    data.high = data.last + 0.5;
    data.low = data.last - 0.5;
    data.open = data.last - 0.2;
    data.close = data.last - 0.1;
    data.implied_volatility = 0.25;
    data.delta = contract.type == types::OptionType::Call ? 0.4 : -0.4;
    data.gamma = 0.02;
    data.theta = -0.01;
    data.vega = 0.15;
    return data;
}

} // namespace tws
