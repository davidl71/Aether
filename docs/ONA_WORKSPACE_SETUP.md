# Ona Workspace Setup Guide

This guide explains how to use your project with **Ona** (formerly Gitpod) cloud development environments and the Gitpod Flex extension in Cursor/VS Code.

## What is Ona?

[Ona](https://ona.com) provides cloud-based development environments that:

- **Pre-configured environments**: All dependencies installed automatically
- **Ona Agents**: AI-powered software engineering agents
- **MCP Integration**: Uses `.ona/mcp-config.json` for extended AI capabilities
- **Port forwarding**: Automatic port exposure for web services and APIs
- **Prebuilds**: Faster workspace startup with pre-configured environments

## Prerequisites

1. **Ona Account**: Sign up at [app.gitpod.io](https://app.gitpod.io) (free tier available)
2. **Gitpod Flex Extension**: Install `gitpod.gitpod-flex` in Cursor/VS Code
3. **Remote - SSH Extension**: Required for VS Code integration (auto-installed)

## Quick Start

### 1. Install Extension

The Gitpod Flex extension is already recommended in `.vscode/extensions.json`. If not installed:

1. Open Cursor/VS Code
2. Go to Extensions (Cmd+Shift+X / Ctrl+Shift+X)
3. Search for "Gitpod Flex"
4. Install `gitpod.gitpod-flex`

### 2. Authenticate with Ona

1. Open Command Palette (Cmd+Shift+P / Ctrl+Shift+P)
2. Type "Gitpod: Sign In"
3. Follow authentication flow in browser
4. Return to Cursor/VS Code

### 3. Open Project in Ona

**Option A: From Cursor/VS Code**

1. Open Command Palette
2. Type "Gitpod: Open in Gitpod"
3. Select your repository
4. Workspace opens in browser

**Option B: From GitHub/GitLab**

1. Navigate to your repository
2. Click the "Ona" button (added by browser extension)
3. Workspace opens automatically

**Option C: Direct URL**

```
https://gitpod.io/#https://github.com/YOUR_USERNAME/YOUR_REPO
```

## Workspace Configuration

### `.gitpod.yml`

The project includes a comprehensive `.gitpod.yml` configuration:

**Features**:

- **Multi-language support**: C++, Python, Rust, TypeScript
- **Automatic setup**: Installs all dependencies on workspace start
- **Port forwarding**: Exposes backend APIs, web frontend, NATS server
- **VS Code extensions**: Pre-installs recommended extensions
- **Prebuilds**: Faster startup with pre-configured environments

**Configured Ports**:

- `8080` - Backend REST API
- `50051` - Backend gRPC API
- `5173` - Web Frontend (Vite dev server)
- `4222` - NATS Server

**Environment Variables**:

- `TWS_MOCK=true` - Uses mock TWS client (no live trading in cloud)
- `CMAKE_BUILD_TYPE=Debug` - Debug build configuration
- `PYTHONPATH` - Python module path

### `.gitpod.Dockerfile`

Custom Docker image with:

- Build tools (CMake, Ninja, GCC/Clang)
- Python 3.11 with pip
- Node.js and npm
- Rust toolchain
- System libraries (OpenSSL, Protobuf)

## Using Ona Workspace

### Initial Setup

When workspace first starts:

1. **Automatic setup runs**: Installs dependencies, configures environment
2. **Extensions install**: VS Code extensions auto-install
3. **Ready to code**: Workspace is ready in ~2-3 minutes

### Building the Project

```bash

# Configure CMake

cmake --preset linux-debug

# Build project

cmake --build build

# Run tests

ctest --test-dir build --output-on-failure
```

### Running Services

**Backend Service**:

```bash
cd agents/backend
cargo run

# REST API available at http://localhost:8080
# gRPC API available at localhost:50051
```

**Web Frontend**:

```bash
cd web
npm run dev

# Frontend available at http://localhost:5173
```

**NATS Server**:

```bash
nats-server -c config/nats-server.conf

# NATS available at nats://localhost:4222
```

### Port Forwarding

Ona automatically forwards configured ports:

- **Public URLs**: Each port gets a public URL (e.g., `https://8080-xxx.gitpod.io`)
- **Private access**: Ports also accessible via `localhost:PORT`
- **Auto-open**: Web frontend (5173) opens in preview automatically

## Ona Agents Integration

### MCP Configuration

Ona Agents use `.ona/mcp-config.json` for extended capabilities:

- **Semgrep**: Security scanning
- **Filesystem**: File operations
- **Git**: Version control
- **Context7**: Documentation access
- **NotebookLM**: Research and knowledge base
- **Tractatus Thinking**: Logical analysis
- **Sequential Thinking**: Implementation workflows

See [ONA_MCP_SETUP.md](ONA_MCP_SETUP.md) for detailed MCP configuration.

### Using Ona Agents

1. **Open Ona Agent interface** in workspace
2. **MCP servers auto-connect** from `.ona/mcp-config.json`
3. **Use AI agents** with extended MCP capabilities
4. **Agents can**:
   - Read/write files
   - Execute commands (safely)
   - Access git history
   - Scan for security issues
   - Research documentation
   - Analyze code structure

## VS Code Integration

### Gitpod Flex Extension Features

**Command Palette Commands**:

- `Gitpod: Open in Gitpod` - Open current workspace in Ona
- `Gitpod: Sign In` - Authenticate with Ona
- `Gitpod: Sign Out` - Sign out from Ona
- `Gitpod: Open Workspace` - Open existing workspace

**Status Bar**:

- Shows Ona workspace status
- Quick access to workspace actions

**Remote Development**:

- Full VS Code/Cursor experience in cloud
- All extensions work in cloud environment
- Terminal, debugger, IntelliSense all functional

## Troubleshooting

### Workspace Won't Start

1. **Check Ona account**: Ensure you're signed in
2. **Check repository access**: Repository must be accessible to Ona
3. **Check `.gitpod.yml`**: Verify configuration is valid YAML
4. **Check logs**: View workspace logs in Ona dashboard

### Build Failures

1. **Third-party dependencies**: May need manual setup

   ```bash
   ./scripts/fetch_third_party.sh
   ```

2. **CMake configuration**: Verify preset exists

   ```bash
   cmake --list-presets
   ```

3. **Missing tools**: Check if all tools installed

   ```bash
   which cmake ninja python3 rustc node npm
   ```

### Port Forwarding Issues

1. **Check port configuration**: Verify ports in `.gitpod.yml`
2. **Check service status**: Ensure service is running
3. **Check firewall**: Ona handles firewall automatically
4. **Use public URL**: Try public URL instead of localhost

### Extension Issues

1. **Reinstall extensions**: Extensions auto-install but may need refresh
2. **Check compatibility**: Some extensions may not work in cloud
3. **Reload window**: Cmd+Shift+P → "Developer: Reload Window"

## Security Considerations

### Trading Software in Cloud

⚠️ **IMPORTANT**: This is trading software. Use extra caution in cloud environments:

1. **Mock Mode**: Workspace defaults to `TWS_MOCK=true` (no live trading)
2. **No Credentials**: Never commit API keys or credentials
3. **Use Ona Secrets**: Store sensitive data in Ona Secrets, not in code
4. **Review Ports**: Only expose necessary ports
5. **Access Control**: Use Ona organization controls for team workspaces

### Best Practices

- **Always use paper trading** in cloud environments
- **Never commit secrets** to repository
- **Use environment variables** for configuration
- **Review MCP tool permissions** in `.ona/mcp-config.json`
- **Enable organization controls** for enterprise deployments

## Cost Management

### Free Tier

Ona free tier includes:

- Limited workspace hours per month
- Basic workspace resources
- Public repository access

### Usage Tips

1. **Stop workspaces** when not in use
2. **Use prebuilds** to reduce startup time
3. **Share workspaces** with team members
4. **Monitor usage** in Ona dashboard

## Additional Resources

- [Ona Documentation](https://ona.com/docs)
- [Ona Getting Started](https://ona.com/docs/ona/getting-started)
- [Ona MCP Setup](ONA_MCP_SETUP.md)
- [Gitpod Flex Extension](https://www.gitpod.io/docs/flex/editors/vscode)
- [Project MCP Servers](research/integration/MCP_SERVERS.md)

## Next Steps

1. ✅ Install Gitpod Flex extension
2. ✅ Authenticate with Ona
3. ✅ Open project in Ona workspace
4. ✅ Verify MCP servers connect (check `.ona/mcp-config.json`)
5. ✅ Start developing with Ona Agents!

---

**Note**: This configuration is optimized for the IBKR Box Spread Generator project. Adjust workspace settings based on your specific needs and resource requirements.
