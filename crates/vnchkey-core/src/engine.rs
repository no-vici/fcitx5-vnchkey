//! Syllable State Machine and Input Orchestrator.

use crate::auto_restore::EnglishAutoRestore;
use crate::casing::CaseTracker;
use crate::macros::MacroTable;
pub use crate::telex::W2UType;
use crate::telex::{has_vietnamese_diacritic, is_valid_vietnamese_syllable, TelexEngine};
use crate::vni::VniEngine;

/// Supported Vietnamese typing methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMethod {
    /// Auto adaptive mode (seamlessly supports both Telex and VNI like lilypad)
    Auto,
    /// Telex typing method (aa->â, aw->ă, s->sắc, f->huyền, etc.)
    Telex,
    /// VNI typing method (a6->â, a8->ă, a1->á, a2->à, etc.)
    Vni,
    /// Simple Telex variant
    SimpleTelex,
}

/// Tone placement standard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToneStyle {
    /// Modern standard: đặt dấu trên nguyên âm chính (hoà, uỷ, hoè)
    Modern,
    /// Classic standard: đặt dấu kiểu cũ (hòa, ủy, hòe)
    Classic,
}

/// Engine configuration options.
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Selected input method (Auto, Telex, or VNI)
    pub method: InputMethod,
    /// Preferred tone placement convention
    pub tone_style: ToneStyle,
    /// Automatically restore English keywords and non-Vietnamese words (Restore English)
    pub auto_restore_english: bool,
    /// Preserve exact capitalization pattern
    pub smart_casing: bool,
    /// Quick undo when pressing Backspace or Escape
    pub quick_undo: bool,
    /// Enable Sentinel Barrier N+1 protocol
    pub sentinel_enabled: bool,
    /// Type w to produce ư rule
    pub type_w_to_u: W2UType,
    /// Type [ -> ơ, ] -> ư, { -> Ơ, } -> Ư
    pub bracket_transform: bool,
    /// Enable macro expansions
    pub enable_macro: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            method: InputMethod::Auto,
            tone_style: ToneStyle::Modern,
            auto_restore_english: true,
            smart_casing: true,
            quick_undo: true,
            sentinel_enabled: true,
            type_w_to_u: W2UType::NonStart,
            bracket_transform: false,
            enable_macro: true,
        }
    }
}

/// Output returned after processing a single keystroke.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineOutput {
    /// Number of backspaces to emit to replace only the modified suffix
    pub backspaces: usize,
    /// Replacement UTF-8 suffix text to commit
    pub commit_text: String,
    /// Whether this stroke was consumed by the Vietnamese engine
    pub consumed: bool,
    /// Whether Restore English restored raw English on this word
    pub auto_restored: bool,
    /// Whether a macro expansion was applied
    pub macro_expanded: bool,
    /// Calculated delay in milliseconds
    pub adaptive_delay_ms: u64,
}

/// Main Vietnamese Input Engine holding working state.
#[derive(Debug)]
pub struct Engine {
    config: EngineConfig,
    telex: TelexEngine,
    vni: VniEngine,
    auto_restore: EnglishAutoRestore,
    casing: CaseTracker,
    macro_table: MacroTable,
    current_word: String,
    raw_strokes: String,
    word_used_telex: bool,
    word_used_vni: bool,
    manual_undo: bool,
}

impl Engine {
    /// Creates a new Engine with the specified configuration.
    pub fn new(config: EngineConfig) -> Self {
        Self {
            telex: TelexEngine::with_options(
                config.tone_style,
                config.type_w_to_u,
                config.bracket_transform,
            ),
            vni: VniEngine::with_bracket_transform(config.tone_style, config.bracket_transform),
            auto_restore: EnglishAutoRestore::new(),
            casing: CaseTracker::new(),
            macro_table: MacroTable::new(),
            config,
            current_word: String::with_capacity(32),
            raw_strokes: String::with_capacity(32),
            word_used_telex: false,
            word_used_vni: false,
            manual_undo: false,
        }
    }

    /// Loads custom macros from configuration file.
    pub fn load_macros_from_file(&mut self, path: &str) {
        self.macro_table.load_from_file(path);
    }

