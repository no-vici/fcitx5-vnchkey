//! Phase 0 audit: representative VietTelex-inspired cases.
//! Passing tests document current behavior; ignored tests mark gaps/opportunities.
//! Run ignored: `cargo test -p vnchkey-core --test viettelex_audit_tests -- --ignored`

use vnchkey_core::engine::{Engine, EngineConfig, InputMethod};

fn type_stream(engine: &mut Engine, input: &str) -> String {
    let mut screen = String::new();
    let mut ts = 100.0;
    for ch in input.chars() {
        let out = engine.process_key(ch, ts);
        ts += 25.0;
        if out.consumed {
            if out.backspaces > 0 {
                let len = screen.chars().count();
                let keep = len.saturating_sub(out.backspaces);
                screen = screen.chars().take(keep).collect();
            }
            screen.push_str(&out.commit_text);
        } else {
            screen.push(ch);
        }
    }
    screen
}

fn telex_engine(cfg: EngineConfig) -> Engine {
    let mut cfg = cfg;
    cfg.method = InputMethod::Telex;
    Engine::new(cfg)
}

// ---------------------------------------------------------------------------
// Auto-circumflex / smart vowel completion (VietTelex AutoCircumflex)
// vnchKey has `smart_fuzzy_vowel` - audit overlap.
// ---------------------------------------------------------------------------

#[test]
fn audit_smart_fuzzy_vowel_vietj() {
    let mut e = telex_engine(EngineConfig {
        smart_fuzzy_vowel: true,
        ..EngineConfig::default()
    });
    assert_eq!(type_stream(&mut e, "vietj "), "việt ");
}

#[test]
fn audit_smart_fuzzy_vowel_viejt() {
    let mut e = telex_engine(EngineConfig {
        smart_fuzzy_vowel: true,
        ..EngineConfig::default()
    });
    assert_eq!(type_stream(&mut e, "viejt "), "việt ");
}

#[test]
#[ignore = "gap: smart_fuzzy_vowel/dict-guarded resolver does not resolve benhj->bệnh"]
fn audit_smart_fuzzy_vowel_benhj() {
    let mut e = telex_engine(EngineConfig {
        smart_fuzzy_vowel: true,
        ..EngineConfig::default()
    });
    assert_eq!(type_stream(&mut e, "benhj "), "bệnh ");
}

#[test]
#[ignore = "gap: smart_fuzzy_vowel/dict-guarded resolver does not resolve luanj->luận"]
fn audit_smart_fuzzy_vowel_luanj() {
    let mut e = telex_engine(EngineConfig {
        smart_fuzzy_vowel: true,
        ..EngineConfig::default()
    });
    assert_eq!(type_stream(&mut e, "luanj "), "luận ");
}

#[test]
#[ignore = "gap: smart_fuzzy_vowel/dict-guarded resolver produces hương, not hưởng"]
fn audit_smart_fuzzy_vowel_huowng() {
    let mut e = telex_engine(EngineConfig {
        smart_fuzzy_vowel: true,
        ..EngineConfig::default()
    });
    assert_eq!(type_stream(&mut e, "huowng "), "hưởng ");
}

#[test]
fn audit_dict_guarded_fuzzy_resolves_even_when_smart_fuzzy_off() {
    let mut e = telex_engine(EngineConfig {
        smart_fuzzy_vowel: false,
        ..EngineConfig::default()
    });
    // vnchKey has an always-on dict-guarded fuzzy vowel resolver for invalid syllables.
    assert_eq!(type_stream(&mut e, "vietj "), "việt ");
}

// ---------------------------------------------------------------------------
// Abbreviation doubler (VietTelex AbbreviationDoubler)
// ---------------------------------------------------------------------------

#[test]
#[ignore = "gap: VNDD/HĐND-style ALL-CAPS abbreviation doubler not confirmed"]
fn audit_abbrev_vndd() {
    let mut e = telex_engine(EngineConfig::default());
    assert_eq!(type_stream(&mut e, "VNDD "), "VNĐ ");
}

#[test]
#[ignore = "gap: VNDD/HĐND-style ALL-CAPS abbreviation doubler not confirmed"]
fn audit_abbrev_hdnd() {
    let mut e = telex_engine(EngineConfig::default());
    assert_eq!(type_stream(&mut e, "HDNDD "), "HĐND ");
}

// ---------------------------------------------------------------------------
// Collision preference (VietTelex CollisionPreference)
// ---------------------------------------------------------------------------

#[test]
fn audit_collision_list_default_keeps_english() {
    let mut e = telex_engine(EngineConfig {
        spell_check: true,
        auto_restore_english: true,
        collision_prefers_vietnamese: false,
        ..EngineConfig::default()
    });
    assert_eq!(type_stream(&mut e, "list "), "list ");
}

