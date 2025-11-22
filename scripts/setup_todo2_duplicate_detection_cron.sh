#!/bin/bash
# Setup cron job for Todo2 duplicate task detection

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CRON_LOG="$PROJECT_ROOT/scripts/todo2_duplicate_detection_cron.log"

# Default schedule: Daily at 9 AM
SCHEDULE="${1:-daily}"
TIME="${2:-09:00}"

case "$SCHEDULE" in
  daily)
    CRON_SCHEDULE="0 ${TIME%%:*} * * *"
    ;;
  weekly)
    DAY="${3:-monday}"
    CRON_SCHEDULE="0 ${TIME%%:*} * * $(date -d "$DAY" +%u 2>/dev/null || echo 1)"
    ;;
  monthly)
    DAY="${3:-1}"
    CRON_SCHEDULE="0 ${TIME%%:*} $DAY * *"
    ;;
  *)
    echo "Invalid schedule: $SCHEDULE"
    echo "Usage: $0 [daily|weekly|monthly] [time] [day]"
    echo "Examples:"
    echo "  $0 daily 09:00"
    echo "  $0 weekly monday 09:00"
    echo "  $0 monthly 1 09:00"
    exit 1
    ;;
esac

CRON_COMMAND="cd $PROJECT_ROOT && python3 scripts/automate_todo2_duplicate_detection.py >> $CRON_LOG 2>&1"
CRON_ENTRY="$CRON_SCHEDULE $CRON_COMMAND"

# Check if cron job already exists
if crontab -l 2>/dev/null | grep -q "automate_todo2_duplicate_detection.py"; then
  echo "⚠️  Cron job already exists. Removing old entry..."
  crontab -l 2>/dev/null | grep -v "automate_todo2_duplicate_detection.py" | crontab -
fi

# Add new cron job
(crontab -l 2>/dev/null; echo "$CRON_ENTRY") | crontab -

echo "✅ Cron job installed successfully!"
echo "   Schedule: $SCHEDULE at $TIME"
echo "   Command: $CRON_COMMAND"
echo "   Log file: $CRON_LOG"
echo ""
echo "To view cron jobs: crontab -l"
echo "To remove this cron job: crontab -e (then delete the line)"
