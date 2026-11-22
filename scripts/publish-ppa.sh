#!/usr/bin/env bash
# ==============================================================================
# Script đóng gói Source Package & đẩy lên Ubuntu Launchpad PPA (Bản Siêu Tốc)
# Dự án: vnchKey (fcitx5-vnchkey)
# ==============================================================================
set -euo pipefail

# --- CẤU HÌNH CỦA BẠN ---
PPA_TARGET="ppa:dmcsdonglao/vnchkey"                  # Đường dẫn PPA Launchpad của bạn
MAINTAINER_NAME="Freedom"                             # Tên khớp với khóa GPG của bạn
MAINTAINER_EMAIL="bluehatvietnam@gmail.com"           # Email khớp với khóa GPG & Launchpad
GPG_KEY_ID=""                                         # Để trống để tự động lấy từ email trên
GPG_PASSPHRASE="krusader."                            # Mật khẩu GPG
UBUNTU_SERIES="noble"                                 # noble (24.04), oracular (24.10), plucky (25.04)
PKG_REVISION="1"
# ------------------------

VERSION="0.4.0"
PKGNAME="fcitx5-vnchkey"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STAGE_DIR="/tmp/${PKGNAME}-ppa-stage"
VENDOR_CACHE="$HOME/.cache/vnchkey-ppa-vendor"
BUILDER_IMAGE="vnchkey-ppa-builder:24.04"

trap 'rm -rf "$STAGE_DIR"' EXIT

if command -v docker >/dev/null 2>&1; then
    CONTAINER_CMD="docker"
elif command -v podman >/dev/null 2>&1; then
    CONTAINER_CMD="podman"
else
    echo "❌ Lỗi: Không tìm thấy Docker hay Podman trên máy. Hãy cài đặt podman hoặc docker trước."
    exit 1
fi

if [ -z "$GPG_KEY_ID" ]; then
    GPG_KEY_ID="$(gpg --list-secret-keys --with-colons "$MAINTAINER_EMAIL" 2>/dev/null | awk -F: '/^fpr:/ {print $10; exit}')"
fi

if [ -z "$GPG_KEY_ID" ]; then
    echo "❌ Lỗi: Không tìm thấy khóa GPG bí mật cho email '$MAINTAINER_EMAIL'."
    exit 1
fi

if [ -z "${GPG_PASSPHRASE:-}" ]; then
    if [ -f "$HOME/.gnupg/ppa-pass" ]; then
        GPG_PASSPHRASE="$(tr -d '\r\n' < "$HOME/.gnupg/ppa-pass")"
    else
        read -rsp "🔑 Nhập GPG Passphrase (chỉ nhập 1 lần cho toàn bộ quá trình ký): " GPG_PASSPHRASE
        echo ""
    fi
fi

echo "==> Sử dụng Container Engine: $CONTAINER_CMD"
echo "==> Phiên bản gói: ${PKGNAME} (${VERSION}-${PKG_REVISION}~${UBUNTU_SERIES}1)"
echo "==> Khóa ký GPG: $GPG_KEY_ID"

if ! $CONTAINER_CMD image inspect "$BUILDER_IMAGE" >/dev/null 2>&1; then
    echo "==> [0/5] Khởi tạo image builder cache ($BUILDER_IMAGE) lần đầu..."
    $CONTAINER_CMD build -t "$BUILDER_IMAGE" - << 'DOCKERFILE_EOF'
