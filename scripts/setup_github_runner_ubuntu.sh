#!/usr/bin/env bash
# setup_github_runner_ubuntu.sh - Setup GitHub Actions runner on Ubuntu agent

set -euo pipefail

REPO_URL="${1:-}"
REGISTRATION_TOKEN="${2:-}"
RUNNER_NAME="${3:-ubuntu-agent}"

if [ -z "$REPO_URL" ] || [ -z "$REGISTRATION_TOKEN" ]; then
    echo "Usage: $0 <REPO_URL> <REGISTRATION_TOKEN> [RUNNER_NAME]"
    echo "Example: $0 https://github.com/user/repo ghs_TOKEN ubuntu-agent"
    exit 1
fi

echo "🚀 Setting up GitHub Actions runner on Ubuntu..."

# Create actions-runner directory
mkdir -p ~/actions-runner
cd ~/actions-runner

# Download latest runner (check https://github.com/actions/runner/releases for latest version)
RUNNER_VERSION="2.311.0"
RUNNER_URL="https://github.com/actions/runner/releases/download/v${RUNNER_VERSION}/actions-runner-linux-x64-${RUNNER_VERSION}.tar.gz"

echo "📥 Downloading runner version ${RUNNER_VERSION}..."
curl -o "actions-runner-linux-x64-${RUNNER_VERSION}.tar.gz" -L "$RUNNER_URL"

echo "📦 Extracting runner..."
tar xzf "actions-runner-linux-x64-${RUNNER_VERSION}.tar.gz"

echo "⚙️  Configuring runner..."
./config.sh --url "$REPO_URL" --token "$REGISTRATION_TOKEN" \
    --name "$RUNNER_NAME" --labels ubuntu,linux --work _work

echo "🔧 Installing as systemd service..."
sudo ./svc.sh install

echo "▶️  Starting service..."
sudo ./svc.sh start

echo "✅ GitHub Actions runner installed and started!"
echo ""
echo "Verification:"
echo "  - Check status: sudo ~/actions-runner/svc.sh status"
echo "  - View logs: sudo journalctl -u actions.runner.* -f"
echo "  - Check in GitHub: Repository → Settings → Actions → Runners"
echo ""
echo "Runner name: $RUNNER_NAME"
echo "Labels: ubuntu,linux"
