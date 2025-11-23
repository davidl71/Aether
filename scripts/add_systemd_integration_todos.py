#!/usr/bin/env python3
"""
Add Systemd Integration Todo2 Tasks

Creates Todo2 tasks for the completed systemd integration work:
- Created systemd service files for all PWA services
- Integrated systemctl support in launch script
- Created installation script
- Added cross-platform OS detection
"""

import json
import sys
from datetime import datetime, timezone
from pathlib import Path

# Project root
PROJECT_ROOT = Path(__file__).parent.parent
TODO2_PATH = PROJECT_ROOT / '.todo2' / 'state.todo2.json'

# Load existing todos
with open(TODO2_PATH, 'r') as f:
    data = json.load(f)

todos = data.get('todos', [])

# Find highest task number
max_task_num = max([int(t['id'][2:]) for t in todos if t['id'].startswith('T-') and t['id'][2:].isdigit()] or [0])

# Systemd integration tasks
now = datetime.now(timezone.utc).isoformat()

new_tasks = [
    {
        'id': f'T-{max_task_num + 1}',
        'name': 'Create systemd service files for all PWA services',
        'long_description': '''🎯 **Objective:** Create systemd user service files for all PWA services to enable proper service management on Linux systems

📋 **Acceptance Criteria:**
- Created systemd service files for all 10 PWA services
- Each service file includes proper working directories
- Environment variables configured (HOME, PATH)
- Restart policies set (on-failure)
- Journal logging configured
- Service dependencies defined (IB depends on Gateway, Rust depends on NATS)
- User-level systemd services (no sudo required)

🚫 **Scope Boundaries (CRITICAL):**
- **Included:** Service file creation only
- **Excluded:** Installation, testing, documentation
- **Clarification Required:** None

🔧 **Technical Requirements:**
- Services: web, alpaca, tradestation, ib-gateway, ib, discount-bank, risk-free-rate, jupyterlab, nats, rust-backend
- Use %h and %i placeholders for user home and username
- Proper ExecStart paths to existing run scripts
- Type=simple for all services

📁 **Files/Components:**
- Create: web/scripts/systemd/pwa-*.service (10 files)
- Location: web/scripts/systemd/

🧪 **Testing Requirements:**
- Verify service file syntax is valid
- Check all paths are correct
- Verify dependencies are properly defined

⚠️ **Edge Cases:**
- IB Gateway has multiple possible run scripts (run-gateway-with-reload.sh, run-gateway.sh, bin/run.sh)
- NATS requires config file path
- Services may need different restart delays

📚 **Dependencies:** None''',
        'status': 'Done',
        'created': now,
        'lastModified': now,
        'taskNumber': max_task_num + 1,
        'priority': 'high',
        'tags': ['systemd', 'linux', 'services', 'infrastructure', 'pwa'],
        'dependencies': [],
        'changes': [
            {
                'field': 'status',
                'oldValue': 'Todo',
                'newValue': 'Done',
                'timestamp': now
            }
        ]
    },
    {
        'id': f'T-{max_task_num + 2}',
        'name': 'Create systemd services installation script',
        'long_description': '''🎯 **Objective:** Create installation script to install systemd service files to user systemd directory with proper path substitution

📋 **Acceptance Criteria:**
- Script detects Linux OS and systemctl availability
- Replaces %h and %i placeholders with actual values
- Installs service files to ~/.config/systemd/user/
- Reloads systemd daemon after installation
- Optional --enable flag to enable services
- Optional --start flag to start services
- Provides helpful usage instructions

🚫 **Scope Boundaries (CRITICAL):**
- **Included:** Installation script only
- **Excluded:** Service file creation, launch script integration
- **Clarification Required:** None

🔧 **Technical Requirements:**
- Use sed to replace placeholders
- Check for systemctl --user availability
- Create ~/.config/systemd/user/ directory if needed
- Run systemctl --user daemon-reload after installation
- Support --enable and --start flags

📁 **Files/Components:**
- Create: web/scripts/install-systemd-services.sh
- Location: web/scripts/

🧪 **Testing Requirements:**
- Verify script works on Linux
- Verify placeholder replacement is correct
- Verify daemon reload happens
- Verify services can be enabled/started

⚠️ **Edge Cases:**
- Script run on non-Linux system (should exit gracefully)
- systemctl not available (should exit with error)
- User systemd directory doesn't exist (should create it)

📚 **Dependencies:** T-{max_task_num + 1} (service files must exist)''',
        'status': 'Done',
        'created': now,
        'lastModified': now,
        'taskNumber': max_task_num + 2,
        'priority': 'high',
        'tags': ['systemd', 'linux', 'installation', 'automation', 'pwa'],
        'dependencies': [f'T-{max_task_num + 1}'],
        'changes': [
            {
                'field': 'status',
                'oldValue': 'Todo',
                'newValue': 'Done',
                'timestamp': now
            }
        ]
    },
    {
        'id': f'T-{max_task_num + 3}',
        'name': 'Integrate systemctl support in launch-all-pwa-services.sh',
        'long_description': '''🎯 **Objective:** Update launch script to automatically detect and use systemctl on Linux, with fallback to brew services (macOS) or manual processes

📋 **Acceptance Criteria:**
- Automatic OS detection (Linux, macOS, other)
- Service manager detection (systemctl, brew, manual)
- Service status checking via systemctl when available
- Service starting/stopping via systemctl when available
- Fallback to existing manual process management
- Status command shows systemctl status when available
- Helpful messages when services not installed

🚫 **Scope Boundaries (CRITICAL):**
- **Included:** Launch script integration only
- **Excluded:** Service file creation, installation script
- **Clarification Required:** None

🔧 **Technical Requirements:**
- detect_service_manager() function
- check_systemctl_service() function
- start_systemctl_service() function
- stop_systemctl_service() function
- Update all service status checks
- Update service starting logic
- Update stop_services() function
- Update status command

📁 **Files/Components:**
- Update: web/scripts/launch-all-pwa-services.sh
- Location: web/scripts/

🧪 **Testing Requirements:**
- Verify systemctl detection works on Linux
- Verify fallback to manual on non-Linux
- Verify service status checking works
- Verify service starting/stopping works
- Verify status command shows correct info

⚠️ **Edge Cases:**
- systemctl available but services not installed (should show helpful message)
- systemctl available but user services not enabled (should handle gracefully)
- Mixed environment (some services via systemctl, some manual)

📚 **Dependencies:** T-{max_task_num + 1}, T-{max_task_num + 2}''',
        'status': 'Done',
        'created': now,
        'lastModified': now,
        'taskNumber': max_task_num + 3,
        'priority': 'high',
        'tags': ['systemd', 'linux', 'macos', 'cross-platform', 'launch-script', 'pwa'],
        'dependencies': [f'T-{max_task_num + 1}', f'T-{max_task_num + 2}'],
        'changes': [
            {
                'field': 'status',
                'oldValue': 'Todo',
                'newValue': 'Done',
                'timestamp': now
            }
        ]
    },
    {
        'id': f'T-{max_task_num + 4}',
        'name': 'Add systemd integration documentation',
        'long_description': '''🎯 **Objective:** Create comprehensive documentation for systemd integration including usage, troubleshooting, and cross-platform compatibility

📋 **Acceptance Criteria:**
- README.md in systemd directory
- Installation instructions
- Usage examples (systemctl commands)
- Service management commands
- Log viewing instructions
- Troubleshooting guide
- Cross-platform compatibility notes
- Uninstallation instructions

🚫 **Scope Boundaries (CRITICAL):**
- **Included:** Documentation only
- **Excluded:** Code changes, testing
- **Clarification Required:** None

🔧 **Technical Requirements:**
- Markdown format
- Clear sections with examples
- Command reference
- Troubleshooting common issues
- Links to related files

📁 **Files/Components:**
- Create: web/scripts/systemd/README.md
- Location: web/scripts/systemd/

🧪 **Testing Requirements:**
- Verify all commands are correct
- Verify file paths are accurate
- Verify examples work as documented

⚠️ **Edge Cases:**
- Different Linux distributions may have slight differences
- User may not have systemd user services enabled

📚 **Dependencies:** T-{max_task_num + 1}, T-{max_task_num + 2}, T-{max_task_num + 3}''',
        'status': 'Done',
        'created': now,
        'lastModified': now,
        'taskNumber': max_task_num + 4,
        'priority': 'medium',
        'tags': ['systemd', 'documentation', 'pwa'],
        'dependencies': [f'T-{max_task_num + 1}', f'T-{max_task_num + 2}', f'T-{max_task_num + 3}'],
        'changes': [
            {
                'field': 'status',
                'oldValue': 'Todo',
                'newValue': 'Done',
                'timestamp': now
            }
        ]
    }
]

# Add new tasks
for task in new_tasks:
    # Fix dependency references to use actual IDs
    if 'dependencies' in task:
        fixed_deps = []
        for dep in task['dependencies']:
            if isinstance(dep, str) and dep.startswith('T-'):
                # Check if it's a reference to one of our new tasks
                dep_num = int(dep[2:])
                if dep_num <= max_task_num + 4:
                    fixed_deps.append(f'T-{dep_num}')
                else:
                    fixed_deps.append(dep)
            else:
                fixed_deps.append(dep)
        task['dependencies'] = fixed_deps

    todos.append(task)

# Update data
data['todos'] = todos

# Save
with open(TODO2_PATH, 'w') as f:
    json.dump(data, f, indent=2)

print(f"✅ Added {len(new_tasks)} systemd integration tasks to Todo2")
print(f"Task IDs: {', '.join([t['id'] for t in new_tasks])}")
print(f"\nTasks created:")
for task in new_tasks:
    print(f"  - {task['id']}: {task['name']} ({task['status']})")
