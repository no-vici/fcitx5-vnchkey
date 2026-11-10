APP_NAME = vnchKey
TARGET_DIR = ../target/release
BUILD_DIR = build
APP_BUNDLE = $(BUILD_DIR)/$(APP_NAME).app
CONTENTS = $(APP_BUNDLE)/Contents
MACOS_DIR = $(CONTENTS)/MacOS
RESOURCES_DIR = $(CONTENTS)/Resources

SWIFT_SOURCES = Sources/main.swift Sources/KeyTapManager.swift
BRIDGING_HEADER = Bridging-Header.h
DMG_NAME = $(BUILD_DIR)/$(APP_NAME)-0.3.0.dmg

.PHONY: all rust app dmg run clean

all: dmg

# 1. Build Rust FFI static library (Universal Binary: cả Intel và Apple Silicon)
rust:
	cd .. && cargo build --release -p fcitx5-vnchkey-ffi

# 2. Đóng gói macOS .app Bundle
app: rust
	@echo "🔨 Đang đóng gói $(APP_NAME).app..."
	@mkdir -p $(MACOS_DIR)
	@mkdir -p $(RESOURCES_DIR)
	@cp -f Resources/* $(RESOURCES_DIR)/ 2>/dev/null || true
	
	@echo '<?xml version="1.0" encoding="UTF-8"?>\
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">\
<plist version="1.0">\
<dict>\
    <key>CFBundleExecutable</key>\
    <string>$(APP_NAME)</string>\
    <key>CFBundleIconFile</key>\
    <string>AppIcon</string>\
    <key>CFBundleIdentifier</key>\
    <string>com.vnchkey.macos</string>\
    <key>CFBundleName</key>\
    <string>$(APP_NAME)</string>\
    <key>CFBundlePackageType</key>\
    <string>APPL</string>\
    <key>CFBundleShortVersionString</key>\
    <string>0.3.0</string>\
    <key>LSMinimumSystemVersion</key>\
    <string>11.0</string>\
    <key>LSUIElement</key>\
    <true/>\
    <key>NSHighResolutionCapable</key>\
    <true/>\
</dict>\
</plist>' > $(CONTENTS)/Info.plist

	swiftc $(SWIFT_SOURCES) \
		-import-objc-header $(BRIDGING_HEADER) \
		-L $(TARGET_DIR) -lfcitx5_vnchkey_ffi \
		-o $(MACOS_DIR)/$(APP_NAME)
	
	# Ad-hoc code signing để macOS không chặn app chưa đăng ký
	codesign --force --deep --sign - $(APP_BUNDLE)
	
	@echo "✅ Đã tạo xong $(APP_BUNDLE)"
	@echo "👉 Bạn có thể chạy thử: open $(APP_BUNDLE)"
	@echo "👉 Hoặc copy vào Applications: cp -r $(APP_BUNDLE) /Applications/"

# 3. Đóng gói ra file .dmg chuẩn macOS
dmg: app
	@echo "📦 Đang tạo file $(DMG_NAME)..."
	@rm -rf $(BUILD_DIR)/dmg_root $(DMG_NAME)
	@mkdir -p $(BUILD_DIR)/dmg_root
	@cp -R $(APP_BUNDLE) $(BUILD_DIR)/dmg_root/
	@ln -s /Applications $(BUILD_DIR)/dmg_root/Applications
	@hdiutil create -volname "$(APP_NAME)" -srcfolder $(BUILD_DIR)/dmg_root -ov -format UDZO $(DMG_NAME)
	@rm -rf $(BUILD_DIR)/dmg_root
	@echo "🎉 Hoàn tất! File cài đặt DMG: $(DMG_NAME)"

run: app
	-killall $(APP_NAME) 2>/dev/null || true
	open $(APP_BUNDLE)

clean:
	rm -rf $(BUILD_DIR)

