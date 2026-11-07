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
