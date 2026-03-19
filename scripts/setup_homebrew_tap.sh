#!/usr/bin/env bash
# setup_homebrew_tap.sh - Set up Homebrew tap for IBKR Box Spread

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
TAP_NAME="${TAP_NAME:-davidl71/ib-box-spread}"
TAP_REPO="${TAP_REPO:-homebrew-ib-box-spread}"

usage() {
  cat <<EOF
Usage: $0 [OPTIONS]

Set up a Homebrew tap for IBKR Box Spread.

Options:
  --tap-name <name>      Tap name (default: davidl71/ib-box-spread)
  --tap-repo <repo>     Tap repository name (default: homebrew-ib-box-spread)
  --github-user <user>  GitHub username (default: davidl71)
  --local-only          Only create local tap, don't create GitHub repo
  -h, --help            Show this message

Examples:
  $0                                    # Set up tap with defaults
  $0 --tap-name myuser/my-tap           # Custom tap name
  $0 --local-only                       # Only create local tap
EOF
}

LOCAL_ONLY=false
GITHUB_USER="davidl71"

while [[ $# -gt 0 ]]; do
  case "$1" in
  --tap-name)
    TAP_NAME="$2"
    shift 2
    ;;
  --tap-repo)
    TAP_REPO="$2"
    shift 2
    ;;
  --github-user)
    GITHUB_USER="$2"
    shift 2
    ;;
  --local-only)
    LOCAL_ONLY=true
    shift
    ;;
  -h | --help)
    usage
    exit 0
    ;;
  *)
    echo "Unknown option: $1" >&2
    usage
    exit 1
    ;;
  esac
done

if ! command -v brew >/dev/null 2>&1; then
  echo "Error: Homebrew is required but not found" >&2
  exit 1
fi

echo "════════════════════════════════════════════════════════════"
echo "  Homebrew Tap Setup"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "Tap Name: ${TAP_NAME}"
echo "Repository: ${TAP_REPO}"
echo ""

# Create local tap directory
TAP_DIR="${PROJECT_ROOT}/../${TAP_REPO}"
FORMULA_DIR="${TAP_DIR}/Formula"

echo "📁 Creating tap directory structure..."
mkdir -p "${FORMULA_DIR}"

# Copy formula files
echo "📋 Copying formula files..."
if [ -d "${PROJECT_ROOT}/homebrew-tap/Formula" ]; then
  cp "${PROJECT_ROOT}/homebrew-tap/Formula"/*.rb "${FORMULA_DIR}/"
  echo "✅ Formulas copied"
else
  echo "⚠️  Formula directory not found, creating basic structure"
fi

# Copy README
if [ -f "${PROJECT_ROOT}/homebrew-tap/README.md" ]; then
  cp "${PROJECT_ROOT}/homebrew-tap/README.md" "${TAP_DIR}/"
fi

# Initialize git repository if needed
if [ ! -d "${TAP_DIR}/.git" ]; then
  echo "🔧 Initializing git repository..."
  cd "${TAP_DIR}"
  git init
  git add .
  git commit -m "Initial Homebrew tap setup"
  echo "✅ Git repository initialized"
fi

# Create GitHub repository (if not local-only)
if [ "${LOCAL_ONLY}" = false ]; then
  echo ""
  echo "🌐 GitHub Repository Setup"
  echo ""
  echo "To create the GitHub repository, run:"
  echo ""
  echo "  gh repo create ${GITHUB_USER}/${TAP_REPO} --public --source=${TAP_DIR} --remote=origin"
  echo "  cd ${TAP_DIR}"
  echo "  git push -u origin main"
  echo ""
  echo "Or create it manually at: https://github.com/new"
  echo "  Name: ${TAP_REPO}"
  echo "  Description: Homebrew tap for IBKR Box Spread"
  echo "  Public repository"
  echo ""
fi

# Add tap locally
echo "🍺 Adding tap locally..."
if brew tap "${TAP_NAME}" "${TAP_DIR}" 2>/dev/null; then
  echo "✅ Tap added successfully"
else
  echo "⚠️  Tap already exists or failed to add"
  echo "   You can add it manually with:"
  echo "   brew tap --force-local ${TAP_NAME} ${TAP_DIR}"
fi

echo ""
echo "════════════════════════════════════════════════════════════"
echo "✅ Tap setup complete!"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "Next steps:"
echo ""
echo "1. Review formulas in: ${FORMULA_DIR}/"
echo "2. Update version and SHA256 in formula files"
echo "3. Test installation:"
echo "   brew install --build-from-source ${TAP_NAME}/ib-box-spread"
echo "4. Create GitHub repository (if not done):"
echo "   gh repo create ${TAP_REPO} --public"
echo "5. Push to GitHub:"
echo "   cd ${TAP_DIR}"
echo "   git push -u origin main"
echo ""
echo "Users can then install with:"
echo "   brew tap ${TAP_NAME}"
echo "   brew install ib-box-spread"
echo ""
