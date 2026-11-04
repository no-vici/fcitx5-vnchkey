#include "vnchkey_addon.h"
#include <fcitx-utils/keysym.h>
#include <fcitx-utils/key.h>
#include <fcitx/inputcontext.h>
#include <chrono>
#include <string>
#include <fstream>
#include <cstdlib>
#include <unistd.h>

VnchkeyEngine::VnchkeyEngine(fcitx::AddonManager* manager)
    : manager_(manager) {
    // 0: Auto (Telex & VNI), 1: Telex, 2: VNI
    ctx_auto_ = fcitx5_vnchkey_init_with_method(0);
    ctx_telex_ = fcitx5_vnchkey_init_with_method(1);
    ctx_vni_ = fcitx5_vnchkey_init_with_method(2);
    loadConfig();
    setupStatusMenu();

    if (manager_ && manager_->instance() && statusAction_) {
        manager_->instance()->userInterfaceManager().registerAction("vnchkey-status", statusAction_.get());
    }
}

VnchkeyEngine::~VnchkeyEngine() {
    if (manager_ && manager_->instance() && statusAction_) {
        manager_->instance()->userInterfaceManager().unregisterAction(statusAction_.get());
    }
    connections_.clear();
    if (ctx_auto_) {
        fcitx5_vnchkey_free(ctx_auto_);
        ctx_auto_ = nullptr;
    }
    if (ctx_telex_) {
        fcitx5_vnchkey_free(ctx_telex_);
        ctx_telex_ = nullptr;
    }
    if (ctx_vni_) {
        fcitx5_vnchkey_free(ctx_vni_);
        ctx_vni_ = nullptr;
    }
}

