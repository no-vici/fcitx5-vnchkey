# Tài Liệu Kỹ Thuật: Port vnchKey Từ Linux (Rust Engine) Sang macOS

> **Mục tiêu**: Hướng dẫn chi tiết, từ gốc rễ lý thuyết đến từng dòng code thực chiến, giúp bất kỳ lập trình viên nào (kể cả beginner) hiểu rõ cách chuyển một bộ gõ tiếng Việt viết bằng Rust thuần trên Linux sang chạy nguyên bản (native) trên macOS (Apple Silicon M-Series và Intel) mà không cần cấu hình phức tạp.

---

## MỤC LỤC

1. [Tổng Quan Kiến Trúc & Chiến Lược Đa Nền Tảng](#1-tổng-quan-kiến-trúc--chiến-lược-đa-nền-tảng)
2. [C-ABI FFI: Cầu Nối Giữa Rust Và Thế Giới Bên Ngoài](#2-c-abi-ffi-cầu-nối-giữa-rust-và-thế-giới-bên-ngoài)
3. [macOS Keyboard Architecture: CGEventTap vs IMK](#3-macos-keyboard-architecture-cgeventtap-vs-imk)
4. [Hướng Dẫn Từng Bước Xây Dựng Dự Án](#4-hướng-dẫn-từng-bước-xây-dựng-dự-án)
   - [Bước 1: Cấu hình Rust Crate Static Library](#bước-1-cấu-hình-rust-crate-static-library)
   - [Bước 2: Viết Bridging Header (C-ABI Định Nghĩa Hàm)](#bước-2-viết-bridging-header-c-abi-định-nghĩa-hàm)
   - [Bước 3: Tầng Hook Bắt Phím `KeyTapManager.swift`](#bước-3-tầng-hook-bắt-phím-keytapmanagerswift)
   - [Bước 4: Giao Diện Menu Bar `main.swift`](#bước-4-giao-diện-menu-bar-mainswift)
   - [Bước 5: Tự Động Hóa Biên Dịch & Đóng Gói Với `Makefile`](#bước-5-tự-động-hóa-biên-dịch--đóng-gói-với-makefile)
5. [Giải Quyết Các Vấn Đề Thực Chiến Trên macOS](#5-giải-quyết-các-vấn-đề-thực-chiến-trên-macos)
   - [5.1. Chống nuốt phím khi Tap bị Timeout](#51-chống-nuốt-phím-khi-tap-bị-timeout)
   - [5.2. Đồng bộ bộ nhớ khi Click chuột hoặc đổi cửa sổ](#52-đồng-bộ-bộ-nhớ-khi-click-chuột-hoặc-đổi-cửa-sổ)
   - [5.3. Hỗ trợ phím Escape hoàn nguyên & Undo Macro](#53-hỗ-trợ-phím-escape-hoàn-nguyên--undo-macro)
   - [5.4. Quản lý bộ nhớ an toàn (Tránh Memory Leak FFI)](#54-quản-lý-bộ-nhớ-an-toàn-tránh-memory-leak-ffi)
6. [Quy Trình Build & Chạy Thử Thực Tế Trên Máy Mac](#6-quy-trình-build--chạy-thử-thực-tế-trên-máy-mac)

---

## 1. Tổng Quan Kiến Trúc & Chiến Lược Đa Nền Tảng

### 1.1. Nguyên lý phân tách Core và OS Interface

Khi viết phần mềm hệ thống (System Software) như bộ gõ (IME - Input Method Editor), sai lầm lớn nhất là **trộn lẫn logic xử lý ngôn ngữ vào API của hệ điều hành**.

Trong `vnchKey`, kiến trúc được tách bạch 100%:

```
┌─────────────────────────────────────────────────────────────┐
│                    TẦNG HỆ ĐIỀU HÀNH (OS)                    │
│   Linux: Fcitx5 C++ Addon  │  macOS: Swift Menu Bar + Hook  │
└──────────────────────────────┬──────────────────────────────┘
                               │ Gọi hàm C-ABI thô
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                TẦNG CẦU NỐI FFI (C-ABI Bridge)               │
│               crates/fcitx5-vnchkey-ffi (Rust)              │
│       - Quản lý con trỏ (Opaque Pointers)                   │
│       - Chuyển đổi kiểu dữ liệu C <-> Rust                  │
│       - Quản lý cấp phát / giải phóng bộ nhớ chuỗi UTF-8    │
└──────────────────────────────┬──────────────────────────────┘
                               │ Safe Rust API
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                 TẦNG LÕI ENGINE (Pure Rust)                  │
│                    crates/vnchkey-core                      │
│ - Máy trạng thái Telex/VNI                                  │
│ - Syllable Parser (Bóc tách âm tiết)                        │
│ - Micro-Diff (Thuật toán tính số Backspace tối thiểu)       │
│ - Kiểm tra Chánh Tả & Tự Động Phục Hồi Tiếng Anh            │
│ - Macro & Cứu Dấu / Cứu Chữ                                 │
└─────────────────────────────────────────────────────────────┘
```

Nhờ mô hình này:
- Toàn bộ thuật toán tiếng Việt nằm trong `crates/vnchkey-core` viết bằng **100% Safe Rust**.
- Khi chuyển sang macOS, ta **không đụng tới một dòng code nào** của `vnchkey-core`. Ta chỉ cần tạo một tầng giao tiếp mới trên macOS để lắng nghe sự kiện phím và gửi vào core.

---

## 2. C-ABI FFI: Cầu Nối Giữa Rust Và Thế Giới Bên Ngoài

### 2.1. FFI là gì?
**FFI (Foreign Function Interface)** là cơ chế cho phép một ngôn ngữ lập trình (như Swift, C, C++, Python) có thể gọi trực tiếp các hàm được viết bằng ngôn ngữ khác (ở đây là Rust).

C-ABI (Application Binary Interface của chuẩn C) là "ngôn ngữ chung" của tất cả các hệ điều hành. Bất kể Swift hay Rust phức tạp ra sao, chúng đều biết cách nói chuyện thông qua chuẩn C-ABI.

### 2.2. Tránh đụng độ tên với `#[no_mangle]` và `extern "C"`
Trong Rust, trình biên dịch thường đổi tên hàm (name mangling) để hỗ trợ module và generics. Muốn Swift gọi được hàm Rust theo đúng tên gốc, ta phải dùng 2 thuộc tính:
```rust
#[no_mangle]
pub unsafe extern "C" fn fcitx5_vnchkey_init() -> *mut Fcitx5VnchkeyContext {
    // ...
}
```
- `#[no_mangle]`: Giữ nguyên tên hàm `fcitx5_vnchkey_init` trong bảng symbol của file binary.
- `extern "C"`: Sử dụng quy ước truyền tham số qua thanh ghi/stack chuẩn của C.

### 2.3. Quản lý con trỏ đục (Opaque Pointer)
Swift không cần và không nên biết cấu trúc bên trong của `Engine` Rust gồm những trường dữ liệu nào. Do đó ta bọc `Engine` trong một cấu trúc và trả về con trỏ thô:
```rust
pub struct Fcitx5VnchkeyContext {
    engine: Engine,
}

#[no_mangle]
pub extern "C" fn fcitx5_vnchkey_init() -> *mut Fcitx5VnchkeyContext {
    let ctx = Box::new(Fcitx5VnchkeyContext {
        engine: Engine::new(EngineConfig::default()),
    });
    Box::into_raw(ctx) // Chuyển quyền quản lý bộ nhớ cho người gọi
}
```
Phía Swift, con trỏ này sẽ xuất hiện dưới dạng `OpaquePointer?`.

---

## 3. macOS Keyboard Architecture: CGEventTap vs IMK

Trên macOS có 2 phương án chính để làm bộ gõ:

| Tiêu chí | Input Method Kit (IMK) | CGEventTap (Event Hook) |
| :--- | :--- | :--- |
| **Bản chất** | Plugin cắm vào hệ thống bàn phím của macOS | Ứng dụng chạy nền chặn và biến đổi sự kiện phím |
| **Quyền hạn** | Không cần quyền Accessibility | **Cần quyền Trợ Năng (Accessibility)** |
| **Độ phức tạp** | Cực kỳ phức tạp (Objective-C/Swift cổ điển) | **Đơn giản, dễ bảo trì, dễ debug** |
| **Khả năng debug** | Phải kill `pboard`/`loginwindow`, reset daemon | Bật/tắt hoặc `Cmd+Q` như app bình thường |
| **Độ trễ** | Thấp | **Gần như bằng 0 (< 1ms với Rust)** |
| **Ứng dụng tiêu biểu** | Gõ Tiếng Việt mặc định của Apple | **EVKey, OpenKey, Dokey** |

👉 **Lựa chọn tối ưu**: Đối với cá nhân hoặc người dùng muốn cài đặt nhanh chóng, **CGEventTap** là lựa chọn số 1 về độ ổn định, dễ hiểu và dễ tuỳ biến.

---

## 4. Hướng Dẫn Từng Bước Xây Dựng Dự Án

### Bước 1: Cấu hình Rust Crate Static Library

Mở file `crates/fcitx5-vnchkey-ffi/Cargo.toml` và đảm bảo có khai báo `staticlib`:

```toml
[package]
name = "fcitx5-vnchkey-ffi"
version.workspace = true
edition.workspace = true

[lib]
# staticlib: sinh ra libfcitx5_vnchkey_ffi.a (nhúng tĩnh vào Swift)
# cdylib: sinh ra .dylib / .so (nếu muốn liên kết động)
crate-type = ["cdylib", "staticlib", "rlib"]

[dependencies]
vnchkey-core = { path = "../vnchkey-core" }
libc = "0.2"
```

Khi chạy lệnh `cargo build --release -p fcitx5-vnchkey-ffi`, Rust sẽ sinh ra file `target/release/libfcitx5_vnchkey_ffi.a`. File này chứa toàn bộ mã máy của Core Engine mà không cần bất kỳ dependency ngoài nào.

---

### Bước 2: Viết Bridging Header (C-ABI Định Nghĩa Hàm)

Tạo file `macos/Bridging-Header.h`. Đây là file header định nghĩa các hàm C mà Swift sẽ thấy:

```c
#ifndef VNCHKEY_BRIDGING_HEADER_H
#define VNCHKEY_BRIDGING_HEADER_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

// Khai báo con trỏ opaque của vnchKey Context
typedef struct Fcitx5VnchkeyContext Fcitx5VnchkeyContext;

// Khởi tạo và giải phóng
Fcitx5VnchkeyContext* fcitx5_vnchkey_init(void);
Fcitx5VnchkeyContext* fcitx5_vnchkey_init_with_method(uint32_t method);
void fcitx5_vnchkey_free(Fcitx5VnchkeyContext* ctx);
void fcitx5_vnchkey_free_string(char* s);

// Reset bộ nhớ đệm (khi click chuột hoặc đổi cửa sổ)
void fcitx5_vnchkey_reset(Fcitx5VnchkeyContext* ctx);

// Xử lý phím gõ chính
int fcitx5_vnchkey_process_key(
    Fcitx5VnchkeyContext* ctx,
    uint32_t key_char,
    double timestamp_ms,
    size_t* out_backspaces,
    char** out_commit
);

// Phím chức năng hoàn nguyên
int fcitx5_vnchkey_undo_escape(Fcitx5VnchkeyContext* ctx, size_t* out_backspaces, char** out_commit);
int fcitx5_vnchkey_undo_macro(Fcitx5VnchkeyContext* ctx, size_t* out_backspaces, char** out_commit);
void fcitx5_vnchkey_handle_backspace(Fcitx5VnchkeyContext* ctx);

// Cấu hình kiểu gõ (0 = Auto, 1 = Telex, 2 = VNI)
void fcitx5_vnchkey_set_method(Fcitx5VnchkeyContext* ctx, uint32_t method);
uint32_t fcitx5_vnchkey_get_method(Fcitx5VnchkeyContext* ctx);

#ifdef __cplusplus
}
#endif

#endif // VNCHKEY_BRIDGING_HEADER_H
```

---

### Bước 3: Tầng Hook Bắt Phím `KeyTapManager.swift`

Tạo file `macos/Sources/KeyTapManager.swift`:
1. **Xin quyền Accessibility**: Gọi `AXIsProcessTrustedWithOptions`.
2. **Gắn hook `CGEvent.tapCreate`**: Lắng nghe mọi sự kiện `keyDown`, `leftMouseDown`, `rightMouseDown`.
3. **Gọi hàm Rust**: Khi người dùng nhấn phím, gửi mã Unicode vào `fcitx5_vnchkey_process_key`.
4. **Giả lập phím**:
   - Nếu Rust trả về `handled == 1`: Hủy sự kiện phím gốc (`return nil`), phát phím xóa `Backspace` (mã phím `51` trên Mac) và gửi chuỗi Unicode mới vào ứng dụng.

```swift
import Cocoa
import CoreGraphics

public class KeyTapManager {
    private var engine: OpaquePointer?
    private var eventTap: CFMachPort?
    private var runLoopSource: CFRunLoopSource?
    
    public var isEnabled: Bool = true {
        didSet { if !isEnabled { resetEngine() } }
    }

    public init(method: UInt32 = 0) {
        self.engine = fcitx5_vnchkey_init_with_method(method)
    }

    deinit {
        stop()
        if let engine = self.engine { fcitx5_vnchkey_free(engine) }
    }

    public func setMethod(_ method: UInt32) {
        if let engine = self.engine { fcitx5_vnchkey_set_method(engine, method) }
    }

    public func resetEngine() {
        if let engine = self.engine { fcitx5_vnchkey_reset(engine) }
    }

    public func checkAccessibilityPermission(prompt: Bool = true) -> Bool {
        let options: NSDictionary = [kAXTrustedCheckOptionPrompt.takeUnretainedValue() as String: prompt]
        return AXIsProcessTrustedWithOptions(options)
    }

    public func start() -> Bool {
        guard checkAccessibilityPermission(prompt: true) else { return false }

        let eventMask = (1 << CGEventType.keyDown.rawValue) |
                        (1 << CGEventType.leftMouseDown.rawValue) |
                        (1 << CGEventType.rightMouseDown.rawValue)

        let observer = UnsafeMutableRawPointer(Unmanaged.passUnretained(self).toOpaque())

        self.eventTap = CGEvent.tapCreate(
            tap: .cgSessionEventTap,
            place: .headInsertEventTap,
            options: .defaultTap,
            eventsOfInterest: CGEventMask(eventMask),
            callback: { (proxy, type, event, refcon) -> Unmanaged<CGEvent>? in
                guard let refcon = refcon else { return Unmanaged.passRetained(event) }
                let manager = Unmanaged<KeyTapManager>.fromOpaque(refcon).takeUnretainedValue()

                // Tự phục hồi nếu Tap bị timeout do hệ điều hành ngắt
                if type == .tapDisabledByTimeout || type == .tapDisabledByUserInput {
                    if let tap = manager.eventTap { CGEvent.tapEnable(tap: tap, enable: true) }
                    return Unmanaged.passRetained(event)
                }

                // Click chuột thì reset từ đang gõ
                if type == .leftMouseDown || type == .rightMouseDown {
                    manager.resetEngine()
                    return Unmanaged.passRetained(event)
                }

                return manager.handleKeyEvent(proxy: proxy, type: type, event: event)
            },
            userInfo: observer
        )

        guard let tap = self.eventTap else { return false }
        self.runLoopSource = CFMachPortCreateRunLoopSource(kCFAllocatorDefault, tap, 0)
        CFRunLoopAddSource(CFRunLoopGetMain(), self.runLoopSource, .commonModes)
        CGEvent.tapEnable(tap: tap, enable: true)
        return true
    }

    public func stop() {
        if let tap = self.eventTap {
            CGEvent.tapEnable(tap: tap, enable: false)
            if let src = self.runLoopSource { CFRunLoopRemoveSource(CFRunLoopGetMain(), src, .commonModes) }
            self.eventTap = nil
            self.runLoopSource = nil
        }
    }

    private func handleKeyEvent(proxy: CGEventTapProxy, type: CGEventType, event: CGEvent) -> Unmanaged<CGEvent>? {
        guard isEnabled, let engine = self.engine else { return Unmanaged.passRetained(event) }

        let flags = event.flags
        let keyCode = event.getIntegerValueField(.keyboardEventKeycode)

        // Bỏ qua phím tắt hệ thống: Command (Cmd+C, Cmd+V...) hoặc Control
        if flags.contains(.maskCommand) || flags.contains(.maskControl) {
            resetEngine()
            return Unmanaged.passRetained(event)
        }

        // Phím Escape (53): Hoàn nguyên về chữ thô
        if keyCode == 53 {
            var backspaces: Int = 0
            var outCommit: UnsafeMutablePointer<CChar>? = nil
            if fcitx5_vnchkey_undo_escape(engine, &backspaces, &outCommit) == 1 {
                emitOutput(backspaces: backspaces, commitPtr: outCommit)
                return nil
            }
            resetEngine()
            return Unmanaged.passRetained(event)
        }

        // Phím Backspace / Delete (51)
        if keyCode == 51 {
            var backspaces: Int = 0
            var outCommit: UnsafeMutablePointer<CChar>? = nil
            if fcitx5_vnchkey_undo_macro(engine, &backspaces, &outCommit) == 1 {
                emitOutput(backspaces: backspaces, commitPtr: outCommit)
                return nil
            }
            fcitx5_vnchkey_handle_backspace(engine)
            return Unmanaged.passRetained(event)
        }

        // Trích xuất ký tự Unicode từ sự kiện
        var length: Int = 0
        var chars = [UniChar](repeating: 0, count: 4)
        event.keyboardGetUnicodeString(maxStringLength: 4, actualStringLength: &length, unicodeString: &chars)
        guard length > 0, let scalar = UnicodeScalar(chars[0]) else {
            return Unmanaged.passRetained(event)
        }

        var backspaces: Int = 0
        var outCommit: UnsafeMutablePointer<CChar>? = nil
        let timestamp = Date().timeIntervalSince1970 * 1000.0

        let handled = fcitx5_vnchkey_process_key(
            engine,
            scalar.value,
            timestamp,
            &backspaces,
            &outCommit
        )

        if handled == 1 {
            emitOutput(backspaces: backspaces, commitPtr: outCommit)
            return nil // Hủy sự kiện phím gốc
        }

        return Unmanaged.passRetained(event)
    }

    private func emitOutput(backspaces: Int, commitPtr: UnsafeMutablePointer<CChar>?) {
        // 1. Giả lập bấm Backspace để xoá ký tự cũ
        if backspaces > 0 {
            for _ in 0..<backspaces { postVirtualKey(keyCode: 51) }
        }
        // 2. Chèn chuỗi commit mới vào ứng dụng
        if let commitPtr = commitPtr {
            let commitStr = String(cString: commitPtr)
            if !commitStr.isEmpty { postUnicodeString(commitStr) }
            fcitx5_vnchkey_free_string(commitPtr) // Giải phóng chuỗi Rust đã allocate
        }
    }

    private func postVirtualKey(keyCode: CGKeyCode) {
        let src = CGEventSource(stateID: .hidSystemState)
        let down = CGEvent(keyboardEventSource: src, virtualKey: keyCode, keyDown: true)
        let up = CGEvent(keyboardEventSource: src, virtualKey: keyCode, keyDown: false)
        down?.post(tap: .cghidEventTap)
        up?.post(tap: .cghidEventTap)
    }

    private func postUnicodeString(_ string: String) {
        let src = CGEventSource(stateID: .hidSystemState)
        let utf16 = Array(string.utf16)
        let down = CGEvent(keyboardEventSource: src, virtualKey: 0, keyDown: true)
        let up = CGEvent(keyboardEventSource: src, virtualKey: 0, keyDown: false)
        down?.keyboardSetUnicodeString(stringLength: utf16.count, unicodeString: utf16)
        up?.keyboardSetUnicodeString(stringLength: utf16.count, unicodeString: utf16)
        down?.post(tap: .cghidEventTap)
        up?.post(tap: .cghidEventTap)
    }
}
```

---

### Bước 4: Giao Diện Menu Bar `main.swift`

Tạo file `macos/Sources/main.swift`. Đây là ứng dụng thanh trạng thái góc phải phía trên màn hình, nạp icon cờ vàng truyền thống:

```swift
import Cocoa

class AppDelegate: NSObject, NSApplicationDelegate {
    var statusItem: NSStatusItem?
    var keyManager: KeyTapManager?
    var currentMethod: UInt32 = 0 // 0: Auto, 1: Telex, 2: VNI
    var vnchIcon: NSImage?

    func applicationDidFinishLaunching(_ notification: Notification) {
        loadIcons()
        setupStatusBar()
        keyManager = KeyTapManager(method: currentMethod)
        _ = keyManager?.start()
    }

    func loadIcons() {
        if let imagePath = Bundle.main.path(forResource: "flag_vnch", ofType: "png") {
            let img = NSImage(contentsOfFile: imagePath)
            img?.size = NSSize(width: 18, height: 18)
            img?.isTemplate = false
            self.vnchIcon = img
        }
    }

    func setupStatusBar() {
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        updateStatusButton()

        let menu = NSMenu()
        let toggleItem = NSMenuItem(title: "Bật gõ tiếng Việt", action: #selector(toggleEnabled), keyEquivalent: "v")
        toggleItem.state = .on
        menu.addItem(toggleItem)
        menu.addItem(NSMenuItem.separator())

        let methodMenu = NSMenu()
        methodMenu.addItem(NSMenuItem(title: "Tự động (Auto)", action: #selector(selectAutoMethod), keyEquivalent: ""))
        methodMenu.addItem(NSMenuItem(title: "Telex", action: #selector(selectTelexMethod), keyEquivalent: ""))
        methodMenu.addItem(NSMenuItem(title: "VNI", action: #selector(selectVniMethod), keyEquivalent: ""))
        
        let methodParent = NSMenuItem(title: "Kiểu gõ", action: nil, keyEquivalent: "")
        methodParent.submenu = methodMenu
        menu.addItem(methodParent)

        menu.addItem(NSMenuItem.separator())
        menu.addItem(NSMenuItem(title: "Thoát vnchKey", action: #selector(quitApp), keyEquivalent: "q"))
        statusItem?.menu = menu
    }

    func updateStatusButton() {
        guard let button = statusItem?.button else { return }
        if keyManager?.isEnabled ?? true {
            if let icon = self.vnchIcon {
                button.image = icon
                button.title = " V"
            } else {
                button.image = nil
                button.title = "🟨 V"
            }
        } else {
            button.image = nil
            button.title = "🇬🇧 E"
        }
    }

    @objc func toggleEnabled(_ sender: NSMenuItem) {
        keyManager?.isEnabled.toggle()
        sender.state = (keyManager?.isEnabled ?? true) ? .on : .off
        updateStatusButton()
    }

    @objc func selectAutoMethod() { keyManager?.setMethod(0) }
    @objc func selectTelexMethod() { keyManager?.setMethod(1) }
    @objc func selectVniMethod() { keyManager?.setMethod(2) }
    @objc func quitApp() { NSApplication.shared.terminate(self) }
}

let app = NSApplication.shared
let delegate = AppDelegate()
app.delegate = delegate
app.run()
```

---

### Bước 5: Tự Động Hóa Biên Dịch & Đóng Gói Với `Makefile`

Tạo file `macos/Makefile` để việc đóng gói chỉ tốn 1 lệnh `make app`:

```makefile
APP_NAME = vnchKey
TARGET_DIR = ../target/release
BUILD_DIR = build
APP_BUNDLE = $(BUILD_DIR)/$(APP_NAME).app
CONTENTS = $(APP_BUNDLE)/Contents
MACOS_DIR = $(CONTENTS)/MacOS
RESOURCES_DIR = $(CONTENTS)/Resources

SWIFT_SOURCES = Sources/main.swift Sources/KeyTapManager.swift
BRIDGING_HEADER = Bridging-Header.h

.PHONY: all rust app run clean

all: app

# 1. Biên dịch lõi Rust ra static library
rust:
	cd .. && cargo build --release -p fcitx5-vnchkey-ffi

# 2. Tạo Bundle .app chuẩn macOS
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
    <key>CFBundleIdentifier</key>\
    <string>com.vnchkey.macos</string>\
    <key>CFBundleName</key>\
    <string>$(APP_NAME)</string>\
    <key>CFBundlePackageType</key>\
    <string>APPL</string>\
    <key>CFBundleShortVersionString</key>\
    <string>0.3.0</string>\
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
	
	@echo "✅ Đã tạo xong $(APP_BUNDLE)"

run: app
	open $(APP_BUNDLE)

clean:
	rm -rf $(BUILD_DIR)
```

---

## 5. Giải Quyết Các Vấn Đề Thực Chiến Trên macOS

### 5.1. Chống nuốt phím khi Tap bị Timeout
Trên macOS, nếu hệ thống bận hoặc một app xử lý phím trễ quá thời gian quy định, macOS sẽ tự động phát sinh sự kiện:
- `kCGEventTapDisabledByTimeout`
- `kCGEventTapDisabledByUserInput`

Nếu không bắt 2 sự kiện này, bộ gõ sẽ đột ngột "chết" hoàn toàn. Trong code Swift, ta chỉ cần:
```swift
if type == .tapDisabledByTimeout || type == .tapDisabledByUserInput {
    if let tap = manager.eventTap {
        CGEvent.tapEnable(tap: tap, enable: true) // Đánh thức tap hoạt động trở lại
    }
    return Unmanaged.passRetained(event)
}
```

### 5.2. Đồng bộ bộ nhớ khi Click chuột hoặc đổi cửa sổ
Khi người dùng đang gõ dở chữ `vie` nhưng click chuột sang một ô tìm kiếm khác hoặc chuyển cửa sổ ứng dụng:
- Nếu không reset, khi gõ tiếp `t`, bộ gõ sẽ tưởng vẫn đang gõ chữ cũ và gửi phím Backspace xóa nhầm nội dung ở cửa sổ mới.
- **Giải pháp**: Lắng nghe `leftMouseDown` và `rightMouseDown` trong `CGEventMask`. Mỗi lần click chuột, gọi ngay `fcitx5_vnchkey_reset(engine)` để làm sạch bộ đệm.

### 5.3. Hỗ trợ phím Escape hoàn nguyên & Undo Macro
- **Dokey style Escape**: Khi gõ sai từ tiếng Việt (ví dụ `tiếng`), nhấn `Escape` sẽ phục hồi lại thành ký tự gốc thô (`tieengs`). Hàm FFI `fcitx5_vnchkey_undo_escape` đã làm sẵn việc này.
- **Undo Macro**: Khi bung một từ viết tắt (ví dụ `vn` $\rightarrow$ `Việt Nam`), nếu người dùng bấm `Backspace`, hàm `fcitx5_vnchkey_undo_macro` sẽ xoá `Việt Nam` và trả lại chữ `vn`.

### 5.4. Quản lý bộ nhớ an toàn (Tránh Memory Leak FFI)
Khi Rust trả về một chuỗi `*mut c_char` cho Swift qua FFI:
```rust
let c_str = CString::new(output.commit_text).unwrap_or_default();
*out_commit = c_str.into_raw(); // Rust từ bỏ quyền sở hữu con trỏ
```
Swift sau khi đọc xong `String(cString: commitPtr)` **bắt buộc phải gọi hàm giải phóng của Rust**:
```swift
fcitx5_vnchkey_free_string(commitPtr)
```
Tuyệt đối không gọi `free()` của C hay để trôi con trỏ, vì Rust sử dụng bộ cấp phát bộ nhớ (allocator) riêng.

---

## 6. Quy Trình Build & Chạy Thử Thực Tế Trên Máy Mac

Khi bạn ngồi trước 2 chiếc MacBook Pro:

1. **Mở Terminal và clone/pull repo**:
   ```bash
   git pull origin main
   ```
2. **Di chuyển vào thư mục `macos` và chạy `make`**:
   ```bash
   cd macos
   make app
   ```
3. **Mở ứng dụng**:
   ```bash
   open build/vnchKey.app
   ```
4. **Cấp quyền Trợ Năng (Accessibility)**:
   - Vào **System Settings > Privacy & Security > Accessibility**.
   - Bật công tắc cho phép **vnchKey**.
5. **Cài đặt vĩnh viễn vào hệ thống**:
   ```bash
   cp -r build/vnchKey.app /Applications/
   ```
   Để ứng dụng tự chạy mỗi khi mở máy: Vào **System Settings > General > Login Items**, nhấn dấu `+` và thêm `vnchKey` từ thư mục `/Applications`.

