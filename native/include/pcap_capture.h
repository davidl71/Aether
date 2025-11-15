// pcap_capture.h - PCAP packet capture for TWS API debugging
#pragma once

#include <string>
#include <memory>
#include <chrono>
#include <fstream>
#include <vector>
#include <cstdint>

namespace pcap {

// ============================================================================
// PCAP File Format Structures
// ============================================================================

// PCAP Global Header (24 bytes)
struct PcapGlobalHeader {
    uint32_t magic_number;    // 0xa1b2c3d4 or 0xa1b23c4d (nanosecond precision)
    uint16_t version_major;    // 2
    uint16_t version_minor;    // 4
    int32_t  thiszone;        // GMT to local correction (0)
    uint32_t sigfigs;         // Accuracy of timestamps (0)
    uint32_t snaplen;         // Max length of captured packets (65535)
    uint32_t network;         // Data link type (1 = Ethernet)
};

// PCAP Packet Header (16 bytes)
struct PcapPacketHeader {
    uint32_t ts_sec;          // Timestamp seconds
    uint32_t ts_usec;         // Timestamp microseconds (or nanoseconds if magic is 0xa1b23c4d)
    uint32_t incl_len;        // Number of bytes of packet data actually captured
    uint32_t orig_len;        // Length of packet as it appeared on the network
};

// ============================================================================
// PCAP Capture Class
// ============================================================================

class PcapCapture {
public:
    // Constructor
    explicit PcapCapture(const std::string& output_file, bool use_nanosecond_precision = false);

    // Destructor - closes file if still open
    ~PcapCapture();

    // Disable copy
    PcapCapture(const PcapCapture&) = delete;
    PcapCapture& operator=(const PcapCapture&) = delete;

    // Enable move
    PcapCapture(PcapCapture&&) noexcept = default;
    PcapCapture& operator=(PcapCapture&&) noexcept = default;

    // Open capture file and write global header
    bool open();

    // Close capture file
    void close();

    // Check if capture is active
    bool is_open() const { return file_.is_open(); }

    // Capture a TCP packet
    // src_ip, dst_ip: IP addresses in network byte order (or 0 for localhost)
    // src_port, dst_port: Port numbers in network byte order
    // data: Packet payload
    // direction: true = client->server, false = server->client
    bool capture_packet(
        uint32_t src_ip,
        uint32_t dst_ip,
        uint16_t src_port,
        uint16_t dst_port,
        const std::vector<uint8_t>& data,
        bool direction = true
    );

    // Capture a TCP packet with explicit timestamp
    bool capture_packet(
        uint32_t src_ip,
        uint32_t dst_ip,
        uint16_t src_port,
        uint16_t dst_port,
        const std::vector<uint8_t>& data,
        const std::chrono::system_clock::time_point& timestamp,
        bool direction = true
    );

    // Capture raw data (for socket-level capture)
    // Creates a minimal Ethernet frame with IP/TCP headers
    bool capture_raw(
        const std::vector<uint8_t>& data,
        bool direction = true,
        uint16_t src_port = 0,
        uint16_t dst_port = 0
    );

    // Flush buffered data to disk
    void flush();

    // Get output file path
    std::string get_output_file() const { return output_file_; }

    // Get statistics
    struct Stats {
        size_t packets_captured = 0;
        size_t bytes_captured = 0;
        std::chrono::system_clock::time_point first_packet_time;
        std::chrono::system_clock::time_point last_packet_time;
    };

    Stats get_stats() const { return stats_; }

private:
    std::string output_file_;
    std::ofstream file_;
    bool use_nanosecond_precision_;
    bool is_open_;
    Stats stats_;

    // Write global header
    bool write_global_header();

    // Write packet header and data
    bool write_packet(
        const std::chrono::system_clock::time_point& timestamp,
        const std::vector<uint8_t>& packet_data
    );

    // Create minimal Ethernet/IP/TCP frame for localhost traffic
    std::vector<uint8_t> create_localhost_frame(
        uint32_t src_ip,
        uint32_t dst_ip,
        uint16_t src_port,
        uint16_t dst_port,
        const std::vector<uint8_t>& payload
    );

    // Calculate IP checksum
    uint16_t calculate_ip_checksum(const uint8_t* data, size_t len);

    // Calculate TCP checksum
    uint16_t calculate_tcp_checksum(
        uint32_t src_ip,
        uint32_t dst_ip,
        const uint8_t* tcp_header,
        size_t tcp_len
    );

    // Get current timestamp
    std::chrono::system_clock::time_point get_timestamp() const {
        return std::chrono::system_clock::now();
    }
};

// ============================================================================
// Helper Functions
// ============================================================================

// Convert IPv4 address string to network byte order
uint32_t ip_to_uint32(const std::string& ip);

// Convert network byte order to IPv4 address string
std::string uint32_to_ip(uint32_t ip);

// Generate output filename with timestamp
std::string generate_pcap_filename(const std::string& prefix = "tws_capture");

} // namespace pcap