void VnchkeyEngine::setupStatusMenu() {
    statusAction_ = std::make_unique<fcitx::SimpleAction>();
    statusAction_->setIcon("vnchkey");
    statusAction_->setShortText(config_method_);
    statusAction_->setLongText("vnchKey (Tự Do Edition)");

    statusMenu_ = std::make_unique<fcitx::Menu>();
    statusAction_->setMenu(statusMenu_.get());

    // 1. Input Method Group
    actionMethodVni_ = std::make_unique<fcitx::SimpleAction>();
    actionMethodVni_->setCheckable(true);
    actionMethodVni_->setShortText("VNI");
    actionMethodVni_->setLongText("Kiểu gõ: VNI");
    connections_.push_back(actionMethodVni_->connect<fcitx::SimpleAction::Activated>([this](fcitx::InputContext* ic) {
        config_method_ = "VNI";
        updateActionState();
        saveConfig();
        if (ic && statusAction_) {
            statusAction_->update(ic);
            ic->updateUserInterface(fcitx::UserInterfaceComponent::StatusArea);
        }
    }));
    statusMenu_->addAction(actionMethodVni_.get());

    actionMethodTelex_ = std::make_unique<fcitx::SimpleAction>();
    actionMethodTelex_->setCheckable(true);
    actionMethodTelex_->setShortText("Telex");
    actionMethodTelex_->setLongText("Kiểu gõ: Telex");
    connections_.push_back(actionMethodTelex_->connect<fcitx::SimpleAction::Activated>([this](fcitx::InputContext* ic) {
        config_method_ = "Telex";
        updateActionState();
        saveConfig();
        if (ic && statusAction_) {
            statusAction_->update(ic);
            ic->updateUserInterface(fcitx::UserInterfaceComponent::StatusArea);
        }
    }));
    statusMenu_->addAction(actionMethodTelex_.get());

    actionMethodAuto_ = std::make_unique<fcitx::SimpleAction>();
    actionMethodAuto_->setCheckable(true);
    actionMethodAuto_->setShortText("Tự Động (Auto)");
    actionMethodAuto_->setLongText("Kiểu gõ: Tự Động (Telex & VNI)");
    connections_.push_back(actionMethodAuto_->connect<fcitx::SimpleAction::Activated>([this](fcitx::InputContext* ic) {
        config_method_ = "Auto";
        updateActionState();
        saveConfig();
        if (ic && statusAction_) {
            statusAction_->update(ic);
            ic->updateUserInterface(fcitx::UserInterfaceComponent::StatusArea);
        }
    }));
    statusMenu_->addAction(actionMethodAuto_.get());

    // Separator 1
    actionSeparator1_ = std::make_unique<fcitx::SimpleAction>();
    actionSeparator1_->setSeparator(true);
    statusMenu_->addAction(actionSeparator1_.get());

    // 2. Output Charset Group
    actionCharsetUnicode_ = std::make_unique<fcitx::SimpleAction>();
    actionCharsetUnicode_->setCheckable(true);
    actionCharsetUnicode_->setShortText("Unicode");
    actionCharsetUnicode_->setLongText("Bảng mã: Unicode");
    connections_.push_back(actionCharsetUnicode_->connect<fcitx::SimpleAction::Activated>([this](fcitx::InputContext* ic) {
        config_charset_ = "Unicode";
        updateActionState();
        saveConfig();
        if (ic && statusAction_) {
            statusAction_->update(ic);
            ic->updateUserInterface(fcitx::UserInterfaceComponent::StatusArea);
        }
    }));
    statusMenu_->addAction(actionCharsetUnicode_.get());

    actionCharsetToHop_ = std::make_unique<fcitx::SimpleAction>();
    actionCharsetToHop_->setCheckable(true);
    actionCharsetToHop_->setShortText("Unicode Tổ Hợp");
    actionCharsetToHop_->setLongText("Bảng mã: Unicode Tổ Hợp");
    connections_.push_back(actionCharsetToHop_->connect<fcitx::SimpleAction::Activated>([this](fcitx::InputContext* ic) {
        config_charset_ = "Unicode Tổ Hợp";
        updateActionState();
        saveConfig();
        if (ic && statusAction_) {
            statusAction_->update(ic);
            ic->updateUserInterface(fcitx::UserInterfaceComponent::StatusArea);
        }
    }));
    statusMenu_->addAction(actionCharsetToHop_.get());

    actionCharsetTcvn3_ = std::make_unique<fcitx::SimpleAction>();
    actionCharsetTcvn3_->setCheckable(true);
    actionCharsetTcvn3_->setShortText("TCVN3 (ABC)");
    actionCharsetTcvn3_->setLongText("Bảng mã: TCVN3");
    connections_.push_back(actionCharsetTcvn3_->connect<fcitx::SimpleAction::Activated>([this](fcitx::InputContext* ic) {
        config_charset_ = "TCVN3 (ABC)";
        updateActionState();
        saveConfig();
        if (ic && statusAction_) {
            statusAction_->update(ic);
            ic->updateUserInterface(fcitx::UserInterfaceComponent::StatusArea);
        }
    }));
    statusMenu_->addAction(actionCharsetTcvn3_.get());

    // Separator 2
    actionSeparator2_ = std::make_unique<fcitx::SimpleAction>();
    actionSeparator2_->setSeparator(true);
    statusMenu_->addAction(actionSeparator2_.get());

    // 3. Toggles: GoNhanh Auto-Restore, Macros, Spell Check
    actionAutoRestore_ = std::make_unique<fcitx::SimpleAction>();
    actionAutoRestore_->setCheckable(true);
    actionAutoRestore_->setShortText("Khôi Phục Tiếng Anh (GoNhanh)");
    connections_.push_back(actionAutoRestore_->connect<fcitx::SimpleAction::Activated>([this](fcitx::InputContext* ic) {
        config_auto_restore_ = !config_auto_restore_;
        for (void* ctx : {ctx_auto_, ctx_telex_, ctx_vni_}) {
            if (ctx) fcitx5_vnchkey_set_auto_restore(ctx, config_auto_restore_);
        }
        updateActionState();
        saveConfig();
        if (ic && statusAction_) {
            statusAction_->update(ic);
            ic->updateUserInterface(fcitx::UserInterfaceComponent::StatusArea);
        }
    }));
    statusMenu_->addAction(actionAutoRestore_.get());

    actionMacro_ = std::make_unique<fcitx::SimpleAction>();
    actionMacro_->setCheckable(true);
    actionMacro_->setShortText("Bật Gõ Tắt (Macro)");
    connections_.push_back(actionMacro_->connect<fcitx::SimpleAction::Activated>([this](fcitx::InputContext* ic) {
        config_macro_ = !config_macro_;
        for (void* ctx : {ctx_auto_, ctx_telex_, ctx_vni_}) {
            if (ctx) fcitx5_vnchkey_set_macro_enabled(ctx, config_macro_);
        }
        updateActionState();
        saveConfig();
        if (ic && statusAction_) {
            statusAction_->update(ic);
            ic->updateUserInterface(fcitx::UserInterfaceComponent::StatusArea);
        }
    }));
    statusMenu_->addAction(actionMacro_.get());

    actionSpellCheck_ = std::make_unique<fcitx::SimpleAction>();
    actionSpellCheck_->setCheckable(true);
    actionSpellCheck_->setShortText("Kiểm Tra Chính Tả");
    connections_.push_back(actionSpellCheck_->connect<fcitx::SimpleAction::Activated>([this](fcitx::InputContext* ic) {
        config_spell_check_ = !config_spell_check_;
        updateActionState();
        saveConfig();
        if (ic && statusAction_) {
            statusAction_->update(ic);
            ic->updateUserInterface(fcitx::UserInterfaceComponent::StatusArea);
        }
    }));
    statusMenu_->addAction(actionSpellCheck_.get());

    // 4. Launch Settings GUI
    actionSettings_ = std::make_unique<fcitx::SimpleAction>();
    actionSettings_->setShortText("Cài Đặt Fcitx5 vnchKey...");
    actionSettings_->setIcon("preferences-system");
    connections_.push_back(actionSettings_->connect<fcitx::SimpleAction::Activated>([](fcitx::InputContext*) {
        if (fork() == 0) {
            execlp("fcitx5-vnchkey-settings", "fcitx5-vnchkey-settings", nullptr);
            _exit(0);
        }
    }));
    statusMenu_->addAction(actionSettings_.get());

    updateActionState();
}

