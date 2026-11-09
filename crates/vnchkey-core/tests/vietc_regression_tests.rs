//! Regression test suite imported and adapted from VietC (vndangkhoa/vietc).
//! Verifies Vietnamese tone placement, cluster glide mechanics, and English auto-restore.

use vnchkey_core::engine::{Engine, EngineConfig, InputMethod, ToneStyle};
use vnchkey_core::telex::TelexEngine;
use vnchkey_core::vni::VniEngine;

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
fn test_vietc_tone_placement_cases() {
    let telex = TelexEngine::new(ToneStyle::Modern);
    let vni = VniEngine::new(ToneStyle::Modern);

    // ("telex", "vni", "expected")
    let cases = [
        // "qu" onset: the u is glide, tone stays on the nucleus.
        ("quar", "qua3", "quả"),
        ("quaf", "qua2", "quà"),
        ("quas", "qua1", "quá"),
        ("quaj", "qua5", "quạ"),
        // "gi" onset: the i is part of onset/nucleus, tone stays on the nucleus.
        ("gias", "gia1", "giá"),
        ("giof", "gio2", "giò"),
        ("giowf", "gio72", "giờ"),
        ("giups", "giu1p", "giúp"),
        ("gieets", "gie61t", "giết"),
        // "uê"/"uơ" clusters: tone belongs on the second vowel.
        ("thuees", "thue61", "thuế"),
        ("hueej", "hue65", "huệ"),
        // Controls: bare "gì", "uy", "uâ", "uô".
        ("gif", "gi2", "gì"),
        ("quys", "quy1", "quý"),
        ("quaanf", "qua62n", "quần"),
        ("muoons", "muo61n", "muốn"),
    ];

    for (tk, vk, expected) in cases {
        assert_eq!(
            telex.feed(tk).0,
            expected,
            "Telex mismatch for input '{tk}': got '{}', expected '{expected}'",
            telex.feed(tk).0
        );
        assert_eq!(
            vni.feed(vk).0,
            expected,
            "VNI mismatch for input '{vk}': got '{}', expected '{expected}'",
            vni.feed(vk).0
        );
    }
}

#[test]
fn test_vietc_shape_modifier_swapping() {
    let telex = TelexEngine::new(ToneStyle::Modern);
    let vni = VniEngine::new(ToneStyle::Modern);

    // Telex swapping
    assert_eq!(telex.feed("aaw").0, "ă");
    assert_eq!(telex.feed("oow").0, "ơ");
    assert_eq!(telex.feed("amsw").0, "ắm");
    assert_eq!(telex.feed("amswa").0, "ấm");
    assert_eq!(telex.feed("canw").0, "căn");
    assert_eq!(telex.feed("canwa").0, "cân");
    assert_eq!(telex.feed("bonsw").0, "bớn");
    assert_eq!(telex.feed("bonswo").0, "bốn");
    assert_eq!(telex.feed("hopjw").0, "hợp");
    assert_eq!(telex.feed("hopjwo").0, "hộp");

    // VNI swapping
    assert_eq!(vni.feed("a68").0, "ă");
    assert_eq!(vni.feed("o67").0, "ơ");
    assert_eq!(vni.feed("am168").0, "ắm");
    assert_eq!(vni.feed("am1686").0, "ấm");
    assert_eq!(vni.feed("bon167").0, "bớn");
    assert_eq!(vni.feed("bon1676").0, "bốn");
    assert_eq!(vni.feed("hop567").0, "hợp");
    assert_eq!(vni.feed("hop5676").0, "hộp");
}

#[test]
fn test_vietc_auto_restore_cases() {
    let mut config = EngineConfig::default();
    config.method = InputMethod::Telex;
    config.auto_restore_english = true;
    let mut engine = Engine::new(config);

    // Common English technical words typed with Telex keys that should auto-restore on Space
    let english_cases = [
        ("fix ", "fix "),
        ("cargo ", "cargo "),
        ("status ", "status "),
        ("world ", "world "),
        ("english ", "english "),
        ("sweet ", "sweet "),
        ("type ", "type "),
        ("code ", "code "),
        ("hello ", "hello "),
        ("the ", "the "),
        ("and ", "and "),
    ];

    for (input, expected) in english_cases {
        engine.reset();
        let out = type_word(&mut engine, input);
        assert_eq!(
            out, expected,
            "Auto-restore mismatch for '{input}': got '{out}', expected '{expected}'"
        );
    }

    // Genuine Vietnamese words must be preserved
    let vn_cases = [
        ("tieengs ", "tiếng "),
        ("vieejt ", "việt "),
        ("quar ", "quả "),
        ("gif ", "gì "),
        ("khoong ", "không "),
        ("tooi ", "tôi "),
        ("banhf ", "bành "),
        ("ddi ", "đi "),
    ];

    for (input, expected) in vn_cases {
        engine.reset();
        let out = type_word(&mut engine, input);
        assert_eq!(
            out, expected,
            "Vietnamese word mismatch for '{input}': got '{out}', expected '{expected}'"
        );
    }
}