    /// Returns the current engine configuration.
    pub fn config(&self) -> &EngineConfig {
        &self.config
    }

    /// Sets the input method.
    pub fn set_method(&mut self, method: InputMethod) {
        self.config.method = method;
    }

    /// Returns the current input method.
    pub fn method(&self) -> InputMethod {
        self.config.method
    }

    /// Enables or disables Restore English (auto-restore).
    pub fn set_auto_restore_english(&mut self, enabled: bool) {
        self.config.auto_restore_english = enabled;
    }

    /// Returns whether Restore English is enabled.
    pub fn auto_restore_english(&self) -> bool {
        self.config.auto_restore_english
    }

    /// Enables or disables smart casing.
    pub fn set_smart_casing(&mut self, enabled: bool) {
        self.config.smart_casing = enabled;
    }

    /// Returns whether smart casing is enabled.
    pub fn smart_casing(&self) -> bool {
        self.config.smart_casing
    }

    /// Enables or disables macro expansion.
    pub fn set_enable_macro(&mut self, enabled: bool) {
        self.config.enable_macro = enabled;
    }

    /// Returns whether macro expansion is enabled.
    pub fn macro_enabled(&self) -> bool {
        self.config.enable_macro
    }

    /// Sets tone placement style.
    pub fn set_tone_style(&mut self, tone_style: ToneStyle) {
        self.config.tone_style = tone_style;
        self.telex.set_tone_style(tone_style);
        self.vni.set_tone_style(tone_style);
    }

    /// Sets W to Ư rule.
    pub fn set_type_w_to_u(&mut self, w2u: W2UType) {
        self.config.type_w_to_u = w2u;
        self.telex.set_type_w_to_u(w2u);
    }

    /// Enables or disables bracket transformation.
    pub fn set_bracket_transform(&mut self, enabled: bool) {
        self.config.bracket_transform = enabled;
        self.telex.set_bracket_transform(enabled);
        self.vni.set_bracket_transform(enabled);
    }

    /// Resets all internal buffers (e.g. on window unfocus or navigation key).
    pub fn reset(&mut self) {
        self.current_word.clear();
        self.raw_strokes.clear();
        self.casing.reset();
        self.word_used_telex = false;
        self.word_used_vni = false;
        self.manual_undo = false;
    }

    /// Handles physical Backspace pressed by the user.
    pub fn handle_backspace(&mut self) {
        self.current_word.pop();
        self.raw_strokes.pop();
        if self.current_word.is_empty() {
            self.reset();
        } else {
            self.manual_undo = true;
        }
    }

