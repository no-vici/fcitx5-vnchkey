//! Syllable State Machine and Input Orchestrator.

use crate::auto_restore::EnglishAutoRestore;
use crate::casing::CaseTracker;
use crate::habit_learner::{CorrectionKind, HabitLearner};
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

/// Output character set encoding standard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Charset {
    /// Precomposed Unicode (NFC / Dựng sẵn) - default standard
    Unicode,
    /// Decomposed Unicode (NFD / Tổ hợp) - for legacy apps, XWayland graphics editors
    UnicodeComposite,
}

/// Engine configuration options.
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Selected input method (Auto, Telex, or VNI)
    pub method: InputMethod,
    /// Preferred tone placement convention
    pub tone_style: ToneStyle,
    /// Output character set (Unicode NFC vs Unicode Composite NFD)
    pub charset: Charset,
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
    /// Type w to produce ư rule
    pub type_w_to_u: W2UType,
    /// Type [ -> ơ, ] -> ư, { -> Ơ, } -> Ư
    pub bracket_transform: bool,
    /// Enable macro expansions
    pub enable_macro: bool,
    /// Idle keystroke timeout in milliseconds (0 = disabled, default = 0ms)
    pub idle_timeout_ms: u64,
    /// Quick Telex consonants (cc->ch, gg->gi, kk->kh, nn->ng, pp->ph, qq->qu, tt->th)
    pub quick_telex: bool,
    /// Quick initial consonants (f->ph, j->gi, w->qu at start of word)
    pub quick_start_consonant: bool,
    /// Quick final consonants (g->ng, h->nh, k->ch at end of word)
    pub quick_end_consonant: bool,
    /// Enables on-device adaptive habit learning (defaults to true)
    pub habit_learning_enabled: bool,
    /// Number of corrections required before auto-promoting to whitelist (default: 4)
    pub habit_promote_threshold: u32,
    /// Harmonize syllable casing based on initial character (ĐƯợc/ĐƯỢc -> Được, đƯỢc -> được). Defaults to false.
    pub initial_case_anchor: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            method: InputMethod::Auto,
            tone_style: ToneStyle::Classic,
            charset: Charset::Unicode,
            auto_restore_english: true,
            spell_check: true,
            enable_dictionary: true,
            smart_casing: true,
            quick_undo: true,
            type_w_to_u: W2UType::NonStart,
            bracket_transform: false,
            enable_macro: true,
            idle_timeout_ms: 0,
            quick_telex: false,
            quick_start_consonant: false,
            quick_end_consonant: false,
            habit_learning_enabled: true,
            habit_promote_threshold: 4,
            initial_case_anchor: false,
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
    habit_learner: HabitLearner,
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
    deleted_accented_candidate: Option<(String, String)>,
    last_delimiter: Option<char>,
    last_delimiter_spaced: bool,
    last_keystroke_time_ms: f64,
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

/// Stage of macro undo tracking on Backspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacroUndoStage {
    /// Newly expanded macro (e.g. "vn" + ' ' -> "Việt Nam ")
    Expanded,
    /// Trailing space was deleted by first Backspace (e.g. "Việt Nam")
    TrailingSpaceRemoved,
}

/// Tracks the most recent macro expansion for two-step undo on Backspace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LastMacroExpansion {
    /// The original abbreviation keystroke sequence (e.g. "vn")
    pub trigger: String,
    /// The expanded replacement string (e.g. "Việt Nam")
    pub expansion: String,
    /// The delimiter character that triggered expansion (e.g. ' ')
    pub delimiter: char,
    /// The current undo stage
    pub stage: MacroUndoStage,
}

/// Returns whether a punctuation mark should snap directly to the preceding word
/// without a leading space (e.g. ',', '.', '!', '?', ';', ':', ')', ']', '}', '>').
#[inline]
pub fn is_snappable_punctuation(ch: char) -> bool {
    matches!(ch, ',' | '.' | '!' | '?' | ';' | ':' | ')' | ']' | '}' | '>')
}

