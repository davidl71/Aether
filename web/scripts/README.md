# Web Scripts

## generate-icons.sh

Generates PWA icons from a source image.

### Usage

```bash
# Generate icons from a custom 512x512 PNG image
./scripts/generate-icons.sh path/to/your-icon.png

# Generate placeholder icons automatically
./scripts/generate-icons.sh
```

### Requirements

- ImageMagick (`convert` command)
  - macOS: `brew install imagemagick`
  - Linux: `apt-get install imagemagick` or `yum install ImageMagick`

### Output

Icons are generated in `public/icons/` with the following sizes:
- 72x72, 96x96, 128x128, 144x144, 152x152, 192x192, 384x384, 512x512

A favicon.ico is also generated in `public/`.

### Custom Icons

To use your own icon:
1. Create or obtain a 512x512 PNG image
2. Run: `./scripts/generate-icons.sh path/to/your-icon.png`
3. Icons will be automatically resized and optimized

The placeholder icon uses a dark theme (#1a1a1a background) with "BS" text in blue (#4a9eff), matching the app's trading desk aesthetic.
