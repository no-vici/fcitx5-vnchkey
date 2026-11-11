#ifndef VNCHKEY_BRIDGING_HEADER_H
#define VNCHKEY_BRIDGING_HEADER_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct Fcitx5VnchkeyContext Fcitx5VnchkeyContext;

/// Initializes a new vnchKey engine instance.
Fcitx5VnchkeyContext* fcitx5_vnchkey_init(void);

/// Initializes a new vnchKey engine instance with a specific method (0 = Auto, 1 = Telex, 2 = VNI).
Fcitx5VnchkeyContext* fcitx5_vnchkey_init_with_method(uint32_t method);

/// Frees a vnchKey engine instance.
void fcitx5_vnchkey_free(Fcitx5VnchkeyContext* ctx);

/// Frees a string allocated by Rust FFI.
void fcitx5_vnchkey_free_string(char* s);

/// Resets context state (e.g. when changing focused app or clicking elsewhere).
void fcitx5_vnchkey_reset(Fcitx5VnchkeyContext* ctx);

/// Handles physical Backspace pressed by the user.
void fcitx5_vnchkey_handle_backspace(Fcitx5VnchkeyContext* ctx);

/// Checks whether the engine has an active composing word.
bool fcitx5_vnchkey_has_active_word(Fcitx5VnchkeyContext* ctx);

/// Sets output character set (0: Unicode NFC, 1: Unicode Composite NFD).
void fcitx5_vnchkey_set_charset(Fcitx5VnchkeyContext* ctx, uint32_t charset);

/// Processes a single keystroke.
/// Returns 1 if consumed/handled, 0 if pass-through.
/// Sets out_backspaces (number of backspaces to emit) and out_commit (new text to insert).
int fcitx5_vnchkey_process_key(
    Fcitx5VnchkeyContext* ctx,
    uint32_t key_char,
    double timestamp_ms,
    size_t* out_backspaces,
    char** out_commit
);

/// Flips the current composing word between processed Vietnamese and raw input strokes (Ctrl/Shift-Shift).
int fcitx5_vnchkey_flip_word(
    Fcitx5VnchkeyContext* ctx,
    size_t* out_backspaces,
    char** out_commit
);

/// Attempts to undo the last macro expansion on Backspace.
int fcitx5_vnchkey_undo_macro(
    Fcitx5VnchkeyContext* ctx,
    size_t* out_backspaces,
    char** out_commit
);

/// Reverts the active composing word back to raw keystrokes on Escape.
int fcitx5_vnchkey_undo_escape(
    Fcitx5VnchkeyContext* ctx,
    size_t* out_backspaces,
    char** out_commit
);

/// Sets active input method (0 = Auto, 1 = Telex, 2 = VNI, 3 = SimpleTelex).
void fcitx5_vnchkey_set_method(Fcitx5VnchkeyContext* ctx, uint32_t method);
uint32_t fcitx5_vnchkey_get_method(Fcitx5VnchkeyContext* ctx);

/// Configures Restore English (auto-restore).
void fcitx5_vnchkey_set_auto_restore(Fcitx5VnchkeyContext* ctx, bool enabled);
bool fcitx5_vnchkey_get_auto_restore(Fcitx5VnchkeyContext* ctx);

/// Configures Chánh Tả spell checking.
void fcitx5_vnchkey_set_spell_check(Fcitx5VnchkeyContext* ctx, bool enabled);
bool fcitx5_vnchkey_get_spell_check(Fcitx5VnchkeyContext* ctx);

/// Configures User Dictionary.
void fcitx5_vnchkey_set_dictionary_enabled(Fcitx5VnchkeyContext* ctx, bool enabled);
bool fcitx5_vnchkey_get_dictionary_enabled(Fcitx5VnchkeyContext* ctx);

/// Configures macro expansion.
void fcitx5_vnchkey_set_macro_enabled(Fcitx5VnchkeyContext* ctx, bool enabled);
bool fcitx5_vnchkey_get_macro_enabled(Fcitx5VnchkeyContext* ctx);

/// Configures bracket transformation ([ -> ơ, ] -> ư).
void fcitx5_vnchkey_set_bracket_transform(Fcitx5VnchkeyContext* ctx, bool enabled);

/// Configures type w to produce ư rule (0 = NonStart, 1 = Always, 2 = Disabled).
void fcitx5_vnchkey_set_w2u(Fcitx5VnchkeyContext* ctx, uint32_t mode);

/// Configures tone style (0 = Modern: oà, uý; 1 = Classic: hòa, ủy).
void fcitx5_vnchkey_set_tone_style(Fcitx5VnchkeyContext* ctx, uint32_t style);
uint32_t fcitx5_vnchkey_get_tone_style(Fcitx5VnchkeyContext* ctx);

/// Sets keystroke idle timeout in milliseconds (0 = disabled).
void fcitx5_vnchkey_set_idle_timeout(Fcitx5VnchkeyContext* ctx, uint64_t ms);
uint64_t fcitx5_vnchkey_get_idle_timeout(Fcitx5VnchkeyContext* ctx);

/// Configures Quick Telex, Quick Initial, and Quick Final consonants.
void fcitx5_vnchkey_set_quick_telex(Fcitx5VnchkeyContext* ctx, bool enabled);
void fcitx5_vnchkey_set_quick_start_consonant(Fcitx5VnchkeyContext* ctx, bool enabled);
void fcitx5_vnchkey_set_quick_end_consonant(Fcitx5VnchkeyContext* ctx, bool enabled);

/// Loads custom macros and dictionary from configuration files.
void fcitx5_vnchkey_load_macros(Fcitx5VnchkeyContext* ctx, const char* path);
void fcitx5_vnchkey_load_dictionary(Fcitx5VnchkeyContext* ctx, const char* path);

/// Rescues a raw sentence or phrase (Ngó Sen Word Rescue / Cứu Chữ).
int fcitx5_vnchkey_rescue_sentence(
    Fcitx5VnchkeyContext* ctx,
    const char* text,
    char** out_rescued
);

/// Applies a tone or diacritic key to a selected text (Selection Accent Rescue / Cứu Dấu).
int fcitx5_vnchkey_apply_accent_to_selection(
    Fcitx5VnchkeyContext* ctx,
    const char* selection,
    uint32_t ch,
    char** out_transformed
);

#ifdef __cplusplus
}
#endif

#endif // VNCHKEY_BRIDGING_HEADER_H

