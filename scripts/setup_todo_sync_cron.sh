#!/bin/bash
# Setup cron job for Shared TODO Table Synchronization

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CRON_SCRIPT="$SCRIPT_DIR/run_todo_sync_cron.sh"

# Create runner script
cat > "$CRON_SCRIPT" << 'EOF'
#!/bin/bash
# Cron runner for Shared TODO Table Synchronization

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
LOG_FILE="$PROJECT_ROOT/scripts/todo_sync_cron.log"
ERROR_LOG="$PROJECT_ROOT/scripts/todo_sync_cron_error.log"
PYTHON_BIN="python3"

cd "$PROJECT_ROOT"

# Log with timestamp
echo "[$(date '+%Y-%m-%d %H:%M:%S')] Starting TODO sync..." >> "$LOG_FILE"

# Run sync (non-dry-run for cron)
$PYTHON_BIN "$SCRIPT_DIR/automate_todo_sync.py" >> "$LOG_FILE" 2>> "$ERROR_LOG"

EXIT_CODE=$?

if [ $EXIT_CODE -eq 0 ]; then
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] TODO sync completed successfully" >> "$LOG_FILE"
else
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] TODO sync failed with exit code $EXIT_CODE" >> "$ERROR_LOG"
fi

exit $EXIT_CODE
EOF

chmod +x "$CRON_SCRIPT"

# Add to crontab (hourly at :00)
CRON_ENTRY="0 * * * * $CRON_SCRIPT"

# Check if already in crontab
if crontab -l 2>/dev/null | grep -q "$CRON_SCRIPT"; then
    echo "✅ TODO sync cron job already exists"
    echo "   Current entry:"
    crontab -l 2>/dev/null | grep "$CRON_SCRIPT"
else
    # Add to crontab
    (crontab -l 2>/dev/null; echo "$CRON_ENTRY") | crontab -
    echo "✅ TODO sync cron job added"
    echo "   Schedule: Every hour at :00"
    echo "   Script: $CRON_SCRIPT"
    echo "   Log: $PROJECT_ROOT/scripts/todo_sync_cron.log"
fi

echo ""
echo "📋 TODO Synchronization Cron Setup Complete"
echo ""
echo "To test manually:"
echo "  $CRON_SCRIPT"
echo ""
echo "To test with dry-run:"
echo "  cd $PROJECT_ROOT && python3 scripts/automate_todo_sync.py --dry-run"
echo ""
echo "To view logs:"
echo "  tail -f $PROJECT_ROOT/scripts/todo_sync_cron.log"
echo "  tail -f $PROJECT_ROOT/scripts/todo_sync_cron_error.log"