/// Converts precomposed Vietnamese characters to decomposed Unicode (NFD / Unicode Tổ Hợp).
pub fn to_composite_unicode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for ch in s.chars() {
        match ch {
            'á' => out.push_str("a\u{0301}"),
            'à' => out.push_str("a\u{0300}"),
            'ả' => out.push_str("a\u{0309}"),
            'ã' => out.push_str("a\u{0303}"),
            'ạ' => out.push_str("a\u{0323}"),
            'â' => out.push_str("a\u{0302}"),
            'ấ' => out.push_str("a\u{0302}\u{0301}"),
            'ầ' => out.push_str("a\u{0302}\u{0300}"),
            'ẩ' => out.push_str("a\u{0302}\u{0309}"),
            'ẫ' => out.push_str("a\u{0302}\u{0303}"),
            'ậ' => out.push_str("a\u{0302}\u{0323}"),
            'ă' => out.push_str("a\u{0306}"),
            'ắ' => out.push_str("a\u{0306}\u{0301}"),
            'ằ' => out.push_str("a\u{0306}\u{0300}"),
            'ẳ' => out.push_str("a\u{0306}\u{0309}"),
            'ẵ' => out.push_str("a\u{0306}\u{0303}"),
            'ặ' => out.push_str("a\u{0306}\u{0323}"),

            'Á' => out.push_str("A\u{0301}"),
            'À' => out.push_str("A\u{0300}"),
            'Ả' => out.push_str("A\u{0309}"),
            'Ã' => out.push_str("A\u{0303}"),
            'Ạ' => out.push_str("A\u{0323}"),
            'Â' => out.push_str("A\u{0302}"),
            'Ấ' => out.push_str("A\u{0302}\u{0301}"),
            'Ầ' => out.push_str("A\u{0302}\u{0300}"),
            'Ẩ' => out.push_str("A\u{0302}\u{0309}"),
            'Ẫ' => out.push_str("A\u{0302}\u{0303}"),
            'Ậ' => out.push_str("A\u{0302}\u{0323}"),
            'Ă' => out.push_str("A\u{0306}"),
            'Ắ' => out.push_str("A\u{0306}\u{0301}"),
            'Ằ' => out.push_str("A\u{0306}\u{0300}"),
            'Ẳ' => out.push_str("A\u{0306}\u{0309}"),
            'Ẵ' => out.push_str("A\u{0306}\u{0303}"),
            'Ặ' => out.push_str("A\u{0306}\u{0323}"),

            'é' => out.push_str("e\u{0301}"),
            'è' => out.push_str("e\u{0300}"),
            'ẻ' => out.push_str("e\u{0309}"),
            'ẽ' => out.push_str("e\u{0303}"),
            'ẹ' => out.push_str("e\u{0323}"),
            'ê' => out.push_str("e\u{0302}"),
            'ế' => out.push_str("e\u{0302}\u{0301}"),
            'ề' => out.push_str("e\u{0302}\u{0300}"),
            'ể' => out.push_str("e\u{0302}\u{0309}"),
            'ễ' => out.push_str("e\u{0302}\u{0303}"),
            'ệ' => out.push_str("e\u{0302}\u{0323}"),

            'É' => out.push_str("E\u{0301}"),
            'È' => out.push_str("E\u{0300}"),
            'Ẻ' => out.push_str("E\u{0309}"),
            'Ẽ' => out.push_str("E\u{0303}"),
            'Ẹ' => out.push_str("E\u{0323}"),
            'Ê' => out.push_str("E\u{0302}"),
            'Ế' => out.push_str("E\u{0302}\u{0301}"),
            'Ề' => out.push_str("E\u{0302}\u{0300}"),
            'Ể' => out.push_str("E\u{0302}\u{0309}"),
            'Ễ' => out.push_str("E\u{0302}\u{0303}"),
            'Ệ' => out.push_str("E\u{0302}\u{0323}"),

            'í' => out.push_str("i\u{0301}"),
            'ì' => out.push_str("i\u{0300}"),
            'ỉ' => out.push_str("i\u{0309}"),
            'ĩ' => out.push_str("i\u{0303}"),
            'ị' => out.push_str("i\u{0323}"),

            'Í' => out.push_str("I\u{0301}"),
            'Ì' => out.push_str("I\u{0300}"),
            'Ỉ' => out.push_str("I\u{0309}"),
            'Ĩ' => out.push_str("I\u{0303}"),
            'Ị' => out.push_str("I\u{0323}"),

            'ó' => out.push_str("o\u{0301}"),
            'ò' => out.push_str("o\u{0300}"),
            'ỏ' => out.push_str("o\u{0309}"),
            'õ' => out.push_str("o\u{0303}"),
            'ọ' => out.push_str("o\u{0323}"),
            'ô' => out.push_str("o\u{0302}"),
            'ố' => out.push_str("o\u{0302}\u{0301}"),
            'ồ' => out.push_str("o\u{0302}\u{0300}"),
            'ổ' => out.push_str("o\u{0302}\u{0309}"),
            'ỗ' => out.push_str("o\u{0302}\u{0303}"),
            'ộ' => out.push_str("o\u{0302}\u{0323}"),
            'ơ' => out.push_str("o\u{031B}"),
            'ớ' => out.push_str("o\u{031B}\u{0301}"),
            'ờ' => out.push_str("o\u{031B}\u{0300}"),
            'ở' => out.push_str("o\u{031B}\u{0309}"),
            'ỡ' => out.push_str("o\u{031B}\u{0303}"),
            'ợ' => out.push_str("o\u{031B}\u{0323}"),

            'Ó' => out.push_str("O\u{0301}"),
            'Ò' => out.push_str("O\u{0300}"),
            'Ỏ' => out.push_str("O\u{0309}"),
            'Õ' => out.push_str("O\u{0303}"),
            'Ọ' => out.push_str("O\u{0323}"),
            'Ô' => out.push_str("O\u{0302}"),
            'Ố' => out.push_str("O\u{0302}\u{0301}"),
            'Ồ' => out.push_str("O\u{0302}\u{0300}"),
            'Ổ' => out.push_str("O\u{0302}\u{0309}"),
            'Ỗ' => out.push_str("O\u{0302}\u{0303}"),
            'Ộ' => out.push_str("O\u{0302}\u{0323}"),
            'Ơ' => out.push_str("O\u{031B}"),
            'Ớ' => out.push_str("O\u{031B}\u{0301}"),
            'Ờ' => out.push_str("O\u{031B}\u{0300}"),
            'Ở' => out.push_str("O\u{031B}\u{0309}"),
            'Ỡ' => out.push_str("O\u{031B}\u{0303}"),
            'Ợ' => out.push_str("O\u{031B}\u{0323}"),

            'ú' => out.push_str("u\u{0301}"),
            'ù' => out.push_str("u\u{0300}"),
            'ủ' => out.push_str("u\u{0309}"),
            'ũ' => out.push_str("u\u{0303}"),
            'ụ' => out.push_str("u\u{0323}"),
            'ư' => out.push_str("u\u{031B}"),
            'ứ' => out.push_str("u\u{031B}\u{0301}"),
            'ừ' => out.push_str("u\u{031B}\u{0300}"),
            'ử' => out.push_str("u\u{031B}\u{0309}"),
            'ữ' => out.push_str("u\u{031B}\u{0303}"),
            'ự' => out.push_str("u\u{031B}\u{0323}"),

            'Ú' => out.push_str("U\u{0301}"),
            'Ù' => out.push_str("U\u{0300}"),
            'Ủ' => out.push_str("U\u{0309}"),
            'Ũ' => out.push_str("U\u{0303}"),
            'Ụ' => out.push_str("U\u{0323}"),
            'Ư' => out.push_str("U\u{031B}"),
            'Ứ' => out.push_str("U\u{031B}\u{0301}"),
            'Ừ' => out.push_str("U\u{031B}\u{0300}"),
            'Ử' => out.push_str("U\u{031B}\u{0309}"),
            'Ữ' => out.push_str("U\u{031B}\u{0303}"),
            'Ự' => out.push_str("U\u{031B}\u{0323}"),

            'ý' => out.push_str("y\u{0301}"),
            'ỳ' => out.push_str("y\u{0300}"),
            'ỷ' => out.push_str("y\u{0309}"),
            'ỹ' => out.push_str("y\u{0303}"),
            'ỵ' => out.push_str("y\u{0323}"),

            'Ý' => out.push_str("Y\u{0301}"),
            'Ỳ' => out.push_str("Y\u{0300}"),
            'Ỷ' => out.push_str("Y\u{0309}"),
            'Ỹ' => out.push_str("Y\u{0303}"),
            'Ỵ' => out.push_str("Y\u{0323}"),

            other => out.push(other),
        }
    }
    out
}

