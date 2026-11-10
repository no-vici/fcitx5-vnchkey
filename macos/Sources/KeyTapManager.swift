import Cocoa
import CoreGraphics

public class KeyTapManager {
    private var engine: OpaquePointer?
    private var eventTap: CFMachPort?
    private var runLoopSource: CFRunLoopSource?

    /// Giá trị định danh gắn vào CGEventUserData để phân biệt phím do vnchKey phát ra
    /// với phím vật lý người dùng bấm (tránh vòng lặp re-entrancy làm lỗi dau61 -> dâấ)
    private let VNCHKEY_SYNTHETIC_TAG: Int64 = 0x564E4348 // "VNCH"

    public var isEnabled: Bool = true {
        didSet {
            if !isEnabled {
                resetEngine()
            }
        }
    }

    public var escRestoreEnabled: Bool = true

    public init(method: UInt32 = 0) { // 0: Auto, 1: Telex, 2: VNI, 3: SimpleTelex
        self.engine = fcitx5_vnchkey_init_with_method(method)
        if let engine = self.engine {
            // Tắt idle timeout mặc định (800ms) trên macOS để khi người dùng gõ VNI hoặc Telex chậm
            // (ví dụ gõ "dau" rồi dừng > 0.8 giây mới tìm phím số "6", "1") bộ đệm âm tiết không bị xoá!
            fcitx5_vnchkey_set_idle_timeout(engine, 0)
        }
        ensureDefaultConfigFiles()
    }

    deinit {
        stop()
        if let engine = self.engine {
            fcitx5_vnchkey_free(engine)
        }
    }

    // MARK: - Quản lý thư mục cấu hình (~/.config/vnchkey/)

    public func configDirPath() -> String {
        let home = FileManager.default.homeDirectoryForCurrentUser.path
        return "\(home)/.config/vnchkey"
    }

    public func macrosFilePath() -> String {
        return "\(configDirPath())/macros.conf"
    }

    public func dictionaryFilePath() -> String {
        return "\(configDirPath())/dictionary.conf"
    }

    public func ensureDefaultConfigFiles() {
        let fm = FileManager.default
        let dir = configDirPath()
        if !fm.fileExists(atPath: dir) {
            try? fm.createDirectory(atPath: dir, withIntermediateDirectories: true)
        }

        let macroFile = macrosFilePath()
        if !fm.fileExists(atPath: macroFile) {
            let defaultMacros = """
            # Bảng Gõ Tắt (Macros) của vnchKey
            # Cú pháp: viết_tắt=cụm_từ_đầy_đủ
            # Hỗ trợ biến động: $DATE, $TIME, $YEAR, $NOW, $DATETIME
            vn=Việt Nam
            vnch=Việt Nam Cộng Hòa
            td=Tự Do
            sg=Sài Gòn
            dc=được
            ko=không
            ng=người
            hn=hôm nay
            cv=công việc
            """
            try? defaultMacros.write(toFile: macroFile, atomically: true, encoding: .utf8)
        }

        let dictFile = dictionaryFilePath()
        if !fm.fileExists(atPath: dictFile) {
            let defaultDict = """
            # Từ Điển Cá Nhân (Custom English / Bypass Words)
            # Mỗi từ trên 1 dòng. Khi gõ các từ này, vnchKey sẽ giữ nguyên tiếng Anh.
            macbook
            xcode
            swift
            github
            terminal
            """
            try? defaultDict.write(toFile: dictFile, atomically: true, encoding: .utf8)
        }

        reloadMacrosAndDictionary()
    }

    /// Đọc danh sách các cặp Gõ Tắt (viết_tắt, cụm_từ) từ macros.conf
    public func readMacrosList() -> [(String, String)] {
        ensureDefaultConfigFiles()
        guard let content = try? String(contentsOfFile: macrosFilePath(), encoding: .utf8) else {
            return []
        }
        var result: [(String, String)] = []
        for line in content.components(separatedBy: .newlines) {
            let trimmed = line.trimmingCharacters(in: .whitespaces)
            if trimmed.isEmpty || trimmed.hasPrefix("#") { continue }
            if let eqRange = trimmed.range(of: "=") {
                let trigger = String(trimmed[..<eqRange.lowerBound]).trimmingCharacters(in: .whitespaces)
                let expansion = String(trimmed[eqRange.upperBound...]).trimmingCharacters(in: .whitespaces)
                if !trigger.isEmpty && !expansion.isEmpty {
                    result.append((trigger, expansion))
                }
            }
        }
        return result
    }

