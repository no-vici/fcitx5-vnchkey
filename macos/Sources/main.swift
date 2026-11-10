import Cocoa

class AppDelegate: NSObject, NSApplicationDelegate {
    var statusItem: NSStatusItem?
    var keyManager: KeyTapManager?
    var vnchIcon: NSImage?

    // Lưu cấu hình bền vững qua UserDefaults
    private let defaults = UserDefaults.standard

    func applicationDidFinishLaunching(_ notification: Notification) {
        loadIcons()
        if let appIconPath = Bundle.main.path(forResource: "AppIcon", ofType: "png"),
           let appIconImg = NSImage(contentsOfFile: appIconPath) {
            NSApp.applicationIconImage = appIconImg
        }
        keyManager = KeyTapManager(method: UInt32(defaults.integer(forKey: "vnchkey.method")))
        applySavedPreferences()
        setupStatusBar()

        if !(keyManager?.start() ?? false) {
            showAccessibilityAlert()
        }
    }

    func loadIcons() {
        if let imagePath = Bundle.main.path(forResource: "flag_vnch", ofType: "png") {
            let img = NSImage(contentsOfFile: imagePath)
            img?.size = NSSize(width: 18, height: 18)
            img?.isTemplate = false
            self.vnchIcon = img
        }
    }

    func applySavedPreferences() {
        guard let km = keyManager else { return }

        // Mặc định lần đầu chạy: Bật tất cả tính năng thông minh
        if defaults.object(forKey: "vnchkey.initialized") == nil {
            defaults.set(true, forKey: "vnchkey.initialized")
            defaults.set(0, forKey: "vnchkey.method")         // Auto
            defaults.set(true, forKey: "vnchkey.spellCheck")  // Kiểm tra Chánh Tả
            defaults.set(true, forKey: "vnchkey.autoRestore") // Tự động khôi phục Tiếng Anh
            defaults.set(true, forKey: "vnchkey.macro")       // Bật Gõ Tắt
            defaults.set(true, forKey: "vnchkey.dictionary")  // Bật Từ Điển Cá Nhân
            defaults.set(true, forKey: "vnchkey.escRestore")  // Bấm Esc hoàn nguyên từ
            defaults.set(0, forKey: "vnchkey.toneStyle")      // Dấu mới (oà, uý)
            defaults.set(0, forKey: "vnchkey.charset")        // Unicode dựng sẵn NFC
            defaults.set(false, forKey: "vnchkey.bracket")    // Gõ [ ] thành ơ ư
        }

        km.setMethod(UInt32(defaults.integer(forKey: "vnchkey.method")))
        km.setSpellCheck(defaults.bool(forKey: "vnchkey.spellCheck"))
        km.setAutoRestore(defaults.bool(forKey: "vnchkey.autoRestore"))
        km.setMacroEnabled(defaults.bool(forKey: "vnchkey.macro"))
        km.setDictionaryEnabled(defaults.bool(forKey: "vnchkey.dictionary"))
        km.escRestoreEnabled = defaults.bool(forKey: "vnchkey.escRestore")
        km.setToneStyle(UInt32(defaults.integer(forKey: "vnchkey.toneStyle")))
        km.setCharset(UInt32(defaults.integer(forKey: "vnchkey.charset")))
        km.setBracketTransform(defaults.bool(forKey: "vnchkey.bracket"))
    }

    func setupStatusBar() {
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        updateStatusButton()
        rebuildMenu()
    }

