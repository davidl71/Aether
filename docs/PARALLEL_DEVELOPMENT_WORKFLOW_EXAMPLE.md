# Parallel Development Workflow Example

**Date:** 2025-01-20
**Purpose:** Practical example of parallel development session using Ubuntu + macOS M4 Cursor agents
**Status:** ✅ **Active Example**

---

## Scenario: Implementing NATS Integration + macOS UI Improvements

**Duration:** 3-4 hours
**Agents:** Ubuntu Agent (Linux) + macOS M4 Agent (Apple Intelligence enabled)
**Goal:** Parallel implementation with coordinated integration

---

## Pre-Session Setup

### 1. Environment Check

**Ubuntu Agent:**
```bash
# Connect to Ubuntu agent
ssh cursor-ubuntu

# Verify system info
uname -a
lscpu | grep "Model name"
free -h
df -h

# Check development tools
git --version
cmake --version
rustc --version
```

**macOS M4 Agent:**
```bash
# Connect to macOS agent
ssh cursor-m4-mac

# Verify system info
sw_vers
sysctl machdep.cpu.brand_string
sysctl hw.memsize
df -h

# Verify Apple Intelligence
# System Settings → General → Apple Intelligence (should be enabled)

# Check development tools
git --version
cmake --version
rustc --version
```

### 2. Repository Setup

**Both Agents:**
```bash
# Pull latest from main
git checkout main
git pull origin main

# Create feature branches
# Ubuntu: NATS integration
git checkout -b feature/nats-integration

# macOS: UI improvements (in separate terminal/SSH session)
git checkout -b feature/macos-ui-improvements
```

### 3. Task Coordination

**Update Shared TODO Table** (`agents/shared/TODO_OVERVIEW.md`):

```markdown
## Current Session Tasks

| Task | Agent | Status | Notes |
|------|-------|--------|-------|
| T-173: NATS Integration | Ubuntu | in_progress | Starting implementation |
| T-191: macOS UI Improvements | macOS M4 | in_progress | Using Apple Intelligence for docs |
```

---

## Hour 1: Initial Implementation

### Ubuntu Agent Tasks

**Focus:** NATS adapter implementation

```bash
# Create NATS adapter module
mkdir -p agents/backend/src/nats
cd agents/backend

# Implement basic NATS adapter
# File: src/nats/adapter.rs
cargo new --lib nats_adapter
cd nats_adapter
```

**Implementation:**
- Connect to NATS server
- Implement publish/subscribe functionality
- Add error handling
- Write unit tests

**Progress Update:**
```bash
# Commit progress
git add .
git commit -m "Add NATS adapter basic structure"

# Update TODO table
# T-173: Status → in_progress, Notes → "Basic adapter structure complete"
```

### macOS M4 Agent Tasks

**Focus:** AppKit UI improvements with Apple Intelligence

```bash
# Create macOS UI improvements branch
cd native/app
# Work on AppKit bundle improvements
```

**Implementation:**
- Improve menu bar integration
- Add keyboard shortcuts
- Enhance native macOS look and feel

**Apple Intelligence Usage:**
1. **Improve Code Comments:**
   - Select existing comments
   - Right-click → "Rewrite" → "Professional"
   - AI improves clarity and documentation

2. **Generate README Updates:**
   - Write draft README for UI improvements
   - Select text → "Proofread"
   - AI fixes grammar and improves clarity

3. **Commit Message Generation:**
   ```bash
   # Draft commit message
   git add .
   git commit -m "improve ui"

   # Use Apple Intelligence to improve:
   # Select "improve ui" → Right-click → "Rewrite" → "Concise"
   # Result: "Enhance macOS AppKit UI with improved menu bar and keyboard shortcuts"
   ```

**Progress Update:**
```bash
# Commit with improved message
git commit -m "Enhance macOS AppKit UI with improved menu bar and keyboard shortcuts"

# Update TODO table
# T-191: Status → in_progress, Notes → "Menu bar and shortcuts implemented, using AI for docs"
```

---

## Hour 2: Advanced Features + Documentation

### Ubuntu Agent Tasks

**Focus:** NATS topic registry and validation

```bash
# Implement topic registry
# File: src/nats/topics.rs
```

**Implementation:**
- Create centralized topic definitions
- Add topic validation
- Implement wildcard subscription helpers
- Write comprehensive tests

