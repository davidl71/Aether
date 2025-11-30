# Parallel Development with Two Remote Cursor Agents (Ubuntu + macOS)

**Date:** 2025-01-20
**Purpose:** Enable parallel development using two remote Cursor agents (Ubuntu and macOS)
**Status:** ✅ **FEASIBLE - Cursor 2.0 Multi-Agent Support**

---

## Overview

**Yes, you can use two remote Cursor agents (Ubuntu + macOS) for parallel development!** Cursor 2.0 introduces multi-agent capabilities that support:

- **Up to 8 agents** working concurrently on a single prompt
- **Background agents** that work independently on remote machines
- **Git worktrees** or **remote machines** for isolation
- **Coordinated workflows** with shared TODO tracking

---

## Cursor 2.0 Multi-Agent Features

### Background Agents

Cursor 2.0 supports **background agents** that can work on remote machines:

1. **Spawn Asynchronous Agents:**
   - Agents edit and run code in remote environments
   - View status, send follow-ups, or take over at any time
   - Access via Background Agent Sidebar

2. **Isolation Strategies:**
   - **Git Worktrees:** Each agent works in isolated git worktree
   - **Remote Machines:** Each agent connected to different remote machine
   - **Separate Branches:** Different branches for parallel work

3. **Multi-Agent Interface:**
   - Up to 8 agents can work concurrently on a single prompt
   - Each agent operates in isolated copy of codebase
   - Prevents file conflicts automatically

---

## Apple Intelligence Advantage: M4 Remote Agent

**🚀 Your macOS remote agent runs on an M4 processor with Apple Intelligence - this is a significant advantage!**

### Apple Intelligence Capabilities on M4

The M4 chip features:

- **Neural Engine**: 38 trillion operations per second optimized for AI tasks
- **Up to 1.7x faster** than M1 in development workloads
- **System-wide AI**: Available in all macOS applications
- **On-device processing**: Privacy-focused, no data sent to cloud
- **Writing Tools**: AI-powered text improvement, summarization, and generation

### How Apple Intelligence Enhances Parallel Development

**macOS M4 Agent Advantages:**

1. **AI-Assisted Documentation:**
   - Generate code comments and documentation
   - Improve commit messages automatically
   - Summarize long API documentation
   - Create architecture diagrams (Image Playground)

2. **Code Quality Improvements:**
   - Rewrite unclear code comments
   - Explain complex compiler errors
   - Improve error messages
   - Generate test case descriptions

3. **Documentation Enhancement:**
   - Auto-summarize research papers
   - Improve technical writing clarity
   - Generate code examples from descriptions
   - Create visual diagrams for complex concepts

4. **Workflow Optimization:**
   - Smart summaries of build logs
   - Error analysis in plain language
   - Automated documentation from code changes
   - Context-aware code suggestions

### Apple Intelligence + Cursor AI Synergy

**Complementary AI Systems:**

| Feature | Cursor AI | Apple Intelligence |
|---------|-----------|-------------------|
| **Code Generation** | ✅ Full IDE integration | ✅ System-wide writing tools |
| **Documentation** | ✅ Markdown generation | ✅ Text improvement, summaries |
| **Error Analysis** | ✅ Code-aware debugging | ✅ Plain language explanations |
| **Visual Generation** | ❌ Text-based | ✅ Image Playground diagrams |
| **Privacy** | Cloud-based | ✅ On-device processing |
| **Availability** | In Cursor only | ✅ All macOS apps |

**Best Practice:** Use Cursor AI for code generation, Apple Intelligence for documentation, summaries, and visual content.

---

## Setup: Two Remote Agents (Ubuntu + macOS)

### Prerequisites

1. **Both Remote Machines:**
   - ✅ SSH enabled and accessible
   - ✅ Git repository cloned
   - ✅ Development environment configured (C++, CMake, dependencies)
   - ✅ Network connectivity to your local machine
   - ✅ **macOS M4 Agent:** Apple Intelligence enabled (System Settings → General → Apple Intelligence)

2. **Local Machine:**
   - ✅ Cursor IDE installed
   - ✅ SSH clients configured for both machines
   - ✅ Access to 1Password (optional, for credential management)

---

## Configuration Steps

### Step 1: Configure SSH Access for Both Machines

