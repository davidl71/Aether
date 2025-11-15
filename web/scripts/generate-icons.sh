#!/bin/bash
# Generate PWA icons from a source image
# Usage: ./scripts/generate-icons.sh [source-image.png]

set -e

ICONS_DIR="public/icons"
SOURCE_IMAGE="${1:-scripts/icon-source.png}"

# Create icons directory
mkdir -p "$ICONS_DIR"

# Check if ImageMagick is available
if ! command -v convert &> /dev/null; then
  echo "Error: ImageMagick (convert) is required to generate icons."
  echo "Install with: brew install imagemagick (macOS) or apt-get install imagemagick (Linux)"
  exit 1
fi

# If source image doesn't exist, create a simple placeholder
if [ ! -f "$SOURCE_IMAGE" ]; then
  echo "Source image not found. Creating a simple placeholder icon..."
  convert -size 512x512 xc:'#1a1a1a' \
    -fill '#4a9eff' \
    -gravity center \
    -pointsize 200 \
    -font 'Helvetica-Bold' \
    -annotate +0+0 'BS' \
    "$SOURCE_IMAGE"
  echo "Created placeholder at $SOURCE_IMAGE"
fi

# Generate all required icon sizes
echo "Generating PWA icons from $SOURCE_IMAGE..."

for size in 72 96 128 144 152 192 384 512; do
  convert "$SOURCE_IMAGE" \
    -resize "${size}x${size}" \
    -background transparent \
    -gravity center \
    -extent "${size}x${size}" \
    "$ICONS_DIR/icon-${size}x${size}.png"
  echo "  ✓ Generated icon-${size}x${size}.png"
done

# Generate favicon
convert "$SOURCE_IMAGE" -resize 32x32 "public/favicon.ico"

echo ""
echo "✓ All icons generated successfully in $ICONS_DIR"
echo ""
echo "To use a custom icon, replace $SOURCE_IMAGE with your own 512x512 PNG image"
echo "and run this script again."
