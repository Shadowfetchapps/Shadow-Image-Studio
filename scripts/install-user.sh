#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
[[ -x "$ROOT/target/release/shadow-image-studio" ]] || (cd "$ROOT" && cargo build --release)
PREFIX="${XDG_DATA_HOME:-$HOME/.local/share}"
install -D -m 0755 "$ROOT/target/release/shadow-image-studio" "$HOME/.local/bin/shadow-image-studio"
install -D -m 0644 "$ROOT/data/com.shadowfetch.ImageStudio.desktop" "$PREFIX/applications/com.shadowfetch.ImageStudio.desktop"
install -D -m 0644 "$ROOT/data/icons/hicolor/scalable/apps/shadow-image-studio.svg" \
  "$PREFIX/icons/hicolor/scalable/apps/shadow-image-studio.svg"
for size in 16 24 32 48 64 128 256 512; do
  install -D -m 0644 "$ROOT/data/icons/hicolor/${size}x${size}/apps/shadow-image-studio.png" \
    "$PREFIX/icons/hicolor/${size}x${size}/apps/shadow-image-studio.png"
done
update-desktop-database "$PREFIX/applications" || true
gtk-update-icon-cache -f -t "$PREFIX/icons/hicolor" >/dev/null 2>&1 || true
desktop-file-validate "$PREFIX/applications/com.shadowfetch.ImageStudio.desktop"
echo "Installed $HOME/.local/bin/shadow-image-studio"
