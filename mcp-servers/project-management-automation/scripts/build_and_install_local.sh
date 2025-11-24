#!/bin/bash
# Build and install automa locally (editable mode)

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

echo -e "${BLUE}Building and installing automa locally...${NC}"
echo "Project directory: $PROJECT_DIR"
echo ""

# Check if we're in the right directory
if [ ! -f "$PROJECT_DIR/pyproject.toml" ]; then
    echo -e "${YELLOW}Error: pyproject.toml not found. Are you in the project directory?${NC}"
    exit 1
fi

# Install in editable mode
echo -e "${BLUE}Installing in editable mode (development)...${NC}"
cd "$PROJECT_DIR"
pip install -e .

echo -e "${GREEN}Installation complete!${NC}"
echo ""
echo "Package installed in editable mode. Changes to code will be immediately available."
echo ""
echo "To configure in Cursor, update .cursor/mcp.json:"
echo '  "project-management-automation": {'
echo '    "command": "python3",'
echo '    "args": ["-m", "project_management_automation.server"]'
echo '  }'
