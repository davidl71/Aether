#!/bin/bash
# Security check for installed extensions
# Checks for known security issues, suspicious publishers, and reputation

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'

echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Extension Security Audit${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""

# Check if Cursor CLI is available
if command -v cursor &> /dev/null; then
  CLI_CMD="cursor"
elif command -v code &> /dev/null; then
  CLI_CMD="code"
else
  echo -e "${RED}Error: Neither Cursor nor VS Code CLI found${NC}"
  exit 1
fi

# Get all installed extensions
ALL_EXTENSIONS=$($CLI_CMD --list-extensions 2>/dev/null | sort)

# Known trusted publishers (Microsoft, major companies, well-known open source)
declare -a TRUSTED_PUBLISHERS=(
  "ms-"
  "microsoft"
  "github"
  "redhat"
  "amazonwebservices"
  "google"
  "anthropic"
  "anysphere"  # Cursor
  "rust-lang"
  "dbaeumer"  # ESLint
  "editorconfig"
  "streetsidesoftware"  # Spell checker
  "yzhang"  # Markdown All in One
  "davidanson"  # Markdown lint
  "eamodio"  # GitLens
  "sswg"  # Swift
  "timonwong"  # ShellCheck
  "ms-python"
  "ms-toolsai"
  "ms-vscode"
  "ms-edgedevtools"
  "ms-azuretools"
  "1password"  # 1Password official
  "firefox-devtools"  # Mozilla official
  "ibm"  # IBM official
  "vercel"  # Vercel official
  "mozilla"  # Mozilla
)

# Extensions with known security concerns or suspicious patterns
declare -a SUSPICIOUS_PATTERNS=(
  # Generic/obscure publishers
  "13xforever"
  "backnotprop"
  "barrettotte"
  "broadcommfd"
  "bewakerai"
  "cheshirekow"
  "christian-kohler"
  "cjl"
  "cschlosser"
  "daninemonic"
  "donjayamanne"
  "franneck94"
  "fridaplatform"
  "gfreezy"
  "guyskk"
  "halcyontechltd"
  "interactive-mcp"
  "jbenden"
  "jborean"
  "jeff-hykin"
  "kirigaya"
  "kylinideteam"
  "mattiasbaake"
  "neonxp"
  "pascalx"
  "pimzino"
  "pinage404"
  "quantconnect"
  "raz-labs"
  "sapegin"
  "serayuzgur"
  "shivamkumar"
  "syntaxsyndicate"
  "tamasfe"
  "todo2"
  "twxs"
  "usernamehw"
  "vadimcn"
  "vercel"
  "virgilsisoe"
  "vscodevim"
  "washan"
  "yeshan333"
  "yutengjing"
  "zowe"
)

# Extensions that require careful review (may have security implications)
declare -a SECURITY_SENSITIVE=(
  # Extensions that can execute code or access files
  "*.runner"
  "*.executor"
  "*.terminal"
  "*.shell"
  "*.command"
  # Extensions that access network
  "*.api"
  "*.client"
  "*.server"
  # Extensions that access system
  "*.system"
  "*.os"
  "*.process"
)

# Check extensions
# Parse optional --ide override
CLI_CMD_AUTO=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --ide)
      case "${2:-}" in
        cursor) CLI_CMD_AUTO="cursor" ;;
        code) CLI_CMD_AUTO="code" ;;
        *) echo -e "${RED}--ide must be 'cursor' or 'code'${NC}"; exit 2 ;;
      esac; shift 2 ;;
    *) echo -e "${YELLOW}Ignoring unknown arg:${NC} $1"; shift ;;
  esac
done

# Detect CLI, honor --ide if provided
if [[ -n "$CLI_CMD_AUTO" ]]; then
  CLI_CMD="$CLI_CMD_AUTO"
  IDE_NAME=$([[ "$CLI_CMD_AUTO" == "cursor" ]] && echo "Cursor" || echo "VS Code")
else
  if command -v cursor &> /dev/null; then
    IDE_NAME="Cursor"
    CLI_CMD="cursor"
  elif command -v code &> /dev/null; then
    IDE_NAME="VS Code"
    CLI_CMD="code"
  else
    echo -e "${RED}Error: Neither Cursor nor VS Code CLI found${NC}"
    exit 1
  fi
fi

echo -e "${CYAN}Analyzing ${IDE_NAME} extensions for security concerns...${NC}"
echo ""

# Check extensions by category for security-sensitive categories
echo -e "${CYAN}Checking security-sensitive categories...${NC}"
echo ""

SENSITIVE_CATEGORIES=("scm providers" "debuggers")
for category in "${SENSITIVE_CATEGORIES[@]}"; do
  category_exts=$($CLI_CMD --list-extensions --category "$category" 2>/dev/null | sort || echo "")
  if [ -n "$category_exts" ]; then
    count=$(echo "$category_exts" | grep -c . || echo "0")
    if [ "$count" -gt 0 ]; then
      echo -e "${YELLOW}${category} (${count} extension(s)) - Review permissions:${NC}"
      while IFS= read -r ext; do
        [ -z "$ext" ] && continue
        echo -e "  ${CYAN}•${NC} $ext"
      done <<< "$category_exts"
      echo ""
    fi
  fi
done

SUSPICIOUS_COUNT=0
UNKNOWN_COUNT=0
TRUSTED_COUNT=0

SUSPICIOUS_LIST=()
UNKNOWN_LIST=()
TRUSTED_LIST=()

