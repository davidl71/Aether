# PCAP Capture for TWS API Debugging

## Overview

The TWS client includes optional PCAP (packet capture) functionality to help debug connection issues and analyze TWS API traffic. This feature captures connection events and can be extended to capture full packet data.

## Features

- **Connection Event Capture**: Records key connection milestones (connectAck, nextValidId, connectionClosed)
- **Standard PCAP Format**: Output files can be opened in Wireshark, tcpdump, or other pcap analysis tools
- **Automatic Filename Generation**: Creates timestamped filenames if not specified
- **Configurable Precision**: Supports microsecond or nanosecond timestamp precision

## Configuration

### Enable PCAP Capture

Add the following to your `config.json`:

```json
{
  "tws": {
    "enable_pcap_capture": true,
    "pcap_output_file": "tws_capture.pcap",
    "pcap_nanosecond_precision": false
  }
}
```

### Configuration Options

- **`enable_pcap_capture`** (boolean, default: `false`)
  - Enable/disable PCAP capture
  - Only active when `use_mock` is `false` (requires real network connection)

- **`pcap_output_file`** (string, default: `""`)
  - Output file path for PCAP data
  - If empty, auto-generates filename: `tws_capture_YYYYMMDD_HHMMSS_mmm.pcap`
  - Example: `tws_capture_20241219_143052_123.pcap`

- **`pcap_nanosecond_precision`** (boolean, default: `false`)
  - Use nanosecond precision timestamps (requires pcap format 2.4)
  - Default uses microsecond precision (pcap format 2.4 standard)

## Usage

### Basic Usage

1. Enable PCAP capture in configuration:

```json
{
  "tws": {
    "enable_pcap_capture": true
  }
}
```

1. Run your application - PCAP file will be created automatically:

```bash
./build/ib_box_spread
```

1. Check logs for PCAP file location:

```
[info] PCAP capture enabled: tws_capture_20241219_143052_123.pcap
```

### Custom Output File

Specify a custom output file:

```json
{
  "tws": {
    "enable_pcap_capture": true,
    "pcap_output_file": "/tmp/tws_debug.pcap"
  }
}
```

### Analyzing PCAP Files

#### Using Wireshark

1. Open the PCAP file in Wireshark:

```bash
wireshark tws_capture_20241219_143052_123.pcap
```

1. Filter for TWS traffic:

```
tcp.port == 7497 || tcp.port == 7496
```

1. View connection events:

```
tcp contains "CONNECTION"
```

#### Using tcpdump

```bash
tcpdump -r tws_capture_20241219_143052_123.pcap -A
```

#### Using tshark (command-line)

```bash
tshark -r tws_capture_20241219_143052_123.pcap -V
```

## Captured Events

The current implementation captures the following connection events:

1. **CONNECT_ATTEMPT**: When `eConnect()` is called
   - Includes: host, port, client_id

2. **CONNECT_SUCCESS/CONNECT_FAILED**: Result of connection attempt

3. **CONNECTION_ACK**: When `connectAck()` callback is received
   - Indicates socket connection established

4. **CONNECTION_COMPLETE**: When `nextValidId()` callback is received
   - Includes: next valid order ID
   - Indicates full connection established

5. **CONNECTION_CLOSED**: When connection is closed
   - Flushes remaining data to file

## PCAP File Format

The output files use standard PCAP format (version 2.4) and include:

- **Global Header**: File format version, timestamp precision, network type
- **Packet Headers**: Timestamp, packet length for each captured event
- **Packet Data**: Minimal Ethernet/IP/TCP frames with event data as payload

### Packet Structure

Each captured event is wrapped in a minimal network frame:

- **Ethernet Header**: 14 bytes (localhost MAC addresses)
- **IP Header**: 20 bytes (127.0.0.1 to 127.0.0.1)
- **TCP Header**: 20 bytes (source/destination ports)
- **Payload**: Event data (ASCII text)

## Statistics

When PCAP capture is closed, statistics are logged:

```
[info] PCAP capture statistics: 5 packets, 1024 bytes
```

Statistics include:

- Total packets captured
- Total bytes captured
- First packet timestamp
- Last packet timestamp

## Limitations

### Current Implementation

The current implementation captures **connection events** rather than raw socket data. This is because:

1. The TWS API (`EClientSocket`) handles socket operations internally
2. Raw socket data is not easily accessible without modifying vendor code
3. Connection events provide sufficient information for debugging connection issues

### Future Enhancements

Potential improvements:

- **Socket-level interception**: Wrap socket operations to capture raw data
- **libpcap integration**: Use libpcap for true network-level capture (requires root/admin)
- **Message-level capture**: Capture TWS API messages if exposed by the API
- **Filtering**: Capture only specific message types or time ranges

## Troubleshooting

### PCAP File Not Created

1. Check that `enable_pcap_capture` is `true` in config
2. Verify `use_mock` is `false` (PCAP only works with real connections)
3. Check file permissions for output directory
4. Review logs for error messages

### Empty PCAP File

- PCAP file is created on initialization
- Packets are captured during connection events
- If connection fails immediately, file may contain only connection attempts
- Check connection logs to understand why events weren't captured

### Cannot Open in Wireshark

- Verify file is valid PCAP format (should start with magic number `0xa1b2c3d4`)
- Check file is not corrupted
- Try opening with `tcpdump -r` first to verify format

## Example Workflow

### Debugging Connection Issues

1. **Enable PCAP capture**:

```json
{
  "tws": {
    "enable_pcap_capture": true,
    "pcap_output_file": "connection_debug.pcap"
  }
}
```

1. **Run application and attempt connection**

2. **Analyze PCAP file**:

```bash

# View connection timeline

tshark -r connection_debug.pcap -T fields -e frame.time -e tcp.payload

# Filter connection events

tshark -r connection_debug.pcap -Y "tcp contains CONNECTION"
```

1. **Correlate with logs**:
   - PCAP timestamps match log timestamps
   - Connection events in PCAP correspond to log messages
   - Identify timing issues or missing events

## Integration with Other Tools

### Wireshark Analysis

1. Open PCAP file in Wireshark
2. Use display filters:
   - `tcp.port == 7497` - Filter TWS port
   - `tcp.flags.syn == 1` - Find connection attempts
   - `tcp.flags.fin == 1` - Find disconnections

3. Follow TCP stream to see connection flow

### Scripting Analysis

```python
import pcapy

# Read PCAP file

reader = pcapy.open_offline("tws_capture.pcap")

# Process packets

for header, packet in reader:
    # Analyze packet data
    pass
```

## Security Considerations

- PCAP files may contain sensitive connection information
- Do not commit PCAP files to version control
- Store PCAP files securely if they contain production data
- Consider adding PCAP files to `.gitignore`

## Related Documentation

- TWS/IBKR integration: see [API_DOCUMENTATION_INDEX.md](API_DOCUMENTATION_INDEX.md) (TWS_INTEGRATION_STATUS doc removed).
- [Configuration Guide](ENVIRONMENT_CONFIGURATION.md)
- [Troubleshooting Guide](TROUBLESHOOTING_BLANK_PAGE.md)
