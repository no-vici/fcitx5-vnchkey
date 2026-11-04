#pragma once

#include <fcitx/addonfactory.h>
#include <fcitx/addoninstance.h>
#include <fcitx/inputmethodengine.h>
#include <fcitx/inputcontext.h>
#include <fcitx/event.h>
#include <cstdint>
#include <cstddef>

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
    void fcitx5_vnchkey_free_string(char* s);
}

class VnchkeyEngine : public fcitx::InputMethodEngineV2 {
public:
    explicit VnchkeyEngine(fcitx::AddonManager* manager);
    ~VnchkeyEngine() override;

    void keyEvent(const fcitx::InputMethodEntry &entry, fcitx::KeyEvent &keyEvent) override;
    void reset(const fcitx::InputMethodEntry &entry, fcitx::InputContextEvent &event) override;

private:
    fcitx::AddonManager* manager_;
    void* ctx_ = nullptr;
    bool is_forwarding_ = false;
};

class VnchkeyEngineFactory : public fcitx::AddonFactory {
public:
    fcitx::AddonInstance *create(fcitx::AddonManager *manager) override {
        return new VnchkeyEngine(manager);
    }
};

