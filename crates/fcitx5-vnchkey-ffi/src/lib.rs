//! C ABI Bridge for Fcitx5 Addon Integration.

use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use vnchkey_core::{Engine, EngineConfig};

/// Opaque handle to the vnchKey Rust Engine.
pub struct Fcitx5VnchkeyContext {
    engine: Engine,
}

/// Initializes a new vnchKey engine instance.
#[no_mangle]
pub extern "C" fn fcitx5_vnchkey_init() -> *mut Fcitx5VnchkeyContext {
    fcitx5_vnchkey_init_with_method(0)
}

/// Initializes a new vnchKey engine instance with a specific method (0 = Auto, 1 = Telex, 2 = VNI).
#[no_mangle]
pub extern "C" fn fcitx5_vnchkey_init_with_method(method: u32) -> *mut Fcitx5VnchkeyContext {
    let input_method = match method {
        1 => vnchkey_core::InputMethod::Telex,
        2 => vnchkey_core::InputMethod::Vni,
        _ => vnchkey_core::InputMethod::Auto,
    };
    let config = EngineConfig {
        method: input_method,
        ..EngineConfig::default()
    };
    let ctx = Box::new(Fcitx5VnchkeyContext {
        engine: Engine::new(config),
    });
    Box::into_raw(ctx)
}

/// Frees a string allocated by Rust FFI.
#[no_mangle]
pub unsafe extern "C" fn fcitx5_vnchkey_free_string(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}

/// Frees a vnchKey engine instance.
#[no_mangle]
pub unsafe extern "C" fn fcitx5_vnchkey_free(ctx: *mut Fcitx5VnchkeyContext) {
    if !ctx.is_null() {
        drop(Box::from_raw(ctx));
    }
}

/// Processes a single keystroke from Fcitx5.
/// Returns number of backspaces to emit and sets `out_commit`.
#[no_mangle]
pub unsafe extern "C" fn fcitx5_vnchkey_process_key(
    ctx: *mut Fcitx5VnchkeyContext,
    key_char: u32,
    timestamp_ms: f64,
    out_backspaces: *mut usize,
    out_commit: *mut *mut c_char,
) -> c_int {
    if ctx.is_null() {
        return 0;
    }

    let ctx = &mut *ctx;
    if let Some(ch) = char::from_u32(key_char) {
        let output = ctx.engine.process_key(ch, timestamp_ms);
        if !out_backspaces.is_null() {
            *out_backspaces = output.backspaces;
        }
        if !out_commit.is_null() {
            let c_str = CString::new(output.commit_text).unwrap_or_default();
            *out_commit = c_str.into_raw();
        }
        if output.consumed { 1 } else { 0 }
    } else {
        0
    }
}

/// Resets context state (e.g. when changing focused client).
#[no_mangle]
pub unsafe extern "C" fn fcitx5_vnchkey_reset(ctx: *mut Fcitx5VnchkeyContext) {
    if !ctx.is_null() {
        let ctx = &mut *ctx;
        ctx.engine.reset();
    }
}

/// Sets output character set (0: Unicode NFC, 1: Unicode Composite NFD).
#[no_mangle]
pub unsafe extern "C" fn fcitx5_vnchkey_set_charset(ctx: *mut Fcitx5VnchkeyContext, charset: u32) {
    if !ctx.is_null() {
        let ctx = &mut *ctx;
        ctx.engine.set_charset(match charset {
            1 => vnchkey_core::Charset::UnicodeComposite,
            _ => vnchkey_core::Charset::Unicode,
        });
    }
}

/// Handles physical Backspace pressed by the user.
#[no_mangle]
pub unsafe extern "C" fn fcitx5_vnchkey_handle_backspace(ctx: *mut Fcitx5VnchkeyContext) {
    if !ctx.is_null() {
        let ctx = &mut *ctx;
        ctx.engine.handle_backspace();
    }
}

/// Returns the active composing word in the engine.
/// Must be freed with `fcitx5_vnchkey_free_string`.
#[no_mangle]
pub unsafe extern "C" fn fcitx5_vnchkey_get_current_word(
    ctx: *mut Fcitx5VnchkeyContext,
) -> *mut c_char {
    if !ctx.is_null() {
        let word = (*ctx).engine.current_word();
        let c_str = CString::new(word).unwrap_or_default();
        c_str.into_raw()
    } else {
        std::ptr::null_mut()
    }
}