impl Engine {
    /// Formats an outgoing string according to the configured output charset.
    pub fn format_commit(&self, text: &str) -> String {
        if self.config.charset == Charset::UnicodeComposite {
            to_composite_unicode(text)
        } else {
            text.to_string()
        }
    }

    /// Sets the output character set encoding (Unicode NFC vs Unicode Composite NFD).
    pub fn set_charset(&mut self, charset: Charset) {
        self.config.charset = charset;
    }

    /// Returns the current output character set.
    pub fn charset(&self) -> Charset {
        self.config.charset
    }

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
            habit_learner: {
                let mut hl = HabitLearner::new();
                hl.set_enabled(config.habit_learning_enabled);
                hl.set_promote_threshold(config.habit_promote_threshold);
                hl
            },
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
            deleted_accented_candidate: None,
            last_delimiter: None,
            last_delimiter_spaced: false,
            last_keystroke_time_ms: 0.0,
        }
    }

    /// Loads custom macros from configuration file.
    pub fn load_macros_from_file(&mut self, path: &str) {
        self.macro_table.load_from_file(path);
    }

    /// Loads custom user dictionary words from configuration file.
    pub fn load_dictionary_from_file(&mut self, path: &str) {
        self.auto_restore.load_from_file(path);
        self.habit_learner.set_dictionary_path(path);
    }

    /// Configures the high-capacity habit log path (default max size: 100 MB with FIFO rotation)
    /// and warms up in-memory adaptive whitelist from previous sessions.
    pub fn set_habit_log_path(&mut self, path: &str) {
        self.habit_learner.set_log_path(path);
    }

    /// Sets the maximum habit log size in bytes (default: 100 MB = 104,857,600 bytes).
    pub fn set_habit_max_log_bytes(&mut self, max_bytes: u64) {
        self.habit_learner.set_max_log_bytes(max_bytes);
    }

    /// Enables or disables on-device adaptive habit learning.
    pub fn set_habit_learning_enabled(&mut self, enabled: bool) {
        self.config.habit_learning_enabled = enabled;
        self.habit_learner.set_enabled(enabled);
    }

    /// Sets the number of corrections before auto-promoting a word into the whitelist.
    pub fn set_habit_promote_threshold(&mut self, threshold: u32) {
        self.config.habit_promote_threshold = threshold.max(1);
        self.habit_learner.set_promote_threshold(threshold);
    }

    /// Returns the number of corrections required to auto-promote a word into the whitelist.
    pub fn habit_promote_threshold(&self) -> u32 {
        self.habit_learner.promote_threshold()
    }

    /// Unlearns a specific word so it can be typed with Vietnamese diacritics normally again.
    pub fn unlearn_word(&mut self, word: &str) {
        self.habit_learner.unlearn_word(word);
        self.auto_restore.remove_keyword(word);
    }

    /// Returns whether on-device adaptive habit learning is enabled.
    pub fn habit_learning_enabled(&self) -> bool {
        self.habit_learner.is_enabled()
    }

    /// Returns a reference to the internal `HabitLearner`.
    pub fn habit_learner(&self) -> &HabitLearner {
        &self.habit_learner
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

    /// Enables or disables Initial Case Anchor (ĐƯợc/ĐƯỢc -> Được, đƯỢc -> được).
    pub fn set_initial_case_anchor(&mut self, enabled: bool) {
        self.config.initial_case_anchor = enabled;
    }

    /// Returns whether Initial Case Anchor is enabled.
    pub fn initial_case_anchor(&self) -> bool {
        self.config.initial_case_anchor
    }

    /// Enables or disables macro expansion.
    pub fn set_enable_macro(&mut self, enabled: bool) {
        self.config.enable_macro = enabled;
    }

    /// Returns whether macro expansion is enabled.
    pub fn macro_enabled(&self) -> bool {
        self.config.enable_macro
    }

    /// Sets Quick Telex option.
    pub fn set_quick_telex(&mut self, enabled: bool) {
        self.config.quick_telex = enabled;
    }

    /// Returns whether Quick Telex is enabled.
    pub fn quick_telex(&self) -> bool {
        self.config.quick_telex
    }

    /// Sets Quick Initial Consonants (f->ph, j->gi, w->qu).
    pub fn set_quick_start_consonant(&mut self, enabled: bool) {
        self.config.quick_start_consonant = enabled;
    }

    /// Returns whether Quick Initial Consonant is enabled.
    pub fn quick_start_consonant(&self) -> bool {
        self.config.quick_start_consonant
    }

    /// Sets Quick Final Consonants (g->ng, h->nh, k->ch).
    pub fn set_quick_end_consonant(&mut self, enabled: bool) {
        self.config.quick_end_consonant = enabled;
    }

    /// Returns whether Quick Final Consonant is enabled.
    pub fn quick_end_consonant(&self) -> bool {
        self.config.quick_end_consonant
    }

    /// Sets keystroke idle timeout in milliseconds (0 = disabled).
    pub fn set_idle_timeout_ms(&mut self, ms: u64) {
        self.config.idle_timeout_ms = ms;
    }

    /// Returns keystroke idle timeout in milliseconds.
    pub fn idle_timeout_ms(&self) -> u64 {
        self.config.idle_timeout_ms
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
        self.last_keystroke_time_ms = 0.0;
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
            let was_accented = has_vietnamese_diacritic(&self.current_word)
                && self.raw_strokes != self.current_word
                && self.raw_strokes.len() >= 2;
            let cand_to_save = if was_accented && self.deleted_accented_candidate.is_none() {
                Some((self.raw_strokes.clone(), self.current_word.clone()))
            } else {
                None
            };

            self.current_word.pop();
            self.raw_strokes.pop();
            if self.current_word.is_empty() {
                // User has deleted the ENTIRE accented word down to empty:
                // Only now do we arm deleted_accented_candidate for BackspaceRetype learning.
                if let Some(cand) = cand_to_save {
                    self.deleted_accented_candidate = Some(cand);
                }
                self.reset_word_buffers();
            } else {
                // If user only partially backspaced (e.g. 'thương' -> 'thư' to fix typing error),
                // cancel any candidate so it doesn't mistakenly learn typos.
                self.deleted_accented_candidate = None;
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

        // Record Escape-to-Raw correction in Adaptive Habit Learner
        if self.habit_learner.record_correction(
            &self.raw_strokes,
            &old_word,
            CorrectionKind::EscapeToRaw,
            self.last_keystroke_time_ms,
        ) {
            self.auto_restore.add_keyword(&self.raw_strokes);
        }

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
            if self.habit_learner.record_correction(
                &self.raw_strokes,
                &old_word,
                CorrectionKind::FlipToRaw,
                self.last_keystroke_time_ms,
            ) {
                self.auto_restore.add_keyword(&self.raw_strokes);
            }
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

    /// Checks if the last action was a macro expansion and handles two-step undo on Backspace.
    ///
    /// - Step 1: If the macro ended with Space, the first Backspace deletes only the trailing Space
    ///   so that the user can immediately type punctuation (e.g. ',', '.', '!', '?') without leading space.
    /// - Step 2: If Backspace is pressed a second time immediately, it reverts the expansion
    ///   back to the original trigger abbreviation and suppresses re-expansion on Space.
    ///
    /// Returns (backspaces_to_send, text_to_commit) if reverted.
    pub fn undo_macro_expansion(&mut self) -> Option<(usize, String)> {
        if let Some(mut macro_state) = self.last_macro.take() {
            if macro_state.delimiter == ' ' && macro_state.stage == MacroUndoStage::Expanded {
                // Step 1: delete only the trailing space
                macro_state.stage = MacroUndoStage::TrailingSpaceRemoved;
                self.last_macro = Some(macro_state);
                Some((1, String::new()))
            } else {
                // Step 2 (or non-space delimiter): revert back to original trigger
                let delete_count = if macro_state.stage == MacroUndoStage::TrailingSpaceRemoved {
                    macro_state.expansion.chars().count()
                } else {
                    1 + macro_state.expansion.chars().count()
                };
                let commit = macro_state.trigger.clone();
                self.suppressed_macro = Some(macro_state.trigger.clone());
                self.current_word = macro_state.trigger;
                self.raw_strokes = self.current_word.clone();
                self.manual_undo = false;
                Some((delete_count, commit))
            }
        } else {
            None
        }
    }

    /// Rescues a single raw word by re-processing its characters as keystrokes (Ngó Sen Word Rescue).
    /// E.g. "tieengs" -> "tiếng", "vieetj" -> "việt", "nef" -> "nè".
    pub fn rescue_word(&self, raw_word: &str) -> String {
        if self.auto_restore.is_english_keyword(raw_word) {
            return raw_word.to_string();
        }

        let mut temp_engine = Engine::new(self.config.clone());
        let mut result = String::new();
        for ch in raw_word.chars() {
            let out = temp_engine.process_key(ch, 0.0);
            for _ in 0..out.backspaces {
                result.pop();
            }
            result.push_str(&out.commit_text);
        }

        if !temp_engine.current_word.is_empty() {
            temp_engine.current_word
        } else {
            result
        }
    }

    /// Rescues a raw sentence or phrase by converting all eligible words to accented Vietnamese.
    /// Preserves whitespace, capitalization and non-alphanumeric punctuation.
    /// Returns Some(rescued_text) if any word was transformed into Vietnamese, or None if unchanged.
    pub fn rescue_sentence(&self, text: &str) -> Option<String> {
        if text.is_empty() {
            return None;
        }

        let mut output = String::with_capacity(text.len());
        let mut current_token = String::new();
        let mut in_word = false;

        for ch in text.chars() {
            if ch.is_alphanumeric() {
                if !in_word && !current_token.is_empty() {
                    output.push_str(&current_token);
                    current_token.clear();
                }
                in_word = true;
                current_token.push(ch);
            } else {
                if in_word && !current_token.is_empty() {
                    let rescued = self.rescue_word(&current_token);
                    output.push_str(&rescued);
                    current_token.clear();
                }
                in_word = false;
                current_token.push(ch);
            }
        }

        if !current_token.is_empty() {
            if in_word {
                let rescued = self.rescue_word(&current_token);
                output.push_str(&rescued);
            } else {
                output.push_str(&current_token);
            }
        }

        if output != text {
            Some(output)
        } else {
            None
        }
    }

    /// Applies a tone or diacritic key to a selected text (Selection Accent Rescue / Cứu Dấu).
    /// If the selection contains multiple words, applies the modifier to the last syllable.
    /// Returns Some(transformed_text) if a valid diacritic transformation occurred.
    pub fn apply_accent_to_selection(&self, selection: &str, key: char) -> Option<String> {
        if selection.is_empty() {
            return None;
        }

        let trimmed_trailing = selection.trim_end_matches(char::is_whitespace);
        let trailing_spaces = &selection[trimmed_trailing.len()..];

        if trimmed_trailing.is_empty() {
            return None;
        }

        // Find the last word boundary in trimmed_trailing
        let split_pos = trimmed_trailing
            .rfind(|c: char| !c.is_alphanumeric())
            .map(|idx| idx + trimmed_trailing[idx..].chars().next().unwrap().len_utf8())
            .unwrap_or(0);

        let preceding = &trimmed_trailing[..split_pos];
        let last_word = &trimmed_trailing[split_pos..];

        if last_word.is_empty() {
            return None;
        }

        // Apply key based on configured method
        let (new_word, transformed) = match self.config.method {
            InputMethod::Telex => {
                let (w, tr, _) = self.telex.apply_key(last_word, key, last_word);
                (w, tr)
            }
            InputMethod::Vni => {
                let (w, tr, _) = self.vni.apply_key(last_word, key);
                (w, tr)
            }
            InputMethod::Auto => {
                let (w, tr, _) = self.telex.apply_key(last_word, key, last_word);
                if tr {
                    (w, tr)
                } else if key.is_ascii_digit() {
                    let (wv, trv, _) = self.vni.apply_key(last_word, key);
                    (wv, trv)
                } else {
                    (w, false)
                }
            }
            InputMethod::SimpleTelex => {
                let (w, tr, _) = self.telex.apply_key(last_word, key, last_word);
                (w, tr)
            }
        };

        if transformed && new_word != last_word {
            Some(format!("{}{}{}", preceding, new_word, trailing_spaces))
        } else {
            None
        }
    }

    /// Process a key press event.
    pub fn process_key(&mut self, ch: char, timestamp_ms: f64) -> EngineOutput {
        self.last_completed_word = None;

        // Keystroke Idle Timeout:
        // If the pause between keystrokes exceeds configured timeout, the user was thinking,
        // reading code, or navigating in Normal mode (e.g. Neovim 'u' -> pause -> 'i' -> '1').
        // Finalize/flush any existing buffer so stale navigation keys never leak across paused contexts.
        let timeout_ms = self.config.idle_timeout_ms as f64;
        if timeout_ms > 0.0 && timestamp_ms > 0.0 && self.last_keystroke_time_ms > 0.0 {
            let elapsed = timestamp_ms - self.last_keystroke_time_ms;
            if elapsed >= timeout_ms && !self.current_word.is_empty() {
                self.reset();
            }
        }
        if timestamp_ms > 0.0 {
            self.last_keystroke_time_ms = timestamp_ms;
        }

        let is_bracket = self.config.bracket_transform && matches!(ch, '[' | ']' | '{' | '}');

        // Smart Punctuation Snapping:
        // If the previous action was a macro expansion ending with Space (stage: Expanded),
        // and the incoming key is a punctuation that attaches directly to the word,
        // automatically eat the preceding space and snap the punctuation to the word!
        // Note: ':' is excluded so users can immediately begin a new inline symbol shortcode (e.g. "Việt Nam :smile:").
        if let Some(macro_state) = self.last_macro.take() {
            if macro_state.delimiter == ' '
                && macro_state.stage == MacroUndoStage::Expanded
                && ch != ':'
                && is_snappable_punctuation(ch)
            {
                return EngineOutput {
                    backspaces: 1,
                    commit_text: ch.to_string(),
                    consumed: true,
                    auto_restored: false,
                    macro_expanded: false,
                    adaptive_delay_ms: 0,
                };
            }
            // For any other key, macro undo state is cleared
        }

        // Inline Math Symbols & Emoji Prefix (:smile:, :alpha:, :check:)
        if ch == ':' {
            self.reopen_state = None;
            self.last_delimiter = Some(':');
            self.last_delimiter_spaced = false;

            if self.current_word.is_empty() {
                // Starts an inline shortcode sequence (e.g. :smile)
                self.raw_strokes.push(':');
                self.current_word.push(':');
                self.english_locked = true;
                return EngineOutput {
                    backspaces: 0,
                    commit_text: self.format_commit(":"),
                    consumed: false,
                    auto_restored: false,
                    macro_expanded: false,
                    adaptive_delay_ms: 0,
                };
            } else if self.current_word.starts_with(':') && self.current_word.len() > 1 {
                // Closing colon of an inline shortcode (e.g. ":smile" + ':' -> ":smile:")
                if let Some(symbol) = crate::symbols::lookup_symbol(&self.current_word) {
                    let backspaces = self.current_word.chars().count();
                    let trigger = format!("{}:", self.current_word);
                    self.reset();
                    self.last_macro = Some(LastMacroExpansion {
                        trigger: trigger.clone(),
                        expansion: symbol.to_string(),
                        delimiter: ':',
                        stage: MacroUndoStage::TrailingSpaceRemoved, // Expansion directly replaces trigger without trailing delimiter
                    });
                    self.last_completed_word = Some(symbol.to_string());
                    return EngineOutput {
                        backspaces,
                        commit_text: symbol.to_string(),
                        consumed: true,
                        auto_restored: false,
                        macro_expanded: true,
                        adaptive_delay_ms: 0,
                    };
                }
            }
        }

        // Developer-Aware: Underscore code identifier handling (e.g. _variable, __init__, snake_case)
        if ch == '_' {
            self.reopen_state = None;
            self.last_delimiter = Some('_');
            self.last_delimiter_spaced = false;

            if self.current_word.is_empty() {
                self.raw_strokes.push('_');
                self.current_word.push('_');
                self.english_locked = true;
                return EngineOutput {
                    backspaces: 0,
                    commit_text: self.format_commit("_"),
                    consumed: false,
                    auto_restored: false,
                    macro_expanded: false,
                    adaptive_delay_ms: 0,
                };
            } else {
                // In the middle of an identifier: if prefix had Vietnamese diacritics, revert prefix to raw strokes
                let old_word = self.current_word.clone();
                self.raw_strokes.push('_');
                self.current_word = self.raw_strokes.clone();
                self.english_locked = true;
                let (backspaces, commit_text) = compare_and_split_strings(&old_word, &self.current_word);
                return EngineOutput {
                    backspaces,
                    commit_text: self.format_commit(&commit_text),
                    consumed: true,
                    auto_restored: true,
                    macro_expanded: false,
                    adaptive_delay_ms: 0,
                };
            }
        }

        // Word delimiter check (Space, Enter, Punctuation)
        if !ch.is_alphanumeric() && !is_bracket {
            self.last_delimiter = Some(ch);
            self.last_delimiter_spaced = ch == ' ';

            if !self.current_word.is_empty() {
                // 0. Inline Math Symbol / Emoji expansion on Space (e.g. ":smile " -> "😄 ")
                // Safety Guard: To prevent collision with time formatting (":30 "), URLs (":// "),
                // or code syntax (e.g. "x : int", "for x : in list", ":in", ":or", ":le", ":ge"),
                // Space expansion ONLY triggers for shortcodes with 3+ alphabetic chars,
                // or where all chars after ':' are strictly alphabetic.
                // 2-character keywords (like :in, :or, :le, :ge, :pi, :nu) REQUIRE a closing ':' (e.g. :in:).
                if ch == ' ' && self.current_word.starts_with(':') && self.current_word.len() > 1 {
                    let code_part = &self.current_word[1..];
                    let is_valid_space_shortcode = code_part.len() >= 3
                        && code_part.chars().all(|c| c.is_alphabetic());

                    if is_valid_space_shortcode {
                        if let Some(symbol) = crate::symbols::lookup_symbol(&self.current_word) {
                            let backspaces = self.current_word.chars().count();
                            let trigger = format!("{} ", self.current_word);
                            let commit_text = format!("{} ", symbol);
                            self.reset();
                            self.last_macro = Some(LastMacroExpansion {
                                trigger,
                                expansion: symbol.to_string(),
                                delimiter: ' ',
                                stage: MacroUndoStage::Expanded,
                            });
                            self.last_completed_word = Some(commit_text.clone());
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
                }

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
                            stage: MacroUndoStage::Expanded,
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
                // For words containing VNI digits: only restore if the word violates Vietnamese syllable structure
                // (e.g. intermediate forms like 'duong2' -> 'duòng ' where the user hit Space before adding the 2nd accent).
                // Valid Vietnamese words formed with digits (e.g. gio72 -> giờ, duoc975 -> được) are never restored.
                let has_digits = self.raw_strokes.chars().any(|c| c.is_ascii_digit());
                let is_invalid_syllable = !is_valid_vietnamese_syllable(&self.current_word);
                let is_known_english = self.config.auto_restore_english
                    && !has_digits
                    && self.current_word != self.raw_strokes
                    && self.auto_restore.is_english_keyword(&self.raw_strokes);

                if is_known_english
                    || (!self.manual_undo
                        && (!has_digits || is_invalid_syllable)
                        && has_vietnamese_diacritic(&self.current_word))
                {
                    let learned_restore = !has_digits && self.habit_learner.is_learned_word(&self.raw_strokes);
                    let dict_restore = !has_digits
                        && self.config.enable_dictionary
                        && self.auto_restore.should_restore(
                            &self.raw_strokes,
                            &self.current_word,
                            true,
                        );
                    // Invalid intermediate syllables (duòng, tién, nuóc) MUST auto-restore on Space even if auto_restore_english is false.
                    let english_or_spell_restore = is_known_english
                        || is_invalid_syllable
                        || (self.config.auto_restore_english
                            && !has_digits
                            && (has_phonology_violation(&self.current_word)
                                || (self.config.spell_check
                                    && !is_in_vietnamese_dictionary(&self.current_word))));

                    if learned_restore || dict_restore || english_or_spell_restore {
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

            // Adaptive Habit Learner:
            // 1. If the user finished a word where they manually undid a Telex/VNI mark via double-tap
            //    (e.g. "ww", "ss", "rr", "ff") resulting in an unaccented ASCII word, record the correction!
            // 2. Or if the user previously deleted an accented word via Backspace and just finished
            //    retyping the exact raw/unaccented word, record `BackspaceRetype`!
            if !self.current_word.is_empty() && !has_vietnamese_diacritic(&self.current_word) {
                if self.manual_undo {
                    if self.habit_learner.record_correction(
                        &self.current_word,
                        &self.raw_strokes,
                        CorrectionKind::DoubleKeyUndo,
                        self.last_keystroke_time_ms,
                    ) {
                        self.auto_restore.add_keyword(&self.current_word);
                    }
                } else if let Some((ref cand_raw, ref cand_acc)) = self.deleted_accented_candidate {
                    if self.current_word.eq_ignore_ascii_case(cand_raw)
                        || self
                            .current_word
                            .to_lowercase()
                            .starts_with(&cand_raw.to_lowercase())
                    {
                        let learned_token = self.current_word.clone();
                        let acc_copy = cand_acc.clone();
                        if self.habit_learner.record_correction(
                            &learned_token,
                            &acc_copy,
                            CorrectionKind::BackspaceRetype,
                            self.last_keystroke_time_ms,
                        ) {
                            self.auto_restore.add_keyword(&learned_token);
                        }
                    }
                }
            }
            self.deleted_accented_candidate = None;

            // No macro or auto-restore needed: reset word buffer and let delimiter pass through natively
            let has_word = !self.current_word.is_empty();
            let raw_target = format!("{}{}", self.current_word, ch);
            let formatted_target = self.format_commit(&raw_target);
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
                self.last_completed_word = Some(formatted_target);
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

        // Developer-Aware: Chained dot accessor detection (e.g. "db.collection.findOne")
        // If an alphanumeric key begins immediately after a dot without whitespace, lock English mode
        if self.current_word.is_empty() && self.last_delimiter == Some('.') && !self.last_delimiter_spaced {
            self.english_locked = true;
        }

        // Developer-Aware: camelCase identifier detection (e.g. "parse" + 'S' -> "parseS", "vnch" + 'K' -> "vnchKey")
        // If an uppercase letter arrives after lowercase letters, and does not form a valid Vietnamese
        // syllable continuation or diacritic transform, restore raw prefix if it had diacritics
        // and lock English mode for this identifier.
        let prev_has_lower = self.raw_strokes.chars().any(|c| c.is_ascii_lowercase());
        let can_transform_vietnamese = match self.config.method {
            InputMethod::Telex | InputMethod::SimpleTelex => {
                let (_, tr, _) = self.telex.apply_key(&self.current_word, ch, &self.raw_strokes);
                tr
            }
            InputMethod::Vni => {
                let (_, tr, _) = self.vni.apply_key(&self.current_word, ch);
                tr
            }
            InputMethod::Auto => {
                let (_, tr_t, _) = self.telex.apply_key(&self.current_word, ch, &self.raw_strokes);
                let (_, tr_v, _) = self.vni.apply_key(&self.current_word, ch);
                tr_t || tr_v
            }
        };

        let test_chars: Vec<char> = format!("{}{}", self.current_word, ch.to_ascii_lowercase()).chars().collect();
        let is_valid_viet_continuation = is_valid_in_progress_syllable(&test_chars);

        if prev_has_lower && ch.is_ascii_uppercase() && !can_transform_vietnamese && !is_valid_viet_continuation {
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
        // Note: If we are actively tracking an inline symbol shortcode starting with ':' (e.g. :100:),
        // let the digit append to the shortcode buffer without resetting.
        if ch.is_ascii_digit() && !self.current_word.starts_with(':') {
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
                                    commit_text: self.format_commit(&commit_text),
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
                        commit_text: self.format_commit(&commit_text),
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
                                    commit_text: self.format_commit(&commit_text),
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
                        commit_text: self.format_commit(&commit_text),
                        consumed,
                        auto_restored: false,
                        macro_expanded: false,
                        adaptive_delay_ms: 0,
                    };
                }
            }
        }

        // Non-digit character (letters and brackets)
        // 0. Quick Consonants pre-transform:
        // Quick Initial: f -> ph, j -> gi, w -> qu (if at start of word)
        if self.config.quick_start_consonant && self.current_word.is_empty() {
            let ch_lower = ch.to_ascii_lowercase();
            let is_upper = ch.is_ascii_uppercase();
            let replacement = match ch_lower {
                'f' => Some(if is_upper { "Ph" } else { "ph" }),
                'j' => Some(if is_upper { "Gi" } else { "gi" }),
                'w' => Some(if is_upper { "Qu" } else { "qu" }),
                _ => None,
            };
            if let Some(rep) = replacement {
                self.raw_strokes.push(ch);
                self.current_word.push_str(rep);
                return EngineOutput {
                    backspaces: 0,
                    commit_text: self.format_commit(rep),
                    consumed: true,
                    auto_restored: false,
                    macro_expanded: false,
                    adaptive_delay_ms: 0,
                };
            }
        }

        // Quick Telex Consonants (cc->ch, gg->gi, kk->kh, nn->ng, pp->ph, qq->qu, tt->th)
        if self.config.quick_telex && matches!(self.config.method, InputMethod::Telex | InputMethod::Auto | InputMethod::SimpleTelex) {
            if let Some(&last_ch) = self.current_word.chars().collect::<Vec<char>>().last() {
                if last_ch.to_ascii_lowercase() == ch.to_ascii_lowercase() && ch.is_ascii_alphabetic() {
                    let is_upper = last_ch.is_ascii_uppercase() || ch.is_ascii_uppercase();
                    let pair_rep = match ch.to_ascii_lowercase() {
                        'c' => Some(if is_upper { "Ch" } else { "ch" }),
                        'g' => Some(if is_upper { "Gi" } else { "gi" }),
                        'k' => Some(if is_upper { "Kh" } else { "kh" }),
                        'n' => Some(if is_upper { "Ng" } else { "ng" }),
                        'p' => Some(if is_upper { "Ph" } else { "ph" }),
                        'q' => Some(if is_upper { "Qu" } else { "qu" }),
                        't' => Some(if is_upper { "Th" } else { "th" }),
                        _ => None,
                    };
                    if let Some(rep) = pair_rep {
                        let old_word = self.current_word.clone();
                        self.current_word.pop();
                        self.current_word.push_str(rep);
                        self.raw_strokes.push(ch);
                        let (backspaces, commit_text) = compare_and_split_strings(&old_word, &self.current_word);
                        return EngineOutput {
                            backspaces,
                            commit_text: self.format_commit(&commit_text),
                            consumed: true,
                            auto_restored: false,
                            macro_expanded: false,
                            adaptive_delay_ms: 0,
                        };
                    }
                }
            }
        }

        // Quick Final Consonants: g->ng, h->nh, k->ch (after a vowel)
        if self.config.quick_end_consonant && matches!(self.config.method, InputMethod::Telex | InputMethod::Auto) {
            let has_vowel = self.current_word.chars().any(|c| crate::telex::is_vietnamese_vowel(c));
            if has_vowel {
                if let Some(&last_ch) = self.current_word.chars().collect::<Vec<char>>().last() {
                    if crate::telex::is_vietnamese_vowel(last_ch) {
                        let is_upper = ch.is_ascii_uppercase();
                        let final_rep = match ch.to_ascii_lowercase() {
                            'g' => Some(if is_upper { "NG" } else { "ng" }),
                            'h' => Some(if is_upper { "NH" } else { "nh" }),
                            'k' => Some(if is_upper { "CH" } else { "ch" }),
                            _ => None,
                        };
                        if let Some(rep) = final_rep {
                            let old_word = self.current_word.clone();
                            self.current_word.push_str(rep);
                            self.raw_strokes.push(ch);
                            let (backspaces, commit_text) = compare_and_split_strings(&old_word, &self.current_word);
                            return EngineOutput {
                                backspaces,
                                commit_text: self.format_commit(&commit_text),
                                consumed: true,
                                auto_restored: false,
                                macro_expanded: false,
                                adaptive_delay_ms: 0,
                            };
                        }
                    }
                }
            }
        }

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
        let learned_match = !is_vni_word
            && has_diacritic
            && self.habit_learner.is_learned_word(&self.raw_strokes);
        let dict_match = !is_vni_word
            && has_diacritic
            && self.config.enable_dictionary
            && self
                .auto_restore
                .should_restore(&self.raw_strokes, &new_word, true);

        if !self.manual_undo
            && !is_vni_word
            && ((self.config.auto_restore_english && invalid_structure)
                || dict_match
                || learned_match)
        {
            new_word = self.raw_strokes.clone();
            self.english_locked = true;
            auto_restored = true;
        } else if self.config.spell_check && !self.manual_undo && was_transformed && invalid_structure
        {
            // If Chánh Tả is enabled (even when Restore English is disabled), prevent invalid Vietnamese syllable marks
            new_word = format!("{}{}", self.current_word, ch);
        }

        // Optional Initial Case Anchor (user configurable, default: false)
        // When enabled by user: ĐƯợc/ĐƯỢc -> Được, đƯỢc -> được.
        if self.config.initial_case_anchor && !new_word.is_empty() && !self.english_locked && !auto_restored {
            new_word = crate::casing::harmonize_syllable_case(&new_word);
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
            commit_text: self.format_commit(&commit_text),
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
        assert_eq!(type_word(&mut engine, "as "), "as ");
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
        let mut engine = Engine::new(EngineConfig::default());

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
}
