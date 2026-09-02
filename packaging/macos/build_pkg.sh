#!/usr/bin/env bash
# Build NeuralDisk.app and wrap it in a distributable .pkg installer.
#
# Usage: build_pkg.sh <path-to-compiled-neuraldisk-binary> <version> <output-dir>
#
# Must be run on macOS (uses sips/iconutil/pkgbuild/productbuild, all
# macOS-only tools). Produces <output-dir>/NeuralDisk-<version>.pkg.
set -euo pipefail

if [ "$#" -ne 3 ]; then
    echo "Usage: $0 <path-to-compiled-neuraldisk-binary> <version> <output-dir>" >&2
    exit 1
fi

BINARY_PATH="$1"
VERSION="$2"
OUTPUT_DIR="$3"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
WORK_DIR="$(mktemp -d)"
trap 'rm -rf "$WORK_DIR"' EXIT

if [ ! -f "$BINARY_PATH" ]; then
    echo "error: binary not found at $BINARY_PATH" >&2
    exit 1
fi

APP_DIR="$WORK_DIR/NeuralDisk.app"
CONTENTS_DIR="$APP_DIR/Contents"
MACOS_DIR="$CONTENTS_DIR/MacOS"
RESOURCES_DIR="$CONTENTS_DIR/Resources"
mkdir -p "$MACOS_DIR" "$RESOURCES_DIR"

# --- App binary -------------------------------------------------------
cp "$BINARY_PATH" "$MACOS_DIR/neuraldisk"
chmod +x "$MACOS_DIR/neuraldisk"

# --- Info.plist ---------------------------------------------------------
sed "s/@VERSION@/$VERSION/g" "$SCRIPT_DIR/Info.plist.in" > "$CONTENTS_DIR/Info.plist"

# --- App icon (.icns), generated from the existing 1024x1024 logo PNG ---
ICONSET_DIR="$WORK_DIR/AppIcon.iconset"
mkdir -p "$ICONSET_DIR"
SOURCE_PNG="$REPO_ROOT/neuraldisk/icons/neuraldisk_logo_flag.png"
if [ ! -f "$SOURCE_PNG" ]; then
    echo "error: source icon not found at $SOURCE_PNG" >&2
    exit 1
fi

# iconutil expects this exact set of sizes/names in an .iconset directory.
for size in 16 32 128 256 512; do
    sips -z "$size" "$size" "$SOURCE_PNG" --out "$ICONSET_DIR/icon_${size}x${size}.png" >/dev/null
    double=$((size * 2))
    sips -z "$double" "$double" "$SOURCE_PNG" --out "$ICONSET_DIR/icon_${size}x${size}@2x.png" >/dev/null
done
iconutil -c icns "$ICONSET_DIR" -o "$RESOURCES_DIR/AppIcon.icns"

# --- Component + product package ----------------------------------------
mkdir -p "$OUTPUT_DIR"
COMPONENT_PKG="$WORK_DIR/NeuralDisk-component.pkg"

pkgbuild \
    --identifier io.neuraldisk.NeuralDisk.pkg \
    --version "$VERSION" \
    --install-location /Applications \
    --scripts "$SCRIPT_DIR/scripts" \
    --component "$APP_DIR" \
    "$COMPONENT_PKG"

DISTRIBUTION_XML="$WORK_DIR/distribution.xml"
cat > "$DISTRIBUTION_XML" <<EOF
<?xml version="1.0" encoding="utf-8"?>
<installer-gui-script minSpecVersion="1">
    <title>NeuralDisk</title>
    <organization>io.neuraldisk</organization>
    <domains enable_localSystem="true"/>
    <options customize="never" require-scripts="false" rootVolumeOnly="true"/>
    <choices-outline>
        <line choice="default">
            <line choice="io.neuraldisk.NeuralDisk.pkg"/>
        </line>
    </choices-outline>
    <choice id="default"/>
    <choice id="io.neuraldisk.NeuralDisk.pkg" visible="false">
        <pkg-ref id="io.neuraldisk.NeuralDisk.pkg"/>
    </choice>
    <pkg-ref id="io.neuraldisk.NeuralDisk.pkg" version="$VERSION" onConclusion="none">$(basename "$COMPONENT_PKG")</pkg-ref>
</installer-gui-script>
EOF

FINAL_PKG="$OUTPUT_DIR/NeuralDisk-$VERSION.pkg"
productbuild \
    --distribution "$DISTRIBUTION_XML" \
    --package-path "$WORK_DIR" \
    "$FINAL_PKG"

echo "Built $FINAL_PKG"