void VnchkeyEngine::updateActionState() {
    if (statusAction_) {
        statusAction_->setShortText(config_method_);
    }

    if (actionMethodVni_) actionMethodVni_->setChecked(config_method_ == "VNI" || config_method_ == "vni");
    if (actionMethodTelex_) actionMethodTelex_->setChecked(config_method_ == "Telex" || config_method_ == "telex");
    if (actionMethodAuto_) actionMethodAuto_->setChecked(config_method_ == "Auto" || config_method_ == "auto");

    if (actionCharsetUnicode_) actionCharsetUnicode_->setChecked(config_charset_ == "Unicode");
    if (actionCharsetToHop_) actionCharsetToHop_->setChecked(config_charset_ == "Unicode Tổ Hợp");
    if (actionCharsetTcvn3_) actionCharsetTcvn3_->setChecked(config_charset_ == "TCVN3 (ABC)");

    if (actionAutoRestore_) actionAutoRestore_->setChecked(config_auto_restore_);
    if (actionMacro_) actionMacro_->setChecked(config_macro_);
    if (actionSpellCheck_) actionSpellCheck_->setChecked(config_spell_check_);
}

void VnchkeyEngine::activate(const fcitx::InputMethodEntry &entry, fcitx::InputContextEvent &event) {
    auto name = entry.uniqueName();
    if (name.find("vni") != std::string::npos) {
        config_method_ = "VNI";
    } else if (name.find("telex") != std::string::npos) {
        config_method_ = "Telex";
    }

    auto* ic = event.inputContext();
    if (ic && statusAction_) {
        updateActionState();
        ic->statusArea().addAction(fcitx::StatusGroup::InputMethod, statusAction_.get());
        statusAction_->update(ic);
        ic->updateUserInterface(fcitx::UserInterfaceComponent::StatusArea);
    }
}

void VnchkeyEngine::deactivate(const fcitx::InputMethodEntry &entry, fcitx::InputContextEvent &event) {
    FCITX_UNUSED(entry);
    auto* ic = event.inputContext();
    if (ic && statusAction_) {
        ic->statusArea().removeAction(statusAction_.get());
        ic->updateUserInterface(fcitx::UserInterfaceComponent::StatusArea);
    }
}