    /// Lưu danh sách Gõ Tắt vào macros.conf và nạp lại vào Engine
    public func saveMacrosList(_ list: [(String, String)]) {
        var lines: [String] = [
            "# Bảng Gõ Tắt (Macros) của vnchKey",
            "# Cú pháp: viết_tắt=cụm_từ_đầy_đủ",
            "# Hỗ trợ biến động: $DATE, $TIME, $YEAR, $NOW, $DATETIME"
        ]
        var seen = Set<String>()
        for (k, v) in list {
            let key = k.trimmingCharacters(in: .whitespaces)
            let val = v.trimmingCharacters(in: .whitespaces)
            let norm = key.lowercased()
            if !norm.isEmpty && !val.isEmpty && !seen.contains(norm) {
                seen.insert(norm)
                lines.append("\(key)=\(val)")
            }
        }
        let output = lines.joined(separator: "\n") + "\n"
        try? output.write(toFile: macrosFilePath(), atomically: true, encoding: .utf8)
        reloadMacrosAndDictionary()
    }

    /// Thêm hoặc cập nhật 1 mục Gõ Tắt rồi nạp lại ngay lập tức
    public func addOrUpdateMacro(trigger: String, expansion: String) {
        let cleanTrigger = trigger.trimmingCharacters(in: .whitespaces)
        let cleanExpansion = expansion.trimmingCharacters(in: .whitespaces)
        guard !cleanTrigger.isEmpty, !cleanExpansion.isEmpty else { return }

        var list = readMacrosList()
        if let idx = list.firstIndex(where: { $0.0.lowercased() == cleanTrigger.lowercased() }) {
            list[idx] = (cleanTrigger, cleanExpansion)
        } else {
            list.append((cleanTrigger, cleanExpansion))
        }
        saveMacrosList(list)
    }

    /// Xoá 1 mục Gõ Tắt theo từ viết tắt
    public func removeMacro(trigger: String) {
        let norm = trigger.trimmingCharacters(in: .whitespaces).lowercased()
        let list = readMacrosList().filter { $0.0.lowercased() != norm }
        saveMacrosList(list)
    }

    /// Đọc danh sách các từ trong Từ Điển Cá Nhân (dictionary.conf)
    public func readDictionaryWords() -> [String] {
        ensureDefaultConfigFiles()
        guard let content = try? String(contentsOfFile: dictionaryFilePath(), encoding: .utf8) else {
            return []
        }
        var words: [String] = []
        for line in content.components(separatedBy: .newlines) {
            let trimmed = line.trimmingCharacters(in: .whitespaces)
            if !trimmed.isEmpty && !trimmed.hasPrefix("#") {
                words.append(trimmed)
            }
        }
        return words
    }

    /// Lưu danh sách Từ Điển Cá Nhân vào dictionary.conf và nạp lại vào Engine
    public func saveDictionaryWords(_ words: [String]) {
        var lines: [String] = [
            "# Từ Điển Cá Nhân (Custom English / Bypass Words)",
            "# Mỗi từ trên 1 dòng. Khi gõ các từ này, vnchKey sẽ giữ nguyên tiếng Anh."
        ]
        var seen = Set<String>()
        for w in words {
            let clean = w.trimmingCharacters(in: .whitespaces)
            let norm = clean.lowercased()
            if !norm.isEmpty && !seen.contains(norm) {
                seen.insert(norm)
                lines.append(clean)
            }
        }
        let output = lines.joined(separator: "\n") + "\n"
        try? output.write(toFile: dictionaryFilePath(), atomically: true, encoding: .utf8)
        reloadMacrosAndDictionary()
    }

    /// Thêm 1 từ mới vào Từ Điển Cá Nhân rồi nạp lại ngay lập tức
    public func addDictionaryWord(_ word: String) {
        let clean = word.trimmingCharacters(in: .whitespaces)
        guard !clean.isEmpty else { return }
        var words = readDictionaryWords()
        if !words.contains(where: { $0.lowercased() == clean.lowercased() }) {
            words.append(clean)
            saveDictionaryWords(words)
        }
    }

    /// Xoá 1 từ khỏi Từ Điển Cá Nhân
    public func removeDictionaryWord(_ word: String) {
        let norm = word.trimmingCharacters(in: .whitespaces).lowercased()
        let words = readDictionaryWords().filter { $0.lowercased() != norm }
        saveDictionaryWords(words)
    }

