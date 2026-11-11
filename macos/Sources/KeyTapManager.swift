import Cocoa
import CoreGraphics

/// Cấu hình Phím tắt toàn cục tự do để Bật / Tắt chế độ gõ Tiếng Việt (tinh thần Tự Do)
/// Người dùng có thể tự chọn bất kỳ tổ hợp Modifier (⌃ Control, ⌥ Option, ⇧ Shift, ⌘ Command)
/// kết hợp với bất kỳ phím nào trên bàn phím, hoặc chỉ dùng tổ hợp Modifier-only (nhấn rồi thả).
public struct CustomToggleShortcut: Equatable {
    public var useControl: Bool
    public var useOption: Bool
    public var useShift: Bool
    public var useCommand: Bool
    /// Mã phím vật lý macOS (`CGKeyCode`), hoặc `-1` nếu chỉ dùng tổ hợp phím Modifier (nhấn rồi thả như Ctrl+Shift)
    public var keyCode: Int64
    /// Nhãn hiển thị của phím chính (ví dụ: `"Space"`, `"Z"`, `"V"`, `""` nếu chỉ dùng Modifier)
    public var keyDisplay: String
    /// Đang bật hay tắt phím tắt chuyển nhanh
    public var isEnabled: Bool

    public static let defaultShortcut = CustomToggleShortcut(
        useControl: true,
        useOption: false,
        useShift: true,
        useCommand: false,
        keyCode: -1,
        keyDisplay: "",
        isEnabled: true
    )

    public var isModifierOnly: Bool {
        return keyCode < 0
    }

    public var hasAnyModifier: Bool {
        return useControl || useOption || useShift || useCommand
    }

    /// Ký hiệu ngắn gọn trên Menu Bar (ví dụ: `⌃⇧`, `⌘⇧Z`, `⌥Space`)
    public var shortBadge: String {
        guard isEnabled, hasAnyModifier else { return "" }
        var s = ""
        if useControl { s += "⌃" }
        if useOption  { s += "⌥" }
        if useShift   { s += "⇧" }
        if useCommand { s += "⌘" }
        if !isModifierOnly && !keyDisplay.isEmpty {
            s += keyDisplay
        }
        return s
    }

    /// Mô tả đầy đủ dễ đọc trong Bảng Điều Khiển
    public var fullDescription: String {
        guard isEnabled, hasAnyModifier else { return "Đã tắt phím tắt chuyển nhanh" }
        var parts: [String] = []
        if useControl { parts.append("⌃ Control") }
        if useOption  { parts.append("⌥ Option") }
        if useShift   { parts.append("⇧ Shift") }
        if useCommand { parts.append("⌘ Command") }
        if !isModifierOnly && !keyDisplay.isEmpty {
            parts.append(keyDisplay)
        }
        return parts.joined(separator: " + ")
    }

