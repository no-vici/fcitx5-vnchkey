//! Syllable State Machine and Input Orchestrator.

use crate::auto_restore::EnglishAutoRestore;
use crate::casing::CaseTracker;
use crate::macros::MacroTable;
pub use crate::telex::W2UType;
use crate::telex::{
    extract_last_syllable_index, has_vietnamese_diacritic, is_valid_in_progress_syllable,
    is_valid_vietnamese_syllable, TelexEngine,
};
use crate::vietnamese_dict::{has_phonology_violation, is_in_vietnamese_dictionary};
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
    /// Classic standard: đặt dấu theo style cũ (hòa, ủy, hòe)
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
    /// Enable Chánh Tả spell checking using built-in ~6,750 Vietnamese dictionary & phonology rules
    pub spell_check: bool,
    /// Enable User Dictionary of English & custom bypass words
    pub enable_dictionary: bool,
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
            spell_check: true,
            enable_dictionary: true,
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
    english_locked: bool,
    last_completed_word: Option<String>,
    last_macro: Option<LastMacroExpansion>,
    suppressed_macro: Option<String>,
    reopen_state: Option<ReopenWordState>,
}

/// Saved state of the previous word completed with a single Space,
/// allowing "Reopen Previous Word" if Backspace is pressed immediately after Space (Dokey style).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReopenWordState {
    /// Active Vietnamese word when space was pressed
    pub current_word: String,
    /// Raw input keystrokes
    pub raw_strokes: String,
    /// Whether Telex rules transformed this word
    pub word_used_telex: bool,
    /// Whether VNI rules transformed this word
    pub word_used_vni: bool,
    /// Manual undo state
    pub manual_undo: bool,
    /// Whether English mode was locked
    pub english_locked: bool,
}

