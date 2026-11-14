//! Nightmare Edge-Case & Real-World Chaos Test Suite for vnchKey (`vnchkey-core`).
//!
//! Covers 80+ extreme real-world typing scenarios:
//! 1. Out-of-order / sloppy typing (delayed tones, late `dd`, rapid tone cycling)
//! 2. Mechanical keyboard chatter / double-tap & triple-tap undo
//! 3. Shift-bleed casing & Initial Case Anchor
//! 4. Linguistic traps (`khuyur`, `ngoèo`, `khuấy`, `gi-`, `qu-`, `uơ` trio, French loanwords, Tây Nguyên)
//! 5. Bilingual Anh-Việt collisions & technical English traps
//! 6. Developer syntax, URLs, emails, CLI flags, timestamps & shortcodes
//! 7. Cursor reset, backspace word reopening, macro 2-step undo & HabitLearner safety

use vnchkey_core::engine::{Engine, EngineConfig, InputMethod, ToneStyle};

/// Helper simulating character-by-character typing with Micro-Diff LCP backspaces.
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

/// Helper simulating typing with explicit Backspace (`\x08`) and Escape (`\x1b`) control tokens.
fn type_with_controls(engine: &mut Engine, input: &str) -> String {
    let mut screen = String::new();
    let mut ts = 100.0;
    for ch in input.chars() {
        ts += 25.0;
        match ch {
            '\x08' => {
                // Physical Backspace
                if let Some((bs, commit)) = engine.undo_macro_expansion() {
                    let len = screen.chars().count();
                    let keep = len.saturating_sub(bs);
                    screen = screen.chars().take(keep).collect();
                    screen.push_str(&commit);
                } else {
                    engine.handle_backspace();
                    screen.pop();
                }
            }
            '\x1b' => {
                // Escape to raw
                if let Some(out) = engine.undo_to_raw() {
                    let len = screen.chars().count();
                    let keep = len.saturating_sub(out.backspaces);
                    screen = screen.chars().take(keep).collect();
                    screen.push_str(&out.commit_text);
                } else {
                    engine.reset();
                }
            }
            other => {
                let out = engine.process_key(other, ts);
                if out.consumed {
                    if out.backspaces > 0 {
                        let len = screen.chars().count();
                        let keep = len.saturating_sub(out.backspaces);
                        screen = screen.chars().take(keep).collect();
                    }
                    screen.push_str(&out.commit_text);
                } else {
                    screen.push(other);
                }
            }
        }
    }
    screen
}

// =============================================================================
// 1. NHÓM GÕ ẨU & NHẤN PHÍM SAI THỨ TỰ (OUT-OF-ORDER & SLOPPY TYPING)
// =============================================================================

#[test]
fn test_01_out_of_order_and_delayed_modifiers() {
    let mut engine = Engine::new(EngineConfig {
        method: InputMethod::Auto,
        tone_style: ToneStyle::Classic,
        ..EngineConfig::default()
    });

    let cases = [
        // Dấu thanh bấm trước khi gõ xong nguyên âm / âm cuối
        ("toasn ", "toán "),
        ("nguowif ", "người "),
        ("nguoifw ", "người "),
        ("dduowcj ", "được "),
        ("dduocwj ", "được "),
        ("duocwdj ", "được "),
        ("duocwjd ", "được "),
        ("khuyesne ", "khuyến "),
        ("tuyeesn ", "tuyến "),
        ("tuyesne ", "tuyến "),
        // Chưa gõ bù mũ ê mà bấm Space -> từ trung gian 'tuyén' tự động hoàn nguyên raw
        ("tuyesn ", "tuyesn "),
        ("nghieeng ", "nghiêng "),
        // Bấm đúp chữ 'đ' ở cuối từ khi đã gõ xong cả âm tiết
        ("duocjd ", "được "),
        ("duowngfd ", "đường "),
        ("dod ", "đo "),
        ("dand ", "đan "),
        ("dinhd ", "đinh "),
        // Đổi dấu liên tục ở cuối từ (sắc -> huyền -> hỏi -> ngã -> nặng -> xóa dấu)
        ("toansf ", "toàn "),
        ("toansfr ", "toản "),
        ("toansfrx ", "toãn "),
        ("toansfrxj ", "toạn "),
        ("toansfrxjz ", "toan "),
        // VNI gõ đảo thứ tự số
        ("duoc975 ", "được "),
        ("duoc795 ", "được "),
        ("duoc759 ", "được "),
        ("duoc579 ", "được "),
        ("nguoi72 ", "người "),
        ("nguoi27 ", "người "),
    ];

    for (raw, expected) in cases {
        engine.reset();
        let actual = type_stream(&mut engine, raw);
        assert_eq!(actual, expected, "Failed sloppy typing '{raw}': got '{actual}', expected '{expected}'");
    }
}