#### Option A: Use Existing 1Password Integration (Recommended)

**For Ubuntu Agent:**

```bash
export OP_CURSOR_REMOTE_HOST_SECRET="op://Engineering/Cursor Remote Ubuntu/host"
export OP_CURSOR_REMOTE_USER_SECRET="op://Engineering/Cursor Remote Ubuntu/username"
export OP_CURSOR_REMOTE_KEY_SECRET="op://Engineering/Cursor Remote Ubuntu/private key"
export OP_CURSOR_REMOTE_PORT_SECRET="op://Engineering/Cursor Remote Ubuntu/port"
export CURSOR_REMOTE_ALIAS="cursor-ubuntu"

./scripts/op_sync_cursor_remote.sh
```

**For macOS Agent:**

```bash
export OP_CURSOR_REMOTE_HOST_SECRET="op://Engineering/Cursor Remote M4/host"
export OP_CURSOR_REMOTE_USER_SECRET="op://Engineering/Cursor Remote M4/username"
export OP_CURSOR_REMOTE_KEY_SECRET="op://Engineering/Cursor Remote M4/private key"
export OP_CURSOR_REMOTE_PORT_SECRET="op://Engineering/Cursor Remote M4/port"
export CURSOR_REMOTE_ALIAS="cursor-m4-mac"

./scripts/op_sync_cursor_remote.sh
```

#### Option B: Manual SSH Configuration

Add to `~/.ssh/config`:

```ssh-config

# Ubuntu Remote Agent

Host cursor-ubuntu
  HostName <ubuntu_ip_or_hostname>
  User <your_username>
  IdentityFile ~/.ssh/cursor_ubuntu_id_ed25519
  StrictHostKeyChecking accept-new
  IdentitiesOnly yes
  Compression yes
  ServerAliveInterval 30
  ServerAliveCountMax 10

# macOS Remote Agent

Host cursor-m4-mac
  HostName <mac_ip_or_hostname>
  User <your_username>
  IdentityFile ~/.ssh/cursor_m4_id_ed25519
  StrictHostKeyChecking accept-new
  IdentitiesOnly yes
  Compression yes
  ServerAliveInterval 30
  ServerAliveCountMax 10
```

### Step 2: Connect to Remote Machines

#### Connect to Ubuntu Agent

1. Open Command Palette (`⌘+Shift+P` / `Ctrl+Shift+P`)
2. Select "Remote-SSH: Connect to Host"
3. Choose `cursor-ubuntu`
4. When prompted, select "Linux" as the remote OS
5. Accept the host fingerprint if prompted

#### Connect to macOS Agent

1. Open Command Palette (`⌘+Shift+P` / `Ctrl+Shift+P`)
2. Select "Remote-SSH: Connect to Host"
3. Choose `cursor-m4-mac`
4. When prompted, select "macOS" as the remote OS
5. Accept the host fingerprint if prompted

**First Connection:** Cursor will automatically download and install the VS Code Server on each remote machine.

### Step 3: Set Up Git Worktrees (Isolation Strategy)

On each remote machine, create isolated worktrees:

**On Ubuntu:**

```bash
cd /path/to/repo
git worktree add ../ib_box_spread_ubuntu-agent -b feature/ubuntu-agent
cd ../ib_box_spread_ubuntu-agent
```

**On macOS:**

```bash
cd /path/to/repo
git worktree add ../ib_box_spread_macos-agent -b feature/macos-agent
cd ../ib_box_spread_macos-agent
```

### Step 4: Configure Background Agents

#### Access Background Agent Sidebar

1. Open Cursor on your local machine
2. Access Background Agent Sidebar (or press `Ctrl+E`)
3. View all background agents associated with your account

#### Spawn Background Agents

1. **Spawn Ubuntu Agent:**
   - Trigger background agent mode (`Ctrl+E`)
   - Submit prompt for Ubuntu-specific work
   - Select "Use Remote Machine: cursor-ubuntu"
   - Agent will work in isolated worktree

2. **Spawn macOS Agent:**
   - Trigger background agent mode (`Ctrl+E`)
   - Submit prompt for macOS-specific work
   - Select "Use Remote Machine: cursor-m4-mac"
   - Agent will work in isolated worktree

---

## Parallel Development Workflows