    public func reloadMacrosAndDictionary() {
        guard let engine = self.engine else { return }
        let macroPath = macrosFilePath()
        let dictPath = dictionaryFilePath()
        macroPath.withCString { ptr in
            fcitx5_vnchkey_load_macros(engine, ptr)
        }
        dictPath.withCString { ptr in
            fcitx5_vnchkey_load_dictionary(engine, ptr)
        }
    }

    // MARK: - Các hàm Cấu Hình Engine (Getters / Setters)

    public func setMethod(_ method: UInt32) {
        if let engine = self.engine {
            fcitx5_vnchkey_set_method(engine, method)
            fcitx5_vnchkey_reset(engine)
        }
    }

    public func getMethod() -> UInt32 {
        guard let engine = self.engine else { return 0 }
        return fcitx5_vnchkey_get_method(engine)
    }

    public func setSpellCheck(_ enabled: Bool) {
        if let engine = self.engine {
            fcitx5_vnchkey_set_spell_check(engine, enabled)
        }
    }

    public func getSpellCheck() -> Bool {
        guard let engine = self.engine else { return true }
        return fcitx5_vnchkey_get_spell_check(engine)
    }

    public func setAutoRestore(_ enabled: Bool) {
        if let engine = self.engine {
            fcitx5_vnchkey_set_auto_restore(engine, enabled)
        }
    }

    public func getAutoRestore() -> Bool {
        guard let engine = self.engine else { return true }
        return fcitx5_vnchkey_get_auto_restore(engine)
    }

    public func setMacroEnabled(_ enabled: Bool) {
        if let engine = self.engine {
            fcitx5_vnchkey_set_macro_enabled(engine, enabled)
        }
    }

    public func getMacroEnabled() -> Bool {
        guard let engine = self.engine else { return true }
        return fcitx5_vnchkey_get_macro_enabled(engine)
    }

    public func setDictionaryEnabled(_ enabled: Bool) {
        if let engine = self.engine {
            fcitx5_vnchkey_set_dictionary_enabled(engine, enabled)
        }
    }

    public func getDictionaryEnabled() -> Bool {
        guard let engine = self.engine else { return true }
        return fcitx5_vnchkey_get_dictionary_enabled(engine)
    }

    public func setToneStyle(_ style: UInt32) { // 0: Modern (oà, uý), 1: Classic (hòa, ủy)
        if let engine = self.engine {
            fcitx5_vnchkey_set_tone_style(engine, style)
        }
    }

    public func getToneStyle() -> UInt32 {
        guard let engine = self.engine else { return 0 }
        return fcitx5_vnchkey_get_tone_style(engine)
    }

    public func setCharset(_ charset: UInt32) { // 0: Unicode NFC, 1: Composite NFD
        if let engine = self.engine {
            fcitx5_vnchkey_set_charset(engine, charset)
        }
    }

    public func setBracketTransform(_ enabled: Bool) {
        if let engine = self.engine {
            fcitx5_vnchkey_set_bracket_transform(engine, enabled)
        }
    }

    public func setW2U(_ mode: UInt32) { // 0: NonStart, 1: Always, 2: Disabled
        if let engine = self.engine {
            fcitx5_vnchkey_set_w2u(engine, mode)
        }
    }

    public func resetEngine() {
        if let engine = self.engine {
            fcitx5_vnchkey_reset(engine)
        }
    }

    // MARK: - Quản lý Quyền Accessibility & CGEventTap

    public func checkAccessibilityPermission(prompt: Bool = true) -> Bool {
        let options: NSDictionary = [kAXTrustedCheckOptionPrompt.takeUnretainedValue() as String: prompt]
        return AXIsProcessTrustedWithOptions(options)
    }

