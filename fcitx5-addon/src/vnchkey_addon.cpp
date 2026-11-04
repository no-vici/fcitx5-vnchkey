#include "vnchkey_addon.h"
#include <fcitx/keysym.h>
#include <fcitx/inputcontext.h>
#include <chrono>
#include <string>

VnchkeyEngine::VnchkeyEngine(fcitx::AddonManager* manager)
    : manager_(manager) {
    telex_ctx_ = fcitx5_vnchkey_init_with_method(0); // Telex
    vni_ctx_ = fcitx5_vnchkey_init_with_method(1);   // VNI
}

VnchkeyEngine::~VnchkeyEngine() {
    if (telex_ctx_) {
        fcitx5_vnchkey_free(telex_ctx_);
        telex_ctx_ = nullptr;
    }
    if (vni_ctx_) {
        fcitx5_vnchkey_free(vni_ctx_);
        vni_ctx_ = nullptr;
    }
}

void* VnchkeyEngine::getContext(const fcitx::InputMethodEntry &entry) {
    const auto &uniqueName = entry.uniqueName();
    if (uniqueName.find("vni") != std::string::npos || uniqueName.find("VNI") != std::string::npos) {
        return vni_ctx_;
    }
    return telex_ctx_;
}

void VnchkeyEngine::keyEvent(const fcitx::InputMethodEntry &entry, fcitx::KeyEvent &keyEvent) {
    // Ignore key release events
    if (keyEvent.isRelease()) {
        return;
    }

    auto rawKey = keyEvent.rawKey();

    // Pass through combination shortcuts (Ctrl, Alt, Super)
    if (rawKey.hasModifier()) {
        reset(entry, keyEvent);
        return;
    }

    auto* ctx = getContext(entry);
    if (!ctx) {
        return;
    }

    uint32_t sym = rawKey.sym();

    // Check if it is a printable character or backspace/escape
    uint32_t ch = 0;
    if (sym == FcitxKey_BackSpace) {
        ch = 0x08;
    } else if (sym == FcitxKey_Escape) {
        ch = 0x1B;
    } else if (sym == FcitxKey_Return) {
        reset(entry, keyEvent);
        return;
    } else {
        ch = fcitx::Key::keySymToUnicode(sym);
        if (ch == 0) {
            return;
        }
    }

    auto now = std::chrono::steady_clock::now();
    double timestamp_ms = std::chrono::duration<double, std::milli>(now.time_since_epoch()).count();

    size_t out_backspaces = 0;
    char* out_commit = nullptr;

    int consumed = fcitx5_vnchkey_process_key(ctx, ch, timestamp_ms, &out_backspaces, &out_commit);

    if (consumed) {
        auto* ic = keyEvent.inputContext();
        if (ic) {
            // Forward backspaces
            for (size_t i = 0; i < out_backspaces; ++i) {
                ic->forwardKey(fcitx::Key(FcitxKey_BackSpace));
            }

            // Commit transformed string
            if (out_commit) {
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
    auto* ctx = getContext(entry);
    if (ctx) {
        fcitx5_vnchkey_reset(ctx);
    }
}

FCITX_ADDON_FACTORY(VnchkeyEngineFactory)

