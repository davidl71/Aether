#!/usr/bin/env python3
"""
Add execution context metadata to TODO2 tasks.

Adds execution context tags and description sections to tasks for parallel agent workflows.
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

# Execution context definitions for MCP-EXT tasks
execution_contexts = {
    "MCP-EXT-1": {
        "location": "any",
        "location_type": "local",  # local or worktree
        "best_mode": "Agent",  # Agent, Plan, or Ask
        "mode": ["automated", "background"],
        "resources": [],
        "remote_agent": "any",
        "background": "yes",
        "local_interaction": "not-required",
        "location_tag": "execution-location-any",
        "location_type_tag": "execution-location-type-local",
        "best_mode_tag": "execution-mode-cursor-agent",
        "mode_tags": ["execution-mode-background", "execution-mode-automated"]
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
        "location_tag": "execution-location-any",
        "location_type_tag": "execution-location-type-local",
        "best_mode_tag": "execution-mode-cursor-agent",
        "mode_tags": ["execution-mode-background", "execution-mode-automated"]
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
        "location_tag": "execution-location-remote",
        "location_type_tag": "execution-location-type-local",
        "best_mode_tag": "execution-mode-cursor-agent",
        "mode_tags": ["execution-mode-background", "execution-mode-automated"],
        "resource_tags": ["execution-resource-network"]
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
        "location_tag": "execution-location-any",
        "location_type_tag": "execution-location-type-local",
        "best_mode_tag": "execution-mode-cursor-agent",
        "mode_tags": ["execution-mode-background", "execution-mode-automated"]
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
        "location_tag": "execution-location-any",
        "location_type_tag": "execution-location-type-local",
        "best_mode_tag": "execution-mode-cursor-agent",
        "mode_tags": ["execution-mode-background", "execution-mode-automated"]
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
        "location_tag": "execution-location-any",
        "location_type_tag": "execution-location-type-local",
        "best_mode_tag": "execution-mode-cursor-agent",
        "mode_tags": ["execution-mode-background", "execution-mode-automated"],
        "resource_tags": ["execution-resource-cpu-intensive"]
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
        "location_tag": "execution-location-any",
        "location_type_tag": "execution-location-type-local",
        "best_mode_tag": "execution-mode-cursor-agent",
        "mode_tags": ["execution-mode-background", "execution-mode-automated"],
        "resource_tags": ["execution-resource-cpu-intensive", "execution-resource-disk-intensive"]
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
        "location_tag": "execution-location-any",
        "location_type_tag": "execution-location-type-local",
        "best_mode_tag": "execution-mode-cursor-plan",
        "mode_tags": ["execution-mode-background", "execution-mode-automated"]
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
        "location_tag": "execution-location-any",
        "location_type_tag": "execution-location-type-local",
        "best_mode_tag": "execution-mode-cursor-agent",
        "mode_tags": ["execution-mode-background", "execution-mode-automated"],
        "resource_tags": ["execution-resource-network"]
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
        "location_tag": "execution-location-any",
        "location_type_tag": "execution-location-type-local",
        "best_mode_tag": "execution-mode-cursor-plan",
        "mode_tags": ["execution-mode-background", "execution-mode-automated"]
    }
}

# Update tasks
updated_count = 0
now = datetime.now(timezone.utc).isoformat()

for task in todo2_data.get('todos', []):
    task_id = task.get('id', '')

    if task_id in execution_contexts:
        ctx = execution_contexts[task_id]

        # Add execution context tags
        existing_tags = set(task.get('tags', []))

        # Add location tag
        if ctx.get('location_tag'):
            existing_tags.add(ctx['location_tag'])

        # Add location type tag
        if ctx.get('location_type_tag'):
            existing_tags.add(ctx['location_type_tag'])

        # Add best mode tag
        if ctx.get('best_mode_tag'):
            existing_tags.add(ctx['best_mode_tag'])

        # Add mode tags
        for mode_tag in ctx.get('mode_tags', []):
            existing_tags.add(mode_tag)

        # Add resource tags
        for resource_tag in ctx.get('resource_tags', []):
            existing_tags.add(resource_tag)

        task['tags'] = sorted(list(existing_tags))

        # Add execution context section to long_description
        long_desc = task.get('long_description', '')

        # Create execution context section
        import re
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

        # Check if execution context already exists, update if needed
        if '📋 **Execution Context:**' in long_desc:
            # Replace all existing execution context sections with updated one
            # Pattern matches from Execution Context to next section (Dependencies or end)
            pattern = r'📋 \*\*Execution Context:\*\*.*?(?=\n\n📚 \*\*Dependencies:\*\*|\n\n[📋🎯🚫🔧📁🧪⚠️]|\Z)'
            long_desc = re.sub(pattern, context_section.strip(), long_desc, flags=re.DOTALL)
        else:
            # Add execution context section before Dependencies or at end
            if '📚 **Dependencies:**' in long_desc:
                long_desc = long_desc.replace('📚 **Dependencies:**', context_section + '\n📚 **Dependencies:**')
            else:
                long_desc += context_section

            task['long_description'] = long_desc
            task['lastModified'] = now
            updated_count += 1

# Save updated TODO2 state
with open(todo2_path, 'w') as f:
    json.dump(todo2_data, f, indent=2)

print(f"✅ Updated {updated_count} TODO2 tasks with execution context metadata")
print(f"   Execution context added to: {', '.join(execution_contexts.keys())}")
