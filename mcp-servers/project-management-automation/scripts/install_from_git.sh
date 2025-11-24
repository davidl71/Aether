#!/bin/bash
# Install automa from Git repository

set -e

# Configuration
REPO_URL="${AUTOMA_REPO_URL:-https://github.com/username/project-management-automation.git}"
BRANCH="${AUTOMA_BRANCH:-main}"
VERSION="${AUTOMA_VERSION:-}"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Installing automa MCP server...${NC}"
echo "Repository: $REPO_URL"
echo "Branch: $BRANCH"
if [ -n "$VERSION" ]; then
    echo "Version: $VERSION"
    INSTALL_URL="$REPO_URL@$VERSION"
else
    INSTALL_URL="$REPO_URL@$BRANCH"
fi

# Install from Git
echo -e "${BLUE}Installing from Git repository...${NC}"
pip install "git+$INSTALL_URL"

echo -e "${GREEN}Installation complete!${NC}"
echo ""
echo "To configure in Cursor, update .cursor/mcp.json:"
echo '  "project-management-automation": {'
echo '    "command": "python3",'
echo '    "args": ["-m", "project_management_automation.server"]'
echo '  }'