    public func start() -> Bool {
        guard checkAccessibilityPermission(prompt: true) else {
            print("⚠️ [vnchKey] Chưa có quyền Accessibility. Vui lòng cấp quyền trong System Settings.")
            return false
        }

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
        // QUAN TRỌNG: Bỏ qua các phím giả lập do chính vnchKey vừa phát ra!
        // Điều này ngăn chặn lỗi gõ VNI dau61 -> dâấ do phím Backspace/Unicode bị bắt lại vào Engine.
        if event.getIntegerValueField(.eventSourceUserData) == VNCHKEY_SYNTHETIC_TAG {
            return Unmanaged.passRetained(event)
        }

        guard isEnabled, let engine = self.engine else {
            return Unmanaged.passRetained(event)
        }

        let flags = event.flags
        let keyCode = event.getIntegerValueField(.keyboardEventKeycode)

        // Bỏ qua nếu đang bấm các phím tổ hợp Command, Control hoặc Option
        if flags.contains(.maskCommand) || flags.contains(.maskControl) || flags.contains(.maskAlternate) {
            resetEngine()
            return Unmanaged.passRetained(event)
        }

        // Xử lý phím điều hướng (Mũi tên, Home, End, PageUp, PageDown, Tab, Return/Enter): Reset từ đang gõ
        if [123, 124, 125, 126, 115, 119, 116, 121, 48, 36, 76].contains(keyCode) {
            resetEngine()
            return Unmanaged.passRetained(event)
        }

                // Xử lý phím Escape (kVK_Escape = 53): hoàn nguyên từ thô về Dokey style
        if keyCode == 53 {
            if escRestoreEnabled {
                var backspaces: Int = 0
                var outCommit: UnsafeMutablePointer<CChar>? = nil
                if fcitx5_vnchkey_undo_escape(engine, &backspaces, &outCommit) == 1 {
                    emitOutput(backspaces: backspaces, commitPtr: outCommit)
                    return nil
                }
            }
            resetEngine()
            return Unmanaged.passRetained(event)
        }

        // Xử lý phím Delete/Backspace (kVK_Delete = 51)
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

        // Lấy ký tự Unicode từ sự kiện phím vật lý
        var length: Int = 0
        var chars = [UniChar](repeating: 0, count: 4)
        event.keyboardGetUnicodeString(maxStringLength: 4, actualStringLength: &length, unicodeString: &chars)

        guard length > 0, let scalar = UnicodeScalar(chars[0]) else {
            return Unmanaged.passRetained(event)
        }

        var backspaces: Int = 0
        var outCommit: UnsafeMutablePointer<CChar>? = nil
        let handled = fcitx5_vnchkey_process_key(
            engine,
            scalar.value,
            0.0, // Truyền 0.0 trên macOS để không bao giờ bị timeout xoá bộ đệm khi dừng tay tìm phím số VNI
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
        let src = CGEventSource(stateID: .hidSystemState)

        // 1. Giả lập phím Backspace để xóa ký tự cũ
        if backspaces > 0 {
            for _ in 0..<backspaces {
                postVirtualKey(src: src, keyCode: 51) // 51 = macOS Backspace
            }
        }

        // 2. Chèn chuỗi commit mới
        if let commitPtr = commitPtr {
            let commitStr = String(cString: commitPtr)
            if !commitStr.isEmpty {
                postUnicodeString(src: src, string: commitStr)
            }
            fcitx5_vnchkey_free_string(commitPtr)
        }
    }

    private func postVirtualKey(src: CGEventSource?, keyCode: CGKeyCode) {
        if let down = CGEvent(keyboardEventSource: src, virtualKey: keyCode, keyDown: true),
           let up = CGEvent(keyboardEventSource: src, virtualKey: keyCode, keyDown: false) {
            down.setIntegerValueField(.eventSourceUserData, value: VNCHKEY_SYNTHETIC_TAG)
            up.setIntegerValueField(.eventSourceUserData, value: VNCHKEY_SYNTHETIC_TAG)
            down.post(tap: .cghidEventTap)
            up.post(tap: .cghidEventTap)
        }
    }

    private func postUnicodeString(src: CGEventSource?, string: String) {
        let utf16 = Array(string.utf16)
        if let down = CGEvent(keyboardEventSource: src, virtualKey: 0, keyDown: true),
           let up = CGEvent(keyboardEventSource: src, virtualKey: 0, keyDown: false) {
            down.setIntegerValueField(.eventSourceUserData, value: VNCHKEY_SYNTHETIC_TAG)
            up.setIntegerValueField(.eventSourceUserData, value: VNCHKEY_SYNTHETIC_TAG)
            down.keyboardSetUnicodeString(stringLength: utf16.count, unicodeString: utf16)
            up.keyboardSetUnicodeString(stringLength: utf16.count, unicodeString: utf16)
            down.post(tap: .cghidEventTap)
            up.post(tap: .cghidEventTap)
        }
    }
}
