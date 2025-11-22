#!/bin/bash
# Test Ona (Gitpod) connectivity and configuration
# This script verifies that the project is properly configured for Ona

set -e

echo "🔍 Testing Ona Project Connectivity"
echo "===================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

# Test 1: Check .gitpod.yml exists
echo "Test 1: Checking .gitpod.yml configuration..."
if [ -f ".gitpod.yml" ]; then
    echo -e "${GREEN}✅ .gitpod.yml exists${NC}"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}❌ .gitpod.yml not found${NC}"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi

# Test 2: Validate .gitpod.yml syntax
echo ""
echo "Test 2: Validating .gitpod.yml syntax..."
if command -v python3 &> /dev/null; then
    if python3 -c "import yaml; yaml.safe_load(open('.gitpod.yml'))" 2>/dev/null; then
        echo -e "${GREEN}✅ .gitpod.yml syntax is valid${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}❌ .gitpod.yml syntax error${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
else
    echo -e "${YELLOW}⚠️  Python3 not available, skipping YAML validation${NC}"
fi

# Test 3: Check .gitpod.Dockerfile exists
echo ""
echo "Test 3: Checking .gitpod.Dockerfile..."
if [ -f ".gitpod.Dockerfile" ]; then
    echo -e "${GREEN}✅ .gitpod.Dockerfile exists${NC}"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}❌ .gitpod.Dockerfile not found${NC}"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi

# Test 4: Check .ona/mcp-config.json exists
echo ""
echo "Test 4: Checking .ona/mcp-config.json..."
if [ -f ".ona/mcp-config.json" ]; then
    echo -e "${GREEN}✅ .ona/mcp-config.json exists${NC}"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}❌ .ona/mcp-config.json not found${NC}"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi

# Test 5: Validate .ona/mcp-config.json syntax
echo ""
echo "Test 5: Validating .ona/mcp-config.json syntax..."
if command -v python3 &> /dev/null; then
    if python3 -c "import json; json.load(open('.ona/mcp-config.json'))" 2>/dev/null; then
        echo -e "${GREEN}✅ .ona/mcp-config.json syntax is valid${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}❌ .ona/mcp-config.json syntax error${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
else
    echo -e "${YELLOW}⚠️  Python3 not available, skipping JSON validation${NC}"
fi

# Test 6: Check required MCP servers in config
echo ""
echo "Test 6: Checking MCP server configuration..."
if [ -f ".ona/mcp-config.json" ]; then
    REQUIRED_SERVERS=("semgrep" "filesystem" "git" "context7" "notebooklm")
    MISSING_SERVERS=()

    for server in "${REQUIRED_SERVERS[@]}"; do
        if ! grep -q "\"$server\"" .ona/mcp-config.json; then
            MISSING_SERVERS+=("$server")
        fi
    done

    if [ ${#MISSING_SERVERS[@]} -eq 0 ]; then
        echo -e "${GREEN}✅ All required MCP servers configured${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}❌ Missing MCP servers: ${MISSING_SERVERS[*]}${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
fi

# Test 7: Check VS Code extension recommendation
echo ""
echo "Test 7: Checking VS Code extension configuration..."
if [ -f ".vscode/extensions.json" ]; then
    if grep -q "gitpod.gitpod-flex" .vscode/extensions.json; then
        echo -e "${GREEN}✅ Gitpod Flex extension recommended${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}❌ Gitpod Flex extension not in recommendations${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
else
    echo -e "${YELLOW}⚠️  .vscode/extensions.json not found${NC}"
fi

# Test 8: Check port configuration in .gitpod.yml
echo ""
echo "Test 8: Checking port configuration..."
if [ -f ".gitpod.yml" ]; then
    REQUIRED_PORTS=("8080" "50051" "5173" "4222")
    MISSING_PORTS=()

    for port in "${REQUIRED_PORTS[@]}"; do
        if ! grep -q "port: $port" .gitpod.yml && ! grep -q "\"$port\"" .gitpod.yml; then
            MISSING_PORTS+=("$port")
        fi
    done

    if [ ${#MISSING_PORTS[@]} -eq 0 ]; then
        echo -e "${GREEN}✅ All required ports configured${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${YELLOW}⚠️  Some ports not configured: ${MISSING_PORTS[*]}${NC}"
    fi
fi

# Test 9: Check environment variables
echo ""
echo "Test 9: Checking environment variable configuration..."
if [ -f ".gitpod.yml" ]; then
    if grep -q "TWS_MOCK" .gitpod.yml; then
        echo -e "${GREEN}✅ TWS_MOCK environment variable configured${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${YELLOW}⚠️  TWS_MOCK not configured (recommended for cloud)${NC}"
    fi
fi

# Summary
echo ""
echo "===================================="
echo "Test Summary"
echo "===================================="
echo -e "${GREEN}Tests Passed: $TESTS_PASSED${NC}"
if [ $TESTS_FAILED -gt 0 ]; then
    echo -e "${RED}Tests Failed: $TESTS_FAILED${NC}"
else
    echo -e "${GREEN}Tests Failed: $TESTS_FAILED${NC}"
fi
echo ""

# Final status
if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ All configuration tests passed!${NC}"
    echo ""
    echo "Next steps:"
    echo "1. Open project in Ona: https://gitpod.io/#https://github.com/YOUR_USERNAME/YOUR_REPO"
    echo "2. Or use Gitpod Flex extension in VS Code/Cursor"
    echo "3. Verify MCP servers connect in Ona Agent interface"
    exit 0
else
    echo -e "${RED}❌ Some tests failed. Please review the errors above.${NC}"
    exit 1
fi
