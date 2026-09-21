#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SVG="$ROOT/data/icons/hicolor/scalable/apps/shadow-image-studio.svg"
for size in 16 24 32 48 64 128 256 512; do
  mkdir -p "$ROOT/data/icons/hicolor/${size}x${size}/apps"
  rsvg-convert -w "$size" -h "$size" -o "$ROOT/data/icons/hicolor/${size}x${size}/apps/shadow-image-studio.png" "$SVG"
done
echo "Generated studio icons"
