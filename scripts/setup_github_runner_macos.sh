#!/usr/bin/env bash
# setup_github_runner_macos.sh - Setup GitHub Actions runner on macOS M4 agent

set -euo pipefail

REPO_URL="${1:-}"
REGISTRATION_TOKEN="${2:-}"
RUNNER_NAME="${3:-macos-m4-agent}"

if [ -z "$REPO_URL" ] || [ -z "$REGISTRATION_TOKEN" ]; then
  echo "Usage: $0 <REPO_URL> <REGISTRATION_TOKEN> [RUNNER_NAME]"
  echo "Example: $0 https://github.com/user/repo ghs_TOKEN macos-m4-agent"
  exit 1
fi

echo "🚀 Setting up GitHub Actions runner on macOS M4..."

# Detect architecture
ARCH=$(uname -m)
if [ "$ARCH" == "arm64" ]; then
  RUNNER_ARCH="osx-arm64"
  echo "✅ Detected Apple Silicon (ARM64)"
else
  RUNNER_ARCH="osx-x64"
  echo "⚠️  Detected Intel (x64) - using x64 runner"
fi

# Create actions-runner directory
mkdir -p ~/actions-runner
cd ~/actions-runner

# Download latest runner (check https://github.com/actions/runner/releases for latest version)
RUNNER_VERSION="2.311.0"
RUNNER_URL="https://github.com/actions/runner/releases/download/v${RUNNER_VERSION}/actions-runner-${RUNNER_ARCH}-${RUNNER_VERSION}.tar.gz"

echo "📥 Downloading runner version ${RUNNER_VERSION} for ${RUNNER_ARCH}..."
curl -o "actions-runner-${RUNNER_ARCH}-${RUNNER_VERSION}.tar.gz" -L "$RUNNER_URL"

echo "📦 Extracting runner..."
tar xzf "actions-runner-${RUNNER_ARCH}-${RUNNER_VERSION}.tar.gz"

echo "⚙️  Configuring runner..."
./config.sh --url "$REPO_URL" --token "$REGISTRATION_TOKEN" \
  --name "$RUNNER_NAME" --labels macos,apple-silicon,m4 --work _work

echo "🔧 Installing as launchd service..."
./svc.sh install

echo "▶️  Starting service..."
./svc.sh start

echo "✅ GitHub Actions runner installed and started!"
echo ""
echo "Verification:"
echo "  - Check status: ~/actions-runner/svc.sh status"
echo "  - View logs: log show --predicate 'process == \"Runner.Listener\"' --last 5m"
echo "  - Check in GitHub: Repository → Settings → Actions → Runners"
echo ""
echo "Runner name: $RUNNER_NAME"
echo "Labels: macos,apple-silicon,m4"
echo "Hostname: $(hostname)"