std::string VnchkeyEngine::subMode(const fcitx::InputMethodEntry &entry, fcitx::InputContext &inputContext) {
    FCITX_UNUSED(entry);
    FCITX_UNUSED(inputContext);
    return config_method_;
}

std::string VnchkeyEngine::subModeIconImpl(const fcitx::InputMethodEntry &entry, fcitx::InputContext &ic) {
    FCITX_UNUSED(entry);
    FCITX_UNUSED(ic);
    return "vnchkey";
}

std::string VnchkeyEngine::subModeLabelImpl(const fcitx::InputMethodEntry &entry, fcitx::InputContext &ic) {
    FCITX_UNUSED(entry);
    FCITX_UNUSED(ic);
    return config_method_;
}

void VnchkeyEngine::loadConfig() {
    const char* home = getenv("HOME");
    if (!home) return;
    std::string path = std::string(home) + "/.config/fcitx5/conf/vnchkey.conf";
    std::ifstream file(path);
    if (file.is_open()) {
        std::string line;
        while (std::getline(file, line)) {
            if (line.rfind("InputMethod=", 0) == 0) {
                std::string val = line.substr(12);
                size_t start = val.find_first_not_of(" \t\"'");
                size_t end = val.find_last_not_of(" \t\"'");
                if (start != std::string::npos && end != std::string::npos) {
                    config_method_ = val.substr(start, end - start + 1);
                }
            } else if (line.rfind("OutputCharset=", 0) == 0) {
                std::string val = line.substr(14);
                size_t start = val.find_first_not_of(" \t\"'");
                size_t end = val.find_last_not_of(" \t\"'");
                if (start != std::string::npos && end != std::string::npos) {
                    config_charset_ = val.substr(start, end - start + 1);
                }
            } else if (line.rfind("AutoRestoreEnglish=", 0) == 0) {
                config_auto_restore_ = (line.find("False") == std::string::npos);
            } else if (line.rfind("EnableMacro=", 0) == 0) {
                config_macro_ = (line.find("False") == std::string::npos);
            } else if (line.rfind("SpellCheck=", 0) == 0) {
                config_spell_check_ = (line.find("False") == std::string::npos);
            }
        }
    }

    // Load macro table if configured
    std::string macro_path = std::string(home) + "/.config/fcitx5/conf/vnchkey-macro-table.conf";
    if (ctx_auto_) fcitx5_vnchkey_load_macros(ctx_auto_, macro_path.c_str());
    if (ctx_telex_) fcitx5_vnchkey_load_macros(ctx_telex_, macro_path.c_str());
    if (ctx_vni_) fcitx5_vnchkey_load_macros(ctx_vni_, macro_path.c_str());

    for (void* ctx : {ctx_auto_, ctx_telex_, ctx_vni_}) {
        if (!ctx) continue;
        fcitx5_vnchkey_set_auto_restore(ctx, config_auto_restore_);
        fcitx5_vnchkey_set_macro_enabled(ctx, config_macro_);
    }
}

void VnchkeyEngine::saveConfig() {
    const char* home = getenv("HOME");
    if (!home) return;
    std::string path = std::string(home) + "/.config/fcitx5/conf/vnchkey.conf";
    
    // Read existing file to preserve other fields
    std::vector<std::string> lines;
    {
        std::ifstream file(path);
        if (file.is_open()) {
            std::string l;
            while (std::getline(file, l)) {
                if (l.rfind("InputMethod=", 0) != 0 &&
                    l.rfind("OutputCharset=", 0) != 0 &&
                    l.rfind("AutoRestoreEnglish=", 0) != 0 &&
                    l.rfind("EnableMacro=", 0) != 0 &&
                    l.rfind("SpellCheck=", 0) != 0) {
                    lines.push_back(l);
                }
            }
        }
    }

    std::ofstream out(path);
    if (out.is_open()) {
        for (const auto& l : lines) {
            out << l << "\n";
        }
        out << "InputMethod=" << config_method_ << "\n";
        out << "OutputCharset=" << config_charset_ << "\n";
        out << "AutoRestoreEnglish=" << (config_auto_restore_ ? "True" : "False") << "\n";
        out << "EnableMacro=" << (config_macro_ ? "True" : "False") << "\n";
        out << "SpellCheck=" << (config_spell_check_ ? "True" : "False") << "\n";
    }
}

