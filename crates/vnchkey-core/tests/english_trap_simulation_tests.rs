//! Automated English Trap Simulation & Collision Protection Tests
//!
//! Validates:
//! 1. Automated simulation of English/Developer traps: deformed sequences (e.g. `text` -> `tẽt`,
//!    `modern` -> `modẻn`, `exit` -> `ẽit`, `reset` -> `rết`) are automatically restored to English
//!    upon typing Space or word boundaries.
//! 2. Vietnamese collision protection: keystroke sequences that spell valid Vietnamese words
//!    (`chaos` -> `cháo`, `its` -> `ít`, `this` -> `thí`, `bus` -> `bú`, `has` -> `há`, `see` -> `sê`)
//!    are strictly protected and NEVER overwritten with English words.
//! 3. Backtick inline code protection: text following backticks (e.g. `` `vnchKey` some text ``)
//!    is not subjected to unwanted auto-capitalization.
//! 4. Token-scoped path guard: typing file paths (`./src`, `~/.config`, `/usr/bin`) only bypasses
//!    the current token and immediately restores full Vietnamese typing after Space.

use vnchkey_core::engine::{Engine, EngineConfig, InputMethod};

fn type_word(engine: &mut Engine, word: &str) -> String {
    let mut result = String::new();
    for (i, ch) in word.chars().enumerate() {
        let out = engine.process_key(ch, (i * 50) as f64);
        if out.consumed {
            if out.backspaces > 0 {
                let current_len = result.chars().count();
                let keep = current_len.saturating_sub(out.backspaces);
                result = result.chars().take(keep).collect();
            }
            result.push_str(&out.commit_text);
        } else {
            result.push(ch);
        }
    }
    result
}

#[test]
fn test_automated_english_trap_simulation() {
    let mut engine = Engine::new(EngineConfig {
        method: InputMethod::Telex,
        auto_restore_english: true,
        spell_check: true,
        enable_dictionary: true,
        ..EngineConfig::default()
    });

    // Tech & developer traps that deform into invalid Vietnamese syllables
    // Must cleanly auto-restore to English on Space:
    let traps = [
        ("text ", "text "),
        ("modern ", "modern "),
        ("exit ", "exit "),
        ("test ", "test "),
        ("window ", "window "),
        ("windows ", "windows "),
        ("server ", "server "),
        ("user ", "user "),
        ("users ", "users "),
        ("address ", "address "),
        ("series ", "series "),
        ("restore ", "restore "),
        ("refactor ", "refactor "),
        ("express ", "express "),
        ("post ", "post "),
        ("linux ", "linux "),
        ("filter ", "filter "),
        ("filters ", "filters "),
        ("struct ", "struct "),
        ("plus ", "plus "),
    ];

    for (input, expected) in traps {
        engine.reset();
        let actual = type_word(&mut engine, input);
        assert_eq!(
            actual, expected,
            "Failed trap restore for '{input}': got '{actual}', expected '{expected}'"
        );
    }
}

#[test]
fn test_vietnamese_collision_exceptions_protected() {
    let mut engine = Engine::new(EngineConfig {
        method: InputMethod::Telex,
        auto_restore_english: true,
        spell_check: true,
        enable_dictionary: true,
        ..EngineConfig::default()
    });

    // Keystroke sequences that collide with valid Vietnamese words
    // MUST remain Vietnamese and NEVER be turned into English:
    let collisions = [
        ("chaos ", "cháo "),    // Ăn cháo (NOT English 'chaos')
        ("its ", "ít "),        // Chút ít (NOT English 'its')
        ("this ", "thí "),      // Thí sinh, thí dụ (NOT English 'this')
        ("bus ", "bú "),        // Bú sữa (NOT English 'bus')
        ("has ", "há "),        // Há miệng (NOT English 'has')
        ("see ", "sê "),        // Sê-ri (NOT English 'see')
        ("these ", "thế "),    // Thực tế, thế giới (NOT English 'these')
        ("there ", "thể "),    // Có thể, cơ thể (NOT English 'there')
        ("reset ", "rết "),    // Con rết, rắn rết (NOT English 'reset')
        ("son ", "son "),      // Son môi / son sắt (stays valid Vietnamese)
        ("con ", "con "),      // Con cái
    ];

    for (input, expected) in collisions {
        engine.reset();
        let actual = type_word(&mut engine, input);
        assert_eq!(
            actual, expected,
            "Collision violation for '{input}': got '{actual}', expected '{expected}'"
        );
    }
}

#[test]
fn test_backtick_inline_code_guard_and_no_autocapitalization() {
    let mut engine = Engine::new(EngineConfig {
        method: InputMethod::Telex,
        auto_restore_english: true,
        spell_check: true,
        enable_dictionary: true,
        ..EngineConfig::default()
    });

    // 1. Text following backtick `vnchKey` must preserve lowercase (no unwanted auto-capitalization)
    let typed = type_word(&mut engine, "`vnchKey` text after backtick ");
    assert_eq!(typed, "`vnchKey` text after backtick ");

    // 2. Direct contiguous text right after backtick (e.g. `vnchKey`<--text)
    engine.reset();
    let typed_contiguous = type_word(&mut engine, "`vnchKey`code ");
    assert_eq!(typed_contiguous, "`vnchKey`code ");

    // 3. Code identifiers inside backticks
    engine.reset();
    let typed_code = type_word(&mut engine, "`user_id` ");
    assert_eq!(typed_code, "`user_id` ");
}

#[test]
fn test_token_scoped_path_guard() {
    let mut engine = Engine::new(EngineConfig {
        method: InputMethod::Telex,
        auto_restore_english: true,
        spell_check: true,
        enable_dictionary: true,
        ..EngineConfig::default()
    });

    // Typing a filesystem path followed by Vietnamese:
    // Path token is preserved, and Vietnamese typing continues normally right after Space.
    let typed = type_word(&mut engine, "./src/engine.rs chaof banj ");
    assert_eq!(typed, "./src/engine.rs chào bạn ");

    engine.reset();
    let typed_config = type_word(&mut engine, "~/.config/fcitx5 tieengs vieetj ");
    assert_eq!(typed_config, "~/.config/fcitx5 tiếng việt ");
}
