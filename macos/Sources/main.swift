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
        
        if !(keyManager?.start() ?? false) {
            // Hiển thị thông báo nếu chưa có quyền Trợ năng
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

    func setupStatusBar() {
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        updateStatusButton()

        let menu = NSMenu()
        
        let toggleItem = NSMenuItem(title: "Bật gõ tiếng Việt", action: #selector(toggleEnabled), keyEquivalent: "v")
        toggleItem.state = .on
        menu.addItem(toggleItem)
        menu.addItem(NSMenuItem.separator())

        let methodMenu = NSMenu()
        let autoItem = NSMenuItem(title: "Tự động (Auto)", action: #selector(selectAutoMethod), keyEquivalent: "")
        autoItem.state = .on
        let telexItem = NSMenuItem(title: "Telex", action: #selector(selectTelexMethod), keyEquivalent: "")
        let vniItem = NSMenuItem(title: "VNI", action: #selector(selectVniMethod), keyEquivalent: "")
        
        methodMenu.addItem(autoItem)
        methodMenu.addItem(telexItem)
        methodMenu.addItem(vniItem)
        
        let methodParent = NSMenuItem(title: "Kiểu gõ", action: nil, keyEquivalent: "")
        methodParent.submenu = methodMenu
        menu.addItem(methodParent)

        menu.addItem(NSMenuItem.separator())
        menu.addItem(NSMenuItem(title: "Kiểm tra quyền Trợ năng", action: #selector(checkPermissionAction), keyEquivalent: ""))
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
        guard let km = keyManager else { return }
        km.isEnabled.toggle()
        sender.state = km.isEnabled ? .on : .off
        updateStatusButton()
    }

    @objc func selectAutoMethod(_ sender: NSMenuItem) {
        currentMethod = 0
        keyManager?.setMethod(0)
        updateMethodMenuState(sender)
    }

    @objc func selectTelexMethod(_ sender: NSMenuItem) {
        currentMethod = 1
        keyManager?.setMethod(1)
        updateMethodMenuState(sender)
    }

    @objc func selectVniMethod(_ sender: NSMenuItem) {
        currentMethod = 2
        keyManager?.setMethod(2)
        updateMethodMenuState(sender)
    }

    func updateMethodMenuState(_ activeItem: NSMenuItem) {
        if let parent = activeItem.parent {
            for item in parent.submenu?.items ?? [] {
                item.state = (item == activeItem) ? .on : .off
            }
        }
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

