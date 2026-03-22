#!/usr/bin/env bash
# Helper to build, package, and bottle an x86_64 release of ib_box_spread.
# Usage: scripts/release_x86.sh <version>

set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "Usage: $0 <version>"
  exit 1
fi

VERSION="$1"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILD_DIR="${ROOT_DIR}/build/macos-x86_64-release"
DIST_DIR="${ROOT_DIR}/dist"
PACKAGE_DIR="${DIST_DIR}/ib_box_spread-v${VERSION}-macos-x86_64"
PACKAGE_TARBALL="${DIST_DIR}/ib_box_spread-v${VERSION}-macos-x86_64.tar.gz"
BOTTLE_TARBALL="${DIST_DIR}/ib-box-spread--${VERSION}.sequoia.bottle.tar.gz"
FORMULA_PATH="${ROOT_DIR}/homebrew-tap/Formula/ib-box-spread.rb"
TAP_FORMULA_PATH="$(brew --repository)/Library/Taps/davidl71/homebrew-ib-box-spread/Formula/ib-box-spread.rb"

echo "==> Building Release (x86_64)"
cmake --preset macos-x86_64-release
cmake --build --preset macos-x86_64-release

echo "==> Packaging artifacts"
rm -rf "${PACKAGE_DIR}" "${PACKAGE_TARBALL}" "${BOTTLE_TARBALL}"
mkdir -p "${PACKAGE_DIR}"
cp "${BUILD_DIR}/bin/ib_box_spread" "${PACKAGE_DIR}/"
cp "${ROOT_DIR}/config/config.example.json" "${PACKAGE_DIR}/"
tar -czf "${PACKAGE_TARBALL}" -C "${DIST_DIR}" "ib_box_spread-v${VERSION}-macos-x86_64"

SHA256_PACKAGE="$(shasum -a 256 "${PACKAGE_TARBALL}" | awk '{print $1}')"
echo "Package SHA256: ${SHA256_PACKAGE}"

echo "==> Updating local formula url and sha256"
tmp_formula="$(mktemp)"
sed \
  -e "s|^  url \".*\"|  url \"https://github.com/davidl71/Aether/releases/download/v${VERSION}/ib_box_spread-v${VERSION}-macos-x86_64.tar.gz\"|" \
  -e "s|^  sha256 \".*\"|  sha256 \"${SHA256_PACKAGE}\"|" \
  "${FORMULA_PATH}" >"${tmp_formula}"
cp "${tmp_formula}" "${FORMULA_PATH}"
cp "${tmp_formula}" "${TAP_FORMULA_PATH}"
rm -f "${tmp_formula}"

echo "==> Installing formula (build-bottle)"
brew install --build-bottle davidl71/ib-box-spread/ib-box-spread

echo "==> Generating bottle"
(cd "${DIST_DIR}" && rm -f "ib-box-spread--${VERSION}.sequoia.bottle.tar.gz")
brew bottle davidl71/ib-box-spread/ib-box-spread
mv "ib-box-spread--${VERSION}.sequoia.bottle.tar.gz" "${DIST_DIR}/"

SHA256_BOTTLE="$(shasum -a 256 "${BOTTLE_TARBALL}" | awk '{print $1}')"
echo "Bottle SHA256: ${SHA256_BOTTLE}"

echo "==> Updating bottle block in formula"
tmp_formula="$(mktemp)"
sed \
  -e "s|^    root_url \".*\"|    root_url \"https://github.com/davidl71/Aether/releases/download/v${VERSION}\"|" \
  -e "s|^    sha256 cellar: :any, sequoia: \".*\"|    sha256 cellar: :any, sequoia: \"${SHA256_BOTTLE}\"|" \
  "${FORMULA_PATH}" >"${tmp_formula}"
cp "${tmp_formula}" "${FORMULA_PATH}"
cp "${tmp_formula}" "${TAP_FORMULA_PATH}"
rm -f "${tmp_formula}"

echo ""
echo "Release artifacts ready:"
echo "  Package: ${PACKAGE_TARBALL}"
echo "  Bottle : ${BOTTLE_TARBALL}"
echo ""
echo "Next steps:"
echo "  1. git add ${FORMULA_PATH}"
echo "  2. git commit / push formula updates"
echo "  3. Upload artifacts via gh release upload v${VERSION} ..."