### Workflow 1: Platform-Specific Development

**Strategy:** Each agent handles platform-specific work

**Ubuntu Agent Tasks:**

- Linux-specific builds and testing
- Docker container development
- CI/CD pipeline testing
- Cross-platform compatibility testing
- Backend services (Rust, Go)

**macOS Agent Tasks (with Apple Intelligence):**

- macOS-specific builds (Universal binaries)
- AppKit bundle development
- macOS system integration
- Xcode toolchain testing
- **AI-Assisted Documentation:** Generate/improve code comments, README updates
- **Error Analysis:** Plain-language explanations of complex build errors
- **Visual Content:** Generate architecture diagrams (Image Playground)
- **Documentation Quality:** Improve technical writing, summarize research

**Coordination:**

- Use shared TODO table (`agents/shared/TODO_OVERVIEW.md`)
- Update API contracts (`agents/shared/API_CONTRACT.md`)
- Document platform-specific changes
- **macOS Agent:** Generate documentation summaries for Ubuntu agent's work

### Workflow 2: Feature Branch Development

**Strategy:** Each agent works on different feature branches

**Setup:**

```bash

# Ubuntu agent works on feature/backend-api
# macOS agent works on feature/frontend-ui

# Both pull from main regularly

git checkout main
git pull origin main

# Create feature branches

git checkout -b feature/backend-api
git checkout -b feature/frontend-ui
```

**Coordination:**

- Regular sync: Pull main → Rebase feature branch
- Communication: Document breaking changes in `API_CONTRACT.md`
- Integration: Merge both features → Test together

### Workflow 3: Component Parallelization

**Strategy:** Split work by component/system

**Example Division:**

- **Ubuntu Agent:** Backend services, NATS integration, Rust components
- **macOS Agent:** Frontend UI, native macOS app, Swift integration

**Coordination:**

- Shared API contracts
- Update TODO table when starting/completing work
- Document component dependencies

---

## Coordination Mechanisms

### 1. Shared TODO Table

**Location:** `agents/shared/TODO_OVERVIEW.md`

**Usage:**

- Update status: `pending` → `in_progress` → `completed`
- Add notes for blockers or dependencies
- Single source of truth for overall status

**Example:**

```markdown
| Task | Agent | Status | Notes |
|------|-------|--------|-------|
| T-173: NATS Integration | Ubuntu | in_progress | Testing on Linux |
| T-191: AppKit Bundle | macOS | in_progress | Universal binary build |
```

### 2. API Contract Documentation

**Location:** `agents/shared/API_CONTRACT.md`

**Usage:**

- Document API changes immediately
- Keep frontend/backend compatible
- Link to PRs/commits for context

### 3. Git Workflow Coordination

**Branch Strategy:**

```bash

# Both agents sync regularly

git checkout main
git pull origin main

# Work in feature branches

git checkout -b feature/agent-name/task-description

# Regular rebasing

git checkout feature/agent-name/task-description
git rebase main

# Push when ready for review

git push origin feature/agent-name/task-description
```

**Conflict Prevention:**

- Different branches per agent
- Different components per agent
- Different files per agent
- Regular communication via TODO table

---

## Best Practices

### 1. Isolation

✅ **DO:**

- Use separate git worktrees or branches
- Work on different components/systems
- Update shared documentation (TODO, API contracts)
- Sync regularly with main branch

❌ **DON'T:**

- Edit the same files simultaneously
- Work on overlapping functionality without coordination
- Forget to update shared TODO table
- Merge without coordination

### 4. Leverage Apple Intelligence (macOS M4 Agent)

✅ **DO:**

- Use Writing Tools to improve all documentation
- Generate commit messages with AI assistance
- Create visual diagrams (Image Playground) for complex concepts
- Summarize research papers and long documentation
- Improve code comments for better readability
- Use AI to explain complex errors in plain language

❌ **DON'T:**

- Rely solely on Apple Intelligence for code generation (use Cursor AI)
- Skip human review of AI-generated content
- Use AI for security-sensitive or proprietary algorithm descriptions

**Apple Intelligence Workflow Examples:**