while IFS= read -r ext; do
  [ -z "$ext" ] && continue

  publisher="${ext%%.*}"
  is_trusted=false
  is_suspicious=false

  # Check if trusted
  for trusted in "${TRUSTED_PUBLISHERS[@]}"; do
    if [[ "$publisher" == "$trusted" ]] || [[ "$publisher" == ${trusted}* ]]; then
      is_trusted=true
      TRUSTED_LIST+=("$ext")
      TRUSTED_COUNT=$((TRUSTED_COUNT + 1))
      break
    fi
  done

  # Check if suspicious (only if not trusted)
  if [ "$is_trusted" = false ]; then
    for suspicious in "${SUSPICIOUS_PATTERNS[@]}"; do
      if [[ "$publisher" == "$suspicious" ]]; then
        is_suspicious=true
        SUSPICIOUS_LIST+=("$ext")
        SUSPICIOUS_COUNT=$((SUSPICIOUS_COUNT + 1))
        break
      fi
    done

    if [ "$is_suspicious" = false ]; then
      UNKNOWN_LIST+=("$ext")
      UNKNOWN_COUNT=$((UNKNOWN_COUNT + 1))
    fi
  fi
done <<< "$ALL_EXTENSIONS"

# Display results
echo -e "${GREEN}✓ Trusted Extensions (${TRUSTED_COUNT}):${NC}"
for ext in "${TRUSTED_LIST[@]}"; do
  echo -e "  ${GREEN}•${NC} $ext"
done

echo ""
echo -e "${YELLOW}⚠ Unknown/Unverified Publishers (${UNKNOWN_COUNT}):${NC}"
for ext in "${UNKNOWN_LIST[@]}"; do
  echo -e "  ${YELLOW}•${NC} $ext"
done

echo ""
echo -e "${MAGENTA}🔍 Extensions Needing Review (${SUSPICIOUS_COUNT}):${NC}"
for ext in "${SUSPICIOUS_LIST[@]}"; do
  echo -e "  ${MAGENTA}•${NC} $ext"
done

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Security Recommendations${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""

if [ ${#SUSPICIOUS_LIST[@]} -gt 0 ] || [ ${#UNKNOWN_LIST[@]} -gt 0 ]; then
  echo -e "${YELLOW}Review the following extensions:${NC}"
  echo ""
  echo "1. Check extension publisher reputation:"
  echo "   - Visit: https://marketplace.visualstudio.com/vscode"
  echo "   - Search for each extension"
  echo "   - Check download count, ratings, reviews"
  echo "   - Verify publisher identity"
  echo ""
  echo "2. Check for security advisories:"
  echo "   - GitHub security advisories"
  echo "   - VS Code Marketplace security notices"
  echo "   - Extension changelog for security updates"
  echo ""
  echo "3. Review extension permissions:"
  echo "   - Check what permissions each extension requests"
  echo "   - Be cautious of extensions that request:"
  echo "     • File system access"
  echo "     • Network access"
  echo "     • Command execution"
  echo "     • Terminal access"
  echo ""
  echo "4. Check for recent updates:"
  echo "   - Outdated extensions may have security vulnerabilities"
  echo "   - Keep extensions updated"
fi

# Check for specific security-sensitive extensions
echo ""
echo -e "${CYAN}Checking for security-sensitive extension types...${NC}"
echo ""

SENSITIVE_FOUND=0
for ext in "${ALL_EXTENSIONS[@]}"; do
  for pattern in "${SECURITY_SENSITIVE[@]}"; do
    if [[ "$ext" == $pattern ]]; then
      echo -e "${YELLOW}⚠️  $ext${NC} (may execute code or access system)"
      SENSITIVE_FOUND=$((SENSITIVE_FOUND + 1))
    fi
  done
done

if [ $SENSITIVE_FOUND -eq 0 ]; then
  echo -e "${GREEN}✓ No obvious security-sensitive extension patterns found${NC}"
fi

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Summary${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""

TOTAL_COUNT=$(echo "$ALL_EXTENSIONS" | wc -l | tr -d ' ')

echo -e "Total Extensions: ${CYAN}${TOTAL_COUNT}${NC}"
echo -e "Trusted: ${GREEN}${TRUSTED_COUNT}${NC}"
echo -e "Unknown/Unverified: ${YELLOW}${UNKNOWN_COUNT}${NC}"
echo -e "Needs Review: ${MAGENTA}${SUSPICIOUS_COUNT}${NC}"
echo ""

if [ $SUSPICIOUS_COUNT -eq 0 ] && [ $UNKNOWN_COUNT -eq 0 ]; then
  echo -e "${GREEN}✓ All extensions are from trusted publishers${NC}"
else
  echo -e "${YELLOW}⚠ Review ${UNKNOWN_COUNT} unknown and ${SUSPICIOUS_COUNT} suspicious extensions${NC}"
  echo ""
  echo "Next steps:"
  echo "1. Research each extension's publisher"
  echo "2. Check extension reviews and ratings"
  echo "3. Verify extension permissions"
  echo "4. Keep extensions updated"
fi

echo ""
echo -e "${BLUE}Security Best Practices:${NC}"
echo "• Only install extensions from trusted publishers"
echo "• Review extension permissions before installing"
echo "• Keep extensions updated"
echo "• Remove unused extensions"
echo "• Check extension changelogs for security updates"
echo "• Be cautious of extensions with low download counts"
echo "• Verify publisher identity matches expected source"
