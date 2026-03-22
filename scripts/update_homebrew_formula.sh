#!/usr/bin/env bash
# update_homebrew_formula.sh - Update Homebrew formula with new version
# For private repositories using GitDownloadStrategy (no SHA256 needed)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

usage() {
  cat <<EOF
Usage: $0 [OPTIONS]

Update Homebrew formula with new version tag.
For private repositories using GitDownloadStrategy (no SHA256 needed).

Options:
  --version <tag>        Version tag (e.g., v1.0.0)
  --formula <name>      Formula name (default: ib-box-spread)
  --tap-dir <dir>       Tap directory (default: ../homebrew-ib-box-spread)
  --github-user <user>  GitHub username (default: davidl71)
  --repo <repo>         Repository name (default: Aether)
  -h, --help            Show this message

Examples:
  $0 --version v1.0.0
EOF
}

VERSION=""
FORMULA_NAME="ib-box-spread"
TAP_DIR="${PROJECT_ROOT}/../homebrew-ib-box-spread"
GITHUB_USER="davidl71"
REPO="Aether"

while [[ $# -gt 0 ]]; do
  case "$1" in
  --version)
    VERSION="$2"
    shift 2
    ;;
  --formula)
    FORMULA_NAME="$2"
    shift 2
    ;;
  --tap-dir)
    TAP_DIR="$2"
    shift 2
    ;;
  --github-user)
    GITHUB_USER="$2"
    shift 2
    ;;
  --repo)
    REPO="$2"
    shift 2
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

if [ -z "${VERSION}" ]; then
  echo "Error: --version is required" >&2
  usage
  exit 1
fi

FORMULA_FILE="${TAP_DIR}/Formula/${FORMULA_NAME}.rb"
if [ ! -f "${FORMULA_FILE}" ]; then
  echo "Error: Formula file not found: ${FORMULA_FILE}" >&2
  exit 1
fi

# Use HTTPS URL - git will rewrite to SSH via git config
# Users need: git config --global url."git@github.com:".insteadOf "https://github.com/"
GIT_URL="https://github.com/${GITHUB_USER}/${REPO}.git"

echo "════════════════════════════════════════════════════════════"
echo "  Updating Homebrew Formula (Private Repo)"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "Formula: ${FORMULA_NAME}"
echo "Version: ${VERSION}"
echo "Git URL: ${GIT_URL}"
echo ""

# Verify tag exists
echo "🔍 Verifying tag exists..."
if ! git ls-remote --tags "${GIT_URL}" | grep -q "refs/tags/${VERSION}$"; then
  echo "Error: Tag ${VERSION} not found in repository" >&2
  echo "Create and push the tag first:" >&2
  echo "  git tag -a ${VERSION} -m 'Release ${VERSION}'" >&2
  echo "  git push origin ${VERSION}" >&2
  exit 1
fi
echo "✅ Tag ${VERSION} found"
echo ""

# Update formula file
echo "📝 Updating formula file..."

# Update URL line with new tag (GitDownloadStrategy format)
if [[ "$OSTYPE" == "darwin"* ]]; then
  # macOS sed - match the url line (may span multiple lines with comments)
  sed -i '' "s|url \".*\"|url \"${GIT_URL}\", tag: \"${VERSION}\", using: :git|" "${FORMULA_FILE}"
  # Remove any sha256 line (not needed for GitDownloadStrategy)
  sed -i '' '/sha256/d' "${FORMULA_FILE}"
else
  # Linux sed
  sed -i "s|url \".*\"|url \"${GIT_URL}\", tag: \"${VERSION}\", using: :git|" "${FORMULA_FILE}"
  sed -i '/sha256/d' "${FORMULA_FILE}"
fi

echo "✅ Formula updated"
echo ""
echo "Updated file: ${FORMULA_FILE}"
echo ""
echo "Next steps:"
echo "1. Review the changes:"
echo "   git diff ${FORMULA_FILE}"
echo ""
echo "2. Test the formula:"
echo "   brew install --build-from-source ${FORMULA_FILE}"
echo ""
echo "3. Commit and push:"
echo "   cd ${TAP_DIR}"
echo "   git add Formula/${FORMULA_NAME}.rb"
echo "   git commit -m \"Update ${FORMULA_NAME} to ${VERSION}\""
echo "   git push"
echo ""