```bash

# Example 1: Improve commit message
# Before: "fixed bug"
# After (AI-assisted): "Fix box spread calculation error when strike width is zero"

# Example 2: Generate documentation
# Select code → Right-click → "Summarize"
# AI generates: "This function calculates the net premium for a box spread..."

# Example 3: Create architecture diagram
# Image Playground: "Box spread trading system architecture showing NATS, TWS API, and Rust backend"
```

### 2. Communication

✅ **DO:**

- Update `agents/shared/TODO_OVERVIEW.md` regularly
- Document breaking changes in `API_CONTRACT.md`
- Add notes about blockers or dependencies
- Communicate major architectural decisions

❌ **DON'T:**

- Work in isolation without updates
- Make breaking changes without documenting
- Skip TODO table updates
- Assume other agent knows your changes

### 3. Testing

✅ **DO:**

- Test on both platforms when possible
- Run cross-platform compatibility tests
- Test integration points regularly
- Document platform-specific behavior

❌ **DON'T:**

- Assume Linux behavior = macOS behavior
- Skip testing after major changes
- Ignore platform-specific edge cases
- Merge without integration testing

---

## Example: Parallel Development Session

### Scenario: Implementing NATS Integration + macOS UI (with Apple Intelligence)

**Setup:**

1. Ubuntu agent: Feature branch `feature/nats-integration`
2. macOS agent (M4 with Apple Intelligence): Feature branch `feature/macos-ui-improvements`

**Session Flow:**

**Hour 1:**

- Ubuntu agent: NATS adapter implementation (Rust backend)
- macOS agent: AppKit UI improvements
- **macOS agent (Apple Intelligence):** Use Writing Tools to improve code comments, generate README updates
- Both: Update TODO table, pull latest main

**Hour 2:**

- Ubuntu agent: NATS topic registry and validation
- macOS agent: Native macOS menu integration
- **macOS agent (Apple Intelligence):** Generate architecture diagram (Image Playground) showing UI components
- Both: Rebase on main, check for conflicts

**Hour 3:**

- Ubuntu agent: NATS health check endpoint
- macOS agent: Universal binary build testing
- **macOS agent (Apple Intelligence):**
  - Summarize Ubuntu agent's NATS implementation for documentation
  - Improve error messages in build logs with plain-language explanations
  - Generate commit message: "Add macOS universal binary support with improved error handling"

- Both: Update API contracts, document changes

**Integration:**

- Ubuntu agent: Create PR for NATS integration
- macOS agent: Create PR for macOS UI (with AI-improved documentation and diagrams)
- **macOS agent (Apple Intelligence):** Generate PR description summary
- Both: Review each other's PRs, test integration

**Merge:**

- Merge NATS integration first (dependency)
- Merge macOS UI improvements (includes AI-enhanced documentation)
- Run integration tests on both platforms

**Apple Intelligence Contributions:**

- ✅ Improved documentation quality (code comments, README)
- ✅ Visual architecture diagrams (Image Playground)
- ✅ Better commit messages
- ✅ Plain-language error explanations
- ✅ Documentation summaries

---

## Troubleshooting

### Issue: File Conflicts

**Symptoms:** Git conflicts when merging

**Solution:**

- Use separate branches or worktrees
- Work on different components
- Sync regularly with main
- Communicate overlapping work

### Issue: Background Agent Not Responding

**Symptoms:** Agent doesn't update or respond

**Solution:**

- Check SSH connection to remote machine
- Verify remote machine is accessible
- Check Background Agent Sidebar for status
- Try reconnecting to remote machine

### Issue: Code Drift Between Agents

**Symptoms:** Agents have different code versions

**Solution:**

- Regular `git pull origin main` on both
- Rebase feature branches regularly
- Use shared TODO table to track progress
- Communicate major changes immediately

### Issue: Remote Machine Disconnection

**Symptoms:** Can't connect to remote agent

**Solution:**

- Check SSH connectivity: `ssh cursor-ubuntu` or `ssh cursor-m4-mac`
- Verify remote machine is online
- Check SSH config settings
- Restart SSH service on remote machine if needed

---

## Integration with Existing Project Patterns

### Leverage Existing Coordination

The project already has coordination patterns in `agents/shared/COORDINATION.md`:

1. **Update TODO Table** - Single source of truth
2. **API Contract Updates** - Keep interfaces compatible
3. **CI Monitoring** - Track build status

### Use Existing Remote Development Scripts

Leverage existing scripts:

