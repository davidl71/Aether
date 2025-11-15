#!/bin/bash
# Check feature parity between TUI and Web App
# This script helps identify missing features by comparing implementations

set -e

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

echo "🔍 Checking Feature Parity: TUI vs Web App"
echo "=========================================="
echo ""

# Check for feature tracking document
if [ ! -f "docs/FEATURE_TRACKING.md" ]; then
  echo "❌ Feature tracking document not found: docs/FEATURE_TRACKING.md"
  exit 1
fi

echo "✅ Feature tracking document found"
echo ""

# Check TUI components
echo "📋 TUI Components:"
TUI_COMPONENTS=(
  "native/src/tui_app.cpp:RenderHeader"
  "native/src/tui_app.cpp:RenderTabs"
  "native/src/tui_app.cpp:RenderDashboard"
  "native/src/tui_app.cpp:RenderPositions"
  "native/src/tui_app.cpp:RenderOrders"
  "native/src/tui_app.cpp:RenderAlerts"
  "native/src/tui_app.cpp:RenderFooter"
)

for component in "${TUI_COMPONENTS[@]}"; do
  file="${component%%:*}"
  func="${component##*:}"
  if grep -q "$func" "$file" 2>/dev/null; then
    echo "  ✅ $func"
  else
    echo "  ❌ $func (not found in $file)"
  fi
done

echo ""
echo "📋 Web App Components:"
WEB_COMPONENTS=(
  "web/src/components/HeaderStatus.tsx"
  "web/src/components/TabNavigation.tsx"
  "web/src/components/DashboardTab.tsx"
  "web/src/components/PositionsTable.tsx"
  "web/src/components/OrdersPanel.tsx"
  "web/src/components/AlertsPanel.tsx"
  "web/src/components/ActionBar.tsx"
  "web/src/components/DetailModal.tsx"
  "web/src/components/ScenarioSummary.tsx"
  "web/src/components/BoxSpreadTable.tsx"
)

for component in "${WEB_COMPONENTS[@]}"; do
  if [ -f "$component" ]; then
    echo "  ✅ $(basename $component)"
  else
    echo "  ❌ $(basename $component) (not found)"
  fi
done

echo ""
echo "📊 Data Schema Check:"
echo "  Checking if Web App types match TUI schema..."

# Check key types
TUI_TYPES=("SymbolSnapshot" "Position" "Order" "Alert" "AccountMetrics" "Snapshot")
WEB_TYPES=("SymbolSnapshot" "PositionSnapshot" "TimelineEvent" "TimelineEvent" "AccountMetrics" "SnapshotPayload")

for i in "${!TUI_TYPES[@]}"; do
  tui_type="${TUI_TYPES[$i]}"
  web_type="${WEB_TYPES[$i]}"

  if grep -q "interface $web_type\|type $web_type" web/src/types/snapshot.ts 2>/dev/null; then
    echo "  ✅ $tui_type → $web_type (mapped)"
  else
    echo "  ⚠️  $tui_type → $web_type (check mapping)"
  fi
done

echo ""
echo "🎯 Quick Actions Check:"
if grep -q "handleBuyCombo\|onBuyCombo" web/src/App.tsx 2>/dev/null; then
  echo "  ✅ Buy Combo (B key)"
else
  echo "  ❌ Buy Combo missing"
fi

if grep -q "handleSellCombo\|onSellCombo" web/src/App.tsx 2>/dev/null; then
  echo "  ✅ Sell Combo (Shift+S)"
else
  echo "  ❌ Sell Combo missing"
fi

echo ""
echo "📝 Next Steps:"
echo "  1. Review docs/FEATURE_TRACKING.md for detailed comparison"
echo "  2. Update feature status as you implement new features"
echo "  3. Run this script regularly to catch drift"
echo ""
echo "✅ Feature parity check complete!"