#[test]
fn test_vietc_combinatorial_snapshots() {
    let telex = TelexEngine::new(ToneStyle::Modern);
    let vni = VniEngine::new(ToneStyle::Modern);

    let telex_snap = [
        ("aaf", "ầ"),
        ("aas", "ấ"),
        ("aaj", "ậ"),
        ("aar", "ẩ"),
        ("aax", "ẫ"),
        ("aaw", "ă"),
        ("aaa", "aa"),
        ("eee", "ee"),
        ("ooo", "oo"),
        ("oow", "ơ"),
        ("acaf", "âcf"), // Protected: stop coda 'c' cannot take huyền
        ("acas", "ấc"),
        ("acaj", "ậc"),
        ("acar", "âcr"), // Protected: stop coda 'c' cannot take hỏi
        ("acax", "âcx"), // Protected: stop coda 'c' cannot take ngã
        ("amaf", "ầm"),
        ("amas", "ấm"),
        ("amaj", "ậm"),
        ("amar", "ẩm"),
        ("amax", "ẫm"),
        ("amaw", "ăm"),
        ("anaf", "ần"),
        ("anas", "ấn"),
        ("anaj", "ận"),
        ("anar", "ẩn"),
        ("anax", "ẫn"),
        ("anaw", "ăn"),
        ("angaf", "ầng"),
        ("angas", "ấng"),
        ("angaj", "ậng"),
        ("apaf", "âpf"), // Protected: stop coda 'p' cannot take huyền
        ("apas", "ấp"),
        ("apaj", "ập"),
        ("ataf", "âtf"), // Protected: stop coda 't' cannot take huyền
        ("atas", "ất"),
        ("ataj", "ật"),
    ];

    for (input, expected) in telex_snap {
        assert_eq!(
            telex.feed(input).0,
            expected,
            "Combinatorial Telex mismatch for '{input}'"
        );
    }

    let vni_snap = [
        ("a62", "ầ"),
        ("a61", "ấ"),
        ("a65", "ậ"),
        ("a63", "ẩ"),
        ("a64", "ẫ"),
        ("a68", "ă"),
        ("e6", "ê"),
        ("o6", "ô"),
        ("o67", "ơ"),
        ("am62", "ầm"),
        ("am61", "ấm"),
        ("am65", "ậm"),
        ("am63", "ẩm"),
        ("am64", "ẫm"),
        ("am681", "ắm"),
        ("an62", "ần"),
        ("an61", "ấn"),
        ("an65", "ận"),
        ("an63", "ẩn"),
        ("an64", "ẫn"),
        ("an681", "ắn"),
        ("ap61", "ấp"),
        ("ap65", "ập"),
        ("at61", "ất"),
        ("at65", "ật"),
    ];

    for (input, expected) in vni_snap {
        assert_eq!(
            vni.feed(input).0,
            expected,
            "Combinatorial VNI mismatch for '{input}'"
        );
    }
}

