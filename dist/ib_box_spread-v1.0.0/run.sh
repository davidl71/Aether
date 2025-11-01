#!/bin/bash
# IBKR Box Spread Generator - Launch Script

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
export DYLD_LIBRARY_PATH="$SCRIPT_DIR/bin:$DYLD_LIBRARY_PATH"

# Check if config exists
if [ ! -f "$SCRIPT_DIR/config/config.json" ]; then
    echo "Creating config.json from example..."
    cp "$SCRIPT_DIR/config/config.example.json" "$SCRIPT_DIR/config/config.json"
    echo "⚠️  Please edit config/config.json before running in production mode"
fi

# Run the application
"$SCRIPT_DIR/bin/ib_box_spread" --config "$SCRIPT_DIR/config/config.json" "$@"
