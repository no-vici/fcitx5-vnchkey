# Maintainer: no-vici <no-vici@users.noreply.github.com>
pkgname=fcitx5-vnchkey-git
pkgver=0.2.0.r1
pkgrel=1
pkgdesc="High-performance memory-safe Vietnamese input engine in 100% Rust for Fcitx5 (vnchKey Tự Do Edition)"
arch=('x86_64')
url="https://github.com/no-vici/fcitx5-vnchkey"
license=('GPL-3.0-or-later')
depends=('fcitx5')
makedepends=('git' 'cargo' 'rust' 'cmake' 'extra-cmake-modules' 'gcc')
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
  cargo build --release --locked

  # 2. Biên dịch Fcitx5 C++ Addon Shim
  if [ -d "fcitx5-addon" ]; then
    mkdir -p fcitx5-addon/build
    cd fcitx5-addon/build
    cmake -DCMAKE_INSTALL_PREFIX=/usr -DCMAKE_BUILD_TYPE=Release ..
    make -j"$(nproc)"
    cd ../..
  fi
}

package() {
  if [ -d "$srcdir/fcitx5-vnchkey" ]; then
    cd "$srcdir/fcitx5-vnchkey"
  fi

  # Cài đặt file thực thi daemon và thư viện Rust FFI
  install -Dm755 target/release/libfcitx5_vnchkey_ffi.so "$pkgdir/usr/lib/libfcitx5_vnchkey_ffi.so"
  install -Dm755 target/release/vnchkey-daemon "$pkgdir/usr/bin/vnchkey-daemon"

  # Cài đặt Fcitx5 native addon module
  if [ -f fcitx5-addon/build/fcitx5-vnchkey-addon.so ]; then
    install -Dm755 fcitx5-addon/build/fcitx5-vnchkey-addon.so "$pkgdir/usr/lib/fcitx5/fcitx5-vnchkey-addon.so"
  fi

  # Cài đặt metadata cấu hình cho Fcitx5 Settings (Hợp nhất vnchKey)
  install -Dm644 data/addon/vnchkey.conf "$pkgdir/usr/share/fcitx5/addon/vnchkey.conf"
  install -Dm644 data/inputmethod/vnchkey.conf "$pkgdir/usr/share/fcitx5/inputmethod/vnchkey.conf"
  install -Dm644 data/icons/vnchkey.png "$pkgdir/usr/share/icons/hicolor/128x128/apps/vnchkey.png"
}

