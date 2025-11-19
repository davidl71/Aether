#!/usr/bin/env bash
# Prepare release: commit, tag, and update Homebrew formulas
# Usage: ./scripts/prepare_release.sh <version> [commit_message]
# Example: ./scripts/prepare_release.sh v1.3.3

set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "Usage: $0 <version> [commit_message]" >&2
  echo "Example: $0 v1.3.3" >&2
  exit 1
fi

VERSION="$1"
COMMIT_MSG="${2:-Release ${VERSION}}"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Validate version format
if [[ ! "${VERSION}" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Error: Version must be in format vX.Y.Z (e.g., v1.3.3)" >&2
  exit 1
fi

echo "════════════════════════════════════════════════════════════"
echo "  Preparing Release: ${VERSION}"
echo "════════════════════════════════════════════════════════════"
echo ""

# Check if we're on main branch
CURRENT_BRANCH=$(git branch --show-current)
if [[ "${CURRENT_BRANCH}" != "main" ]]; then
  echo "⚠️  Warning: Not on main branch (current: ${CURRENT_BRANCH})" >&2
  read -p "Continue anyway? (y/N) " -n 1 -r
  echo
  if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    exit 1
  fi
fi

# Check for uncommitted changes
if ! git diff-index --quiet HEAD --; then
  echo "📝 Staging changes..."
  git add -A
  echo "✅ Changes staged"
  echo ""
fi

# Create commit if there are staged changes
if ! git diff --cached --quiet; then
  echo "📝 Creating commit..."
  git commit -m "${COMMIT_MSG}"
  echo "✅ Commit created"
  echo ""
fi

# Create and push tag
echo "🏷️  Creating tag ${VERSION}..."
if git rev-parse "${VERSION}" >/dev/null 2>&1; then
  echo "⚠️  Tag ${VERSION} already exists" >&2
  read -p "Delete and recreate? (y/N) " -n 1 -r
  echo
  if [[ $REPLY =~ ^[Yy]$ ]]; then
    git tag -d "${VERSION}" 2>/dev/null || true
    git push origin ":refs/tags/${VERSION}" 2>/dev/null || true
  else
    echo "Skipping tag creation"
    exit 0
  fi
fi

git tag -a "${VERSION}" -m "Release ${VERSION}

${COMMIT_MSG}

Changes:
- Enhanced IB Gateway integration with auto-reload wrapper
- Added brew services support for IB Gateway
- Improved error message formatting with [PATH] markers for Cursor AI
- Fixed configuration file path normalization issues
- Added file watching for automatic gateway restarts on config changes"
echo "✅ Tag created"
echo ""

# Push commit and tag
echo "📤 Pushing to remote..."
git push origin "${CURRENT_BRANCH}"
git push origin "${VERSION}"
echo "✅ Pushed to remote"
echo ""

# Update Homebrew formulas
echo "🍺 Updating Homebrew formulas..."
TAP_DIR="${ROOT_DIR}/../homebrew-ib-box-spread"
if [ -d "${TAP_DIR}" ]; then
  echo "Found tap directory: ${TAP_DIR}"
  
  # Update main formula
  if [ -f "${TAP_DIR}/Formula/ib-box-spread.rb" ]; then
    "${ROOT_DIR}/scripts/update_homebrew_formula.sh" \
      --version "${VERSION}" \
      --formula ib-box-spread \
      --tap-dir "${TAP_DIR}"
  fi
  
  # Update TUI formula if it exists
  if [ -f "${TAP_DIR}/Formula/ib-box-spread-tui.rb" ]; then
    "${ROOT_DIR}/scripts/update_homebrew_formula.sh" \
      --version "${VERSION}" \
      --formula ib-box-spread-tui \
      --tap-dir "${TAP_DIR}"
  fi
  
  echo ""
  echo "✅ Homebrew formulas updated"
  echo ""
  echo "Next steps:"
  echo "1. Review formula changes:"
  echo "   cd ${TAP_DIR}"
  echo "   git diff"
  echo ""
  echo "2. Test formula installation:"
  echo "   brew install --build-from-source ${TAP_DIR}/Formula/ib-box-spread.rb"
  echo ""
  echo "3. Commit and push formula updates:"
  echo "   cd ${TAP_DIR}"
  echo "   git add Formula/"
  echo "   git commit -m 'Update to ${VERSION}'"
  echo "   git push"
else
  echo "⚠️  Tap directory not found: ${TAP_DIR}" >&2
  echo "   Skipping Homebrew formula update" >&2
  echo "   You can update manually using:" >&2
  echo "   ${ROOT_DIR}/scripts/update_homebrew_formula.sh --version ${VERSION}" >&2
fi

echo ""
echo "════════════════════════════════════════════════════════════"
echo "  Release ${VERSION} prepared successfully!"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "Summary:"
echo "  ✅ Commit created and pushed"
echo "  ✅ Tag ${VERSION} created and pushed"
echo "  ✅ Homebrew formulas updated (if tap found)"
echo ""
echo "For multi-platform builds (macOS x86/arm64, Ubuntu):"
echo "  See: scripts/release_x86.sh (macOS x86_64)"
echo "  See: scripts/release_arm64.sh (macOS arm64 - if exists)"
echo "  See: .github/workflows/ci.yml (GitHub Actions for Ubuntu)"
echo ""

