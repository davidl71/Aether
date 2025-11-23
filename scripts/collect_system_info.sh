#!/usr/bin/env bash
# collect_system_info.sh - Collect system information for environment documentation
# Usage: ./scripts/collect_system_info.sh > system_info.json

set -euo pipefail

# Detect OS
OS="$(uname -s)"

echo "{"
echo "  \"collection_date\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\","
echo "  \"hostname\": \"$(hostname)\","
echo "  \"operating_system\": \"$OS\","

if [[ "$OS" == "Darwin" ]]; then
    # macOS System Information
    echo "  \"os_version\": \"$(sw_vers -productVersion)\","
    echo "  \"os_build\": \"$(sw_vers -buildVersion)\","
    echo "  \"os_name\": \"$(sw_vers -productName)\","
    echo "  \"cpu_brand\": \"$(sysctl -n machdep.cpu.brand_string)\","
    echo "  \"cpu_cores\": $(sysctl -n hw.ncpu),"
    echo "  \"cpu_physical_cores\": $(sysctl -n hw.physicalcpu),"
    echo "  \"cpu_logical_cores\": $(sysctl -n hw.logicalcpu),"
    echo "  \"memory_total_gb\": $(($(sysctl -n hw.memsize) / 1024 / 1024 / 1024)),"
    echo "  \"memory_available_gb\": $(vm_stat | grep 'Pages free' | awk '{print $3}' | sed 's/\.//' | awk '{print int($1*4096/1024/1024/1024)}'),"
    echo "  \"apple_intelligence_available\": $(sysctl -n machdep.cpu.brand_string | grep -q 'Apple' && echo 'true' || echo 'false'),"
    echo "  \"neural_engine_available\": $(sysctl -n machdep.cpu.brand_string | grep -E 'M[1-4]' && echo 'true' || echo 'false'),"
    echo "  \"disk_info\": ["
    df -h | tail -n +2 | while read -r line; do
        filesystem=$(echo "$line" | awk '{print $1}')
        size=$(echo "$line" | awk '{print $2}')
        used=$(echo "$line" | awk '{print $3}')
        available=$(echo "$line" | awk '{print $4}')
        use_percent=$(echo "$line" | awk '{print $5}')
        mount=$(echo "$line" | awk '{print $9}')
        echo "    {"
        echo "      \"filesystem\": \"$filesystem\","
        echo "      \"size\": \"$size\","
        echo "      \"used\": \"$used\","
        echo "      \"available\": \"$available\","
        echo "      \"use_percent\": \"$use_percent\","
        echo "      \"mount\": \"$mount\""
        echo "    }$(if [ "$line" != "$(df -h | tail -n 1)" ]; then echo ','; fi)"
    done | sed '$ s/,$//'
    echo "  ],"
elif [[ "$OS" == "Linux" ]]; then
    # Linux System Information
    if command -v lsb_release >/dev/null 2>&1; then
        echo "  \"os_version\": \"$(lsb_release -d | cut -f2)\","
        echo "  \"os_codename\": \"$(lsb_release -c | cut -f2)\","
    else
        echo "  \"os_version\": \"$(cat /etc/os-release | grep PRETTY_NAME | cut -d'=' -f2 | tr -d '"')\","
    fi
    echo "  \"kernel_version\": \"$(uname -r)\","
    echo "  \"cpu_model\": \"$(lscpu | grep 'Model name' | cut -d':' -f2 | xargs)\","
    echo "  \"cpu_cores\": $(lscpu | grep '^CPU(s):' | awk '{print $2}'),"
    echo "  \"cpu_physical_cores\": $(lscpu | grep '^Socket(s):' | awk '{print $2}'),"
    echo "  \"cpu_threads_per_core\": $(lscpu | grep 'Thread(s) per core' | awk '{print $4}'),"
    echo "  \"cpu_max_freq_mhz\": $(lscpu | grep 'CPU max MHz' | awk '{print $4}' | cut -d'.' -f1),"
    echo "  \"memory_total_gb\": $(free -g | grep Mem | awk '{print $2}'),"
    echo "  \"memory_available_gb\": $(free -g | grep Mem | awk '{print $7}'),"
    echo "  \"memory_used_gb\": $(free -g | grep Mem | awk '{print $3}'),"
    echo "  \"disk_info\": ["
    df -h | tail -n +2 | while read -r line; do
        filesystem=$(echo "$line" | awk '{print $1}')
        size=$(echo "$line" | awk '{print $2}')
        used=$(echo "$line" | awk '{print $3}')
        available=$(echo "$line" | awk '{print $4}')
        use_percent=$(echo "$line" | awk '{print $5}')
        mount=$(echo "$line" | awk '{print $6}')
        echo "    {"
        echo "      \"filesystem\": \"$filesystem\","
        echo "      \"size\": \"$size\","
        echo "      \"used\": \"$used\","
        echo "      \"available\": \"$available\","
        echo "      \"use_percent\": \"$use_percent\","
        echo "      \"mount\": \"$mount\""
        echo "    }$(if [ "$line" != "$(df -h | tail -n 1)" ]; then echo ','; fi)"
    done | sed '$ s/,$//'
    echo "  ],"
fi

# Network Information
echo "  \"network_interfaces\": ["
if [[ "$OS" == "Darwin" ]]; then
    networksetup -listallhardwareports | grep -A 1 "Hardware Port" | while read -r line1; do
        read -r line2
        interface=$(echo "$line2" | awk '{print $2}')
        if [ -n "$interface" ]; then
            ip=$(ipconfig getifaddr "$interface" 2>/dev/null || echo "")
            mac=$(ifconfig "$interface" 2>/dev/null | grep ether | awk '{print $2}' || echo "")
            if [ -n "$ip" ] || [ -n "$mac" ]; then
                echo "    {"
                echo "      \"name\": \"$interface\","
                echo "      \"ip_address\": \"$ip\","
                echo "      \"mac_address\": \"$mac\""
                echo "    },"
            fi
        fi
    done | sed '$ s/,$//'
elif [[ "$OS" == "Linux" ]]; then
    ip -j addr show | jq -r '.[] | select(.addr_info) | {name: .ifname, ip_address: .addr_info[0].local, mac_address: .address}' | sed 's/^/    /' | sed '$ s/,$//' || true
fi
echo "  ],"

# Development Tools
echo "  \"development_tools\": {"
echo "    \"git_version\": \"$(git --version | cut -d' ' -f3)\","
echo "    \"cmake_version\": \"$(cmake --version | head -n 1 | cut -d' ' -f3)\","
if command -v cargo >/dev/null 2>&1; then
    echo "    \"rust_version\": \"$(rustc --version | cut -d' ' -f2)\","
fi
if command -v go >/dev/null 2>&1; then
    echo "    \"go_version\": \"$(go version | cut -d' ' -f3 | sed 's/go//')\","
fi
if command -v node >/dev/null 2>&1; then
    echo "    \"node_version\": \"$(node --version | sed 's/v//')\","
fi
if command -v python3 >/dev/null 2>&1; then
    echo "    \"python3_version\": \"$(python3 --version | cut -d' ' -f2)\","
fi
echo "    \"cursor_agent_version\": \"$(cursor --version 2>/dev/null || echo 'unknown')\""
echo "  }"

echo "}"