**Test Execution:**
```bash
# Run tests
cargo test nats_adapter

# Check coverage
cargo test --coverage
```

**Progress Update:**
```bash
git add .
git commit -m "Add NATS topic registry and validation layer"

# Push for review
git push origin feature/nats-integration
```

### macOS M4 Agent Tasks

**Focus:** Native macOS menu integration + Documentation

**Implementation:**
- Integrate native macOS menu items
- Add system menu bar integration
- Implement native dialogs

**Apple Intelligence Usage:**

1. **Generate Architecture Diagram:**
   ```bash
   # Open Shortcuts app
   # Create shortcut: "Generate Architecture Diagram"
   # Description: "macOS AppKit UI architecture showing menu bar, dialogs, and system integration"
   # Run shortcut → Export image → Add to docs/
   ```

2. **Summarize Research:**
   - Copy AppKit documentation from Apple Developer site
   - Paste into Notes app
   - Select all → "Summarize"
   - AI generates key concepts summary
   - Use summary for implementation reference

3. **Improve Error Messages:**
   ```swift
   // Original error handling
   print("Error: Failed to show dialog")

   // Select → "Rewrite" → "Friendly"
   // Result: "Unable to display dialog. Please check system permissions and try again."
   ```

4. **Documentation Generation:**
   - Write implementation notes
   - Use Writing Tools to improve clarity
   - Generate code examples from descriptions

**Progress Update:**
```bash
# Add generated diagram to docs
git add docs/architecture/macos_ui_architecture.png

# Commit with AI-improved message
git commit -m "Integrate native macOS menu with system menu bar and native dialogs

- Add menu bar integration
- Implement native dialog system
- Add keyboard shortcuts
- Include architecture diagram (Image Playground)"

git push origin feature/macos-ui-improvements
```

---

## Hour 3: Integration & Testing

### Ubuntu Agent Tasks

**Focus:** NATS health check endpoint

**Implementation:**
- Add health check endpoint to backend
- Integrate NATS status monitoring
- Add metrics collection

**Testing:**
```bash
# Run integration tests
cargo test --test integration

# Test NATS connection
curl http://localhost:8080/health/nats
```

**Documentation:**
```bash
# Update API contract
# File: agents/shared/API_CONTRACT.md

# Add NATS health endpoint documentation
```

**Progress Update:**
```bash
git add .
git commit -m "Add NATS health check endpoint to backend API

- Health check endpoint at /health/nats
- NATS connection status monitoring
- Metrics collection integration"

# Update TODO table
# T-173: Status → review, Notes → "Ready for integration testing"
```

### macOS M4 Agent Tasks

**Focus:** Universal binary build + Documentation

**Implementation:**
- Configure universal binary build
- Test on both Intel and Apple Silicon
- Fix platform-specific issues

**Apple Intelligence Usage:**

1. **Summarize Ubuntu Agent's Work:**
   - Copy NATS implementation summary from Ubuntu agent's PR
   - Paste into Notes
   - Select → "Summarize"
   - AI generates concise summary for documentation

2. **Generate Integration Guide:**
   - Write integration steps
   - Use Writing Tools to improve clarity
   - Generate code examples

3. **Error Analysis:**
   ```bash
   # Build error occurs
   # Copy error message
   # Select → "Rewrite" → "Friendly"
   # AI explains error in plain language:
   # "The build failed because the linker cannot find the AppKit framework.
   # Solution: Add '-framework AppKit' to linker flags in CMakeLists.txt"
   ```

**Testing:**
```bash
# Build universal binary
cmake --preset macos-universal-debug
cmake --build build

# Test on both architectures
arch -x86_64 ./build/ib_box_spread_app.app/Contents/MacOS/ib_box_spread_app
arch -arm64 ./build/ib_box_spread_app.app/Contents/MacOS/ib_box_spread_app
```

**Progress Update:**
```bash
git add .
git commit -m "Complete macOS UI improvements with universal binary support

- Universal binary configuration
- Cross-architecture testing
- Integration documentation
- Architecture diagram included"

# Update TODO table
# T-191: Status → review, Notes → "Ready for integration with NATS backend"
```

---

## Integration Phase

### Both Agents Coordinate