/// Returns the last completed word if a word boundary (delimiter/macro/restore) was just reached.
/// Must be freed with `fcitx5_vnchkey_free_string`.
#[no_mangle]
pub unsafe extern "C" fn fcitx5_vnchkey_get_completed_word(
    ctx: *mut Fcitx5VnchkeyContext,
) -> *mut c_char {
    if !ctx.is_null() {
        if let Some(word) = (*ctx).engine.last_completed_word() {
            let c_str = CString::new(word).unwrap_or_default();
            c_str.into_raw()
        } else {
            std::ptr::null_mut()
        }
    } else {
        std::ptr::null_mut()
    }
}

/// Checks whether the engine has an active composing word.
#[no_mangle]
pub unsafe extern "C" fn fcitx5_vnchkey_has_active_word(
    ctx: *mut Fcitx5VnchkeyContext,
) -> bool {
    if !ctx.is_null() {
        (*ctx).engine.has_active_word()
    } else {
        false
    }
}

/// Flips the current composing word between processed Vietnamese and raw input strokes.
/// Returns 1 if flipped, and writes the required backspaces and replacement string.
#[no_mangle]
pub unsafe extern "C" fn fcitx5_vnchkey_flip_word(
    ctx: *mut Fcitx5VnchkeyContext,
    out_backspaces: *mut usize,
    out_commit: *mut *mut c_char,
) -> c_int {
    if ctx.is_null() {
        return 0;
    }
    let ctx = &mut *ctx;
    if let Some(output) = ctx.engine.flip_current_word() {
        if !out_backspaces.is_null() {
            *out_backspaces = output.backspaces;
        }
        if !out_commit.is_null() {
            let c_str = CString::new(output.commit_text).unwrap_or_default();
            *out_commit = c_str.into_raw();
        }
        1
    } else {
        0
    }
}

/// Attempts to undo the last macro expansion on Backspace.
/// Returns 1 if reverted, and writes the required backspaces and trigger string to out_commit.
#[no_mangle]
pub unsafe extern "C" fn fcitx5_vnchkey_undo_macro(
    ctx: *mut Fcitx5VnchkeyContext,
    out_backspaces: *mut usize,
    out_commit: *mut *mut c_char,
) -> c_int {
    if ctx.is_null() {
        return 0;
    }
    let ctx = &mut *ctx;
    if let Some((backspaces, commit)) = ctx.engine.undo_macro_expansion() {
        if !out_backspaces.is_null() {
            *out_backspaces = backspaces;
        }
        if !out_commit.is_null() {
            let c_str = CString::new(commit).unwrap_or_default();
            *out_commit = c_str.into_raw();
        }
        1
    } else {
        0
    }
}

/// Reverts the active composing word back to raw keystrokes on Escape (Dokey style).
/// Returns 1 if reverted, and writes the required backspaces and raw string to out_commit.
#[no_mangle]
pub unsafe extern "C" fn fcitx5_vnchkey_undo_escape(
    ctx: *mut Fcitx5VnchkeyContext,
    out_backspaces: *mut usize,
    out_commit: *mut *mut c_char,
) -> c_int {
    if ctx.is_null() {
        return 0;
    }
    let ctx = &mut *ctx;
    if let Some(output) = ctx.engine.undo_to_raw() {
        if !out_backspaces.is_null() {
            *out_backspaces = output.backspaces;
        }
        if !out_commit.is_null() {
            let c_str = CString::new(output.commit_text).unwrap_or_default();
            *out_commit = c_str.into_raw();
        }
        1
    } else {
        0
    }
}

/// Rescues a raw sentence or phrase (Ngó Sen Word Rescue / Cứu Chữ).
/// If any words were converted to accented Vietnamese, allocates out_rescued and returns 1.
#[no_mangle]
pub unsafe extern "C" fn fcitx5_vnchkey_rescue_sentence(
    ctx: *mut Fcitx5VnchkeyContext,
    text: *const c_char,
    out_rescued: *mut *mut c_char,
) -> c_int {
    if ctx.is_null() || text.is_null() || out_rescued.is_null() {
        return 0;
    }
    let ctx = &mut *ctx;
    if let Ok(c_str) = std::ffi::CStr::from_ptr(text).to_str() {
        if let Some(rescued) = ctx.engine.rescue_sentence(c_str) {
            let res_c = CString::new(rescued).unwrap_or_default();
            *out_rescued = res_c.into_raw();
            return 1;
        }
    }
    0
}

