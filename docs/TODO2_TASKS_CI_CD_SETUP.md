# TODO2 Tasks for CI/CD and Parallel Agent Setup

**Date:** 2025-01-20
**Purpose:** TODO2 tasks to track CI/CD setup and parallel agent configuration
**Status:** ✅ **Tasks Documented - Ready to Create**

---

## Required TODO2 Tasks

### Task 1: Setup GitHub Actions Runner on Ubuntu Agent

**TODO2 Task Details:**

- **Name:** Setup GitHub Actions runner on Ubuntu agent
- **Priority:** high
- **Tags:** `ci-cd`, `infrastructure`, `ubuntu`, `parallel-agents`
- **Status:** Todo
- **Dependencies:** None

**Long Description:**

```markdown
🎯 **Objective:** Install and configure GitHub Actions self-hosted runner on Ubuntu remote agent (david@192.168.192.57) for CI/CD workflow execution.

📋 **Acceptance Criteria:**

- GitHub Actions runner installed on Ubuntu agent
- Runner configured with correct labels (ubuntu, linux)
- Runner service running automatically
- Runner visible in GitHub repository (Settings → Actions → Runners)
- Runner executes test workflows successfully

🚫 **Scope Boundaries (CRITICAL):**

- **Included:** Runner installation, configuration, service setup, verification
- **Excluded:** Workflow creation (separate task), runner maintenance (ongoing)

🔧 **Technical Requirements:**

- Use setup script: scripts/setup_github_runner_ubuntu.sh
- Runner name: ubuntu-agent
- Labels: ubuntu, linux
- Work directory: ~/actions-runner/_work
- Service: systemd (auto-start)

📁 **Files/Components:**

- Run: scripts/setup_github_runner_ubuntu.sh
- Create: ~/actions-runner/ directory on Ubuntu agent
- Update: GitHub repository runners configuration

🧪 **Testing Requirements:**

- Test runner connectivity from GitHub
- Execute test workflow on Ubuntu runner
- Verify runner appears online in GitHub
- Check runner logs for errors

⚠️ **Edge Cases:**

- Network connectivity issues
- Service fails to start
- Runner token expiration
- Disk space limitations

📚 **Dependencies:** None
```

**Shared TODO Entry:**

```markdown
| CI-1 | Setup GitHub Actions runner on Ubuntu agent | ubuntu | pending |
```

---

### Task 2: Setup GitHub Actions Runner on macOS M4 Agent

**TODO2 Task Details:**

- **Name:** Setup GitHub Actions runner on macOS M4 agent
- **Priority:** high
- **Tags:** `ci-cd`, `infrastructure`, `macos`, `m4`, `parallel-agents`, `apple-intelligence`
- **Status:** Todo
- **Dependencies:** None

**Long Description:**

```markdown
🎯 **Objective:** Install and configure GitHub Actions self-hosted runner on macOS M4 remote agent (davidl@192.168.192.141) for CI/CD workflow execution, leveraging Apple Intelligence capabilities.

📋 **Acceptance Criteria:**

- GitHub Actions runner installed on macOS M4 agent
- Runner configured with correct labels (macos, apple-silicon, m4)
- Runner service running automatically (launchd)
- Runner visible in GitHub repository (Settings → Actions → Runners)
- Runner executes test workflows successfully
- Apple Intelligence verified available (M4 chip detected)

🚫 **Scope Boundaries (CRITICAL):**

- **Included:** Runner installation, configuration, service setup, verification, Apple Intelligence check
- **Excluded:** Workflow creation (separate task), Apple Intelligence usage (documentation only)

🔧 **Technical Requirements:**

- Use setup script: scripts/setup_github_runner_macos.sh
- Runner name: macos-m4-agent
- Labels: macos, apple-silicon, m4
- Work directory: ~/actions-runner/_work
- Service: launchd (auto-start)
- Verify: sysctl machdep.cpu.brand_string shows M4

📁 **Files/Components:**

- Run: scripts/setup_github_runner_macos.sh
- Create: ~/actions-runner/ directory on macOS M4 agent
- Update: GitHub repository runners configuration

🧪 **Testing Requirements:**

- Test runner connectivity from GitHub
- Execute test workflow on macOS M4 runner
- Verify runner appears online in GitHub
- Check runner logs for errors
- Verify Apple Intelligence availability

⚠️ **Edge Cases:**

- Network connectivity issues
- Service fails to start
- Runner token expiration
- Disk space limitations
- Apple Intelligence not available (should still work)

📚 **Dependencies:** None
```

