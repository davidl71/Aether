#!/usr/bin/env bash
# Check working copy status across all agents and runners
# Reports uncommitted changes, branch status, and sync status

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$REPO_ROOT"

# Agent configurations
declare -A AGENTS=(
    ["local"]="$(pwd)"
    ["ubuntu"]="david@192.168.192.57:~/ib_box_spread_full_universal"
    ["macos"]="davidl@192.168.192.141:/Users/davidl/Projects/Trading/ib_box_spread_full_universal"
)

echo "================================================================================"
echo "WORKING COPY STATUS ACROSS AGENTS"
echo "================================================================================"
echo ""

for agent_name in "${!AGENTS[@]}"; do
    agent_path="${AGENTS[$agent_name]}"

    echo "--- $agent_name Agent ---"

    if [[ "$agent_name" == "local" ]]; then
        # Local agent
        cd "$agent_path"

        # Check git status
        if ! git rev-parse --git-dir > /dev/null 2>&1; then
            echo "  ❌ Not a git repository"
            continue
        fi

        branch=$(git branch --show-current 2>/dev/null || echo "unknown")

        # Check for uncommitted changes
        if [[ -n "$(git status --porcelain 2>/dev/null)" ]]; then
            echo "  ⚠️  Has uncommitted changes:"
            git status --short 2>/dev/null | sed 's/^/    /'
        else
            echo "  ✅ Working copy clean"
        fi

        # Check branch sync
        git fetch --quiet 2>/dev/null || true
        local_commit=$(git rev-parse HEAD 2>/dev/null || echo "unknown")
        remote_commit=$(git rev-parse origin/main 2>/dev/null || echo "unknown")

        if [[ "$local_commit" == "$remote_commit" ]]; then
            echo "  ✅ In sync with origin/main"
        else
            behind=$(git rev-list --count HEAD..origin/main 2>/dev/null || echo "0")
            ahead=$(git rev-list --count origin/main..HEAD 2>/dev/null || echo "0")
            if [[ "$behind" -gt 0 ]]; then
                echo "  ⚠️  Behind origin/main by $behind commit(s)"
            fi
            if [[ "$ahead" -gt 0 ]]; then
                echo "  ⚠️  Ahead of origin/main by $ahead commit(s)"
            fi
        fi

        echo "  Branch: $branch"
        echo "  Latest commit: $(git log -1 --oneline 2>/dev/null || echo 'unknown')"

    else
        # Remote agent
        ssh_host="${agent_path%%:*}"
        remote_path="${agent_path#*:}"

        # Check if SSH connection works
        if ! ssh -o ConnectTimeout=5 -o BatchMode=yes "$ssh_host" "exit" 2>/dev/null; then
            echo "  ❌ Cannot connect to $ssh_host"
            continue
        fi

        # Get status via SSH
        status_output=$(ssh "$ssh_host" "cd $remote_path && git status --short 2>/dev/null || echo 'ERROR'" 2>/dev/null || echo "ERROR")

        if [[ "$status_output" == "ERROR" ]]; then
            echo "  ❌ Error checking status"
            continue
        fi

        if [[ -n "$status_output" ]]; then
            echo "  ⚠️  Has uncommitted changes:"
            # shellcheck disable=SC2001
            echo "$status_output" | sed 's/^/    /'
        else
            echo "  ✅ Working copy clean"
        fi

        # Get branch and commit info
        branch=$(ssh "$ssh_host" "cd $remote_path && git branch --show-current 2>/dev/null" 2>/dev/null || echo "unknown")
        latest_commit=$(ssh "$ssh_host" "cd $remote_path && git log -1 --oneline 2>/dev/null" 2>/dev/null || echo "unknown")

        # Check sync status
        ssh "$ssh_host" "cd $remote_path && git fetch --quiet 2>/dev/null || true" 2>/dev/null || true
        behind=$(ssh "$ssh_host" "cd $remote_path && git rev-list --count HEAD..origin/main 2>/dev/null || echo '0'" 2>/dev/null || echo "0")
        ahead=$(ssh "$ssh_host" "cd $remote_path && git rev-list --count origin/main..HEAD 2>/dev/null || echo '0'" 2>/dev/null || echo "0")

        if [[ "$behind" -gt 0 ]]; then
            echo "  ⚠️  Behind origin/main by $behind commit(s)"
        elif [[ "$ahead" -gt 0 ]]; then
            echo "  ⚠️  Ahead of origin/main by $ahead commit(s)"
        else
            echo "  ✅ In sync with origin/main"
        fi

        echo "  Branch: $branch"
        echo "  Latest commit: $latest_commit"
    fi

    echo ""
done

echo "================================================================================"
echo "SUMMARY"
echo "================================================================================"
echo ""
echo "Recommendations:"
echo "  1. Commit and push any uncommitted changes"
echo "  2. Pull latest changes on agents that are behind"
echo "  3. Ensure all agents are on the same branch (main)"
echo ""
