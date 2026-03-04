#!/usr/bin/env python3
"""
Create TODO2 tasks for Project Automation MCP Server extensions.

DEPRECATED: The Python project-management-automation MCP server is no longer used.
Use exarp-go for project automation (see docs/MCP_REQUIRED_SERVERS.md).
This script is kept for historical reference; tasks refer to mcp-servers/project-management-automation
which is not part of this repo anymore.
"""

import json
import sys
from datetime import datetime, timezone
from pathlib import Path

# Project root
project_root = Path(__file__).parent.parent
todo2_path = project_root / '.todo2' / 'state.todo2.json'

# Load existing TODO2 state
with open(todo2_path, 'r') as f:
    todo2_data = json.load(f)

# Get next task number
next_task_num = todo2_data.get('nextTaskNumber', 1)
if isinstance(next_task_num, str) and next_task_num.startswith('2025'):
    # Extract number from timestamp format
    base_num = int(next_task_num) if next_task_num.isdigit() else 1
else:
    base_num = int(next_task_num) if isinstance(next_task_num, (int, str)) else 1

# Task definitions
tasks = [
    {
        "id": f"MCP-EXT-1",
        "name": "Implement validate_ci_cd_workflow_tool for CI/CD validation",
        "priority": "high",
        "tags": ["mcp", "ci-cd", "validation", "parallel-agents"],
        "long_description": """🎯 **Objective:** Implement `validate_ci_cd_workflow_tool` MCP tool to validate CI/CD workflows and runner configurations for parallel agent development.

📋 **Acceptance Criteria:**
- Tool validates GitHub Actions workflow syntax
- Tool checks self-hosted runner configurations
- Tool verifies workflow job dependencies
- Tool validates runner labels and matrix builds
- Tool checks workflow trigger configurations
- Tool validates artifact uploads/downloads
- Tool generates validation report
- Tool integrated into MCP server with proper error handling

🚫 **Scope Boundaries (CRITICAL):**
- **Included:** Workflow validation, runner config checks, report generation, MCP integration
- **Excluded:** Actual workflow execution, runner installation, workflow modifications

🔧 **Technical Requirements:**
- Follow existing MCP tool pattern (see `tools/docs_health.py`)
- Use IntelligentAutomationBase framework
- Parse YAML workflow files
- Validate runner labels and matrix builds
- Generate structured validation report
- Register tool in `server.py`

📁 **Files/Components:**
- Create: `mcp-servers/project-management-automation/tools/ci_cd_validation.py`
- Update: `mcp-servers/project-management-automation/server.py`
- Update: `mcp-servers/project-management-automation/TOOLS_STATUS.md`

🧪 **Testing Requirements:**
- Unit tests for workflow parsing
- Integration test with sample workflow
- Manual test via MCP interface
- Validate error handling

⚠️ **Edge Cases:**
- Invalid YAML syntax
- Missing workflow file
- Runner configuration issues
- Circular job dependencies

📚 **Dependencies:** None

📋 **Execution Context:**
- **Location:** `any` (can run on any agent)
- **Mode:** `automated` | `background`
- **Resources:** None
- **Remote Agent:** `any` (ubuntu-agent or macos-m4-agent)
- **Background:** `yes`
- **Local Interaction:** `not-required`"""
    },
    {
        "id": f"MCP-EXT-2",
        "name": "Implement validate_agent_coordination_tool for parallel agent coordination",
        "priority": "high",
        "tags": ["mcp", "coordination", "parallel-agents", "validation"],
        "long_description": """🎯 **Objective:** Implement `validate_agent_coordination_tool` MCP tool to validate coordination between parallel agents (Ubuntu + macOS).

📋 **Acceptance Criteria:**
- Tool validates shared TODO table format
- Tool checks API contract consistency
- Tool validates TODO2 sync status
- Tool checks for merge conflicts
- Tool validates agent task assignments
- Tool generates coordination validation report
- Tool wraps existing validation scripts
- Tool integrated into MCP server

🚫 **Scope Boundaries (CRITICAL):**
- **Included:** Coordination validation, script orchestration, report generation
- **Excluded:** Script implementation (already exists), actual fixes

🔧 **Technical Requirements:**
- Wrap existing scripts: `validate_todo_table.sh`, `validate_api_contract.sh`, `validate_todo2_sync.sh`
- Use IntelligentAutomationBase framework
- Combine validation results into unified report
- Follow existing MCP tool pattern
- Register tool in `server.py`

📁 **Files/Components:**
- Create: `mcp-servers/project-management-automation/tools/agent_coordination.py`
- Update: `mcp-servers/project-management-automation/server.py`
- Update: `mcp-servers/project-management-automation/TOOLS_STATUS.md`
- Use: `scripts/validate_todo_table.sh`, `scripts/validate_api_contract.sh`, `scripts/validate_todo2_sync.sh`

🧪 **Testing Requirements:**
- Test with valid coordination state
- Test with coordination issues
- Test individual validation components
- Manual test via MCP interface

⚠️ **Edge Cases:**
- Validation script failures
- Missing validation scripts
- Partial validation failures
- Network issues (for remote checks)

📚 **Dependencies:** MCP-EXT-1 (validation pattern established)"""
    },
    {
        "id": f"MCP-EXT-3",
        "name": "Implement collect_agent_environment_tool for environment documentation",
        "priority": "high",
        "tags": ["mcp", "environment", "documentation", "parallel-agents"],
        "long_description": """🎯 **Objective:** Implement `collect_agent_environment_tool` MCP tool to collect and document agent environment information from remote agents.

📋 **Acceptance Criteria:**
- Tool collects system info from remote agents via SSH
- Tool documents OS, CPU, RAM, disk information
- Tool detects Apple Intelligence availability
- Tool verifies development tool versions
- Tool generates environment documentation
- Tool compares agent environments
- Tool updates DEVELOPMENT_ENVIRONMENT.md
- Tool integrated into MCP server

🚫 **Scope Boundaries (CRITICAL):**
- **Included:** Environment collection, documentation generation, MCP integration
- **Excluded:** SSH configuration, system changes, agent setup

🔧 **Technical Requirements:**
- Wrap `scripts/collect_system_info_python.py`
- Use IntelligentAutomationBase framework
- Support SSH connection to remote agents
- Parse and format system information
- Update documentation file
- Follow existing MCP tool pattern
- Register tool in `server.py`

📁 **Files/Components:**
- Create: `mcp-servers/project-management-automation/tools/agent_environment.py`
- Update: `mcp-servers/project-management-automation/server.py`
- Update: `mcp-servers/project-management-automation/TOOLS_STATUS.md`
- Use: `scripts/collect_system_info_python.py`

🧪 **Testing Requirements:**
- Test local environment collection
- Test remote environment collection (if SSH available)
- Test documentation updates
- Test error handling for connection failures
- Manual test via MCP interface

⚠️ **Edge Cases:**
- SSH connection failures
- Missing system information
- Permission issues
- Network connectivity problems

📚 **Dependencies:** None"""
    },
    {
        "id": f"MCP-EXT-4",
        "name": "Implement validate_api_contract_tool for API contract validation",
        "priority": "medium",
        "tags": ["mcp", "api", "validation", "integration"],
        "long_description": """🎯 **Objective:** Implement `validate_api_contract_tool` MCP tool to validate API contract consistency across agents.

📋 **Acceptance Criteria:**
- Tool parses backend code for API endpoints (Rust/Python)
- Tool extracts request/response schemas
- Tool compares with API_CONTRACT.md
- Tool detects API drift
- Tool flags breaking changes
- Tool generates diff report
- Tool integrated into MCP server

🚫 **Scope Boundaries (CRITICAL):**
- **Included:** API parsing, contract comparison, drift detection, report generation
- **Excluded:** API fixes, contract updates, actual code changes

🔧 **Technical Requirements:**
- Parse Rust code for API endpoints (use syn crate or similar)
- Parse Python code for API endpoints (use AST)
- Parse API_CONTRACT.md format
- Compare and detect differences
- Generate structured drift report
- Follow existing MCP tool pattern
- Register tool in `server.py`

📁 **Files/Components:**
- Create: `mcp-servers/project-management-automation/tools/api_contract_validation.py`
- Update: `mcp-servers/project-management-automation/server.py`
- Update: `mcp-servers/project-management-automation/TOOLS_STATUS.md`
- Use: `agents/shared/API_CONTRACT.md`

🧪 **Testing Requirements:**
- Test with valid API contract
- Test with API drift
- Test with breaking changes
- Test with missing contract file
- Manual test via MCP interface

⚠️ **Edge Cases:**
- Missing API endpoints in code
- Missing API contract file
- Invalid contract format
- Complex schema differences

📚 **Dependencies:** None"""
    },
    {
        "id": f"MCP-EXT-5",
        "name": "Implement monitor_feature_parity_tool for TUI/PWA parity tracking",
        "priority": "medium",
        "tags": ["mcp", "feature-parity", "monitoring", "tui", "pwa"],
        "long_description": """🎯 **Objective:** Implement `monitor_feature_parity_tool` MCP tool to monitor feature parity between TUI and PWA implementations.

📋 **Acceptance Criteria:**
- Tool compares TUI vs PWA features
- Tool identifies feature gaps
- Tool tracks parity trends over time
- Tool generates parity report
- Tool optionally creates TODO2 tasks for gaps
- Tool enhances existing feature parity script
- Tool integrated into MCP server

🚫 **Scope Boundaries (CRITICAL):**
- **Included:** Feature comparison, gap analysis, trend tracking, report generation
- **Excluded:** Feature implementation, actual fixes

🔧 **Technical Requirements:**
- Enhance `scripts/check_feature_parity.sh`
- Parse TUI and PWA codebases
- Compare feature implementations
- Track trends over time
- Generate parity report
- Follow existing MCP tool pattern
- Register tool in `server.py`

📁 **Files/Components:**
- Create: `mcp-servers/project-management-automation/tools/feature_parity.py`
- Update: `mcp-servers/project-management-automation/server.py`
- Update: `mcp-servers/project-management-automation/TOOLS_STATUS.md`
- Enhance: `scripts/check_feature_parity.sh`

🧪 **Testing Requirements:**
- Test feature comparison logic
- Test gap detection
- Test trend tracking
- Test TODO2 task creation
- Manual test via MCP interface

⚠️ **Edge Cases:**
- Missing feature implementations
- Partial feature implementations
- Feature naming mismatches
- Complex feature dependencies

📚 **Dependencies:** None"""
    },
    {
        "id": f"MCP-EXT-6",
        "name": "Implement track_test_coverage_tool for test coverage monitoring",
        "priority": "medium",
        "tags": ["mcp", "testing", "coverage", "monitoring"],
        "long_description": """🎯 **Objective:** Implement `track_test_coverage_tool` MCP tool to track test coverage trends across languages (C++, Python, Rust).

📋 **Acceptance Criteria:**
- Tool runs tests with coverage (C++, Python, Rust)
- Tool compares coverage with previous runs
- Tool tracks trends over time
- Tool identifies coverage gaps
- Tool generates coverage report
- Tool alerts on coverage drops
- Tool integrated into MCP server

🚫 **Scope Boundaries (CRITICAL):**
- **Included:** Coverage collection, trend tracking, report generation, alerts
- **Excluded:** Test implementation, coverage fixes

🔧 **Technical Requirements:**
- Use coverage tools: gcov (C++), coverage.py (Python), cargo-tarpaulin (Rust)
- Store historical coverage data
- Compare and track trends
- Generate coverage reports
- Follow existing MCP tool pattern
- Register tool in `server.py`

📁 **Files/Components:**
- Create: `mcp-servers/project-management-automation/tools/test_coverage.py`
- Update: `mcp-servers/project-management-automation/server.py`
- Update: `mcp-servers/project-management-automation/TOOLS_STATUS.md`
- Create: Coverage data storage location

🧪 **Testing Requirements:**
- Test coverage collection for each language
- Test trend tracking
- Test coverage drop alerts
- Test with missing coverage data
- Manual test via MCP interface

⚠️ **Edge Cases:**
- Missing coverage tools
- Test failures during coverage collection
- Missing historical data
- Coverage tool errors

📚 **Dependencies:** None"""
    },
    {
        "id": f"MCP-EXT-7",
        "name": "Implement monitor_build_health_tool for build health monitoring",
        "priority": "low",
        "tags": ["mcp", "build", "monitoring", "ci-cd"],
        "long_description": """🎯 **Objective:** Implement `monitor_build_health_tool` MCP tool to monitor build health across platforms.

📋 **Acceptance Criteria:**
- Tool tests builds for all platforms (macOS, Linux, WASM)
- Tool checks build times (regression detection)
- Tool validates build outputs
- Tool tracks build health trends
- Tool generates build health report
- Tool integrated into MCP server

🚫 **Scope Boundaries (CRITICAL):**
- **Included:** Build execution, time tracking, validation, report generation
- **Excluded:** Build fixes, platform-specific optimizations

🔧 **Technical Requirements:**
- Execute build commands for each platform
- Measure and track build times
- Validate build outputs
- Track trends over time
- Generate health reports
- Follow existing MCP tool pattern
- Register tool in `server.py`

📁 **Files/Components:**
- Create: `mcp-servers/project-management-automation/tools/build_health.py`
- Update: `mcp-servers/project-management-automation/server.py`
- Update: `mcp-servers/project-management-automation/TOOLS_STATUS.md`

🧪 **Testing Requirements:**
- Test build execution
- Test build time tracking
- Test build validation
- Test trend tracking
- Manual test via MCP interface

⚠️ **Edge Cases:**
- Build failures
- Missing build tools
- Platform-specific issues
- Build timeout issues

📚 **Dependencies:** None"""
    },
    {
        "id": f"MCP-EXT-8",
        "name": "Implement analyze_agent_task_distribution_tool for workload analysis",
        "priority": "low",
        "tags": ["mcp", "analysis", "task-distribution", "parallel-agents"],
        "long_description": """🎯 **Objective:** Implement `analyze_agent_task_distribution_tool` MCP tool to analyze task distribution across agents.

📋 **Acceptance Criteria:**
- Tool analyzes TODO2 task distribution by agent
- Tool identifies workload imbalances
- Tool suggests task reassignments
- Tool tracks agent productivity metrics
- Tool generates distribution report
- Tool integrated into MCP server

🚫 **Scope Boundaries (CRITICAL):**
- **Included:** Task analysis, distribution metrics, recommendations
- **Excluded:** Actual task reassignments, agent management

🔧 **Technical Requirements:**
- Parse TODO2 tasks by agent assignment
- Calculate distribution metrics
- Identify imbalances
- Generate recommendations
- Track productivity over time
- Follow existing MCP tool pattern
- Register tool in `server.py`

📁 **Files/Components:**
- Create: `mcp-servers/project-management-automation/tools/agent_distribution.py`
- Update: `mcp-servers/project-management-automation/server.py`
- Update: `mcp-servers/project-management-automation/TOOLS_STATUS.md`
- Use: `.todo2/state.todo2.json`

🧪 **Testing Requirements:**
- Test distribution analysis
- Test imbalance detection
- Test recommendation generation
- Test with various task distributions
- Manual test via MCP interface

⚠️ **Edge Cases:**
- No agent assignments
- Uneven task distribution
- Missing task metadata
- Complex dependencies

📚 **Dependencies:** None"""
    },
    {
        "id": f"MCP-EXT-9",
        "name": "Implement validate_runner_health_tool for runner health monitoring",
        "priority": "low",
        "tags": ["mcp", "runners", "health", "ci-cd"],
        "long_description": """🎯 **Objective:** Implement `validate_runner_health_tool` MCP tool to monitor self-hosted runner health.

📋 **Acceptance Criteria:**
- Tool checks runner connectivity
- Tool monitors runner status
- Tool detects offline runners
- Tool checks runner resource usage
- Tool generates runner health report
- Tool integrated into MCP server

🚫 **Scope Boundaries (CRITICAL):**
- **Included:** Runner status checks, health monitoring, report generation
- **Excluded:** Runner installation, runner fixes

🔧 **Technical Requirements:**
- Use GitHub Actions API or runner status checks
- Check runner connectivity
- Monitor resource usage
- Detect offline runners
- Generate health reports
- Follow existing MCP tool pattern
- Register tool in `server.py`

📁 **Files/Components:**
- Create: `mcp-servers/project-management-automation/tools/runner_health.py`
- Update: `mcp-servers/project-management-automation/server.py`
- Update: `mcp-servers/project-management-automation/TOOLS_STATUS.md`

🧪 **Testing Requirements:**
- Test runner status checks
- Test offline detection
- Test resource monitoring
- Test with various runner states
- Manual test via MCP interface

⚠️ **Edge Cases:**
- Runner API unavailable
- Network connectivity issues
- Missing runner permissions
- Runner configuration issues

📚 **Dependencies:** None"""
    },
    {
        "id": f"MCP-EXT-10",
        "name": "Implement generate_coordination_report_tool for unified coordination reports",
        "priority": "low",
        "tags": ["mcp", "coordination", "reporting", "orchestration"],
        "long_description": """🎯 **Objective:** Implement `generate_coordination_report_tool` MCP tool to generate comprehensive coordination reports.

📋 **Acceptance Criteria:**
- Tool combines all coordination validations
- Tool generates unified report
- Tool tracks coordination health trends
- Tool identifies coordination issues
- Tool optionally creates follow-up TODO2 tasks
- Tool orchestrates multiple validation tools
- Tool integrated into MCP server

🚫 **Scope Boundaries (CRITICAL):**
- **Included:** Report orchestration, unified reporting, trend tracking
- **Excluded:** Individual validation implementations, actual fixes

🔧 **Technical Requirements:**
- Orchestrate multiple validation tools
- Combine validation results
- Generate unified reports
- Track coordination health
- Create TODO2 tasks for issues
- Follow existing MCP tool pattern
- Register tool in `server.py`

📁 **Files/Components:**
- Create: `mcp-servers/project-management-automation/tools/coordination_report.py`
- Update: `mcp-servers/project-management-automation/server.py`
- Update: `mcp-servers/project-management-automation/TOOLS_STATUS.md`
- Use: Other coordination validation tools

🧪 **Testing Requirements:**
- Test orchestration of validation tools
- Test unified report generation
- Test trend tracking
- Test TODO2 task creation
- Manual test via MCP interface

⚠️ **Edge Cases:**
- Validation tool failures
- Missing validation tools
- Partial validation failures
- Report generation errors

📚 **Dependencies:** MCP-EXT-2 (agent coordination tool)"""
    }
]

