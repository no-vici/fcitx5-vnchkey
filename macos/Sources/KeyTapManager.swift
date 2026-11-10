import Cocoa
import CoreGraphics

public class KeyTapManager {
    private var engine: OpaquePointer?
    private var eventTap: CFMachPort?
    private var runLoopSource: CFRunLoopSource?
    public var isEnabled: Bool = true {
        didSet {
            if !isEnabled {
                resetEngine()
            }
        }
    }

    public init(method: UInt32 = 0) { // 0: Auto, 1: Telex, 2: VNI
        self.engine = fcitx5_vnchkey_init_with_method(method)
    }

    deinit {
        stop()
        if let engine = self.engine {
            fcitx5_vnchkey_free(engine)
        }
    }

    public func setMethod(_ method: UInt32) {
        if let engine = self.engine {
            fcitx5_vnchkey_free(engine)
        }
        self.engine = fcitx5_vnchkey_init_with_method(method)
    }

    public func resetEngine() {
        if let engine = self.engine {
            fcitx5_vnchkey_reset(engine)
        }
    }

    /// Kiểm tra và mở popup xin quyền Trợ Năng (Accessibility) nếu chưa có
    public func checkAccessibilityPermission(prompt: Bool = true) -> Bool {
        let options: NSDictionary = [kAXTrustedCheckOptionPrompt.takeUnretainedValue() as String: prompt]
        return AXIsProcessTrustedWithOptions(options)
    }

    /// Bắt đầu lắng nghe bàn phím hệ thống
    public func start() -> Bool {
        guard checkAccessibilityPermission(prompt: true) else {
            print("⚠️ [vnchKey] Chưa có quyền Accessibility. Vui lòng cấp quyền trong System Settings.")
            return false
        }

        // Bắt sự kiện KeyDown và chuột click để reset bộ gõ
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

                // Phục hồi tự động khi macOS ngắt tap do quá tải hoặc timeout
                if type == .tapDisabledByTimeout || type == .tapDisabledByUserInput {
                    if let tap = manager.eventTap {
                        CGEvent.tapEnable(tap: tap, enable: true)
                    }
                    return Unmanaged.passRetained(event)
                }

                // Khi click chuột ở vị trí mới, reset lại trạng thái từ đang gõ
                if type == .leftMouseDown || type == .rightMouseDown {
                    manager.resetEngine()
                    return Unmanaged.passRetained(event)
                }

                return manager.handleKeyEvent(proxy: proxy, type: type, event: event)
            },
            userInfo: observer
        )

        guard let tap = self.eventTap else {
            print("❌ [vnchKey] Không thể khởi tạo CGEventTap.")
            return false
        }

        self.runLoopSource = CFMachPortCreateRunLoopSource(kCFAllocatorDefault, tap, 0)
        CFRunLoopAddSource(CFRunLoopGetMain(), self.runLoopSource, .commonModes)
        CGEvent.tapEnable(tap: tap, enable: true)
        print("✅ [vnchKey] Hook bàn phím CGEventTap đã hoạt động.")
        return true
    }

    public func stop() {
        if let tap = self.eventTap {
            CGEvent.tapEnable(tap: tap, enable: false)
            if let src = self.runLoopSource {
                CFRunLoopRemoveSource(CFRunLoopGetMain(), src, .commonModes)
            }
            self.eventTap = nil
            self.runLoopSource = nil
        }
    }

    private func handleKeyEvent(proxy: CGEventTapProxy, type: CGEventType, event: CGEvent) -> Unmanaged<CGEvent>? {
        guard isEnabled, let engine = self.engine else {
            return Unmanaged.passRetained(event)
        }

        let flags = event.flags
        let keyCode = event.getIntegerValueField(.keyboardEventKeycode)

        // Bỏ qua nếu đang bấm các phím tổ hợp Command hoặc Control (copy, paste, quit, terminal shortcuts, ...)
        if flags.contains(.maskCommand) || flags.contains(.maskControl) {
            resetEngine()
            return Unmanaged.passRetained(event)
        }

        // Xử lý phím Escape (kVK_Escape = 53): hoàn nguyên từ thô về Dokey style
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

        // Xử lý phím Delete/Backspace (kVK_Delete = 51)
        if keyCode == 51 {
            // Kiểm tra xem có đang undo macro không
            var backspaces: Int = 0
            var outCommit: UnsafeMutablePointer<CChar>? = nil
            if fcitx5_vnchkey_undo_macro(engine, &backspaces, &outCommit) == 1 {
                emitOutput(backspaces: backspaces, commitPtr: outCommit)
                return nil
            }
            fcitx5_vnchkey_handle_backspace(engine)
            return Unmanaged.passRetained(event)
        }

        // Lấy ký tự Unicode
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
            return nil // Chặn phím gốc
        }

        return Unmanaged.passRetained(event)
    }

    private func emitOutput(backspaces: Int, commitPtr: UnsafeMutablePointer<CChar>?) {
        // 1. Giả lập phím Backspace để xóa ký tự cũ
        if backspaces > 0 {
            for _ in 0..<backspaces {
                postVirtualKey(keyCode: 51) // macOS Backspace
            }
        }

        // 2. Chèn chuỗi commit mới
        if let commitPtr = commitPtr {
            let commitStr = String(cString: commitPtr)
            if !commitStr.isEmpty {
                postUnicodeString(commitStr)
            }
            fcitx5_vnchkey_free_string(commitPtr)
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

