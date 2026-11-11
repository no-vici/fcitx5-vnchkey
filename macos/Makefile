APP_NAME = vnchKey
TARGET_DIR = ../target/release
BUILD_DIR = build
APP_BUNDLE = $(BUILD_DIR)/$(APP_NAME).app
CONTENTS = $(APP_BUNDLE)/Contents
MACOS_DIR = $(CONTENTS)/MacOS
RESOURCES_DIR = $(CONTENTS)/Resources

SWIFT_SOURCES = Sources/main.swift Sources/KeyTapManager.swift
BRIDGING_HEADER = Bridging-Header.h
DMG_NAME = $(BUILD_DIR)/$(APP_NAME)-0.3.2.dmg

.PHONY: all rust app dmg run clean

all: dmg

# 1. Build Rust FFI static library (Universal Binary: cả Intel x86_64 và Apple Silicon arm64)
rust:
	@echo "🔨 Đang biên dịch Rust FFI Universal Binary (x86_64 + arm64)..."
	cd .. && rustup target add aarch64-apple-darwin x86_64-apple-darwin 2>/dev/null || true
	cd .. && cargo build --release -p fcitx5-vnchkey-ffi --target aarch64-apple-darwin || true
	cd .. && cargo build --release -p fcitx5-vnchkey-ffi --target x86_64-apple-darwin || true
	@mkdir -p $(TARGET_DIR)
	@rm -f $(TARGET_DIR)/libfcitx5_vnchkey_ffi.dylib
	@if [ -f "../target/aarch64-apple-darwin/release/libfcitx5_vnchkey_ffi.a" ] && [ -f "../target/x86_64-apple-darwin/release/libfcitx5_vnchkey_ffi.a" ]; then \
		lipo -create -output $(TARGET_DIR)/libfcitx5_vnchkey_ffi.a \
			../target/aarch64-apple-darwin/release/libfcitx5_vnchkey_ffi.a \
			../target/x86_64-apple-darwin/release/libfcitx5_vnchkey_ffi.a; \
		echo "✅ Đã tạo lipo Universal static library: $(TARGET_DIR)/libfcitx5_vnchkey_ffi.a"; \
	else \
		cd .. && cargo build --release -p fcitx5-vnchkey-ffi; \
	fi

# 2. Đóng gói macOS .app Bundle (Universal Binary)
app: rust
	@echo "🔨 Đang đóng gói $(APP_NAME).app (Universal Binary)..."
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
    <string>0.3.2</string>\
    <key>LSMinimumSystemVersion</key>\
    <string>11.0</string>\
    <key>LSUIElement</key>\
    <true/>\
    <key>NSHighResolutionCapable</key>\
    <true/>\
</dict>\
</plist>' > $(CONTENTS)/Info.plist

	@if swiftc -target arm64-apple-macos11.0 -version >/dev/null 2>&1; then \
		echo "🍎 Đang biên dịch Swift Universal Binary (x86_64 + arm64)..."; \
		swiftc $(SWIFT_SOURCES) -target arm64-apple-macos11.0 \
			-import-objc-header $(BRIDGING_HEADER) \
			$(TARGET_DIR)/libfcitx5_vnchkey_ffi.a \
			-o $(MACOS_DIR)/$(APP_NAME)-arm64 || true; \
		swiftc $(SWIFT_SOURCES) -target x86_64-apple-macos11.0 \
			-import-objc-header $(BRIDGING_HEADER) \
			$(TARGET_DIR)/libfcitx5_vnchkey_ffi.a \
			-o $(MACOS_DIR)/$(APP_NAME)-x86_64 || true; \
		if [ -f "$(MACOS_DIR)/$(APP_NAME)-arm64" ] && [ -f "$(MACOS_DIR)/$(APP_NAME)-x86_64" ]; then \
			lipo -create -output $(MACOS_DIR)/$(APP_NAME) \
				$(MACOS_DIR)/$(APP_NAME)-arm64 \
				$(MACOS_DIR)/$(APP_NAME)-x86_64; \
			rm -f $(MACOS_DIR)/$(APP_NAME)-arm64 $(MACOS_DIR)/$(APP_NAME)-x86_64; \
			echo "✅ Đã tạo lipo Universal executable!"; \
		else \
			swiftc $(SWIFT_SOURCES) \
				-import-objc-header $(BRIDGING_HEADER) \
				$(TARGET_DIR)/libfcitx5_vnchkey_ffi.a \
				-o $(MACOS_DIR)/$(APP_NAME); \
		fi \
	else \
		swiftc $(SWIFT_SOURCES) \
			-import-objc-header $(BRIDGING_HEADER) \
			$(TARGET_DIR)/libfcitx5_vnchkey_ffi.a \
			-o $(MACOS_DIR)/$(APP_NAME); \
	fi
	
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