    /// Process a key press event.
    pub fn process_key(&mut self, ch: char, _timestamp_ms: f64) -> EngineOutput {
        let is_bracket = self.config.bracket_transform && matches!(ch, '[' | ']' | '{' | '}');

        // Word delimiter check (Space, Enter, Punctuation)
        if !ch.is_alphanumeric() && !is_bracket {
            if !self.current_word.is_empty() {
                // 1. Macro expansion check
                if self.config.enable_macro {
                    if let Some(expansion) = self.macro_table.expand(&self.current_word) {
                        let target = format!("{}{}", expansion, ch);
                        let (backspaces, commit_text) =
                            compare_and_split_strings(&self.current_word, &target);
                        self.reset();
                        return EngineOutput {
                            backspaces,
                            commit_text,
                            consumed: true,
                            auto_restored: false,
                            macro_expanded: true,
                            adaptive_delay_ms: 0,
                        };
                    }
                }

                // 2. Restore English check on word completion
                if self.config.auto_restore_english
                    && !self.manual_undo
                    && has_vietnamese_diacritic(&self.current_word)
                    && (self.auto_restore.is_english_keyword(&self.raw_strokes)
                        || !is_valid_vietnamese_syllable(&self.current_word))
                {
                    let target = format!("{}{}", self.raw_strokes, ch);
                    let (backspaces, commit_text) =
                        compare_and_split_strings(&self.current_word, &target);
                    self.reset();
                    return EngineOutput {
                        backspaces,
                        commit_text,
                        consumed: true,
                        auto_restored: true,
                        macro_expanded: false,
                        adaptive_delay_ms: 0,
                    };
                }
            }

            // No macro or auto-restore needed: reset word buffer and let delimiter pass through natively
            self.reset();
            return EngineOutput {
                backspaces: 0,
                commit_text: ch.to_string(),
                consumed: false,
                auto_restored: false,
                macro_expanded: false,
                adaptive_delay_ms: 0,
            };
        }

        // Handle digits (0-9)
        if ch.is_ascii_digit() {
            match self.config.method {
                InputMethod::Telex | InputMethod::SimpleTelex => {
                    self.reset();
                    return EngineOutput {
                        backspaces: 0,
                        commit_text: ch.to_string(),
                        consumed: false,
                        auto_restored: false,
                        macro_expanded: false,
                        adaptive_delay_ms: 0,
                    };
                }
                InputMethod::Auto => {
                    if self.word_used_telex || self.current_word.is_empty() {
                        self.reset();
                        return EngineOutput {
                            backspaces: 0,
                            commit_text: ch.to_string(),
                            consumed: false,
                            auto_restored: false,
                            macro_expanded: false,
                            adaptive_delay_ms: 0,
                        };
                    }
                    let (new_word, was_transformed, was_undo) =
                        self.vni.apply_key(&self.current_word, ch);
                    if !was_transformed {
                        self.reset();
                        return EngineOutput {
                            backspaces: 0,
                            commit_text: ch.to_string(),
                            consumed: false,
                            auto_restored: false,
                            macro_expanded: false,
                            adaptive_delay_ms: 0,
                        };
                    }
                    self.word_used_vni = true;
                    self.manual_undo |= was_undo;
                    self.raw_strokes.push(ch);
                    let (backspaces, commit_text) =
                        compare_and_split_strings(&self.current_word, &new_word);
                    self.current_word = new_word;
                    let consumed = backspaces > 0 || commit_text != ch.to_string();
                    return EngineOutput {
                        backspaces,
                        commit_text,
                        consumed,
                        auto_restored: false,
                        macro_expanded: false,
                        adaptive_delay_ms: 0,
                    };
                }
                InputMethod::Vni => {
                    if self.current_word.is_empty() {
                        return EngineOutput {
                            backspaces: 0,
                            commit_text: ch.to_string(),
                            consumed: false,
                            auto_restored: false,
                            macro_expanded: false,
                            adaptive_delay_ms: 0,
                        };
                    }
                    let (new_word, was_transformed, was_undo) =
                        self.vni.apply_key(&self.current_word, ch);
                    if !was_transformed {
                        self.reset();
                        return EngineOutput {
                            backspaces: 0,
                            commit_text: ch.to_string(),
                            consumed: false,
                            auto_restored: false,
                            macro_expanded: false,
                            adaptive_delay_ms: 0,
                        };
                    }
                    self.word_used_vni = true;
                    self.manual_undo |= was_undo;
                    self.raw_strokes.push(ch);
                    let (backspaces, commit_text) =
                        compare_and_split_strings(&self.current_word, &new_word);
                    self.current_word = new_word;
                    let consumed = backspaces > 0 || commit_text != ch.to_string();
                    return EngineOutput {
                        backspaces,
                        commit_text,
                        consumed,
                        auto_restored: false,
                        macro_expanded: false,
                        adaptive_delay_ms: 0,
                    };
                }
            }
        }

        // Non-digit character (letters and brackets)
        let (mut new_word, was_transformed, was_undo) = match self.config.method {
            InputMethod::Auto => {
                if self.word_used_vni {
                    self.vni.apply_key(&self.current_word, ch)
                } else {
                    let res = self
                        .telex
                        .apply_key(&self.current_word, ch, &self.raw_strokes);
                    if res.1 {
                        self.word_used_telex = true;
                    }
                    res
                }
            }
            InputMethod::Telex => {
                let res = self
                    .telex
                    .apply_key(&self.current_word, ch, &self.raw_strokes);
                if res.1 {
                    self.word_used_telex = true;
                }
                res
            }
            InputMethod::SimpleTelex => {
                let mut simple = self.telex.clone();
                simple.set_type_w_to_u(W2UType::Disabled);
                let res = simple.apply_key(&self.current_word, ch, &self.raw_strokes);
                if res.1 {
                    self.word_used_telex = true;
                }
                res
            }
            InputMethod::Vni => self.vni.apply_key(&self.current_word, ch),
        };

        self.raw_strokes.push(ch);
        self.casing.track(ch);
        self.manual_undo |= was_undo;

        // Live Restore English check: if the syllable had Vietnamese marks and just became
        // orthographically impossible (e.g. 'text' -> 'tẽt', 'general' -> 'gểna', 'case' -> 'cáe'),
        // immediately restore to raw_strokes just like fcitx5-lilypad!
        let mut auto_restored = false;
        if self.config.auto_restore_english
            && !self.manual_undo
            && has_vietnamese_diacritic(&new_word)
            && !is_valid_vietnamese_syllable(&new_word)
        {
            new_word = self.raw_strokes.clone();
            self.manual_undo = true;
            auto_restored = true;
        }

        let (backspaces, commit_text) = compare_and_split_strings(&self.current_word, &new_word);
        self.current_word = new_word;

        // Only consume the key event if earlier characters were modified or the committed character differs from `ch`
        let consumed = was_transformed || auto_restored || backspaces > 0 || commit_text != ch.to_string();

        EngineOutput {
            backspaces,
            commit_text,
            consumed,
            auto_restored,
            macro_expanded: false,
            adaptive_delay_ms: 0,
        }
    }
}

