#!/usr/bin/env bash
# op_sync_distcc_host.sh - Populate distcc inventory and ssh config from 1Password

set -euo pipefail

if ! command -v op >/dev/null 2>&1; then
  echo "[op-sync] 1Password CLI (op) is not installed or not in PATH." >&2
  exit 1
fi

HOST_SECRET=${OP_DISTCC_HOST_SECRET:-"op://Engineering/Distcc M4/host"}
USER_SECRET=${OP_DISTCC_USER_SECRET:-"op://Engineering/Distcc M4/username"}
KEY_SECRET=${OP_DISTCC_KEY_SECRET:-"op://Engineering/Distcc M4/private key"}
CORES_SECRET=${OP_DISTCC_CORES_SECRET:-""}
PORT_SECRET=${OP_DISTCC_PORT_SECRET:-""}

ALIAS=${DISTCC_REMOTE_ALIAS:-distcc-m4}
DEFAULT_CORES=${DISTCC_REMOTE_CORES:-12}

ANSIBLE_HOSTS_FILE=${ANSIBLE_HOSTS_FILE:-"ansible/hosts"}
SSH_DIR="${HOME}/.ssh"
DISTCC_HOSTS_FILE="${HOME}/.distcc/hosts"
ZSHRC="${HOME}/.zshrc"

host=$(op read "$HOST_SECRET" | tr -d '[:space:]')
user=$(op read "$USER_SECRET" | tr -d '[:space:]')
key=$(op read "$KEY_SECRET")

if [[ -z "$host" || -z "$user" || -z "$key" ]]; then
  echo "[op-sync] Failed to read host, user, or key from 1Password secrets." >&2
  exit 1
fi

if [[ -n "$CORES_SECRET" ]]; then
  cores=$(op read "$CORES_SECRET" | tr -d '[:space:]')
else
  cores="$DEFAULT_CORES"
fi

if [[ -z "$cores" ]]; then
  cores=12
fi

if [[ -n "$PORT_SECRET" ]]; then
  port=$(op read "$PORT_SECRET" | tr -d '[:space:]')
else
  port=""
fi

mkdir -p "$(dirname "$ANSIBLE_HOSTS_FILE")"

ssh_key_path="${SSH_DIR}/${ALIAS}_id_ed25519"
mkdir -p "$SSH_DIR"
printf '%s\n' "$key" >"$ssh_key_path"
chmod 600 "$ssh_key_path"

# Write Ansible inventory (overwrites distcc group section with latest values)
cat >"$ANSIBLE_HOSTS_FILE" <<EOF
[distcc_macos_workers]
$ALIAS ansible_host=$host ansible_user=$user ansible_port=${port:-22} ansible_ssh_private_key_file=$ssh_key_path
EOF

# Update ~/.ssh/config entry
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
result.append(f"  IdentityFile {key_path}")
result.append("  StrictHostKeyChecking accept-new")
result.append("  IdentitiesOnly yes")
result.append("")

config_path.write_text("\n".join(result).rstrip() + "\n")
config_path.chmod(0o600)
PY

# Update ~/.distcc/hosts
mkdir -p "$(dirname "$DISTCC_HOSTS_FILE")"
python3 - "$DISTCC_HOSTS_FILE" "$host" "$cores" <<'PY'
import sys, pathlib

path = pathlib.Path(sys.argv[1])
host = sys.argv[2]
cores = sys.argv[3]

entries = []
if path.exists():
    entries = [line.strip() for line in path.read_text().splitlines() if line.strip()]

def add_entry(text):
    if text not in entries:
        entries.append(text)

add_entry("localhost/4")
add_entry(f"{host}/{cores},lzo")

path.write_text("\n".join(entries) + "\n")
PY

# Ensure shell profile exports DISTCC_HOSTS
if [[ -f "$ZSHRC" ]]; then
  python3 - "$ZSHRC" "$host" "$cores" <<'PY'
import sys, pathlib

path = pathlib.Path(sys.argv[1])
host = sys.argv[2]
cores = sys.argv[3]
export_line = f'export DISTCC_HOSTS="localhost/4 {host}/{cores},lzo"'

lines = []
if path.exists():
    lines = [line.rstrip("\n") for line in path.read_text().splitlines()]

lines = [line for line in lines if not line.startswith("export DISTCC_HOSTS=")]
lines.append(export_line)

path.write_text("\n".join(lines) + "\n")
PY
fi

echo "[op-sync] Updated inventory, SSH config, and distcc hosts using 1Password secrets."
echo "  Host alias : $ALIAS"
echo "  Host       : $host"
echo "  User       : $user"
echo "  SSH key    : $ssh_key_path"
echo "  Cores      : $cores"
echo ""
echo "Run provisioning:"
echo "  ansible-playbook -i ansible/hosts ansible/playbooks/setup_distcc_macos.yml"
