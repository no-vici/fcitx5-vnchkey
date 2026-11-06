#include "vnchkey_addon.h"
#include <fcitx-utils/keysym.h>
#include <fcitx-utils/key.h>
#include <fcitx/inputcontext.h>
#include <fcitx/statusarea.h>
#include <fcitx/userinterfacemanager.h>
#include <fcitx/instance.h>
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

    // Register all actions with Fcitx5 UserInterfaceManager for Status Area and Main Menu
    if (manager_ && manager_->instance()) {
        auto& uiMgr = manager_->instance()->userInterfaceManager();
        if (statusAction_) uiMgr.registerAction("vnchkey-status", statusAction_.get());
        if (actionOutputModeDirect_) uiMgr.registerAction("vnchkey-mode-direct", actionOutputModeDirect_.get());
        if (actionOutputModePreedit_) uiMgr.registerAction("vnchkey-mode-preedit", actionOutputModePreedit_.get());
        if (actionAutoRestore_) uiMgr.registerAction("vnchkey-autorestore", actionAutoRestore_.get());
        if (actionMacro_) uiMgr.registerAction("vnchkey-macro", actionMacro_.get());
        if (actionSpellCheck_) uiMgr.registerAction("vnchkey-spellcheck", actionSpellCheck_.get());
        if (actionDictionary_) uiMgr.registerAction("vnchkey-dictionary", actionDictionary_.get());
        if (actionSettings_) uiMgr.registerAction("vnchkey-settings", actionSettings_.get());
    }
}

