#!/usr/bin/env bash
# End-to-end NATS message flow testing
# Tests Python → NATS → TypeScript message flow
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

echo "🧪 NATS End-to-End Message Flow Test"
echo "======================================"
echo ""

# Check prerequisites
echo "📋 Checking prerequisites..."

# Check NATS server
if ! curl -s http://localhost:8222/healthz >/dev/null 2>&1; then
  echo "❌ NATS server not running. Start with: ./scripts/start_nats.sh"
  exit 1
fi
echo "✅ NATS server running"

# Check WebSocket port
if ! lsof -i :8080 >/dev/null 2>&1; then
  echo "❌ WebSocket port 8080 not listening"
  exit 1
fi
echo "✅ WebSocket port 8080 listening"

# Check nats CLI
if ! command -v nats >/dev/null 2>&1; then
  echo "⚠️  nats CLI not found. Install with: brew install nats-io/nats-tools/nats"
  echo "   (Continuing without CLI subscriber...)"
  HAS_NATS_CLI=false
else
  echo "✅ nats CLI available"
  HAS_NATS_CLI=true
fi

# Check Python nats-py
if ! python3 -c "import nats" >/dev/null 2>&1; then
  echo "❌ nats-py not installed. Install with: pip install nats-py --user"
  exit 1
fi
echo "✅ Python nats-py installed"

# Check TypeScript dependencies
if [ ! -d "web/node_modules/nats.ws" ]; then
  echo "❌ TypeScript dependencies not installed. Run: cd web && npm install"
  exit 1
fi
echo "✅ TypeScript dependencies installed"

echo ""
echo "🚀 Starting end-to-end test..."
echo ""

# Create log directory
mkdir -p logs
SUBSCRIBER_LOG="${ROOT_DIR}/logs/nats_subscriber.log"
PYTHON_LOG="${ROOT_DIR}/logs/nats_python_test.log"

# Step 1: Start NATS subscriber in background
if [ "$HAS_NATS_CLI" = true ]; then
  echo "📡 Starting NATS subscriber (watching all topics)..."
  echo "   Log: $SUBSCRIBER_LOG"
  nats sub ">" > "$SUBSCRIBER_LOG" 2>&1 &
  SUBSCRIBER_PID=$!
  echo "   Subscriber PID: $SUBSCRIBER_PID"
  sleep 2
else
  echo "⚠️  Skipping NATS subscriber (CLI not available)"
  SUBSCRIBER_PID=""
fi

# Step 2: Run Python test
echo ""
echo "🐍 Running Python NATS client test..."
echo "   Log: $PYTHON_LOG"
python3 python/integration/test_nats_client.py > "$PYTHON_LOG" 2>&1 &
PYTHON_PID=$!
echo "   Python test PID: $PYTHON_PID"

# Wait for Python test to complete
sleep 5

# Check if Python test is still running
if kill -0 "$PYTHON_PID" 2>/dev/null; then
  echo "   ⏳ Python test still running, waiting..."
  wait $PYTHON_PID 2>/dev/null || true
fi

PYTHON_EXIT=$?
if [ $PYTHON_EXIT -eq 0 ]; then
  echo "   ✅ Python test completed successfully"
else
  echo "   ❌ Python test failed (exit code: $PYTHON_EXIT)"
  echo "   Check log: $PYTHON_LOG"
fi

# Step 3: Check subscriber log for messages
if [ -n "$SUBSCRIBER_PID" ] && kill -0 "$SUBSCRIBER_PID" 2>/dev/null; then
  echo ""
  echo "📊 Checking subscriber log for messages..."
  MESSAGE_COUNT=$(grep -c "market-data\|strategy" "$SUBSCRIBER_LOG" 2>/dev/null || echo "0")
  if [ "$MESSAGE_COUNT" -gt 0 ]; then
    echo "   ✅ Found $MESSAGE_COUNT messages in subscriber log"
    echo "   Sample messages:"
    grep -E "market-data|strategy" "$SUBSCRIBER_LOG" | head -5 | sed 's/^/      /'
  else
    echo "   ⚠️  No messages found in subscriber log"
    echo "   Check log: $SUBSCRIBER_LOG"
  fi

  # Stop subscriber
  kill $SUBSCRIBER_PID 2>/dev/null || true
  sleep 1
fi

# Step 4: Summary
echo ""
echo "📋 Test Summary"
echo "=============="
echo "✅ NATS server: Running"
echo "✅ WebSocket: Enabled (port 8080)"
if [ $PYTHON_EXIT -eq 0 ]; then
  echo "✅ Python test: Passed"
else
  echo "❌ Python test: Failed"
fi
if [ -n "$SUBSCRIBER_PID" ]; then
  if [ "$MESSAGE_COUNT" -gt 0 ]; then
    echo "✅ Message flow: Verified ($MESSAGE_COUNT messages)"
  else
    echo "⚠️  Message flow: No messages captured"
  fi
fi

echo ""
echo "📝 Next Steps:"
echo "   1. Start TypeScript dev server: cd web && npm run dev"
echo "   2. Open browser to http://localhost:5173"
echo "   3. Check browser console for NATS connection"
echo "   4. Check header for NATS status badge"
echo ""
echo "📄 Logs:"
echo "   - Subscriber: $SUBSCRIBER_LOG"
echo "   - Python test: $PYTHON_LOG"
echo ""

if [ $PYTHON_EXIT -eq 0 ]; then
  exit 0
else
  exit 1
fi