FROM ubuntu:24.04
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update -qq && apt-get install -y -qq --no-install-recommends \
    devscripts debhelper dput xz-utils gnupg2 ca-certificates \
    && rm -rf /var/lib/apt/lists/*
DOCKERFILE_EOF
fi

echo "==> [1/5] Sao chép mã nguồn sang vùng tạm $STAGE_DIR..."
rm -rf "$STAGE_DIR"
mkdir -p "$STAGE_DIR/src" "$VENDOR_CACHE"
tar --exclude='.git' \
    --exclude='target' \
    --exclude='fcitx5-addon/build' \
    --exclude='macos/build' \
    --exclude='windows/target' \
    --exclude='*.deb' \
    --exclude='*.tar.zst' \
    --exclude='*.txt' \
    -cf - -C "$ROOT_DIR" . | tar -xf - -C "$STAGE_DIR/src"

cd "$STAGE_DIR/src"

echo "==> [2/5] Đồng bộ thư viện Rust offline (cargo vendor)..."
rm -rf "$VENDOR_CACHE"
mkdir -p .cargo "$VENDOR_CACHE"
cargo vendor "$VENDOR_CACHE" > .cargo/config.toml || true
sed -i "s|$VENDOR_CACHE|vendor|g" .cargo/config.toml
cp -al "$VENDOR_CACHE" ./vendor 2>/dev/null || cp -a "$VENDOR_CACHE" ./vendor

# Đảm bảo file cấu hình .cargo/config.toml luôn trỏ tới thư mục vendor (kể cả khi không có external crate nào)
cat << 'CARGO_CFG' > .cargo/config.toml
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"
CARGO_CFG

# Chuẩn hóa Cargo.lock về chuẩn v3 tương thích với Cargo 1.75 của Ubuntu Noble
python3 -c '
import re
with open("Cargo.lock", "r") as f:
    c = f.read()
c = re.sub(r"^version = 4", "version = 3", c, flags=re.M)
c = re.sub(r"\"([a-zA-Z0-9_\-]+)\s+([0-9\.\-a-zA-Z\+]+)\"", lambda m: f"\"{m.group(1)} {m.group(2)} (registry+https://github.com/rust-lang/crates.io-index)\"", c)
with open("Cargo.lock", "w") as f:
    f.write(c)
'

echo "==> [3/5] Nén đa luồng tệp nguồn gốc ${PKGNAME}_${VERSION}.orig.tar.xz..."
XZ_OPT="-T0 -3" tar --exclude='debian' -cJf "$STAGE_DIR/${PKGNAME}_${VERSION}.orig.tar.xz" .

echo "==> [4/5] Tạo cấu hình đóng gói debian/..."
mkdir -p debian/source
echo "3.0 (quilt)" > debian/source/format

cat << CONTROL_EOF > debian/control
Source: ${PKGNAME}
Section: utils
Priority: optional
Maintainer: ${MAINTAINER_NAME} <${MAINTAINER_EMAIL}>
Build-Depends: debhelper-compat (= 13),
               cargo,
               rustc,
               cmake,
               ninja-build,
               extra-cmake-modules,
               libfcitx5core-dev,
               libfcitx5utils-dev,
               libgtk-3-dev,
               libxkbcommon-dev,
               libwayland-dev,
               libegl1-mesa-dev,
               libssl-dev,
               pkg-config
Standards-Version: 4.6.2
Homepage: https://github.com/no-vici/fcitx5-vnchkey

Package: ${PKGNAME}
Architecture: any
Depends: \${shlibs:Depends}, \${misc:Depends}, fcitx5
Recommends: fcitx5-config-qt
Suggests: neovim
Description: High-performance, memory-safe Vietnamese input engine for Fcitx5
 vnchKey is a modern, lightweight Vietnamese Input Method Engine for Linux
 (Fcitx5 / Wayland / X11), powered by a 100% Safe Rust core and a native
 C++ Fcitx5 addon.
CONTROL_EOF

cat << 'RULES_EOF' > debian/rules
#!/usr/bin/make -f
export DEB_BUILD_MAINT_OPTIONS = hardening=+all
export CARGO_HOME = $(CURDIR)/debian/cargo_home

%:
	dh $@

override_dh_auto_configure:
	# Bỏ qua cmake tự động ở thư mục gốc

override_dh_auto_build:
	python3 -c 'import re; c = open("Cargo.lock").read(); c = re.sub(r"^version = 4", "version = 3", c, flags=re.M); c = re.sub(r"\"([a-zA-Z0-9_\-]+)\s+([0-9\.\-a-zA-Z\+]+)\"", lambda m: f"\"{m.group(1)} {m.group(2)} (registry+https://github.com/rust-lang/crates.io-index)\"", c); open("Cargo.lock", "w").write(c)' || true
	mkdir -p $(CURDIR)/.cargo $(CURDIR)/debian/cargo_home
	printf '[source.crates-io]\nreplace-with = "vendored-sources"\n\n[source.vendored-sources]\ndirectory = "vendor"\n' > $(CURDIR)/.cargo/config.toml
	cp $(CURDIR)/.cargo/config.toml $(CURDIR)/.cargo/config
	cp $(CURDIR)/.cargo/config.toml $(CURDIR)/debian/cargo_home/config.toml
	cp $(CURDIR)/.cargo/config.toml $(CURDIR)/debian/cargo_home/config
	cargo build --release --offline -p fcitx5-vnchkey-ffi
	cargo build --release --offline -p fcitx5-vnchkey-settings
	cmake -S fcitx5-addon -B fcitx5-addon/build -GNinja \
		-DCMAKE_INSTALL_PREFIX=/usr \
		-DCMAKE_BUILD_TYPE=Release \
		-DRUST_TARGET_DIR=$(CURDIR)/target/release
	ninja -C fcitx5-addon/build

override_dh_auto_install:
	install -Dm755 target/release/libfcitx5_vnchkey_ffi.so \
		debian/fcitx5-vnchkey/usr/lib/$(DEB_HOST_MULTIARCH)/libfcitx5_vnchkey_ffi.so
	install -Dm755 fcitx5-addon/build/fcitx5-vnchkey-addon.so \
		debian/fcitx5-vnchkey/usr/lib/$(DEB_HOST_MULTIARCH)/fcitx5/fcitx5-vnchkey-addon.so
	mkdir -p debian/fcitx5-vnchkey/usr/lib/fcitx5
	ln -sf /usr/lib/$(DEB_HOST_MULTIARCH)/fcitx5/fcitx5-vnchkey-addon.so \
		debian/fcitx5-vnchkey/usr/lib/fcitx5/fcitx5-vnchkey-addon.so
	install -Dm755 target/release/fcitx5-vnchkey-settings \
		debian/fcitx5-vnchkey/usr/bin/fcitx5-vnchkey-settings
	ln -sf fcitx5-vnchkey-settings debian/fcitx5-vnchkey/usr/bin/vnchkey-settings
	install -Dm644 data/addon/vnchkey.conf \
		debian/fcitx5-vnchkey/usr/share/fcitx5/addon/vnchkey.conf
	install -Dm644 data/inputmethod/vnchkey.conf \
		debian/fcitx5-vnchkey/usr/share/fcitx5/inputmethod/vnchkey.conf
	install -Dm644 data/org.fcitx.Fcitx5.Addon.Vnchkey.Settings.desktop \
		debian/fcitx5-vnchkey/usr/share/applications/org.fcitx.Fcitx5.Addon.Vnchkey.Settings.desktop
	install -Dm644 data/neovim/lua/vnchkey.lua \
		debian/fcitx5-vnchkey/usr/share/nvim/runtime/lua/vnchkey.lua 2>/dev/null || true
	mkdir -p debian/fcitx5-vnchkey/usr/share/icons
	cp -r data/icons/hicolor data/icons/breeze data/icons/breeze-dark \
		debian/fcitx5-vnchkey/usr/share/icons/

override_dh_auto_clean:
	rm -rf target fcitx5-addon/build
RULES_EOF
chmod 755 debian/rules

cat << 'POSTINST_EOF' > debian/postinst
#!/bin/sh
set -e
if [ "$1" = "configure" ]; then
    ldconfig
    if command -v update-icon-caches >/dev/null 2>&1; then
        update-icon-caches /usr/share/icons/* || true
    elif command -v gtk-update-icon-cache >/dev/null 2>&1; then
        gtk-update-icon-cache -f -t /usr/share/icons/hicolor || true
    fi
    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database -q /usr/share/applications || true
    fi
fi
exit 0
POSTINST_EOF
chmod 755 debian/postinst

DATE_RFC2822="$(LC_ALL=C date -R)"
cat << CHANGELOG_EOF > debian/changelog
${PKGNAME} (${VERSION}-${PKG_REVISION}~${UBUNTU_SERIES}1) ${UBUNTU_SERIES}; urgency=medium

  * Release vnchKey v${VERSION} (revision ${PKG_REVISION}) for Ubuntu (${UBUNTU_SERIES}).

 -- ${MAINTAINER_NAME} <${MAINTAINER_EMAIL}>  ${DATE_RFC2822}
CHANGELOG_EOF

echo "==> [5/5] Ký GPG và tải Source Package lên ${PPA_TARGET}..."
printf '%s\n' "$GPG_PASSPHRASE" | gpg --batch --yes --pinentry-mode loopback --passphrase-fd 0 \
    --armor --export-secret-keys "$GPG_KEY_ID" > "$STAGE_DIR/secret-key.asc"

$CONTAINER_CMD run --rm -i \
    -e GPG_PASSPHRASE="$GPG_PASSPHRASE" \
    -v "$STAGE_DIR:/work" \
    -w "/work/src" \
    "$BUILDER_IMAGE" bash -c "
        set -euo pipefail
        export USER=root
        mkdir -p /root/.gnupg
        chmod 700 /root/.gnupg
        echo 'allow-loopback-pinentry' > /root/.gnupg/gpg-agent.conf
        printf '%s\n' \"\$GPG_PASSPHRASE\" > /root/.gnupg/pass.txt
        chmod 600 /root/.gnupg/pass.txt
        gpg --batch --yes --pinentry-mode loopback --passphrase-file /root/.gnupg/pass.txt --import /work/secret-key.asc
        cat << 'WRAPPER' > /usr/local/bin/gpg-batch-sign
#!/bin/sh
exec gpg --batch --yes --pinentry-mode loopback --passphrase-file /root/.gnupg/pass.txt \"\$@\"
WRAPPER
        chmod +x /usr/local/bin/gpg-batch-sign
        debuild -S -sa -d -k${GPG_KEY_ID} -p/usr/local/bin/gpg-batch-sign
        dput ${PPA_TARGET} /work/${PKGNAME}_${VERSION}-${PKG_REVISION}~${UBUNTU_SERIES}1_source.changes
    "

echo "🎉 HOÀN TẤT ĐẨY GÓI LÊN PPA!"
