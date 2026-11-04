//! C ABI Bridge for Fcitx5 Addon Integration.

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use vnchkey_core::{Engine, EngineConfig, EngineOutput};

/// Opaque handle to the vnchKey Rust Engine.
pub struct Fcitx5VnchkeyContext {
    engine: Engine,
}

/// Initializes a new vnchKey engine instance.
#[no_mangle]
pub extern "C" fn fcitx5_vnchkey_init() -> *mut Fcitx5VnchkeyContext {
    let config = EngineConfig::default();
    let ctx = Box::new(Fcitx5VnchkeyContext {
        engine: Engine::new(config),
    });
    Box::into_raw(ctx)
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
