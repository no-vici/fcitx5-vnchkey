#!/usr/bin/env bash
# ==============================================================================
# vnchKey (Tự Do Edition) - 1-Click Installer for Linux (Arch / Debian / Fedora)
# ==============================================================================
set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

WORKSPACE_DIR="$PWD"

echo -e "${BLUE}══════════════════════════════════════════════════════════════${NC}"
echo -e "${YELLOW}  vnchKey: TỰ DO EDITION - 1-CLICK INSTALLER                 ${NC}"
echo -e "${YELLOW}  Bộ Gõ Tiếng Việt 100% Safe Rust cho Fcitx5                 ${NC}"
echo -e "${BLUE}══════════════════════════════════════════════════════════════${NC}"

# Check for Rust / Cargo
if ! command -v cargo >/dev/null 2>&1; then
    echo -e "${RED}[LỖI] Không tìm thấy Cargo/Rust. Vui lòng cài đặt Rust: sudo pacman -S rust cargo${NC}"
    exit 1
fi

# Workaround for Cargo & Make when workspace path contains a colon ':'
if [[ "$WORKSPACE_DIR" == *:* ]]; then
    export CARGO_TARGET_DIR="/tmp/cargo-vnchkey-target"
    echo -e "${YELLOW}[TỰ ĐỘNG XỬ LÝ] Đường dẫn thư mục chứa dấu ':' ($WORKSPACE_DIR).${NC}"
    echo -e "${YELLOW}Đang tự động chuyển vùng build sang /tmp để bảo đảm GNU Make và Cargo biên dịch thành công 100%.${NC}\n"
fi

TARGET_DIR="${CARGO_TARGET_DIR:-target}/release"

echo -e "\n${GREEN}==> 1/4. Biên dịch mã nguồn Rust (Release Mode - LTO Fat)...${NC}"
cargo build --release

ADDON_SO=""
echo -e "\n${GREEN}==> 2/4. Biên dịch Fcitx5 Native C++ Addon Module...${NC}"
if command -v cmake >/dev/null 2>&1; then
    if [[ "$WORKSPACE_DIR" == *:* ]]; then
        SHADOW_BUILD_DIR="/tmp/vnchkey-addon-build"
        rm -rf "$SHADOW_BUILD_DIR"
        mkdir -p "$SHADOW_BUILD_DIR"
        cp -r "$WORKSPACE_DIR/fcitx5-addon" "$WORKSPACE_DIR/data" "$SHADOW_BUILD_DIR/"
        rm -rf "$SHADOW_BUILD_DIR/fcitx5-addon/build"
        mkdir -p "$SHADOW_BUILD_DIR/fcitx5-addon/build"
        cd "$SHADOW_BUILD_DIR/fcitx5-addon/build"
        if command -v ninja >/dev/null 2>&1; then
            cmake -GNinja -DCMAKE_INSTALL_PREFIX=/usr -DCMAKE_BUILD_TYPE=Release -DRUST_TARGET_DIR="$TARGET_DIR" ..
            ninja
        else
            cmake -DCMAKE_INSTALL_PREFIX=/usr -DCMAKE_BUILD_TYPE=Release -DRUST_TARGET_DIR="$TARGET_DIR" ..
            cmake --build . -j"$(nproc)"
        fi
        ADDON_SO="$SHADOW_BUILD_DIR/fcitx5-addon/build/fcitx5-vnchkey-addon.so"
        cd "$WORKSPACE_DIR"
    else
        mkdir -p fcitx5-addon/build
        cd fcitx5-addon/build
        if command -v ninja >/dev/null 2>&1; then
            cmake -GNinja -DCMAKE_INSTALL_PREFIX=/usr -DCMAKE_BUILD_TYPE=Release -DRUST_TARGET_DIR="$TARGET_DIR" ..
            ninja
        else
            cmake -DCMAKE_INSTALL_PREFIX=/usr -DCMAKE_BUILD_TYPE=Release -DRUST_TARGET_DIR="$TARGET_DIR" ..
            cmake --build . -j"$(nproc)"
        fi
        ADDON_SO="$WORKSPACE_DIR/fcitx5-addon/build/fcitx5-vnchkey-addon.so"
        cd "$WORKSPACE_DIR"
    fi
