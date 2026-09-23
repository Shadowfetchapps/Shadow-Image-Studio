#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT/Cargo.toml" | head -n1)"
ARCH="$(dpkg --print-architecture)"
cd "$ROOT"
cargo build --release
STAGE="$ROOT/dist/deb-root"
rm -rf "$STAGE"
mkdir -p "$STAGE/DEBIAN" "$STAGE/usr/bin" "$STAGE/usr/share/applications" \
  "$STAGE/usr/share/doc/shadow-image-studio" "$STAGE/usr/share/icons/hicolor/scalable/apps"
install -m 0755 target/release/shadow-image-studio "$STAGE/usr/bin/"
install -m 0644 data/com.shadowfetch.ImageStudio.desktop "$STAGE/usr/share/applications/"
install -m 0644 data/icons/hicolor/scalable/apps/shadow-image-studio.svg \
  "$STAGE/usr/share/icons/hicolor/scalable/apps/"
for size in 16 24 32 48 64 128 256 512; do
  install -D -m 0644 "data/icons/hicolor/${size}x${size}/apps/shadow-image-studio.png" \
    "$STAGE/usr/share/icons/hicolor/${size}x${size}/apps/shadow-image-studio.png"
done
install -m 0644 README.md LICENSE "$STAGE/usr/share/doc/shadow-image-studio/"
SIZE="$(du -sk "$STAGE" | cut -f1)"
cat > "$STAGE/DEBIAN/control" <<EOF
Package: shadow-image-studio
Version: ${VERSION}
Section: graphics
Priority: optional
Architecture: ${ARCH}
Maintainer: Shadow Image Studio contributors <209457103+ShadowfetchLinux@users.noreply.github.com>
Depends: libgtk-4-1, libadwaita-1-0
Installed-Size: ${SIZE}
Homepage: https://github.com/Shadowfetchapps/Shadow-Image-Studio
Description: Fast lightweight image editor for Linux
 Crop, resize, adjust, and export local image copies.
EOF
mkdir -p "$ROOT/dist"
dpkg-deb --build "$STAGE" "$ROOT/dist/shadow-image-studio_${VERSION}_${ARCH}.deb"
echo "Wrote dist/shadow-image-studio_${VERSION}_${ARCH}.deb"
