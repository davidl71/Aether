# SSH Setup for Nightly Automation

The nightly automation tool requires SSH access to remote hosts for parallel task execution. This guide explains how to set up SSH keys for automated access.

## Current Configuration

The automation tool is configured to connect to:

1. **Ubuntu host**: `david@192.168.192.57`
2. **macOS host**: `davidl@192.168.192.141`

## SSH Key Setup

### Step 1: Generate SSH Key (if not already done)

```bash

# Check if you have an SSH key

ls -la ~/.ssh/id_*

# If no key exists, generate one

ssh-keygen -t ed25519 -C "your_email@example.com"
```

### Step 2: Copy Public Key to Remote Hosts

#### For macOS host (192.168.192.141)

```bash

# Copy your public key to the remote host

ssh-copy-id davidl@192.168.192.141

# Or manually:

cat ~/.ssh/id_ed25519.pub | ssh davidl@192.168.192.141 "mkdir -p ~/.ssh && cat >> ~/.ssh/authorized_keys"
```

#### For Ubuntu host (192.168.192.57)

```bash

# Copy your public key to the remote host

ssh-copy-id david@192.168.192.57

# Or manually:

cat ~/.ssh/id_ed25519.pub | ssh david@192.168.192.57 "mkdir -p ~/.ssh && cat >> ~/.ssh/authorized_keys"
```

### Step 3: Test SSH Connection

```bash

# Test macOS host

ssh -o ConnectTimeout=5 davidl@192.168.192.141 "echo 'Connection successful'"

# Test Ubuntu host

ssh -o ConnectTimeout=5 david@192.168.192.57 "echo 'Connection successful'"
```

### Step 4: Add Host Keys to known_hosts

The automation tool will automatically accept new host keys, but you can pre-add them:

```bash

# Add macOS host

ssh-keyscan -H 192.168.192.141 >> ~/.ssh/known_hosts

# Add Ubuntu host (when it's reachable)

ssh-keyscan -H 192.168.192.57 >> ~/.ssh/known_hosts
```

## Troubleshooting

### Issue: "Permission denied (publickey)"

**Cause**: SSH public key not authorized on remote host.

**Solution**:

1. Copy your public key to the remote host (see Step 2 above)
2. Verify permissions on remote host:

   ```bash
   ssh davidl@192.168.192.141 "chmod 700 ~/.ssh && chmod 600 ~/.ssh/authorized_keys"
   ```

### Issue: "Host key verification failed"

**Cause**: Host key not in known_hosts.

**Solution**:

```bash

# Remove old host key (if exists)

ssh-keygen -R 192.168.192.141

# Add new host key

ssh-keyscan -H 192.168.192.141 >> ~/.ssh/known_hosts
```

### Issue: "Connection timed out"

**Cause**: Host is unreachable or SSH service not running.

**Solution**:

1. Check if host is reachable: `ping 192.168.192.141`
2. Check if SSH service is running on remote host
3. Check firewall settings
4. Verify network connectivity

### Issue: "Too many authentication failures"

**Cause**: SSH trying multiple keys, hitting rate limit.

**Solution**: The automation tool now uses `IdentitiesOnly=yes` to only use the specified key.

## SSH Configuration (Optional)

You can create `~/.ssh/config` for easier connection:

```ssh-config
Host ubuntu-agent
    HostName 192.168.192.57
    User david
    IdentityFile ~/.ssh/id_ed25519
    StrictHostKeyChecking accept-new
    IdentitiesOnly yes

Host macos-agent
    HostName 192.168.192.141
    User davidl
    IdentityFile ~/.ssh/id_ed25519
    StrictHostKeyChecking accept-new
    IdentitiesOnly yes
```

Then update the automation tool configuration to use these host aliases.

## Verification

After setup, verify SSH access works:

```bash

# Test macOS host

ssh davidl@192.168.192.141 "cd ~/Projects/Trading/ib_box_spread_full_universal && git status"

# Test Ubuntu host (when reachable)

ssh david@192.168.192.57 "cd ib_box_spread_full_universal && git status"
```

## Automation Tool Updates

The automation tool has been updated to:

- Use `StrictHostKeyChecking=accept-new` to automatically accept new host keys
- Use `IdentitiesOnly=yes` to prevent authentication failures
- Use `PreferredAuthentications=publickey` for key-based auth only
- Provide better error messages for connection issues

## See Also

- [Nightly Automation Tool](../mcp-servers/project-management-automation/tools/nightly_task_automation.py)
- [Working Copy Health Tool](../mcp-servers/project-management-automation/tools/working_copy_health.py)