// =============================================================================
// 2. NHÓM BẪY BÀN PHÍM CƠ & NHẢ PHÍM SHIFT CHẬM (CHATTER & SHIFT-BLEED)
// =============================================================================

#[test]
fn test_02_mechanical_keyboard_chatter_and_double_tap_undo() {
    let mut engine = Engine::new(EngineConfig {
        method: InputMethod::Telex,
        auto_restore_english: false,
        spell_check: false,
        ..EngineConfig::default()
    });

    let cases = [
        // Double-tap tone undo
        ("toanss ", "toans "),
        ("toanff ", "toanf "),
        ("toanrr ", "toanr "),
        ("toanxx ", "toanx "),
        ("dduowcjj ", "đươcj "),
        // Triple-vowel / modifier undo
        ("aaa ", "aa "),
        ("eee ", "ee "),
        ("ooo ", "oo "),
        ("ddd ", "dd "),
        ("tww ", "tw "),
        ("khongoo ", "khongo "),
        ("toanaa ", "toana "),
        ("tiengee ", "tienge "),
    ];

    for (raw, expected) in cases {
        engine.reset();
        let actual = type_stream(&mut engine, raw);
        assert_eq!(actual, expected, "Failed chatter undo '{raw}': got '{actual}', expected '{expected}'");
    }
}

#[test]
fn test_03_shift_bleed_and_initial_case_anchor() {
    // Mặc định initial_case_anchor = false: bảo toàn ALL CAPS và TitleCase chuẩn
    let mut engine = Engine::new(EngineConfig {
        method: InputMethod::Auto,
        initial_case_anchor: false,
        ..EngineConfig::default()
    });

    assert_eq!(type_stream(&mut engine, "VIEEJT NAM "), "VIỆT NAM ");
    engine.reset();
    assert_eq!(type_stream(&mut engine, "DDUOWCJ "), "ĐƯỢC ");
    engine.reset();
    assert_eq!(type_stream(&mut engine, "QUOOCS "), "QUỐC ");
    engine.reset();
    // VNI TitleCase không bị Shift-bleed 'ĐƯờng'
    assert_eq!(type_stream(&mut engine, "Duong972 "), "Đường ");
    engine.reset();
    assert_eq!(type_stream(&mut engine, "DUong972 "), "Đường ");

    // Bật initial_case_anchor = true: tự sửa lỗi nhả Shift chậm trên bàn phím cơ
    engine.set_initial_case_anchor(true);
    engine.reset();
    assert_eq!(type_stream(&mut engine, "ĐƯợc "), "Được ");
    engine.reset();
    assert_eq!(type_stream(&mut engine, "VIệt "), "Việt ");
    engine.reset();
    assert_eq!(type_stream(&mut engine, "đƯỢc "), "được ");
}

// =============================================================================
// 3. NHÓM TỪ TIẾNG VIỆT HIỂM HÓC & BẪY NGUYÊN ÂM BA (LINGUISTIC TRAPS)
// =============================================================================

