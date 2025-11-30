# Homebrew Tap Setup Guide

This guide explains how to set up and use the Homebrew tap for IBKR Box Spread.

## What is a Homebrew Tap?

A Homebrew tap is a Git repository containing formula files that define how to install software via Homebrew. It allows you to distribute your software through Homebrew without submitting to the main Homebrew repository.

## Quick Start

### For Users

**Prerequisites**: SSH access to private repositories

```bash

# Add the tap (use SSH URL for private repos)

brew tap davidl71/ib-box-spread git@github.com:davidl71/homebrew-ib-box-spread.git

# Install packages

brew install davidl71/ib-box-spread/ib-box-spread        # Main trading binary
brew install davidl71/ib-box-spread/ib-box-spread-tui    # Terminal UI
```

### For Developers

```bash

# Clone the tap repository

git clone https://github.com/davidl71/homebrew-ib-box-spread.git
cd homebrew-ib-box-spread

# Test formulas locally

brew install --build-from-source Formula/ib-box-spread.rb
```

## Setting Up the Tap

### Step 1: Create Tap Repository

1. **Create GitHub repository**:
   - Name: `homebrew-ib-box-spread` (must start with `homebrew-`)
   - Description: "Homebrew tap for IBKR Box Spread"
   - **Private repository** (since main repo is private)
   - Initialize with README

2. **Clone locally** (use SSH for private repos):

   ```bash
   git clone git@github.com:davidl71/homebrew-ib-box-spread.git
   cd homebrew-ib-box-spread
   ```

### Step 2: Add Formula Files

Copy formula files from `homebrew-tap/Formula/`:

```bash

# From project root

cp homebrew-tap/Formula/*.rb /path/to/homebrew-ib-box-spread/Formula/
```

### Step 3: Update Formula Metadata

Edit each formula file and update:

1. **Version**: Update `url` with new tag (for private repos using GitDownloadStrategy)
2. **No SHA256 needed**: GitDownloadStrategy doesn't require SHA256 checksums
3. **Dependencies**: Ensure all dependencies are listed

**For private repositories**, formulas use GitDownloadStrategy:

```ruby
url "git@github.com:davidl71/synthetic-financing-platform.git", tag: "v1.0.0", using: :git
```

### Step 4: Test Formulas

```bash

# Test installation

brew install --build-from-source Formula/ib-box-spread.rb

# Run audit

brew audit --new --formula Formula/ib-box-spread.rb

# Test functionality

brew test ib-box-spread
```

### Step 5: Commit and Push

```bash
git add Formula/
git commit -m "Add ib-box-spread formulas"
git push origin main
```

## Creating Releases

### Step 1: Create Release Tag

**For private repositories using GitDownloadStrategy:**

```bash

# Create a release tag

git tag -a v1.0.0 -m "Release v1.0.0"
git push origin v1.0.0

# Verify tag exists

git ls-remote --tags git@github.com:davidl71/synthetic-financing-platform.git
```

### Step 2: Update Formulas

**No SHA256 calculation needed** - GitDownloadStrategy uses git tags directly.

Update `url` in formula files:

```ruby
url "git@github.com:davidl71/synthetic-financing-platform.git", tag: "v1.0.0", using: :git
```

**Note**: For private repos, we use GitDownloadStrategy which clones the repo and checks out the tag. No tarball or SHA256 is needed.

### Step 4: Test and Commit

```bash

# Test installation

brew install --build-from-source Formula/ib-box-spread.rb

# Commit changes

git add Formula/
git commit -m "Update to v1.0.0"
git push origin main
```

## Formula Structure

### Main Binary Formula (`ib-box-spread.rb`)

```ruby
class IbBoxSpread < Formula
  desc "Automated options arbitrage trading system"
  homepage "https://github.com/davidl71/synthetic-financing-platform"
  url "..." # Release tarball URL
  sha256 "..." # Tarball checksum

  depends_on "cmake" => :build
  depends_on "ninja" => :build
  depends_on "boost"
  # ... other dependencies

  def install
    # Build and install steps
  end

  test do
    # Test steps
  end
end
```

### TUI Formula (`ib-box-spread-tui.rb`)

```ruby
class IbBoxSpreadTui < Formula
  desc "Terminal User Interface for IBKR Box Spread"
  # ... similar structure
  depends_on "go" => :build

  def install
    # Build Go binary
  end
end
```

## Testing Formulas

### Local Testing

```bash

# Install from local tap

brew tap --force-local davidl71/ib-box-spread /path/to/homebrew-ib-box-spread

# Install formula

brew install --build-from-source ib-box-spread

# Test formula

brew test ib-box-spread
```

### Audit Checks

```bash

# Run Homebrew audit

brew audit --new --formula Formula/ib-box-spread.rb

# Fix any issues reported
```

### Manual Testing

```bash

# Install

brew install ib-box-spread

# Verify installation

which ib_box_spread
ib_box_spread --version

# Test functionality

ib_box_spread --help
```

## Common Issues

### SHA256 Mismatch

**Problem**: `sha256` in formula doesn't match tarball

**Solution**:

1. Download tarball
2. Calculate SHA256: `shasum -a 256 <file>`
3. Update formula

### Build Failures

**Problem**: Formula fails to build

**Solution**:

1. Check dependencies: `brew install <dependency>`
2. Check build logs: `brew install --verbose <formula>`
3. Verify source URL is accessible
4. Check for missing dependencies in formula

### Installation Path Issues

**Problem**: Binaries not found after installation

**Solution**:

1. Check installation: `brew list ib-box-spread`
2. Verify PATH: `echo $PATH`
3. Check binary location: `brew --prefix ib-box-spread`

## Best Practices

1. **Version Management**: Always update version and SHA256 for new releases
2. **Testing**: Test formulas before pushing to tap
3. **Documentation**: Keep README.md updated
4. **Dependencies**: List all required dependencies
5. **Tests**: Include test blocks in formulas
6. **Audit**: Run `brew audit` before committing

## Automation

### Script to Update Formula

```bash

#!/bin/bash
# scripts/update_homebrew_formula.sh

VERSION="v1.0.0"
TAP_DIR="$HOME/homebrew-ib-box-spread"

# Download and calculate SHA256

curl -L -o /tmp/release.tar.gz \
  "https://github.com/davidl71/synthetic-financing-platform/archive/refs/tags/${VERSION}.tar.gz"
SHA256=$(shasum -a 256 /tmp/release.tar.gz | awk '{print $1}')

# Update formula

sed -i '' "s|url \".*\"|url \"https://github.com/davidl71/ib_box_spread_full_universal/archive/refs/tags/${VERSION}.tar.gz\"|" \
  "${TAP_DIR}/Formula/ib-box-spread.rb"
sed -i '' "s|sha256 \".*\"|sha256 \"${SHA256}\"|" \
  "${TAP_DIR}/Formula/ib-box-spread.rb"

echo "Updated formula to ${VERSION}"
echo "SHA256: ${SHA256}"
```

## See Also

- **Homebrew Tap Documentation**: <https://docs.brew.sh/Taps>
- **Formula Cookbook**: <https://docs.brew.sh/Formula-Cookbook>
- **Tap Repository**: `homebrew-tap/` directory in project
