#!/bin/bash
# Script to help create a fresh, optimized NotebookLM notebook
# This script opens NotebookLM and provides all URLs needed

set -e

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  NotebookLM Optimized Notebook Creator"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "This script will help you create a fresh, optimized NotebookLM notebook"
echo "with only essential documentation files (instead of the entire GitHub repo)."
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Open NotebookLM in browser
echo "📝 Step 1: Opening NotebookLM..."
echo "   If it doesn't open automatically, go to: https://notebooklm.google.com"
echo ""
open "https://notebooklm.google.com" 2>/dev/null || xdg-open "https://notebooklm.google.com" 2>/dev/null || echo "   Please open https://notebooklm.google.com manually"

sleep 2
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Instructions"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "1. In NotebookLM, click '+ New' to create a new notebook"
echo "2. Name it: 'TWS Automated Trading - Optimized Resources'"
echo "3. You'll add sources in the next steps"
echo ""
read -p "Press Enter when you've created the new notebook..."
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Priority 1: Core Documentation Files"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Click '+ Add source' → 'Website' or 'URL', then paste these URLs one by one:"
echo ""

cat << 'EOF'
https://raw.githubusercontent.com/davidl71/ib_box_spread_full_universal/main/README.md
https://raw.githubusercontent.com/davidl71/ib_box_spread_full_universal/main/docs/ECLIENT_EWRAPPER_ARCHITECTURE.md
https://raw.githubusercontent.com/davidl71/ib_box_spread_full_universal/main/docs/TWS_INTEGRATION_STATUS.md
https://raw.githubusercontent.com/davidl71/ib_box_spread_full_universal/main/docs/API_DOCUMENTATION_INDEX.md
https://raw.githubusercontent.com/davidl71/ib_box_spread_full_universal/main/docs/TWS_API_BEST_PRACTICES.md
EOF

echo ""
read -p "Press Enter when Priority 1 files are added..."
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Priority 2: Implementation Guides"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

cat << 'EOF'
https://raw.githubusercontent.com/davidl71/ib_box_spread_full_universal/main/docs/IMPLEMENTATION_GUIDE.md
https://raw.githubusercontent.com/davidl71/ib_box_spread_full_universal/main/docs/CODEBASE_ARCHITECTURE.md
https://raw.githubusercontent.com/davidl71/ib_box_spread_full_universal/main/docs/COMMON_PATTERNS.md
EOF

echo ""
read -p "Press Enter when Priority 2 files are added..."
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Priority 3: Additional Documentation"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

cat << 'EOF'
https://raw.githubusercontent.com/davidl71/ib_box_spread_full_universal/main/docs/EWRAPPER_BEST_PRACTICES.md
https://raw.githubusercontent.com/davidl71/ib_box_spread_full_universal/main/docs/IBC_LEARNINGS.md
https://raw.githubusercontent.com/davidl71/ib_box_spread_full_universal/main/config/config.example.json
EOF

echo ""
read -p "Press Enter when Priority 3 files are added..."
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  YouTube Videos (8 videos)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Add these YouTube videos:"
echo ""

cat << 'EOF'
https://www.youtube.com/watch?v=n-9bdREECTQ
https://www.youtube.com/watch?v=5moyX0qwkCA
https://www.youtube.com/watch?v=hJ7ewxQVhJw
https://www.youtube.com/watch?v=4zpYhHn5p90
https://www.youtube.com/watch?v=rC02897uiuc
https://www.youtube.com/watch?v=ZxwdTgMY44g
https://www.youtube.com/watch?v=ICZH89GdUGQ
https://www.youtube.com/watch?v=W6OJy32sE_g
EOF

echo ""
read -p "Press Enter when YouTube videos are added..."
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  External Article"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

cat << 'EOF'
https://www.vitaltrades.com/2024/02/02/making-a-c-interactive-brokers-tws-client-with-a-custom-socket-implementation/
EOF

echo ""
read -p "Press Enter when the article is added..."
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Final Steps"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "1. Share the notebook:"
echo "   - Click '⚙️ Share' (top right)"
echo "   - Select 'Anyone with link'"
echo "   - Click 'Copy link'"
echo ""
echo "2. Copy the notebook URL and paste it here:"
echo ""
read -p "Notebook URL: " notebook_url
echo ""

if [ ! -z "$notebook_url" ]; then
    echo "✅ Notebook URL saved: $notebook_url"
    echo ""
    echo "Next steps:"
    echo "1. Tell me the notebook URL is: $notebook_url"
    echo "2. I'll add it to the library with proper metadata"
    echo "3. I'll update all documentation files"
    echo ""
    echo "You can also add it to the library yourself by saying:"
    echo "  'Add $notebook_url to library tagged \"tws-api, trading, options, documentation\"'"
else
    echo "⚠️  No URL provided. You can add it later."
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "✅ Created optimized notebook with ~20 sources:"
echo "   - 11 Documentation files"
echo "   - 8 YouTube videos"
echo "   - 1 Article"
echo ""
echo "Much cleaner than the original 50+ sources!"
echo ""
