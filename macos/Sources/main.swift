import Cocoa

class AppDelegate: NSObject, NSApplicationDelegate {
    var statusItem: NSStatusItem?
    var keyManager: KeyTapManager?
    var vnchIcon: NSImage?
    var dashboardWindow: NSWindow?

    // Lưu cấu hình bền vững qua UserDefaults
    private let defaults = UserDefaults.standard

    // Các control trong Bảng Điều Khiển để đồng bộ 2 chiều với Menu Bar
    private var dashEnableCheckbox: NSButton?
    private var dashMethodPopup: NSPopUpButton?
    private var dashCharsetPopup: NSPopUpButton?
    private var dashTonePopup: NSPopUpButton?
    private var dashShortcutEnableCheckbox: NSButton?
    private var dashShortcutCtrlCheck: NSButton?
    private var dashShortcutOptCheck: NSButton?
    private var dashShortcutShiftCheck: NSButton?
    private var dashShortcutCmdCheck: NSButton?
    private var dashShortcutKeyBadge: NSTextField?
    private var dashRecordShortcutBtn: NSButton?
    private var dashSpellCheckbox: NSButton?
    private var dashRestoreCheckbox: NSButton?
    private var dashEscCheckbox: NSButton?
    private var dashBracketCheckbox: NSButton?
    private var dashW2UPopup: NSPopUpButton?
    private var dashTimeoutSlider: NSSlider?
    private var dashTimeoutValueLabel: NSTextField?
    private var dashMacroCheckbox: NSButton?
    private var dashDictCheckbox: NSButton?

    func applicationDidFinishLaunching(_ notification: Notification) {
        loadIcons()
        if let appIconPath = Bundle.main.path(forResource: "AppIcon", ofType: "png"),
           let appIconImg = NSImage(contentsOfFile: appIconPath) {
            NSApp.applicationIconImage = appIconImg
        }
        keyManager = KeyTapManager(method: UInt32(defaults.integer(forKey: "vnchkey.method")))
        keyManager?.onToggleEnabledByShortcut = { [weak self] in
            self?.updateStatusButton()
            self?.rebuildMenu()
            self?.syncDashboardControls()
        }
        keyManager?.onShortcutRecorded = { [weak self] recorded in
            self?.saveCustomShortcut(recorded)
            self?.rebuildMenu()
            self?.syncDashboardControls()
        }
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
            defaults.set(0, forKey: "vnchkey.w2u")            // W -> ư (Không ở đầu từ)
            defaults.set(800, forKey: "vnchkey.idleTimeout")  // Khoảng nghỉ 800ms cho Neovim
            saveCustomShortcut(.defaultShortcut)
        }

        if defaults.object(forKey: "vnchkey.idleTimeout") == nil {
            defaults.set(800, forKey: "vnchkey.idleTimeout")
        }
        if defaults.object(forKey: "vnchkey.sc.initialized") == nil {
            saveCustomShortcut(.defaultShortcut)
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
        km.setW2U(UInt32(defaults.integer(forKey: "vnchkey.w2u")))
        km.setIdleTimeout(UInt64(defaults.integer(forKey: "vnchkey.idleTimeout")))
        km.setQuickTelex(defaults.bool(forKey: "vnchkey.quick_telex"))
        km.setQuickStartConsonant(defaults.bool(forKey: "vnchkey.quick_start"))
        km.setQuickEndConsonant(defaults.bool(forKey: "vnchkey.quick_end"))
        km.customShortcut = loadCustomShortcut()
    }

    func saveCustomShortcut(_ sc: CustomToggleShortcut) {
        defaults.set(true, forKey: "vnchkey.sc.initialized")
        defaults.set(sc.useControl, forKey: "vnchkey.sc.ctrl")
        defaults.set(sc.useOption, forKey: "vnchkey.sc.opt")
        defaults.set(sc.useShift, forKey: "vnchkey.sc.shift")
        defaults.set(sc.useCommand, forKey: "vnchkey.sc.cmd")
        defaults.set(Int(sc.keyCode), forKey: "vnchkey.sc.keyCode")
        defaults.set(sc.keyDisplay, forKey: "vnchkey.sc.keyDisplay")
        defaults.set(sc.isEnabled, forKey: "vnchkey.sc.enabled")
    }

