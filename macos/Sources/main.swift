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
    private var dashCaseAnchorCheckbox: NSButton?
    private var dashW2UPopup: NSPopUpButton?
    private var dashTimeoutSlider: NSSlider?
    private var dashTimeoutValueLabel: NSTextField?
    private var dashMacroCheckbox: NSButton?
    private var dashDictCheckbox: NSButton?
    private var dashOpenAtLoginCheckbox: NSButton?
    private var dashPerAppCheckbox: NSButton?

    // Biến instance giữ reference tới CLI IPC server
    private var ipcServer: VnchKeyIPCServer?

    private var permissionPollTimer: Timer?

    // Per-App Input State: Lưu trạng thái bật/tắt theo từng Bundle ID
    private var appStates: [String: Bool] = [:]
    private var currentActiveBundleId: String?

    func applicationDidFinishLaunching(_ notification: Notification) {
        loadIcons()
        if let appIconPath = Bundle.main.path(forResource: "AppIcon", ofType: "png"),
           let appIconImg = NSImage(contentsOfFile: appIconPath) {
            NSApp.applicationIconImage = appIconImg
        }
        keyManager = KeyTapManager(method: UInt32(defaults.integer(forKey: "vnchkey.method")))
        keyManager?.onToggleEnabledByShortcut = { [weak self] in
            guard let self = self else { return }
            if self.isPerAppEnabled(), let currentId = self.currentActiveBundleId, let km = self.keyManager {
                self.appStates[currentId] = km.isEnabled
            }
            self.updateStatusButton()
            self.rebuildMenu()
            self.syncDashboardControls()
        }
        keyManager?.onShortcutRecorded = { [weak self] recorded in
            self?.saveCustomShortcut(recorded)
            self?.rebuildMenu()
            self?.syncDashboardControls()
        }
        applySavedPreferences()
        setupStatusBar()

        // Tự động lắng nghe chuyển đổi ứng dụng để lưu và phục hồi trạng thái gõ tương ứng
        setupAppFocusWatcher()

        // Khởi chạy IPC server lắng nghe lệnh toggle/on/off từ Neovim / CLI
        startIPCServer()

        if !(keyManager?.start() ?? false) {
            // Chỉ hiển thị hộp thoại hệ thống một lần duy nhất (như xKey)
            _ = keyManager?.checkAccessibilityPermission(prompt: true)
            // Lắng nghe người dùng bật công tắc trong System Settings để tự gắn hook
            startPermissionWatcher()
        }
    }

    private func startPermissionWatcher() {
        permissionPollTimer?.invalidate()
        permissionPollTimer = Timer.scheduledTimer(withTimeInterval: 1.0, repeats: true) { [weak self] timer in
            guard let self = self, let km = self.keyManager else { return }
            if km.checkAccessibilityPermission(prompt: false) {
                if km.start() {
                    timer.invalidate()
                    self.permissionPollTimer = nil
                    self.updateStatusButton()
                    self.rebuildMenu()
                }
            }
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
            defaults.set(true, forKey: "vnchkey.habitLearning") // Bật Học thói quen sửa từ
            defaults.set(4, forKey: "vnchkey.habitThreshold")   // Ngưỡng sửa/xoá (mặc định 4)
            defaults.set(true, forKey: "vnchkey.escRestore")  // Bấm Esc hoàn nguyên từ
            defaults.set(1, forKey: "vnchkey.toneStyle")      // Dấu cũ (hòa, ủy)
            defaults.set(0, forKey: "vnchkey.charset")        // Unicode dựng sẵn NFC
            defaults.set(false, forKey: "vnchkey.bracket")    // Gõ [ ] thành ơ ư
            defaults.set(false, forKey: "vnchkey.caseAnchor") // Đồng nhất chữ hoa theo chữ cái đầu
            defaults.set(0, forKey: "vnchkey.w2u")            // W -> ư (Không ở đầu từ)
            defaults.set(0, forKey: "vnchkey.idleTimeout")  // Mặc định tắt (0ms) để không ngắt gõ dấu VNI / flip word khi dừng tay
            defaults.set(true, forKey: "vnchkey.perApp")     // Mặc định nhớ trạng thái theo từng ứng dụng
            saveCustomShortcut(.defaultShortcut)
        }

        if defaults.object(forKey: "vnchkey.perApp") == nil {
            defaults.set(true, forKey: "vnchkey.perApp")
        }

        if defaults.object(forKey: "vnchkey.idleTimeout") == nil {
            defaults.set(0, forKey: "vnchkey.idleTimeout")
        }
        if defaults.object(forKey: "vnchkey.sc.initialized") == nil {
            saveCustomShortcut(.defaultShortcut)
        }

        km.setMethod(UInt32(defaults.integer(forKey: "vnchkey.method")))
        km.setSpellCheck(defaults.bool(forKey: "vnchkey.spellCheck"))
        km.setAutoRestore(defaults.bool(forKey: "vnchkey.autoRestore"))
        km.setMacroEnabled(defaults.bool(forKey: "vnchkey.macro"))
        km.setDictionaryEnabled(defaults.bool(forKey: "vnchkey.dictionary"))
        let habitLearningVal = defaults.object(forKey: "vnchkey.habitLearning") != nil ? defaults.bool(forKey: "vnchkey.habitLearning") : true
        km.setHabitLearningEnabled(habitLearningVal)
        let habitThresholdVal = defaults.object(forKey: "vnchkey.habitThreshold") != nil ? UInt32(defaults.integer(forKey: "vnchkey.habitThreshold")) : 4
        km.setHabitPromoteThreshold(habitThresholdVal)
        km.escRestoreEnabled = defaults.bool(forKey: "vnchkey.escRestore")
        let toneStyleVal = defaults.object(forKey: "vnchkey.toneStyle") != nil ? UInt32(defaults.integer(forKey: "vnchkey.toneStyle")) : 1
        km.setToneStyle(toneStyleVal)
        km.setCharset(UInt32(defaults.integer(forKey: "vnchkey.charset")))
        km.setBracketTransform(defaults.bool(forKey: "vnchkey.bracket"))
        km.setInitialCaseAnchor(defaults.bool(forKey: "vnchkey.caseAnchor"))
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

        let caseAnchorItem = NSMenuItem(title: "Đồng nhất chữ hoa theo chữ cái đầu (Được, được)", action: #selector(toggleCaseAnchor(_:)), keyEquivalent: "")
        caseAnchorItem.state = defaults.bool(forKey: "vnchkey.caseAnchor") ? .on : .off
        menu.addItem(caseAnchorItem)

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
        macroSubmenu.addItem(NSMenuItem(title: "📜 Nạp Mẫu Sài Gòn Xưa...", action: #selector(loadSaigonXuaPresets), keyEquivalent: ""))
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

        // Tự động học thói quen (Adaptive Habit Learning)
        let habitSubmenu = NSMenu()
        let habitToggleItem = NSMenuItem(title: "Bật Tự Động Học Sửa Từ", action: #selector(toggleHabitLearning(_:)), keyEquivalent: "")
        habitToggleItem.state = km.getHabitLearningEnabled() ? .on : .off
        habitSubmenu.addItem(habitToggleItem)
        habitSubmenu.addItem(NSMenuItem.separator())

        let curTh = km.getHabitPromoteThreshold()
        let thHint = NSMenuItem(title: "Ngưỡng học hiện tại: [\(curTh) lần sửa/xoá]", action: nil, keyEquivalent: "")
        thHint.isEnabled = false
        habitSubmenu.addItem(thHint)

        for th in [2, 3, 4, 5, 6, 8] {
            let label = th == 4 ? "\(th) lần (Khuyên dùng)" : (th == 5 ? "\(th) lần (An toàn)" : "\(th) lần")
            let item = NSMenuItem(title: label, action: #selector(selectHabitThreshold(_:)), keyEquivalent: "")
            item.tag = th
            item.state = (curTh == th) ? .on : .off
            habitSubmenu.addItem(item)
        }

        let habitParent = NSMenuItem(title: "Học thói quen sửa từ (Adaptive Learning)...", action: nil, keyEquivalent: "")
        habitParent.submenu = habitSubmenu
        menu.addItem(habitParent)

        menu.addItem(NSMenuItem.separator())
        let perAppItem = NSMenuItem(title: "Nhớ theo từng ứng dụng (Per-App)", action: #selector(togglePerAppFromMenu(_:)), keyEquivalent: "")
        perAppItem.state = isPerAppEnabled() ? .on : .off
        menu.addItem(perAppItem)

        let loginItem = NSMenuItem(title: "Khởi động cùng máy (Open at login)", action: #selector(toggleOpenAtLoginFromMenu(_:)), keyEquivalent: "")
        loginItem.state = isOpenAtLoginEnabled() ? .on : .off
        menu.addItem(loginItem)

        menu.addItem(NSMenuItem(title: "Kiểm tra quyền Trợ năng (Accessibility)", action: #selector(checkPermissionAction), keyEquivalent: ""))
        menu.addItem(NSMenuItem.separator())

        menu.addItem(NSMenuItem(title: "Kiểm tra bản cập nhật mới...", action: #selector(checkForUpdatesMenuAction), keyEquivalent: ""))
        menu.addItem(NSMenuItem(title: "Thông tin vnchKey (About)...", action: #selector(showAboutWindow), keyEquivalent: ""))
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
        if isPerAppEnabled(), let currentId = currentActiveBundleId {
            appStates[currentId] = km.isEnabled
        }
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

    @objc func toggleCaseAnchor(_ sender: NSMenuItem) {
        let next = !defaults.bool(forKey: "vnchkey.caseAnchor")
        defaults.set(next, forKey: "vnchkey.caseAnchor")
        keyManager?.setInitialCaseAnchor(next)
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

    @objc func toggleHabitLearning(_ sender: NSMenuItem) {
        guard let km = keyManager else { return }
        let next = !km.getHabitLearningEnabled()
        defaults.set(next, forKey: "vnchkey.habitLearning")
        km.setHabitLearningEnabled(next)
        rebuildMenu()
    }

    @objc func selectHabitThreshold(_ sender: NSMenuItem) {
        guard let km = keyManager else { return }
        let th = UInt32(sender.tag)
        defaults.set(th, forKey: "vnchkey.habitThreshold")
        km.setHabitPromoteThreshold(th)
        rebuildMenu()
    }

    // MARK: - Bảng Điều Khiển (Dashboard Window kiểu xKey / OpenKey)

    @objc func openDashboardWindow() {
        NSApp.activate(ignoringOtherApps: true)
        if let win = dashboardWindow {
            syncDashboardControls()
            win.makeKeyAndOrderFront(nil)
            return
        }

        let winRect = NSRect(x: 0, y: 0, width: 520, height: 655)
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
            let imageView = NSImageView(frame: NSRect(x: 24, y: 587, width: 48, height: 48))
            imageView.image = iconImg
            content.addSubview(imageView)
        }

        let titleLabel = NSTextField(labelWithString: "vnchKey cho macOS — Bảng Điều Khiển")
        titleLabel.font = NSFont.boldSystemFont(ofSize: 16)
        titleLabel.frame = NSRect(x: 84, y: 611, width: 410, height: 22)
        content.addSubview(titleLabel)

        let subtitleLabel = NSTextField(labelWithString: "Bộ gõ Tiếng Việt hiệu năng cao • Tự do tuỳ biến phím tắt & tính năng")
        subtitleLabel.font = NSFont.systemFont(ofSize: 12)
        subtitleLabel.textColor = .secondaryLabelColor
        subtitleLabel.frame = NSRect(x: 84, y: 589, width: 410, height: 18)
        content.addSubview(subtitleLabel)

        // Khung 1: Chế độ gõ & Phím tắt tự do (kiểu xKey / OpenKey)
        let generalBox = NSBox(frame: NSRect(x: 20, y: 363, width: 480, height: 214))
        generalBox.title = "Thiết lập Kiểu gõ & Phím tắt chuyển Việt / Anh tự do"
        content.addSubview(generalBox)

        let enableCheck = NSButton(checkboxWithTitle: "Bật chế độ gõ Tiếng Việt", target: self, action: #selector(dashToggleEnabled(_:)))
        enableCheck.frame = NSRect(x: 16, y: 162, width: 210, height: 22)
        generalBox.addSubview(enableCheck)
        self.dashEnableCheckbox = enableCheck

        let perAppCheck = NSButton(checkboxWithTitle: "Nhớ theo từng ứng dụng (Per-App)", target: self, action: #selector(dashPerAppChanged(_:)))
        perAppCheck.frame = NSRect(x: 230, y: 162, width: 230, height: 22)
        generalBox.addSubview(perAppCheck)
        self.dashPerAppCheckbox = perAppCheck

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
        let smartBox = NSBox(frame: NSRect(x: 20, y: 189, width: 480, height: 166))
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

        let timeoutSlider = NSSlider(value: 0, minValue: 0, maxValue: 2000, target: self, action: #selector(dashTimeoutChanged(_:)))
        timeoutSlider.frame = NSRect(x: 185, y: 14, width: 195, height: 20)
        smartBox.addSubview(timeoutSlider)
        self.dashTimeoutSlider = timeoutSlider

        let timeoutValLabel = NSTextField(labelWithString: "Tắt (0ms)")
        timeoutValLabel.frame = NSRect(x: 388, y: 14, width: 70, height: 20)
        smartBox.addSubview(timeoutValLabel)
        self.dashTimeoutValueLabel = timeoutValLabel

        // Khung 3: Quản lý Gõ Tắt & Từ Điển Cá Nhân
        let dataBox = NSBox(frame: NSRect(x: 20, y: 52, width: 480, height: 129))
        dataBox.title = "Gõ Tắt (Macros), Từ Điển & Thói Quen (Habit Learner)"
        content.addSubview(dataBox)

        let macroCheck = NSButton(checkboxWithTitle: "Bật Gõ Tắt (Macro)", target: self, action: #selector(dashMacroChanged(_:)))
        macroCheck.frame = NSRect(x: 16, y: 81, width: 155, height: 22)
        dataBox.addSubview(macroCheck)
        self.dashMacroCheckbox = macroCheck

        let addMacroBtn = NSButton(title: "➕ Thêm Gõ Tắt", target: self, action: #selector(promptAddMacro))
        addMacroBtn.bezelStyle = .rounded
        addMacroBtn.frame = NSRect(x: 180, y: 77, width: 140, height: 28)
        dataBox.addSubview(addMacroBtn)

        let openMacroBtn = NSButton(title: "Mở macros.conf", target: self, action: #selector(openMacrosFile))
        openMacroBtn.bezelStyle = .rounded
        openMacroBtn.frame = NSRect(x: 326, y: 77, width: 135, height: 28)
        dataBox.addSubview(openMacroBtn)

        let dictCheck = NSButton(checkboxWithTitle: "Bật Từ Điển Riêng", target: self, action: #selector(dashDictChanged(_:)))
        dictCheck.frame = NSRect(x: 16, y: 51, width: 155, height: 22)
        dataBox.addSubview(dictCheck)
        self.dashDictCheckbox = dictCheck

        let addDictBtn = NSButton(title: "➕ Thêm Từ Điển", target: self, action: #selector(promptAddDictionaryWord))
        addDictBtn.bezelStyle = .rounded
        addDictBtn.frame = NSRect(x: 180, y: 47, width: 140, height: 28)
        dataBox.addSubview(addDictBtn)

        let openDictBtn = NSButton(title: "Mở dictionary.conf", target: self, action: #selector(openDictionaryFile))
        openDictBtn.bezelStyle = .rounded
        openDictBtn.frame = NSRect(x: 326, y: 47, width: 135, height: 28)
        dataBox.addSubview(openDictBtn)

        let exportHabitBtn = NSButton(title: "📥 Xuất Sao Lưu & Thói Quen (JSON)", target: self, action: #selector(exportBackupAndHabits))
        exportHabitBtn.bezelStyle = .rounded
        exportHabitBtn.frame = NSRect(x: 16, y: 12, width: 225, height: 28)
        dataBox.addSubview(exportHabitBtn)

        let importHabitBtn = NSButton(title: "📤 Phục Hồi Dữ Liệu (JSON)", target: self, action: #selector(importBackupAndHabits))
        importHabitBtn.bezelStyle = .rounded
        importHabitBtn.frame = NSRect(x: 246, y: 12, width: 215, height: 28)
        dataBox.addSubview(importHabitBtn)

        // Thanh nút dưới cùng
        let resetBtn = NSButton(title: "Khôi phục mặc định", target: self, action: #selector(dashResetDefaults))
        resetBtn.bezelStyle = .rounded
        resetBtn.frame = NSRect(x: 20, y: 12, width: 155, height: 30)
        content.addSubview(resetBtn)

        let openAtLoginCheck = NSButton(checkboxWithTitle: "Khởi động cùng máy (Open at login)", target: self, action: #selector(dashOpenAtLoginChanged(_:)))
        openAtLoginCheck.frame = NSRect(x: 180, y: 16, width: 215, height: 22)
        content.addSubview(openAtLoginCheck)
        self.dashOpenAtLoginCheckbox = openAtLoginCheck

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
        dashCaseAnchorCheckbox?.state = defaults.bool(forKey: "vnchkey.caseAnchor") ? .on : .off
        dashW2UPopup?.selectItem(at: defaults.integer(forKey: "vnchkey.w2u"))

        let timeout = defaults.integer(forKey: "vnchkey.idleTimeout")
        dashTimeoutSlider?.integerValue = timeout
        dashTimeoutValueLabel?.stringValue = (timeout == 0) ? "Tắt (0ms)" : "\(timeout) ms"

        dashMacroCheckbox?.state = km.getMacroEnabled() ? .on : .off
        dashDictCheckbox?.state = km.getDictionaryEnabled() ? .on : .off
        dashOpenAtLoginCheckbox?.state = isOpenAtLoginEnabled() ? .on : .off
        dashPerAppCheckbox?.state = isPerAppEnabled() ? .on : .off
    }

    @objc func dashToggleEnabled(_ sender: NSButton) {
        guard let km = keyManager else { return }
        km.isEnabled = (sender.state == .on)
        if isPerAppEnabled(), let currentId = currentActiveBundleId {
            appStates[currentId] = km.isEnabled
        }
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
        let domain = Bundle.main.bundleIdentifier ?? "com.vnchkey.macos"
        defaults.removePersistentDomain(forName: domain)
        defaults.synchronize()
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

    @objc func exportBackupAndHabits() {
        guard let km = keyManager else { return }
        km.ensureDefaultConfigFiles()

        let panel = NSSavePanel()
        panel.title = "Xuất Toàn Bộ Dữ Liệu vnchKey & Thói Quen Học Tập"
        panel.nameFieldStringValue = "vnchkey-backup.json"
        panel.allowedContentTypes = [.json]

        NSApp.activate(ignoringOtherApps: true)
        if panel.runModal() == .OK, let url = panel.url {
            let macros = (try? String(contentsOfFile: km.macrosFilePath(), encoding: .utf8)) ?? ""
            let dict = (try? String(contentsOfFile: km.dictionaryFilePath(), encoding: .utf8)) ?? ""
            let habit = (try? String(contentsOfFile: km.habitLogFilePath(), encoding: .utf8)) ?? ""

            var dictWords: [String] = []
            for line in dict.components(separatedBy: .newlines) {
                let trimmed = line.trimmingCharacters(in: .whitespaces)
                if !trimmed.isEmpty && !trimmed.hasPrefix("#") {
                    dictWords.append(trimmed)
                }
            }

            var macroPairs: [[String]] = []
            for line in macros.components(separatedBy: .newlines) {
                let trimmed = line.trimmingCharacters(in: .whitespaces)
                if !trimmed.isEmpty && !trimmed.hasPrefix("#"), let eqIdx = trimmed.firstIndex(of: "=") {
                    let k = String(trimmed[..<eqIdx]).trimmingCharacters(in: .whitespaces)
                    let v = String(trimmed[trimmed.index(after: eqIdx)...]).trimmingCharacters(in: .whitespaces)
                    macroPairs.append([k, v])
                }
            }

            let backupDict: [String: Any] = [
                "version": "0.3.4",
                "method": km.getMethod(),
                "charset": defaults.integer(forKey: "vnchkey.charset"),
                "toneStyle": km.getToneStyle(),
                "macros": macroPairs,
                "dictionary": dictWords,
                "habit_log": habit
            ]

            if let jsonData = try? JSONSerialization.data(withJSONObject: backupDict, options: .prettyPrinted) {
                try? jsonData.write(to: url)
                let alert = NSAlert()
                alert.messageText = "Xuất Sao Lưu Thành Công"
                alert.informativeText = "Dữ liệu cấu hình, từ điển, gõ tắt và toàn bộ lịch sử thói quen học tập đã được lưu tại:\n\(url.path)"
                alert.alertStyle = .informational
                alert.runModal()
            }
        }
    }

    @objc func importBackupAndHabits() {
        guard let km = keyManager else { return }
        km.ensureDefaultConfigFiles()

        let panel = NSOpenPanel()
        panel.title = "Phục Hồi Dữ Liệu vnchKey & Thói Quen Học Tập"
        panel.allowedContentTypes = [.json]
        panel.allowsMultipleSelection = false

        NSApp.activate(ignoringOtherApps: true)
        if panel.runModal() == .OK, let url = panel.url {
            guard let data = try? Data(contentsOf: url),
                  let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any] else {
                let alert = NSAlert()
                alert.messageText = "Lỗi đọc tệp sao lưu"
                alert.informativeText = "Tệp JSON không hợp lệ hoặc không đúng định dạng sao lưu vnchKey."
                alert.alertStyle = .warning
                alert.runModal()
                return
            }

            if let dictWords = json["dictionary"] as? [String] {
                km.saveDictionaryWords(dictWords)
            }
            if let macroPairs = json["macros"] as? [[String]] {
                var lines = ["# Bảng Gõ Tắt (Macros)"]
                for p in macroPairs where p.count == 2 {
                    lines.append("\(p[0])=\(p[1])")
                }
                try? lines.joined(separator: "\n").write(toFile: km.macrosFilePath(), atomically: true, encoding: .utf8)
            }
            if let habitLog = json["habit_log"] as? String, !habitLog.isEmpty {
                try? habitLog.write(toFile: km.habitLogFilePath(), atomically: true, encoding: .utf8)
            }

            km.reloadMacrosAndDictionary()
            syncDashboardControls()

            let alert = NSAlert()
            alert.messageText = "Phục Hồi Dữ Liệu Thành Công"
            alert.informativeText = "Toàn bộ cài đặt, gõ tắt, từ điển và dữ liệu thói quen học tập đã được nạp lại thành công vào máy!"
            alert.alertStyle = .informational
            alert.runModal()
        }
    }

    @objc func loadSaigonXuaPresets() {
        guard let km = keyManager else { return }
        let alert = NSAlert()
        alert.messageText = "Nạp Bảng Gõ Tắt Sài Gòn Xưa"
        alert.informativeText = "Bạn có muốn nạp thêm hơn 60 thành ngữ và tiếng lóng Sài Gòn Xưa vào danh sách Gõ Tắt hiện tại không? (Các mục trùng lặp sẽ được giữ nguyên)."
        alert.alertStyle = .informational
        alert.addButton(withTitle: "Nạp ngay")
        alert.addButton(withTitle: "Huỷ")

        NSApp.activate(ignoringOtherApps: true)
        if alert.runModal() == .alertFirstButtonReturn {
            let saigonPresets: [(String, String)] = [
                ("bcbc", "bá cháy bù chét"),
                ("blbx", "bá láp bá xàm"),
                ("blbxao", "ba la ba xạo"),
                ("bxbk", "ba xạo ba kép"),
                ("bcl", "bà chằn lửa"),
                ("cdcc", "cứng đầu cứng cổ"),
                ("crct", "cà rịch cà tang"),
                ("cncn", "cà nhỏng cà nhỏng"),
                ("ccca", "cà chớn cà cháo"),
                ("cccc", "cà chớn cà chác"),
                ("ccl", "cù lần lữa"),
                ("hghs", "hết ga hết số"),
                ("hxqu", "hết xí quách"),
                ("mtd", "mưa thúi đất"),
                ("nlcd", "ngon lành cành đào"),
                ("qtmb", "quỉ tha ma bắt"),
                ("tdtd", "trời đánh thánh đâm"),
                ("tlml", "tào lao mía lao"),
                ("tlcd", "tào lao chi địa"),
                ("vglc", "vắt giò lên cổ"),
                ("xmdn", "xách mé đâm ngoáy"),
                ("xtxt", "xoắn tít mù khơi"),
                ("bct", "bạc con thoi"),
                ("cct", "chạy có cờ"),
                ("ctt", "chạy tụt quần"),
                ("dtc", "đứt từng khúc ruột"),
                ("dcd", "đâm cha chém chú"),
                ("kck", "kêu cha gọi mẹ"),
                ("lcl", "lên bờ xuống ruộng"),
                ("mcm", "mặt chai mày đá"),
                ("ncn", "ngậm đắng nuốt cay"),
                ("tct", "thấp thỏm không yên"),
                ("vcv", "vò đầu bứt tai"),
                ("xch", "xuống chó lên voi"),
                ("bđ", "bảnh bao"),
                ("cc", "chảnh chọe"),
                ("cl", "cà lơ phất phơ"),
                ("cp", "cù phơ cù bất"),
                ("đb", "đỏm dáng"),
                ("đc", "điệu đàng"),
                ("hl", "hầm hố"),
                ("ht", "hổ báo"),
                ("kl", "khệnh khạng"),
                ("ll", "lấc cấc"),
                ("lp", "lấc phấc"),
                ("nb", "ngổ ngáo"),
                ("pd", "phách lối"),
                ("pl", "phè phỡn"),
                ("tb", "tinh tướng"),
                ("tc", "trịch thượng")
            ]
            var list = km.readMacrosList()
            var existingKeys = Set(list.map { $0.0.lowercased() })
            var addedCount = 0
            for (trig, exp) in saigonPresets {
                if !existingKeys.contains(trig.lowercased()) {
                    list.append((trig, exp))
                    existingKeys.insert(trig.lowercased())
                    addedCount += 1
                }
            }
            km.saveMacrosList(list)
            rebuildMenu()

            let doneAlert = NSAlert()
            doneAlert.messageText = "Đã Nạp Xong"
            doneAlert.informativeText = "Đã bổ sung thành công \(addedCount) mục Gõ Tắt Sài Gòn Xưa."
            doneAlert.alertStyle = .informational
            doneAlert.runModal()
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
        alert.informativeText = "Nhập từ Tiếng Anh hoặc thuật ngữ riêng mà bạn muốn vnchKey luôn giữ nguyên (không bỏ dấu Tiếng Việt):"
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
        if !(keyManager?.checkAccessibilityPermission(prompt: false) ?? false) {
            _ = keyManager?.checkAccessibilityPermission(prompt: true)
            startPermissionWatcher()
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

    @objc func toggleOpenAtLoginFromMenu(_ sender: NSMenuItem) {
        let current = isOpenAtLoginEnabled()
        setOpenAtLoginEnabled(!current)
        rebuildMenu()
        syncDashboardControls()
    }

    @objc func dashOpenAtLoginChanged(_ sender: NSButton) {
        setOpenAtLoginEnabled(sender.state == .on)
        rebuildMenu()
        syncDashboardControls()
    }

    // MARK: - Open at Login (LaunchAgent)
    private var launchAgentPlistPath: URL {
        let home = FileManager.default.homeDirectoryForCurrentUser
        return home.appendingPathComponent("Library/LaunchAgents/com.vnchkey.macos.plist")
    }

    func isOpenAtLoginEnabled() -> Bool {
        return FileManager.default.fileExists(atPath: launchAgentPlistPath.path)
    }

    func setOpenAtLoginEnabled(_ enabled: Bool) {
        let fileManager = FileManager.default
        let plistURL = launchAgentPlistPath
        if enabled {
            let appBundlePath = Bundle.main.bundlePath
            let executablePath = Bundle.main.executablePath ?? (appBundlePath + "/Contents/MacOS/vnchKey")
            let plistContent = """
            <?xml version="1.0" encoding="UTF-8"?>
            <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
            <plist version="1.0">
            <dict>
                <key>Label</key>
                <string>com.vnchkey.macos</string>
                <key>ProgramArguments</key>
                <array>
                    <string>\(executablePath)</string>
                </array>
                <key>RunAtLoad</key>
                <true/>
                <key>KeepAlive</key>
                <false/>
                <key>ProcessType</key>
                <string>Interactive</string>
            </dict>
            </plist>
            """
            do {
                let launchAgentsDir = plistURL.deletingLastPathComponent()
                if !fileManager.fileExists(atPath: launchAgentsDir.path) {
                    try fileManager.createDirectory(at: launchAgentsDir, withIntermediateDirectories: true)
                }
                try plistContent.write(to: plistURL, atomically: true, encoding: .utf8)
            } catch {
                NSLog("[vnchKey] Lỗi khi tạo LaunchAgent plist: \(error)")
            }
        } else {
            if fileManager.fileExists(atPath: plistURL.path) {
                try? fileManager.removeItem(at: plistURL)
            }
        }
    }

    // MARK: - Per-App Input State (Nhớ trạng thái theo từng ứng dụng)

    func isPerAppEnabled() -> Bool {
        if defaults.object(forKey: "vnchkey.perApp") == nil { return true }
        return defaults.bool(forKey: "vnchkey.perApp")
    }

    func setupAppFocusWatcher() {
        if let frontApp = NSWorkspace.shared.frontmostApplication {
            currentActiveBundleId = frontApp.bundleIdentifier ?? frontApp.localizedName
        }
        NSWorkspace.shared.notificationCenter.addObserver(
            self,
            selector: #selector(handleAppActivated(_:)),
            name: NSWorkspace.didActivateApplicationNotification,
            object: nil
        )
    }

    @objc func handleAppActivated(_ notification: Notification) {
        guard isPerAppEnabled(), let km = keyManager else { return }
        guard let app = notification.userInfo?[NSWorkspace.applicationUserInfoKey] as? NSRunningApplication else { return }
        let bundleId = app.bundleIdentifier ?? app.localizedName ?? "unknown"

        // Bỏ qua nếu là chính app vnchKey
        if bundleId == Bundle.main.bundleIdentifier { return }

        // Lưu trạng thái ứng dụng trước khi rời đi
        if let prevId = currentActiveBundleId {
            appStates[prevId] = km.isEnabled
        }

        currentActiveBundleId = bundleId

        // Khôi phục trạng thái cho ứng dụng mới (mặc định true nếu chưa có trong bộ nhớ)
        let nextState = appStates[bundleId] ?? true
        if km.isEnabled != nextState {
            km.isEnabled = nextState
            updateStatusButton()
            rebuildMenu()
            syncDashboardControls()
        }
    }

    @objc func togglePerAppFromMenu(_ sender: NSMenuItem) {
        let next = !isPerAppEnabled()
        defaults.set(next, forKey: "vnchkey.perApp")
        rebuildMenu()
        syncDashboardControls()
    }

    @objc func dashPerAppChanged(_ sender: NSButton) {
        let next = (sender.state == .on)
        defaults.set(next, forKey: "vnchkey.perApp")
        rebuildMenu()
        syncDashboardControls()
    }

    // MARK: - CLI & IPC Server for Neovim Modal Toggle
    func startIPCServer() {
        self.ipcServer = VnchKeyIPCServer { [weak self] command in
            DispatchQueue.main.async {
                guard let self = self, let km = self.keyManager else { return }
                switch command.trimmingCharacters(in: .whitespacesAndNewlines).lowercased() {
                case "on", "enable", "vi":
                    km.isEnabled = true
                case "off", "disable", "en":
                    km.isEnabled = false
                case "toggle":
                    km.isEnabled.toggle()
                case "status":
                    break
                default:
                    break
                }
                if self.isPerAppEnabled(), let currentId = self.currentActiveBundleId {
                    self.appStates[currentId] = km.isEnabled
                }
                self.updateStatusButton()
                self.rebuildMenu()
                self.syncDashboardControls()
            }
        }
        self.ipcServer?.start()
    }

    @objc func showAboutWindow() {
        NSApp.activate(ignoringOtherApps: true)
        let alert = NSAlert()
        alert.messageText = "vnchKey — Tự Do Edition"
        let curVer = "0.3.4"
        alert.informativeText = """
        Phiên bản: v\(curVer) (Universal Binary: Apple Silicon & Intel)

        • Bộ gõ Tiếng Việt hiện đại và gọn nhẹ cho macOS
        • Tinh Thần Tự Do và Di Sản Cờ Vàng Ba Sọc Đỏ
        • 100% Offline, bảo mật cao và không gởi bất kỳ dữ liệu nào ra khỏi máy người dùng
        • Tự động khôi phục Tiếng Anh, Smart Casing, hồi sinh từ

        Mã nguồn: https://github.com/no-vici/fcitx5-vnchkey
        Giấy phép: GNU General Public License v3.0 (GPLv3)
        """
        if let iconPath = Bundle.main.path(forResource: "AppIcon", ofType: "png"),
           let iconImg = NSImage(contentsOfFile: iconPath) {
            alert.icon = iconImg
        }
        alert.addButton(withTitle: "Đóng")
        alert.addButton(withTitle: "Kiểm tra bản mới...")
        alert.addButton(withTitle: "Xem GitHub")

        let resp = alert.runModal()
        if resp == .alertSecondButtonReturn {
            // Kiểm tra bản mới
            checkForUpdates(silentIfLatest: false)
        } else if resp == .alertThirdButtonReturn {
            if let url = URL(string: "https://github.com/no-vici/fcitx5-vnchkey/releases") {
                NSWorkspace.shared.open(url)
            }
        }
    }

    @objc func checkForUpdatesMenuAction() {
        checkForUpdates(silentIfLatest: false)
    }

    func checkForUpdates(silentIfLatest: Bool) {
        guard let url = URL(string: "https://api.github.com/repos/no-vici/fcitx5-vnchkey/releases/latest") else { return }
        var request = URLRequest(url: url)
        request.setValue("vnchKey-AutoUpdater", forHTTPHeaderField: "User-Agent")
        request.timeoutInterval = 10.0

        let curVer = "0.3.4"
        let task = URLSession.shared.dataTask(with: request) { data, response, error in
            DispatchQueue.main.async {
                if let data = data,
                   let json = try? JSONSerialization.jsonObject(with: data, options: []) as? [String: Any],
                   let tagName = json["tag_name"] as? String {

                    let cleanLatest = tagName.trimmingCharacters(in: CharacterSet(charactersIn: "vV "))
                    let cleanCur = curVer.trimmingCharacters(in: CharacterSet(charactersIn: "vV "))

                    if cleanLatest.compare(cleanCur, options: .numeric) == .orderedDescending {
                        // Tìm link download DMG trong assets
                        var dmgDownloadUrl: String? = nil
                        if let assets = json["assets"] as? [[String: Any]] {
                            for asset in assets {
                                if let name = asset["name"] as? String, name.hasSuffix(".dmg"),
                                   let dl = asset["browser_download_url"] as? String {
                                    dmgDownloadUrl = dl
                                    break
                                }
                            }
                        }
                        if dmgDownloadUrl == nil {
                            dmgDownloadUrl = "https://github.com/no-vici/fcitx5-vnchkey/releases/download/\(tagName)/vnchKey-\(cleanLatest).dmg"
                        }

                        let alert = NSAlert()
                        alert.messageText = "Đã có phiên bản mới: \(tagName)"
                        alert.informativeText = "Phiên bản hiện tại của bạn là v\(curVer).\nBạn có muốn tự động tải về và mở bản cập nhật ngay bây giờ không?"
                        alert.alertStyle = .informational
                        alert.addButton(withTitle: "Cập nhật ngay")
                        alert.addButton(withTitle: "Để sau")

                        if alert.runModal() == .alertFirstButtonReturn, let dlStr = dmgDownloadUrl, let dlUrl = URL(string: dlStr) {
                            self.downloadAndInstallUpdate(dmgUrl: dlUrl, tagName: tagName)
                        }
                    } else if !silentIfLatest {
                        let alert = NSAlert()
                        alert.messageText = "Bạn đang dùng bản mới nhất"
                        alert.informativeText = "vnchKey v\(curVer) là phiên bản mới nhất hiện nay. Không có bản cập nhật nào mới hơn."
                        alert.alertStyle = .informational
                        alert.runModal()
                    }
                } else if !silentIfLatest {
                    let alert = NSAlert()
                    alert.messageText = "Không thể kiểm tra cập nhật"
                    alert.informativeText = "Không thể kết nối đến máy chủ GitHub lúc này. Vui lòng kiểm tra lại kết nối mạng hoặc thử lại sau."
                    alert.alertStyle = .warning
                    alert.runModal()
                }
            }
        }
        task.resume()
    }

    private func downloadAndInstallUpdate(dmgUrl: URL, tagName: String) {
        let tempDir = FileManager.default.temporaryDirectory
        let destinationUrl = tempDir.appendingPathComponent("vnchKey_update.dmg")
        try? FileManager.default.removeItem(at: destinationUrl)

        var req = URLRequest(url: dmgUrl)
        req.setValue("vnchKey-AutoUpdater", forHTTPHeaderField: "User-Agent")

        let alert = NSAlert()
        alert.messageText = "Đang tải bản cập nhật..."
        alert.informativeText = "Đang tải \(tagName) từ GitHub. Ứng dụng sẽ tự động mở gói cài đặt DMG khi hoàn tất."
        let downloadTask = URLSession.shared.downloadTask(with: req) { localUrl, resp, err in
            DispatchQueue.main.async {
                guard let localUrl = localUrl, err == nil else {
                    let errAlert = NSAlert()
                    errAlert.messageText = "Tải cập nhật thất bại"
                    errAlert.informativeText = "Không thể tải file DMG cập nhật. Bạn có thể tải thủ công từ GitHub."
                    errAlert.addButton(withTitle: "Mở GitHub")
                    errAlert.addButton(withTitle: "Đóng")
                    if errAlert.runModal() == .alertFirstButtonReturn {
                        if let u = URL(string: "https://github.com/no-vici/fcitx5-vnchkey/releases") {
                            NSWorkspace.shared.open(u)
                        }
                    }
                    return
                }

                do {
                    try FileManager.default.moveItem(at: localUrl, to: destinationUrl)
                    // Mở file DMG vừa tải
                    NSWorkspace.shared.open(destinationUrl)
                } catch {
                    NSWorkspace.shared.open(dmgUrl)
                }
            }
        }
        downloadTask.resume()
    }

    @objc func quitApp() {
        NSApplication.shared.terminate(self)
    }
}

// MARK: - IPC Server & CLI Client (Unix Domain Socket)
class VnchKeyIPCServer {
    private let socketPath = "/tmp/vnchkey.sock"
    private let onCommand: (String) -> Void
    private var serverSource: DispatchSourceRead?
    private var serverFd: Int32 = -1

    init(onCommand: @escaping (String) -> Void) {
        self.onCommand = onCommand
    }

    func start() {
        // Ignore SIGPIPE globally so client disconnecting doesn't terminate vnchKey
        signal(SIGPIPE, SIG_IGN)

        unlink(socketPath)

        let fd = socket(AF_UNIX, SOCK_STREAM, 0)
        guard fd >= 0 else { return }
        self.serverFd = fd

        var addr = sockaddr_un()
        addr.sun_family = sa_family_t(AF_UNIX)
        let pathBytes = socketPath.utf8CString
        withUnsafeMutablePointer(to: &addr.sun_path) { ptr in
            ptr.withMemoryRebound(to: CChar.self, capacity: MemoryLayout.size(ofValue: ptr.pointee)) { dest in
                _ = pathBytes.withUnsafeBufferPointer { src in
                    strncpy(dest, src.baseAddress!, MemoryLayout.size(ofValue: ptr.pointee) - 1)
                }
            }
        }

        let addrLen = socklen_t(MemoryLayout<sockaddr_un>.size)
        let bindResult = withUnsafePointer(to: &addr) { ptr in
            ptr.withMemoryRebound(to: sockaddr.self, capacity: 1) { sa in
                bind(fd, sa, addrLen)
            }
        }
        guard bindResult == 0 else {
            close(fd)
            return
        }

        guard listen(fd, 5) == 0 else {
            close(fd)
            return
        }

        // Set socket file permissions to 0666 so any user/app can connect
        chmod(socketPath, 0o600)

        let source = DispatchSource.makeReadSource(fileDescriptor: fd, queue: DispatchQueue.global(qos: .utility))
        source.setEventHandler { [weak self] in
            guard let self = self else { return }
            var clientAddr = sockaddr_un()
            var clientLen = socklen_t(MemoryLayout<sockaddr_un>.size)
            let clientFd = withUnsafeMutablePointer(to: &clientAddr) { ptr in
                ptr.withMemoryRebound(to: sockaddr.self, capacity: 1) { sa in
                    accept(self.serverFd, sa, &clientLen)
                }
            }
            guard clientFd >= 0 else { return }

            var buffer = [UInt8](repeating: 0, count: 256)
            let bytesRead = read(clientFd, &buffer, 255)
            if bytesRead > 0 {
                let cmd = String(decoding: buffer[0..<bytesRead], as: UTF8.self)
                self.onCommand(cmd)
                let reply = "OK\n"
                reply.withCString {
                    #if os(macOS)
                    var nosig: Int32 = 1
                    setsockopt(clientFd, SOL_SOCKET, SO_NOSIGPIPE, &nosig, socklen_t(MemoryLayout<Int32>.size))
                    #endif
                    _ = write(clientFd, $0, reply.utf8.count)
                }
            }
            close(clientFd)
        }
        source.resume()
        self.serverSource = source
    }

    deinit {
        serverSource?.cancel()
        if serverFd >= 0 {
            close(serverFd)
        }
        unlink(socketPath)
    }
}

func handleCommandLineArguments() -> Bool {
    let args = CommandLine.arguments
    guard args.count > 1 else { return false }

    let flag = args[1].lowercased()
    var commandToSend: String?

    switch flag {
    case "--on", "-on", "on", "--enable", "enable":
        commandToSend = "on"
    case "--off", "-off", "off", "--disable", "disable":
        commandToSend = "off"
    case "--toggle", "-toggle", "toggle", "-t":
        commandToSend = "toggle"
    case "--status", "-status", "status":
        commandToSend = "status"
    case "--help", "-h", "help":
        print("""
        vnchKey CLI Controller (macOS)
        Sử dụng cho Neovim autocommands, Scripts hoặc Terminal:
          vnchKey --on       Bật gõ Tiếng Việt
          vnchKey --off      Tắt gõ Tiếng Việt (chuyển sang Tiếng Anh)
          vnchKey --toggle   Đảo trạng thái Tiếng Việt / Tiếng Anh
          vnchKey --status   Kiểm tra trạng thái bộ gõ
        """)
        exit(0)
    default:
        return false
    }

    if let cmd = commandToSend {
        let socketPath = "/tmp/vnchkey.sock"
        let fd = socket(AF_UNIX, SOCK_STREAM, 0)
        if fd >= 0 {
            var addr = sockaddr_un()
            addr.sun_family = sa_family_t(AF_UNIX)
            let pathBytes = socketPath.utf8CString
            withUnsafeMutablePointer(to: &addr.sun_path) { ptr in
                ptr.withMemoryRebound(to: CChar.self, capacity: MemoryLayout.size(ofValue: ptr.pointee)) { dest in
                    _ = pathBytes.withUnsafeBufferPointer { src in
                        strncpy(dest, src.baseAddress!, MemoryLayout.size(ofValue: ptr.pointee) - 1)
                    }
                }
            }
            let addrLen = socklen_t(MemoryLayout<sockaddr_un>.size)
            let connectResult = withUnsafePointer(to: &addr) { ptr in
                ptr.withMemoryRebound(to: sockaddr.self, capacity: 1) { sa in
                    connect(fd, sa, addrLen)
                }
            }
            if connectResult == 0 {
                let msg = cmd + "\n"
                _ = msg.withCString { write(fd, $0, msg.utf8.count) }
                var replyBuf = [UInt8](repeating: 0, count: 64)
                let n = read(fd, &replyBuf, 63)
                if n > 0 {
                    let rep = String(decoding: replyBuf[0..<n], as: UTF8.self)
                    print(rep.trimmingCharacters(in: .whitespacesAndNewlines))
                }
                close(fd)
                exit(0)
            } else {
                close(fd)
                fputs("❌ vnchKey daemon không chạy hoặc không thể kết nối tới /tmp/vnchkey.sock\n", stderr)
                exit(1)
            }
        }
    }
    return false
}

// Kiểm tra tham số dòng lệnh trước khi khởi tạo UI application
_ = handleCommandLineArguments()

let app = NSApplication.shared
let delegate = AppDelegate()
app.delegate = delegate
app.run()