    func rebuildMenu() {
        guard let km = keyManager else { return }
        let menu = NSMenu()

        // 1. Bật / Tắt Tiếng Việt
        let toggleItem = NSMenuItem(title: "Bật gõ Tiếng Việt", action: #selector(toggleEnabled(_:)), keyEquivalent: "v")
        toggleItem.state = km.isEnabled ? .on : .off
        menu.addItem(toggleItem)
        menu.addItem(NSMenuItem.separator())

        // 2. Chọn Kiểu Gõ
        let methodMenu = NSMenu()
        let currentMethod = km.getMethod()

        let autoItem = NSMenuItem(title: "Tự động (Telex + VNI)", action: #selector(selectMethod(_:)), keyEquivalent: "")
        autoItem.tag = 0
        autoItem.state = (currentMethod == 0) ? .on : .off

        let telexItem = NSMenuItem(title: "Telex", action: #selector(selectMethod(_:)), keyEquivalent: "")
        telexItem.tag = 1
        telexItem.state = (currentMethod == 1) ? .on : .off

        let vniItem = NSMenuItem(title: "VNI", action: #selector(selectMethod(_:)), keyEquivalent: "")
        vniItem.tag = 2
        vniItem.state = (currentMethod == 2) ? .on : .off

        let simpleTelexItem = NSMenuItem(title: "Simple Telex", action: #selector(selectMethod(_:)), keyEquivalent: "")
        simpleTelexItem.tag = 3
        simpleTelexItem.state = (currentMethod == 3) ? .on : .off

        methodMenu.addItem(autoItem)
        methodMenu.addItem(telexItem)
        methodMenu.addItem(vniItem)
        methodMenu.addItem(simpleTelexItem)

        let methodParent = NSMenuItem(title: "Kiểu gõ", action: nil, keyEquivalent: "")
        methodParent.submenu = methodMenu
        menu.addItem(methodParent)

        // 3. Bảng Mã & Quy Tắc Dấu
        let styleMenu = NSMenu()
        let toneStyle = km.getToneStyle()
        let modernTone = NSMenuItem(title: "Bỏ dấu kiểu mới (hoà, uỷ)", action: #selector(selectToneStyle(_:)), keyEquivalent: "")
        modernTone.tag = 0
        modernTone.state = (toneStyle == 0) ? .on : .off

        let classicTone = NSMenuItem(title: "Bỏ dấu kiểu cũ (hòa, ủy)", action: #selector(selectToneStyle(_:)), keyEquivalent: "")
        classicTone.tag = 1
        classicTone.state = (toneStyle == 1) ? .on : .off

        styleMenu.addItem(modernTone)
        styleMenu.addItem(classicTone)
        styleMenu.addItem(NSMenuItem.separator())

        let currentCharset = defaults.integer(forKey: "vnchkey.charset")
        let nfcItem = NSMenuItem(title: "Bảng mã: Unicode dựng sẵn (NFC)", action: #selector(selectCharset(_:)), keyEquivalent: "")
        nfcItem.tag = 0
        nfcItem.state = (currentCharset == 0) ? .on : .off

        let nfdItem = NSMenuItem(title: "Bảng mã: Unicode tổ hợp (NFD)", action: #selector(selectCharset(_:)), keyEquivalent: "")
        nfdItem.tag = 1
        nfdItem.state = (currentCharset == 1) ? .on : .off

        styleMenu.addItem(nfcItem)
        styleMenu.addItem(nfdItem)

        let styleParent = NSMenuItem(title: "Quy tắc dấu & Bảng mã", action: nil, keyEquivalent: "")
        styleParent.submenu = styleMenu
        menu.addItem(styleParent)

        menu.addItem(NSMenuItem.separator())

        // 4. Các tuỳ chọn Thông Minh (Chánh Tả, Khôi phục Tiếng Anh, Esc, Gõ tắt...)
        let spellItem = NSMenuItem(title: "Kiểm tra Chánh Tả", action: #selector(toggleSpellCheck(_:)), keyEquivalent: "")
        spellItem.state = km.getSpellCheck() ? .on : .off
        menu.addItem(spellItem)

        let restoreItem = NSMenuItem(title: "Tự động khôi phục Tiếng Anh", action: #selector(toggleAutoRestore(_:)), keyEquivalent: "")
        restoreItem.state = km.getAutoRestore() ? .on : .off
        menu.addItem(restoreItem)

        let escItem = NSMenuItem(title: "Hoàn nguyên từ bằng phím Esc", action: #selector(toggleEscRestore(_:)), keyEquivalent: "")
        escItem.state = km.escRestoreEnabled ? .on : .off
        menu.addItem(escItem)

        let bracketItem = NSMenuItem(title: "Gõ [ ] thành ơ ư", action: #selector(toggleBracket(_:)), keyEquivalent: "")
        bracketItem.state = defaults.bool(forKey: "vnchkey.bracket") ? .on : .off
        menu.addItem(bracketItem)

        menu.addItem(NSMenuItem.separator())

        // 5. Quản lý Gõ Tắt (Macros) & Từ Điển Cá Nhân (Custom Dictionary) - Kiểu xKey trực tiếp trên Menu
        let macroToggleItem = NSMenuItem(title: "Bật Gõ Tắt (Macro)", action: #selector(toggleMacro(_:)), keyEquivalent: "")
        macroToggleItem.state = km.getMacroEnabled() ? .on : .off
        menu.addItem(macroToggleItem)

        let addMacroItem = NSMenuItem(title: "➕ Thêm Gõ Tắt mới...", action: #selector(promptAddMacro), keyEquivalent: "m")
        menu.addItem(addMacroItem)

        let macrosList = km.readMacrosList()
        let macroSubmenu = NSMenu()
        if macrosList.isEmpty {
            let emptyItem = NSMenuItem(title: "(Chưa có mục Gõ Tắt nào)", action: nil, keyEquivalent: "")
            emptyItem.isEnabled = false
            macroSubmenu.addItem(emptyItem)
        } else {
            let hintItem = NSMenuItem(title: "Bấm vào mục bất kỳ để Sửa hoặc Xoá:", action: nil, keyEquivalent: "")
            hintItem.isEnabled = false
            macroSubmenu.addItem(hintItem)
            macroSubmenu.addItem(NSMenuItem.separator())
            for (trigger, expansion) in macrosList {
                let item = NSMenuItem(title: "\(trigger)  →  \(expansion)", action: #selector(editOrRemoveMacroItem(_:)), keyEquivalent: "")
                item.representedObject = trigger
                macroSubmenu.addItem(item)
            }
        }
        macroSubmenu.addItem(NSMenuItem.separator())
        macroSubmenu.addItem(NSMenuItem(title: "Mở tập tin macros.conf bằng TextEdit...", action: #selector(openMacrosFile), keyEquivalent: ""))

        let macroListParent = NSMenuItem(title: "Danh sách Gõ Tắt (\(macrosList.count))", action: nil, keyEquivalent: "")
        macroListParent.submenu = macroSubmenu
        menu.addItem(macroListParent)

        menu.addItem(NSMenuItem.separator())

        let dictToggleItem = NSMenuItem(title: "Bật Từ Điển Cá Nhân", action: #selector(toggleDictionary(_:)), keyEquivalent: "")
        dictToggleItem.state = km.getDictionaryEnabled() ? .on : .off
        menu.addItem(dictToggleItem)

        let addDictItem = NSMenuItem(title: "➕ Thêm từ mới vô Từ Điển...", action: #selector(promptAddDictionaryWord), keyEquivalent: "d")
        menu.addItem(addDictItem)

        let dictWords = km.readDictionaryWords()
        let dictSubmenu = NSMenu()
        if dictWords.isEmpty {
            let emptyItem = NSMenuItem(title: "(Chưa có từ nào trong Từ Điển)", action: nil, keyEquivalent: "")
            emptyItem.isEnabled = false
            dictSubmenu.addItem(emptyItem)
        } else {
            let hintItem = NSMenuItem(title: "Bấm vào từ bất kỳ để Xoá khỏi Từ Điển:", action: nil, keyEquivalent: "")
            hintItem.isEnabled = false
            dictSubmenu.addItem(hintItem)
            dictSubmenu.addItem(NSMenuItem.separator())
            for word in dictWords {
                let item = NSMenuItem(title: word, action: #selector(removeDictionaryWordItem(_:)), keyEquivalent: "")
                item.representedObject = word
                dictSubmenu.addItem(item)
            }
        }
        dictSubmenu.addItem(NSMenuItem.separator())
        dictSubmenu.addItem(NSMenuItem(title: "Mở tập tin dictionary.conf bằng TextEdit...", action: #selector(openDictionaryFile), keyEquivalent: ""))

        let dictListParent = NSMenuItem(title: "Danh sách Từ Điển Cá Nhân (\(dictWords.count))", action: nil, keyEquivalent: "")
        dictListParent.submenu = dictSubmenu
        menu.addItem(dictListParent)

        let reloadFilesItem = NSMenuItem(title: "Nạp lại Gõ Tắt & Từ Điển", action: #selector(reloadConfigFiles), keyEquivalent: "r")
        menu.addItem(reloadFilesItem)

        menu.addItem(NSMenuItem.separator())
        menu.addItem(NSMenuItem(title: "Kiểm tra quyền Trợ năng (Accessibility)", action: #selector(checkPermissionAction), keyEquivalent: ""))
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

    // MARK: - Actions

    @objc func toggleEnabled(_ sender: NSMenuItem) {
        guard let km = keyManager else { return }
        km.isEnabled.toggle()
        updateStatusButton()
        rebuildMenu()
    }

    @objc func selectMethod(_ sender: NSMenuItem) {
        let method = UInt32(sender.tag)
        defaults.set(Int(method), forKey: "vnchkey.method")
        keyManager?.setMethod(method)
        rebuildMenu()
    }

    @objc func selectToneStyle(_ sender: NSMenuItem) {
        let style = UInt32(sender.tag)
        defaults.set(Int(style), forKey: "vnchkey.toneStyle")
        keyManager?.setToneStyle(style)
        rebuildMenu()
    }

    @objc func selectCharset(_ sender: NSMenuItem) {
        let charset = UInt32(sender.tag)
        defaults.set(Int(charset), forKey: "vnchkey.charset")
        keyManager?.setCharset(charset)
        rebuildMenu()
    }

    @objc func toggleSpellCheck(_ sender: NSMenuItem) {
        guard let km = keyManager else { return }
        let next = !km.getSpellCheck()
        defaults.set(next, forKey: "vnchkey.spellCheck")
        km.setSpellCheck(next)
        rebuildMenu()
    }

    @objc func toggleAutoRestore(_ sender: NSMenuItem) {
        guard let km = keyManager else { return }
        let next = !km.getAutoRestore()
        defaults.set(next, forKey: "vnchkey.autoRestore")
        km.setAutoRestore(next)
        rebuildMenu()
    }

    @objc func toggleEscRestore(_ sender: NSMenuItem) {
        guard let km = keyManager else { return }
        km.escRestoreEnabled.toggle()
        defaults.set(km.escRestoreEnabled, forKey: "vnchkey.escRestore")
        rebuildMenu()
    }

    @objc func toggleBracket(_ sender: NSMenuItem) {
        let next = !defaults.bool(forKey: "vnchkey.bracket")
        defaults.set(next, forKey: "vnchkey.bracket")
        keyManager?.setBracketTransform(next)
        rebuildMenu()
    }

    @objc func toggleMacro(_ sender: NSMenuItem) {
        guard let km = keyManager else { return }
        let next = !km.getMacroEnabled()
        defaults.set(next, forKey: "vnchkey.macro")
        km.setMacroEnabled(next)
        rebuildMenu()
    }

    @objc func toggleDictionary(_ sender: NSMenuItem) {
        guard let km = keyManager else { return }
        let next = !km.getDictionaryEnabled()
        defaults.set(next, forKey: "vnchkey.dictionary")
        km.setDictionaryEnabled(next)
        rebuildMenu()
    }

    private func openConfigFileInTextEditor(_ filePath: String) {
        let task = Process()
        task.executableURL = URL(fileURLWithPath: "/usr/bin/open")
        task.arguments = ["-t", filePath] // "-t" buộc macOS mở bằng TextEdit hoặc trình sửa văn bản mặc định
        do {
            try task.run()
        } catch {
            let alert = NSAlert()
            alert.messageText = "Không thể mở tập tin cấu hình"
            alert.informativeText = "Đường dẫn: \(filePath)\nLỗi: \(error.localizedDescription)"
            alert.alertStyle = .warning
            alert.runModal()
        }
    }

    @objc func openMacrosFile() {
        guard let km = keyManager else { return }
        km.ensureDefaultConfigFiles()
        openConfigFileInTextEditor(km.macrosFilePath())
    }

    @objc func openDictionaryFile() {
        guard let km = keyManager else { return }
        km.ensureDefaultConfigFiles()
        openConfigFileInTextEditor(km.dictionaryFilePath())
    }

    @objc func promptAddMacro() {
        showMacroDialog(initialTrigger: "", initialExpansion: "", isEditing: false)
    }

    @objc func editOrRemoveMacroItem(_ sender: NSMenuItem) {
        guard let km = keyManager, let trigger = sender.representedObject as? String else { return }
        let currentExpansion = km.readMacrosList().first(where: { $0.0.lowercased() == trigger.lowercased() })?.1 ?? ""
        showMacroDialog(initialTrigger: trigger, initialExpansion: currentExpansion, isEditing: true)
    }

    private func showMacroDialog(initialTrigger: String, initialExpansion: String, isEditing: Bool) {
        guard let km = keyManager else { return }
        NSApp.activate(ignoringOtherApps: true)

        let alert = NSAlert()
        alert.messageText = isEditing ? "Chỉnh sửa mục Gõ Tắt" : "Thêm mục Gõ Tắt mới"
        alert.informativeText = "Nhập từ viết tắt và cụm từ đầy đủ (hỗ trợ $DATE, $TIME, $YEAR, $NOW, $DATETIME):"
        alert.alertStyle = .informational
        if let appIconPath = Bundle.main.path(forResource: "AppIcon", ofType: "png"),
           let iconImg = NSImage(contentsOfFile: appIconPath) {
            alert.icon = iconImg
        }

        let container = NSView(frame: NSRect(x: 0, y: 0, width: 340, height: 112))

        let triggerLabel = NSTextField(labelWithString: "Từ viết tắt (ví dụ: vnch, sg):")
        triggerLabel.frame = NSRect(x: 0, y: 90, width: 340, height: 18)
        container.addSubview(triggerLabel)

        let triggerField = NSTextField(frame: NSRect(x: 0, y: 62, width: 340, height: 24))
        triggerField.placeholderString = "Ví dụ: vnch"
        triggerField.stringValue = initialTrigger
        container.addSubview(triggerField)

        let expansionLabel = NSTextField(labelWithString: "Cụm từ thay thế:")
        expansionLabel.frame = NSRect(x: 0, y: 34, width: 340, height: 18)
        container.addSubview(expansionLabel)

        let expansionField = NSTextField(frame: NSRect(x: 0, y: 6, width: 340, height: 24))
        expansionField.placeholderString = "Ví dụ: Việt Nam Cộng Hòa"
        expansionField.stringValue = initialExpansion
        container.addSubview(expansionField)

        alert.accessoryView = container
        alert.addButton(withTitle: isEditing ? "Lưu thay đổi" : "Thêm mới")
        alert.addButton(withTitle: "Huỷ")
        if isEditing {
            alert.addButton(withTitle: "Xoá mục này")
        }

        alert.window.initialFirstResponder = isEditing ? expansionField : triggerField
        let response = alert.runModal()

        if response == .alertFirstButtonReturn {
            let newTrigger = triggerField.stringValue.trimmingCharacters(in: .whitespaces)
            let newExpansion = expansionField.stringValue.trimmingCharacters(in: .whitespaces)
            if !newTrigger.isEmpty && !newExpansion.isEmpty {
                if isEditing && newTrigger.lowercased() != initialTrigger.lowercased() {
                    km.removeMacro(trigger: initialTrigger)
                }
                km.addOrUpdateMacro(trigger: newTrigger, expansion: newExpansion)
                rebuildMenu()
            }
        } else if isEditing && response == .alertThirdButtonReturn {
            km.removeMacro(trigger: initialTrigger)
            rebuildMenu()
        }
    }

    @objc func promptAddDictionaryWord() {
        guard let km = keyManager else { return }
        NSApp.activate(ignoringOtherApps: true)

        let alert = NSAlert()
        alert.messageText = "Thêm từ mới vô Từ Điển Cá Nhân"
        alert.informativeText = "Nhập từ tiếng Anh hoặc thuật ngữ riêng mà bạn muốn vnchKey luôn giữ nguyên (không bỏ dấu Tiếng Việt):"
        alert.alertStyle = .informational
        if let appIconPath = Bundle.main.path(forResource: "AppIcon", ofType: "png"),
           let iconImg = NSImage(contentsOfFile: appIconPath) {
            alert.icon = iconImg
        }

        let wordField = NSTextField(frame: NSRect(x: 0, y: 0, width: 320, height: 24))
        wordField.placeholderString = "Ví dụ: macbook, kubernetes, tailwind..."
        alert.accessoryView = wordField
        alert.addButton(withTitle: "Thêm vô Từ Điển")
        alert.addButton(withTitle: "Huỷ")

        alert.window.initialFirstResponder = wordField
        if alert.runModal() == .alertFirstButtonReturn {
            let word = wordField.stringValue.trimmingCharacters(in: .whitespaces)
            if !word.isEmpty {
                km.addDictionaryWord(word)
                rebuildMenu()
            }
        }
    }

    @objc func removeDictionaryWordItem(_ sender: NSMenuItem) {
        guard let km = keyManager, let word = sender.representedObject as? String else { return }
        NSApp.activate(ignoringOtherApps: true)

        let alert = NSAlert()
        alert.messageText = "Xoá từ khỏi Từ Điển Cá Nhân?"
        alert.informativeText = "Bạn có muốn xoá từ \"\(word)\" khỏi danh sách từ điển riêng không?"
        alert.alertStyle = .warning
        alert.addButton(withTitle: "Xoá")
        alert.addButton(withTitle: "Huỷ")

        if alert.runModal() == .alertFirstButtonReturn {
            km.removeDictionaryWord(word)
            rebuildMenu()
        }
    }

    @objc func reloadConfigFiles() {
        keyManager?.reloadMacrosAndDictionary()
        rebuildMenu()
    }

    @objc func checkPermissionAction() {
        if !(keyManager?.checkAccessibilityPermission(prompt: true) ?? false) {
            showAccessibilityAlert()
        } else {
            let alert = NSAlert()
            alert.messageText = "Quyền Trợ năng (Accessibility) đã được cấp!"
            alert.informativeText = "vnchKey đã sẵn sàng bắt phím trên macOS."
            alert.alertStyle = .informational
            alert.runModal()
            _ = keyManager?.start()
        }
    }

    func showAccessibilityAlert() {
        let alert = NSAlert()
        alert.messageText = "vnchKey cần quyền Trợ năng (Accessibility)"
        alert.informativeText = "Vui lòng vào System Settings > Privacy & Security > Accessibility và kích hoạt cho vnchKey để bắt phím."
        alert.alertStyle = .warning
        alert.addButton(withTitle: "Mở System Settings")
        alert.addButton(withTitle: "Để sau")

        if alert.runModal() == .alertFirstButtonReturn {
            if let url = URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility") {
                NSWorkspace.shared.open(url)
            }
        }
    }

    @objc func quitApp() {
        NSApplication.shared.terminate(self)
    }
}

let app = NSApplication.shared
let delegate = AppDelegate()
app.delegate = delegate
app.run()