void VnchkeyEngine::reloadConfig() {
    loadConfig();
    updateActionState();
    if (ctx_auto_) fcitx5_vnchkey_reset(ctx_auto_);
    if (ctx_telex_) fcitx5_vnchkey_reset(ctx_telex_);
    if (ctx_vni_) fcitx5_vnchkey_reset(ctx_vni_);
}

void* VnchkeyEngine::getContext(const std::string& entryName) {
    if (entryName.find("telex") != std::string::npos) {
        return ctx_telex_;
    } else if (entryName.find("vni") != std::string::npos) {
        return ctx_vni_;
    }

    // Default entry: obey user's preference in ~/.config/fcitx5/conf/vnchkey.conf
    if (config_method_ == "VNI" || config_method_ == "vni") {
        return ctx_vni_;
    } else if (config_method_ == "Telex" || config_method_ == "telex") {
        return ctx_telex_;
    } else {
        return ctx_auto_;
    }
}

void VnchkeyEngine::keyEvent(const fcitx::InputMethodEntry &entry, fcitx::KeyEvent &keyEvent) {
    if (is_forwarding_) {
        return;
    }

    if (keyEvent.isRelease()) {
        return;
    }

    auto rawKey = keyEvent.rawKey();
    auto states = rawKey.states();

    // Pass through combination shortcuts (Ctrl, Alt, Super) but allow Shift / CapsLock
    if (states.test(fcitx::KeyState::Ctrl) ||
        states.test(fcitx::KeyState::Alt) ||
        states.test(fcitx::KeyState::Super)) {
        reset(entry, keyEvent);
        return;
    }

    void* ctx = getContext(entry.uniqueName());
    if (!ctx) {
        return;
    }

    auto sym = rawKey.sym();

    // User pressed physical Backspace: let the app delete character naturally, sync engine state
    if (sym == FcitxKey_BackSpace) {
        if (rawKey.isVirtual() || pending_backspaces_ > 0) {
            if (pending_backspaces_ > 0) {
                --pending_backspaces_;
            }
            return;
        }
        fcitx5_vnchkey_handle_backspace(ctx);
        return;
    }

    // Reset on Enter or Escape
    if (sym == FcitxKey_Escape || sym == FcitxKey_Return) {
        reset(entry, keyEvent);
        return;
    }

    uint32_t ch = fcitx::Key::keySymToUnicode(sym);
    if (ch == 0) {
        return;
    }

    auto now = std::chrono::steady_clock::now();
    double timestamp_ms = std::chrono::duration<double, std::milli>(now.time_since_epoch()).count();

    size_t out_backspaces = 0;
    char* out_commit = nullptr;

    int consumed = fcitx5_vnchkey_process_key(ctx, ch, timestamp_ms, &out_backspaces, &out_commit);

    if (consumed) {
        auto* ic = keyEvent.inputContext();
        if (ic) {
            if (out_backspaces > 0) {
                pending_backspaces_ += out_backspaces;
                is_forwarding_ = true;
                for (size_t i = 0; i < out_backspaces; ++i) {
                    ic->forwardKey(fcitx::Key(FcitxKey_BackSpace));
                }
                is_forwarding_ = false;
            }

            // Commit transformed string
            if (out_commit && out_commit[0] != '\0') {
                ic->commitString(out_commit);
            }
        }

        keyEvent.filterAndAccept();
    }

    if (out_commit) {
        fcitx5_vnchkey_free_string(out_commit);
    }
}

void VnchkeyEngine::reset(const fcitx::InputMethodEntry &entry, fcitx::InputContextEvent &event) {
    FCITX_UNUSED(event);
    void* ctx = getContext(entry.uniqueName());
    if (ctx) {
        fcitx5_vnchkey_reset(ctx);
    }
}

fcitx::AddonInstance *VnchkeyEngineFactory::create(fcitx::AddonManager *manager) {
    return new VnchkeyEngine(manager);
}

FCITX_ADDON_FACTORY(VnchkeyEngineFactory);
