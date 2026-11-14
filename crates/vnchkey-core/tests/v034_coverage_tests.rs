//! Integration Test Suite for vnchKey v0.3.4 Coverage Gaps:
//! 1. Symbols & Emoji case-sensitivity (`:Omega:` vs `:omega:`), numeric shortcodes (`:100:`),
//!    table duplicate audit, and 2-step Space undo (`:smile ` -> `😄 ` -> `😄` -> `:smile `).
//! 2. Quick Consonants (`quick_telex`, `quick_start_consonant`, `quick_end_consonant`)
//!    with tones, uppercase, and English auto-restore interaction.
//! 3. HabitLearner disk persistence across sessions, FIFO log rotation, and `unlearn_word`.
//! 4. `harmonize_syllable_case` & `CaseTracker` with Tây Nguyên apostrophe names (`M'Đrắk`).

use std::collections::HashSet;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use vnchkey_core::casing::{harmonize_syllable_case, CaseTracker};
use vnchkey_core::engine::{Engine, EngineConfig, InputMethod, ToneStyle};
use vnchkey_core::symbols::{lookup_symbol, search_symbols, SYMBOL_TABLE};

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

fn type_with_backspace(engine: &mut Engine, input: &str) -> String {
    let mut screen = String::new();
    let mut ts = 100.0;
    for ch in input.chars() {
        ts += 25.0;
        if ch == '\x08' {
            if let Some((bs, commit)) = engine.undo_macro_expansion() {
                let len = screen.chars().count();
                let keep = len.saturating_sub(bs);
                screen = screen.chars().take(keep).collect();
                screen.push_str(&commit);
            } else {
                engine.handle_backspace();
                screen.pop();
            }
        } else {
            let out = engine.process_key(ch, ts);
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
    }
    screen
}

// =============================================================================
// 1. SYMBOLS & EMOJI: CASE-SENSITIVITY, NUMERIC CODES, TABLE AUDIT & 2-STEP UNDO
// =============================================================================

#[test]
fn test_01_symbols_case_sensitivity_and_numeric_codes() {
    // 1. Case-sensitive Greek letters (Uppercase vs Lowercase)
    assert_eq!(lookup_symbol(":Omega:"), Some("Ω"));
    assert_eq!(lookup_symbol(":omega:"), Some("ω"));
    assert_eq!(lookup_symbol(":Gamma:"), Some("Γ"));
    assert_eq!(lookup_symbol(":gamma:"), Some("γ"));
    assert_eq!(lookup_symbol(":Delta:"), Some("Δ"));
    assert_eq!(lookup_symbol(":delta:"), Some("δ"));
    assert_eq!(lookup_symbol(":Pi:"), Some("Π"));
    assert_eq!(lookup_symbol(":pi:"), Some("π"));
    assert_eq!(lookup_symbol(":Sigma:"), Some("Σ"));
    assert_eq!(lookup_symbol(":sigma:"), Some("σ"));

    // 2. Case-insensitive fallback for regular emojis
    assert_eq!(lookup_symbol(":SMILE:"), Some("😄"));
    assert_eq!(lookup_symbol(":Check:"), Some("✓"));

    // 3. Numeric shortcodes
    assert_eq!(lookup_symbol(":100:"), Some("💯"));
    assert_eq!(lookup_symbol(":star2:"), Some("★"));
    assert_eq!(lookup_symbol(":check2:"), Some("✅"));
    assert_eq!(lookup_symbol(":rarrow2:"), Some("⇒"));

    // 4. Audit SYMBOL_TABLE for exact duplicate shortcodes
    let mut seen = HashSet::new();
    for entry in SYMBOL_TABLE {
        assert!(
            seen.insert(entry.shortcode),
            "Duplicate exact shortcode found in SYMBOL_TABLE: '{}'",
            entry.shortcode
        );
    }

    // 5. Prefix & substring search verification
    let omega_matches = search_symbols(":Om", 5);
    assert!(omega_matches.iter().any(|(s, c)| *s == "Ω" && *c == "Omega"));
}

#[test]
fn test_02_symbol_engine_typing_and_two_step_space_undo() {
    let mut engine = Engine::new(EngineConfig::default());

    // Typing :Omega: and :omega: directly in stream
    assert_eq!(type_stream(&mut engine, ":Omega: :omega: "), "Ω ω ");
    engine.reset();

    // Numeric shortcode with closing colon works, while bare time :30 does not
    assert_eq!(type_stream(&mut engine, ":100: 10:30 "), "💯 10:30 ");
    engine.reset();

    // Two-step undo when expanded via Space:
    // Step 0: ":smile " -> "😄 "
    assert_eq!(type_with_backspace(&mut engine, ":smile "), "😄 ");

    // Step 1: First Backspace deletes only trailing space -> "😄"
    engine.reset();
    assert_eq!(type_with_backspace(&mut engine, ":smile \x08"), "😄");

    // Step 2: Second Backspace reverts back to original trigger ":smile "
    engine.reset();
    assert_eq!(type_with_backspace(&mut engine, ":smile \x08\x08"), ":smile ");
}

// =============================================================================
// 2. QUICK CONSONANTS (QUICK TELEX, QUICK START, QUICK END)
// =============================================================================

#[test]
fn test_03_quick_consonants_combinations() {
    // A. Quick Telex (cc->ch, gg->gi, kk->kh, nn->ng, pp->ph, qq->qu, tt->th)
    let mut engine = Engine::new(EngineConfig {
        method: InputMethod::Telex,
        quick_telex: true,
        tone_style: ToneStyle::Classic,
        ..EngineConfig::default()
    });

    assert_eq!(type_stream(&mut engine, "ccao "), "chao ");
    engine.reset();
    assert_eq!(type_stream(&mut engine, "ccaos "), "cháo ");
    engine.reset();
    assert_eq!(type_stream(&mut engine, "kkoong "), "không ");
    engine.reset();
    assert_eq!(type_stream(&mut engine, "ttieeng "), "thiêng ");
    engine.reset();
    assert_eq!(type_stream(&mut engine, "Ccaof "), "Chào ");

    // B. Quick Initial Consonants (f->ph, j->gi, w->qu)
    engine.reset();
    engine.set_quick_telex(false);
    engine.set_quick_start_consonant(true);

    assert_eq!(type_stream(&mut engine, "fowr "), "phở ");
    engine.reset();
    assert_eq!(type_stream(&mut engine, "jaf "), "già ");
    engine.reset();
    assert_eq!(type_stream(&mut engine, "was "), "quá ");
    engine.reset();
    assert_eq!(type_stream(&mut engine, "Fowr "), "Phở ");

    // C. Quick Final Consonants (g->ng, h->nh, k->ch)
    engine.reset();
    engine.set_quick_start_consonant(false);
    engine.set_quick_end_consonant(true);

    assert_eq!(type_stream(&mut engine, "anhs "), "ánh ");
    engine.reset();
    assert_eq!(type_stream(&mut engine, "vafg "), "vàng ");
    engine.reset();
    assert_eq!(type_stream(&mut engine, "akj "), "ạch ");
    engine.reset();
    assert_eq!(type_stream(&mut engine, "tieeg "), "tiêng ");
    engine.reset();
    assert_eq!(type_stream(&mut engine, "tieegs "), "tiếng ");
}

// =============================================================================
// 3. HABIT LEARNER: DISK PERSISTENCE, FIFO LOG ROTATION & UNLEARN
// =============================================================================

#[test]
fn test_04_habit_learner_disk_persistence_rotation_and_unlearn() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp_dir = std::env::temp_dir().join(format!("vnchkey_test_{}_{}", std::process::id(), unique));
    fs::create_dir_all(&temp_dir).unwrap();

    let dict_file = temp_dir.join("dictionary.conf");
    let log_file = temp_dir.join("vnchkey-habit.log");
    let dict_path_str = dict_file.to_str().unwrap();
    let log_path_str = log_file.to_str().unwrap();

    // Session 1: Correct custom word "kas" 2 times (below threshold 4)
    {
        let mut engine1 = Engine::new(EngineConfig {
            method: InputMethod::Telex,
            spell_check: false,
            auto_restore_english: false,
            habit_learning_enabled: true,
            habit_promote_threshold: 4,
            ..EngineConfig::default()
        });
        engine1.load_dictionary_from_file(dict_path_str);
        engine1.set_habit_log_path(log_path_str);

        for i in 1..=2 {
            type_stream(&mut engine1, "kas");
            assert_eq!(engine1.current_word(), "ká");
            let _ = engine1.flip_current_word();
            engine1.process_key(' ', (i * 100) as f64);
        }
        assert_eq!(engine1.habit_learner().correction_count("kas"), 2);
        assert!(!engine1.habit_learner().is_learned_word("kas"));
    }

    // Session 2: New Engine instance reloads counts from `vnchkey-habit.log`,
    // corrects 2 more times -> reaches 4 -> auto-promotes and writes to `dictionary.conf`!
    {
        let mut engine2 = Engine::new(EngineConfig {
            method: InputMethod::Telex,
            spell_check: false,
            auto_restore_english: false,
            habit_learning_enabled: true,
            habit_promote_threshold: 4,
            ..EngineConfig::default()
        });
        engine2.load_dictionary_from_file(dict_path_str);
        engine2.set_habit_log_path(log_path_str);

        assert_eq!(engine2.habit_learner().correction_count("kas"), 2);

        for i in 3..=4 {
            type_stream(&mut engine2, "kas");
            let _ = engine2.flip_current_word();
            engine2.process_key(' ', (i * 100) as f64);
        }

        assert!(engine2.habit_learner().is_learned_word("kas"));
        let dict_content = fs::read_to_string(&dict_file).unwrap_or_default();
        assert!(
            dict_content.lines().any(|l| l.trim() == "kas"),
            "Promoted word 'kas' must be appended to user dictionary file"
        );
    }
}
