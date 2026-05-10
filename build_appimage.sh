#!/usr/bin/env bash
set -e

ARCH=$(uname -m)
RUST_TARGET="${ARCH}-unknown-linux-gnu"

rm -rf AppDir
export LD_LIBRARY_PATH="target/release:$LD_LIBRARY_PATH"
export LINUXDEPLOY_EXCLUDED_LIBRARIES="libxkbcommon.so*;libxkbcommon-x11.so*;libwayland-client.so*;libwayland-cursor.so*;libwayland-egl.so*"
NO_STRIP=1 linuxdeploy --appdir AppDir -l target/release/libSDL3.so.0 -e target/release/SISR -d sisr.desktop -i docs/SISR.svg --output appimage
mkdir -p dist/appimage
mv SISR-${ARCH}.AppImage dist/appimage/