    /// Chuyển mã `keyCode` của macOS sang tên phím dễ đọc
    public static func keyName(for keyCode: Int64, fallbackChar: String? = nil) -> String {
        switch keyCode {
        case 49: return "Space"
        case 36: return "Return"
        case 48: return "Tab"
        case 51: return "Delete"
        case 53: return "Esc"
        case 50: return "`"
        case 27: return "-"
        case 24: return "="
        case 33: return "["
        case 30: return "]"
        case 42: return "\\"
        case 41: return ";"
        case 39: return "'"
        case 43: return ","
        case 47: return "."
        case 44: return "/"
        case 0:  return "A"
        case 11: return "B"
        case 8:  return "C"
        case 2:  return "D"
        case 14: return "E"
        case 3:  return "F"
        case 5:  return "G"
        case 4:  return "H"
        case 34: return "I"
        case 38: return "J"
        case 40: return "K"
        case 37: return "L"
        case 46: return "M"
        case 45: return "N"
        case 31: return "O"
        case 35: return "P"
        case 12: return "Q"
        case 15: return "R"
        case 1:  return "S"
        case 17: return "T"
        case 32: return "U"
        case 9:  return "V"
        case 13: return "W"
        case 7:  return "X"
        case 16: return "Y"
        case 6:  return "Z"
        case 29: return "0"
        case 18: return "1"
        case 19: return "2"
        case 20: return "3"
        case 21: return "4"
        case 23: return "5"
        case 22: return "6"
        case 26: return "7"
        case 28: return "8"
        case 25: return "9"
        case 122: return "F1"
        case 120: return "F2"
        case 99:  return "F3"
        case 118: return "F4"
        case 96:  return "F5"
        case 97:  return "F6"
        case 98:  return "F7"
        case 100: return "F8"
        case 101: return "F9"
        case 109: return "F10"
        case 103: return "F11"
        case 111: return "F12"
        default:
            if let fb = fallbackChar?.uppercased(), !fb.isEmpty {
                return fb
            }
            return "Key(\(keyCode))"
        }
    }
}

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
    public var customShortcut: CustomToggleShortcut = .defaultShortcut
    public var onToggleEnabledByShortcut: (() -> Void)?

    /// Chế độ thu phím tắt trực tiếp từ bàn phím người dùng (Live Key Recorder)
    public var isRecordingShortcut: Bool = false
    public var onShortcutRecorded: ((CustomToggleShortcut) -> Void)?
    private var recordedModifiersPeak: CGEventFlags = []

    // Trạng thái theo dõi tổ hợp phím chỉ gồm Modifier (ví dụ Ctrl+Shift, Cmd+Shift, Ctrl+Option...)
    private var modifierChordArmed: Bool = false
    private var otherKeyPressedDuringChord: Bool = false

    public init(method: UInt32 = 0) { // 0: Auto, 1: Telex, 2: VNI, 3: SimpleTelex
        self.engine = fcitx5_vnchkey_init_with_method(method)
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
            # Mỗi từ trên 1 dòng. Khi gõ các từ này, vnchKey sẽ giữ nguyên Tiếng Anh.
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
            "# Mỗi từ trên 1 dòng. Khi gõ các từ này, vnchKey sẽ giữ nguyên Tiếng Anh."
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

    public func setIdleTimeout(_ ms: UInt64) {
        if let engine = self.engine {
            fcitx5_vnchkey_set_idle_timeout(engine, ms)
        }
    }

    public func getIdleTimeout() -> UInt64 {
        guard let engine = self.engine else { return 800 }
        return fcitx5_vnchkey_get_idle_timeout(engine)
    }

    public func setQuickTelex(_ enabled: Bool) {
        if let engine = self.engine {
            fcitx5_vnchkey_set_quick_telex(engine, enabled)
        }
    }

    public func setQuickStartConsonant(_ enabled: Bool) {
        if let engine = self.engine {
            fcitx5_vnchkey_set_quick_start_consonant(engine, enabled)
        }
    }

    public func setQuickEndConsonant(_ enabled: Bool) {
        if let engine = self.engine {
            fcitx5_vnchkey_set_quick_end_consonant(engine, enabled)
        }
    }

    public func flipWord() -> Bool {
        guard let engine = self.engine else { return false }
        var backspaces: Int = 0
        var outCommit: UnsafeMutablePointer<CChar>? = nil
        if fcitx5_vnchkey_flip_word(engine, &backspaces, &outCommit) == 1 {
            emitOutput(backspaces: backspaces, commitPtr: outCommit)
            return true
        }
        return false
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
                        (1 << CGEventType.flagsChanged.rawValue) |
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

    public func startRecordingShortcut() {
        recordedModifiersPeak = []
        isRecordingShortcut = true
    }

    public func stopRecordingShortcut() {
        isRecordingShortcut = false
        recordedModifiersPeak = []
    }

    private func handleFlagsChanged(event: CGEvent) {
        let flags = event.flags
        let ctrl = flags.contains(.maskControl)
        let shift = flags.contains(.maskShift)
        let cmd = flags.contains(.maskCommand)
        let opt = flags.contains(.maskAlternate)

        // Nếu đang ở chế độ "Thu phím tắt trực tiếp":
        // Cho phép người dùng nhấn tổ hợp chỉ gồm >= 2 phím Modifier (ví dụ Ctrl+Shift, Cmd+Option...) rồi thả ra!
        if isRecordingShortcut {
            if ctrl  { recordedModifiersPeak.insert(.maskControl) }
            if shift { recordedModifiersPeak.insert(.maskShift) }
            if cmd   { recordedModifiersPeak.insert(.maskCommand) }
            if opt   { recordedModifiersPeak.insert(.maskAlternate) }

            if !ctrl && !shift && !cmd && !opt {
                let pCtrl  = recordedModifiersPeak.contains(.maskControl)
                let pShift = recordedModifiersPeak.contains(.maskShift)
                let pCmd   = recordedModifiersPeak.contains(.maskCommand)
                let pOpt   = recordedModifiersPeak.contains(.maskAlternate)
                let modCount = [pCtrl, pShift, pCmd, pOpt].filter { $0 }.count

                if modCount >= 2 {
                    let newSc = CustomToggleShortcut(
                        useControl: pCtrl,
                        useOption: pOpt,
                        useShift: pShift,
                        useCommand: pCmd,
                        keyCode: -1,
                        keyDisplay: "",
                        isEnabled: true
                    )
                    isRecordingShortcut = false
                    recordedModifiersPeak = []
                    customShortcut = newSc
                    DispatchQueue.main.async { [weak self] in
                        self?.onShortcutRecorded?(newSc)
                    }
                    return
                }
                recordedModifiersPeak = []
            }
            return
        }

        guard customShortcut.isEnabled, customShortcut.isModifierOnly, customShortcut.hasAnyModifier else {
            modifierChordArmed = false
            otherKeyPressedDuringChord = false
            return
        }

        let targetArmed = (ctrl == customShortcut.useControl) &&
                          (shift == customShortcut.useShift) &&
                          (cmd == customShortcut.useCommand) &&
                          (opt == customShortcut.useOption)

        if targetArmed {
            modifierChordArmed = true
        } else {
            // Khi tất cả các phím modifier được thả ra hoàn toàn sau khi đã nhấn đúng tổ hợp mục tiêu
            if !ctrl && !shift && !cmd && !opt {
                if modifierChordArmed && !otherKeyPressedDuringChord {
                    isEnabled.toggle()
                    DispatchQueue.main.async { [weak self] in
                        self?.onToggleEnabledByShortcut?()
                    }
                }
                modifierChordArmed = false
                otherKeyPressedDuringChord = false
            }
        }
    }

    private func matchesKeyDownToggleShortcut(keyCode: Int64, flags: CGEventFlags) -> Bool {
        guard customShortcut.isEnabled, !customShortcut.isModifierOnly else {
            return false
        }
        let ctrl = flags.contains(.maskControl)
        let shift = flags.contains(.maskShift)
        let cmd = flags.contains(.maskCommand)
        let opt = flags.contains(.maskAlternate)

        return keyCode == customShortcut.keyCode &&
               ctrl == customShortcut.useControl &&
               shift == customShortcut.useShift &&
               cmd == customShortcut.useCommand &&
               opt == customShortcut.useOption
    }

    private func handleKeyEvent(proxy: CGEventTapProxy, type: CGEventType, event: CGEvent) -> Unmanaged<CGEvent>? {
        // QUAN TRỌNG: Bỏ qua các phím giả lập do chính vnchKey vừa phát ra!
        // Điều này ngăn chặn lỗi gõ VNI dau61 -> dâấ do phím Backspace/Unicode bị bắt lại vào Engine.
        if event.getIntegerValueField(.eventSourceUserData) == VNCHKEY_SYNTHETIC_TAG {
            return Unmanaged.passRetained(event)
        }

        if type == .flagsChanged {
            handleFlagsChanged(event: event)
            return Unmanaged.passRetained(event)
        }

        let flags = event.flags
        let keyCode = event.getIntegerValueField(.keyboardEventKeycode)

        // Nếu người dùng đang bấm nút "Thu phím tắt mới..." trên Bảng Điều Khiển
        if isRecordingShortcut {
            // Bấm Esc một mình sẽ huỷ thu phím tắt
            let ctrl = flags.contains(.maskControl)
            let shift = flags.contains(.maskShift)
            let cmd = flags.contains(.maskCommand)
            let opt = flags.contains(.maskAlternate)

            if keyCode == 53 && !ctrl && !shift && !cmd && !opt {
                stopRecordingShortcut()
                DispatchQueue.main.async { [weak self] in
                    guard let self = self else { return }
                    self.onShortcutRecorded?(self.customShortcut)
                }
                return nil
            }

            // Yêu cầu có ít nhất 1 phím Modifier (hoặc phím F1..F12) để không chặn phím gõ chữ bình thường
            let isFunctionKey = [122, 120, 99, 118, 96, 97, 98, 100, 101, 109, 103, 111].contains(keyCode)
            if ctrl || shift || cmd || opt || isFunctionKey {
                var length: Int = 0
                var chars = [UniChar](repeating: 0, count: 4)
                event.keyboardGetUnicodeString(maxStringLength: 4, actualStringLength: &length, unicodeString: &chars)
                let fallback = (length > 0 && UnicodeScalar(chars[0]) != nil) ? String(UnicodeScalar(chars[0])!) : nil
                let disp = CustomToggleShortcut.keyName(for: keyCode, fallbackChar: fallback)

                let newSc = CustomToggleShortcut(
                    useControl: ctrl,
                    useOption: opt,
                    useShift: shift,
                    useCommand: cmd,
                    keyCode: keyCode,
                    keyDisplay: disp,
                    isEnabled: true
                )
                stopRecordingShortcut()
                customShortcut = newSc
                DispatchQueue.main.async { [weak self] in
                    self?.onShortcutRecorded?(newSc)
                }
                return nil
            }
        }

        // Nếu có phím thường được bấm trong lúc đang giữ tổ hợp modifier, huỷ kích hoạt chord
        if modifierChordArmed {
            otherKeyPressedDuringChord = true
        }

        // Kiểm tra phím tắt Bật/Tắt nhanh dạng tổ hợp có phím ký tự
        if matchesKeyDownToggleShortcut(keyCode: keyCode, flags: flags) {
            isEnabled.toggle()
            DispatchQueue.main.async { [weak self] in
                self?.onToggleEnabledByShortcut?()
            }
            return nil // Chặn tổ hợp phím tắt không cho truyền xuống ứng dụng đang mở
        }

        guard isEnabled, let engine = self.engine else {
            return Unmanaged.passRetained(event)
        }

        // Bỏ qua nếu đang bấm các phím tổ hợp Command, Control hoặc Option
        // Ngoại lệ: Cho phép tổ hợp Đảo từ (Ctrl+Shift+Z hoặc custom flip shortcut)
        let isCtrl = flags.contains(.maskControl)
        let isShift = flags.contains(.maskShift)
        let isCmd = flags.contains(.maskCommand)
        let isOpt = flags.contains(.maskAlternate)

        // Hỗ trợ đảo từ bằng Ctrl+Shift+Z (hoặc Cmd+Shift+Z tuỳ chọn)
        if (isCtrl || isCmd) && isShift && keyCode == 6 { // 6 = 'Z'
            if flipWord() {
                return nil
            }
        }

        if isCmd || isCtrl || isOpt {
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
