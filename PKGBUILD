# Maintainer: no-vici <no-vici@users.noreply.github.com>
pkgname=fcitx5-vnchkey-git
pkgver=0.2.0.r1
pkgrel=1
pkgdesc="High-performance memory-safe Vietnamese input engine in 100% Rust for Fcitx5 (vnchKey Tự Do Edition)"
arch=('x86_64')
url="https://github.com/no-vici/fcitx5-vnchkey"
license=('GPL-3.0-or-later')
depends=('fcitx5')
makedepends=('git' 'cargo' 'rust' 'cmake' 'extra-cmake-modules' 'gcc' 'ninja')
provides=('fcitx5-vnchkey')
conflicts=('fcitx5-vnchkey')
source=("git+https://github.com/no-vici/fcitx5-vnchkey.git")
md5sums=('SKIP')

pkgver() {
  if [ -d "$srcdir/fcitx5-vnchkey" ]; then
    cd "$srcdir/fcitx5-vnchkey"
    printf "0.2.0.r%s.%s" "$(git rev-list --count HEAD 2>/dev/null || echo 1)" "$(git rev-parse --short HEAD 2>/dev/null || echo g0)"
  else
    printf "0.2.0.r1"
  fi
}

build() {
  if [ -d "$srcdir/fcitx5-vnchkey" ]; then
    cd "$srcdir/fcitx5-vnchkey"
  fi

  # 1. Biên dịch bộ máy Rust lõi
  cargo build --release

  # 2. Biên dịch Fcitx5 C++ Addon Shim
  if [ -d "fcitx5-addon" ]; then
    ROOT_DIR="$PWD"
    mkdir -p fcitx5-addon/build
    cd fcitx5-addon/build
    cmake -GNinja -DCMAKE_INSTALL_PREFIX=/usr -DCMAKE_BUILD_TYPE=Release -DRUST_TARGET_DIR="$ROOT_DIR/target/release" ..
    ninja
    cd "$ROOT_DIR"
  fi
}

package() {
  if [ -d "$srcdir/fcitx5-vnchkey" ]; then
    cd "$srcdir/fcitx5-vnchkey"
  fi

  # Cài đặt thư viện Rust FFI
  install -Dm755 target/release/libfcitx5_vnchkey_ffi.so "$pkgdir/usr/lib/libfcitx5_vnchkey_ffi.so"

  # Cài đặt Fcitx5 native addon module
  install -Dm755 fcitx5-addon/build/fcitx5-vnchkey-addon.so "$pkgdir/usr/lib/fcitx5/fcitx5-vnchkey-addon.so"

  # Cài đặt metadata cấu hình cho Fcitx5 Settings (1 gói vnchKey duy nhất)
  install -Dm644 data/addon/vnchkey.conf "$pkgdir/usr/share/fcitx5/addon/vnchkey.conf"
  install -Dm644 data/inputmethod/vnchkey.conf "$pkgdir/usr/share/fcitx5/inputmethod/vnchkey.conf"

  # Cài đặt biểu tượng hệ thống chuẩn kích thước giống lilypad (24x24, 22x22, scalable SVG, status)
  mkdir -p "$pkgdir/usr/share/icons"
  cp -r data/icons/hicolor "$pkgdir/usr/share/icons/"
  cp -r data/icons/breeze "$pkgdir/usr/share/icons/"
  cp -r data/icons/breeze-dark "$pkgdir/usr/share/icons/"

  # Cài đặt ứng dụng cấu hình Settings GUI native 100% Rust (fcitx5-vnchkey-settings & alias vnchkey-settings)
  install -Dm755 target/release/fcitx5-vnchkey-settings "$pkgdir/usr/bin/fcitx5-vnchkey-settings"
  ln -sf fcitx5-vnchkey-settings "$pkgdir/usr/bin/vnchkey-settings"
  install -Dm644 data/org.fcitx.Fcitx5.Addon.Vnchkey.Settings.desktop "$pkgdir/usr/share/applications/org.fcitx.Fcitx5.Addon.Vnchkey.Settings.desktop"
}