#[test]
fn test_04_vietnamese_linguistic_traps_and_dialects() {
    let mut engine = Engine::new(EngineConfig {
        method: InputMethod::Telex,
        tone_style: ToneStyle::Classic,
        auto_restore_english: true,
        spell_check: true,
        enable_dictionary: true,
        ..EngineConfig::default()
    });

    let cases = [
        // Cụm nguyên âm ba & vần khó
        ("khuyur ", "khuỷu "),
        ("ngoefo ", "ngoèo "),
        ("khoefo ", "khoèo "),
        ("khuayas ", "khuấy "),
        ("ngueechj ", "nguệch "),
        ("khueechs ", "khuếch "),
        ("uoorng ", "uổng "),
        ("uoonf ", "uồn "),
        ("huouw ", "hươu "),
        ("ruowuj ", "rượu "),
        ("buwouws ", "bướu "),
        ("khuwouws ", "khướu "),
        // Bộ ba ngoại lệ vần 'uơ' (thuở, quởn/quỡn/quơ, huơ)
        ("thuowr ", "thuở "),
        ("quonwr ", "quởn "),
        ("quonwx ", "quỡn "),
        ("quow ", "quơ "),
        ("huow ", "huơ "),
        // Âm đầu gi- và qu- (không rớt dấu nhầm lên i/u)
        ("gieengs ", "giếng "),
        ("giuowngf ", "giường "),
        ("gieeux ", "giễu "),
        ("giawcj ", "giặc "),
        ("giawtj ", "giặt "),
        ("giaays ", "giấy "),
        ("giayf ", "giày "),
        ("gif ", "gì "),
        ("ginf ", "gìn "),
        ("gir ", "gỉ "),
        ("quys ", "quý "),
        ("quyr ", "quỷ "),
        ("quyx ", "quỹ "),
        ("quyj ", "quỵ "),
        ("quoocs ", "quốc "),
        ("quaanr ", "quẩn "),
        ("quefo ", "quèo "),
        ("quetj ", "quẹt "),
        // Từ mượn gốc Pháp & phương ngữ Nam Bộ
        ("xooong ", "xoong "),
        ("booong ", "boong "),
        ("gooongf ", "goòng "),
        ("dziaf ", "dzìa "),
        ("dzoo ", "dzô "),
        ("dzis ", "dzí "),
        ("dzaayj ", "dzậy "),
        ("dzachs ", "dzách "),
        // Địa danh Tây Nguyên
        ("Ddawsk Lawsk ", "Đắk Lắk "),
        ("Kroong Noo ", "Krông Nô "),
        ("M'Ddrawsk ", "M'Đrắk "),
        ("Ea H'leo ", "Ea H'leo "),
        ("Pleiku ", "Pleiku "),
        ("Cuw M'garr ", "Cư M'gar "),
        ("Ia Grai ", "Ia Grai "),
        ("Chuw Proong ", "Chư Prông "),
        ("Chuw Pawh ", "Chư Păh "),
    ];

    for (raw, expected) in cases {
        engine.reset();
        let actual = type_stream(&mut engine, raw);
        assert_eq!(actual, expected, "Failed linguistic trap '{raw}': got '{actual}', expected '{expected}'");
    }
}
#[test]
fn test_05_tone_style_modern_vs_classic() {
let mut classic = Engine::new(EngineConfig {
method: InputMethod::Telex,
tone_style: ToneStyle::Classic,
..EngineConfig::default()
});
let mut modern = Engine::new(EngineConfig {
method: InputMethod::Telex,
tone_style: ToneStyle::Modern,
..EngineConfig::default()
});
assert_eq!(type_stream(&mut classic, "hoaf thuyr khoer "), "hòa thủy khỏe ");
assert_eq!(type_stream(&mut modern, "hoaf thuyr khoer "), "hoà thuỷ khoẻ ");
// Khi có phụ âm cuối hoặc nguyên âm có dấu phụ (ê), cả hai chuẩn đều đặt dấu trên âm chính
classic.reset();
modern.reset();
assert_eq!(type_stream(&mut classic, "toans thuees huees "), "toán thuế huế ");
assert_eq!(type_stream(&mut modern, "toans thuees huees "), "toán thuế huế ");
}
// =============================================================================
// 4. NHÓM BẪY GIAO THOA ANH - VIỆT & KÝ TỰ LẬP TRÌNH (CODE & BILINGUAL)
// =============================================================================
#[test]
fn test_06_bilingual_collisions_and_english_traps() {
let mut engine = Engine::new(EngineConfig {
method: InputMethod::Telex,
auto_restore_english: true,
spell_check: true,
enable_dictionary: true,
..EngineConfig::default()
});
// Chiều 1: Từ tiếng Việt trùng mặt chữ tiếng Anh PHẢI giữ dấu tiếng Việt
let vn_protected = [
("this ", "thí "),
("these ", "thế "),
("there ", "thể "),
("its ", "ít "),
("chaos ", "cháo "),
("bus ", "bú "),
("has ", "há "),
("see ", "sê "),
("reset ", "rết "),
];
for (raw, expected) in vn_protected {
engine.reset();
let actual = type_stream(&mut engine, raw);
assert_eq!(actual, expected, "Vietnamese collision broken for '{raw}': got '{actual}', expected '{expected}'");
}
// Chiều 2: Từ tiếng Anh kết thúc bằng phím dấu Telex PHẢI khôi phục tiếng Anh
let en_restored = [
("status ", "status "),
("focus ", "focus "),
("virus ", "virus "),
("plus ", "plus "),
("class ", "class "),
("process ", "process "),
("express ", "express "),
("windows ", "windows "),
("users ", "users "),
("filters ", "filters "),
("server ", "server "),
("user ", "user "),
("render ", "render "),
("filter ", "filter "),
("docker ", "docker "),
("order ", "order "),
("error ", "error "),
("mirror ", "mirror "),
("cursor ", "cursor "),
("browser ", "browser "),
("diff ", "diff "),
("off ", "off "),
("linux ", "linux "),
("unix ", "unix "),
("nginx ", "nginx "),
("prefix ", "prefix "),
("suffix ", "suffix "),
("syntax ", "syntax "),
("regex ", "regex "),
("matrix ", "matrix "),
("index ", "index "),
("context ", "context "),
("firefox ", "firefox "),
("window ", "window "),
("view ", "view "),
];
for (raw, expected) in en_restored {
engine.reset();
let actual = type_stream(&mut engine, raw);
assert_eq!(actual, expected, "English trap not restored for '{raw}': got '{actual}', expected '{expected}'");
}
}
#[test]
fn test_07_developer_syntax_urls_cli_and_shortcodes() {
let mut engine = Engine::new(EngineConfig {
method: InputMethod::Telex,
auto_restore_english: true,
spell_check: true,
enable_dictionary: true,
..EngineConfig::default()
});
let code_cases = [
// Biến camelCase, snake_case, leading underscore, dot accessor
("useState ", "useState "),
("parseSyllable ", "parseSyllable "),
("user_id ", "user_id "),
("_variable ", "_variable "),
("init ", "init "),
("db.collection.findOne ", "db.collection.findOne "),
// URL, Email, Path & CLI flags
("https://github.com/no-vici/fcitx5-vnchkey ", "https://github.com/no-vici/fcitx5-vnchkey "),
("user@domain.com ", "user@domain.com "),
("./src/engine.rs ", "./src/engine.rs "),
("--enable-wayland-ime ", "--enable-wayland-ime "),
// Giờ giấc & shortcode 2 ký tự không có dấu ':' đóng không được bung nhầm
("10:30 ", "10:30 "),
("14:00 ", "14:00 "),
(":in ", ":in "),
(":pi ", ":pi "),
// Shortcode hợp lệ có dấu ':' đóng hoặc >= 3 ký tự với Space
(":in: ", "∈ "),
(":pi: ", "π "),
(":smile: ", "😄 "),
(":smile ", "😄 "),
];
for (raw, expected) in code_cases {
engine.reset();
let actual = type_stream(&mut engine, raw);
assert_eq!(actual, expected, "Developer/URL/Shortcode failed for '{raw}': got '{actual}', expected '{expected}'");
}
}
// =============================================================================
// 5. NHÓM CON TRỎ, HOÀN TÁC & BẢO VỆ HABIT LEARNER (CONTROLS & STATE SAFETY)
// =============================================================================
#[test]
fn test_08_backspace_reopen_escape_and_habit_learner_safety() {
let mut engine = Engine::new(EngineConfig {
method: InputMethod::Telex,
habit_learning_enabled: true,
habit_promote_threshold: 2,
..EngineConfig::default()
});
// 1. Hồi sinh từ sau phím Space bằng Backspace (tiêng + Space + Backspace + s -> tiếng)
assert_eq!(type_with_controls(&mut engine, "tieeng \x08s "), "tiếng ");
// 2. Hoàn nguyên từ bằng Escape (tiếng + Esc -> tieengs)
engine.reset();
assert_eq!(type_with_controls(&mut engine, "tieengs\x1b "), "tieengs ");
// 3. Hoàn tác Macro 2 bước & Hút dấu câu thông minh
engine.reset();
// Bước 1: vn + Space -> "Việt Nam ", Backspace 1 -> xóa dấu cách -> "Việt Nam"
assert_eq!(type_with_controls(&mut engine, "vn \x08"), "Việt Nam");
// Bước 2: Backspace lần 2 -> hoàn nguyên về "vn"
engine.reset();
assert_eq!(type_with_controls(&mut engine, "vn \x08\x08"), "vn");
// Hút dấu câu sau macro: vn + Space + ',' -> "Việt Nam,"
engine.reset();
assert_eq!(type_stream(&mut engine, "vn , "), "Việt Nam, ");
// 4. Bảo vệ HabitLearner: Xóa một phần từ tiếng Việt để sửa lỗi (thương -> xóa 'ơng' -> thư)
// tuyệt đối không được học nhầm 'thư' là từ tiếng Anh!
engine.reset();
for _ in 0..5 {
let out = type_with_controls(&mut engine, "thuwowng\x08\x08\x08 ");
assert_eq!(out, "thư ");
}
assert!(
!engine.habit_learner().is_learned_word("thuw"),
"HabitLearner must NOT learn partial Vietnamese backspace edits!"
);
}