- `scripts/op_sync_cursor_remote.sh` - SSH configuration
- `scripts/op_sync_distcc_host.sh` - Remote compilation setup
- `scripts/setup_worktree.sh` - Git worktree management

### Follow Existing Workflows

- Branch strategy: Feature branches per agent
- Commit messages: Imperative mood, 72-character subject
- Testing: Run tests before merging
- Documentation: Update relevant docs

---

## Advanced: Distributed Compilation

Both remote agents can also serve as distributed compilation workers:

**Ubuntu Agent:**

```bash

# Run distcc daemon for distributed C++ builds

distccd --daemon --allow 192.168.1.0/24 --jobs $(nproc)
```

**macOS Agent:**

```bash

# Run distcc daemon for distributed C++ builds

distccd --daemon --allow 192.168.1.0/24 --jobs $(sysctl -n hw.ncpu)
```

**Client Configuration:**

```bash
export DISTCC_HOSTS="localhost/8 \
  ubuntu-agent.local/8 \
  macos-agent.local/8"

cmake -S . -B build -DCMAKE_CXX_COMPILER_LAUNCHER=distcc
make -j24 -C build  # Uses all machines
```

---

## Security Considerations

1. **SSH Keys:**
   - Use Ed25519 keys (stronger, faster)
   - Never commit keys to repository
   - Use 1Password for credential management
   - Rotate keys regularly

2. **Network Security:**
   - Use VPN for remote access if possible
   - Limit SSH access to specific IPs
   - Use firewall rules to restrict access
   - Monitor SSH connection logs

3. **Remote Machine Security:**
   - Keep remote machines updated
   - Use strong passwords or key-only auth
   - Disable password authentication
   - Monitor remote machine access logs

---

## Summary

✅ **Parallel Development is Feasible:**

- Cursor 2.0 supports multi-agent workflows
- Background agents work on remote machines
- Git worktrees provide isolation
- Existing coordination patterns support parallel work
- **Apple Intelligence on M4 provides AI-enhanced documentation and quality improvements**

✅ **Recommended Setup:**

- Configure SSH for both Ubuntu and macOS
- Use separate git worktrees or branches
- Leverage Background Agent Sidebar
- Coordinate via shared TODO table
- **Enable Apple Intelligence on macOS M4 agent for enhanced workflows**

✅ **Best Practices:**

- Isolate work (different branches/components)
- Communicate regularly (TODO updates, API contracts)
- Test on both platforms
- Sync frequently with main branch
- **Leverage Apple Intelligence for documentation, summaries, and visual content**

✅ **Apple Intelligence Advantage:**

- Use macOS M4 agent for documentation-heavy tasks
- Generate visual diagrams with Image Playground
- Improve all text content with Writing Tools
- Get plain-language error explanations
- Create better commit messages automatically

**Next Steps:**

1. Configure SSH access for both remote machines
2. Set up git worktrees on each remote
3. **Enable Apple Intelligence on macOS M4** (System Settings → General → Apple Intelligence)
4. Test Background Agent connections
5. **Start parallel development with AI-enhanced workflows!**

**Workflow Optimization:**

- **Ubuntu Agent:** Focus on implementation, testing, builds
- **macOS M4 Agent:** Focus on implementation + documentation quality, visual content, summaries
- Both coordinate via shared TODO table and API contracts

---

## References

- [Cursor 2.0 Changelog - Multi-Agent](https://cursor.com/changelog/2-0/)
- [Cursor Background Agents Documentation](https://docs.cursor.com/en/background-agents)
- [Remote Development Workflow](./REMOTE_DEVELOPMENT_WORKFLOW.md) - Single remote agent setup
- [Coordination Guidelines](../agents/shared/COORDINATION.md) - Multi-agent coordination
- [Development Environment](./DEVELOPMENT_ENVIRONMENT.md) - System specifications and environment documentation
- [Parallel Development Workflow Example](./PARALLEL_DEVELOPMENT_WORKFLOW_EXAMPLE.md) - Practical session example
- [Apple Intelligence Quick Reference](./APPLE_INTELLIGENCE_QUICK_REFERENCE.md) - AI features for macOS M4 agent
- [Device Task Delegation](./DEVICE_TASK_DELEGATION.md) - Multi-machine workflows
