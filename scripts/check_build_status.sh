#!/usr/bin/env bash
# Check the status of the Debian repository build
# Usage: ./scripts/check_build_status.sh

set -e

REPO_DIR="/home/david/Projects/trading/ib_box_spread_full_universal/deb-repo"
PACKAGES_DIR="/home/david/Projects/trading/ib_box_spread_full_universal/deb-packages"
GPG_DIR="/home/david/Projects/trading/ib_box_spread_full_universal/.gnupg-repo"

echo "=== Debian Repository Build Status ==="
echo ""

# Check if build is running
echo "Build Process:"
if pgrep -f "create_deb_repo" > /dev/null; then
  echo "  ✓ Build script is running (PID: $(pgrep -f 'create_deb_repo'))"
else
  echo "  ✗ Build script is not running"
fi
echo ""

# Check GPG key
echo "GPG Key:"
if [ -f "$GPG_DIR/public.key" ] || [ -f "$REPO_DIR/public.key" ]; then
  echo "  ✓ GPG key exists"
  if [ -f "$REPO_DIR/public.key" ]; then
    echo "    Location: $REPO_DIR/public.key"
    echo "    Size: $(du -h "$REPO_DIR/public.key" | cut -f1)"
  fi
else
  echo "  ✗ GPG key not found"
fi
echo ""

# Check packages
echo "Packages Built:"
if [ -d "$REPO_DIR/pool" ]; then
  PACKAGE_COUNT=$(find "$REPO_DIR/pool" -name "*.deb" 2>/dev/null | wc -l)
  if [ "$PACKAGE_COUNT" -gt 0 ]; then
    echo "  ✓ Found $PACKAGE_COUNT .deb package(s)"
    echo ""
    echo "  Packages:"
    find "$REPO_DIR/pool" -name "*.deb" -exec basename {} \; | sed 's/^/    - /'
  else
    echo "  ✗ No .deb packages found in pool/"
  fi
else
  echo "  ✗ pool/ directory does not exist"
fi
echo ""

# Check package build directories
echo "Package Build Directories:"
if [ -d "$PACKAGES_DIR" ]; then
  DIR_COUNT=$(find "$PACKAGES_DIR" -maxdepth 1 -type d | wc -l)
  echo "  ✓ Found $((DIR_COUNT - 1)) package directory(ies)"
  ls -1 "$PACKAGES_DIR" 2>/dev/null | sed 's/^/    - /' || echo "    (empty)"
else
  echo "  ✗ Package build directory does not exist"
fi
echo ""

# Check repository metadata
echo "Repository Metadata:"
if [ -d "$REPO_DIR/dists/stable" ]; then
  echo "  ✓ Repository metadata directory exists"
  if [ -f "$REPO_DIR/dists/stable/Release" ]; then
    echo "  ✓ Release file exists"
  fi
  if [ -f "$REPO_DIR/dists/stable/Release.gpg" ] || [ -f "$REPO_DIR/dists/stable/InRelease" ]; then
    echo "  ✓ Release file is signed"
  fi
else
  echo "  ✗ Repository metadata not created yet"
fi
echo ""

# Summary
echo "=== Summary ==="
TOTAL_CHECKS=0
PASSED_CHECKS=0

[ -f "$REPO_DIR/public.key" ] && PASSED_CHECKS=$((PASSED_CHECKS + 1))
TOTAL_CHECKS=$((TOTAL_CHECKS + 1))

[ -d "$REPO_DIR/pool" ] && [ "$(find "$REPO_DIR/pool" -name "*.deb" | wc -l)" -gt 0 ] && PASSED_CHECKS=$((PASSED_CHECKS + 1))
TOTAL_CHECKS=$((TOTAL_CHECKS + 1))

[ -d "$REPO_DIR/dists/stable" ] && PASSED_CHECKS=$((PASSED_CHECKS + 1))
TOTAL_CHECKS=$((TOTAL_CHECKS + 1))

echo "Progress: $PASSED_CHECKS/$TOTAL_CHECKS checks passed"
echo ""

if [ "$PASSED_CHECKS" -eq "$TOTAL_CHECKS" ]; then
  echo "✓ Repository build is complete!"
  echo ""
  echo "Next step: Install the repository"
  echo "  sudo ./scripts/install_deb_repo.sh"
elif [ "$PASSED_CHECKS" -eq 0 ]; then
  echo "⚠ Build has not started yet"
  echo ""
  echo "Start the build with:"
  echo "  ./scripts/create_deb_repo.sh"
else
  echo "⏳ Build is in progress..."
  echo ""
  echo "Check again in a few minutes:"
  echo "  ./scripts/check_build_status.sh"
fi
