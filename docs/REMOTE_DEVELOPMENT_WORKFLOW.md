# Remote Development Workflow with Cursor

This document describes how to integrate Cursor IDE with a remote M4 Mac accessible via SSH for distributed development workflows.

## Overview

**Yes, Cursor can be integrated with a remote M4 Mac via SSH**, leveraging the existing project infrastructure for remote access. This setup allows you to:

- Develop directly on the remote M4 Mac using Cursor's full feature set
- Leverage the M4's compute power for compilation and testing
- Use Cursor's AI features with code running on the remote machine
- Maintain a consistent development environment across machines

## Prerequisites

1. **Remote M4 Mac Setup:**
   - SSH enabled (System Settings > General > Sharing > Remote Login)
   - User account with appropriate permissions
   - Network access from your local machine

2. **Local Machine:**
   - Cursor IDE installed
   - SSH client configured
   - Access to 1Password (optional, for credential management)

3. **Existing Project Infrastructure:**
   - The project already has SSH configuration patterns via `scripts/op_sync_distcc_host.sh`
   - 1Password integration for secure credential management
   - Ansible playbooks for remote Mac provisioning

## Installation

### Step 1: Install Remote-SSH Extension

1. Open Cursor on your local machine
2. Navigate to Extensions view (`⌘+Shift+X` / `Ctrl+Shift+X`)
3. Search for "Remote - SSH" or install `anysphere.remote-ssh`
4. Install the extension (by Anysphere)

### Step 2: Configure SSH Access

You can use the existing project infrastructure or configure manually:

#### Option A: Use Existing 1Password Integration (Recommended)

The project provides a dedicated script for Cursor remote development SSH configuration:

```bash
export OP_CURSOR_REMOTE_HOST_SECRET="op://Engineering/Cursor Remote M4/host"
export OP_CURSOR_REMOTE_USER_SECRET="op://Engineering/Cursor Remote M4/username"
export OP_CURSOR_REMOTE_KEY_SECRET="op://Engineering/Cursor Remote M4/private key"

# optional

export OP_CURSOR_REMOTE_PORT_SECRET="op://Engineering/Cursor Remote M4/port"
export CURSOR_REMOTE_ALIAS="cursor-m4-mac"

./scripts/op_sync_cursor_remote.sh
```

This script automatically:

- Reads credentials from 1Password
- Creates SSH key file with correct permissions
- Updates `~/.ssh/config` with Cursor-optimized settings (compression, keep-alive, connection multiplexing)
- Removes old entries and adds updated configuration

See [1Password Integration](./ONEPASSWORD_INTEGRATION.md) for more details.

#### Option B: Manual SSH Configuration

Add to `~/.ssh/config`:

```
Host cursor-m4-mac
  HostName <remote_mac_ip_or_hostname>
  User <your_username>
  IdentityFile ~/.ssh/cursor_m4_id_ed25519
  StrictHostKeyChecking accept-new
  IdentitiesOnly yes
```

### Step 3: Connect to Remote Mac

1. Open Command Palette (`⌘+Shift+P` / `Ctrl+Shift+P`)
2. Select "Remote-SSH: Connect to Host"
3. Choose your configured host (e.g., `cursor-m4-mac`)
4. When prompted, select "macOS" as the remote OS
5. Accept the host fingerprint if prompted

**First Connection:** Cursor will automatically download and install the VS Code Server on the remote Mac. This may take a few minutes.

### Step 4: Open Workspace

After connection:

1. A new Cursor window opens for the remote connection
2. Open your workspace folder (File > Open Folder)
3. Navigate to your project directory on the remote Mac

## Configuration

### SSH Server Optimization (Remote Mac)

To prevent SSH timeouts and improve stability, configure the SSH server on your remote Mac:

```bash

# Edit SSH daemon config on remote Mac

sudo nano /etc/ssh/sshd_config
```

Add or modify:

```
AllowTcpForwarding yes
ClientAliveInterval 30
ClientAliveCountMax 10
```

Restart SSH service:

```bash

# macOS uses launchd

sudo launchctl unload /System/Library/LaunchDaemons/ssh.plist
sudo launchctl load -w /System/Library/LaunchDaemons/ssh.plist

# Or simply restart the remote Mac
```

### Cursor Configuration (Local Machine)

Optimize workspace analysis in `~/.cursor/config.json`:

```json
{
  "python.analysis.include": ["src/**/*"],
  "remote.SSH.lockReconnect": false,
  "remote.SSH.connectTimeout": 60,
  "remote.SSH.useLocalServer": false
}
```

## Known Issues & Workarounds

### Issue 1: VS Code Server Download Failures

**Symptom:** "Failed to download VS Code Server (Server returned 404)"

**Solutions:**

