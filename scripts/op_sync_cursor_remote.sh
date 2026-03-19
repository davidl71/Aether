#!/usr/bin/env bash
# op_sync_cursor_remote.sh - Populate SSH config for Cursor remote development from 1Password

set -euo pipefail

if ! command -v op >/dev/null 2>&1; then
  echo "[op-sync] 1Password CLI (op) is not installed or not in PATH." >&2
  exit 1
fi

HOST_SECRET=${OP_CURSOR_REMOTE_HOST_SECRET:-"op://Engineering/Cursor Remote M4/host"}
USER_SECRET=${OP_CURSOR_REMOTE_USER_SECRET:-"op://Engineering/Cursor Remote M4/username"}
KEY_SECRET=${OP_CURSOR_REMOTE_KEY_SECRET:-""}
PORT_SECRET=${OP_CURSOR_REMOTE_PORT_SECRET:-""}

ALIAS=${CURSOR_REMOTE_ALIAS:-"Cursor Remote - SSH"}

SSH_DIR="${HOME}/.ssh"

host=$(op read "$HOST_SECRET" | tr -d '[:space:]')
user=$(op read "$USER_SECRET" | tr -d '[:space:]')

if [[ -z "$host" || -z "$user" ]]; then
  echo "[op-sync] Failed to read host or user from 1Password secrets." >&2
  exit 1
fi

# Private key is optional - only read if KEY_SECRET is provided
if [[ -n "$KEY_SECRET" ]]; then
  key=$(op read "$KEY_SECRET" 2>&1) || {
    echo "[op-sync] Warning: Could not read private key from '$KEY_SECRET'. Continuing without key." >&2
    key=""
  }
else
  key=""
fi

if [[ -n "$PORT_SECRET" ]]; then
  port=$(op read "$PORT_SECRET" | tr -d '[:space:]')
else
  port=""
fi

# Only create SSH key file if we have a key
ssh_key_path=""
if [[ -n "$key" ]]; then
  ssh_key_path="${SSH_DIR}/${ALIAS}_id_ed25519"
  mkdir -p "$SSH_DIR"
  printf '%s\n' "$key" >"$ssh_key_path"
  chmod 600 "$ssh_key_path"
fi

# Update ~/.ssh/config entry with Cursor-optimized settings
ssh_config="${SSH_DIR}/config"
python3 - "$ssh_config" "$ALIAS" "$host" "$user" "$ssh_key_path" "$port" <<'PY'
import sys, pathlib

config_path = pathlib.Path(sys.argv[1])
alias = sys.argv[2]
host = sys.argv[3]
user = sys.argv[4]
key_path = sys.argv[5]
port = sys.argv[6]

if config_path.exists():
    lines = config_path.read_text().splitlines()
else:
    lines = []

result = []
i = 0
while i < len(lines):
    line = lines[i]
    if line.strip() == f"Host {alias}":
        i += 1
        while i < len(lines) and lines[i].strip() and not lines[i].startswith("Host "):
            i += 1
        continue
    result.append(line)
    i += 1

result.append(f"Host {alias}")
result.append(f"  HostName {host}")
result.append(f"  User {user}")
if port:
    result.append(f"  Port {port}")
if key_path:
    result.append(f"  IdentityFile {key_path}")
    result.append("  IdentitiesOnly yes")
result.append("  StrictHostKeyChecking accept-new")
result.append("  Compression yes")
result.append("  ServerAliveInterval 30")
result.append("  ServerAliveCountMax 10")
result.append("  ControlMaster auto")
result.append("  ControlPath ~/.ssh/control-%h-%p-%r")
result.append("  ControlPersist 10m")
result.append("")

config_path.write_text("\n".join(result).rstrip() + "\n")
config_path.chmod(0o600)
PY

echo "[op-sync] Updated SSH config for Cursor remote development using 1Password secrets."
echo "  Host alias : $ALIAS"
echo "  Host       : $host"
echo "  User       : $user"
if [[ -n "$ssh_key_path" ]]; then
  echo "  SSH key    : $ssh_key_path"
else
  echo "  SSH key    : (not provided - using default SSH keys)"
fi
if [[ -n "$port" ]]; then
  echo "  Port       : $port"
fi
echo ""
echo "Next steps:"
echo "  1. Install Remote-SSH extension in Cursor (anysphere.remote-ssh) if not already installed"
echo "  2. In Cursor, open Command Palette (⌘+Shift+P) and select 'Remote-SSH: Connect to Host'"
echo "  3. Choose '$ALIAS' from the list"
echo "  4. Wait for VS Code Server to install on remote Mac (first connection only)"
