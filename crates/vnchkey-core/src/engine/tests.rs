    use super::*;

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
    fn test_engine_auto_telex_and_vni() {
        let mut engine = Engine::new(EngineConfig {
            method: InputMethod::Auto,
            ..EngineConfig::default()
        });

        assert_eq!(type_word(&mut engine, "dduocwj "), "được ");
        assert_eq!(type_word(&mut engine, "duoc975 "), "được ");
        assert_eq!(type_word(&mut engine, "tinhf "), "tình ");
        assert_eq!(type_word(&mut engine, "tinh2 "), "tình ");
        assert_eq!(type_word(&mut engine, "duocwdj "), "được ");
        assert_eq!(type_word(&mut engine, "duocdj "), "được ");
        assert_eq!(type_word(&mut engine, "duocwdj7duo "), "được7duo ");

        let mut engine_classic_telex = Engine::new(EngineConfig {
            method: InputMethod::Telex,
            tone_style: ToneStyle::Classic,
            ..EngineConfig::default()
        });
        assert_eq!(type_word(&mut engine_classic_telex, "thuees "), "thuế ");
        assert_eq!(type_word(&mut engine_classic_telex, "huees "), "huế ");
        assert_eq!(type_word(&mut engine_classic_telex, "thusee "), "thuế ");
        assert_eq!(type_word(&mut engine_classic_telex, "husee "), "huế ");
        assert_eq!(type_word(&mut engine_classic_telex, "thuese "), "thuế ");
        assert_eq!(type_word(&mut engine_classic_telex, "huese "), "huế ");
    }

    #[test]
    fn test_vni_gio72_and_tone_editing() {
        let mut engine = Engine::new(EngineConfig {
            method: InputMethod::Auto,
            ..EngineConfig::default()
        });

        // gio72 -> giờ (both in Auto and VNI mode)
        assert_eq!(type_word(&mut engine, "gio72 "), "giờ ");
        // Edit tone after typing gio72: gio721 -> giớ
        assert_eq!(type_word(&mut engine, "gio721 "), "giớ ");
        // Remove tone with 0: gio720 -> giơ
        assert_eq!(type_word(&mut engine, "gio720 "), "giơ ");
        // Double tone digit undoes tone: gio722 -> giơ2
        assert_eq!(type_word(&mut engine, "gio722 "), "giơ2 ");
        // Words starting with s, r, x work with VNI digits in Auto mode
        assert_eq!(type_word(&mut engine, "sao1 "), "sáo ");
        assert_eq!(type_word(&mut engine, "ra2 "), "rà ");
        assert_eq!(type_word(&mut engine, "xa3 "), "xả ");
    }

    #[test]
    fn test_telex_khongo_and_end_of_word_undo() {
        let mut engine = Engine::new(EngineConfig {
            method: InputMethod::Telex,
            ..EngineConfig::default()
        });

        assert_eq!(type_word(&mut engine, "khongo "), "không ");
        assert_eq!(type_word(&mut engine, "khongos "), "khống ");
        assert_eq!(type_word(&mut engine, "khongosz "), "không ");
        assert_eq!(type_word(&mut engine, "khongoo "), "khongo ");
        assert_eq!(type_word(&mut engine, "toans "), "toán ");
        assert_eq!(type_word(&mut engine, "toanss "), "toans ");
        assert_eq!(type_word(&mut engine, "toansz "), "toan ");
        // Ensure "thí", "thế", "thể" work naturally in Telex
        assert_eq!(type_word(&mut engine, "this "), "thí ");
        assert_eq!(type_word(&mut engine, "thees "), "thế ");
        assert_eq!(type_word(&mut engine, "theer "), "thể ");
    }

    #[test]
    fn test_micro_diff_minimal_backspaces() {
        let mut engine = Engine::new(EngineConfig {
            method: InputMethod::Telex,
            ..EngineConfig::default()
        });

        // Type t-o-a-n: none of these should consume or emit backspaces
        for ch in ['t', 'o', 'a', 'n'] {
            let out = engine.process_key(ch, 0.0);
            assert!(!out.consumed);
            assert_eq!(out.backspaces, 0);
        }

        // Now type 's' -> transforms "toan" to "toán":
        // Common prefix is "toa", so only "n" (1 char) is deleted and "án" is committed!
        let out = engine.process_key('s', 0.0);
        assert!(out.consumed);
        assert_eq!(out.backspaces, 2);
        assert_eq!(out.commit_text, "án");
    }

    #[test]
    fn test_restore_english_and_chanh_ta_dictionary() {
        let mut engine = Engine::new(EngineConfig {
            method: InputMethod::Telex,
            auto_restore_english: true,
            spell_check: true,
            enable_dictionary: true,
            ..EngineConfig::default()
        });

        assert_eq!(type_word(&mut engine, "text "), "text ");
        // Single vowel words with accents ('á', 'í', 'ó', 'ú') are valid Vietnamese words and must NOT be restored to English
        assert_eq!(type_word(&mut engine, "as "), "á ");
        assert_eq!(type_word(&mut engine, "post "), "post ");
        assert_eq!(type_word(&mut engine, "general "), "general ");
        assert_eq!(type_word(&mut engine, "filter "), "filter ");
        assert_eq!(type_word(&mut engine, "filters "), "filters ");
        assert_eq!(type_word(&mut engine, "return "), "return ");
        assert_eq!(type_word(&mut engine, "struct "), "struct ");
        assert_eq!(type_word(&mut engine, "window "), "window ");
        assert_eq!(type_word(&mut engine, "windows "), "windows ");
        assert_eq!(type_word(&mut engine, "user "), "user ");
        assert_eq!(type_word(&mut engine, "users "), "users ");
        assert_eq!(type_word(&mut engine, "linux "), "linux ");
        assert_eq!(type_word(&mut engine, "center "), "center ");
        // Valid Southern & Standard Vietnamese words pass Chánh Tả check
        assert_eq!(type_word(&mut engine, "chanhs "), "chánh ");
        assert_eq!(type_word(&mut engine, "tar "), "tả ");
        assert_eq!(type_word(&mut engine, "khuyur "), "khuỷu ");
    }

    #[test]
    fn test_vni_neovim_ci_quote_and_preceding_chars() {
        let mut engine = Engine::new(EngineConfig {
            method: InputMethod::Vni,
            auto_restore_english: true,
            spell_check: false,
            enable_dictionary: true,
            ..EngineConfig::default()
        });

        // 1. Typing "biet61" inside "" followed by Neovim's `jk` -> <ESC> mapping
        // must NOT auto-restore "biết" back to "biet61"!
        assert_eq!(type_word(&mut engine, "biet61jk"), "biếtjk");
        engine.reset();

        // 2. Typing VNI words immediately after Vim Normal-mode commands (`i`, `a`, `cw`, `ciw`, `jk`)
        // or adjacent characters without a space delimiter (extract_last_syllable_index)
        assert_eq!(type_word(&mut engine, "ibiet61 "), "ibiết ");
        assert_eq!(type_word(&mut engine, "abiet61 "), "abiết ");
        assert_eq!(type_word(&mut engine, "cwbiet61 "), "cwbiết ");
        assert_eq!(type_word(&mut engine, "ciwbiet61 "), "ciwbiết ");
        assert_eq!(type_word(&mut engine, "jkbiet61 "), "jkbiết ");
        assert_eq!(type_word(&mut engine, "bvibiet61 "), "bvibiết ");
        assert_eq!(type_word(&mut engine, "ibie6t1 "), "ibiết ");
        assert_eq!(type_word(&mut engine, "ikhong61 "), "ikhống ");
        assert_eq!(type_word(&mut engine, "achinh1 "), "achính ");
        // 3. VNI Numeric Guard: sequential digits must remain literal digits (never mutate into tones)
        assert_eq!(type_word(&mut engine, "height=\"50\""), "height=\"50\"");
        engine.reset();
        assert_eq!(type_word(&mut engine, "501 "), "501 ");
        assert_eq!(type_word(&mut engine, "12345 "), "12345 ");
        assert_eq!(type_word(&mut engine, "width100 "), "width100 ");

        // 4. Idle Timeout (when configured > 0): pauses in typing (e.g. Normal mode command 'u', then 1s pause, then 'i' and '1')
        // must automatically flush old buffer so 'u' + 'i' + '1' does NOT produce "úi"!
        let mut sim_engine = Engine::new(EngineConfig {
            method: InputMethod::Vni,
            idle_timeout_ms: 800,
            ..EngineConfig::default()
        });
        // User presses 'u' in Normal mode at t = 0ms
        let out_u = sim_engine.process_key('u', 0.0);
        assert!(!out_u.consumed);
        // User pauses 1500ms to navigate, then presses 'i' to enter Insert mode at t = 1500ms
        let out_i = sim_engine.process_key('i', 1500.0);
        assert!(!out_i.consumed);
        // User pauses 1000ms, then types digit '1' at t = 2500ms
        let out_1 = sim_engine.process_key('1', 2500.0);
        // With idle timeout (>= 800ms), 'i' was flushed on pause, so '1' is committed cleanly as "1", not "í" or "úi"!
        assert_eq!(out_1.commit_text, "1");
        assert_eq!(out_1.backspaces, 0);
        assert!(!out_1.consumed);
    }

    #[test]
    fn test_tay_nguyen_and_rare_words_engine() {
        // Telex test with Spell Check and Auto Restore English enabled
        let mut telex_engine = Engine::new(EngineConfig {
            method: InputMethod::Telex,
            auto_restore_english: true,
            spell_check: true,
            enable_dictionary: true,
            ..EngineConfig::default()
        });

        assert_eq!(type_word(&mut telex_engine, "Ddawsk Lawsk "), "Đắk Lắk ");
        assert_eq!(type_word(&mut telex_engine, "Kroong Noo "), "Krông Nô ");
        assert_eq!(type_word(&mut telex_engine, "Buoon Ma Thuoojt "), "Buôn Ma Thuột ");
        assert_eq!(type_word(&mut telex_engine, "Pleiku "), "Pleiku ");
        assert_eq!(type_word(&mut telex_engine, "Ea H'leo "), "Ea H'leo ");
        assert_eq!(type_word(&mut telex_engine, "M'Ddrawsk "), "M'Đrắk ");
        assert_eq!(type_word(&mut telex_engine, "Ia Grai "), "Ia Grai ");
        assert_eq!(type_word(&mut telex_engine, "Chuw Proong "), "Chư Prông ");
        assert_eq!(type_word(&mut telex_engine, "Chuw Pawh "), "Chư Păh ");
        assert_eq!(type_word(&mut telex_engine, "khueechs "), "khuếch ");
        assert_eq!(type_word(&mut telex_engine, "nguaayf "), "nguầy ");
        assert_eq!(type_word(&mut telex_engine, "nguaayj "), "nguậy ");
        assert_eq!(type_word(&mut telex_engine, "dzaayj "), "dzậy ");
        assert_eq!(type_word(&mut telex_engine, "zij "), "zị ");

        // VNI test with Spell Check and Auto Restore English enabled
        let mut vni_engine = Engine::new(EngineConfig {
            method: InputMethod::Vni,
            auto_restore_english: true,
            spell_check: true,
            enable_dictionary: true,
            ..EngineConfig::default()
        });

        assert_eq!(type_word(&mut vni_engine, "D9a81k La81k "), "Đắk Lắk ");
        assert_eq!(type_word(&mut vni_engine, "Kro6ng No6 "), "Krông Nô ");
        assert_eq!(type_word(&mut vni_engine, "Buo6n Ma Thuo65t "), "Buôn Ma Thuột ");
        assert_eq!(type_word(&mut vni_engine, "Chu7 Pro6ng "), "Chư Prông ");
        assert_eq!(type_word(&mut vni_engine, "khue61ch "), "khuếch ");
    }

    #[test]
    fn test_flip_current_word() {
        let mut engine = Engine::new(EngineConfig {
            method: InputMethod::Telex,
            ..EngineConfig::default()
        });

        // Type 'mas' -> 'má'
        for ch in ['m', 'a', 's'] {
            engine.process_key(ch, 0.0);
        }
        assert_eq!(engine.current_word(), "má");
        assert_eq!(engine.raw_strokes(), "mas");

        // First flip: 'má' -> 'mas' (raw strokes)
        let out1 = engine.flip_current_word().unwrap();
        assert_eq!(engine.current_word(), "mas");
        assert_eq!(out1.backspaces, 1);
        assert_eq!(out1.commit_text, "as");

        // Second flip: 'mas' -> 'má' (re-transformed)
        let out2 = engine.flip_current_word().unwrap();
        assert_eq!(engine.current_word(), "má");
        assert_eq!(out2.backspaces, 2);
        assert_eq!(out2.commit_text, "á");
    }

    #[test]
    fn test_cycle_current_word_case() {
        let mut engine = Engine::new(EngineConfig {
            method: InputMethod::Telex,
            ..EngineConfig::default()
        });

        // Type "vieetj" -> "việt"
        for ch in ['v', 'i', 'e', 'e', 't', 'j'] {
            engine.process_key(ch, 0.0);
        }
        assert_eq!(engine.current_word(), "việt");

        // Cycle 1: "việt" -> "VIỆT"
        let out1 = engine.cycle_current_word_case().unwrap();
        assert_eq!(engine.current_word(), "VIỆT");
        assert_eq!(out1.commit_text, "VIỆT");

        // Cycle 2: "VIỆT" -> "Việt" (common prefix "V", backspaces = 3 for "IỆT", commit_text = "iệt")
        let out2 = engine.cycle_current_word_case().unwrap();
        assert_eq!(engine.current_word(), "Việt");
        assert_eq!(out2.backspaces, 3);
        assert_eq!(out2.commit_text, "iệt");

        // Cycle 3: "Việt" -> "việt" (no common prefix because 'V' != 'v', backspaces = 4, commit_text = "việt")
        let out3 = engine.cycle_current_word_case().unwrap();
        assert_eq!(engine.current_word(), "việt");
        assert_eq!(out3.backspaces, 4);
        assert_eq!(out3.commit_text, "việt");
    }

    #[test]
    fn test_macro_undo_on_backspace() {
        let mut engine = Engine::new(EngineConfig {
            method: InputMethod::Telex,
            enable_macro: true,
            ..EngineConfig::default()
        });

        // 'vn' is a default macro expanding to 'Việt Nam'
        for ch in ['v', 'n'] {
            engine.process_key(ch, 0.0);
        }
        assert_eq!(engine.current_word(), "vn");

        // Type space to trigger macro expansion
        let out = engine.process_key(' ', 0.0);
        assert!(out.macro_expanded);
        assert_eq!(out.commit_text, "Việt Nam ");

        // Step 1: First Backspace deletes only trailing space
        let undo1 = engine.undo_macro_expansion();
        assert!(undo1.is_some());
        let (bs1, commit1) = undo1.unwrap();
        assert_eq!(bs1, 1);
        assert_eq!(commit1, "");

        // Step 2: Second Backspace reverts "Việt Nam" back to original trigger "vn"
        let undo2 = engine.undo_macro_expansion();
        assert!(undo2.is_some());
        let (bs2, commit2) = undo2.unwrap();
        assert_eq!(bs2, 8); // "Việt Nam".chars().count()
        assert_eq!(commit2, "vn");
        assert_eq!(engine.current_word(), "vn");

        // Subsequent Backspace has no more macro to undo
        assert!(engine.undo_macro_expansion().is_none());

        // Pressing Space right after undoing does NOT re-trigger macro (prevents infinite loop!)
        let space_out = engine.process_key(' ', 0.0);
        assert!(!space_out.macro_expanded);
        assert_eq!(space_out.commit_text, " ");
        assert_eq!(engine.last_completed_word(), Some("vn "));

        // Future typing of 'vn ' in subsequent words will expand normally
        for ch in ['v', 'n'] {
            engine.process_key(ch, 0.0);
        }
        let out_future = engine.process_key(' ', 0.0);
        assert!(out_future.macro_expanded);
        assert_eq!(out_future.commit_text, "Việt Nam ");

        // Test: Typing another key cancels macro undo
        for ch in ['v', 'n', ' '] {
            engine.process_key(ch, 0.0);
        }
        engine.process_key('a', 0.0);
        assert!(engine.undo_macro_expansion().is_none());
    }

    #[test]
    fn test_smart_punctuation_snapping_after_macro() {
        let mut engine = Engine::new(EngineConfig {
            method: InputMethod::Telex,
            enable_macro: true,
            ..EngineConfig::default()
        });

        for ch in ['v', 'n'] {
            engine.process_key(ch, 0.0);
        }
        let out = engine.process_key(' ', 0.0);
        assert!(out.macro_expanded);
        assert_eq!(out.commit_text, "Việt Nam ");

        // Immediate comma snaps: eats trailing space and commits comma directly
        let snap_comma = engine.process_key(',', 0.0);
        assert_eq!(snap_comma.backspaces, 1);
        assert_eq!(snap_comma.commit_text, ",");

        // Test with exclamation mark
        for ch in ['v', 'n'] {
            engine.process_key(ch, 0.0);
        }
        let out2 = engine.process_key(' ', 0.0);
        assert!(out2.macro_expanded);

        let snap_excl = engine.process_key('!', 0.0);
        assert_eq!(snap_excl.backspaces, 1);
        assert_eq!(snap_excl.commit_text, "!");

        // Test: First Backspace deletes space, then typing comma commits comma normally
        for ch in ['v', 'n'] {
            engine.process_key(ch, 0.0);
        }
        let out3 = engine.process_key(' ', 0.0);
        assert!(out3.macro_expanded);

        let undo_space = engine.undo_macro_expansion();
        assert_eq!(undo_space, Some((1, String::new())));

        let comma_after_bs = engine.process_key(',', 0.0);
        assert_eq!(comma_after_bs.backspaces, 0);
        assert_eq!(comma_after_bs.commit_text, ",");
    }

    #[test]
    fn test_inline_symbol_and_emoji_expansion() {
        let mut engine = Engine::new(EngineConfig {
            habit_promote_threshold: 4,
            ..EngineConfig::default()
        });

        // 1. Emoji expansion with closing colon: ":smile:" -> "😄"
        let res1 = type_word(&mut engine, ":smile:");
        assert_eq!(res1, "😄");

        // 2. Math symbol expansion with closing colon: ":alpha:" -> "α"
        engine.reset();
        let res2 = type_word(&mut engine, ":alpha:");
        assert_eq!(res2, "α");

        // 3. Math symbol expansion with space delimiter: ":sum " -> "∑ "
        engine.reset();
        let res3 = type_word(&mut engine, ":sum ");
        assert_eq!(res3, "∑ ");

        // 4. Arrow expansion: ":arrow:" -> "➔"
        engine.reset();
        let res4 = type_word(&mut engine, ":arrow:");
        assert_eq!(res4, "➔");

        // 5. Check mark expansion: ":check:" -> "✓"
        engine.reset();
        let res5 = type_word(&mut engine, ":check:");
        assert_eq!(res5, "✓");

        // 6. Two-step undo on Backspace: ":smile:" expanded to "😄", Backspace restores ":smile:"
        engine.reset();
        type_word(&mut engine, ":smile:");
        let undo = engine.undo_macro_expansion();
        assert!(undo.is_some());
        let (bs, restore) = undo.unwrap();
        // Since delimiter was ':', first backspace deletes the single expansion char and restores shortcode
        assert_eq!(bs, 1);
        assert_eq!(restore, ":smile:");

        // 7. Safety against collision: 2-letter keywords do NOT expand on Space (e.g. ":in ", ":pi ")
        engine.reset();
        let res_in = type_word(&mut engine, ":in ");
        assert_eq!(res_in, ":in ");

        let res_pi = type_word(&mut engine, ":pi ");
        assert_eq!(res_pi, ":pi ");

        // But 2-letter keywords expand with closing colon: ":in:" -> "∈", ":pi:" -> "π"
        engine.reset();
        let res_in_colon = type_word(&mut engine, ":in:");
        assert_eq!(res_in_colon, "∈");

        engine.reset();
        let res_pi_colon = type_word(&mut engine, ":pi:");
        assert_eq!(res_pi_colon, "π");

        // 8. Safety against numbers / time collision: ":30 " does NOT expand
        engine.reset();
        let res_time = type_word(&mut engine, ":30 ");
        assert_eq!(res_time, ":30 ");
    }

    #[test]
    fn test_reopen_previous_word_on_backspace() {
        let mut engine = Engine::new(EngineConfig {
            method: InputMethod::Telex,
            ..EngineConfig::default()
        });

        // Test 1: User types 'tieeng' (tiêng) + space -> "tiêng "
        for ch in ['t', 'i', 'e', 'e', 'n', 'g', ' '] {
            engine.process_key(ch, 0.0);
        }
        assert_eq!(engine.current_word(), "");

        // User realizes they forgot tone and presses Backspace immediately!
        let reopened = engine.handle_backspace();
        assert!(reopened);
        assert_eq!(engine.current_word(), "tiêng");
        assert_eq!(engine.raw_strokes(), "tieeng");

        // Now user types 's' -> dynamically becomes 'tiếng'!
        let out = engine.process_key('s', 0.0);
        assert!(out.consumed);
        assert_eq!(engine.current_word(), "tiếng");
        assert_eq!(out.commit_text, "ếng");

        // Finish the word with space
        engine.process_key(' ', 0.0);

        // Test 2: User types 'xin' + space -> "xin "
        for ch in ['x', 'i', 'n', ' '] {
            engine.process_key(ch, 0.0);
        }
        let reopened2 = engine.handle_backspace();
        assert!(reopened2);
        assert_eq!(engine.current_word(), "xin");

        // Types 'f' -> becomes 'xìn'!
        let out2 = engine.process_key('f', 0.0);
        assert!(out2.consumed);
        assert_eq!(engine.current_word(), "xìn");
        assert_eq!(out2.commit_text, "ìn");
    }

    #[test]
    fn test_undo_to_raw_escape() {
        let mut engine = Engine::new(EngineConfig {
            method: InputMethod::Telex,
            ..EngineConfig::default()
        });

        // User types 'tieengs' -> becomes 'tiếng'
        for ch in ['t', 'i', 'e', 'e', 'n', 'g', 's'] {
            engine.process_key(ch, 0.0);
        }
        assert_eq!(engine.current_word(), "tiếng");

        // User hits Escape to undo to raw keys (e.g. they wanted code/test string 'tieengs')
        let out = engine.undo_to_raw().unwrap();
        assert_eq!(engine.current_word(), "tieengs");
        assert_eq!(engine.raw_strokes(), "tieengs");
        // Replaces "ếng" with "eengs" (3 backspaces)
        assert_eq!(out.backspaces, 3);
        assert_eq!(out.commit_text, "eengs");
    }

    #[test]
    fn test_developer_camel_case_detection() {
        let mut engine = Engine::new(EngineConfig {
            method: InputMethod::Telex,
            ..EngineConfig::default()
        });

        // User types 'parse' -> 'parse'
        for ch in ['p', 'a', 'r', 's', 'e'] {
            engine.process_key(ch, 0.0);
        }

        // Now user types uppercase 'S' (camelCase transition: parse + S)
        let out = engine.process_key('S', 0.0);
        assert!(out.consumed);
        assert_eq!(engine.current_word(), "parseS");

        // Subsequent letters like 'y', 'l', 'l', 'a', 'b', 'l', 'e' pass untouched (no accidental Vietnamese marks!)
        for ch in ['y', 'l', 'l', 'a', 'b', 'l', 'e'] {
            let out_char = engine.process_key(ch, 0.0);
            assert!(!out_char.consumed);
        }
        assert_eq!(engine.current_word(), "parseSyllable");
    }

    #[test]
    fn test_word_and_sentence_rescue() {
        let engine = Engine::new(EngineConfig {
            method: InputMethod::Telex,
            ..EngineConfig::default()
        });

        // Test single words
        assert_eq!(engine.rescue_word("tieengs"), "tiếng");
        assert_eq!(engine.rescue_word("vieetj"), "việt");
        assert_eq!(engine.rescue_word("nef"), "nè");
        assert_eq!(engine.rescue_word("English"), "English");

        // Test sentence rescue
        let raw = "Chaof banj, tieengs vieetj nef!";
        let rescued = engine.rescue_sentence(raw);
        assert_eq!(rescued, Some("Chào bạn, tiếng việt nè!".to_string()));

        // Test English sentence with keywords (not transformed)
        let english = "server linux browser window";
        assert_eq!(engine.rescue_sentence(english), None);
    }

    #[test]
    fn test_apply_accent_to_selection() {
        let engine = Engine::new(EngineConfig {
            method: InputMethod::Telex,
            ..EngineConfig::default()
        });

        // 'nhanh' + 's' -> 'nhánh'
        assert_eq!(engine.apply_accent_to_selection("nhanh", 's'), Some("nhánh".to_string()));

        // 'nhánh' + 'f' -> 'nhành'
        assert_eq!(engine.apply_accent_to_selection("nhánh", 'f'), Some("nhành".to_string()));

        // 'nhánh' + 'z' -> 'nhanh' (remove tone)
        assert_eq!(engine.apply_accent_to_selection("nhánh", 'z'), Some("nhanh".to_string()));

        // Multi-word selection: 'chạy thật nhanh' + 's' -> 'chạy thật nhánh'
        assert_eq!(
            engine.apply_accent_to_selection("chạy thật nhanh", 's'),
            Some("chạy thật nhánh".to_string())
        );

        // With trailing space: 'nhanh ' + 's' -> 'nhánh '
        assert_eq!(engine.apply_accent_to_selection("nhanh ", 's'), Some("nhánh ".to_string()));

        // Non-modifier key should return None
        assert_eq!(engine.apply_accent_to_selection("nhanh", 'k'), None);

        // Auto mode with VNI digit
        let auto_engine = Engine::new(EngineConfig {
            method: InputMethod::Auto,
            ..EngineConfig::default()
        });
        assert_eq!(auto_engine.apply_accent_to_selection("nhanh", '1'), Some("nhánh".to_string()));
        assert_eq!(auto_engine.apply_accent_to_selection("nhanh", '2'), Some("nhành".to_string()));
    }

    #[test]
    fn test_developer_underscore_identifiers() {
        let mut engine = Engine::new(EngineConfig {
            method: InputMethod::Telex,
            ..EngineConfig::default()
        });

        // Typing _variable: leading underscore locks English mode
        engine.process_key('_', 0.0);
        for ch in ['v', 'a', 'r', 'i', 'a', 'b', 'l', 'e'] {
            let out = engine.process_key(ch, 0.0);
            assert!(!out.consumed);
        }
        assert_eq!(engine.current_word(), "_variable");

        // Typing space resets buffer
        engine.process_key(' ', 0.0);

        // Typing snake_case: user_id
        for ch in ['u', 's', 'e', 'r'] {
            engine.process_key(ch, 0.0);
        }
        let _out_under = engine.process_key('_', 0.0);
        assert_eq!(engine.current_word(), "user_");
        for ch in ['i', 'd'] {
            let out = engine.process_key(ch, 0.0);
            assert!(!out.consumed);
        }
        assert_eq!(engine.current_word(), "user_id");
    }

    #[test]
    fn test_developer_chained_dot_accessors() {
        let mut engine = Engine::new(EngineConfig {
            method: InputMethod::Telex,
            ..EngineConfig::default()
        });

        // Type 'db.'
        for ch in ['d', 'b', '.'] {
            engine.process_key(ch, 0.0);
        }

        // Immediately type 'collection' without space: automatically locked as English!
        for ch in ['c', 'o', 'l', 'l', 'e', 'c', 't', 'i', 'o', 'n'] {
            let out = engine.process_key(ch, 0.0);
            assert!(!out.consumed);
        }
        assert_eq!(engine.current_word(), "collection");
    }

    #[test]
    fn test_developer_technical_tokens_hex_and_css_units() {
        let mut engine = Engine::new(EngineConfig {
            method: InputMethod::Telex,
            ..EngineConfig::default()
        });

        // 1. CSS Hex color: #f97316 (f and s must NOT transform into Vietnamese diacritics!)
        assert_eq!(type_word(&mut engine, "#f97316 "), "#f97316 ");
        engine.reset();
        assert_eq!(type_word(&mut engine, "#ef4444 "), "#ef4444 ");
        engine.reset();
        assert_eq!(type_word(&mut engine, "#root "), "#root ");

        // 2. CSS Units: 10px, 2rem, 50vh, 100vw
        engine.reset();
        assert_eq!(type_word(&mut engine, "10px "), "10px ");
        engine.reset();
        assert_eq!(type_word(&mut engine, "2rem "), "2rem ");
        engine.reset();
        assert_eq!(type_word(&mut engine, "50vh "), "50vh ");
        engine.reset();
        assert_eq!(type_word(&mut engine, "100vw "), "100vw ");

        // 3. Technical tokens with digits and letters: v1s, sha256, md5, utf8
        engine.reset();
        assert_eq!(type_word(&mut engine, "v1s "), "v1s ");
        engine.reset();
        assert_eq!(type_word(&mut engine, "sha256 "), "sha256 ");
        engine.reset();
        assert_eq!(type_word(&mut engine, "md5 "), "md5 ");
        engine.reset();
        assert_eq!(type_word(&mut engine, "utf8 "), "utf8 ");

        // 4. Normal Vietnamese typing works immediately after space
        engine.reset();
        assert_eq!(type_word(&mut engine, "10px tieengs vieetj "), "10px tiếng việt ");
    }

    #[test]
    fn test_composite_unicode_charset() {
        assert_eq!(to_composite_unicode("tiếng"), "tie\u{0302}\u{0301}ng");
        assert_eq!(to_composite_unicode("Việt Nam"), "Vie\u{0302}\u{0323}t Nam");

        let mut engine = Engine::new(EngineConfig {
            method: InputMethod::Telex,
            charset: Charset::UnicodeComposite,
            ..EngineConfig::default()
        });

        for ch in ['t', 'i', 'e', 'e', 'n', 'g'] {
            engine.process_key(ch, 0.0);
        }
        let out_s = engine.process_key('s', 0.0);
        // The output commit on 's' must be in Unicode Tổ Hợp (NFD)
        assert!(out_s.commit_text.contains('\u{0302}') || out_s.commit_text.contains('\u{0301}'));
    }

    #[test]
    fn test_vni_dau61_and_common_words() {
        let mut engine = Engine::new(EngineConfig {
            method: InputMethod::Vni,
            ..EngineConfig::default()
        });

        let mut screen = String::new();
        for ch in "dau61".chars() {
            let out = engine.process_key(ch, 0.0);
            if out.consumed {
                let keep = screen.chars().count().saturating_sub(out.backspaces);
                screen = screen.chars().take(keep).collect();
                screen.push_str(&out.commit_text);
            } else {
                screen.push(ch);
            }
        }
        assert_eq!(screen, "dấu");
        assert_eq!(engine.current_word(), "dấu");
    }

    #[test]
    fn test_adaptive_habit_learner_flip_and_backspace_retype() {
        let mut engine = Engine::new(EngineConfig {
            method: InputMethod::Telex,
            spell_check: false, // disable built-in dictionary spell check to test pure habit learning
            auto_restore_english: false,
            habit_learning_enabled: true, // explicitly enable for this test
            habit_promote_threshold: 4, // Higher threshold (4 times) to prevent accidental whitelisting
            ..EngineConfig::default()
        });

        // Suppose user types a custom foreign name/term "kas" which Telex turns into "ká"
        // Repeat corrections 3 times: should NOT yet be promoted to Whitelist!
        for i in 1..=3 {
            for ch in "kas".chars() {
                engine.process_key(ch, 100.0 * i as f64);
            }
            assert_eq!(engine.current_word(), "ká");
            let _ = engine.flip_current_word().unwrap();
            engine.process_key(' ', 100.0 * i as f64 + 50.0);
            assert_eq!(engine.habit_learner().correction_count("kas"), i);
            assert!(!engine.habit_learner().is_learned_word("kas"), "Should not learn after {} times (threshold 4)", i);
        }

        // 4th time: user corrects "kas" again -> now reaches threshold 4 and is promoted!
        for ch in "kas".chars() {
            engine.process_key(ch, 400.0);
        }
        assert_eq!(engine.current_word(), "ká");
        let _ = engine.flip_current_word().unwrap();
        engine.process_key(' ', 450.0);
        assert_eq!(engine.habit_learner().correction_count("kas"), 4);
        assert!(engine.habit_learner().is_learned_word("kas"));

        // 5th time: user types "kas " -> passes through as "kas " automatically without needing undo!
        let result = type_word(&mut engine, "kas ");
        assert_eq!(result, "kas ");

        // Test unlearn_word: if user unlearns "kas", it can be typed with diacritics again!
        engine.unlearn_word("kas");
        assert!(!engine.habit_learner().is_learned_word("kas"));
        assert_eq!(type_word(&mut engine, "kas "), "ká ");
    }

    #[test]
    fn test_initial_case_anchor_toggle() {
        let mut engine = Engine::new(EngineConfig {
            method: InputMethod::Telex,
            initial_case_anchor: false, // Default is OFF
            ..EngineConfig::default()
        });

        // When OFF: typing "dduOWCJ " preserves mixed case ("đượC ")
        let out_off = type_word(&mut engine, "dduOWCJ ");
        assert_eq!(out_off, "đượC ");

        // Turn ON: typing "dduOWCJ " harmonizes to lowercase based on initial 'đ' ("được ")
        engine.set_initial_case_anchor(true);
        assert!(engine.initial_case_anchor());
        let out_on = type_word(&mut engine, "dduOWCJ ");
        assert_eq!(out_on, "được ");
    }

    #[test]
    fn test_intermediate_two_stroke_accent_and_space_restore() {
        // Test VNI: intermediate state 'duong2' -> 'duòng', 'duong27' -> 'dường', 'duong2 ' -> 'duong2 '
        let mut vni_engine = Engine::new(EngineConfig {
            method: InputMethod::Vni,
            auto_restore_english: false, // Must restore even when auto_restore_english is false!
            ..EngineConfig::default()
        });

        // 1. In-progress typing shows intermediate state
        let out_duong2 = type_word(&mut vni_engine, "duong2");
        assert_eq!(out_duong2, "duòng");

        // 2. Typing second accent resolves to proper Vietnamese word
        vni_engine.reset();
        let out_dường = type_word(&mut vni_engine, "duong27 ");
        assert_eq!(out_dường, "dường ");

        // 3. Typing Space while in intermediate state restores to raw text
        vni_engine.reset();
        let out_space = type_word(&mut vni_engine, "duong2 ");
        assert_eq!(out_space, "duong2 ");

        vni_engine.reset();
        let out_nuoc1 = type_word(&mut vni_engine, "nuoc1 ");
        assert_eq!(out_nuoc1, "nuoc1 ");

        vni_engine.reset();
        let out_tien1 = type_word(&mut vni_engine, "tien1 ");
        assert_eq!(out_tien1, "tien1 ");

        // 4. Genuine complete words with numbers (gio72 -> giờ, duoc975 -> được) MUST NOT restore
        vni_engine.reset();
        let out_gio = type_word(&mut vni_engine, "gio72 ");
        assert_eq!(out_gio, "giờ ");

        vni_engine.reset();
        let out_duoc = type_word(&mut vni_engine, "duoc975 ");
        assert_eq!(out_duoc, "được ");

        // Test Telex: intermediate state 'duongf' -> 'duòng', 'duongfw' -> 'dường', 'duongf ' -> 'duongf '
        let mut telex_engine = Engine::new(EngineConfig {
            method: InputMethod::Telex,
            auto_restore_english: false,
            ..EngineConfig::default()
        });

        let out_tf = type_word(&mut telex_engine, "duongf");
        assert_eq!(out_tf, "duòng");

        telex_engine.reset();
        let out_tfw = type_word(&mut telex_engine, "duongfw ");
        assert_eq!(out_tfw, "dường ");

        telex_engine.reset();
        let out_tf_space = type_word(&mut telex_engine, "duongf ");
        assert_eq!(out_tf_space, "duongf ");

        telex_engine.reset();
        let out_ts_space = type_word(&mut telex_engine, "tiens ");
        assert_eq!(out_ts_space, "tiens ");

        // Intermediate forms with 'uye' (Huyèn, chuyén) and Space restore
        vni_engine.reset();
        assert_eq!(type_word(&mut vni_engine, "Huyen2"), "Huyèn");
        vni_engine.reset();
        assert_eq!(type_word(&mut vni_engine, "Huyen26 "), "Huyền ");
        vni_engine.reset();
        assert_eq!(type_word(&mut vni_engine, "Huyen2 "), "Huyen2 ");

        telex_engine.reset();
        assert_eq!(type_word(&mut telex_engine, "Huyenf"), "Huyèn");
        telex_engine.reset();
        assert_eq!(type_word(&mut telex_engine, "Huyenfe "), "Huyền ");
        telex_engine.reset();
        assert_eq!(type_word(&mut telex_engine, "Huyenf "), "Huyenf ");

        telex_engine.reset();
        let out_tse = type_word(&mut telex_engine, "tiense ");
        assert_eq!(out_tse, "tiến ");
    }

    #[test]
    fn test_batch1_vietlotus_edge_cases_and_guards() {
        let mut engine = Engine::new(EngineConfig {
            method: InputMethod::Telex,
            auto_restore_english: true,
            spell_check: true,
            enable_dictionary: true,
            enable_macro: true,
            ..EngineConfig::default()
        });

        // 1. Vowel elongation after horned syllables ('ư'/'ơ') & toned syllables:
        // 'uwfaaa ' -> 'ừaaa ', 'owfaaa ' -> 'ờaaa ', 'muwaaaa ' -> 'mưaaaa ', 'uwfeee ' -> 'ừeee ', 'trowifiii ' -> 'trờiiii '
        assert_eq!(type_word(&mut engine, "uwfaaa "), "ừaaa ");
        assert_eq!(type_word(&mut engine, "owfaaa "), "ờaaa ");
        assert_eq!(type_word(&mut engine, "muwaaaa "), "mưaaaa ");
        assert_eq!(type_word(&mut engine, "uwfeee "), "ừeee ");
        assert_eq!(type_word(&mut engine, "trowifiii "), "trờiiii ");
        // While keeping English words with 'w' + 'e' intact:
        assert_eq!(type_word(&mut engine, "owe "), "owe ");
        assert_eq!(type_word(&mut engine, "twee "), "twee ");
        assert_eq!(type_word(&mut engine, "weee "), "weee ");

        // 2. Completed horn/breve syllable saturation on trailing 'w' and subsequent Backspace:
        // 'dduowcjw' -> 'đượcw', then Backspace -> 'được'
        engine.reset();
        let out_duocw = type_word(&mut engine, "dduowcjw");
        assert_eq!(out_duocw, "đượcw");
        engine.handle_backspace();
        assert_eq!(engine.current_word(), "được");
        let out_space = engine.process_key(' ', 0.0);
        assert!(!out_space.auto_restored);

        // And English words with two 'w's restore both 'w's cleanly:
        engine.reset();
        assert_eq!(type_word(&mut engine, "awkward "), "awkward ");
        assert_eq!(type_word(&mut engine, "downward "), "downward ");

        // 3. Restore non-Vietnamese words before digits when Macro & Auto-Restore are both enabled:
        // 'test123 ' -> 'test123 ', 'user2026 ' -> 'user2026 ', while keeping numeric macros ('k8s', 'mp3')
        engine.reset();
        engine.macro_table.insert("k8s".to_string(), "Kubernetes".to_string());
        engine.macro_table.insert("mp3".to_string(), "MPEG Layer 3".to_string());
        assert_eq!(type_word(&mut engine, "test123 "), "test123 ");
        assert_eq!(type_word(&mut engine, "user2026 "), "user2026 ");
        assert_eq!(type_word(&mut engine, "k8s "), "Kubernetes ");
        assert_eq!(type_word(&mut engine, "mp3 "), "MPEG Layer 3 ");

        // 4. Technical delimiter macro suppression (URLs, emails, paths, function calls):
        // Note: 'vn' -> 'Việt Nam' and 'dc' -> 'được' are default macros!
        engine.reset();
        assert_eq!(type_word(&mut engine, "google.com.vn "), "google.com.vn ");
        assert_eq!(type_word(&mut engine, "https://vn/dc "), "https://vn/dc ");
        assert_eq!(type_word(&mut engine, "user@vn.com "), "user@vn.com ");
        assert_eq!(type_word(&mut engine, "dc() "), "dc() ");
        assert_eq!(type_word(&mut engine, "vn[0] "), "vn[0] ");
        // Normal prose still expands macros as expected:
        assert_eq!(type_word(&mut engine, "vn "), "Việt Nam ");
        assert_eq!(type_word(&mut engine, "dc "), "được ");

        // 5. Leading slash command & Unix path protection ('/compact', '/aws', '/os', '/usr/bin'):
        engine.reset();
        assert_eq!(type_word(&mut engine, "/compact "), "/compact ");
        assert_eq!(type_word(&mut engine, "/aws "), "/aws ");
        assert_eq!(type_word(&mut engine, "/os "), "/os ");
        assert_eq!(type_word(&mut engine, "/if "), "/if ");
        assert_eq!(type_word(&mut engine, "/as "), "/as ");
        assert_eq!(type_word(&mut engine, "/usr/local/bin "), "/usr/local/bin ");

        // 6. Extra edge-cases from VieLotus test_suite.csv:
        // 'os.path ', 'cocoa ', 'momo ', 'mono ', 'qusy ' -> 'quý '
        engine.reset();
        assert_eq!(type_word(&mut engine, "os.path "), "os.path ");
        assert_eq!(type_word(&mut engine, "cocoa "), "cocoa ");
        assert_eq!(type_word(&mut engine, "momo "), "momo ");
        assert_eq!(type_word(&mut engine, "mono "), "mono ");
        assert_eq!(type_word(&mut engine, "qusy "), "quý ");
    }

    #[test]
    fn test_batch2_developer_mode_backtick_and_sentence_momentum() {
        let mut engine = Engine::new(EngineConfig {
            method: InputMethod::Telex,
            auto_restore_english: true,
            spell_check: true,
            enable_dictionary: true,
            enable_macro: true,
            developer_mode: true,
            sentence_momentum: true,
            math_context: true,
            ..EngineConfig::default()
        });

        // 1. Markdown Inline Code (`...`) and Fenced Code Block (```...```) FSM:
        // Inside inline backticks, Vietnamese diacritics and macro expansions are bypassed:
        assert_eq!(
            type_word(&mut engine, "Dufng leenhj `git status --short` vaf `dc vn` trong terminal "),
            "Dùng lệnh `git status --short` và `dc vn` trong terminal "
        );
        // Unclosed inline backtick automatically closes on newline '\n':
        engine.reset();
        assert_eq!(
            type_word(&mut engine, "`code_line\ntieengs Vieetj "),
            "`code_line\ntiếng Việt "
        );
        // Fenced code block (```...```) stays bypassed across newlines until closing ```:
        engine.reset();
        assert_eq!(
            type_word(
                &mut engine,
                "```rust\nlet dduowcj = vn();\nlet tieengs = 1;\n```\nTieengs Vieetj "
            ),
            "```rust\nlet dduowcj = vn();\nlet tieengs = 1;\n```\nTiếng Việt "
        );

        // 2. Bilingual Sentence Momentum & Math Function Context:
        // Standalone Vietnamese collisions remain Vietnamese when typed alone:
        engine.reset();
        assert_eq!(type_word(&mut engine, "tree "), "trê ");
        engine.reset();
        assert_eq!(type_word(&mut engine, "cos "), "có ");
        engine.reset();
        assert_eq!(type_word(&mut engine, "must "), "mút ");

        // Preceded by English anchor / English momentum -> ambiguous collisions resolve to English:
        engine.reset();
        assert_eq!(type_word(&mut engine, "the tree "), "the tree ");
        engine.reset();
        assert_eq!(type_word(&mut engine, "that is a tree "), "that is a tree ");
        engine.reset();
        assert_eq!(type_word(&mut engine, "he must see this room. "), "he must see this room. ");

        // Sentence boundary '.' resets English momentum so the next word is Vietnamese again:
        assert_eq!(type_word(&mut engine, "tree "), "trê ");

        // Vietnamese context reset word ('anh', 'em', 'tôi', 'không') resets momentum back to Vietnamese:
        engine.reset();
        assert_eq!(type_word(&mut engine, "the anh cos "), "the anh có ");

        // Math function context ('sin and cos ', 'tan or cos '):
        engine.reset();
        assert_eq!(type_word(&mut engine, "sin and cos "), "sin and cos ");
        engine.reset();
        assert_eq!(type_word(&mut engine, "tan or cos "), "tan or cos ");
    }

    #[test]
    fn test_vni_telex_parity_table_and_fuzz() {
        let parity_cases: &[(&str, &str, &str)] = &[
            ("tieengs ", "tie6ng1 ", "tiếng "),
            ("duowcj ", "duo7c5 ", "dược "),
            ("dduowcj ", "d9uo7c5 ", "được "),
            ("nguowif ", "nguo7i2 ", "người "),
            ("thuowr ", "thuo73 ", "thuở "),
            ("khuowng ", "khuo7ng ", "khương "),
            ("quow ", "quo7 ", "quơ "),
            ("giuwax ", "giu7a4 ", "giữa "),
            ("hoaf ", "hoa2 ", "hòa "),
            ("homx ", "hom4 ", "hõm "),
            ("truwowngf ", "truo7ng2 ", "trường "),
            ("nghieeng ", "nghie6ng ", "nghiêng "),
            ("khuyeens ", "khuye6n1 ", "khuyến "),
            ("aans ", "a6n1 ", "ấn "),
            ("mawns ", "ma8n1 ", "mắn "),
            ("oors ", "o61 ", "ố "),
            ("oows ", "o71 ", "ớ "),
            ("nuowcs ", "nuo7c1 ", "nước "),
            // TitleCase & ALL-CAPS parity:
            ("Tieengs ", "Tie6ng1 ", "Tiếng "),
            ("TIEENGS ", "TIE6NG1 ", "TIẾNG "),
            ("Dduowcj ", "D9uo7c5 ", "Được "),
            ("DDUOWCJ ", "D9UO7C5 ", "ĐƯỢC "),
            ("Nguowif ", "Nguo7i2 ", "Người "),
            ("NGUOWIF ", "NGUO7I2 ", "NGƯỜI "),
            ("Nuowcs ", "Nuo7c1 ", "Nước "),
            ("NUOWCS ", "NUO7C1 ", "NƯỚC "),
        ];

        for &(telex_in, vni_in, expected) in parity_cases {
            let mut telex_engine = Engine::new(EngineConfig {
                method: InputMethod::Telex,
                ..EngineConfig::default()
            });
            let mut vni_engine = Engine::new(EngineConfig {
                method: InputMethod::Vni,
                ..EngineConfig::default()
            });
            let t_out = type_word(&mut telex_engine, telex_in);
            let v_out = type_word(&mut vni_engine, vni_in);
            assert_eq!(t_out, expected, "Telex failed for {:?}", telex_in);
            assert_eq!(v_out, expected, "VNI failed for {:?}", vni_in);
        }

        // Deterministic fuzzing over random ASCII/modifier sequences: must never panic
        let mut seed: u64 = 0x5EED_CAFE_BABE_1234;
        let telex_chars = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ ";
        let vni_chars = b"abcdefghijklmnopqrstuvwxyz0123456789 ";
        for method in [InputMethod::Telex, InputMethod::Vni, InputMethod::Auto] {
            let mut engine = Engine::new(EngineConfig {
                method,
                ..EngineConfig::default()
            });
            let alphabet = if method == InputMethod::Vni {
                vni_chars.as_slice()
            } else {
                telex_chars.as_slice()
            };
            for _ in 0..500 {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                let ch = alphabet[(seed as usize) % alphabet.len()] as char;
                let _ = engine.process_key(ch, 0.0);
            }
        }
    }

    #[test]
    fn test_smart_fuzzy_vowel_nguoi2() {
        let mut engine = Engine::new(EngineConfig {
            method: InputMethod::Vni,
            smart_fuzzy_vowel: true,
            ..EngineConfig::default()
        });
        assert_eq!(type_word(&mut engine, "nguoi2 "), "người ");
    }

    #[test]
    fn test_smart_fuzzy_vowel_uye() {
        let mut engine = Engine::new(EngineConfig {
            method: InputMethod::Vni,
            smart_fuzzy_vowel: true,
            ..EngineConfig::default()
        });
        assert_eq!(type_word(&mut engine, "khuyen1 "), "khuyến ");
        assert_eq!(type_word(&mut engine, "huyen2 "), "huyền ");
        assert_eq!(type_word(&mut engine, "tuyet5 "), "tuyệt ");
    }

    #[test]
    fn test_sentence_momentum_math_context_optional() {
        // Default OFF: d9a4 -> đã (no trap)
        let mut engine_off = Engine::new(EngineConfig {
            method: InputMethod::Vni,
            ..EngineConfig::default()
        });
        assert_eq!(type_word(&mut engine_off, "d9a4 "), "đã ");

        // Default OFF: bilingual collisions resolve to Vietnamese
        let mut engine_off2 = Engine::new(EngineConfig {
            method: InputMethod::Telex,
            ..EngineConfig::default()
        });
        assert_eq!(type_word(&mut engine_off2, "the tree "), "the trê ");
        assert_eq!(type_word(&mut engine_off2, "sin and cos "), "sin and có ");

        // ON: preserve English collision after anchor
        let mut engine_on = Engine::new(EngineConfig {
            method: InputMethod::Telex,
            sentence_momentum: true,
            ..EngineConfig::default()
        });
        assert_eq!(type_word(&mut engine_on, "the tree "), "the tree ");

        // Math context ON
        let mut engine_math = Engine::new(EngineConfig {
            method: InputMethod::Telex,
            math_context: true,
            ..EngineConfig::default()
        });
        assert_eq!(type_word(&mut engine_math, "sin and cos "), "sin and cos ");
    }

    #[test]
    fn test_auto_space_after_punctuation_default_off() {
        // Default OFF: không chèn space tự động
        let mut engine = Engine::new(EngineConfig {
            method: InputMethod::Vni,
            ..EngineConfig::default()
        });
        // Gõ "chào." không nên có space tự động
        assert_eq!(type_word(&mut engine, "chào."), "chào.");
        // Gõ "chào ?" cũng không
        assert_eq!(type_word(&mut engine, "chào ?"), "chào ?");
    }

    #[test]
    fn test_auto_space_after_punctuation_enabled() {
        let mut engine = Engine::new(EngineConfig {
            method: InputMethod::Vni,
            auto_space_after_punctuation: true,
            ..EngineConfig::default()
        });
        // Gõ "chào." -> tự chèn space (chấm sau là chữ cái trước đó)
        assert_eq!(type_word(&mut engine, "chào."), "chào. ");
        // Gõ "chào?" -> tự chèn space
        assert_eq!(type_word(&mut engine, "chào?"), "chào? ");
        // Gõ "chào!" -> tự chèn space
        assert_eq!(type_word(&mut engine, "chào!"), "chào! ");
        // Gõ "chào ," (dấu phẩy không trong list) không nên chèn space
        assert_eq!(type_word(&mut engine, "chào ,"), "chào ,");
        // Gõ "1.5" (số chấm số) không nên chèn space (prev_char là số)
        assert_eq!(type_word(&mut engine, "1.5 "), "1.5 ");
    }