1. **Manual Server Installation:**
   - SSH into remote Mac manually
   - Download VS Code Server manually from [VS Code releases](https://code.visualstudio.com/sha/download?build=stable&os=cli-alpine-x64)
   - Extract to `~/.vscode-server/bin/<commit-hash>/`

2. **Use Microsoft VS Code:**
   - If persistent issues, consider using official VS Code for remote development
   - Remote-SSH extension has more mature support there

3. **Check Network/Proxy:**
   - Ensure remote Mac can access internet/download VS Code releases
   - Configure proxy if needed

### Issue 2: SSH Timeouts

**Symptom:** Connection drops or times out frequently

**Solutions:**

1. Configure SSH server keep-alive (see Configuration section above)
2. Increase Cursor's SSH timeout in `~/.cursor/config.json`:

   ```json
   {
     "remote.SSH.connectTimeout": 120
   }
   ```

3. Use SSH multiplexing (add to `~/.ssh/config`):

   ```
   ControlMaster auto
   ControlPath ~/.ssh/control:%h:%p:%r
   ControlPersist 10m
   ```

### Issue 3: Extension Compatibility

**Symptom:** Some extensions don't work on remote

**Solutions:**

- Most Cursor extensions work on remote, but some may have limitations
- Check extension documentation for remote support
- Install extensions on remote after connecting

### Issue 4: Performance Considerations

**Recommendations:**

- Use SSH key-based authentication (no passwords)
- Optimize network connection (prefer wired or high-speed WiFi)
- Configure SSH compression if network is slow:

  ```
  Compression yes
  ```

- Use local file watching judiciously (exclude `node_modules`, `build/`, etc.)

## Workflow Integration

### Development Workflow

1. **Local Development:**
   - Use Cursor locally for quick edits, documentation
   - Leverage local machine for lightweight tasks

2. **Remote Development:**
   - Connect to remote M4 Mac for:
     - Heavy compilation (C++ builds)
     - Testing on macOS
     - Multi-platform development
     - Resource-intensive tasks

3. **Hybrid Approach:**
   - Develop locally, compile remotely
   - Use remote for CI-like testing environments
   - Sync code via git between local and remote

### Integration with Existing Tools

#### Distcc Remote Compilation

The remote M4 Mac can serve dual purposes:

- **Cursor Remote Development:** Full IDE access
- **Distcc Compilation Worker:** Distributed C++ builds

Both can run simultaneously - just use different SSH aliases:

- `cursor-m4-mac` for Cursor Remote-SSH
- `distcc-m4` for distcc compilation (existing setup)

#### Git Workflow

Recommended approach:

1. **Single Source of Truth:** Use GitHub as primary repo
2. **Both Machines Sync:** Pull/push to same repo
3. **Branch Strategy:** Use feature branches for work on either machine
4. **Conflict Prevention:** Coordinate work or use different branches per machine

## Alternative Solutions

If Cursor Remote-SSH proves challenging:

### 1. Microsoft VS Code Remote-SSH

- More mature remote development support
- Full feature parity with local development
- Better compatibility with Remote-SSH extension

### 2. Apple Remote Desktop (ARD)

- Full desktop access to remote Mac
- Native macOS integration
- Better for GUI-heavy workflows
- Requires macOS Server or ARD license

### 3. Cloud Solutions

- **AWS EC2 Mac Instances:** M1 Mac instances accessible via SSH
- **MacRDS:** Dedicated macOS environments via VNC/SSH
- Flexible, scalable, no physical hardware maintenance

## Security Considerations

1. **SSH Keys:**
   - Use Ed25519 keys (stronger, faster)
   - Never commit keys to repository
   - Use 1Password for key management (existing pattern)

2. **Network Security:**
   - Use VPN if accessing over public internet
   - Consider SSH key-only authentication (disable passwords)
   - Use firewall rules to limit SSH access

3. **Credentials:**
   - Leverage existing 1Password integration
   - Never store credentials in code/config files
   - Use environment variables or secure vaults

## Troubleshooting

### Connection Won't Establish

1. **Test SSH manually:**

   ```bash
   ssh cursor-m4-mac
   ```

2. **Check SSH config:**

   ```bash
   cat ~/.ssh/config | grep -A 10 cursor-m4-mac
   ```

3. **Verify network access:**

   ```bash
   ping <remote_mac_ip>
   ```

### Remote Extensions Won't Install

1. **Check remote permissions:**

   ```bash
   ssh cursor-m4-mac "ls -la ~/.vscode-server"
   ```

2. **Manually install extensions:**
   - Connect via SSH
   - Download extension VSIX
   - Install via Cursor UI or CLI

### Performance Issues

1. **Network:**
   - Test bandwidth: `speedtest-cli` on remote Mac
   - Enable SSH compression if slow
   - Consider local development for small changes

2. **Remote Machine:**
   - Check CPU/memory usage: `htop`
   - Close unnecessary applications
   - Ensure sufficient disk space

## References

- [Cursor Remote Development Demo](https://notes.kodekloud.com/docs/Cursor-AI/Understanding-and-Customizing-Cursor/Demo-Remote-Development)
- [VS Code Remote Development](https://code.visualstudio.com/docs/remote/remote-overview)
- [Project SSH Configuration Script](../scripts/op_sync_distcc_host.sh)
- [1Password Integration](./ONEPASSWORD_INTEGRATION.md)
- [Distributed Compilation Setup]( DISTRIBUTED_COMPILATION.md)

## Next Steps

1. **Try Connection:** Follow Installation steps above
2. **Test Workflow:** Open a project and verify functionality
3. **Optimize:** Configure based on your usage patterns
4. **Document Issues:** Note any problems specific to your setup
5. **Share Feedback:** Update this doc with your experiences

---

**Note:** This integration leverages Cursor's VS Code-compatible remote development features. Some edge cases may exist, but overall support is solid for most development workflows.