else
    echo -e "${YELLOW}[CẢNH BÁO] Không tìm thấy CMake. Bỏ qua bước biên dịch C++ addon.${NC}"
fi

echo -e "\n${GREEN}==> 3/4. Cài đặt các tệp vào hệ thống (/usr)...${NC}"
PREFIX="/usr"

# Core libraries
sudo install -Dm755 "${TARGET_DIR}/libfcitx5_vnchkey_ffi.so" "$PREFIX/lib/libfcitx5_vnchkey_ffi.so"
sudo ldconfig 2>/dev/null || true

# Fcitx5 addon library
if [ -n "$ADDON_SO" ] && [ -f "$ADDON_SO" ]; then
    sudo install -Dm755 "$ADDON_SO" "$PREFIX/lib/fcitx5/fcitx5-vnchkey-addon.so"
elif [ -f fcitx5-addon/build/fcitx5-vnchkey-addon.so ]; then
    sudo install -Dm755 fcitx5-addon/build/fcitx5-vnchkey-addon.so "$PREFIX/lib/fcitx5/fcitx5-vnchkey-addon.so"
fi

# Fcitx5 metadata configurations (Auto, Telex, VNI)
sudo install -Dm644 data/addon/vnchkey.conf "$PREFIX/share/fcitx5/addon/vnchkey.conf"
sudo install -Dm644 data/inputmethod/vnchkey.conf "$PREFIX/share/fcitx5/inputmethod/vnchkey.conf"
sudo install -Dm644 data/inputmethod/vnchkey-telex.conf "$PREFIX/share/fcitx5/inputmethod/vnchkey-telex.conf"
sudo install -Dm644 data/inputmethod/vnchkey-vni.conf "$PREFIX/share/fcitx5/inputmethod/vnchkey-vni.conf"
# Icons (Scaled to 24x24 / 22x22 matching lilypad status bar and panel standards)
sudo mkdir -p "$PREFIX/share/icons"
sudo cp -r data/icons/hicolor "$PREFIX/share/icons/"
sudo cp -r data/icons/breeze "$PREFIX/share/icons/"
sudo cp -r data/icons/breeze-dark "$PREFIX/share/icons/"
sudo gtk-update-icon-cache -q -t -f "$PREFIX/share/icons/hicolor" 2>/dev/null || true

# Native 100% Rust Settings GUI Application
sudo install -Dm755 "${TARGET_DIR}/fcitx5-vnchkey-settings" "$PREFIX/bin/fcitx5-vnchkey-settings"
sudo ln -sf fcitx5-vnchkey-settings "$PREFIX/bin/vnchkey-settings"
sudo install -Dm644 data/org.fcitx.Fcitx5.Addon.Vnchkey.Settings.desktop "$PREFIX/share/applications/org.fcitx.Fcitx5.Addon.Vnchkey.Settings.desktop"

echo -e "\n${GREEN}==> 4/4. Thiết lập quyền kernel Linux /dev/uinput...${NC}"
sudo usermod -aG input "$USER" 2>/dev/null || true
echo 'KERNEL=="uinput", MODE="0660", GROUP="input", OPTIONS+="static_node=uinput"' | sudo tee /etc/udev/rules.d/99-uinput.rules >/dev/null
echo "uinput" | sudo tee /etc/modules-load.d/uinput.conf >/dev/null
sudo modprobe uinput 2>/dev/null || true
sudo udevadm control --reload-rules 2>/dev/null || true
sudo udevadm trigger 2>/dev/null || true

# Restart Fcitx5 to pick up new input methods
echo -e "\n${GREEN}==> Khởi động lại Fcitx5 để nạp bộ gõ mới...${NC}"
fcitx5 -r -d 2>/dev/null || true

echo -e "\n${BLUE}══════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}  CÀI ĐẶT HOÀN TẤT THÀNH CÔNG!                               ${NC}"
echo -e "${YELLOW}  1. Mở 'fcitx5-configtool' để chọn 'vnchKey Telex' hoặc 'VNI'${NC}"
echo -e "${YELLOW}  2. Bấm Ctrl + Space để bật/tắt gõ tiếng Việt               ${NC}"
echo -e "${BLUE}══════════════════════════════════════════════════════════════${NC}"
