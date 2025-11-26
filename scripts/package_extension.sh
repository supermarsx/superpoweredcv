#!/bin/bash
set -e

SRC_DIR="extension"
DIST_DIR="dist"
BROWSERS=("chrome" "firefox" "edge" "safari")

if [ ! -d "$SRC_DIR" ]; then
    echo "Error: 'extension' directory not found. Please run this script from the project root."
    exit 1
fi

rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

for browser in "${BROWSERS[@]}"; do
    echo "Packaging for $browser..."
    BUILD_DIR="$DIST_DIR/${browser}_build"
    mkdir -p "$BUILD_DIR"

    # Copy src directory
    cp -r "$SRC_DIR/src" "$BUILD_DIR/"

    # Copy manifest
    MANIFEST_SOURCE="$SRC_DIR/manifests/$browser.json"
    if [ -f "$MANIFEST_SOURCE" ]; then
        cp "$MANIFEST_SOURCE" "$BUILD_DIR/manifest.json"
    else
        echo "Warning: Manifest for $browser not found at $MANIFEST_SOURCE"
    fi

    # Zip it
    pushd "$BUILD_DIR" > /dev/null
    zip -r "../superpoweredcv-$browser.zip" .
    popd > /dev/null

    echo "Created $DIST_DIR/superpoweredcv-$browser.zip"
done
