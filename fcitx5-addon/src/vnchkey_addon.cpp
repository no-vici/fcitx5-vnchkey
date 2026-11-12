#include "vnchkey_addon.h"
#include <fcitx-utils/keysym.h>
#include <fcitx-utils/key.h>
#include <fcitx-utils/misc.h>
#include <fcitx/inputcontext.h>
#include <fcitx/inputcontextmanager.h>
#include <fcitx/statusarea.h>
#include <fcitx/userinterfacemanager.h>
#include <fcitx/instance.h>
#include <chrono>
#include <thread>
#include <string>
#include <fstream>
#include <cstdlib>
#include <unistd.h>
#include <spawn.h>

extern char **environ;

static void launchVnchkeySettings() {
    // 1. Try absolute standard path first with posix_spawnp (avoids PATH environment issues)
    pid_t pid;
    char* const argv1[] = {(char*)"fcitx5-vnchkey-settings", nullptr};
    if (posix_spawnp(&pid, "/usr/bin/fcitx5-vnchkey-settings", nullptr, nullptr, argv1, environ) == 0) {
        return;
    }
    char* const argv2[] = {(char*)"vnchkey-settings", nullptr};
    if (posix_spawnp(&pid, "/usr/bin/vnchkey-settings", nullptr, nullptr, argv2, environ) == 0) {
        return;
    }

    // 2. Try looking up in PATH
    if (posix_spawnp(&pid, "fcitx5-vnchkey-settings", nullptr, nullptr, argv1, environ) == 0) {
        return;
    }
    if (posix_spawnp(&pid, "vnchkey-settings", nullptr, nullptr, argv2, environ) == 0) {
        return;
    }

    // 3. Last fallback: fcitx::startProcess
    fcitx::startProcess({"fcitx5-vnchkey-settings"});
}

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
        if (actionPreeditTooltip_) uiMgr.registerAction("vnchkey-preedit-tooltip", actionPreeditTooltip_.get());
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
        if (actionPreeditTooltip_) uiMgr.unregisterAction(actionPreeditTooltip_.get());
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
        if (!ic && manager_ && manager_->instance()) {
            ic = manager_->instance()->inputContextManager().mostRecentInputContext();
            if (!ic) ic = manager_->instance()->inputContextManager().lastFocusedInputContext();
        }
        setOutputModeForContext(ic, "Direct");
        updateActionState(ic);
        if (actionOutputModeDirect_) actionOutputModeDirect_->update(ic);
        if (actionOutputModePreedit_) actionOutputModePreedit_->update(ic);
        if (statusAction_) statusAction_->update(ic);
        if (ic) {
            ic->updateUserInterface(fcitx::UserInterfaceComponent::StatusArea);
        }
    }));
    statusMenu_->addAction(actionOutputModeDirect_.get());

    actionOutputModePreedit_ = std::make_unique<fcitx::SimpleAction>();
    actionOutputModePreedit_->setCheckable(true);
    actionOutputModePreedit_->setLongText("Chế độ xuất: Gạch chân (Preedit - Web/Google Docs)");
    connections_.push_back(actionOutputModePreedit_->connect<fcitx::SimpleAction::Activated>([this](fcitx::InputContext* ic) {
        if (!ic && manager_ && manager_->instance()) {
            ic = manager_->instance()->inputContextManager().mostRecentInputContext();
            if (!ic) ic = manager_->instance()->inputContextManager().lastFocusedInputContext();
        }
        setOutputModeForContext(ic, "Preedit");
        updateActionState(ic);
        if (actionOutputModeDirect_) actionOutputModeDirect_->update(ic);
        if (actionOutputModePreedit_) actionOutputModePreedit_->update(ic);
        if (actionPreeditTooltip_) actionPreeditTooltip_->update(ic);
        if (statusAction_) statusAction_->update(ic);
        if (ic) {
            ic->updateUserInterface(fcitx::UserInterfaceComponent::StatusArea);
        }
    }));
    statusMenu_->addAction(actionOutputModePreedit_.get());

    actionPreeditTooltip_ = std::make_unique<fcitx::SimpleAction>();
    actionPreeditTooltip_->setCheckable(true);
    actionPreeditTooltip_->setLongText("Khung chữ nổi khi gõ gạch chân (Floating Preedit Tooltip)");
    connections_.push_back(actionPreeditTooltip_->connect<fcitx::SimpleAction::Activated>([this](fcitx::InputContext* ic) {
        config_preedit_tooltip_ = !config_preedit_tooltip_;
        updateActionState(ic);
        saveConfig();
        if (ic) {
            if (actionPreeditTooltip_) actionPreeditTooltip_->update(ic);
            ic->updateUserInterface(fcitx::UserInterfaceComponent::StatusArea);
        }
    }));
    statusMenu_->addAction(actionPreeditTooltip_.get());

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

    // 4. Launch Settings GUI safely without deadlock in multi-threaded event loop
    actionSettings_ = std::make_unique<fcitx::SimpleAction>();
    actionSettings_->setShortText("Cài Đặt");
    actionSettings_->setLongText("Cài Đặt Fcitx5 vnchKey...");
    actionSettings_->setIcon("preferences-system");
    connections_.push_back(actionSettings_->connect<fcitx::SimpleAction::Activated>([](fcitx::InputContext*) {
        launchVnchkeySettings();
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
    if (actionPreeditTooltip_) {
        actionPreeditTooltip_->setChecked(config_preedit_tooltip_);
        actionPreeditTooltip_->setShortText(std::string(config_preedit_tooltip_ ? "✔ " : "✖ ") + "Khung Nổi (Tooltip)");
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
    last_rescue_raw_.clear();
    last_rescue_converted_.clear();

    auto* ic = event.inputContext();
    if (ic) {
        // Feature: On-Activation Word Rescue (Ngó Sen style Cứu Chữ)
        // If the user forgot to enable Vietnamese IME and typed unaccented text (e.g. "tieengs vieetj nef"),
        // when activating vnchKey, automatically convert it to accented Vietnamese!
        if (config_word_rescue_ &&
            ic->capabilityFlags().test(fcitx::CapabilityFlag::SurroundingText) &&
            ic->surroundingText().isValid()) {
            const auto& surrounding = ic->surroundingText();
            if (surrounding.cursor() == surrounding.anchor() && surrounding.cursor() > 0) {
                const std::string& full_text = surrounding.text();
                // Map character index of cursor to byte index
                size_t byte_cursor = 0;
                size_t char_count = 0;
                for (size_t i = 0; i < full_text.length(); ++i) {
                    if (char_count == surrounding.cursor()) {
                        byte_cursor = i;
                        break;
                    }
                    if ((static_cast<unsigned char>(full_text[i]) & 0xC0) != 0x80) {
                        char_count++;
                    }
                }
                if (char_count == surrounding.cursor() && byte_cursor == 0) {
                    byte_cursor = full_text.length();
                }

                std::string before_caret = full_text.substr(0, byte_cursor);
                size_t last_nl = before_caret.rfind('\n');
                size_t start_idx = (last_nl != std::string::npos) ? (last_nl + 1) : 0;
                if (before_caret.length() - start_idx > 80) {
                    start_idx = before_caret.length() - 80;
                    size_t sp = before_caret.find(' ', start_idx);
                    if (sp != std::string::npos && sp < before_caret.length()) {
                        start_idx = sp + 1;
                    }
                }

                std::string cand_raw = before_caret.substr(start_idx);
                size_t lead_sp = cand_raw.find_first_not_of(" \t");
                if (lead_sp != std::string::npos) {
                    cand_raw = cand_raw.substr(lead_sp);
                }

                if (!cand_raw.empty() && cand_raw.length() >= 2) {
                    void* ctx = getContext(entry.uniqueName());
                    char* rescued = nullptr;
                    if (ctx && fcitx5_vnchkey_rescue_sentence(ctx, cand_raw.c_str(), &rescued)) {
                        if (rescued && std::string(rescued) != cand_raw) {
                            size_t cand_char_len = utf8CharCount(cand_raw);
                            ic->deleteSurroundingText(-static_cast<int>(cand_char_len), cand_char_len);
                            ic->commitString(rescued);
                            last_rescue_raw_ = cand_raw;
                            last_rescue_converted_ = rescued;
                        }
                        if (rescued) {
                            fcitx5_vnchkey_free_string(rescued);
                        }
                    }
                }
            }
        }

        updateActionState(ic);
        ic->statusArea().clearGroup(fcitx::StatusGroup::InputMethod);
        if (actionOutputModeDirect_) ic->statusArea().addAction(fcitx::StatusGroup::InputMethod, actionOutputModeDirect_.get());
        if (actionOutputModePreedit_) ic->statusArea().addAction(fcitx::StatusGroup::InputMethod, actionOutputModePreedit_.get());
        if (actionPreeditTooltip_) ic->statusArea().addAction(fcitx::StatusGroup::InputMethod, actionPreeditTooltip_.get());
        if (actionAutoRestore_) ic->statusArea().addAction(fcitx::StatusGroup::InputMethod, actionAutoRestore_.get());
        if (actionMacro_) ic->statusArea().addAction(fcitx::StatusGroup::InputMethod, actionMacro_.get());
        if (actionSpellCheck_) ic->statusArea().addAction(fcitx::StatusGroup::InputMethod, actionSpellCheck_.get());
        if (actionDictionary_) ic->statusArea().addAction(fcitx::StatusGroup::InputMethod, actionDictionary_.get());
        if (actionSettings_) ic->statusArea().addAction(fcitx::StatusGroup::InputMethod, actionSettings_.get());

        if (actionOutputModeDirect_) actionOutputModeDirect_->update(ic);
        if (actionOutputModePreedit_) actionOutputModePreedit_->update(ic);
        if (actionPreeditTooltip_) actionPreeditTooltip_->update(ic);
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

    auto read_conf = [this](const std::string& path) {
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
                    std::string a = app.substr(a_start, a_end - a_start + 1);
                    std::string m = mode.substr(m_start, m_end - m_start + 1);
                    app_output_modes_[a] = m;
                }
            }
        }
    };

    read_conf(std::string(home) + "/.config/fcitx5/conf/vnchkey-apps.conf");
    read_conf(std::string(home) + "/.config/fcitx5/conf/vnchkey-app-modes.conf");
}

void VnchkeyEngine::saveAppModes() {
    const char* home = getenv("HOME");
    if (!home) return;
    auto write_conf = [this](const std::string& path) {
        std::ofstream out(path);
        if (!out.is_open()) return;
        out << "# vnchKey Per-Application Modes\n";
        for (const auto& pair : app_output_modes_) {
            out << pair.first << "=" << pair.second << "\n";
        }
    };
    write_conf(std::string(home) + "/.config/fcitx5/conf/vnchkey-apps.conf");
    write_conf(std::string(home) + "/.config/fcitx5/conf/vnchkey-app-modes.conf");
}

static bool isWpsOffice(const std::string& prog) {
    if (prog.empty()) return false;
    std::string lower = prog;
    const auto slash = lower.find_last_of('/');
    if (slash != std::string::npos) {
        lower = lower.substr(slash + 1);
    }
    for (char& c : lower) c = std::tolower(c);
    return (lower == "wps" || lower == "wpp" || lower == "et" || lower == "wpspdf");
}

static bool isSensitiveContext(fcitx::InputContext* ic) {
    if (!ic) return false;
    auto flags = ic->capabilityFlags();
    return flags.test(fcitx::CapabilityFlag::Password) ||
           flags.test(fcitx::CapabilityFlag::Sensitive) ||
           flags.test(fcitx::CapabilityFlag::Number) ||
           flags.test(fcitx::CapabilityFlag::Digit) ||
           flags.test(fcitx::CapabilityFlag::Dialable) ||
           flags.test(fcitx::CapabilityFlag::Alpha);
}

static bool isGamingProgram(const std::string& lower_prog) {
    if (lower_prog.empty()) return false;
    static const char* const kGamingPatterns[] = {
        "steam", "gamescope", "lutris", "heroic", "bottles", "prismlauncher",
        "minecraft", "java-minecraft", "retroarch", "rpcs3", "yuzu", "ryujinx",
        "pcsx2", "dolphin-emu", "ppsspp", "duckstation", "cemu", "citra",
        "cs2", "csgo", "dota2", "hl2", "tf_linux64", "valorant", "league",
        "genshin", "starrail", "zenless", "eldenring", "cyberpunk", "witcher",
        "baldursgate", "bg3", "factorio", "terraria", "stardew", "valheim",
        "overwatch", "apex", "r5apex", "roblox", "sober", "osu!", "osu",
        "unreal", "unity", "godot"
    };
    for (const char* pat : kGamingPatterns) {
        if (lower_prog == pat || lower_prog.find(pat) != std::string::npos) {
            return true;
        }
    }
    // Check common Wine/Proton game executable patterns (excluding productivity apps like Zalo/Notepad)
    if ((lower_prog.find("wine64-preloader") != std::string::npos ||
         lower_prog.find("wine-preloader") != std::string::npos ||
         lower_prog.find("proton") != std::string::npos ||
         (lower_prog.size() > 4 && lower_prog.substr(lower_prog.size() - 4) == ".exe")) &&
        lower_prog.find("zalo") == std::string::npos &&
        lower_prog.find("notepad") == std::string::npos &&
        lower_prog.find("unikey") == std::string::npos &&
        lower_prog.find("office") == std::string::npos &&
        lower_prog.find("word") == std::string::npos &&
        lower_prog.find("excel") == std::string::npos) {
        return true;
    }
    return false;
}

std::string VnchkeyEngine::getOutputModeForContext(fcitx::InputContext* ic) {
    if (ic) {
        std::string prog = ic->program();
        // WPS Office does not implement surrounding text editing and hides preedit
        if (isWpsOffice(prog)) {
            return "Preedit";
        }
        // GNOME Wayland proxy guard: do not pollute per-app cache with generic proxy name
        if (prog == "gnome-shell") {
            return config_output_mode_;
        }
        if (config_remember_app_mode_ && !prog.empty()) {
            // 1. Direct exact match in app_output_modes_
            auto it = app_output_modes_.find(prog);
            if (it != app_output_modes_.end()) {
                return it->second;
            }

            // 2. Basename & case-insensitive match against user configured app modes
            std::string lower_prog = prog;
            auto last_slash = lower_prog.find_last_of('/');
            if (last_slash != std::string::npos) {
                lower_prog = lower_prog.substr(last_slash + 1);
            }
            for (char& c : lower_prog) c = std::tolower(c);

            for (const auto& [app, app_mode] : app_output_modes_) {
                std::string lower_app = app;
                for (char& c : lower_app) c = std::tolower(c);
                if (lower_prog == lower_app ||
                    lower_prog.find(lower_app) != std::string::npos ||
                    lower_app.find(lower_prog) != std::string::npos) {
                    return app_mode;
                }
            }

            // 3. Built-in defaults only apply if the application is NOT configured by user
            if (isGamingProgram(lower_prog)) {
                return "Bypass";
            }
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
        if (!prog.empty() && prog != "gnome-shell") {
            std::string base_prog = prog;
            auto last_slash = base_prog.find_last_of('/');
            if (last_slash != std::string::npos) {
                base_prog = base_prog.substr(last_slash + 1);
            }
            app_output_modes_[prog] = mode;
            app_output_modes_[base_prog] = mode;
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
            } else if (line.rfind("PreeditTooltip=", 0) == 0) {
                config_preedit_tooltip_ = (line.find("False") == std::string::npos);
            } else if (line.rfind("PersistentContextGuard=", 0) == 0) {
                config_persistent_guard_ = (line.find("False") == std::string::npos);
            } else if (line.rfind("SelectAndReplaceWeb=", 0) == 0) {
                config_select_and_replace_web_ = (line.find("False") == std::string::npos && line.find("false") == std::string::npos);
            } else if (line.rfind("EnableWordRescue=", 0) == 0) {
                config_word_rescue_ = (line.find("False") == std::string::npos && line.find("false") == std::string::npos);
            } else if (line.rfind("DirectDelayMs=", 0) == 0) {
                try {
                    config_direct_delay_ms_ = std::stoi(line.substr(14));
                } catch (...) {
                    config_direct_delay_ms_ = 0;
                }
            } else if (line.rfind("IdleTimeoutMs=", 0) == 0) {
                try {
                    config_idle_timeout_ms_ = std::stoull(line.substr(14));
                } catch (...) {
                    config_idle_timeout_ms_ = 0;
                }
            } else if (line.rfind("AutoRestoreEnglish=", 0) == 0) {
                config_auto_restore_ = (line.find("False") == std::string::npos);
            } else if (line.rfind("EnableMacro=", 0) == 0) {
                config_macro_ = (line.find("False") == std::string::npos);
            } else if (line.rfind("CapitalizeMacro=", 0) == 0) {
                // Read into config if needed
            } else if (line.rfind("EscRestoreRaw=", 0) == 0) {
                config_esc_restore_raw_ = (line.find("True") != std::string::npos || line.find("true") != std::string::npos);
            } else if (line.rfind("FlipShortcut=", 0) == 0) {
                std::string val = line.substr(13);
                size_t start = val.find_first_not_of(" \t\"'");
                size_t end = val.find_last_not_of(" \t\"'");
                if (start != std::string::npos && end != std::string::npos) {
                    config_flip_shortcut_ = val.substr(start, end - start + 1);
                }
            } else if (line.rfind("SpellCheck=", 0) == 0) {
                config_spell_check_ = (line.find("False") == std::string::npos);
            } else if (line.rfind("EnableDictionary=", 0) == 0) {
                config_dictionary_ = (line.find("False") == std::string::npos);
            } else if (line.rfind("ModernStyle=", 0) == 0) {
                config_modern_style_ = (line.find("False") == std::string::npos && line.find("false") == std::string::npos);
            } else if (line.rfind("BracketTransform=", 0) == 0) {
                config_bracket_transform_ = (line.find("True") != std::string::npos || line.find("true") != std::string::npos);
            } else if (line.rfind("FastConsonantDoubling=", 0) == 0 || line.rfind("QuickTelex=", 0) == 0) {
                config_quick_telex_ = (line.find("True") != std::string::npos || line.find("true") != std::string::npos);
            } else if (line.rfind("QuickStartConsonant=", 0) == 0) {
                config_quick_start_consonant_ = (line.find("True") != std::string::npos || line.find("true") != std::string::npos);
            } else if (line.rfind("QuickEndConsonant=", 0) == 0) {
                config_quick_end_consonant_ = (line.find("True") != std::string::npos || line.find("true") != std::string::npos);
            } else if (line.rfind("TypeW2U=", 0) == 0) {
                std::string val = line.substr(8);
                if (val.find("Always") != std::string::npos) {
                    config_w2u_ = 1;
                } else if (val.find("Disabled") != std::string::npos) {
                    config_w2u_ = 2;
                } else {
                    config_w2u_ = 0;
                }
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
        fcitx5_vnchkey_set_tone_style(ctx, config_modern_style_ ? 0 : 1);
        fcitx5_vnchkey_set_bracket_transform(ctx, config_bracket_transform_);
        fcitx5_vnchkey_set_quick_telex(ctx, config_quick_telex_);
        fcitx5_vnchkey_set_quick_start_consonant(ctx, config_quick_start_consonant_);
        fcitx5_vnchkey_set_quick_end_consonant(ctx, config_quick_end_consonant_);
        fcitx5_vnchkey_set_w2u(ctx, config_w2u_);
        uint32_t charset_code = (config_charset_.find("Composite") != std::string::npos || config_charset_.find("Tổ hợp") != std::string::npos) ? 1 : 0;
        fcitx5_vnchkey_set_charset(ctx, charset_code);
        fcitx5_vnchkey_set_idle_timeout(ctx, config_idle_timeout_ms_);
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
                    l.rfind("PreeditTooltip=", 0) != 0 &&
                    l.rfind("PersistentContextGuard=", 0) != 0 &&
                    l.rfind("DirectDelayMs=", 0) != 0 &&
                    l.rfind("IdleTimeoutMs=", 0) != 0 &&
                    l.rfind("AutoRestoreEnglish=", 0) != 0 &&
                    l.rfind("EnableMacro=", 0) != 0 &&
                    l.rfind("EscRestoreRaw=", 0) != 0 &&
                    l.rfind("FlipShortcut=", 0) != 0 &&
                    l.rfind("FastConsonantDoubling=", 0) != 0 &&
                    l.rfind("QuickTelex=", 0) != 0 &&
                    l.rfind("QuickStartConsonant=", 0) != 0 &&
                    l.rfind("QuickEndConsonant=", 0) != 0 &&
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
        out << "OutputCharset=" << config_charset_ << "\n";
        out << "OutputMode=" << config_output_mode_ << "\n";
        out << "RememberAppMode=" << (config_remember_app_mode_ ? "True" : "False") << "\n";
        out << "PreeditTooltip=" << (config_preedit_tooltip_ ? "True" : "False") << "\n";
        out << "PersistentContextGuard=" << (config_persistent_guard_ ? "True" : "False") << "\n";
        out << "SelectAndReplaceWeb=" << (config_select_and_replace_web_ ? "True" : "False") << "\n";
        out << "EnableWordRescue=" << (config_word_rescue_ ? "True" : "False") << "\n";
        out << "DirectDelayMs=" << config_direct_delay_ms_ << "\n";
        out << "IdleTimeoutMs=" << config_idle_timeout_ms_ << "\n";
        out << "AutoRestoreEnglish=" << (config_auto_restore_ ? "True" : "False") << "\n";
        out << "EnableMacro=" << (config_macro_ ? "True" : "False") << "\n";
        out << "EscRestoreRaw=" << (config_esc_restore_raw_ ? "True" : "False") << "\n";
        out << "FlipShortcut=" << config_flip_shortcut_ << "\n";
        out << "FastConsonantDoubling=" << (config_quick_telex_ ? "True" : "False") << "\n";
        out << "QuickTelex=" << (config_quick_telex_ ? "True" : "False") << "\n";
        out << "QuickStartConsonant=" << (config_quick_start_consonant_ ? "True" : "False") << "\n";
        out << "QuickEndConsonant=" << (config_quick_end_consonant_ ? "True" : "False") << "\n";
        out << "SpellCheck=" << (config_spell_check_ ? "True" : "False") << "\n";
        out << "EnableDictionary=" << (config_dictionary_ ? "True" : "False") << "\n";
    }
}

void VnchkeyEngine::reloadConfig() {
    loadConfig();
    loadAppModes();
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

    // Persistent Context Guard: check if returning from Java IDE / Wine temporary popup
    if (config_persistent_guard_ && !last_focus_out_prog_.empty()) {
        auto now = std::chrono::steady_clock::now();
        auto diff_ms = std::chrono::duration_cast<std::chrono::milliseconds>(now - last_focus_out_time_).count();
        if (ic && ic->program() == last_focus_out_prog_ && diff_ms <= 350) {
            // Returned to same application within grace period -> keep active buffer!
            last_focus_out_prog_.clear();
        } else if (diff_ms > 350 || (ic && ic->program() != last_focus_out_prog_)) {
            // Grace period expired or switched to a different window
            void* prev_ctx = getContext(last_focus_out_entry_);
            if (prev_ctx) fcitx5_vnchkey_reset(prev_ctx);
            last_focus_out_prog_.clear();
        }
    }

    // Zero-Config Password / PIN / Sensitive Field Protection:
    // Immediately flush any composing buffer and bypass IME completely in password/PIN fields.
    if (isSensitiveContext(ic)) {
        fcitx5_vnchkey_reset(ctx);
        return;
    }

    std::string mode = getOutputModeForContext(ic);
    if (mode.rfind("Bypass", 0) == 0 || mode == "OFF") {
        fcitx5_vnchkey_reset(ctx);
        return;
    }
    const bool isPreeditMode = (mode == "Preedit" || mode == "preedit");

    auto rawKey = keyEvent.rawKey();
    auto states = rawKey.states();
    auto sym = rawKey.sym();

    // Flip Word shortcut: instantly toggle between Vietnamese and raw strokes
    // Options: "Ctrl+Shift+Z", "Alt+X", "Ctrl+Alt+Z", or "Disabled"
    bool isFlipWord = false;
    if (config_flip_shortcut_ == "Ctrl+Shift+Z") {
        isFlipWord = states.test(fcitx::KeyState::Ctrl) &&
                     states.test(fcitx::KeyState::Shift) &&
                     !states.test(fcitx::KeyState::Alt) &&
                     (sym == FcitxKey_Z || sym == FcitxKey_z);
    } else if (config_flip_shortcut_ == "Alt+X") {
        isFlipWord = states.test(fcitx::KeyState::Alt) &&
                     !states.test(fcitx::KeyState::Ctrl) &&
                     !states.test(fcitx::KeyState::Shift) &&
                     (sym == FcitxKey_X || sym == FcitxKey_x);
    } else if (config_flip_shortcut_ == "Ctrl+Alt+Z") {
        isFlipWord = states.test(fcitx::KeyState::Ctrl) &&
                     states.test(fcitx::KeyState::Alt) &&
                     !states.test(fcitx::KeyState::Shift) &&
                     (sym == FcitxKey_Z || sym == FcitxKey_z);
    }
    if (isFlipWord && fcitx5_vnchkey_has_active_word(ctx)) {
        size_t bs = 0;
        char* commitText = nullptr;
        if (fcitx5_vnchkey_flip_word(ctx, &bs, &commitText) != 0) {
            std::string prog = ic ? ic->program() : "";
            if (isPreeditMode && ic) {
                char* cur = fcitx5_vnchkey_get_current_word(ctx);
                if (cur && cur[0] != '\0') {
                    fcitx::Text text(cur, fcitx::TextFormatFlag::Underline);
                    ic->inputPanel().setClientPreedit(text);
                    if (config_preedit_tooltip_ || isWpsOffice(prog)) {
                        ic->inputPanel().setPreedit(text);
                    }
                    ic->updatePreedit();
                    ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
                    fcitx5_vnchkey_free_string(cur);
                }
            } else if (ic) {
                for (size_t i = 0; i < bs; ++i) {
                    ic->forwardKey(fcitx::Key(FcitxKey_BackSpace));
                }
                if (commitText && commitText[0] != '\0') {
                    ic->commitString(commitText);
                }
            }
            if (commitText) {
                fcitx5_vnchkey_free_string(commitText);
            }
            keyEvent.filterAndAccept();
            return;
        }
    }

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

    // Word Rescue Undo (Escape or Backspace immediately after On-Activation rescue)
    if (!last_rescue_raw_.empty()) {
        if (sym == FcitxKey_Escape || sym == FcitxKey_BackSpace) {
            if (ic) {
                size_t conv_chars = utf8CharCount(last_rescue_converted_);
                if (ic->capabilityFlags().test(fcitx::CapabilityFlag::SurroundingText) &&
                    ic->surroundingText().isValid()) {
                    ic->deleteSurroundingText(-static_cast<int>(conv_chars), conv_chars);
                } else {
                    bool is_web = isWebApp(ic->program());
                    is_forwarding_ = true;
                    if (config_select_and_replace_web_ && is_web) {
                        for (size_t i = 0; i < conv_chars; ++i) {
                            ic->forwardKey(fcitx::Key(FcitxKey_Left, fcitx::KeyState::Shift));
                        }
                    } else {
                        for (size_t i = 0; i < conv_chars; ++i) {
                            ic->forwardKey(fcitx::Key(FcitxKey_BackSpace));
                        }
                    }
                    is_forwarding_ = false;
                }
                ic->commitString(last_rescue_raw_);
            }
            last_rescue_raw_.clear();
            last_rescue_converted_.clear();
            keyEvent.filterAndAccept();
            return;
        }
        last_rescue_raw_.clear();
        last_rescue_converted_.clear();
    }

    // Escape key handling:
    // - If config_esc_restore_raw_ is true (Gõ Nhanh / UniKey style):
    //   Bare Escape reverts active composing word to raw typed strokes.
    // - If config_esc_restore_raw_ is false (Default / Neovim-friendly mode):
    //   Shift + Escape reverts to raw typed strokes, while bare Escape passes directly to app (so Neovim exits INSERT mode on 1st press!)
    if (sym == FcitxKey_Escape) {
        const bool isShiftEscape = states.test(fcitx::KeyState::Shift);
        const bool shouldUndoToRaw = (config_esc_restore_raw_ && !isShiftEscape) || isShiftEscape;

        if (shouldUndoToRaw && fcitx5_vnchkey_has_active_word(ctx)) {
            size_t esc_bs = 0;
            char* esc_commit = nullptr;
            if (fcitx5_vnchkey_undo_escape(ctx, &esc_bs, &esc_commit)) {
                if (isPreeditMode && ic) {
                    // In Preedit Mode, replace preedit with raw word or update preedit
                    char* cur = fcitx5_vnchkey_get_current_word(ctx);
                    if (cur && cur[0] != '\0') {
                        std::string prog = ic->program();
                        fcitx::Text text(cur, fcitx::TextFormatFlag::Underline);
                        ic->inputPanel().setClientPreedit(text);
                        if (config_preedit_tooltip_ || isWpsOffice(prog)) {
                            ic->inputPanel().setPreedit(text);
                        }
                        ic->updatePreedit();
                        ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
                        fcitx5_vnchkey_free_string(cur);
                    }
                } else if (ic) {
                    if (esc_bs > 0) {
                        if (ic->capabilityFlags().test(fcitx::CapabilityFlag::SurroundingText) &&
                            ic->surroundingText().isValid()) {
                            ic->deleteSurroundingText(-static_cast<int>(esc_bs), esc_bs);
                        } else {
                            bool is_web = isWebApp(ic->program());
                            is_forwarding_ = true;
                            if (config_select_and_replace_web_ && is_web && esc_commit && esc_commit[0] != '\0') {
                                for (size_t i = 0; i < esc_bs; ++i) {
                                    ic->forwardKey(fcitx::Key(FcitxKey_Left, fcitx::KeyState::Shift));
                                }
                            } else {
                                for (size_t i = 0; i < esc_bs; ++i) {
                                    ic->forwardKey(fcitx::Key(FcitxKey_BackSpace));
                                }
                            }
                            is_forwarding_ = false;
                        }
                    }
                    if (esc_commit && esc_commit[0] != '\0') {
                        ic->commitString(esc_commit);
                    }
                }
                if (esc_commit) {
                    fcitx5_vnchkey_free_string(esc_commit);
                }
                keyEvent.filterAndAccept();
                return;
            }
        }

        // Bare Escape: commit active Preedit word if any, reset engine buffer, and let Escape pass through to Neovim/App!
        if (isPreeditMode && ic && fcitx5_vnchkey_has_active_word(ctx)) {
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
        // Macro Undo (OpenViet style): if previous action was a macro expansion and user presses Backspace immediately,
        // revert the expansion back to the original abbreviation strokes.
        size_t undo_bs = 0;
        char* undo_commit = nullptr;
        if (fcitx5_vnchkey_undo_macro(ctx, &undo_bs, &undo_commit)) {
            if (ic) {
                if (undo_bs > 0) {
                    if (ic->capabilityFlags().test(fcitx::CapabilityFlag::SurroundingText) &&
                        ic->surroundingText().isValid()) {
                        ic->deleteSurroundingText(-static_cast<int>(undo_bs), undo_bs);
                    } else {
                        bool is_web = isWebApp(ic->program());
                        is_forwarding_ = true;
                        if (config_select_and_replace_web_ && is_web && undo_commit && undo_commit[0] != '\0') {
                            for (size_t i = 0; i < undo_bs; ++i) {
                                ic->forwardKey(fcitx::Key(FcitxKey_Left, fcitx::KeyState::Shift));
                            }
                        } else {
                            for (size_t i = 0; i < undo_bs; ++i) {
                                ic->forwardKey(fcitx::Key(FcitxKey_BackSpace));
                            }
                        }
                        is_forwarding_ = false;
                    }
                }
                if (isPreeditMode) {
                    ic->inputPanel().reset();
                    ic->updatePreedit();
                    ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
                }
                if (undo_commit && undo_commit[0] != '\0') {
                    ic->commitString(undo_commit);
                }
            }
            if (undo_commit) {
                fcitx5_vnchkey_free_string(undo_commit);
            }
            keyEvent.filterAndAccept();
            return;
        }

        if (isPreeditMode && ic && fcitx5_vnchkey_has_active_word(ctx)) {
            fcitx5_vnchkey_handle_backspace(ctx);
            if (fcitx5_vnchkey_has_active_word(ctx)) {
                char* cur = fcitx5_vnchkey_get_current_word(ctx);
                if (cur && cur[0] != '\0') {
                    fcitx::Text text(cur, fcitx::TextFormatFlag::Underline);
                    ic->inputPanel().setClientPreedit(text);
                    if (config_preedit_tooltip_ || isWpsOffice(ic->program())) {
                        ic->inputPanel().setPreedit(text);
                    }
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

    // Feature: Cứu Dấu (Selection Accent Rescue - Ngó Sen style)
    // If text is currently highlighted / selected and the key pressed is a tone/diacritic modifier,
    // transform the selected word with the diacritic instead of typing over it!
    if (config_word_rescue_ && ic &&
        ic->capabilityFlags().test(fcitx::CapabilityFlag::SurroundingText) &&
        ic->surroundingText().isValid()) {
        const auto& surrounding = ic->surroundingText();
        if (surrounding.cursor() != surrounding.anchor()) {
            std::string sel = surrounding.selectedText();
            if (!sel.empty() && sel.length() <= 80) {
                char* transformed = nullptr;
                if (fcitx5_vnchkey_apply_accent_to_selection(ctx, sel.c_str(), ch, &transformed)) {
                    if (transformed && std::string(transformed) != sel) {
                        int start = std::min(surrounding.cursor(), surrounding.anchor());
                        int end = std::max(surrounding.cursor(), surrounding.anchor());
                        int sel_len = end - start;
                        int offset = start - surrounding.cursor();
                        ic->deleteSurroundingText(offset, sel_len);
                        ic->commitString(transformed);
                        fcitx5_vnchkey_free_string(transformed);
                        keyEvent.filterAndAccept();
                        return;
                    }
                    if (transformed) {
                        fcitx5_vnchkey_free_string(transformed);
                    }
                }
            }
        }
    }

    auto now = std::chrono::steady_clock::now();
    double timestamp_ms = std::chrono::duration<double, std::milli>(now.time_since_epoch()).count();

    size_t out_backspaces = 0;
    char* out_commit = nullptr;

    int consumed = fcitx5_vnchkey_process_key(ctx, ch, timestamp_ms, &out_backspaces, &out_commit);

    // Google Sheets / Autocomplete selection cleanup: if there is an active selection after caret, clear it
    if (ic && ic->surroundingText().isValid()) {
        const auto& surrounding = ic->surroundingText();
        if (surrounding.cursor() < surrounding.anchor()) {
            unsigned int deleteAfter = surrounding.anchor() - surrounding.cursor();
            if (deleteAfter > 0) {
                ic->deleteSurroundingText(0, deleteAfter);
            }
        }
    }

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
                if (config_preedit_tooltip_ || isWpsOffice(ic->program())) {
                    ic->inputPanel().setPreedit(text);
                }
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
                        bool is_web = isWebApp(ic->program());

                        is_forwarding_ = true;
                        if (config_select_and_replace_web_ && is_web) {
                            // Dokey Innovation: Select-and-Replace (Shift + Left Arrow)
                            // Overcomes ProseMirror, Notion, Slack, and React virtual DOM Backspace suppression!
                            for (size_t i = 0; i < out_backspaces; ++i) {
                                ic->forwardKey(fcitx::Key(FcitxKey_Left, fcitx::KeyState::Shift));
                            }
                        } else {
                            for (size_t i = 0; i < out_backspaces; ++i) {
                                ic->forwardKey(fcitx::Key(FcitxKey_BackSpace));
                            }
                        }
                        is_forwarding_ = false;

                        // Adaptive Micro-Pause: allow browser JavaScript event loop / DOM to settle
                        int effective_delay = config_direct_delay_ms_;
                        if (effective_delay <= 0 && is_web) {
                            effective_delay = 4; // 4ms is ideal for web DOM cursor settlement
                        }
                        if (effective_delay > 0) {
                            std::this_thread::sleep_for(std::chrono::milliseconds(effective_delay));
                        }
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
    if (!ic) return;

    std::string prog = ic->program();
    std::string lower_prog = prog;
    for (char& c : lower_prog) c = std::tolower(c);

    const bool is_java_or_wine = (
        lower_prog.find("idea") != std::string::npos ||
        lower_prog.find("pycharm") != std::string::npos ||
        lower_prog.find("clion") != std::string::npos ||
        lower_prog.find("studio") != std::string::npos ||
        lower_prog.find("java") != std::string::npos ||
        lower_prog.find("wine") != std::string::npos ||
        lower_prog.find("zalo") != std::string::npos ||
        lower_prog.find("notepad") != std::string::npos
    );

    if (config_persistent_guard_ && is_java_or_wine) {
        last_focus_out_time_ = std::chrono::steady_clock::now();
        last_focus_out_prog_ = prog;
        last_focus_out_entry_ = entry.uniqueName();
        return;
    }

    ic->inputPanel().reset();
    ic->updatePreedit();
    ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);

    void* ctx = getContext(entry.uniqueName());
    if (ctx) {
        fcitx5_vnchkey_reset(ctx);
    }
}

bool VnchkeyEngine::isWebApp(const std::string& prog) const {
    std::string lower_prog = prog;
    for (char& c : lower_prog) c = std::tolower(c);
    return (
        lower_prog.find("chrome") != std::string::npos ||
        lower_prog.find("firefox") != std::string::npos ||
        lower_prog.find("chromium") != std::string::npos ||
        lower_prog.find("brave") != std::string::npos ||
        lower_prog.find("edge") != std::string::npos ||
        lower_prog.find("opera") != std::string::npos ||
        lower_prog.find("vivaldi") != std::string::npos ||
        lower_prog.find("electron") != std::string::npos ||
        lower_prog.find("slack") != std::string::npos ||
        lower_prog.find("discord") != std::string::npos ||
        lower_prog.find("notion") != std::string::npos ||
        lower_prog.find("element") != std::string::npos ||
        lower_prog.find("teams") != std::string::npos
    );
}

size_t VnchkeyEngine::utf8CharCount(const std::string& str) const {
    size_t count = 0;
    for (unsigned char c : str) {
        if ((c & 0xC0) != 0x80) count++;
    }
    return count;
}

fcitx::AddonInstance *VnchkeyEngineFactory::create(fcitx::AddonManager *manager) {
    return new VnchkeyEngine(manager);
}

FCITX_ADDON_FACTORY(VnchkeyEngineFactory);