/// Tracks the most recent macro expansion for one-step undo on Backspace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LastMacroExpansion {
    /// The original abbreviation keystroke sequence (e.g. "tvi")
    pub trigger: String,
    /// The expanded replacement string (e.g. "Tiếng Việt")
    pub expansion: String,
    /// The delimiter character that triggered expansion (e.g. ' ')
    pub delimiter: char,
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
            english_locked: false,
            last_completed_word: None,
            last_macro: None,
            suppressed_macro: None,
            reopen_state: None,
        }
    }

    /// Loads custom macros from configuration file.
    pub fn load_macros_from_file(&mut self, path: &str) {
        self.macro_table.load_from_file(path);
    }

    /// Loads custom user dictionary words from configuration file.
    pub fn load_dictionary_from_file(&mut self, path: &str) {
        self.auto_restore.load_from_file(path);
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

    /// Enables or disables Chánh Tả spell checking.
    pub fn set_spell_check(&mut self, enabled: bool) {
        self.config.spell_check = enabled;
    }

    /// Returns whether Chánh Tả spell checking is enabled.
    pub fn spell_check(&self) -> bool {
        self.config.spell_check
    }

    /// Enables or disables the User Dictionary of English & custom bypass words.
    pub fn set_enable_dictionary(&mut self, enabled: bool) {
        self.config.enable_dictionary = enabled;
    }

    /// Returns whether the User Dictionary is enabled.
    pub fn dictionary_enabled(&self) -> bool {
        self.config.enable_dictionary
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

    /// Returns the active tone placement style.
    pub fn tone_style(&self) -> ToneStyle {
        self.config.tone_style
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

    fn reset_word_buffers(&mut self) {
        self.current_word.clear();
        self.raw_strokes.clear();
        self.casing.reset();
        self.word_used_telex = false;
        self.word_used_vni = false;
        self.manual_undo = false;
        self.english_locked = false;
        self.last_completed_word = None;
        self.suppressed_macro = None;
    }

    /// Resets all internal buffers (e.g. on window unfocus or navigation key).
    pub fn reset(&mut self) {
        self.reset_word_buffers();
        self.reopen_state = None;
    }

    /// Handles physical Backspace pressed by the user.
    /// If Backspace is pressed immediately after completing a word with a single Space,
    /// reopens the previous word so marks/tones can be applied (Dokey style).
    /// Returns true if a previously completed word was reopened.
    pub fn handle_backspace(&mut self) -> bool {
        if self.current_word.is_empty() {
            if let Some(saved) = self.reopen_state.take() {
                self.current_word = saved.current_word;
                self.raw_strokes = saved.raw_strokes;
                self.word_used_telex = saved.word_used_telex;
                self.word_used_vni = saved.word_used_vni;
                self.manual_undo = false;
                self.english_locked = saved.english_locked;
                return true;
            }
            self.reset();
            false
        } else {
            self.reopen_state = None;
            self.current_word.pop();
            self.raw_strokes.pop();
            if self.current_word.is_empty() {
                self.reset();
            } else {
                self.manual_undo = true;
            }
            false
        }
    }

    /// Reverts the active composing word back to the raw keystrokes (Escape to Undo)
    /// and temporarily locks English mode for this word.
    pub fn undo_to_raw(&mut self) -> Option<EngineOutput> {
        if self.current_word.is_empty() || self.raw_strokes.is_empty() {
            return None;
        }
        if self.current_word == self.raw_strokes {
            self.english_locked = true;
            return None;
        }

        let old_word = self.current_word.clone();
        self.current_word = self.raw_strokes.clone();
        self.english_locked = true;

        let (backspaces, commit_text) = compare_and_split_strings(&old_word, &self.current_word);
        Some(EngineOutput {
            backspaces,
            commit_text,
            consumed: true,
            auto_restored: true,
            macro_expanded: false,
            adaptive_delay_ms: 0,
        })
    }

    /// Returns the active composing word.
    pub fn current_word(&self) -> &str {
        &self.current_word
    }

    /// Returns the raw keystrokes sequence.
    pub fn raw_strokes(&self) -> &str {
        &self.raw_strokes
    }

    /// Returns whether the engine is currently composing an active word.
    pub fn has_active_word(&self) -> bool {
        !self.current_word.is_empty()
    }

    /// Returns the last completed word if a word boundary was just reached.
    pub fn last_completed_word(&self) -> Option<&str> {
        self.last_completed_word.as_deref()
    }

    /// Flips the current composing word between processed Vietnamese and raw input strokes.
    /// Returns the minimal suffix replacement (backspaces and commit_text) to update the document.
    pub fn flip_current_word(&mut self) -> Option<EngineOutput> {
        if self.current_word.is_empty() || self.raw_strokes.is_empty() {
            return None;
        }

        let is_currently_raw = self.current_word == self.raw_strokes;
        let old_word = self.current_word.clone();

        let new_word = if is_currently_raw {
            // Re-transform raw strokes into Vietnamese
            let strokes = self.raw_strokes.clone();
            self.reset();
            for ch in strokes.chars() {
                self.process_key(ch, 0.0);
            }
            self.current_word.clone()
        } else {
            // Revert to raw strokes and lock English mode for this word
            self.current_word = self.raw_strokes.clone();
            self.english_locked = true;
            self.current_word.clone()
        };

        let (backspaces, commit_text) = compare_and_split_strings(&old_word, &new_word);
        Some(EngineOutput {
            backspaces,
            commit_text,
            consumed: true,
            auto_restored: false,
            macro_expanded: false,
            adaptive_delay_ms: 0,
        })
    }

    /// Checks if the last action was a macro expansion and reverts it on Backspace.
    /// Returns (backspaces_to_send, text_to_commit) if reverted.
    pub fn undo_macro_expansion(&mut self) -> Option<(usize, String)> {
        if let Some(macro_state) = self.last_macro.take() {
            // To revert "Expansion + Delimiter" back to "Trigger":
            // 1 delimiter character + all characters of the expansion
            let delete_count = 1 + macro_state.expansion.chars().count();
            let commit = macro_state.trigger.clone();
            self.suppressed_macro = Some(macro_state.trigger.clone());
            self.current_word = macro_state.trigger;
            self.raw_strokes = self.current_word.clone();
            self.manual_undo = false;
            Some((delete_count, commit))
        } else {
            None
        }
    }

    /// Process a key press event.
    pub fn process_key(&mut self, ch: char, _timestamp_ms: f64) -> EngineOutput {
        self.last_completed_word = None;
        let is_bracket = self.config.bracket_transform && matches!(ch, '[' | ']' | '{' | '}');

        // Any new keypress (other than Backspace) cancels pending macro undo state
        self.last_macro = None;

        // Word delimiter check (Space, Enter, Punctuation)
        if !ch.is_alphanumeric() && !is_bracket {
            if !self.current_word.is_empty() {
                // 1. Macro expansion check (suppressed if this word was just reverted via Backspace)
                let is_suppressed = self.suppressed_macro.as_deref() == Some(&self.current_word);
                if self.config.enable_macro && !is_suppressed {
                    if let Some(expansion) = self.macro_table.expand(&self.current_word) {
                        let target = format!("{}{}", expansion, ch);
                        let (backspaces, commit_text) =
                            compare_and_split_strings(&self.current_word, &target);
                        let trigger = self.current_word.clone();
                        self.reset();
                        self.last_macro = Some(LastMacroExpansion {
                            trigger,
                            expansion: expansion.clone(),
                            delimiter: ch,
                        });
                        self.last_completed_word = Some(target);
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

                // 2. User Dictionary & Restore English + Chánh Tả check on word completion
                // Never auto-restore VNI digit sequences (e.g. gio72, duoc975) because English words never contain digits.
                let has_digits = self.raw_strokes.chars().any(|c| c.is_ascii_digit());
                if !self.manual_undo && !has_digits && has_vietnamese_diacritic(&self.current_word) {
                    let dict_restore = self.config.enable_dictionary
                        && self.auto_restore.should_restore(
                            &self.raw_strokes,
                            &self.current_word,
                            true,
                        );
                    let english_or_spell_restore = self.config.auto_restore_english
                        && (self.auto_restore.is_english_keyword(&self.raw_strokes)
                            || !is_valid_vietnamese_syllable(&self.current_word)
                            || has_phonology_violation(&self.current_word)
                            || (self.config.spell_check
                                && !is_in_vietnamese_dictionary(&self.current_word)));

                    if dict_restore || english_or_spell_restore {
                        let target = format!("{}{}", self.raw_strokes, ch);
                        let (backspaces, commit_text) =
                            compare_and_split_strings(&self.current_word, &target);
                        self.reset();
                        self.last_completed_word = Some(target);
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
            }

            // No macro or auto-restore needed: reset word buffer and let delimiter pass through natively
            let has_word = !self.current_word.is_empty();
            let target = format!("{}{}", self.current_word, ch);
            if ch == ' ' && has_word && self.raw_strokes.len() <= 32 {
                self.reopen_state = Some(ReopenWordState {
                    current_word: self.current_word.clone(),
                    raw_strokes: self.raw_strokes.clone(),
                    word_used_telex: self.word_used_telex,
                    word_used_vni: self.word_used_vni,
                    manual_undo: self.manual_undo,
                    english_locked: self.english_locked,
                });
            } else {
                self.reopen_state = None;
            }
            self.reset_word_buffers();
            if has_word {
                self.last_completed_word = Some(target);
            }
            return EngineOutput {
                backspaces: 0,
                commit_text: ch.to_string(),
                consumed: false,
                auto_restored: false,
                macro_expanded: false,
                adaptive_delay_ms: 0,
            };
        }

        // Any non-delimiter key cancels pending reopen state
        self.reopen_state = None;

        // Developer-Aware: camelCase identifier detection (e.g. "parse" + 'S' -> "parseS")
        // If an uppercase letter arrives after lowercase letters, restore raw prefix if it had diacritics
        // and lock English mode for this identifier.
        let prev_has_lower = self.raw_strokes.chars().any(|c| c.is_ascii_lowercase());
        if prev_has_lower && ch.is_ascii_uppercase() {
            let old_word = self.current_word.clone();
            self.raw_strokes.push(ch);
            self.current_word = self.raw_strokes.clone();
            self.english_locked = true;
            let (backspaces, commit_text) = compare_and_split_strings(&old_word, &self.current_word);
            return EngineOutput {
                backspaces,
                commit_text,
                consumed: true,
                auto_restored: true,
                macro_expanded: false,
                adaptive_delay_ms: 0,
            };
        }

        // If this word was already restored/locked as English, pass subsequent letters through untouched
        if self.english_locked
            && !(ch.is_ascii_digit()
                && matches!(self.config.method, InputMethod::Vni | InputMethod::Auto))
        {
            self.raw_strokes.push(ch);
            self.current_word.push(ch);
            self.casing.track(ch);
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
                        let split_idx = extract_last_syllable_index(&self.current_word);
                        let char_count = self.current_word.chars().count();
                        if split_idx > 0 && split_idx < char_count {
                            let tail: String = self.current_word.chars().skip(split_idx).collect();
                            let (new_tail, tail_transformed, tail_undo) =
                                self.vni.apply_key(&tail, ch);
                            if tail_transformed {
                                self.word_used_vni = true;
                                self.english_locked = false;
                                self.manual_undo |= tail_undo;
                                self.raw_strokes = format!("{}{}", tail, ch);
                                let (backspaces, commit_text) =
                                    compare_and_split_strings(&tail, &new_tail);
                                self.current_word = new_tail;
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
                    self.english_locked = false;
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
                        let split_idx = extract_last_syllable_index(&self.current_word);
                        let char_count = self.current_word.chars().count();
                        if split_idx > 0 && split_idx < char_count {
                            let tail: String = self.current_word.chars().skip(split_idx).collect();
                            let (new_tail, tail_transformed, tail_undo) =
                                self.vni.apply_key(&tail, ch);
                            if tail_transformed {
                                self.word_used_vni = true;
                                self.english_locked = false;
                                self.manual_undo |= tail_undo;
                                self.raw_strokes = format!("{}{}", tail, ch);
                                let (backspaces, commit_text) =
                                    compare_and_split_strings(&tail, &new_tail);
                                self.current_word = new_tail;
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
                    self.english_locked = false;
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

        // Live Restore English, User Dictionary & Chánh Tả phonology check:
        // 1. If the syllable has Vietnamese marks and violates Vietnamese phonology/structure
        //    (e.g. 'text' -> 'tẽt', 'general' -> 'gểna', 'center' -> 'cẻnt', 'case' -> 'cáe'),
        //    or matches a known English word in the User Dictionary, immediately restore to raw_strokes!
        //    Never auto-restore VNI digit sequences (e.g. "biet61" followed by Neovim's `jk` escape mapping).
        let mut auto_restored = false;
        let has_digits = self.raw_strokes.chars().any(|c| c.is_ascii_digit());
        let is_vni_word =
            has_digits || self.word_used_vni || self.config.method == InputMethod::Vni;
        let has_diacritic = has_vietnamese_diacritic(&new_word);
        let new_chars: Vec<char> = new_word.chars().collect();
        let invalid_structure = has_diacritic
            && (!is_valid_in_progress_syllable(&new_chars) || has_phonology_violation(&new_word));
        let dict_match = !is_vni_word
            && has_diacritic
            && self.config.enable_dictionary
            && self
                .auto_restore
                .should_restore(&self.raw_strokes, &new_word, true);

        if !self.manual_undo
            && !is_vni_word
            && ((self.config.auto_restore_english && invalid_structure) || dict_match)
        {
            new_word = self.raw_strokes.clone();
            self.english_locked = true;
            auto_restored = true;
        } else if self.config.spell_check && !self.manual_undo && was_transformed && invalid_structure
        {
            // If Chánh Tả is enabled (even when Restore English is disabled), prevent invalid Vietnamese syllable marks
            new_word = format!("{}{}", self.current_word, ch);
        }

        let (backspaces, commit_text) = compare_and_split_strings(&self.current_word, &new_word);
        self.current_word = new_word;

        // If a VNI word already has diacritics and a subsequent character (such as Neovim's `jk` escape
        // or the start of an adjacent word) breaks the syllable structure, keep the committed Vietnamese
        // word intact on screen and advance the buffer to the new trailing syllable.
        if is_vni_word && !was_transformed && invalid_structure {
            let split_idx = extract_last_syllable_index(&self.current_word);
            if split_idx > 0 {
                let tail: String = self.current_word.chars().skip(split_idx).collect();
                self.current_word = tail.clone();
                self.raw_strokes = tail;
                self.word_used_vni = false;
                self.manual_undo = false;
            }
        }

        // Only consume the key event if earlier characters were modified or the committed character differs from `ch`
        let consumed =
            was_transformed || auto_restored || backspaces > 0 || commit_text != ch.to_string();

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
    fn test_restore_english_and_chanh_ta_dictionary() {
        let mut engine = Engine::new(EngineConfig {
            method: InputMethod::Telex,
            auto_restore_english: true,
            spell_check: true,
            enable_dictionary: true,
            ..EngineConfig::default()
        });

        assert_eq!(type_word(&mut engine, "text "), "text ");
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
        assert_eq!(type_word(&mut engine, "athuong72 "), "athường ");
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

        // User immediately presses Backspace: macro expansion is undone!
        let undo = engine.undo_macro_expansion();
        assert!(undo.is_some());
        let (bs, commit) = undo.unwrap();
        // 1 (space) + 8 ("Việt Nam".chars().count()) = 9 backspaces
        assert_eq!(bs, 9);
        assert_eq!(commit, "vn");
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

        // Test 2: Typing another key cancels macro undo
        for ch in ['v', 'n', ' '] {
            engine.process_key(ch, 0.0);
        }
        engine.process_key('a', 0.0);
        assert!(engine.undo_macro_expansion().is_none());
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
}
