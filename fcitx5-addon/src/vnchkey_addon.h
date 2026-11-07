#pragma once

#include <fcitx/addonfactory.h>
#include <fcitx/addoninstance.h>
#include <fcitx/addonmanager.h>
#include <fcitx/inputmethodengine.h>
#include <fcitx/inputcontext.h>
#include <fcitx/inputpanel.h>
#include <fcitx/text.h>
#include <fcitx/event.h>
#include <fcitx/action.h>
#include <fcitx/menu.h>
#include <fcitx/statusarea.h>
#include <fcitx/instance.h>
#include <fcitx/userinterfacemanager.h>
#include <fcitx-utils/signals.h>
#include <fcitx-utils/textformatflags.h>
#include <cstdint>
#include <cstddef>
#include <chrono>
#include <string>
#include <vector>
#include <unordered_map>
#include <memory>

// C-ABI declarations exported from Rust (crates/fcitx5-vnchkey-ffi)
extern "C" {
    void* fcitx5_vnchkey_init();
    void* fcitx5_vnchkey_init_with_method(uint32_t method);
    void fcitx5_vnchkey_free(void* ctx);
    int fcitx5_vnchkey_process_key(
        void* ctx,
        uint32_t key_char,
        double timestamp_ms,
        size_t* out_backspaces,
        char** out_commit
    );
    void fcitx5_vnchkey_reset(void* ctx);
    void fcitx5_vnchkey_handle_backspace(void* ctx);
    void fcitx5_vnchkey_load_macros(void* ctx, const char* path);
    void fcitx5_vnchkey_load_dictionary(void* ctx, const char* path);
    void fcitx5_vnchkey_free_string(char* s);
    void fcitx5_vnchkey_set_method(void* ctx, uint32_t method);
    uint32_t fcitx5_vnchkey_get_method(void* ctx);
    void fcitx5_vnchkey_set_auto_restore(void* ctx, bool enabled);
    bool fcitx5_vnchkey_get_auto_restore(void* ctx);
    void fcitx5_vnchkey_set_spell_check(void* ctx, bool enabled);
    bool fcitx5_vnchkey_get_spell_check(void* ctx);
    void fcitx5_vnchkey_set_dictionary_enabled(void* ctx, bool enabled);
    bool fcitx5_vnchkey_get_dictionary_enabled(void* ctx);
    void fcitx5_vnchkey_set_macro_enabled(void* ctx, bool enabled);
    bool fcitx5_vnchkey_get_macro_enabled(void* ctx);
    void fcitx5_vnchkey_set_bracket_transform(void* ctx, bool enabled);
    void fcitx5_vnchkey_set_w2u(void* ctx, uint32_t mode);
    void fcitx5_vnchkey_set_tone_style(void* ctx, uint32_t style);
    uint32_t fcitx5_vnchkey_get_tone_style(void* ctx);

    char* fcitx5_vnchkey_get_current_word(void* ctx);
    char* fcitx5_vnchkey_get_completed_word(void* ctx);
    bool fcitx5_vnchkey_has_active_word(void* ctx);
    int fcitx5_vnchkey_flip_word(void* ctx, size_t* out_backspaces, char** out_commit);
    int fcitx5_vnchkey_undo_macro(void* ctx, size_t* out_backspaces, char** out_commit);
    int fcitx5_vnchkey_undo_escape(void* ctx, size_t* out_backspaces, char** out_commit);
}

class VnchkeyEngine : public fcitx::InputMethodEngineV2 {
public:
    explicit VnchkeyEngine(fcitx::AddonManager* manager);
    ~VnchkeyEngine() override;

    void keyEvent(const fcitx::InputMethodEntry &entry, fcitx::KeyEvent &keyEvent) override;
    void reset(const fcitx::InputMethodEntry &entry, fcitx::InputContextEvent &event) override;
    void activate(const fcitx::InputMethodEntry &entry, fcitx::InputContextEvent &event) override;
    void deactivate(const fcitx::InputMethodEntry &entry, fcitx::InputContextEvent &event) override;
    void reloadConfig() override;

private:
    void* getContext(const std::string& entryName);
    void loadConfig();
    void saveConfig();
    void loadAppModes();
    void saveAppModes();
    std::string getOutputModeForContext(fcitx::InputContext* ic);
    void setOutputModeForContext(fcitx::InputContext* ic, const std::string& mode);
    void setupStatusMenu();
    void updateActionState(fcitx::InputContext* ic = nullptr);

    fcitx::AddonManager* manager_;
    void* ctx_auto_ = nullptr;
    void* ctx_telex_ = nullptr;
    void* ctx_vni_ = nullptr;

    std::string config_method_ = "Auto";
    std::string config_charset_ = "Unicode";
    std::string config_output_mode_ = "Direct";
    bool config_remember_app_mode_ = true;
    bool config_preedit_tooltip_ = true;
    bool config_persistent_guard_ = true;
    int config_direct_delay_ms_ = 0; // 0 = auto (0ms for editors, 4ms for web browsers)
    std::unordered_map<std::string, std::string> app_output_modes_;

    // Grace Period Focus Tracker for Java IDEs & Wine
    std::chrono::steady_clock::time_point last_focus_out_time_;
    std::string last_focus_out_prog_;
    std::string last_focus_out_entry_;

    bool config_auto_restore_ = true;
    bool config_macro_ = true;
    bool config_spell_check_ = true;
    bool config_dictionary_ = true;
    bool config_modern_style_ = true;
    bool config_bracket_transform_ = false;
    uint32_t config_w2u_ = 0;

    bool is_forwarding_ = false;

    // Fcitx5 Tray & Status Menu
    std::unique_ptr<fcitx::SimpleAction> statusAction_;
    std::unique_ptr<fcitx::Menu> statusMenu_;

    std::unique_ptr<fcitx::SimpleAction> actionMethodAuto_;
    std::unique_ptr<fcitx::SimpleAction> actionMethodTelex_;
    std::unique_ptr<fcitx::SimpleAction> actionMethodVni_;

    std::unique_ptr<fcitx::SimpleAction> actionOutputModeDirect_;
    std::unique_ptr<fcitx::SimpleAction> actionOutputModePreedit_;
    std::unique_ptr<fcitx::SimpleAction> actionPreeditTooltip_;

    std::unique_ptr<fcitx::SimpleAction> actionCharsetUnicode_;

    std::unique_ptr<fcitx::SimpleAction> actionAutoRestore_;
    std::unique_ptr<fcitx::SimpleAction> actionMacro_;
    std::unique_ptr<fcitx::SimpleAction> actionSpellCheck_;
    std::unique_ptr<fcitx::SimpleAction> actionDictionary_;

    std::unique_ptr<fcitx::SimpleAction> actionSeparator1_;
    std::unique_ptr<fcitx::SimpleAction> actionSeparator2_;
    std::unique_ptr<fcitx::SimpleAction> actionSeparator3_;
    std::unique_ptr<fcitx::SimpleAction> actionSettings_;

    std::vector<fcitx::Connection> connections_;
};

class VnchkeyEngineFactory : public fcitx::AddonFactory {
public:
    fcitx::AddonInstance *create(fcitx::AddonManager *manager) override;
};
