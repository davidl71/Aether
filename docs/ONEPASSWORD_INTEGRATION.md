## 1Password Integration

The project can pull credentials directly from 1Password so secrets never land in source control. This includes:
- Distcc host credentials
- Cursor remote development credentials
- Alpaca API credentials
- Other service credentials

### Requirements

- [1Password CLI (`op`)](https://developer.1password.com/docs/cli)
- Authentication method (choose one):
  - **Personal account**: Signed-in session (`op signin …`) - for local development
  - **Service Account**: `OP_SERVICE_ACCOUNT_TOKEN` environment variable - for CI/CD and automation
- Optional: Cursor 1Password extension (for inline secret references)

See [1Password Secrets Automation](https://developer.1password.com/docs/secrets-automation) for details on Service Accounts vs Connect Servers.

### Sync distcc host from 1Password

Use `scripts/op_sync_distcc_host.sh` to populate:

- `ansible/hosts`
- `~/.ssh/<alias>_id_ed25519` and SSH config
- `~/.distcc/hosts`
- `~/.zshrc` (`DISTCC_HOSTS` export)

```bash
export OP_DISTCC_HOST_SECRET="op://Engineering/Distcc M4/host"
export OP_DISTCC_USER_SECRET="op://Engineering/Distcc M4/username"
export OP_DISTCC_KEY_SECRET="op://Engineering/Distcc M4/private key"
# optional
export OP_DISTCC_CORES_SECRET="op://Engineering/Distcc M4/cores"
export DISTCC_REMOTE_ALIAS="distcc-m4"

./scripts/op_sync_distcc_host.sh
```

Then run the provisioning playbook:

```bash
ansible-playbook -i ansible/hosts ansible/playbooks/setup_distcc_macos.yml
```

### Cursor references

You can reference the same secrets inside Cursor prompts using the extension, e.g.:

```
op://Engineering/Distcc M4/host
op://Engineering/Distcc M4/username
op://Engineering/Distcc M4/private key
```

### Notes

- `OP_DISTCC_*` variables accept any 1Password item paths.
- The script rewrites `ansible/hosts` for the `distcc_macos_workers` group each run.
- Update `DISTCC_REMOTE_ALIAS` or `DISTCC_REMOTE_CORES` to match new hosts.

### Sync Cursor remote development from 1Password

Use `scripts/op_sync_cursor_remote.sh` to populate SSH config for Cursor remote development:

- `~/.ssh/<alias>_id_ed25519` and SSH config
- Cursor-optimized SSH settings (compression, keep-alive, connection multiplexing)

```bash
export OP_CURSOR_REMOTE_HOST_SECRET="op://Engineering/Cursor Remote M4/host"
export OP_CURSOR_REMOTE_USER_SECRET="op://Engineering/Cursor Remote M4/username"
export OP_CURSOR_REMOTE_KEY_SECRET="op://Engineering/Cursor Remote M4/private key"
# optional
export OP_CURSOR_REMOTE_PORT_SECRET="op://Engineering/Cursor Remote M4/port"
export CURSOR_REMOTE_ALIAS="cursor-m4-mac"

./scripts/op_sync_cursor_remote.sh
```

After running the script:

1. Install Remote-SSH extension in Cursor (`anysphere.remote-ssh`) if not already installed
2. In Cursor, open Command Palette (`⌘+Shift+P`) and select "Remote-SSH: Connect to Host"
3. Choose your configured alias (e.g., `cursor-m4-mac`) from the list
4. Wait for VS Code Server to install on remote Mac (first connection only)

**SSH Settings Included:**
- Compression enabled for better performance over slow networks
- Keep-alive settings to prevent connection timeouts
- Connection multiplexing for faster subsequent connections
- Strict host key checking with auto-accept for first connection

See [Remote Development Workflow](./REMOTE_DEVELOPMENT_WORKFLOW.md) for complete setup instructions.

### Notes
- `OP_CURSOR_REMOTE_*` variables accept any 1Password item paths.
- The script updates `~/.ssh/config` with Cursor-optimized settings.
- Update `CURSOR_REMOTE_ALIAS` to match your preferred SSH host alias.

## Alpaca API Credentials

Use 1Password for Alpaca API credentials when running the PWA service:

```bash
export OP_ALPACA_API_KEY_ID_SECRET="op://Vault/Item Name/API Key ID"
export OP_ALPACA_API_SECRET_KEY_SECRET="op://Vault/Item Name/API Secret Key"

./web/scripts/run-alpaca-service.sh
```

The script will automatically:
1. Try to read from 1Password if `OP_ALPACA_*_SECRET` variables are set
2. Fall back to `ALPACA_API_KEY_ID` and `ALPACA_API_SECRET_KEY` environment variables if 1Password is not available

**Authentication Methods:**
- **Personal Account**: Run `op signin` first (for local development)
- **Service Account**: Set `OP_SERVICE_ACCOUNT_TOKEN` (for CI/CD, see [Service Accounts docs](https://developer.1password.com/docs/service-accounts))

See `web/ALPACA_INTEGRATION.md` for complete setup instructions.
