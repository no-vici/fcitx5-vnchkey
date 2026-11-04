#pragma once

#include <fcitx/addonfactory.h>
#include <fcitx/addoninstance.h>
#include <fcitx/inputmethodengine.h>
#include <fcitx/inputcontext.h>
#include <fcitx/event.h>
#include <fcitx/action.h>
#include <fcitx/menu.h>
#include <fcitx/statusarea.h>
#include <fcitx-utils/signals.h>
#include <cstdint>
#include <cstddef>
#include <string>
#include <vector>
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
    void fcitx5_vnchkey_free_string(char* s);
}

class VnchkeyEngine : public fcitx::InputMethodEngineV2 {
public:
    explicit VnchkeyEngine(fcitx::AddonManager* manager);
    ~VnchkeyEngine() override;

    void keyEvent(const fcitx::InputMethodEntry &entry, fcitx::KeyEvent &keyEvent) override;
    void reset(const fcitx::InputMethodEntry &entry, fcitx::InputContextEvent &event) override;
    void activate(const fcitx::InputMethodEntry &entry, fcitx::InputContextEvent &event) override;
    void reloadConfig() override;

private:
    void* getContext(const std::string& entryName);
    void loadConfig();
    void saveConfig();
    void setupStatusMenu();
    void updateActionState();

    fcitx::AddonManager* manager_;
    void* ctx_auto_ = nullptr;
    void* ctx_telex_ = nullptr;
    void* ctx_vni_ = nullptr;

    std::string config_method_ = "VNI";
    std::string config_charset_ = "Unicode";
    bool config_auto_restore_ = true;
    bool config_macro_ = true;
    bool config_spell_check_ = true;

    bool is_forwarding_ = false;

    // Fcitx5 Tray & Status Menu
    std::unique_ptr<fcitx::SimpleAction> statusAction_;
    std::unique_ptr<fcitx::Menu> statusMenu_;

    std::unique_ptr<fcitx::SimpleAction> actionMethodVni_;
    std::unique_ptr<fcitx::SimpleAction> actionMethodTelex_;
    std::unique_ptr<fcitx::SimpleAction> actionMethodAuto_;

    std::unique_ptr<fcitx::SimpleAction> actionCharsetUnicode_;
    std::unique_ptr<fcitx::SimpleAction> actionCharsetToHop_;
    std::unique_ptr<fcitx::SimpleAction> actionCharsetTcvn3_;

    std::unique_ptr<fcitx::SimpleAction> actionAutoRestore_;
    std::unique_ptr<fcitx::SimpleAction> actionMacro_;
    std::unique_ptr<fcitx::SimpleAction> actionSpellCheck_;

    std::unique_ptr<fcitx::SimpleAction> actionSeparator1_;
    std::unique_ptr<fcitx::SimpleAction> actionSeparator2_;
    std::unique_ptr<fcitx::SimpleAction> actionSettings_;

    std::vector<fcitx::Connection> connections_;
};

class VnchkeyEngineFactory : public fcitx::AddonFactory {
public:
    fcitx::AddonInstance *create(fcitx::AddonManager *manager) override {
        return new VnchkeyEngine(manager);
    }
};