VnchkeyEngine::~VnchkeyEngine() {
    if (manager_ && manager_->instance()) {
        auto& uiMgr = manager_->instance()->userInterfaceManager();
        if (statusAction_) uiMgr.unregisterAction(statusAction_.get());
        if (actionOutputModeDirect_) uiMgr.unregisterAction(actionOutputModeDirect_.get());
        if (actionOutputModePreedit_) uiMgr.unregisterAction(actionOutputModePreedit_.get());
        if (actionAutoRestore_) uiMgr.unregisterAction(actionAutoRestore_.get());
        if (actionMacro_) uiMgr.unregisterAction(actionMacro_.get());
        if (actionSpellCheck_) uiMgr.unregisterAction(actionSpellCheck_.get());
        if (actionDictionary_) uiMgr.unregisterAction(actionDictionary_.get());
        if (actionSettings_) uiMgr.unregisterAction(actionSettings_.get());
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
    // Main Status Action on Status Area: shows current method (Auto, Telex, VNI)
    statusAction_ = std::make_unique<fcitx::SimpleAction>();
    statusAction_->setIcon("vnchkey");
    statusAction_->setShortText(config_method_);
    statusAction_->setLongText("vnchKey (Tự Do Edition)");

    // Clicking the status action directly cycles between Auto -> Telex -> VNI
    connections_.push_back(statusAction_->connect<fcitx::SimpleAction::Activated>([this](fcitx::InputContext* ic) {
        if (config_method_ == "Auto" || config_method_ == "auto") {
            config_method_ = "Telex";
        } else if (config_method_ == "Telex" || config_method_ == "telex") {
            config_method_ = "VNI";
        } else {
            config_method_ = "Auto";
        }
        updateActionState();
        saveConfig();
        if (ic && statusAction_) {
            statusAction_->update(ic);
            ic->updateUserInterface(fcitx::UserInterfaceComponent::StatusArea);
        }
    }));

    statusMenu_ = std::make_unique<fcitx::Menu>();
    statusAction_->setMenu(statusMenu_.get());

    // 1. Input Method Group in Quick Menu
    actionMethodAuto_ = std::make_unique<fcitx::SimpleAction>();
    actionMethodAuto_->setCheckable(true);
    actionMethodAuto_->setLongText("Kiểu gõ: Tự Động (Auto Telex & VNI)");
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

    actionMethodTelex_ = std::make_unique<fcitx::SimpleAction>();
    actionMethodTelex_->setCheckable(true);
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

    actionMethodVni_ = std::make_unique<fcitx::SimpleAction>();
    actionMethodVni_->setCheckable(true);
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

    // Separator 1
    actionSeparator1_ = std::make_unique<fcitx::SimpleAction>();
    actionSeparator1_->setSeparator(true);
    statusMenu_->addAction(actionSeparator1_.get());

    // Output Mode (Direct vs Preedit)
    actionOutputModeDirect_ = std::make_unique<fcitx::SimpleAction>();
    actionOutputModeDirect_->setCheckable(true);
    actionOutputModeDirect_->setLongText("Chế độ xuất: Trực tiếp (Direct - Editor/Terminal)");
    connections_.push_back(actionOutputModeDirect_->connect<fcitx::SimpleAction::Activated>([this](fcitx::InputContext* ic) {
        setOutputModeForContext(ic, "Direct");
        updateActionState(ic);
        if (ic) {
            if (actionOutputModeDirect_) actionOutputModeDirect_->update(ic);
            if (actionOutputModePreedit_) actionOutputModePreedit_->update(ic);
            if (statusAction_) statusAction_->update(ic);
            ic->updateUserInterface(fcitx::UserInterfaceComponent::StatusArea);
        }
    }));
    statusMenu_->addAction(actionOutputModeDirect_.get());

    actionOutputModePreedit_ = std::make_unique<fcitx::SimpleAction>();
    actionOutputModePreedit_->setCheckable(true);
    actionOutputModePreedit_->setLongText("Chế độ xuất: Gạch chân (Preedit - Web/Google Docs)");
    connections_.push_back(actionOutputModePreedit_->connect<fcitx::SimpleAction::Activated>([this](fcitx::InputContext* ic) {
        setOutputModeForContext(ic, "Preedit");
        updateActionState(ic);
        if (ic) {
            if (actionOutputModeDirect_) actionOutputModeDirect_->update(ic);
            if (actionOutputModePreedit_) actionOutputModePreedit_->update(ic);
            if (statusAction_) statusAction_->update(ic);
            ic->updateUserInterface(fcitx::UserInterfaceComponent::StatusArea);
        }
    }));
    statusMenu_->addAction(actionOutputModePreedit_.get());

    // 2. Output Charset (Pure standard Unicode only - TCVN3 removed as requested)
    actionCharsetUnicode_ = std::make_unique<fcitx::SimpleAction>();
    actionCharsetUnicode_->setCheckable(true);
    actionCharsetUnicode_->setChecked(true);
    actionCharsetUnicode_->setLongText("Bảng mã: Unicode Chuẩn");
    statusMenu_->addAction(actionCharsetUnicode_.get());

    // Separator 2
    actionSeparator2_ = std::make_unique<fcitx::SimpleAction>();
    actionSeparator2_->setSeparator(true);
    statusMenu_->addAction(actionSeparator2_.get());

    // 3. Toggles: Restore English, Macro, Chánh Tả, Custom Dictionary (with Lilypad ✔ / ✖ style)
    actionAutoRestore_ = std::make_unique<fcitx::SimpleAction>();
    actionAutoRestore_->setCheckable(true);
    actionAutoRestore_->setLongText("Khôi Phục Tiếng Anh (Restore English)");
    connections_.push_back(actionAutoRestore_->connect<fcitx::SimpleAction::Activated>([this](fcitx::InputContext* ic) {
        config_auto_restore_ = !config_auto_restore_;
        for (void* ctx : {ctx_auto_, ctx_telex_, ctx_vni_}) {
            if (ctx) fcitx5_vnchkey_set_auto_restore(ctx, config_auto_restore_);
        }
        updateActionState();
        saveConfig();
        if (ic && actionAutoRestore_) {
            actionAutoRestore_->update(ic);
            ic->updateUserInterface(fcitx::UserInterfaceComponent::StatusArea);
        }
    }));
    statusMenu_->addAction(actionAutoRestore_.get());

    actionMacro_ = std::make_unique<fcitx::SimpleAction>();
    actionMacro_->setCheckable(true);
    actionMacro_->setLongText("Bật Gõ Tắt (Macro)");
    connections_.push_back(actionMacro_->connect<fcitx::SimpleAction::Activated>([this](fcitx::InputContext* ic) {
        config_macro_ = !config_macro_;
        for (void* ctx : {ctx_auto_, ctx_telex_, ctx_vni_}) {
            if (ctx) fcitx5_vnchkey_set_macro_enabled(ctx, config_macro_);
        }
        updateActionState();
        saveConfig();
        if (ic && actionMacro_) {
            actionMacro_->update(ic);
            ic->updateUserInterface(fcitx::UserInterfaceComponent::StatusArea);
        }
    }));
    statusMenu_->addAction(actionMacro_.get());

    actionSpellCheck_ = std::make_unique<fcitx::SimpleAction>();
    actionSpellCheck_->setCheckable(true);
    actionSpellCheck_->setLongText("Kiểm Tra Chánh Tả");
    connections_.push_back(actionSpellCheck_->connect<fcitx::SimpleAction::Activated>([this](fcitx::InputContext* ic) {
        config_spell_check_ = !config_spell_check_;
        for (void* ctx : {ctx_auto_, ctx_telex_, ctx_vni_}) {
            if (ctx) fcitx5_vnchkey_set_spell_check(ctx, config_spell_check_);
        }
        updateActionState();
        saveConfig();
        if (ic && actionSpellCheck_) {
            actionSpellCheck_->update(ic);
            ic->updateUserInterface(fcitx::UserInterfaceComponent::StatusArea);
        }
    }));
    statusMenu_->addAction(actionSpellCheck_.get());

    actionDictionary_ = std::make_unique<fcitx::SimpleAction>();
    actionDictionary_->setCheckable(true);
    actionDictionary_->setLongText("Từ Điển Cá Nhân (Custom Dictionary)");
    connections_.push_back(actionDictionary_->connect<fcitx::SimpleAction::Activated>([this](fcitx::InputContext* ic) {
        config_dictionary_ = !config_dictionary_;
        for (void* ctx : {ctx_auto_, ctx_telex_, ctx_vni_}) {
            if (ctx) fcitx5_vnchkey_set_dictionary_enabled(ctx, config_dictionary_);
        }
        updateActionState();
        saveConfig();
        if (ic && actionDictionary_) {
            actionDictionary_->update(ic);
            ic->updateUserInterface(fcitx::UserInterfaceComponent::StatusArea);
        }
    }));
    statusMenu_->addAction(actionDictionary_.get());

    // Separator 3
    actionSeparator3_ = std::make_unique<fcitx::SimpleAction>();
    actionSeparator3_->setSeparator(true);
    statusMenu_->addAction(actionSeparator3_.get());

    // 4. Launch Settings GUI
    actionSettings_ = std::make_unique<fcitx::SimpleAction>();
    actionSettings_->setShortText("Cài Đặt");
    actionSettings_->setLongText("Cài Đặt Fcitx5 vnchKey...");
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

void VnchkeyEngine::updateActionState(fcitx::InputContext* ic) {
    if (statusAction_) {
        statusAction_->setShortText(config_method_);
    }

    const bool isAuto = (config_method_ == "Auto" || config_method_ == "auto");
    const bool isTelex = (config_method_ == "Telex" || config_method_ == "telex");
    const bool isVni = (config_method_ == "VNI" || config_method_ == "vni");

    if (actionMethodAuto_) {
        actionMethodAuto_->setChecked(isAuto);
        actionMethodAuto_->setShortText(std::string(isAuto ? "✔ " : "   ") + "Tự Động (Auto)");
    }
    if (actionMethodTelex_) {
        actionMethodTelex_->setChecked(isTelex);
        actionMethodTelex_->setShortText(std::string(isTelex ? "✔ " : "   ") + "Telex");
    }
    if (actionMethodVni_) {
        actionMethodVni_->setChecked(isVni);
        actionMethodVni_->setShortText(std::string(isVni ? "✔ " : "   ") + "VNI");
    }

    std::string current_mode = ic ? getOutputModeForContext(ic) : config_output_mode_;
    const bool isPreedit = (current_mode == "Preedit" || current_mode == "preedit");
    if (actionOutputModeDirect_) {
        actionOutputModeDirect_->setChecked(!isPreedit);
        actionOutputModeDirect_->setShortText(std::string(!isPreedit ? "✔ " : "   ") + "Gõ Trực Tiếp (Direct)");
    }
    if (actionOutputModePreedit_) {
        actionOutputModePreedit_->setChecked(isPreedit);
        actionOutputModePreedit_->setShortText(std::string(isPreedit ? "✔ " : "   ") + "Gõ Gạch Chân (Preedit)");
    }

    if (actionCharsetUnicode_) {
        actionCharsetUnicode_->setChecked(true);
        actionCharsetUnicode_->setShortText("✔ Unicode");
    }

    // Match fcitx5-lilypad style: prefix toggle items with "✔ " when enabled and "✖ " when disabled
    if (actionAutoRestore_) {
        actionAutoRestore_->setChecked(config_auto_restore_);
        actionAutoRestore_->setShortText(std::string(config_auto_restore_ ? "✔ " : "✖ ") + "Restore English");
    }
    if (actionMacro_) {
        actionMacro_->setChecked(config_macro_);
        actionMacro_->setShortText(std::string(config_macro_ ? "✔ " : "✖ ") + "Macro");
    }
    if (actionSpellCheck_) {
        actionSpellCheck_->setChecked(config_spell_check_);
        actionSpellCheck_->setShortText(std::string(config_spell_check_ ? "✔ " : "✖ ") + "Chánh Tả");
    }
    if (actionDictionary_) {
        actionDictionary_->setChecked(config_dictionary_);
        actionDictionary_->setShortText(std::string(config_dictionary_ ? "✔ " : "✖ ") + "Custom Dictionary");
    }
}

void VnchkeyEngine::activate(const fcitx::InputMethodEntry &entry, fcitx::InputContextEvent &event) {
    FCITX_UNUSED(entry);
    if (ctx_auto_) fcitx5_vnchkey_reset(ctx_auto_);
    if (ctx_telex_) fcitx5_vnchkey_reset(ctx_telex_);
    if (ctx_vni_) fcitx5_vnchkey_reset(ctx_vni_);
    auto* ic = event.inputContext();
    if (ic) {
        updateActionState(ic);
        ic->statusArea().clearGroup(fcitx::StatusGroup::InputMethod);
        if (actionOutputModeDirect_) ic->statusArea().addAction(fcitx::StatusGroup::InputMethod, actionOutputModeDirect_.get());
        if (actionOutputModePreedit_) ic->statusArea().addAction(fcitx::StatusGroup::InputMethod, actionOutputModePreedit_.get());
        if (actionAutoRestore_) ic->statusArea().addAction(fcitx::StatusGroup::InputMethod, actionAutoRestore_.get());
        if (actionMacro_) ic->statusArea().addAction(fcitx::StatusGroup::InputMethod, actionMacro_.get());
        if (actionSpellCheck_) ic->statusArea().addAction(fcitx::StatusGroup::InputMethod, actionSpellCheck_.get());
        if (actionDictionary_) ic->statusArea().addAction(fcitx::StatusGroup::InputMethod, actionDictionary_.get());
        if (actionSettings_) ic->statusArea().addAction(fcitx::StatusGroup::InputMethod, actionSettings_.get());

        if (actionOutputModeDirect_) actionOutputModeDirect_->update(ic);
        if (actionOutputModePreedit_) actionOutputModePreedit_->update(ic);
        if (actionAutoRestore_) actionAutoRestore_->update(ic);
        if (actionMacro_) actionMacro_->update(ic);
        if (actionSpellCheck_) actionSpellCheck_->update(ic);
        if (actionDictionary_) actionDictionary_->update(ic);
        if (actionSettings_) actionSettings_->update(ic);

        ic->updateUserInterface(fcitx::UserInterfaceComponent::StatusArea);
    }
}

void VnchkeyEngine::deactivate(const fcitx::InputMethodEntry &entry, fcitx::InputContextEvent &event) {
    FCITX_UNUSED(entry);
    if (ctx_auto_) fcitx5_vnchkey_reset(ctx_auto_);
    if (ctx_telex_) fcitx5_vnchkey_reset(ctx_telex_);
    if (ctx_vni_) fcitx5_vnchkey_reset(ctx_vni_);
    auto* ic = event.inputContext();
    if (ic) {
        ic->inputPanel().reset();
        ic->updatePreedit();
        ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
        ic->statusArea().clearGroup(fcitx::StatusGroup::InputMethod);
        ic->updateUserInterface(fcitx::UserInterfaceComponent::StatusArea);
    }
}

void VnchkeyEngine::loadAppModes() {
    app_output_modes_.clear();
    const char* home = getenv("HOME");
    if (!home) return;
    std::string path = std::string(home) + "/.config/fcitx5/conf/vnchkey-apps.conf";
    std::ifstream file(path);
    if (!file.is_open()) return;
    std::string line;
    while (std::getline(file, line)) {
        if (line.empty() || line[0] == '#') continue;
        auto pos = line.find('=');
        if (pos != std::string::npos) {
            std::string app = line.substr(0, pos);
            std::string mode = line.substr(pos + 1);
            size_t a_start = app.find_first_not_of(" \t");
            size_t a_end = app.find_last_not_of(" \t");
            size_t m_start = mode.find_first_not_of(" \t");
            size_t m_end = mode.find_last_not_of(" \t");
            if (a_start != std::string::npos && m_start != std::string::npos) {
                app_output_modes_[app.substr(a_start, a_end - a_start + 1)] = mode.substr(m_start, m_end - m_start + 1);
            }
        }
    }
}

void VnchkeyEngine::saveAppModes() {
    const char* home = getenv("HOME");
    if (!home) return;
    std::string path = std::string(home) + "/.config/fcitx5/conf/vnchkey-apps.conf";
    std::ofstream out(path);
    if (!out.is_open()) return;
    out << "# vnchKey Per-Application Modes\n";
    for (const auto& pair : app_output_modes_) {
        out << pair.first << "=" << pair.second << "\n";
    }
}

std::string VnchkeyEngine::getOutputModeForContext(fcitx::InputContext* ic) {
    if (ic && config_remember_app_mode_) {
        std::string prog = ic->program();
        if (!prog.empty()) {
            auto it = app_output_modes_.find(prog);
            if (it != app_output_modes_.end()) {
                return it->second;
            }
            std::string lower_prog = prog;
            for (char& c : lower_prog) c = std::tolower(c);
            if (lower_prog.find("chrome") != std::string::npos ||
                lower_prog.find("firefox") != std::string::npos ||
                lower_prog.find("chromium") != std::string::npos ||
                lower_prog.find("brave") != std::string::npos ||
                lower_prog.find("edge") != std::string::npos ||
                lower_prog.find("opera") != std::string::npos ||
                lower_prog.find("vivaldi") != std::string::npos ||
                lower_prog.find("slack") != std::string::npos ||
                lower_prog.find("discord") != std::string::npos ||
                lower_prog.find("telegram") != std::string::npos ||
                lower_prog.find("teams") != std::string::npos ||
                lower_prog.find("electron") != std::string::npos) {
                return "Preedit";
            }
            if (lower_prog.find("alacritty") != std::string::npos ||
                lower_prog.find("kitty") != std::string::npos ||
                lower_prog.find("wezterm") != std::string::npos ||
                lower_prog.find("terminal") != std::string::npos ||
                lower_prog.find("code") != std::string::npos ||
                lower_prog.find("sublime") != std::string::npos ||
                lower_prog.find("nvim") != std::string::npos ||
                lower_prog.find("emacs") != std::string::npos ||
                lower_prog.find("idea") != std::string::npos) {
                return "Direct";
            }
        }
    }
    return config_output_mode_;
}

void VnchkeyEngine::setOutputModeForContext(fcitx::InputContext* ic, const std::string& mode) {
    if (ic && config_remember_app_mode_) {
        std::string prog = ic->program();
        if (!prog.empty()) {
            app_output_modes_[prog] = mode;
            saveAppModes();
        }
    }
    config_output_mode_ = mode;
    saveConfig();
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
            } else if (line.rfind("OutputMode=", 0) == 0) {
                std::string val = line.substr(11);
                size_t start = val.find_first_not_of(" \t\"'");
                size_t end = val.find_last_not_of(" \t\"'");
                if (start != std::string::npos && end != std::string::npos) {
                    config_output_mode_ = val.substr(start, end - start + 1);
                }
            } else if (line.rfind("RememberAppMode=", 0) == 0) {
                config_remember_app_mode_ = (line.find("False") == std::string::npos);
            } else if (line.rfind("AutoRestoreEnglish=", 0) == 0) {
                config_auto_restore_ = (line.find("False") == std::string::npos);
            } else if (line.rfind("EnableMacro=", 0) == 0) {
                config_macro_ = (line.find("False") == std::string::npos);
            } else if (line.rfind("SpellCheck=", 0) == 0) {
                config_spell_check_ = (line.find("False") == std::string::npos);
            } else if (line.rfind("EnableDictionary=", 0) == 0) {
                config_dictionary_ = (line.find("False") == std::string::npos);
            }
        }
    }

    loadAppModes();

    // Load macro table & user dictionary if configured
    std::string macro_path = std::string(home) + "/.config/fcitx5/conf/vnchkey-macro-table.conf";
    std::string dict_path = std::string(home) + "/.config/fcitx5/conf/vnchkey-dictionary.conf";

    for (void* ctx : {ctx_auto_, ctx_telex_, ctx_vni_}) {
        if (!ctx) continue;
        fcitx5_vnchkey_load_macros(ctx, macro_path.c_str());
        fcitx5_vnchkey_load_dictionary(ctx, dict_path.c_str());
        fcitx5_vnchkey_set_auto_restore(ctx, config_auto_restore_);
        fcitx5_vnchkey_set_macro_enabled(ctx, config_macro_);
        fcitx5_vnchkey_set_spell_check(ctx, config_spell_check_);
        fcitx5_vnchkey_set_dictionary_enabled(ctx, config_dictionary_);
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
                    l.rfind("OutputMode=", 0) != 0 &&
                    l.rfind("RememberAppMode=", 0) != 0 &&
                    l.rfind("AutoRestoreEnglish=", 0) != 0 &&
                    l.rfind("EnableMacro=", 0) != 0 &&
                    l.rfind("SpellCheck=", 0) != 0 &&
                    l.rfind("EnableDictionary=", 0) != 0) {
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
        out << "OutputCharset=Unicode\n";
        out << "OutputMode=" << config_output_mode_ << "\n";
        out << "RememberAppMode=" << (config_remember_app_mode_ ? "True" : "False") << "\n";
        out << "AutoRestoreEnglish=" << (config_auto_restore_ ? "True" : "False") << "\n";
        out << "EnableMacro=" << (config_macro_ ? "True" : "False") << "\n";
        out << "SpellCheck=" << (config_spell_check_ ? "True" : "False") << "\n";
        out << "EnableDictionary=" << (config_dictionary_ ? "True" : "False") << "\n";
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
    FCITX_UNUSED(entryName);
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

    if (keyEvent.isRelease() || keyEvent.rawKey().isModifier()) {
        return;
    }

    void* ctx = getContext(entry.uniqueName());
    if (!ctx) {
        return;
    }

    auto* ic = keyEvent.inputContext();
    std::string mode = getOutputModeForContext(ic);
    const bool isPreeditMode = (mode == "Preedit" || mode == "preedit");

    auto rawKey = keyEvent.rawKey();
    auto states = rawKey.states();
    auto sym = rawKey.sym();

    // Pass through combination shortcuts (Ctrl, Alt, Super) and navigation/cursor keys
    if (states.test(fcitx::KeyState::Ctrl) ||
        states.test(fcitx::KeyState::Alt) ||
        states.test(fcitx::KeyState::Super) ||
        keyEvent.key().isCursorMove() ||
        sym == FcitxKey_Tab ||
        sym == FcitxKey_KP_Tab ||
        sym == FcitxKey_ISO_Left_Tab ||
        sym == FcitxKey_Delete) {
        if (isPreeditMode && ic && fcitx5_vnchkey_has_active_word(ctx)) {
            // Commit preedit buffer before cursor navigation
            char* cur = fcitx5_vnchkey_get_current_word(ctx);
            if (cur && cur[0] != '\0') {
                ic->commitString(cur);
                fcitx5_vnchkey_free_string(cur);
            }
            ic->inputPanel().reset();
            ic->updatePreedit();
            ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
        }
        fcitx5_vnchkey_reset(ctx);
        return;
    }

    // Escape key
    if (sym == FcitxKey_Escape) {
        if (isPreeditMode && ic && fcitx5_vnchkey_has_active_word(ctx)) {
            // Cancel preedit composition without committing
            fcitx5_vnchkey_reset(ctx);
            ic->inputPanel().reset();
            ic->updatePreedit();
            ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
            keyEvent.filterAndAccept();
            return;
        }
        fcitx5_vnchkey_reset(ctx);
        return;
    }

    // Return/Enter key
    if (sym == FcitxKey_Return || sym == FcitxKey_KP_Enter) {
        if (isPreeditMode && ic && fcitx5_vnchkey_has_active_word(ctx)) {
            // Enter commits current preedit word
            char* cur = fcitx5_vnchkey_get_current_word(ctx);
            if (cur && cur[0] != '\0') {
                ic->commitString(cur);
                fcitx5_vnchkey_free_string(cur);
            }
            ic->inputPanel().reset();
            ic->updatePreedit();
            ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
            fcitx5_vnchkey_reset(ctx);
            keyEvent.filterAndAccept();
            return;
        }
        fcitx5_vnchkey_reset(ctx);
        return;
    }

    // Backspace handling
    if (sym == FcitxKey_BackSpace) {
        if (rawKey.isVirtual()) {
            return;
        }
        if (isPreeditMode && ic && fcitx5_vnchkey_has_active_word(ctx)) {
            fcitx5_vnchkey_handle_backspace(ctx);
            if (fcitx5_vnchkey_has_active_word(ctx)) {
                char* cur = fcitx5_vnchkey_get_current_word(ctx);
                if (cur && cur[0] != '\0') {
                    fcitx::Text text(cur, fcitx::TextFormatFlag::Underline);
                    ic->inputPanel().setClientPreedit(text);
                    ic->updatePreedit();
                    ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
                    fcitx5_vnchkey_free_string(cur);
                } else {
                    ic->inputPanel().reset();
                    ic->updatePreedit();
                    ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
                }
            } else {
                ic->inputPanel().reset();
                ic->updatePreedit();
                ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
            }
            keyEvent.filterAndAccept();
            return;
        }

        // Direct Mode: let the app delete character naturally, sync engine state
        fcitx5_vnchkey_handle_backspace(ctx);
        return;
    }

    // Normalize numpad digits and space (matching fcitx5-lilypad)
    if (sym >= FcitxKey_KP_0 && sym <= FcitxKey_KP_9) {
        sym = static_cast<fcitx::KeySym>(FcitxKey_0 + (sym - FcitxKey_KP_0));
    } else if (sym == FcitxKey_KP_Space) {
        sym = FcitxKey_space;
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

    if (isPreeditMode && ic) {
        char* completed = fcitx5_vnchkey_get_completed_word(ctx);
        if (completed) {
            // Word finalized (delimiter, punctuation, macro expansion)
            ic->inputPanel().reset();
            ic->updatePreedit();
            ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
            if (completed[0] != '\0') {
                ic->commitString(completed);
            }
            fcitx5_vnchkey_free_string(completed);
            keyEvent.filterAndAccept();
        } else if (fcitx5_vnchkey_has_active_word(ctx)) {
            // Active word being composed inline with underline
            char* cur = fcitx5_vnchkey_get_current_word(ctx);
            if (cur && cur[0] != '\0') {
                fcitx::Text text(cur, fcitx::TextFormatFlag::Underline);
                ic->inputPanel().setClientPreedit(text);
                ic->updatePreedit();
                ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
                fcitx5_vnchkey_free_string(cur);
            }
            keyEvent.filterAndAccept();
        } else {
            ic->inputPanel().reset();
            ic->updatePreedit();
            ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
            if (consumed) {
                if (out_commit && out_commit[0] != '\0') {
                    ic->commitString(out_commit);
                }
                keyEvent.filterAndAccept();
            }
        }
    } else {
        // Direct Mode: micro-diff backspaces + suffix commit
        if (consumed) {
            if (ic) {
                if (out_backspaces > 0) {
                    if (ic->capabilityFlags().test(fcitx::CapabilityFlag::SurroundingText) &&
                        ic->surroundingText().isValid()) {
                        ic->deleteSurroundingText(-static_cast<int>(out_backspaces), out_backspaces);
                    } else {
                        is_forwarding_ = true;
                        for (size_t i = 0; i < out_backspaces; ++i) {
                            ic->forwardKey(fcitx::Key(FcitxKey_BackSpace));
                        }
                        is_forwarding_ = false;
                    }
                }

                if (out_commit && out_commit[0] != '\0') {
                    ic->commitString(out_commit);
                }
            }

            keyEvent.filterAndAccept();
        }
    }

    if (out_commit) {
        fcitx5_vnchkey_free_string(out_commit);
    }
}

void VnchkeyEngine::reset(const fcitx::InputMethodEntry &entry, fcitx::InputContextEvent &event) {
    if (event.type() != fcitx::EventType::InputContextFocusOut) {
        return;
    }
    auto* ic = event.inputContext();
    if (ic) {
        ic->inputPanel().reset();
        ic->updatePreedit();
        ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
    }
    void* ctx = getContext(entry.uniqueName());
    if (ctx) {
        fcitx5_vnchkey_reset(ctx);
    }
}

fcitx::AddonInstance *VnchkeyEngineFactory::create(fcitx::AddonManager *manager) {
    return new VnchkeyEngine(manager);
}

FCITX_ADDON_FACTORY(VnchkeyEngineFactory);