**Shared TODO Entry:**

```markdown
| CI-2 | Setup GitHub Actions runner on macOS M4 agent | macos | pending |
```

---

### Task 3: Configure Enhanced CI/CD Workflow for Parallel Agents

**TODO2 Task Details:**

- **Name:** Configure enhanced CI/CD workflow for parallel agent testing
- **Priority:** high
- **Tags:** `ci-cd`, `workflow`, `parallel-agents`, `testing`
- **Status:** Todo
- **Dependencies:** CI-1, CI-2 (runners must be set up first)

**Long Description:**

```markdown
🎯 **Objective:** Implement enhanced GitHub Actions workflow that supports parallel testing across Ubuntu and macOS M4 agents with integration validation and coordination checks.

📋 **Acceptance Criteria:**

- Enhanced workflow file created (.github/workflows/parallel-agents-ci.yml)
- Parallel agent test jobs configured (Ubuntu + macOS)
- Integration test job configured
- Coordination validation job configured
- Workflow triggers on PR and push to main/develop
- All jobs execute successfully on self-hosted runners
- Validation scripts integrated into workflow

🚫 **Scope Boundaries (CRITICAL):**

- **Included:** Workflow YAML configuration, job dependencies, validation integration
- **Excluded:** Validation script implementation (already done), runner setup (separate tasks)

🔧 **Technical Requirements:**

- Use self-hosted runners with proper labels
- Matrix builds for component testing
- Integration tests after agent tests complete
- Coordination validation (TODO table, API contract, TODO2 sync)
- Artifact uploads for test results

📁 **Files/Components:**

- Create: .github/workflows/parallel-agents-ci.yml
- Update: Existing .github/workflows/ci.yml (if needed)
- Use: scripts/validate_*.sh scripts

🧪 **Testing Requirements:**

- Workflow executes on test PR
- All jobs complete successfully
- Validation scripts run correctly
- Artifacts uploaded properly
- Coordination validation works

⚠️ **Edge Cases:**

- Runner unavailable (should fall back gracefully)
- Matrix build failures (should report correctly)
- Validation script errors (should be non-blocking where appropriate)
- Workflow syntax errors

📚 **Dependencies:** CI-1 (Ubuntu runner), CI-2 (macOS M4 runner)
```

**Shared TODO Entry:**

```markdown
| CI-3 | Configure enhanced CI/CD workflow for parallel agents | shared | pending |
```

---

### Task 4: Document Agent Environment and System Specifications

**TODO2 Task Details:**

- **Name:** Document agent environment and system specifications
- **Priority:** medium
- **Tags:** `documentation`, `environment`, `parallel-agents`
- **Status:** Todo
- **Dependencies:** None

**Long Description:**

```markdown
🎯 **Objective:** Collect and document comprehensive system information from both Ubuntu and macOS M4 remote agents for environment documentation and task delegation.

📋 **Acceptance Criteria:**

- System information collected from Ubuntu agent
- System information collected from macOS M4 agent
- Development Environment document populated with specs
- Agent hostnames and paths documented
- System info collection scripts tested and working

🚫 **Scope Boundaries (CRITICAL):**

- **Included:** System info collection, documentation updates, script testing
- **Excluded:** System changes, configuration modifications

🔧 **Technical Requirements:**

- Use collection script: scripts/collect_system_info_python.py
- Document OS, CPU, RAM, disk, network info
- Document development tool versions
- Update docs/DEVELOPMENT_ENVIRONMENT.md

📁 **Files/Components:**

- Run: scripts/collect_system_info_python.py on both agents
- Update: docs/DEVELOPMENT_ENVIRONMENT.md
- Create: system_info_ubuntu.json, system_info_macos.json

🧪 **Testing Requirements:**

- Collection scripts run successfully on both agents
- JSON output is valid and complete
- Documentation reflects actual system specs
- Scripts handle errors gracefully

⚠️ **Edge Cases:**

- Script fails on one agent
- Missing system information
- Permission issues
- Network connectivity problems

📚 **Dependencies:** None
```

