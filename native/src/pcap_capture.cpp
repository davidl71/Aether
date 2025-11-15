// pcap_capture.cpp - PCAP packet capture implementation
#include "pcap_capture.h"
#include <spdlog/spdlog.h>
#include <iomanip>
#include <sstream>
#include <algorithm>
#include <cstring>

#ifdef __APPLE__
#include <arpa/inet.h>
#elif __linux__
#include <arpa/inet.h>
#elif _WIN32
#include <winsock2.h>
#include <ws2tcpip.h>
#endif

namespace pcap {

// ============================================================================
// PcapCapture Implementation
// ============================================================================

PcapCapture::PcapCapture(const std::string& output_file, bool use_nanosecond_precision)
    : output_file_(output_file)
    , use_nanosecond_precision_(use_nanosecond_precision)
    , is_open_(false)
{
    stats_.packets_captured = 0;
    stats_.bytes_captured = 0;
}

PcapCapture::~PcapCapture() {
    close();
}

bool PcapCapture::open() {
    if (is_open_) {
        spdlog::warn("PCAP capture already open: {}", output_file_);
        return true;
    }

    file_.open(output_file_, std::ios::binary | std::ios::out);
    if (!file_.is_open()) {
        spdlog::error("Failed to open PCAP file: {}", output_file_);
        return false;
    }

    if (!write_global_header()) {
        spdlog::error("Failed to write PCAP global header");
        file_.close();
        return false;
    }

    is_open_ = true;
    spdlog::info("PCAP capture started: {}", output_file_);
    return true;
}

void PcapCapture::close() {
    if (!is_open_) {
        return;
    }

    flush();
    file_.close();
    is_open_ = false;

    spdlog::info("PCAP capture closed: {} ({} packets, {} bytes)",
                 output_file_,
                 stats_.packets_captured,
                 stats_.bytes_captured);
}

bool PcapCapture::write_global_header() {
    PcapGlobalHeader header;

    // Magic number: 0xa1b2c3d4 for microsecond precision, 0xa1b23c4d for nanosecond
    header.magic_number = use_nanosecond_precision_ ? 0xa1b23c4d : 0xa1b2c3d4;
    header.version_major = 2;
    header.version_minor = 4;
    header.thiszone = 0;  // UTC
    header.sigfigs = 0;
    header.snaplen = 65535;  // Max packet size
    header.network = 1;  // Ethernet

    file_.write(reinterpret_cast<const char*>(&header), sizeof(header));
    return file_.good();
}

bool PcapCapture::write_packet(
    const std::chrono::system_clock::time_point& timestamp,
    const std::vector<uint8_t>& packet_data
) {
    if (!is_open_ || !file_.is_open()) {
        return false;
    }

    // Convert timestamp
    auto duration = timestamp.time_since_epoch();
    auto seconds = std::chrono::duration_cast<std::chrono::seconds>(duration);
    auto subseconds = duration - seconds;

    PcapPacketHeader pkt_header;
    pkt_header.ts_sec = seconds.count();

    if (use_nanosecond_precision_) {
        auto nanoseconds = std::chrono::duration_cast<std::chrono::nanoseconds>(subseconds);
        pkt_header.ts_usec = nanoseconds.count();
    } else {
        auto microseconds = std::chrono::duration_cast<std::chrono::microseconds>(subseconds);
        pkt_header.ts_usec = microseconds.count();
    }

    pkt_header.incl_len = static_cast<uint32_t>(packet_data.size());
    pkt_header.orig_len = static_cast<uint32_t>(packet_data.size());

    // Write packet header
    file_.write(reinterpret_cast<const char*>(&pkt_header), sizeof(pkt_header));

    // Write packet data
    file_.write(reinterpret_cast<const char*>(packet_data.data()), packet_data.size());

    // Update statistics
    stats_.packets_captured++;
    stats_.bytes_captured += packet_data.size();

    if (stats_.packets_captured == 1) {
        stats_.first_packet_time = timestamp;
    }
    stats_.last_packet_time = timestamp;

    return file_.good();
}

bool PcapCapture::capture_packet(
    uint32_t src_ip,
    uint32_t dst_ip,
    uint16_t src_port,
    uint16_t dst_port,
    const std::vector<uint8_t>& data,
    bool direction
) {
    return capture_packet(src_ip, dst_ip, src_port, dst_port, data, get_timestamp(), direction);
}

bool PcapCapture::capture_packet(
    uint32_t src_ip,
    uint32_t dst_ip,
    uint16_t src_port,
    uint16_t dst_port,
    const std::vector<uint8_t>& data,
    const std::chrono::system_clock::time_point& timestamp,
    bool direction
) {
    if (!is_open_) {
        return false;
    }

    // Create Ethernet/IP/TCP frame
    auto frame = create_localhost_frame(src_ip, dst_ip, src_port, dst_port, data);

    return write_packet(timestamp, frame);
}

bool PcapCapture::capture_raw(
    const std::vector<uint8_t>& data,
    bool direction,
    uint16_t src_port,
    uint16_t dst_port
) {
    if (!is_open_) {
        return false;
    }

    // Use localhost IPs (127.0.0.1 = 0x7f000001 in network byte order)
    uint32_t localhost_ip = 0x0100007f;  // 127.0.0.1 in network byte order

    // Default ports if not provided
    if (src_port == 0) {
        src_port = direction ? 0x1234 : 0x1d49;  // Default client/server ports
    }
    if (dst_port == 0) {
        dst_port = direction ? 0x1d49 : 0x1234;  // TWS default port 7497 = 0x1d49
    }

    return capture_packet(
        localhost_ip,
        localhost_ip,
        src_port,
        dst_port,
        data,
        direction
    );
}

void PcapCapture::flush() {
    if (file_.is_open()) {
        file_.flush();
    }
}

std::vector<uint8_t> PcapCapture::create_localhost_frame(
    uint32_t src_ip,
    uint32_t dst_ip,
    uint16_t src_port,
    uint16_t dst_port,
    const std::vector<uint8_t>& payload
) {
    // If IPs are 0, use localhost
    if (src_ip == 0) {
        src_ip = 0x0100007f;  // 127.0.0.1
    }
    if (dst_ip == 0) {
        dst_ip = 0x0100007f;  // 127.0.0.1
    }

    std::vector<uint8_t> frame;
    frame.reserve(14 + 20 + 20 + payload.size());  // Ethernet + IP + TCP + payload

    // Ethernet header (14 bytes) - minimal for localhost
    // Dst MAC: 00:00:00:00:00:00
    frame.insert(frame.end(), 6, 0x00);
    // Src MAC: 00:00:00:00:00:01
    frame.insert(frame.end(), 5, 0x00);
    frame.push_back(0x01);
    // EtherType: IPv4 (0x0800)
    frame.push_back(0x08);
    frame.push_back(0x00);

    // IP header (20 bytes)
    uint8_t ip_header[20] = {
        0x45, 0x00,  // Version (4) + IHL (5) + DSCP + ECN
        0x00, 0x00,  // Total length (will be set)
        0x00, 0x00,  // Identification
        0x40, 0x00,  // Flags + Fragment offset
        0x40,        // TTL (64)
        0x06,        // Protocol (TCP)
        0x00, 0x00,  // Header checksum (will be calculated)
    };

    // Set IP addresses
    std::memcpy(&ip_header[12], &src_ip, 4);
    std::memcpy(&ip_header[16], &dst_ip, 4);

    // Set total length
    uint16_t total_len = 20 + 20 + static_cast<uint16_t>(payload.size());
    ip_header[2] = (total_len >> 8) & 0xff;
    ip_header[3] = total_len & 0xff;

    // Calculate IP checksum
    uint16_t ip_checksum = calculate_ip_checksum(ip_header, 20);
    ip_header[10] = (ip_checksum >> 8) & 0xff;
    ip_header[11] = ip_checksum & 0xff;

    frame.insert(frame.end(), ip_header, ip_header + 20);

    // TCP header (20 bytes minimum)
    uint8_t tcp_header[20] = {
        0x00, 0x00,  // Source port (will be set)
        0x00, 0x00,  // Destination port (will be set)
        0x00, 0x00, 0x00, 0x00,  // Sequence number
        0x00, 0x00, 0x00, 0x00,  // Acknowledgment number
        0x50,        // Data offset (5 * 4 = 20 bytes) + Reserved
        0x10,        // Flags (ACK)
        0x00, 0x00,  // Window size
        0x00, 0x00,  // Checksum (will be calculated)
        0x00, 0x00,  // Urgent pointer
    };

    // Set ports (network byte order)
    tcp_header[0] = (src_port >> 8) & 0xff;
    tcp_header[1] = src_port & 0xff;
    tcp_header[2] = (dst_port >> 8) & 0xff;
    tcp_header[3] = dst_port & 0xff;

    // Calculate TCP checksum
    uint16_t tcp_checksum = calculate_tcp_checksum(
        src_ip, dst_ip, tcp_header, 20 + static_cast<uint16_t>(payload.size())
    );
    tcp_header[16] = (tcp_checksum >> 8) & 0xff;
    tcp_header[17] = tcp_checksum & 0xff;

    frame.insert(frame.end(), tcp_header, tcp_header + 20);

    // Payload
    frame.insert(frame.end(), payload.begin(), payload.end());

    return frame;
}

uint16_t PcapCapture::calculate_ip_checksum(const uint8_t* data, size_t len) {
    uint32_t sum = 0;
    for (size_t i = 0; i < len; i += 2) {
        if (i + 1 < len) {
            sum += (static_cast<uint16_t>(data[i]) << 8) | data[i + 1];
        } else {
            sum += static_cast<uint16_t>(data[i]) << 8;
        }
    }
    while (sum >> 16) {
        sum = (sum & 0xffff) + (sum >> 16);
    }
    return static_cast<uint16_t>(~sum);
}

uint16_t PcapCapture::calculate_tcp_checksum(
    uint32_t src_ip,
    uint32_t dst_ip,
    const uint8_t* tcp_header,
    size_t tcp_len
) {
    uint32_t sum = 0;

    // Pseudo header
    sum += (src_ip >> 16) & 0xffff;
    sum += src_ip & 0xffff;
    sum += (dst_ip >> 16) & 0xffff;
    sum += dst_ip & 0xffff;
    sum += 0x0006;  // TCP protocol
    sum += static_cast<uint16_t>(tcp_len);

    // TCP header and data
    for (size_t i = 0; i < tcp_len; i += 2) {
        if (i + 1 < tcp_len) {
            sum += (static_cast<uint16_t>(tcp_header[i]) << 8) | tcp_header[i + 1];
        } else {
            sum += static_cast<uint16_t>(tcp_header[i]) << 8;
        }
    }

    while (sum >> 16) {
        sum = (sum & 0xffff) + (sum >> 16);
    }
    return static_cast<uint16_t>(~sum);
}

// ============================================================================
// Helper Functions
// ============================================================================

uint32_t ip_to_uint32(const std::string& ip) {
    uint32_t result = 0;
    std::istringstream iss(ip);
    std::string segment;
    int shift = 24;

    while (std::getline(iss, segment, '.')) {
        if (shift < 0) break;
        uint32_t octet = static_cast<uint32_t>(std::stoul(segment));
        result |= (octet << shift);
        shift -= 8;
    }

    // Convert to network byte order (host to network long)
    #if defined(__APPLE__) || defined(__linux__) || defined(_WIN32)
    return htonl(result);
    #else
    // Fallback: assume little-endian
    return ((result & 0xff000000) >> 24) |
           ((result & 0x00ff0000) >> 8) |
           ((result & 0x0000ff00) << 8) |
           ((result & 0x000000ff) << 24);
    #endif
}

std::string uint32_to_ip(uint32_t ip) {
    // Convert from network byte order (network to host long)
    #if defined(__APPLE__) || defined(__linux__) || defined(_WIN32)
    ip = ntohl(ip);
    #else
    // Fallback: assume little-endian
    ip = ((ip & 0xff000000) >> 24) |
         ((ip & 0x00ff0000) >> 8) |
         ((ip & 0x0000ff00) << 8) |
         ((ip & 0x000000ff) << 24);
    #endif
    std::ostringstream oss;
    oss << ((ip >> 24) & 0xff) << "."
        << ((ip >> 16) & 0xff) << "."
        << ((ip >> 8) & 0xff) << "."
        << (ip & 0xff);
    return oss.str();
}

std::string generate_pcap_filename(const std::string& prefix) {
    auto now = std::chrono::system_clock::now();
    auto time_t = std::chrono::system_clock::to_time_t(now);
    auto ms = std::chrono::duration_cast<std::chrono::milliseconds>(
        now.time_since_epoch()
    ) % 1000;

    std::ostringstream oss;
    oss << prefix << "_"
        << std::put_time(std::localtime(&time_t), "%Y%m%d_%H%M%S")
        << "_" << std::setfill('0') << std::setw(3) << ms.count()
        << ".pcap";
    return oss.str();
}

} // namespace pcap