    func loadCustomShortcut() -> CustomToggleShortcut {
        return CustomToggleShortcut(
            useControl: defaults.bool(forKey: "vnchkey.sc.ctrl"),
            useOption: defaults.bool(forKey: "vnchkey.sc.opt"),
            useShift: defaults.bool(forKey: "vnchkey.sc.shift"),
            useCommand: defaults.bool(forKey: "vnchkey.sc.cmd"),
            keyCode: Int64(defaults.integer(forKey: "vnchkey.sc.keyCode")),
            keyDisplay: defaults.string(forKey: "vnchkey.sc.keyDisplay") ?? "",
            isEnabled: defaults.bool(forKey: "vnchkey.sc.enabled")
        )
    }

    func setupStatusBar() {
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        updateStatusButton()
        rebuildMenu()
    }

    func rebuildMenu() {
        guard let km = keyManager else { return }
        let menu = NSMenu()

        // 1. Bật / Tắt Tiếng Việt (hiển thị kèm tổ hợp phím tắt tự do của người dùng)
        let badge = km.customShortcut.shortBadge
        let toggleTitle = badge.isEmpty ? "Bật gõ Tiếng Việt" : "Bật gõ Tiếng Việt  (\(badge))"
        let toggleItem = NSMenuItem(title: toggleTitle, action: #selector(toggleEnabled(_:)), keyEquivalent: "")
        toggleItem.state = km.isEnabled ? .on : .off
        menu.addItem(toggleItem)

        // Mở nhanh Bảng điều khiển... (kiểu xKey / OpenKey)
        let dashItem = NSMenuItem(title: "Bảng điều khiển...", action: #selector(openDashboardWindow), keyEquivalent: ",")
        menu.addItem(dashItem)

        // Phím tắt chuyển chế độ Việt / Anh tự do
        let shortcutMenu = NSMenu()
        let currentDesc = NSMenuItem(title: "Hiện tại: \(km.customShortcut.fullDescription)", action: nil, keyEquivalent: "")
        currentDesc.isEnabled = false
        shortcutMenu.addItem(currentDesc)
        shortcutMenu.addItem(NSMenuItem.separator())

        let recordItem = NSMenuItem(title: "⌨️ Tự bấm phím tắt theo ý muốn...", action: #selector(openDashboardAndRecordShortcut), keyEquivalent: "")
        shortcutMenu.addItem(recordItem)

        let toggleScEnableItem = NSMenuItem(
            title: km.customShortcut.isEnabled ? "Tắt phím tắt chuyển nhanh" : "Bật lại phím tắt chuyển nhanh",
            action: #selector(toggleShortcutEnabledFromMenu),
            keyEquivalent: ""
        )
        shortcutMenu.addItem(toggleScEnableItem)

        let shortcutParent = NSMenuItem(title: "Phím tắt chuyển Việt / Anh", action: nil, keyEquivalent: "")
        shortcutParent.submenu = shortcutMenu
        menu.addItem(shortcutParent)

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

        // Menu con: Tùy chọn Tốc ký (Quick Consonants)
        let quickMenu = NSMenu()
        let quickTelexItem = NSMenuItem(title: "Quick Telex (cc->ch, kk->kh, tt->th...)", action: #selector(toggleQuickTelex(_:)), keyEquivalent: "")
        quickTelexItem.state = defaults.bool(forKey: "vnchkey.quick_telex") ? .on : .off
        quickMenu.addItem(quickTelexItem)

        let quickStartItem = NSMenuItem(title: "Phụ âm đầu nhanh (f->ph, j->gi, w->qu)", action: #selector(toggleQuickStart(_:)), keyEquivalent: "")
        quickStartItem.state = defaults.bool(forKey: "vnchkey.quick_start") ? .on : .off
        quickMenu.addItem(quickStartItem)

        let quickEndItem = NSMenuItem(title: "Phụ âm cuối nhanh (g->ng, h->nh, k->ch)", action: #selector(toggleQuickEnd(_:)), keyEquivalent: "")
        quickEndItem.state = defaults.bool(forKey: "vnchkey.quick_end") ? .on : .off
        quickMenu.addItem(quickEndItem)

        let quickParent = NSMenuItem(title: "Tùy chọn Tốc ký...", action: nil, keyEquivalent: "")
        quickParent.submenu = quickMenu
        menu.addItem(quickParent)

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
        syncDashboardControls()
    }

    @objc func toggleShortcutEnabledFromMenu() {
        guard let km = keyManager else { return }
        km.customShortcut.isEnabled.toggle()
        saveCustomShortcut(km.customShortcut)
        rebuildMenu()
        syncDashboardControls()
    }

    @objc func openDashboardAndRecordShortcut() {
        openDashboardWindow()
        startRecordingShortcutAction()
    }

    @objc func toggleQuickTelex(_ sender: NSMenuItem) {
        guard let km = keyManager else { return }
        let next = !defaults.bool(forKey: "vnchkey.quick_telex")
        defaults.set(next, forKey: "vnchkey.quick_telex")
        km.setQuickTelex(next)
        rebuildMenu()
    }

    @objc func toggleQuickStart(_ sender: NSMenuItem) {
        guard let km = keyManager else { return }
        let next = !defaults.bool(forKey: "vnchkey.quick_start")
        defaults.set(next, forKey: "vnchkey.quick_start")
        km.setQuickStartConsonant(next)
        rebuildMenu()
    }

    @objc func toggleQuickEnd(_ sender: NSMenuItem) {
        guard let km = keyManager else { return }
        let next = !defaults.bool(forKey: "vnchkey.quick_end")
        defaults.set(next, forKey: "vnchkey.quick_end")
        km.setQuickEndConsonant(next)
        rebuildMenu()
    }

    @objc func selectMethod(_ sender: NSMenuItem) {
        let method = UInt32(sender.tag)
        defaults.set(Int(method), forKey: "vnchkey.method")
        keyManager?.setMethod(method)
        rebuildMenu()
        syncDashboardControls()
    }

    @objc func selectToneStyle(_ sender: NSMenuItem) {
        let style = UInt32(sender.tag)
        defaults.set(Int(style), forKey: "vnchkey.toneStyle")
        keyManager?.setToneStyle(style)
        rebuildMenu()
        syncDashboardControls()
    }

    @objc func selectCharset(_ sender: NSMenuItem) {
        let charset = UInt32(sender.tag)
        defaults.set(Int(charset), forKey: "vnchkey.charset")
        keyManager?.setCharset(charset)
        rebuildMenu()
        syncDashboardControls()
    }

    @objc func toggleSpellCheck(_ sender: NSMenuItem) {
        guard let km = keyManager else { return }
        let next = !km.getSpellCheck()
        defaults.set(next, forKey: "vnchkey.spellCheck")
        km.setSpellCheck(next)
        rebuildMenu()
        syncDashboardControls()
    }

    @objc func toggleAutoRestore(_ sender: NSMenuItem) {
        guard let km = keyManager else { return }
        let next = !km.getAutoRestore()
        defaults.set(next, forKey: "vnchkey.autoRestore")
        km.setAutoRestore(next)
        rebuildMenu()
        syncDashboardControls()
    }

    @objc func toggleEscRestore(_ sender: NSMenuItem) {
        guard let km = keyManager else { return }
        km.escRestoreEnabled.toggle()
        defaults.set(km.escRestoreEnabled, forKey: "vnchkey.escRestore")
        rebuildMenu()
        syncDashboardControls()
    }

    @objc func toggleBracket(_ sender: NSMenuItem) {
        let next = !defaults.bool(forKey: "vnchkey.bracket")
        defaults.set(next, forKey: "vnchkey.bracket")
        keyManager?.setBracketTransform(next)
        rebuildMenu()
        syncDashboardControls()
    }

    @objc func toggleMacro(_ sender: NSMenuItem) {
        guard let km = keyManager else { return }
        let next = !km.getMacroEnabled()
        defaults.set(next, forKey: "vnchkey.macro")
        km.setMacroEnabled(next)
        rebuildMenu()
        syncDashboardControls()
    }

    @objc func toggleDictionary(_ sender: NSMenuItem) {
        guard let km = keyManager else { return }
        let next = !km.getDictionaryEnabled()
        defaults.set(next, forKey: "vnchkey.dictionary")
        km.setDictionaryEnabled(next)
        rebuildMenu()
        syncDashboardControls()
    }

    // MARK: - Bảng Điều Khiển (Dashboard Window kiểu xKey / OpenKey)

    @objc func openDashboardWindow() {
        NSApp.activate(ignoringOtherApps: true)
        if let win = dashboardWindow {
            syncDashboardControls()
            win.makeKeyAndOrderFront(nil)
            return
        }

        let winRect = NSRect(x: 0, y: 0, width: 520, height: 610)
        let win = NSWindow(
            contentRect: winRect,
            styleMask: [.titled, .closable, .miniaturizable],
            backing: .buffered,
            defer: false
        )
        win.title = "Bảng Điều Khiển vnchKey (Tự Do Edition)"
        win.center()
        win.isReleasedWhenClosed = false

        guard let content = win.contentView else { return }

        // Header: Logo + Tiêu đề
        if let appIconPath = Bundle.main.path(forResource: "AppIcon", ofType: "png"),
           let iconImg = NSImage(contentsOfFile: appIconPath) {
            let imageView = NSImageView(frame: NSRect(x: 24, y: 542, width: 48, height: 48))
            imageView.image = iconImg
            content.addSubview(imageView)
        }

        let titleLabel = NSTextField(labelWithString: "vnchKey cho macOS — Bảng Điều Khiển")
        titleLabel.font = NSFont.boldSystemFont(ofSize: 16)
        titleLabel.frame = NSRect(x: 84, y: 566, width: 410, height: 22)
        content.addSubview(titleLabel)

        let subtitleLabel = NSTextField(labelWithString: "Bộ gõ Tiếng Việt hiệu năng cao • Tự do tuỳ biến phím tắt & tính năng")
        subtitleLabel.font = NSFont.systemFont(ofSize: 12)
        subtitleLabel.textColor = .secondaryLabelColor
        subtitleLabel.frame = NSRect(x: 84, y: 544, width: 410, height: 18)
        content.addSubview(subtitleLabel)

        // Khung 1: Chế độ gõ & Phím tắt tự do (kiểu xKey / OpenKey)
        let generalBox = NSBox(frame: NSRect(x: 20, y: 318, width: 480, height: 214))
        generalBox.title = "Thiết lập Kiểu gõ & Phím tắt chuyển Việt / Anh tự do"
        content.addSubview(generalBox)

        let enableCheck = NSButton(checkboxWithTitle: "Bật chế độ gõ Tiếng Việt", target: self, action: #selector(dashToggleEnabled(_:)))
        enableCheck.frame = NSRect(x: 16, y: 162, width: 240, height: 22)
        generalBox.addSubview(enableCheck)
        self.dashEnableCheckbox = enableCheck

        let methodLabel = NSTextField(labelWithString: "Kiểu gõ:")
        methodLabel.frame = NSRect(x: 16, y: 130, width: 150, height: 20)
        generalBox.addSubview(methodLabel)

        let methodPopup = NSPopUpButton(frame: NSRect(x: 175, y: 126, width: 280, height: 26), pullsDown: false)
        methodPopup.addItems(withTitles: [
            "Tự động (Telex + VNI)",
            "Telex",
            "VNI",
            "Simple Telex"
        ])
        methodPopup.target = self
        methodPopup.action = #selector(dashMethodChanged(_:))
        generalBox.addSubview(methodPopup)
        self.dashMethodPopup = methodPopup

        let charsetLabel = NSTextField(labelWithString: "Bảng mã & Quy tắc dấu:")
        charsetLabel.frame = NSRect(x: 16, y: 96, width: 150, height: 20)
        generalBox.addSubview(charsetLabel)

        let charsetPopup = NSPopUpButton(frame: NSRect(x: 175, y: 92, width: 145, height: 26), pullsDown: false)
        charsetPopup.addItems(withTitles: ["Unicode (NFC)", "Tổ hợp (NFD)"])
        charsetPopup.target = self
        charsetPopup.action = #selector(dashCharsetChanged(_:))
        generalBox.addSubview(charsetPopup)
        self.dashCharsetPopup = charsetPopup

        let tonePopup = NSPopUpButton(frame: NSRect(x: 325, y: 92, width: 130, height: 26), pullsDown: false)
        tonePopup.addItems(withTitles: ["Dấu mới (hoà)", "Dấu cũ (hòa)"])
        tonePopup.target = self
        tonePopup.action = #selector(dashToneChanged(_:))
        generalBox.addSubview(tonePopup)
        self.dashTonePopup = tonePopup

        // Hàng tự tích chọn Modifier + Phím bất kỳ (như xKey / OpenKey)
        let scEnableCheck = NSButton(checkboxWithTitle: "Phím tắt chuyển:", target: self, action: #selector(dashShortcutComponentsChanged(_:)))
        scEnableCheck.frame = NSRect(x: 16, y: 54, width: 135, height: 22)
        generalBox.addSubview(scEnableCheck)
        self.dashShortcutEnableCheckbox = scEnableCheck

        let ctrlCheck = NSButton(checkboxWithTitle: "⌃ Ctrl", target: self, action: #selector(dashShortcutComponentsChanged(_:)))
        ctrlCheck.frame = NSRect(x: 152, y: 54, width: 68, height: 22)
        generalBox.addSubview(ctrlCheck)
        self.dashShortcutCtrlCheck = ctrlCheck

        let optCheck = NSButton(checkboxWithTitle: "⌥ Opt", target: self, action: #selector(dashShortcutComponentsChanged(_:)))
        optCheck.frame = NSRect(x: 222, y: 54, width: 68, height: 22)
        generalBox.addSubview(optCheck)
        self.dashShortcutOptCheck = optCheck

        let shiftCheck = NSButton(checkboxWithTitle: "⇧ Shift", target: self, action: #selector(dashShortcutComponentsChanged(_:)))
        shiftCheck.frame = NSRect(x: 292, y: 54, width: 72, height: 22)
        generalBox.addSubview(shiftCheck)
        self.dashShortcutShiftCheck = shiftCheck

        let cmdCheck = NSButton(checkboxWithTitle: "⌘ Cmd", target: self, action: #selector(dashShortcutComponentsChanged(_:)))
        cmdCheck.frame = NSRect(x: 368, y: 54, width: 78, height: 22)
        generalBox.addSubview(cmdCheck)
        self.dashShortcutCmdCheck = cmdCheck

        let currentKeyTitle = NSTextField(labelWithString: "Tổ hợp đang dùng:")
        currentKeyTitle.frame = NSRect(x: 16, y: 14, width: 125, height: 20)
        generalBox.addSubview(currentKeyTitle)

        let keyBadge = NSTextField(labelWithString: "⌃ Control + ⇧ Shift")
        keyBadge.font = NSFont.boldSystemFont(ofSize: 12)
        keyBadge.textColor = .systemBlue
        keyBadge.frame = NSRect(x: 142, y: 14, width: 158, height: 20)
        generalBox.addSubview(keyBadge)
        self.dashShortcutKeyBadge = keyBadge

        let recordBtn = NSButton(title: "⌨️ Tự bấm phím...", target: self, action: #selector(startRecordingShortcutAction))
        recordBtn.bezelStyle = .rounded
        recordBtn.frame = NSRect(x: 302, y: 10, width: 155, height: 28)
        generalBox.addSubview(recordBtn)
        self.dashRecordShortcutBtn = recordBtn

        // Khung 2: Tính năng thông minh & Tối ưu Neovim
        let smartBox = NSBox(frame: NSRect(x: 20, y: 144, width: 480, height: 166))
        smartBox.title = "Tính năng thông minh & Tối ưu Neovim / Terminal"
        content.addSubview(smartBox)

        let spellCheck = NSButton(checkboxWithTitle: "Kiểm tra Chánh Tả Tiếng Việt", target: self, action: #selector(dashSpellChanged(_:)))
        spellCheck.frame = NSRect(x: 16, y: 114, width: 220, height: 22)
        smartBox.addSubview(spellCheck)
        self.dashSpellCheckbox = spellCheck

        let restoreCheck = NSButton(checkboxWithTitle: "Tự động khôi phục từ Tiếng Anh", target: self, action: #selector(dashRestoreChanged(_:)))
        restoreCheck.frame = NSRect(x: 245, y: 114, width: 220, height: 22)
        smartBox.addSubview(restoreCheck)
        self.dashRestoreCheckbox = restoreCheck

        let escCheck = NSButton(checkboxWithTitle: "Hoàn nguyên từ bằng phím Esc", target: self, action: #selector(dashEscChanged(_:)))
        escCheck.frame = NSRect(x: 16, y: 86, width: 220, height: 22)
        smartBox.addSubview(escCheck)
        self.dashEscCheckbox = escCheck

        let bracketCheck = NSButton(checkboxWithTitle: "Gõ [ ] thành ơ ư", target: self, action: #selector(dashBracketChanged(_:)))
        bracketCheck.frame = NSRect(x: 245, y: 86, width: 220, height: 22)
        smartBox.addSubview(bracketCheck)
        self.dashBracketCheckbox = bracketCheck

        let w2uLabel = NSTextField(labelWithString: "Quy tắc gõ phím W → ư:")
        w2uLabel.frame = NSRect(x: 16, y: 50, width: 165, height: 20)
        smartBox.addSubview(w2uLabel)

        let w2uPopup = NSPopUpButton(frame: NSRect(x: 185, y: 46, width: 270, height: 26), pullsDown: false)
        w2uPopup.addItems(withTitles: [
            "Không đổi ở đầu từ (Khuyên dùng)",
            "Luôn đổi W thành ư",
            "Tắt đổi W thành ư"
        ])
        w2uPopup.target = self
        w2uPopup.action = #selector(dashW2UChanged(_:))
        smartBox.addSubview(w2uPopup)
        self.dashW2UPopup = w2uPopup

        let timeoutLabel = NSTextField(labelWithString: "Ngắt bộ đệm Neovim:")
        timeoutLabel.frame = NSRect(x: 16, y: 14, width: 150, height: 20)
        smartBox.addSubview(timeoutLabel)

        let timeoutSlider = NSSlider(value: 800, minValue: 0, maxValue: 2000, target: self, action: #selector(dashTimeoutChanged(_:)))
        timeoutSlider.frame = NSRect(x: 185, y: 14, width: 195, height: 20)
        smartBox.addSubview(timeoutSlider)
        self.dashTimeoutSlider = timeoutSlider

        let timeoutValLabel = NSTextField(labelWithString: "800 ms")
        timeoutValLabel.frame = NSRect(x: 388, y: 14, width: 70, height: 20)
        smartBox.addSubview(timeoutValLabel)
        self.dashTimeoutValueLabel = timeoutValLabel

        // Khung 3: Quản lý Gõ Tắt & Từ Điển Cá Nhân
        let dataBox = NSBox(frame: NSRect(x: 20, y: 52, width: 480, height: 84))
        dataBox.title = "Gõ Tắt (Macros) & Từ Điển Cá Nhân"
        content.addSubview(dataBox)

        let macroCheck = NSButton(checkboxWithTitle: "Bật Gõ Tắt (Macro)", target: self, action: #selector(dashMacroChanged(_:)))
        macroCheck.frame = NSRect(x: 16, y: 36, width: 155, height: 22)
        dataBox.addSubview(macroCheck)
        self.dashMacroCheckbox = macroCheck

        let addMacroBtn = NSButton(title: "➕ Thêm Gõ Tắt", target: self, action: #selector(promptAddMacro))
        addMacroBtn.bezelStyle = .rounded
        addMacroBtn.frame = NSRect(x: 180, y: 32, width: 140, height: 28)
        dataBox.addSubview(addMacroBtn)

        let openMacroBtn = NSButton(title: "Mở macros.conf", target: self, action: #selector(openMacrosFile))
        openMacroBtn.bezelStyle = .rounded
        openMacroBtn.frame = NSRect(x: 326, y: 32, width: 135, height: 28)
        dataBox.addSubview(openMacroBtn)

        let dictCheck = NSButton(checkboxWithTitle: "Bật Từ Điển Riêng", target: self, action: #selector(dashDictChanged(_:)))
        dictCheck.frame = NSRect(x: 16, y: 6, width: 155, height: 22)
        dataBox.addSubview(dictCheck)
        self.dashDictCheckbox = dictCheck

        let addDictBtn = NSButton(title: "➕ Thêm Từ Điển", target: self, action: #selector(promptAddDictionaryWord))
        addDictBtn.bezelStyle = .rounded
        addDictBtn.frame = NSRect(x: 180, y: 2, width: 140, height: 28)
        dataBox.addSubview(addDictBtn)

        let openDictBtn = NSButton(title: "Mở dictionary.conf", target: self, action: #selector(openDictionaryFile))
        openDictBtn.bezelStyle = .rounded
        openDictBtn.frame = NSRect(x: 326, y: 2, width: 135, height: 28)
        dataBox.addSubview(openDictBtn)

        // Thanh nút dưới cùng
        let resetBtn = NSButton(title: "Khôi phục mặc định", target: self, action: #selector(dashResetDefaults))
        resetBtn.bezelStyle = .rounded
        resetBtn.frame = NSRect(x: 20, y: 12, width: 155, height: 30)
        content.addSubview(resetBtn)

        let closeBtn = NSButton(title: "Đóng", target: self, action: #selector(dashCloseWindow))
        closeBtn.bezelStyle = .rounded
        closeBtn.keyEquivalent = "\r"
        closeBtn.frame = NSRect(x: 400, y: 12, width: 100, height: 30)
        content.addSubview(closeBtn)

        self.dashboardWindow = win
        syncDashboardControls()
        win.makeKeyAndOrderFront(nil)
    }

    func syncDashboardControls() {
        guard let km = keyManager else { return }
        dashEnableCheckbox?.state = km.isEnabled ? .on : .off
        dashMethodPopup?.selectItem(at: Int(km.getMethod()))
        dashCharsetPopup?.selectItem(at: defaults.integer(forKey: "vnchkey.charset"))
        dashTonePopup?.selectItem(at: Int(km.getToneStyle()))

        let sc = km.customShortcut
        dashShortcutEnableCheckbox?.state = sc.isEnabled ? .on : .off
        dashShortcutCtrlCheck?.state = sc.useControl ? .on : .off
        dashShortcutOptCheck?.state = sc.useOption ? .on : .off
        dashShortcutShiftCheck?.state = sc.useShift ? .on : .off
        dashShortcutCmdCheck?.state = sc.useCommand ? .on : .off
        dashShortcutKeyBadge?.stringValue = sc.fullDescription
        dashRecordShortcutBtn?.title = km.isRecordingShortcut ? "🔴 Đang chờ bấm phím..." : "⌨️ Tự bấm phím..."

        dashSpellCheckbox?.state = km.getSpellCheck() ? .on : .off
        dashRestoreCheckbox?.state = km.getAutoRestore() ? .on : .off
        dashEscCheckbox?.state = km.escRestoreEnabled ? .on : .off
        dashBracketCheckbox?.state = defaults.bool(forKey: "vnchkey.bracket") ? .on : .off
        dashW2UPopup?.selectItem(at: defaults.integer(forKey: "vnchkey.w2u"))

        let timeout = defaults.integer(forKey: "vnchkey.idleTimeout")
        dashTimeoutSlider?.integerValue = timeout
        dashTimeoutValueLabel?.stringValue = (timeout == 0) ? "Tắt (0ms)" : "\(timeout) ms"

        dashMacroCheckbox?.state = km.getMacroEnabled() ? .on : .off
        dashDictCheckbox?.state = km.getDictionaryEnabled() ? .on : .off
    }

    @objc func dashToggleEnabled(_ sender: NSButton) {
        guard let km = keyManager else { return }
        km.isEnabled = (sender.state == .on)
        updateStatusButton()
        rebuildMenu()
    }

    @objc func dashMethodChanged(_ sender: NSPopUpButton) {
        let method = UInt32(sender.indexOfSelectedItem)
        defaults.set(Int(method), forKey: "vnchkey.method")
        keyManager?.setMethod(method)
        rebuildMenu()
    }

    @objc func dashCharsetChanged(_ sender: NSPopUpButton) {
        let charset = UInt32(sender.indexOfSelectedItem)
        defaults.set(Int(charset), forKey: "vnchkey.charset")
        keyManager?.setCharset(charset)
        rebuildMenu()
    }

    @objc func dashToneChanged(_ sender: NSPopUpButton) {
        let style = UInt32(sender.indexOfSelectedItem)
        defaults.set(Int(style), forKey: "vnchkey.toneStyle")
        keyManager?.setToneStyle(style)
        rebuildMenu()
    }

    @objc func dashShortcutComponentsChanged(_ sender: NSButton) {
        guard let km = keyManager else { return }
        km.stopRecordingShortcut()
        var sc = km.customShortcut
        sc.isEnabled = (dashShortcutEnableCheckbox?.state == .on)
        sc.useControl = (dashShortcutCtrlCheck?.state == .on)
        sc.useOption = (dashShortcutOptCheck?.state == .on)
        sc.useShift = (dashShortcutShiftCheck?.state == .on)
        sc.useCommand = (dashShortcutCmdCheck?.state == .on)
        km.customShortcut = sc
        saveCustomShortcut(sc)
        rebuildMenu()
        syncDashboardControls()
    }

    @objc func startRecordingShortcutAction() {
        guard let km = keyManager else { return }
        if km.isRecordingShortcut {
            km.stopRecordingShortcut()
            syncDashboardControls()
        } else {
            km.startRecordingShortcut()
            dashShortcutKeyBadge?.stringValue = "Hãy bấm tổ hợp phím bất kỳ..."
            dashRecordShortcutBtn?.title = "🔴 Đang chờ bấm phím (Esc để huỷ)"
        }
    }

    @objc func dashSpellChanged(_ sender: NSButton) {
        let enabled = (sender.state == .on)
        defaults.set(enabled, forKey: "vnchkey.spellCheck")
        keyManager?.setSpellCheck(enabled)
        rebuildMenu()
    }

    @objc func dashRestoreChanged(_ sender: NSButton) {
        let enabled = (sender.state == .on)
        defaults.set(enabled, forKey: "vnchkey.autoRestore")
        keyManager?.setAutoRestore(enabled)
        rebuildMenu()
    }

    @objc func dashEscChanged(_ sender: NSButton) {
        let enabled = (sender.state == .on)
        defaults.set(enabled, forKey: "vnchkey.escRestore")
        keyManager?.escRestoreEnabled = enabled
        rebuildMenu()
    }

    @objc func dashBracketChanged(_ sender: NSButton) {
        let enabled = (sender.state == .on)
        defaults.set(enabled, forKey: "vnchkey.bracket")
        keyManager?.setBracketTransform(enabled)
        rebuildMenu()
    }

    @objc func dashW2UChanged(_ sender: NSPopUpButton) {
        let mode = UInt32(sender.indexOfSelectedItem)
        defaults.set(Int(mode), forKey: "vnchkey.w2u")
        keyManager?.setW2U(mode)
    }

    @objc func dashTimeoutChanged(_ sender: NSSlider) {
        // Làm tròn theo nấc 100ms để dễ kéo
        let raw = sender.integerValue
        let stepped = (raw / 100) * 100
        defaults.set(stepped, forKey: "vnchkey.idleTimeout")
        keyManager?.setIdleTimeout(UInt64(stepped))
        dashTimeoutValueLabel?.stringValue = (stepped == 0) ? "Tắt (0ms)" : "\(stepped) ms"
    }

    @objc func dashMacroChanged(_ sender: NSButton) {
        let enabled = (sender.state == .on)
        defaults.set(enabled, forKey: "vnchkey.macro")
        keyManager?.setMacroEnabled(enabled)
        rebuildMenu()
    }

    @objc func dashDictChanged(_ sender: NSButton) {
        let enabled = (sender.state == .on)
        defaults.set(enabled, forKey: "vnchkey.dictionary")
        keyManager?.setDictionaryEnabled(enabled)
        rebuildMenu()
    }

    @objc func dashResetDefaults() {
        defaults.removeObject(forKey: "vnchkey.initialized")
        applySavedPreferences()
        updateStatusButton()
        rebuildMenu()
        syncDashboardControls()
    }

    @objc func dashCloseWindow() {
        dashboardWindow?.orderOut(nil)
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