/// Applies a tone or diacritic key to a selected text (Selection Accent Rescue / Cứu Dấu).
/// If a valid accent modification occurred, allocates out_transformed and returns 1.
#[no_mangle]
pub unsafe extern "C" fn fcitx5_vnchkey_apply_accent_to_selection(
    ctx: *mut Fcitx5VnchkeyContext,
    selection: *const c_char,
    ch: u32,
    out_transformed: *mut *mut c_char,
) -> c_int {
    if ctx.is_null() || selection.is_null() || out_transformed.is_null() {
        return 0;
    }
    let ctx = &mut *ctx;
    let key_char = match char::from_u32(ch) {
        Some(c) => c,
        None => return 0,
    };
    if let Ok(c_str) = std::ffi::CStr::from_ptr(selection).to_str() {
        if let Some(transformed) = ctx.engine.apply_accent_to_selection(c_str, key_char) {
            let res_c = CString::new(transformed).unwrap_or_default();
            *out_transformed = res_c.into_raw();
            return 1;
        }
    }
    0
}

/// Loads custom macros from a configuration file.
#[no_mangle]
pub unsafe extern "C" fn fcitx5_vnchkey_load_macros(
    ctx: *mut Fcitx5VnchkeyContext,
    path: *const c_char,
) {
    if !ctx.is_null() && !path.is_null() {
        if let Ok(c_str) = std::ffi::CStr::from_ptr(path).to_str() {
            let ctx = &mut *ctx;
            ctx.engine.load_macros_from_file(c_str);
        }
    }
}

/// Loads custom user dictionary words from a configuration file.
#[no_mangle]
pub unsafe extern "C" fn fcitx5_vnchkey_load_dictionary(
    ctx: *mut Fcitx5VnchkeyContext,
    path: *const c_char,
) {
    if !ctx.is_null() && !path.is_null() {
        if let Ok(c_str) = std::ffi::CStr::from_ptr(path).to_str() {
            let ctx = &mut *ctx;
            ctx.engine.load_dictionary_from_file(c_str);
        }
    }
}

/// Sets active input method (0 = Auto, 1 = Telex, 2 = VNI).
#[no_mangle]
pub unsafe extern "C" fn fcitx5_vnchkey_set_method(ctx: *mut Fcitx5VnchkeyContext, method: u32) {
    if !ctx.is_null() {
        let m = match method {
            1 => vnchkey_core::InputMethod::Telex,
            2 => vnchkey_core::InputMethod::Vni,
            _ => vnchkey_core::InputMethod::Auto,
        };
        (*ctx).engine.set_method(m);
    }
}

/// Returns active input method (0 = Auto, 1 = Telex, 2 = VNI).
#[no_mangle]
pub unsafe extern "C" fn fcitx5_vnchkey_get_method(ctx: *mut Fcitx5VnchkeyContext) -> u32 {
    if !ctx.is_null() {
        match (*ctx).engine.method() {
            vnchkey_core::InputMethod::Auto => 0,
            vnchkey_core::InputMethod::Telex => 1,
            vnchkey_core::InputMethod::Vni => 2,
            vnchkey_core::InputMethod::SimpleTelex => 3,
        }
    } else {
        0
    }
}

/// Configures Restore English (auto-restore).
#[no_mangle]
pub unsafe extern "C" fn fcitx5_vnchkey_set_auto_restore(ctx: *mut Fcitx5VnchkeyContext, enabled: bool) {
    if !ctx.is_null() {
        (*ctx).engine.set_auto_restore_english(enabled);
    }
}

/// Returns whether Restore English is enabled.
#[no_mangle]
pub unsafe extern "C" fn fcitx5_vnchkey_get_auto_restore(ctx: *mut Fcitx5VnchkeyContext) -> bool {
    if !ctx.is_null() {
        (*ctx).engine.auto_restore_english()
    } else {
        false
    }
}

/// Configures Chánh Tả spell checking.
#[no_mangle]
pub unsafe extern "C" fn fcitx5_vnchkey_set_spell_check(ctx: *mut Fcitx5VnchkeyContext, enabled: bool) {
    if !ctx.is_null() {
        (*ctx).engine.set_spell_check(enabled);
    }
}

/// Returns whether Chánh Tả spell checking is enabled.
#[no_mangle]
pub unsafe extern "C" fn fcitx5_vnchkey_get_spell_check(ctx: *mut Fcitx5VnchkeyContext) -> bool {
    if !ctx.is_null() {
        (*ctx).engine.spell_check()
    } else {
        false
    }
}

/// Configures User Dictionary of English & custom bypass words.
#[no_mangle]
pub unsafe extern "C" fn fcitx5_vnchkey_set_dictionary_enabled(ctx: *mut Fcitx5VnchkeyContext, enabled: bool) {
    if !ctx.is_null() {
        (*ctx).engine.set_enable_dictionary(enabled);
    }
}

