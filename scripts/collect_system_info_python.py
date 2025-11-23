#!/usr/bin/env python3
"""
collect_system_info_python.py - Cross-platform system information collection

Generates comprehensive system information JSON for environment documentation.
Works on both macOS and Linux.
"""

import json
import platform
import subprocess
import sys
from datetime import datetime, UTC
from pathlib import Path


def run_command(cmd, shell=False, decode=True):
    """Run a command and return output."""
    try:
        result = subprocess.run(
            cmd if isinstance(cmd, list) else cmd.split(),
            shell=shell,
            capture_output=True,
            text=decode,
            check=False
        )
        return result.stdout.strip() if result.returncode == 0 else ""
    except Exception:
        return ""


def get_macos_info():
    """Collect macOS system information."""
    info = {}

    # OS Version
    info["os_version"] = run_command("sw_vers -productVersion")
    info["os_build"] = run_command("sw_vers -buildVersion")
    info["os_name"] = run_command("sw_vers -productName")

    # CPU Information
    cpu_brand = run_command("sysctl -n machdep.cpu.brand_string")
    info["cpu_brand"] = cpu_brand
    info["cpu_cores"] = int(run_command("sysctl -n hw.ncpu") or "0")
    info["cpu_physical_cores"] = int(run_command("sysctl -n hw.physicalcpu") or "0")
    info["cpu_logical_cores"] = int(run_command("sysctl -n hw.logicalcpu") or "0")

    # Memory
    memsize = int(run_command("sysctl -n hw.memsize") or "0")
    info["memory_total_gb"] = memsize // (1024 ** 3)

    # Apple Intelligence
    info["apple_intelligence_available"] = "Apple" in cpu_brand
    info["neural_engine_available"] = any(m in cpu_brand for m in ["M1", "M2", "M3", "M4"])

    # Disk Information
    df_output = run_command("df -h")
    info["disk_info"] = []
    for line in df_output.split("\n")[1:]:
        parts = line.split()
        if len(parts) >= 9:
            info["disk_info"].append({
                "filesystem": parts[0],
                "size": parts[1],
                "used": parts[2],
                "available": parts[3],
                "use_percent": parts[4],
                "mount": parts[8] if len(parts) > 8 else ""
            })

    # Network Interfaces
    info["network_interfaces"] = []
    interfaces = run_command("networksetup -listallhardwareports")
    current_interface = None
    for line in interfaces.split("\n"):
        if "Hardware Port:" in line:
            current_interface = line.split(":")[1].strip()
        elif "Device:" in line and current_interface:
            interface_name = line.split(":")[1].strip()
            ip = run_command(f"ipconfig getifaddr {interface_name}")
            mac_output = run_command(f"ifconfig {interface_name}")
            mac = ""
            for mac_line in mac_output.split("\n"):
                if "ether" in mac_line:
                    mac = mac_line.split()[1]
                    break

            if ip or mac:
                info["network_interfaces"].append({
                    "name": interface_name,
                    "description": current_interface,
                    "ip_address": ip,
                    "mac_address": mac
                })

    return info


def get_linux_info():
    """Collect Linux system information."""
    info = {}

    # OS Version
    if Path("/etc/os-release").exists():
        os_release = Path("/etc/os-release").read_text()
        for line in os_release.split("\n"):
            if line.startswith("PRETTY_NAME="):
                info["os_version"] = line.split("=", 1)[1].strip('"')
                break
    else:
        info["os_version"] = platform.platform()

    info["kernel_version"] = platform.release()

    # CPU Information
    lscpu_output = run_command("lscpu")
    cpu_info = {}
    for line in lscpu_output.split("\n"):
        if ":" in line:
            key, value = line.split(":", 1)
            cpu_info[key.strip()] = value.strip()

    info["cpu_model"] = cpu_info.get("Model name", "")
    info["cpu_cores"] = int(cpu_info.get("CPU(s)", "0"))
    info["cpu_physical_cores"] = int(cpu_info.get("Socket(s)", "0"))
    info["cpu_threads_per_core"] = int(cpu_info.get("Thread(s) per core", "1"))
    if "CPU max MHz" in cpu_info:
        info["cpu_max_freq_mhz"] = int(float(cpu_info["CPU max MHz"]))

    # Memory
    free_output = run_command("free -g")
    for line in free_output.split("\n"):
        if line.startswith("Mem:"):
            parts = line.split()
            info["memory_total_gb"] = int(parts[1])
            info["memory_used_gb"] = int(parts[2])
            info["memory_available_gb"] = int(parts[6]) if len(parts) > 6 else int(parts[3])
            break

    # Disk Information
    df_output = run_command("df -h")
    info["disk_info"] = []
    for line in df_output.split("\n")[1:]:
        parts = line.split()
        if len(parts) >= 6:
            info["disk_info"].append({
                "filesystem": parts[0],
                "size": parts[1],
                "used": parts[2],
                "available": parts[3],
                "use_percent": parts[4],
                "mount": parts[5]
            })

    # Network Interfaces
    info["network_interfaces"] = []
    try:
        ip_output = run_command("ip -j addr show")
        if ip_output:
            import json as json_lib
            interfaces = json_lib.loads(ip_output)
            for iface in interfaces:
                if "addr_info" in iface and iface["addr_info"]:
                    info["network_interfaces"].append({
                        "name": iface.get("ifname", ""),
                        "ip_address": iface["addr_info"][0].get("local", ""),
                        "mac_address": iface.get("address", "")
                    })
    except Exception:
        # Fallback to ifconfig
        ifconfig_output = run_command("ifconfig -a")
        current_interface = None
        for line in ifconfig_output.split("\n"):
            if line and not line.startswith(" ") and not line.startswith("\t"):
                parts = line.split(":")
                current_interface = parts[0] if parts else None
            elif current_interface and "inet " in line:
                parts = line.split()
                ip = parts[1] if len(parts) > 1 else ""
                if ip and ip != "127.0.0.1":
                    info["network_interfaces"].append({
                        "name": current_interface,
                        "ip_address": ip
                    })

    return info


def get_development_tools():
    """Collect development tool versions."""
    tools = {}

    version_commands = {
        "git_version": ["git", "--version"],
        "cmake_version": ["cmake", "--version"],
        "rust_version": ["rustc", "--version"],
        "go_version": ["go", "version"],
        "node_version": ["node", "--version"],
        "python3_version": ["python3", "--version"]
    }

    for key, cmd in version_commands.items():
        output = run_command(cmd)
        if output:
            if key == "git_version":
                tools[key] = output.split()[2]
            elif key == "cmake_version":
                tools[key] = output.split()[2]
            elif key == "rust_version":
                tools[key] = output.split()[1]
            elif key == "go_version":
                tools[key] = output.split()[2].replace("go", "")
            elif key == "node_version":
                tools[key] = output.replace("v", "")
            elif key == "python3_version":
                tools[key] = output.split()[1]

    # Cursor version
    cursor_version = run_command("cursor --version")
    tools["cursor_agent_version"] = cursor_version if cursor_version else "unknown"

    return tools


def main():
    """Main function to collect and output system information."""
    system = platform.system()

    data = {
        "collection_date": datetime.now(datetime.UTC).isoformat().replace("+00:00", "Z"),
        "hostname": platform.node(),
        "operating_system": system
    }

    # Collect OS-specific information
    if system == "Darwin":
        data.update(get_macos_info())
    elif system == "Linux":
        data.update(get_linux_info())
    else:
        data["error"] = f"Unsupported operating system: {system}"

    # Collect development tools
    data["development_tools"] = get_development_tools()

    # Output JSON
    print(json.dumps(data, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