/// Computes the minimal suffix replacement between `old_word` and `new_word`
/// using Longest Common Prefix diffing (ported from `fcitx5-lilypad` `compareAndSplitStrings`).
/// Returns `(backspaces_to_delete, suffix_to_commit)`.
pub fn compare_and_split_strings(old_word: &str, new_word: &str) -> (usize, String) {
    let old_chars: Vec<char> = old_word.chars().collect();
    let new_chars: Vec<char> = new_word.chars().collect();

    let mut common_len = 0;
    let min_len = old_chars.len().min(new_chars.len());
    while common_len < min_len && old_chars[common_len] == new_chars[common_len] {
        common_len += 1;
    }

    let backspaces = old_chars.len() - common_len;
    let commit_text: String = new_chars[common_len..].iter().collect();
    (backspaces, commit_text)
}

#[cfg(test)]
mod tests {
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
    }

    #[test]
    fn test_micro_diff_minimal_backspaces() {
        let mut engine = Engine::new(EngineConfig {
            method: InputMethod::Telex,
            ..EngineConfig::default()
        });

        // Type k-h-o-n-g: none of these should consume or emit backspaces
        for ch in ['k', 'h', 'o', 'n', 'g'] {
            let out = engine.process_key(ch, 0.0);
            assert!(!out.consumed);
            assert_eq!(out.backspaces, 0);
        }

        // Now type 'o' -> transforms "khong" to "không":
        // Common prefix is "kh", so only "ong" (3 chars) is deleted and "ông" is committed!
        let out = engine.process_key('o', 0.0);
        assert!(out.consumed);
        assert_eq!(out.backspaces, 3);
        assert_eq!(out.commit_text, "ông");
    }

    #[test]
    fn test_restore_english_live_and_on_space() {
        let mut engine = Engine::new(EngineConfig {
            method: InputMethod::Telex,
            auto_restore_english: true,
            ..EngineConfig::default()
        });

        assert_eq!(type_word(&mut engine, "text "), "text ");
        assert_eq!(type_word(&mut engine, "post "), "post ");
        assert_eq!(type_word(&mut engine, "general "), "general ");
        assert_eq!(type_word(&mut engine, "filter "), "filter ");
        assert_eq!(type_word(&mut engine, "return "), "return ");
        assert_eq!(type_word(&mut engine, "struct "), "struct ");
        assert_eq!(type_word(&mut engine, "windows "), "windows ");
    }
}