/// Returns whether User Dictionary is enabled.
#[no_mangle]
pub unsafe extern "C" fn fcitx5_vnchkey_get_dictionary_enabled(ctx: *mut Fcitx5VnchkeyContext) -> bool {
    if !ctx.is_null() {
        (*ctx).engine.dictionary_enabled()
    } else {
        false
    }
}

/// Configures macro expansion.
#[no_mangle]
pub unsafe extern "C" fn fcitx5_vnchkey_set_macro_enabled(ctx: *mut Fcitx5VnchkeyContext, enabled: bool) {
    if !ctx.is_null() {
        (*ctx).engine.set_enable_macro(enabled);
    }
}

/// Returns whether macro expansion is enabled.
#[no_mangle]
pub unsafe extern "C" fn fcitx5_vnchkey_get_macro_enabled(ctx: *mut Fcitx5VnchkeyContext) -> bool {
    if !ctx.is_null() {
        (*ctx).engine.macro_enabled()
    } else {
        false
    }
}

/// Configures bracket transformation ([ -> ơ, ] -> ư).
#[no_mangle]
pub unsafe extern "C" fn fcitx5_vnchkey_set_bracket_transform(ctx: *mut Fcitx5VnchkeyContext, enabled: bool) {
    if !ctx.is_null() {
        (*ctx).engine.set_bracket_transform(enabled);
    }
}

/// Configures type w to produce ư rule (0 = NonStart, 1 = Always, 2 = Disabled).
#[no_mangle]
pub unsafe extern "C" fn fcitx5_vnchkey_set_w2u(ctx: *mut Fcitx5VnchkeyContext, mode: u32) {
    if !ctx.is_null() {
        let w2u = match mode {
            1 => vnchkey_core::W2UType::Always,
            2 => vnchkey_core::W2UType::Disabled,
            _ => vnchkey_core::W2UType::NonStart,
        };
        (*ctx).engine.set_type_w_to_u(w2u);
    }
}

/// Configures tone style (0 = Modern: oà, uý; 1 = Classic: hòa, ủy).
#[no_mangle]
pub unsafe extern "C" fn fcitx5_vnchkey_set_tone_style(ctx: *mut Fcitx5VnchkeyContext, style: u32) {
    if !ctx.is_null() {
        let tone_style = match style {
            1 => vnchkey_core::ToneStyle::Classic,
            _ => vnchkey_core::ToneStyle::Modern,
        };
        (*ctx).engine.set_tone_style(tone_style);
    }
}

/// Sets keystroke idle timeout in milliseconds (0 = disabled).
#[no_mangle]
pub unsafe extern "C" fn fcitx5_vnchkey_set_idle_timeout(ctx: *mut Fcitx5VnchkeyContext, ms: u64) {
    if !ctx.is_null() {
        (*ctx).engine.set_idle_timeout_ms(ms);
    }
}

/// Returns keystroke idle timeout in milliseconds.
#[no_mangle]
pub unsafe extern "C" fn fcitx5_vnchkey_get_idle_timeout(ctx: *mut Fcitx5VnchkeyContext) -> u64 {
    if !ctx.is_null() {
        (*ctx).engine.idle_timeout_ms()
    } else {
        0
    }
}

