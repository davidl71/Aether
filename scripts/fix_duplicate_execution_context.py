#!/usr/bin/env python3
"""
Fix duplicate execution context sections in TODO2 tasks.

Removes duplicate execution context sections and ensures only one exists.
"""

import json
import re
from datetime import datetime, timezone
from pathlib import Path

# Project root
project_root = Path(__file__).parent.parent
todo2_path = project_root / '.todo2' / 'state.todo2.json'

# Load existing TODO2 state
with open(todo2_path, 'r') as f:
    todo2_data = json.load(f)

# Execution context definitions (same as add script)
execution_contexts = {
    "MCP-EXT-1": {
        "location": "any",
        "location_type": "local",
        "best_mode": "Agent",
        "mode": ["automated", "background"],
        "resources": [],
        "remote_agent": "any",
        "background": "yes",
        "local_interaction": "not-required",
    },
    "MCP-EXT-2": {
        "location": "any",
        "location_type": "local",
        "best_mode": "Agent",
        "mode": ["automated", "background"],
        "resources": [],
        "remote_agent": "any",
        "background": "yes",
        "local_interaction": "not-required",
    },
    "MCP-EXT-3": {
        "location": "remote",
        "location_type": "local",
        "best_mode": "Agent",
        "mode": ["automated", "background"],
        "resources": ["network"],
        "remote_agent": "ubuntu-agent or macos-m4-agent",
        "background": "yes",
        "local_interaction": "not-required",
    },
    "MCP-EXT-4": {
        "location": "any",
        "location_type": "local",
        "best_mode": "Agent",
        "mode": ["automated", "background"],
        "resources": [],
        "remote_agent": "any",
        "background": "yes",
        "local_interaction": "not-required",
    },
    "MCP-EXT-5": {
        "location": "any",
        "location_type": "local",
        "best_mode": "Agent",
        "mode": ["automated", "background"],
        "resources": [],
        "remote_agent": "any",
        "background": "yes",
        "local_interaction": "not-required",
    },
    "MCP-EXT-6": {
        "location": "any",
        "location_type": "local",
        "best_mode": "Agent",
        "mode": ["automated", "background"],
        "resources": ["cpu-intensive"],
        "remote_agent": "any",
        "background": "yes",
        "local_interaction": "not-required",
    },
    "MCP-EXT-7": {
        "location": "any",
        "location_type": "local",
        "best_mode": "Agent",
        "mode": ["automated", "background"],
        "resources": ["cpu-intensive", "disk-intensive"],
        "remote_agent": "any",
        "background": "yes",
        "local_interaction": "not-required",
    },
    "MCP-EXT-8": {
        "location": "any",
        "location_type": "local",
        "best_mode": "Plan",
        "mode": ["automated", "background"],
        "resources": [],
        "remote_agent": "any",
        "background": "yes",
        "local_interaction": "not-required",
    },
    "MCP-EXT-9": {
        "location": "any",
        "location_type": "local",
        "best_mode": "Agent",
        "mode": ["automated", "background"],
        "resources": ["network"],
        "remote_agent": "any",
        "background": "yes",
        "local_interaction": "not-required",
    },
    "MCP-EXT-10": {
        "location": "any",
        "location_type": "local",
        "best_mode": "Plan",
        "mode": ["automated", "background"],
        "resources": [],
        "remote_agent": "any",
        "background": "yes",
        "local_interaction": "not-required",
    }
}

# Update tasks
fixed_count = 0
now = datetime.now(timezone.utc).isoformat()

for task in todo2_data.get('todos', []):
    task_id = task.get('id', '')

    if task_id in execution_contexts:
        ctx = execution_contexts[task_id]

        # Create proper execution context section
        context_section = f"""📋 **Execution Context:**
- **Location:** `{ctx['location']}` ({ctx.get('remote_agent', 'any')})
- **Location Type:** `{ctx.get('location_type', 'local')}` (where to execute)
- **Best Mode:** `{ctx.get('best_mode', 'Agent')}` (Cursor AI mode: Agent/Plan/Ask)
- **Mode:** `{'` | `'.join(ctx['mode'])}`
- **Resources:** {', '.join(ctx['resources']) if ctx['resources'] else 'None'}
- **Remote Agent:** `{ctx['remote_agent']}`
- **Background:** `{ctx['background']}`
- **Local Interaction:** `{ctx['local_interaction']}`
"""

        long_desc = task.get('long_description', '')

        # Remove all existing execution context sections
        # Pattern matches from Execution Context to next major section
        pattern = r'📋 \*\*Execution Context:\*\*.*?(?=\n\n📚 \*\*Dependencies:\*\*|\n\n[📋🎯🚫🔧📁🧪⚠️]|\Z)'
        long_desc = re.sub(pattern, '', long_desc, flags=re.DOTALL)

        # Clean up multiple newlines
        long_desc = re.sub(r'\n\n\n+', '\n\n', long_desc)

        # Add single execution context section before Dependencies
        if '📚 **Dependencies:**' in long_desc:
            long_desc = long_desc.replace('📚 **Dependencies:**', context_section + '\n📚 **Dependencies:**')
        else:
            long_desc += '\n' + context_section

        task['long_description'] = long_desc
        task['lastModified'] = now
        fixed_count += 1

# Save updated TODO2 state
with open(todo2_path, 'w') as f:
    json.dump(todo2_data, f, indent=2)

print(f"✅ Fixed {fixed_count} TODO2 tasks with duplicate execution context sections")
print(f"   Tasks fixed: {', '.join(execution_contexts.keys())}")
