#!/bin/bash
# Import tasks from EXARP_TASKS_IMPORT.json into an exarp-go repo's .todo2 (legacy).
# Usage: ./scripts/import_exarp_tasks.sh [path-to-exarp-go-repo]
# We only use exarp-go; exarp-project-management is no longer used.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
IMPORT_FILE="$PROJECT_ROOT/docs/EXARP_TASKS_IMPORT.json"

# Check if import file exists
if [ ! -f "$IMPORT_FILE" ]; then
    echo "❌ Import file not found: $IMPORT_FILE"
    exit 1
fi

# Get exarp-go repository path
if [ -n "$1" ]; then
    EXARP_REPO="$1"
else
    if [ -n "${EXARP_GO_ROOT:-}" ] && [ -d "$EXARP_GO_ROOT" ]; then
        EXARP_REPO="$EXARP_GO_ROOT"
    elif [ -d "$PROJECT_ROOT/../exarp-go" ]; then
        EXARP_REPO="$PROJECT_ROOT/../exarp-go"
    else
        echo "❌ exarp-go repository not found"
        echo "Usage: $0 [path-to-exarp-go-repo]"
        echo "Or set EXARP_GO_ROOT to the exarp-go repo root."
        exit 1
    fi
fi

# Verify exarp-go repository
if [ ! -d "$EXARP_REPO" ]; then
    echo "❌ exarp-go repository not found: $EXARP_REPO"
    exit 1
fi

if [ ! -d "$EXARP_REPO/.todo2" ]; then
    echo "⚠️  Creating .todo2 directory in exarp-go repository..."
    mkdir -p "$EXARP_REPO/.todo2"
fi

# Copy import file
echo "📋 Copying import file to exarp-go repository..."
cp "$IMPORT_FILE" "$EXARP_REPO/EXARP_TASKS_IMPORT.json"
echo "✅ Copied to: $EXARP_REPO/EXARP_TASKS_IMPORT.json"

# Run Python import script if it exists, or create one
IMPORT_SCRIPT="$EXARP_REPO/scripts/import_tasks.py"
if [ ! -f "$IMPORT_SCRIPT" ]; then
    echo "📝 Creating import script..."
    mkdir -p "$EXARP_REPO/scripts"
    cat > "$IMPORT_SCRIPT" << 'PYTHON_SCRIPT'
#!/usr/bin/env python3
"""Import tasks from EXARP_TASKS_IMPORT.json into Todo2 state (legacy; exarp-go repo)."""
import json
import sys
from datetime import datetime
from pathlib import Path

def import_tasks():
    # Paths
    repo_root = Path(__file__).parent.parent
    import_file = repo_root / "EXARP_TASKS_IMPORT.json"
    todo2_file = repo_root / ".todo2" / "state.todo2.json"

    # Read import file
    if not import_file.exists():
        print(f"❌ Import file not found: {import_file}")
        sys.exit(1)

    with open(import_file, 'r') as f:
        import_data = json.load(f)

    # Read or create Todo2 state
    if todo2_file.exists():
        with open(todo2_file, 'r') as f:
            todo2_data = json.load(f)
    else:
        todo2_data = {
            "todos": [],
            "metadata": {
                "created": datetime.now().isoformat(),
                "version": "1.0"
            }
        }

    # ID mapping
    id_mapping = {
        'T-1': 'EXARP-1',
        'T-2': 'EXARP-2',
        'T-3': 'EXARP-3',
        'T-4': 'EXARP-4',
        'T-5': 'EXARP-5'
    }

    # Import tasks
    imported_count = 0
    for task in import_data['tasks']:
        old_id = task['id']
        new_id = id_mapping.get(old_id, f"EXARP-{old_id}")

        # Check if task already exists
        existing = [t for t in todo2_data.get('todos', []) if t.get('id') == new_id]
        if existing:
            print(f"⚠️  Task {new_id} already exists, skipping...")
            continue

        # Update task
        task['id'] = new_id
        task['migrated_from'] = old_id
        task['migrated_at'] = datetime.now().isoformat()

        # Ensure todos list exists
        if 'todos' not in todo2_data:
            todo2_data['todos'] = []

        # Add task
        todo2_data['todos'].append(task)
        imported_count += 1
        print(f"✅ Imported {new_id}: {task.get('content', '')[:50]}...")

    # Save updated state
    todo2_file.parent.mkdir(parents=True, exist_ok=True)
    with open(todo2_file, 'w') as f:
        json.dump(todo2_data, f, indent=2)

    print(f"\n✅ Successfully imported {imported_count} tasks")
    print(f"📁 Todo2 state: {todo2_file}")

    return imported_count

if __name__ == "__main__":
    try:
        count = import_tasks()
        sys.exit(0 if count > 0 else 1)
    except Exception as e:
        print(f"❌ Error: {e}", file=sys.stderr)
        sys.exit(1)
PYTHON_SCRIPT
    chmod +x "$IMPORT_SCRIPT"
    echo "✅ Created import script: $IMPORT_SCRIPT"
fi

# Run import script
echo ""
echo "🚀 Running import script..."
cd "$EXARP_REPO"
python3 "$IMPORT_SCRIPT"

echo ""
echo "✅ Import complete!"
echo "📁 exarp-go repository: $EXARP_REPO"
echo "📋 Tasks imported as EXARP-1 through EXARP-5"