**Shared TODO Entry:**

```markdown
| CI-4 | Document agent environment and system specifications | shared | pending |
```

---

### Task 5: Test Parallel Agent CI/CD Workflow

**TODO2 Task Details:**

- **Name:** Test parallel agent CI/CD workflow with sample changes
- **Priority:** high
- **Tags:** `ci-cd`, `testing`, `parallel-agents`, `verification`
- **Status:** Todo
- **Dependencies:** CI-1, CI-2, CI-3 (all setup must be complete)

**Long Description:**

```markdown
🎯 **Objective:** Verify the complete parallel agent CI/CD workflow functions correctly by creating test PRs that trigger workflows on both Ubuntu and macOS M4 agents.

📋 **Acceptance Criteria:**

- Test PR created with changes to Ubuntu-related files
- Test PR created with changes to macOS-related files
- Both workflows execute on correct agents
- Integration tests pass
- Coordination validation passes
- Test results uploaded as artifacts
- PR comments show validation status

🚫 **Scope Boundaries (CRITICAL):**

- **Included:** Test PR creation, workflow execution verification, result validation
- **Excluded:** Actual feature implementation, production changes

🔧 **Technical Requirements:**

- Create test branch with minimal changes
- Trigger workflows on both agents
- Verify self-hosted runners execute jobs
- Check artifact uploads
- Validate coordination checks work

📁 **Files/Components:**

- Create: Test branch and PR
- Verify: Workflow execution in GitHub Actions
- Review: Artifact downloads
- Check: PR comments from validation

🧪 **Testing Requirements:**

- Workflow triggers correctly
- Jobs run on intended runners
- Tests execute successfully
- Validation scripts run
- Artifacts are accessible
- PR comments are created

⚠️ **Edge Cases:**

- Workflow fails on one agent
- Validation scripts fail
- Runners not available
- Network issues during execution

📚 **Dependencies:** CI-1, CI-2, CI-3 (runners and workflow must be configured)
```

**Shared TODO Entry:**

```markdown
| CI-5 | Test parallel agent CI/CD workflow | shared | pending |
```

---

## Task Creation Commands

### Using TODO2 MCP Tool (Recommended)

If TODO2 MCP tool is available:

```bash

# Create tasks via MCP tool
# (Tool would be called automatically by AI assistant)
```

### Manual Creation

**Option 1: Use Sync Script**

- Add entries to `agents/shared/TODO_OVERVIEW.md`
- Run: `python3 scripts/automate_todo_sync.py`
- Script auto-creates TODO2 tasks

**Option 2: Direct JSON Edit**

- Edit `.todo2/state.todo2.json`
- Add tasks following existing format
- Use next task number: `20251122115564`

---

## Shared TODO Table Updates

Add to `agents/shared/TODO_OVERVIEW.md`:

```markdown
| TODO ID | Description | Owner Agent | Status |
|---------|-------------|-------------|--------|
| CI-1 | Setup GitHub Actions runner on Ubuntu agent | ubuntu | pending |
| CI-2 | Setup GitHub Actions runner on macOS M4 agent | macos | pending |
| CI-3 | Configure enhanced CI/CD workflow for parallel agents | shared | pending |
| CI-4 | Document agent environment and system specifications | shared | pending |
| CI-5 | Test parallel agent CI/CD workflow | shared | pending |
```

---

## Synchronization

### After Creating Tasks

**Run Sync:**

```bash
python3 scripts/automate_todo_sync.py
```

**Verify Sync:**

```bash
bash scripts/validate_todo2_sync.sh
```

---

## References

- [TODO2 CI/CD Integration](./TODO2_CI_CD_INTEGRATION.md) - Integration guide
- [CI/CD Enhancement Plan](./CI_CD_ENHANCEMENT_PLAN.md) - CI/CD setup
- [Self-Hosted Runner Setup](./SELF_HOSTED_RUNNER_SETUP.md) - Runner installation
- [TODO Sync Automation](./TODO_SYNC_AUTOMATION.md) - Sync documentation

---

**Status:** ✅ Tasks documented, ready for creation in TODO2 system

**Next Action:** Create tasks in TODO2 (via MCP tool or manual sync) and add entries to shared TODO table.
