#!/usr/bin/env bash
# ==============================================================================
# Script đóng gói .deb cho Debian / Ubuntu / Linux Mint
# Dự án: vnchKey (fcitx5-vnchkey)
# ==============================================================================
set -euo pipefail

VERSION="0.3.3"
PKGNAME="fcitx5-vnchkey"
ARCH="amd64"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILD_DIR="$ROOT_DIR/target/deb-pkg"
PKG_DIR="$BUILD_DIR/${PKGNAME}_${VERSION}_${ARCH}"
OUTPUT_DEB="$ROOT_DIR/${PKGNAME}_${VERSION}_${ARCH}.deb"

echo "==> [1/4] Chuẩn bị môi trường đóng gói .deb tại: $PKG_DIR"
rm -rf "$BUILD_DIR"
mkdir -p "$PKG_DIR/DEBIAN"
mkdir -p "$PKG_DIR/usr/lib"
mkdir -p "$PKG_DIR/usr/lib/x86_64-linux-gnu/fcitx5"
mkdir -p "$PKG_DIR/usr/bin"
mkdir -p "$PKG_DIR/usr/share/fcitx5/addon"
mkdir -p "$PKG_DIR/usr/share/fcitx5/inputmethod"
mkdir -p "$PKG_DIR/usr/share/applications"
mkdir -p "$PKG_DIR/usr/share/icons"
mkdir -p "$PKG_DIR/usr/share/nvim/runtime/lua"

echo "==> [2/4] Đảm bảo các thành phần đã được biên dịch..."
if [ ! -f "$ROOT_DIR/target/release/libfcitx5_vnchkey_ffi.so" ] || [ ! -f "$ROOT_DIR/target/release/fcitx5-vnchkey-settings" ]; then
    echo "Biên dịch Rust workspace..."
    cargo build --release --workspace
fi

if [ ! -f "$ROOT_DIR/fcitx5-addon/build/fcitx5-vnchkey-addon.so" ]; then
    echo "Biên dịch C++ Addon Shim..."
    mkdir -p "$ROOT_DIR/fcitx5-addon/build"
    cmake -S "$ROOT_DIR/fcitx5-addon" -B "$ROOT_DIR/fcitx5-addon/build" \
        -GNinja -DCMAKE_BUILD_TYPE=Release -DRUST_TARGET_DIR="$ROOT_DIR/target/release"
    ninja -C "$ROOT_DIR/fcitx5-addon/build"
fi

echo "==> [3/4] Copy các tệp phân phối vào cây thư mục Debian..."
# 1. Thư viện FFI Rust
cp "$ROOT_DIR/target/release/libfcitx5_vnchkey_ffi.so" "$PKG_DIR/usr/lib/libfcitx5_vnchkey_ffi.so"

# 2. Addon fcitx5 (hỗ trợ cả /usr/lib/fcitx5 và /usr/lib/x86_64-linux-gnu/fcitx5)
cp "$ROOT_DIR/fcitx5-addon/build/fcitx5-vnchkey-addon.so" "$PKG_DIR/usr/lib/x86_64-linux-gnu/fcitx5/"
mkdir -p "$PKG_DIR/usr/lib/fcitx5"
ln -sf /usr/lib/x86_64-linux-gnu/fcitx5/fcitx5-vnchkey-addon.so "$PKG_DIR/usr/lib/fcitx5/fcitx5-vnchkey-addon.so"

# 3. Settings App & Desktop entry
cp "$ROOT_DIR/target/release/fcitx5-vnchkey-settings" "$PKG_DIR/usr/bin/"
ln -sf fcitx5-vnchkey-settings "$PKG_DIR/usr/bin/vnchkey-settings"
cp "$ROOT_DIR/data/org.fcitx.Fcitx5.Addon.Vnchkey.Settings.desktop" "$PKG_DIR/usr/share/applications/"

# 4. Fcitx5 addon & inputmethod metadata
cp "$ROOT_DIR/data/addon/vnchkey.conf" "$PKG_DIR/usr/share/fcitx5/addon/"
cp "$ROOT_DIR/data/inputmethod/vnchkey.conf" "$PKG_DIR/usr/share/fcitx5/inputmethod/"

# 5. Icons
cp -r "$ROOT_DIR/data/icons/hicolor" "$PKG_DIR/usr/share/icons/"
cp -r "$ROOT_DIR/data/icons/breeze" "$PKG_DIR/usr/share/icons/"
cp -r "$ROOT_DIR/data/icons/breeze-dark" "$PKG_DIR/usr/share/icons/"

# 6. Neovim Lua integration
cp "$ROOT_DIR/data/neovim/lua/vnchkey.lua" "$PKG_DIR/usr/share/nvim/runtime/lua/"

# Tạo tệp DEBIAN/control
cat << CONTROL_EOF > "$PKG_DIR/DEBIAN/control"
Package: fcitx5-vnchkey
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: amd64
Depends: fcitx5, libc6 (>= 2.31), libstdc++6 (>= 10)
Recommends: fcitx5-config-qt
Suggests: neovim
Maintainer: no-vici <no-vici@users.noreply.github.com>
Homepage: https://github.com/no-vici/fcitx5-vnchkey
Description: High-performance memory-safe Vietnamese input engine for Fcitx5
 vnchKey (Tự Do Edition) is a next-generation Vietnamese input engine
 written in 100% Safe Rust for Linux Fcitx5, Wayland, and X11.
 Features Keystroke Idle Timeout, Modal Editor (Vim/Neovim) compatibility,
 and an intuitive native GUI settings control panel.
CONTROL_EOF

# Tạo script postinst để update cache
cat << 'POSTINST_EOF' > "$PKG_DIR/DEBIAN/postinst"
#!/bin/sh
set -e
if [ "$1" = "configure" ]; then
    ldconfig
    if which update-icon-caches >/dev/null 2>&1; then
        update-icon-caches /usr/share/icons/* || true
    elif which gtk-update-icon-cache >/dev/null 2>&1; then
        gtk-update-icon-cache -f -t /usr/share/icons/hicolor || true
    fi
    if which update-desktop-database >/dev/null 2>&1; then
        update-desktop-database -q /usr/share/applications || true
    fi
fi
exit 0
POSTINST_EOF
chmod 755 "$PKG_DIR/DEBIAN/postinst"

echo "==> [4/4] Đóng gói thành file .deb..."
cd "$PKG_DIR"
tar --numeric-owner --owner=0 --group=0 -czf "$BUILD_DIR/data.tar.gz" usr
tar --numeric-owner --owner=0 --group=0 -czf "$BUILD_DIR/control.tar.gz" -C DEBIAN control postinst
echo "2.0" > "$BUILD_DIR/debian-binary"

ar rcs "$OUTPUT_DEB" "$BUILD_DIR/debian-binary" "$BUILD_DIR/control.tar.gz" "$BUILD_DIR/data.tar.gz"

# Dọn dẹp thư mục tạm để tránh tồn đọng tệp build cũ
rm -rf "$BUILD_DIR"

echo ""
echo "🎉 ĐÓNG GÓI THÀNH CÔNG!"
echo "Tệp cài đặt: $OUTPUT_DEB"
echo "Kích thước: $(du -h "$OUTPUT_DEB" | cut -f1)"