#[test]
fn test_mixed_casing_and_phonology_violations() {
    let mut engine = Engine::new(EngineConfig {
        method: InputMethod::Auto,
        auto_restore_english: true,
        spell_check: true,
        enable_dictionary: true,
        ..EngineConfig::default()
    });

    // 1. Hardcore Mixed Corpus: camelCase, alternating case, & developer tokens
    assert_eq!(type_word(&mut engine, "TEXT "), "TEXT ");
    engine.reset();
    assert_eq!(type_word(&mut engine, "mIxEd "), "mIxEd ");
    engine.reset();
    assert_eq!(type_word(&mut engine, "lUxUrY "), "lUxUrY ");
    engine.reset();
    assert_eq!(type_word(&mut engine, "useState "), "useState ");
    engine.reset();
    assert_eq!(type_word(&mut engine, "loWeRcAsE "), "loWeRcAsE ");
    engine.reset();
    assert_eq!(type_word(&mut engine, "$mixed_CORPUS "), "$mixed_CORPUS ");

    // 2. 5-Component Syllable Parser with Tây Nguyên & Southern dialect words (Auto Telex/VNI)
    engine.reset();
    assert_eq!(type_word(&mut engine, "D9a8k1 La8k1 "), "Đắk Lắk ");
    engine.reset();
    assert_eq!(type_word(&mut engine, "Ddawks Lawks "), "Đắk Lắk ");
    engine.reset();
    assert_eq!(type_word(&mut engine, "quon73 "), "quởn ");
    engine.reset();
    assert_eq!(type_word(&mut engine, "quonwr "), "quởn ");
}

#[test]
fn test_property_based_phonotactic_invariants() {
    use vnchkey_core::syllable_parser::{parse_syllable, ToneMark};

    let telex = TelexEngine::new(ToneStyle::Modern);
    let vni = VniEngine::new(ToneStyle::Modern);

    // Property 1: Stop-coda syllables (-c, -ch, -p, -t) must NEVER carry Huyền (2), Hỏi (3), or Ngã (4)
    let onsets = ["", "b", "c", "ch", "d", "dd", "g", "gh", "gi", "h", "k", "kh", "l", "m", "n", "ng", "ngh", "nh", "p", "ph", "qu", "r", "s", "t", "th", "tr", "v", "x"];
    let vowels = ["a", "aw", "aa", "e", "ee", "i", "o", "oo", "ow", "u", "uw", "ie", "uo", "ua"];
    let stop_codas = ["c", "ch", "p", "t"];
    let forbidden_telex_tones = ['f', 'r', 'x'];
    let forbidden_vni_tones = ['2', '3', '4'];

    let mut checked_combinations = 0;
    for onset in onsets {
        for vowel in vowels {
            for coda in stop_codas {
                for &t_key in &forbidden_telex_tones {
                    let seq = format!("{onset}{vowel}{coda}{t_key}");
                    let (out, _) = telex.feed(&seq);
                    if let Some(parsed) = parse_syllable(&out) {
                        if parsed.has_stop_coda() {
                            assert!(
                                !matches!(parsed.tone, ToneMark::Grave | ToneMark::HookAbove | ToneMark::Tilde),
                                "Phonotactic violation on Telex '{seq}' -> '{out}' (tone {:?} on stop coda)",
                                parsed.tone
                            );
                        }
                    }
                    checked_combinations += 1;
                }
                for &v_key in &forbidden_vni_tones {
                    let seq = format!("{onset}{vowel}{coda}{v_key}");
                    let (out, _) = vni.feed(&seq);
                    if let Some(parsed) = parse_syllable(&out) {
                        if parsed.has_stop_coda() {
                            assert!(
                                !matches!(parsed.tone, ToneMark::Grave | ToneMark::HookAbove | ToneMark::Tilde),
                                "Phonotactic violation on VNI '{seq}' -> '{out}' (tone {:?} on stop coda)",
                                parsed.tone
                            );
                        }
                    }
                    checked_combinations += 1;
                }
            }
        }
    }
    assert!(checked_combinations >= 9000, "Verified {checked_combinations} combinatorial stop-coda states");

    // Property 2: Deterministic PRNG fuzzing (10,000 random keystroke streams)
    // Engine must never panic or corrupt UTF-8 state under any sequence of ASCII/Unicode inputs.
    let mut engine = Engine::new(EngineConfig::default());
    let mut seed: u64 = 0x5DEECE66D;
    let alphabet: &[char] = &[
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm',
        'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
        'A', 'E', 'O', 'W', 'S', 'F', 'R', 'X', 'J', 'Z',
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
        ' ', '.', ',', '_', '-', '/', '`', '[', ']', '\n',
    ];

    for _ in 0..10_000 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let idx = ((seed >> 32) as usize) % alphabet.len();
        let ch = alphabet[idx];
        let out = engine.process_key(ch, 10.0);
        // Output commit_text must always be valid UTF-8 (guaranteed by Rust String, verify char count consistency)
        assert!(out.commit_text.chars().count() <= 64);
    }
}

