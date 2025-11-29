#!/bin/bash
# Setup Daily Automation Cron Job
#
# This script sets up a cron job to run daily automation tasks
# including documentation link fixing, format validation, and TODO sync.
#
# Usage: ./scripts/setup_daily_automation_cron.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DAILY_SCRIPT="$SCRIPT_DIR/daily_automation_with_link_fixing.sh"
CRON_LOG="$PROJECT_ROOT/logs/daily_automation.log"

# Create logs directory if it doesn't exist
mkdir -p "$(dirname "$CRON_LOG")"

# Create cron entry (runs daily at 2 AM)
CRON_TIME="0 2 * * *"
CRON_ENTRY="$CRON_TIME cd $PROJECT_ROOT && $DAILY_SCRIPT >> $CRON_LOG 2>&1"

# Check if cron job already exists
if crontab -l 2>/dev/null | grep -q "daily_automation_with_link_fixing.sh"; then
    echo "⚠️  Daily automation cron job already exists"
    echo ""
    echo "Current cron entry:"
    crontab -l 2>/dev/null | grep "daily_automation_with_link_fixing.sh"
    echo ""
    read -p "Replace existing cron job? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Cancelled."
        exit 0
    fi
    # Remove existing entry
    crontab -l 2>/dev/null | grep -v "daily_automation_with_link_fixing.sh" | crontab -
fi

# Add new cron job
(crontab -l 2>/dev/null; echo "$CRON_ENTRY") | crontab -

echo "✅ Daily automation cron job installed"
echo ""
echo "Schedule: Daily at 2:00 AM"
echo "Script: $DAILY_SCRIPT"
echo "Log: $CRON_LOG"
echo ""
echo "To view cron jobs: crontab -l"
echo "To remove cron job: crontab -e (then delete the line)"
echo ""
echo "To test manually: $DAILY_SCRIPT"
