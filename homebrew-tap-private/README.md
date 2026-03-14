# Homebrew Tap for Private Projects

This is a [Homebrew tap](https://docs.brew.sh/Taps) for installing **private** Synthetic Financing Platform tools via Homebrew.

**Note**: This tap contains formulas for private repositories that require SSH authentication. For public/open-source projects, see `homebrew-tap-public/`.

**Note**: Box spreads are one strategy component of this comprehensive multi-asset financing optimization platform.

## Installation

### Prerequisites

**This tap uses a private repository. You need SSH access configured:**

1. **Set up SSH keys** (if not already done):

   ```bash
   # Check if you have SSH keys
   ls -la ~/.ssh/id_*.pub

   # If not, generate one
   ssh-keygen -t ed25519 -C "your_email@example.com"

   # Add to GitHub: https://github.com/settings/keys
   cat ~/.ssh/id_ed25519.pub
   ```

2. **Test SSH access**:

   ```bash
   ssh -T git@github.com
   # Should see: "Hi davidl71! You've successfully authenticated..."
   ```

3. **Configure git to use SSH for GitHub** (required for Homebrew):

   ```bash
   # This makes git automatically rewrite HTTPS URLs to SSH
   git config --global url."git@github.com:".insteadOf "https://github.com/"

   # Verify configuration
   git config --global --get-regexp url
   ```

### Add the Tap

**For private repositories, use SSH URL:**

```bash
brew tap davidl71/ib-box-spread git@github.com:davidl71/homebrew-ib-box-spread.git
```

### Install Packages

```bash
# Install main trading binary
brew install davidl71/ib-box-spread/ib-box-spread
```

**Note**: For public projects like Exarp Oh My Zsh plugin, use the public tap:

```bash
brew tap davidl71/public-projects https://github.com/davidl71/homebrew-public-projects.git
brew install davidl71/public-projects/exarp-oh-my-zsh
```

## Packages

### synthetic-financing-platform

Main C++ trading binary with comprehensive multi-asset financing optimization. Box spreads are one strategy component.

**Dependencies**:

- CMake
- Ninja
- Boost
- Protocol Buffers
- Abseil

**Installation**:

```bash
brew install synthetic-financing-platform
```

**Usage**:

```bash
synthetic-financing-platform --config ~/.config/synthetic-financing-platform/config.json --dry-run
```

## Development

### Using This Tap Locally

If you want to test formulas before publishing:

```bash
# Add local tap
brew tap --force-local davidl71/synthetic-financing-platform /path/to/homebrew-tap

# Install from local tap
brew install --build-from-source synthetic-financing-platform
```

### Updating Formulas

**For private repositories:**

1. Update version tag in formula files (no SHA256 needed)
2. Ensure tag exists in repository: `git tag v1.0.0 && git push origin v1.0.0`
3. Test installation: `brew install --build-from-source <formula>`
4. Run audit: `brew audit --new --formula <formula>`
5. Commit and push changes

### Creating a Release

**For private repositories using GitDownloadStrategy:**

1. Create a Git tag: `git tag v1.0.0`
2. Push tag to repository: `git push origin v1.0.0`
3. Update formula with:
   - New version tag in `url` line
   - No SHA256 needed (GitDownloadStrategy doesn't use it)

Example:

```ruby
url "git@github.com:davidl71/synthetic-financing-platform.git", tag: "v1.0.1", using: :git
```

## Troubleshooting

### SSH Authentication Issues

**Problem**: `Permission denied (publickey)` when installing

**Solution**:

1. Verify SSH keys: `ls -la ~/.ssh/id_*.pub`
2. Test GitHub SSH: `ssh -T git@github.com`
3. Add SSH key to GitHub: https://github.com/settings/keys
4. Ensure tap uses SSH URL: `brew tap davidl71/synthetic-financing-platform git@github.com:davidl71/homebrew-synthetic-financing-platform.git`

### Installation Fails

- Check dependencies: `brew install cmake ninja protobuf abseil`
- Check build logs: `brew install --verbose <formula>`
- Check for issues: `brew doctor`
- Verify SSH access: `ssh -T git@github.com`
- Ensure tag exists: `git ls-remote --tags git@github.com:davidl71/synthetic-financing-platform.git`

### Binary Not Found

- Check installation: `brew list synthetic-financing-platform`
- Check PATH: `which synthetic-financing-platform`
- Reinstall: `brew reinstall synthetic-financing-platform`

### Git Tag Not Found

**Problem**: `fatal: couldn't find remote ref refs/tags/v1.0.0`

**Solution**:

1. Verify tag exists: `git ls-remote --tags git@github.com:davidl71/synthetic-financing-platform.git`
2. Create and push tag: `git tag v1.0.0 && git push origin v1.0.0`
3. Update formula with correct tag name

## See Also

- **Main Project**: https://github.com/davidl71/synthetic-financing-platform
- **Homebrew Docs**: https://docs.brew.sh/Taps
- **Formula Cookbook**: https://docs.brew.sh/Formula-Cookbook