**1. Review Each Other's Work:**
```bash
# Ubuntu agent reviews macOS PR
# macOS agent reviews Ubuntu PR

# Both check for integration compatibility
```

**2. Update API Contracts:**
```bash
# Both update agents/shared/API_CONTRACT.md
# Document any breaking changes
# Ensure compatibility
```

**3. Integration Testing:**
```bash
# macOS agent: Test UI with NATS backend
# Ubuntu agent: Verify NATS endpoint accessible from macOS UI

# Both: Run integration tests
```

**4. Merge Strategy:**
```bash
# Merge NATS integration first (dependency)
git checkout main
git merge feature/nats-integration

# Then merge macOS UI improvements
git merge feature/macos-ui-improvements

# Run full integration tests
./scripts/run_integration_tests.sh
```

---

## Post-Session Documentation

### Ubuntu Agent

**Documentation Updates:**
- NATS integration guide
- API endpoint documentation
- Testing procedures

**Files Created/Modified:**
- `agents/backend/src/nats/adapter.rs`
- `agents/backend/src/nats/topics.rs`
- `agents/shared/API_CONTRACT.md`
- `docs/NATS_INTEGRATION.md`

### macOS M4 Agent

**Documentation Updates (with Apple Intelligence):**

1. **UI Architecture Document:**
   - Generated diagram (Image Playground)
   - Improved text (Writing Tools)
   - Code examples

2. **Integration Guide:**
   - Summarized Ubuntu agent's work
   - Improved clarity with Writing Tools
   - Generated examples

3. **README Updates:**
   - Improved commit messages
   - Better code comments
   - Professional documentation

**Files Created/Modified:**
- `native/app/AppKitBundle.mm`
- `docs/architecture/macos_ui_architecture.png` (AI-generated)
- `docs/macos_ui_integration.md`
- `README.md` (improved sections)

---

## Apple Intelligence Contributions

### Time Saved

| Task | Without AI | With Apple Intelligence | Time Saved |
|------|------------|------------------------|------------|
| Improve 15 code comments | 15 min | 4 min | 11 min |
| Generate commit messages (5 commits) | 15 min | 3 min | 12 min |
| Create architecture diagram | 45 min | 5 min | 40 min |
| Summarize NATS documentation | 20 min | 2 min | 18 min |
| Improve error messages (3 errors) | 10 min | 1 min | 9 min |
| Documentation improvement | 30 min | 8 min | 22 min |

**Total Time Saved: ~112 minutes (~1.9 hours)**

### Quality Improvements

- ✅ **Better Documentation:** More professional, clearer explanations
- ✅ **Visual Content:** Architecture diagrams generated quickly
- ✅ **Error Understanding:** Plain-language explanations speed debugging
- ✅ **Consistency:** Uniform documentation style
- ✅ **Commit Messages:** More descriptive, follow conventions

---

## Key Learnings

### Ubuntu Agent

- NATS integration requires careful error handling
- Topic validation prevents runtime issues
- Integration testing essential before merge

### macOS M4 Agent

- Apple Intelligence significantly speeds documentation
- Image Playground useful for architecture diagrams
- Writing Tools improve code comment quality
- Summarization helps understand other agent's work

### Coordination

- Shared TODO table essential for tracking
- API contract updates prevent integration issues
- Regular communication via commit messages
- Both agents benefit from each other's documentation

---

## Next Steps

1. **Complete Integration:**
   - Run full test suite
   - Address any integration issues
   - Update documentation with findings

2. **Optimize Workflow:**
   - Identify bottlenecks
   - Improve coordination mechanisms
   - Refine Apple Intelligence usage

3. **Document Patterns:**
   - Add successful patterns to workflow guide
   - Update best practices
   - Share learnings with team

---

## References

- [Parallel Cursor Agents Workflow](./PARALLEL_CURSOR_AGENTS_WORKFLOW.md) - Complete parallel development guide
- [Apple Intelligence Quick Reference](./APPLE_INTELLIGENCE_QUICK_REFERENCE.md) - AI features guide
- [Development Environment](./DEVELOPMENT_ENVIRONMENT.md) - System specifications
- [Coordination Guidelines](../agents/shared/COORDINATION.md) - Multi-agent coordination

---

**This example demonstrates a realistic parallel development session using both agents effectively with Apple Intelligence enhancing productivity on the macOS M4 agent.**