#[test]
fn audit_collision_list_prefers_vietnamese() {
    let mut e = telex_engine(EngineConfig {
        spell_check: true,
        auto_restore_english: true,
        collision_prefers_vietnamese: true,
        ..EngineConfig::default()
    });
    assert_eq!(type_stream(&mut e, "list "), "lít ");
}

#[test]
fn audit_collision_last_prefers_vietnamese() {
    let mut e = telex_engine(EngineConfig {
        spell_check: true,
        auto_restore_english: true,
        collision_prefers_vietnamese: true,
        ..EngineConfig::default()
    });
    assert_eq!(type_stream(&mut e, "last "), "lát ");
}

#[test]
fn audit_collision_prefers_vn_does_not_break_non_vn_english() {
    let mut e = telex_engine(EngineConfig {
        spell_check: true,
        auto_restore_english: true,
        collision_prefers_vietnamese: true,
        ..EngineConfig::default()
    });
    // "text" -> "tẽt" is not a valid Vietnamese word; should still restore to English.
    assert_eq!(type_stream(&mut e, "text "), "text ");
}

// ---------------------------------------------------------------------------
// Acronym / glue behavior
// ---------------------------------------------------------------------------

#[test]
#[ignore = "gap: OS becomes Ó because 'ó' is in VN dict and acronym_bypass yields to VN dict"]
fn audit_acronym_bypass_os() {
    let mut e = telex_engine(EngineConfig {
        acronym_bypass: true,
        auto_restore_english: true,
        ..EngineConfig::default()
    });
    // Desired by many users: keep OS. Current behavior: "Ó " (ó is in Vietnamese dict).
    assert_eq!(type_stream(&mut e, "OS "), "OS ");
}

#[test]
fn audit_acronym_bypass_os_current_behavior() {
    let mut e = telex_engine(EngineConfig {
        acronym_bypass: true,
        auto_restore_english: true,
        ..EngineConfig::default()
    });
    // Documents current behavior; see ignored gap test above.
    assert_eq!(type_stream(&mut e, "OS "), "Ó ");
}

#[test]
#[ignore = "gap: verify rare ALL-CAPS acronym behavior"]
fn audit_acronym_bypass_rare_caps() {
    let mut e = telex_engine(EngineConfig {
        acronym_bypass: true,
        auto_restore_english: true,
        ..EngineConfig::default()
    });
    // VietTelex keeps rare ALL-CAPS as acronym; vnchKey behavior may differ.
    assert_eq!(type_stream(&mut e, "NASA "), "NASA ");
}

#[test]
#[ignore = "gap: verify code/path glue behavior"]
fn audit_glue_underscore_identifier() {
    let mut e = telex_engine(EngineConfig::default());
    // Typing inside/after identifier-like tokens should not corrupt.
    let out = type_stream(&mut e, "my_var ");
    assert_eq!(out, "my_var ");
}

#[test]
#[ignore = "gap: verify number+unit glue behavior"]
fn audit_glue_number_unit() {
    let mut e = telex_engine(EngineConfig::default());
    let out = type_stream(&mut e, "5k/thang ");
    // Desired: keep predictable, not garbled; exact expected depends on policy.
    assert_eq!(out, "5k/tháng ");
}
// ---------------------------------------------------------------------------
// Frequency disambiguation (Phase 3) - default OFF, easy to remove
// ---------------------------------------------------------------------------

#[test]
fn audit_freq_disambig_off_keeps_default_english_restore() {
    let mut e = telex_engine(EngineConfig {
        spell_check: true,
        auto_restore_english: true,
        frequency_disambiguation: false,
        ..EngineConfig::default()
    });
    // "rust" -> "rút" is a valid VN word, but default len>=4 English keyword restore keeps English.
    assert_eq!(type_stream(&mut e, "rust "), "rust ");
}

#[test]
fn audit_freq_disambig_on_prefers_vn_when_vn_score_strong() {
    let mut e = telex_engine(EngineConfig {
        spell_check: true,
        auto_restore_english: true,
        frequency_disambiguation: true,
        ..EngineConfig::default()
    });
    // "rust" (en 89, vn 152): VN-dominant collision; prefer Vietnamese when enabled.
    assert_eq!(type_stream(&mut e, "rust "), "rút ");
}

#[test]
fn audit_freq_disambig_on_keeps_english_when_en_dominant() {
    let mut e = telex_engine(EngineConfig {
        spell_check: true,
        auto_restore_english: true,
        frequency_disambiguation: true,
        ..EngineConfig::default()
    });
    // "best" (en 173, vn 95): EN-dominant; keep English restore.
    assert_eq!(type_stream(&mut e, "best "), "best ");
}