# Add tasks to TODO2
now = datetime.now(timezone.utc).isoformat()
task_number = base_num

for task_def in tasks:
    task = {
        "id": task_def["id"],
        "name": task_def["name"],
        "long_description": task_def["long_description"],
        "status": "Todo",
        "created": now,
        "lastModified": now,
        "taskNumber": task_number,
        "priority": task_def["priority"],
        "tags": task_def["tags"],
        "dependencies": []
    }

    # Add dependencies if specified
    if "Dependencies:" in task_def["long_description"]:
        deps_line = [l for l in task_def["long_description"].split("\n") if "Dependencies:" in l]
        if deps_line:
            deps_text = deps_line[0].split("Dependencies:")[-1].strip()
            if deps_text and deps_text.lower() != "none":
                # Extract task IDs from dependencies text
                for dep_task_def in tasks:
                    if dep_task_def["id"] in deps_text:
                        task["dependencies"].append(dep_task_def["id"])

    todo2_data.setdefault("todos", []).append(task)
    task_number += 1

# Update next task number
todo2_data["nextTaskNumber"] = task_number
todo2_data["lastModified"] = now

# Save updated TODO2 state
with open(todo2_path, 'w') as f:
    json.dump(todo2_data, f, indent=2)

print(f"✅ Created {len(tasks)} TODO2 tasks for MCP extensions")
print(f"   Task IDs: {', '.join([t['id'] for t in tasks])}")
