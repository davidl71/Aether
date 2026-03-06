#!/bin/bash
# Repository Rename Script using GitHub CLI
# Renames repository from ib_box_spread_full_universal to synthetic-financing-platform
#
# Usage: ./scripts/rename_repository.sh [--dry-run] [--yes]
#
# Options:
#   --dry-run    Show what would be done without making changes
#   --yes        Skip confirmation prompts

set -euo pipefail

OLD_NAME="ib_box_spread_full_universal"
NEW_NAME="synthetic-financing-platform"
REPO_OWNER="davidl71"

DRY_RUN=false
SKIP_CONFIRM=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --yes)
            SKIP_CONFIRM=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

echo "🔧 Repository Rename Script"
echo "=========================="
echo "Current: $REPO_OWNER/$OLD_NAME"
echo "Target:  $REPO_OWNER/$NEW_NAME"
echo ""

# Check if gh is installed
if ! command -v gh &> /dev/null; then
    echo "❌ Error: GitHub CLI (gh) is not installed"
    echo "   Install from: https://cli.github.com/"
    exit 1
fi

# Check if authenticated
if ! gh auth status &> /dev/null; then
    echo "❌ Error: Not authenticated with GitHub CLI"
    echo "   Run: gh auth login"
    exit 1
fi

# Verify we're in the right repository
CURRENT_REPO=$(gh repo view --json name,owner -q '.owner.login + "/" + .name' 2>/dev/null || echo "")
if [[ "$CURRENT_REPO" != "$REPO_OWNER/$OLD_NAME" ]]; then
    echo "⚠️  Warning: Current repository is $CURRENT_REPO"
    echo "   Expected: $REPO_OWNER/$OLD_NAME"
    if [[ "$SKIP_CONFIRM" == "false" ]]; then
        read -p "Continue anyway? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
fi

# Show what will be done
echo "📋 Steps to execute:"
echo "   1. Rename GitHub repository: $OLD_NAME → $NEW_NAME"
echo "   2. Update local git remote URL"
echo "   3. Verify remote URL update"
echo ""

if [[ "$DRY_RUN" == "true" ]]; then
    echo "🔍 DRY RUN MODE - No changes will be made"
    echo ""
    echo "Would execute:"
    echo "  gh repo rename $NEW_NAME"
    echo "  git remote set-url origin git@github.com:$REPO_OWNER/$NEW_NAME.git"
    echo "  git remote -v"
    exit 0
fi

# Confirm before proceeding
if [[ "$SKIP_CONFIRM" == "false" ]]; then
    echo "⚠️  WARNING: This will rename the GitHub repository!"
    echo "   - All clones will need to update their remote URL"
    echo "   - GitHub will redirect old URLs automatically"
    echo "   - Make sure you've coordinated with NATS work"
    echo ""
    read -p "Proceed with rename? (yes/no) " -r
    echo
    if [[ ! $REPLY =~ ^[Yy][Ee][Ss]$ ]]; then
        echo "❌ Rename cancelled"
        exit 1
    fi
fi

# Step 1: Rename repository on GitHub
echo "🔄 Step 1: Renaming GitHub repository..."
if [[ "$SKIP_CONFIRM" == "true" ]]; then
    gh repo rename "$NEW_NAME" --yes
else
    gh repo rename "$NEW_NAME"
fi

if gh repo view "$REPO_OWNER/$NEW_NAME" --json name &>/dev/null; then
    echo "✅ Repository renamed successfully on GitHub"
else
    echo "❌ Failed to rename repository"
    exit 1
fi

# Step 2: Update local git remote
echo ""
echo "🔄 Step 2: Updating local git remote URL..."
if git remote set-url origin "git@github.com:$REPO_OWNER/$NEW_NAME.git"; then
    echo "✅ Local git remote updated"
else
    echo "❌ Failed to update git remote"
    exit 1
fi

# Step 3: Verify remote URL
echo ""
echo "🔄 Step 3: Verifying remote URL..."
echo "Current remotes:"
git remote -v

# Verify the remote points to the new repository
CURRENT_REMOTE=$(git remote get-url origin)
EXPECTED_REMOTE="git@github.com:$REPO_OWNER/$NEW_NAME.git"

if [[ "$CURRENT_REMOTE" == "$EXPECTED_REMOTE" ]]; then
    echo "✅ Remote URL verified"
else
    echo "⚠️  Warning: Remote URL doesn't match expected value"
    echo "   Current:  $CURRENT_REMOTE"
    echo "   Expected: $EXPECTED_REMOTE"
fi

echo ""
echo "✅ Repository rename complete!"
echo ""
echo "📝 Next steps:"
echo "   1. Update documentation URLs (if not already done)"
echo "   2. Update Homebrew tap repository URLs"
echo "   3. Update CI/CD configurations (if any)"
echo "   4. Notify collaborators about the rename"
echo "   5. Test all links and references"
echo ""
echo "💡 Tip: Use 'git remote -v' to verify the remote URL"