/// Returns tone style (0 = Modern: oà, uý; 1 = Classic: hòa, ủy).
#[no_mangle]
pub unsafe extern "C" fn fcitx5_vnchkey_get_tone_style(ctx: *mut Fcitx5VnchkeyContext) -> u32 {
    if !ctx.is_null() {
        match (*ctx).engine.tone_style() {
            vnchkey_core::ToneStyle::Modern => 0,
            vnchkey_core::ToneStyle::Classic => 1,
        }
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;
    use std::ptr;

    unsafe fn take_ffi_string(ptr: *mut c_char) -> String {
        if ptr.is_null() {
            return String::new();
        }
        let s = CStr::from_ptr(ptr).to_string_lossy().into_owned();
        fcitx5_vnchkey_free_string(ptr);
        s
    }

    #[test]
    fn test_ffi_null_pointer_safety() {
        unsafe {
            // Every FFI entry point must gracefully handle null pointers without crashing or UB
            let mut bs: usize = 0;
            let mut commit: *mut c_char = ptr::null_mut();
            assert_eq!(fcitx5_vnchkey_process_key(ptr::null_mut(), 'a' as u32, 0.0, &mut bs, &mut commit), 0);
            fcitx5_vnchkey_reset(ptr::null_mut());
            fcitx5_vnchkey_set_charset(ptr::null_mut(), 1);
            fcitx5_vnchkey_handle_backspace(ptr::null_mut());
            assert!(fcitx5_vnchkey_get_current_word(ptr::null_mut()).is_null());
            assert!(fcitx5_vnchkey_get_completed_word(ptr::null_mut()).is_null());
            assert!(!fcitx5_vnchkey_has_active_word(ptr::null_mut()));
            assert_eq!(fcitx5_vnchkey_flip_word(ptr::null_mut(), &mut bs, &mut commit), 0);
            assert_eq!(fcitx5_vnchkey_undo_macro(ptr::null_mut(), &mut bs, &mut commit), 0);
            assert_eq!(fcitx5_vnchkey_undo_escape(ptr::null_mut(), &mut bs, &mut commit), 0);
            assert_eq!(fcitx5_vnchkey_rescue_sentence(ptr::null_mut(), ptr::null(), &mut commit), 0);
            assert_eq!(fcitx5_vnchkey_apply_accent_to_selection(ptr::null_mut(), ptr::null(), 's' as u32, &mut commit), 0);
            fcitx5_vnchkey_free_string(ptr::null_mut());
            fcitx5_vnchkey_free(ptr::null_mut());
        }
    }

    #[test]
    fn test_ffi_full_lifecycle_and_memory_management() {
        unsafe {
            let ctx = fcitx5_vnchkey_init_with_method(1); // Telex
            assert!(!ctx.is_null());

            // Verify configuration getters & setters over C-ABI
            assert_eq!(fcitx5_vnchkey_get_method(ctx), 1);
            fcitx5_vnchkey_set_auto_restore(ctx, true);
            assert!(fcitx5_vnchkey_get_auto_restore(ctx));
            fcitx5_vnchkey_set_spell_check(ctx, true);
            assert!(fcitx5_vnchkey_get_spell_check(ctx));

            // Type "tieengs" -> "tiếng"
            let mut buffer = String::new();
            for (i, ch) in "tieengs".chars().enumerate() {
                let mut bs: usize = 0;
                let mut out_ptr: *mut c_char = ptr::null_mut();
                let consumed = fcitx5_vnchkey_process_key(ctx, ch as u32, (i * 20) as f64, &mut bs, &mut out_ptr);
                let commit = take_ffi_string(out_ptr);
                if consumed != 0 {
                    let keep = buffer.chars().count().saturating_sub(bs);
                    buffer = buffer.chars().take(keep).collect();
                    buffer.push_str(&commit);
                } else {
                    buffer.push(ch);
                }
            }
            assert_eq!(buffer, "tiếng");
            assert!(fcitx5_vnchkey_has_active_word(ctx));

            let current = take_ffi_string(fcitx5_vnchkey_get_current_word(ctx));
            assert_eq!(current, "tiếng");

            // Test Ngó Sen Word Rescue over FFI
            let raw_c = CString::new("chaof banj").unwrap();
            let mut rescued_ptr: *mut c_char = ptr::null_mut();
            let rescued_ok = fcitx5_vnchkey_rescue_sentence(ctx, raw_c.as_ptr(), &mut rescued_ptr);
            assert_eq!(rescued_ok, 1);
            assert_eq!(take_ffi_string(rescued_ptr), "chào bạn");

            // Test Selection Accent Rescue over FFI
            let sel_c = CString::new("tiêng").unwrap();
            let mut sel_out: *mut c_char = ptr::null_mut();
            let sel_ok = fcitx5_vnchkey_apply_accent_to_selection(ctx, sel_c.as_ptr(), 's' as u32, &mut sel_out);
            assert_eq!(sel_ok, 1);
            assert_eq!(take_ffi_string(sel_out), "tiếng");

            fcitx5_vnchkey_free(ctx);
        }
    }

    #[test]
    fn test_ffi_rapid_focus_switch_stress() {
        unsafe {
            let ctx = fcitx5_vnchkey_init();
            // Simulate 5,000 rapid window focus switches (Alt+Tab between Terminal Direct and Browser Preedit)
            for i in 0..5_000 {
                let mut bs: usize = 0;
                let mut out_ptr: *mut c_char = ptr::null_mut();
                fcitx5_vnchkey_process_key(ctx, 'v' as u32, i as f64, &mut bs, &mut out_ptr);
                fcitx5_vnchkey_free_string(out_ptr);

                let mut out_ptr2: *mut c_char = ptr::null_mut();
                fcitx5_vnchkey_process_key(ctx, 'i' as u32, (i + 1) as f64, &mut bs, &mut out_ptr2);
                fcitx5_vnchkey_free_string(out_ptr2);

                // Immediate focus switch reset mid-syllable
                fcitx5_vnchkey_reset(ctx);
                assert!(!fcitx5_vnchkey_has_active_word(ctx));
            }
            fcitx5_vnchkey_free(ctx);
        }
    }
}

