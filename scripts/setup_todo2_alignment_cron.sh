#!/bin/bash
# Setup cron job for automated Todo2 alignment analysis
#
# This script sets up a cron job to run the Todo2 alignment analysis
# on a scheduled basis (weekly by default).
#
# Usage:
#   ./scripts/setup_todo2_alignment_cron.sh [frequency] [day] [time]
#
# Examples:
#   ./scripts/setup_todo2_alignment_cron.sh weekly monday 02:00
#   ./scripts/setup_todo2_alignment_cron.sh monthly 1 03:00
#   ./scripts/setup_todo2_alignment_cron.sh daily 04:00

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CRON_SCRIPT="$SCRIPT_DIR/run_todo2_alignment_cron.sh"

FREQUENCY="${1:-weekly}"
DAY="${2:-monday}"
TIME="${3:-02:00}"

# Create the cron runner script
cat > "$CRON_SCRIPT" << 'EOF'
#!/bin/bash
# Cron runner for Todo2 alignment analysis
# This script is called by cron and handles logging and error reporting

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
LOG_FILE="$SCRIPT_DIR/todo2_alignment.log"
ERROR_LOG="$SCRIPT_DIR/todo2_alignment_errors.log"

cd "$PROJECT_ROOT"

# Activate Python environment if it exists
if [ -f "venv/bin/activate" ]; then
    source venv/bin/activate
elif [ -f ".venv/bin/activate" ]; then
    source .venv/bin/activate
fi

# Run the analysis script
if python3 "$SCRIPT_DIR/automate_todo2_alignment.py" >> "$LOG_FILE" 2>&1; then
    echo "$(date): Todo2 alignment analysis completed successfully" >> "$LOG_FILE"
else
    echo "$(date): ERROR - Todo2 alignment analysis failed" >> "$ERROR_LOG"
    # Optional: Send notification (uncomment and configure)
    # mail -s "Todo2 Alignment Analysis Failed" your-email@example.com < "$ERROR_LOG"
    exit 1
fi
EOF

chmod +x "$CRON_SCRIPT"

# Function to convert day name to number
day_to_num() {
    case "$1" in
        sunday) echo 0 ;;
        monday) echo 1 ;;
        tuesday) echo 2 ;;
        wednesday) echo 3 ;;
        thursday) echo 4 ;;
        friday) echo 5 ;;
        saturday) echo 6 ;;
        *) echo 1 ;;
    esac
}

# Build cron schedule
case "$FREQUENCY" in
    daily)
        HOUR=$(echo "$TIME" | cut -d: -f1)
        MINUTE=$(echo "$TIME" | cut -d: -f2)
        CRON_SCHEDULE="$MINUTE $HOUR * * *"
        ;;
    weekly)
        DAY_NUM=$(day_to_num "$DAY")
        HOUR=$(echo "$TIME" | cut -d: -f1)
        MINUTE=$(echo "$TIME" | cut -d: -f2)
        CRON_SCHEDULE="$MINUTE $HOUR * * $DAY_NUM"
        ;;
    monthly)
        DAY_NUM="$DAY"
        HOUR=$(echo "$TIME" | cut -d: -f1)
        MINUTE=$(echo "$TIME" | cut -d: -f2)
        CRON_SCHEDULE="$MINUTE $HOUR $DAY_NUM * *"
        ;;
    *)
        echo "Error: Invalid frequency. Use 'daily', 'weekly', or 'monthly'"
        exit 1
        ;;
esac

# Add to crontab
CRON_ENTRY="$CRON_SCHEDULE $CRON_SCRIPT"

# Check if entry already exists
if crontab -l 2>/dev/null | grep -q "$CRON_SCRIPT"; then
    echo "Cron entry already exists. Removing old entry..."
    crontab -l 2>/dev/null | grep -v "$CRON_SCRIPT" | crontab -
fi

# Add new entry
(crontab -l 2>/dev/null; echo "$CRON_ENTRY") | crontab -

LOG_FILE="$SCRIPT_DIR/todo2_alignment.log"
ERROR_LOG="$SCRIPT_DIR/todo2_alignment_errors.log"

echo "✅ Cron job configured successfully!"
echo ""
echo "Schedule: $FREQUENCY at $TIME"
echo "Cron entry: $CRON_ENTRY"
echo ""
echo "To view cron jobs: crontab -l"
echo "To remove this cron job: crontab -l | grep -v '$CRON_SCRIPT' | crontab -"
echo ""
echo "Logs will be written to:"
echo "  - $LOG_FILE (successful runs)"
echo "  - $ERROR_LOG (errors)"
