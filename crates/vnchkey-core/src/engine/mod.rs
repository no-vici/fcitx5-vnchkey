//! Syllable State Machine and Input Orchestrator.

use crate::auto_restore::EnglishAutoRestore;
use crate::casing::CaseTracker;
use crate::habit_learner::{CorrectionKind, HabitLearner};
use crate::macros::MacroTable;
pub use crate::telex::W2UType;
use crate::telex::{
    extract_last_syllable_index, has_vietnamese_diacritic, is_valid_elongated_vietnamese_word,
    is_valid_in_progress_syllable, is_valid_vietnamese_syllable, TelexEngine,
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
    /// Hiển thị gợi ý macro (preedit hint) khi gõ trigger macro. Mặc định TẮT.
    pub macro_preview: bool,
    /// Idle keystroke timeout in milliseconds (0 = disabled, default = 0ms)
    pub idle_timeout_ms: u64,
    /// Hardware key debounce filter in milliseconds (0 = disabled, default = 0ms)
    pub debounce_ms: u64,
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
    /// Automatically capitalize the first letter of a new sentence (after `.`, `?`, `!`, `\n`). Defaults to false.
    pub auto_capitalize: bool,
    /// Xóa toàn bộ âm tiết khi nhấn Backspace (thay vì từng ký tự). Mặc định TẮT.
    pub backspace_by_syllable: bool,
    /// Automatically complete missing circumflex/horn on unambiguous closed syllables (e.g. `truongf` -> `trường`). Defaults to false.
    pub smart_fuzzy_vowel: bool,
    /// Protect ALL-CAPS acronyms (2..=5 chars, e.g. OS, IOS, VPS, AWS) from Telex tone distortion. Defaults to true.
    pub acronym_bypass: bool,
    /// Developer Mode: automatically bypass Vietnamese diacritics and macro expansion inside Markdown inline (`` `...` ``) and fenced (```` ```...``` ````) code blocks. Defaults to true.
    pub developer_mode: bool,
    /// Allow typing Vietnamese inside Markdown backticks (`` ` ``) and fenced code blocks (```` ``` ````).
    pub allow_vietnamese_in_backtick: bool,
    /// Auto-space after sentence-ending punctuation (`.`, `?`, `!`) when the previous char is a letter. Defaults to false (opt-in).
    pub auto_space_after_punctuation: bool,
    /// Auto-capitalize proper nouns from a curated list (e.g. `Việt Nam`, `Hà Nội`, `Sài Gòn`, `OpenAI`, `GitHub`). Defaults to false (opt-in).
    pub auto_capitalize_proper_nouns: bool,
    /// Danh sách từ riêng do user định nghĩa (chữ thường). Danh sách rỗng nghĩa tính năng tắt.
    /// Ví dụ: ["hanoi", "openai", "github", "sai gon", "university"].
    /// List này được tải từ JSON config (vnchkey-settings) và cập nhật động qua Settings UI.
    pub proper_nouns_list: Vec<String>,
    /// Sentence Momentum: preserve English for ambiguous collision words after English anchor words (e.g. `the tree`, `he must see this room`). Defaults to false (opt-in).
    pub sentence_momentum: bool,
    /// Math Context: preserve English for ambiguous collision words after math function anchors (e.g. `sin and cos`, `tan or cos`). Defaults to false (opt-in).
    pub math_context: bool,
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
            debounce_ms: 0,
            quick_telex: false,
            quick_start_consonant: false,
            quick_end_consonant: false,
            habit_learning_enabled: false,
            habit_promote_threshold: 0,
            initial_case_anchor: false,
            auto_capitalize: false,
            smart_fuzzy_vowel: false,
            acronym_bypass: true,
            developer_mode: true,
            allow_vietnamese_in_backtick: false,
            auto_space_after_punctuation: false,
            auto_capitalize_proper_nouns: false,
            proper_nouns_list: vec![],
            backspace_by_syllable: false,
            sentence_momentum: false,
            math_context: false,
            macro_preview: false,
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

/// Offline typing statistics tracked by the engine.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TypingStats {
    /// Total Vietnamese words committed
    pub words_typed: u64,
    /// Total times Smart English Restore rescued an English word
    pub english_restored: u64,
    /// Total macro expansions triggered
    pub macros_expanded: u64,
    /// Total keystrokes saved by macro expansion
    pub keystrokes_saved: u64,
}

impl TypingStats {
    /// Serializes stats in compact `key=value` format for persistent storage.
    pub fn serialize_compact(&self) -> String {
        format!(
            "words_typed={}\nenglish_restored={}\nmacros_expanded={}\nkeystrokes_saved={}\n",
            self.words_typed, self.english_restored, self.macros_expanded, self.keystrokes_saved
        )
    }

    /// Parses stats from compact `key=value` format.
    pub fn parse_compact(content: &str) -> Self {
        let mut st = Self::default();
        for line in content.lines() {
            if let Some((k, v)) = line.trim().split_once('=') {
                let val = v.trim().parse::<u64>().unwrap_or(0);
                match k.trim() {
                    "words_typed" => st.words_typed = val,
                    "english_restored" => st.english_restored = val,
                    "macros_expanded" => st.macros_expanded = val,
                    "keystrokes_saved" => st.keystrokes_saved = val,
                    _ => {}
                }
            }
        }
        st
    }
}

/// Thử suy luận dấu mũ/móc bị thiếu trên các vần trung gian duy nhất của tiếng Việt
/// khi người dùng bật `smart_fuzzy_vowel`.
fn try_resolve_fuzzy_intermediate_vowel(word: &str, tone_style: ToneStyle) -> Option<String> {
    let mut parsed = crate::syllable_parser::parse_syllable(word)?;
    if parsed.tone == crate::syllable_parser::ToneMark::None {
        return None;
    }

    let nuc_lower = parsed.nucleus.to_lowercase();
    // Open syllables like `nguòi` (người) have no coda; allow them only for the
    // unambiguous `uoi` -> `ươi` promotion. Closed syllables keep the original rule.
    let coda_lower = parsed.coda.to_lowercase();
    let allow_open = nuc_lower == "uoi";
    if parsed.coda.is_empty() && !allow_open {
        return None;
    }

    let is_upper_second = parsed.nucleus.chars().nth(1).map(|c| c.is_uppercase()).unwrap_or(false);
    let is_upper_first = parsed.nucleus.chars().next().map(|c| c.is_uppercase()).unwrap_or(false);
    let is_upper_third = parsed.nucleus.chars().nth(2).map(|c| c.is_uppercase()).unwrap_or(false);

    let resolved_nucleus = match nuc_lower.as_str() {
        "uo" if matches!(coda_lower.as_str(), "c" | "ch" | "i" | "m" | "n" | "ng" | "p" | "t") => {
            let u_ch = if is_upper_first { 'Ư' } else { 'ư' };
            let o_ch = if is_upper_second { 'Ơ' } else { 'ơ' };
            format!("{}{}", u_ch, o_ch)
        }
        "uoi" => {
            let u_ch = if is_upper_first { 'Ư' } else { 'ư' };
            let o_ch = if is_upper_second { 'Ơ' } else { 'ơ' };
            let i_ch = parsed.nucleus.chars().nth(2).unwrap_or('i');
            let i_out = if is_upper_third { i_ch.to_uppercase().next().unwrap_or(i_ch) } else { i_ch.to_lowercase().next().unwrap_or(i_ch) };
            format!("{}{}{}", u_ch, o_ch, i_out)
        }
        "uye" => {
            let u_ch = parsed.nucleus.chars().next().unwrap_or('u');
            let y_ch = parsed.nucleus.chars().nth(1).unwrap_or('y');
            let e_ch = if is_upper_third { 'Ê' } else { 'ê' };
            format!("{}{}{}", u_ch, y_ch, e_ch)
        }
        "ye" => {
            let y_ch = parsed.nucleus.chars().next().unwrap_or('y');
            let e_ch = if is_upper_second { 'Ê' } else { 'ê' };
            format!("{}{}", y_ch, e_ch)
        }
        "ie" => {
            let i_ch = parsed.nucleus.chars().next().unwrap_or('i');
            let e_ch = if is_upper_second { 'Ê' } else { 'ê' };
            format!("{}{}", i_ch, e_ch)
        }
        _ => return None,
    };

    parsed.nucleus = resolved_nucleus;
    let candidate = parsed.reconstruct(tone_style);
    if is_in_vietnamese_dictionary(&candidate) {
        Some(candidate)
    } else {
        None
    }
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
    user_vietnamese_words: std::collections::HashSet<String>,
    current_word: String,
    raw_strokes: String,
    word_used_telex: bool,
    word_used_vni: bool,
    has_letter_after_digit: bool,
    sentence_start_pending: bool,
    manual_undo: bool,
    english_locked: bool,
    last_completed_word: Option<String>,
    last_macro: Option<LastMacroExpansion>,
    pending_macro_preview: Option<PendingMacroPreview>,
    suppressed_macro: Option<String>,
    reopen_history: std::collections::VecDeque<ReopenWordState>,
    deleted_accented_candidate: Option<(String, String)>,
    last_delimiter: Option<char>,
    last_delimiter_spaced: bool,
    preceded_by_tech_delimiter: bool,
    preceded_by_leading_slash: bool,
    dev_code_state: DevCodeState,
    in_english_context: bool,
    in_vietnamese_context: bool,
    math_context_words_remaining: u8,
    last_keystroke_time_ms: f64,
    last_char: Option<char>,
    prev_word_was_english: bool,
    stats: TypingStats,
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

/// Preview state for an inline macro expansion hint.
#[derive(Debug, Clone)]
pub struct PendingMacroPreview {
    /// The trigger keystroke that started the preview (e.g. "vn")
    pub trigger: String,
    /// The expansion that will be shown as hint (e.g. "Việt Nam")
    pub expansion: String,
}

/// Returns whether a punctuation mark should snap directly to the preceding word
/// without a leading space (e.g. ',', '.', '!', '?', ';', ':', ')', ']', '}', '>').
#[inline]
pub fn is_snappable_punctuation(ch: char) -> bool {
    matches!(ch, ',' | '.' | '!' | '?' | ';' | ':' | ')' | ']' | '}' | '>')
}

/// Returns `true` when `ch` is a technical delimiter (URL, path, email, variable, code syntax)
/// that should suppress shorthand macro expansion immediately before or after a token
/// (e.g. `google.com.vn`, `https://vn`, `user@vn`, `user_vn`, `dc()`).
#[inline]
pub fn is_macro_technical_delimiter(ch: char) -> bool {
    ch.is_ascii_digit()
        || matches!(
            ch,
            '.' | ':' | '/' | '@' | '_' | '-' | '#' | '\\' | '=' | '$' | '~' | '`'
        )
}

/// Returns `true` when shorthand macro expansion must be suppressed due to technical context.
#[inline]
pub fn should_suppress_macro_expansion(
    preceded_by_tech_delimiter: bool,
    boundary_char: char,
) -> bool {
    if preceded_by_tech_delimiter {
        return true;
    }
    is_macro_technical_delimiter(boundary_char) || matches!(boundary_char, '(' | '[' | '{' | '<')
}

/// Returns `true` if `raw_lower` is a common CLI/chat slash command or Unix path component
/// that must remain raw English when immediately preceded by a leading `/` (e.g. `/compact`, `/aws`, `/os`, `/usr`).
pub fn is_leading_slash_command_or_path_token(raw_lower: &str) -> bool {
    matches!(
        raw_lower,
        "add"
            | "agents"
            | "api"
            | "app"
            | "apps"
            | "ar"
            | "as"
            | "aw"
            | "awk"
            | "aws"
            | "bash"
            | "bin"
            | "boost"
            | "boot"
            | "browser"
            | "bug"
            | "build"
            | "cast"
            | "cash"
            | "clear"
            | "compact"
            | "config"
            | "core"
            | "cost"
            | "dash"
            | "dev"
            | "diff"
            | "dist"
            | "doc"
            | "docs"
            | "doctor"
            | "down"
            | "env"
            | "es"
            | "etc"
            | "ex"
            | "exit"
            | "fast"
            | "git"
            | "goal"
            | "grill"
            | "hash"
            | "help"
            | "home"
            | "host"
            | "if"
            | "image"
            | "info"
            | "init"
            | "ir"
            | "is"
            | "ix"
            | "last"
            | "learn"
            | "lib"
            | "list"
            | "log"
            | "login"
            | "logout"
            | "logs"
            | "loop"
            | "lost"
            | "mask"
            | "mcp"
            | "memory"
            | "mnt"
            | "model"
            | "models"
            | "more"
            | "most"
            | "new"
            | "of"
            | "opt"
            | "or"
            | "os"
            | "out"
            | "permissions"
            | "pist"
            | "pkg"
            | "plan"
            | "plugin"
            | "port"
            | "post"
            | "pr"
            | "proc"
            | "quit"
            | "raw"
            | "reset"
            | "review"
            | "room"
            | "root"
            | "run"
            | "sbin"
            | "schedule"
            | "search"
            | "share"
            | "sort"
            | "src"
            | "srv"
            | "ssh"
            | "start"
            | "status"
            | "stop"
            | "store"
            | "sys"
            | "task"
            | "tasks"
            | "teamwork"
            | "terminal"
            | "test"
            | "tests"
            | "tmp"
            | "tool"
            | "tools"
            | "ur"
            | "us"
            | "user"
            | "users"
            | "usr"
            | "var"
            | "vim"
            | "voice"
            | "war"
            | "was"
            | "web"
            | "zoom"
    )
}

/// Finite State Machine tracking Markdown inline (`` `...` ``) and fenced (```` ```...``` ````) code blocks.
mod dev_code;
pub use dev_code::{
    is_context_english_collision_word, is_english_context_anchor, is_math_function_anchor,
    is_vietnamese_context_reset, to_composite_unicode, DevCodeState,
};

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
            user_vietnamese_words: std::collections::HashSet::new(),
            config,
            current_word: String::with_capacity(32),
            raw_strokes: String::with_capacity(32),
            word_used_telex: false,
            word_used_vni: false,
            has_letter_after_digit: false,
            sentence_start_pending: false,
            manual_undo: false,
            english_locked: false,
            last_completed_word: None,
            last_macro: None,
            pending_macro_preview: None,
            suppressed_macro: None,
            reopen_history: std::collections::VecDeque::with_capacity(16),
            deleted_accented_candidate: None,
            last_delimiter: None,
            last_delimiter_spaced: false,
            preceded_by_tech_delimiter: false,
            preceded_by_leading_slash: false,
            dev_code_state: DevCodeState::default(),
            in_english_context: false,
            in_vietnamese_context: false,
            math_context_words_remaining: 0,
            last_keystroke_time_ms: 0.0,
            last_char: None,
            prev_word_was_english: false,
            stats: TypingStats::default(),
        }
    }

    /// Returns offline typing statistics for the current session.
    pub fn stats(&self) -> TypingStats {
        self.stats
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
        self.config.habit_promote_threshold = threshold;
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

    /// Updates cached clipboard text for `$CLIPBOARD` macro expansion.
    pub fn set_clipboard_text(&mut self, text: impl Into<String>) {
        self.macro_table.set_clipboard_text(text);
    }

    /// Enables or disables automatic sentence-start capitalization.
    pub fn set_auto_capitalize(&mut self, enabled: bool) {
        self.config.auto_capitalize = enabled;
    }

    /// Returns whether automatic sentence-start capitalization is enabled.
    pub fn auto_capitalize(&self) -> bool {
        self.config.auto_capitalize
    }

    /// Replaces the user's personal Vietnamese dictionary (slang/custom words).
    /// Words in this set are treated as valid Vietnamese even when `spell_check` is enabled.
    pub fn set_user_vietnamese_words(&mut self, words: Vec<String>) {
        self.user_vietnamese_words = words
            .into_iter()
            .map(|w| w.trim().to_lowercase())
            .filter(|w| !w.is_empty())
            .collect();
    }

    /// Returns the sorted list of personal Vietnamese dictionary words.
    pub fn user_vietnamese_words(&self) -> Vec<String> {
        let mut words: Vec<String> = self.user_vietnamese_words.iter().cloned().collect();
        words.sort();
        words
    }

    /// Returns true if `word` (case-insensitive) is in the user's personal Vietnamese dictionary.
    pub fn is_user_vietnamese_word(&self, word: &str) -> bool {
        !self.user_vietnamese_words.is_empty()
            && self.user_vietnamese_words.contains(&word.to_lowercase())
    }

    /// Returns true if `word` is in either the built-in Vietnamese dictionary or the user's personal Vietnamese dictionary.
    fn is_in_any_vietnamese_dictionary(&self, word: &str) -> bool {
        is_in_vietnamese_dictionary(word) || self.is_user_vietnamese_word(word)
    }

    /// Enables or disables smart fuzzy vowel completion on Space.
    pub fn set_smart_fuzzy_vowel(&mut self, enabled: bool) {
        self.config.smart_fuzzy_vowel = enabled;
    }

    /// Enables or disables ALL-CAPS acronym bypass protection.
    pub fn set_acronym_bypass(&mut self, enabled: bool) {
        self.config.acronym_bypass = enabled;
    }

    /// Enables or disables Developer Mode (Markdown inline `` `...` `` & fenced ```` ```...``` ```` code bypass).
    pub fn set_developer_mode(&mut self, enabled: bool) {
        self.config.developer_mode = enabled;
        if !enabled {
            self.dev_code_state.reset();
        }
    }

    /// Returns whether Developer Mode is enabled.
    pub fn developer_mode(&self) -> bool {
        self.config.developer_mode
    }

    /// Enables or disables typing Vietnamese inside Markdown backticks and fenced code blocks.
    pub fn set_allow_vietnamese_in_backtick(&mut self, enabled: bool) {
        self.config.allow_vietnamese_in_backtick = enabled;
        if enabled {
            self.dev_code_state.reset();
        }
    }

    /// Returns whether typing Vietnamese inside Markdown backticks is allowed.
    pub fn allow_vietnamese_in_backtick(&self) -> bool {
        self.config.allow_vietnamese_in_backtick
    }

    /// Enables or disables Auto-space after sentence-ending punctuation.
    pub fn set_auto_space_after_punctuation(&mut self, enabled: bool) {
        self.config.auto_space_after_punctuation = enabled;
    }

    /// Returns whether Auto-space after sentence-ending punctuation is enabled.
    pub fn auto_space_after_punctuation(&self) -> bool {
        self.config.auto_space_after_punctuation
    }

    /// Enables or disables Auto-capitalize proper nouns from a curated list.
    pub fn set_auto_capitalize_proper_nouns(&mut self, enabled: bool) {
        self.config.auto_capitalize_proper_nouns = enabled;
    }

    /// Returns whether Auto-capitalize proper nouns is enabled.
    pub fn auto_capitalize_proper_nouns(&self) -> bool {
        self.config.auto_capitalize_proper_nouns
    }

    /// Enables or disables Macro preview (preedit hint when typing macro trigger).
    pub fn set_macro_preview(&mut self, enabled: bool) {
        self.config.macro_preview = enabled;
    }

    /// Returns whether Macro preview is enabled.
    pub fn macro_preview(&self) -> bool {
        self.config.macro_preview
    }

    /// Enables or disables Backspace xóa cả âm tiết.
    pub fn set_backspace_by_syllable(&mut self, enabled: bool) {
        self.config.backspace_by_syllable = enabled;
    }

    /// Returns whether Backspace xóa cả âm tiết là được kích hoạt.
    pub fn backspace_by_syllable(&self) -> bool {
        self.config.backspace_by_syllable
    }

    /// Enables or disables Sentence Momentum (preserve English for ambiguous collision words after English anchor words).
    pub fn set_sentence_momentum(&mut self, enabled: bool) {
        self.config.sentence_momentum = enabled;
        if !enabled {
            self.in_english_context = false;
            self.prev_word_was_english = false;
        }
    }

    /// Returns whether Sentence Momentum is enabled.
    pub fn sentence_momentum(&self) -> bool {
        self.config.sentence_momentum
    }

    /// Enables or disables Math Context (preserve English for ambiguous collision words after math function anchors).
    pub fn set_math_context(&mut self, enabled: bool) {
        self.config.math_context = enabled;
        if !enabled {
            self.math_context_words_remaining = 0;
        }
    }

    /// Returns whether Math Context is enabled.
    pub fn math_context(&self) -> bool {
        self.config.math_context
    }

    /// Returns a compact live-inspector snapshot of the current engine state and syllable breakdown.
    pub fn inspect_state(&self) -> String {
        let syllable_info = if let Some(p) = crate::syllable_parser::parse_syllable(&self.current_word) {
            format!(
                "onset={:?}, glide={:?}, nucleus={:?}, coda={:?}, tone={:?}",
                p.onset, p.glide, p.nucleus, p.coda, p.tone
            )
        } else {
            "unparsed".to_string()
        };
        format!(
            "raw={:?} | word={:?} | locked={} | undo={} | prev_en={} | reopen_buf={} | {}",
            self.raw_strokes,
            self.current_word,
            self.english_locked,
            self.manual_undo,
            self.prev_word_was_english,
            self.reopen_history.len(),
            syllable_info
        )
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
        self.has_letter_after_digit = false;
        self.manual_undo = false;
        self.english_locked = false;
        self.last_completed_word = None;
        self.suppressed_macro = None;
    }

    /// Resets all internal buffers (e.g. on window unfocus or navigation key).
    pub fn reset(&mut self) {
        self.reset_word_buffers();
        self.reopen_history.clear();
        self.last_keystroke_time_ms = 0.0;
        self.last_char = None;
        self.prev_word_was_english = false;
        self.last_delimiter = None;
        self.last_delimiter_spaced = false;
        self.preceded_by_tech_delimiter = false;
        self.preceded_by_leading_slash = false;
        self.dev_code_state.reset();
        self.in_english_context = false;
        self.in_vietnamese_context = false;
        self.math_context_words_remaining = 0;
    }

    fn update_sentence_context_after_word(
        &mut self,
        raw_lower: &str,
        final_word: &str,
        was_english_restore: bool,
        delimiter: char,
    ) {
        if matches!(delimiter, '.' | '!' | '?' | '\n' | '\r') {
            self.in_english_context = false;
            self.in_vietnamese_context = false;
            self.math_context_words_remaining = 0;
            self.prev_word_was_english = false;
            return;
        }
        let has_non_ascii = !final_word.is_ascii();
        let final_lower = final_word.to_lowercase();
        let is_anchor = !has_non_ascii && is_english_context_anchor(raw_lower);

        if self.config.math_context {
            if !has_non_ascii && is_math_function_anchor(raw_lower) {
                self.math_context_words_remaining = 4;
            } else if has_non_ascii {
                self.math_context_words_remaining = 0;
            } else if self.math_context_words_remaining > 0 {
                let is_joiner = matches!(raw_lower, "and" | "or" | "plus" | "minus");
                if is_joiner {
                    self.math_context_words_remaining -= 1;
                } else {
                    self.math_context_words_remaining = 0;
                }
            }
        } else {
            self.math_context_words_remaining = 0;
        }

        if has_non_ascii || is_vietnamese_context_reset(&final_lower) {
            self.in_english_context = false;
            self.in_vietnamese_context = true;
            self.prev_word_was_english = false;
        } else if is_anchor && self.config.sentence_momentum {
            self.in_english_context = true;
            self.in_vietnamese_context = false;
            self.prev_word_was_english = true;
        } else if !self.config.sentence_momentum {
            self.in_english_context = false;
            self.prev_word_was_english = was_english_restore || self.auto_restore.is_english_keyword(raw_lower);
        } else { self.prev_word_was_english = was_english_restore || self.auto_restore.is_english_keyword(raw_lower); }
    }

    /// Handles physical Backspace pressed by the user.
    /// If Backspace is pressed immediately after completing a word with a single Space,
    /// reopens the previous word so marks/tones can be applied (Dokey style).
    /// Returns true if a previously completed word was reopened.
    pub fn handle_backspace(&mut self) -> bool {
        if self.current_word.is_empty() {
            if let Some(saved) = self.reopen_history.pop_back() {
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
        } else if self.config.backspace_by_syllable {
            // Xóa toàn bộ âm tiết đang gõ khi cờ bật
            let was_accented = has_vietnamese_diacritic(&self.current_word)
                && self.raw_strokes != self.current_word
                && self.raw_strokes.len() >= 2;
            let cand_to_save = if was_accented && self.deleted_accented_candidate.is_none() {
                Some((self.raw_strokes.clone(), self.current_word.clone()))
            } else {
                None
            };

            self.current_word.clear();
            self.raw_strokes.clear();
            // Arm deleted_accented_candidate for BackspaceRetype learning only if the word was accented
            if let Some(cand) = cand_to_save {
                self.deleted_accented_candidate = Some(cand);
            }
            self.reset_word_buffers();
            false
        } else {
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
                if self.english_locked
                    && !self.current_word.chars().any(|c| !c.is_alphabetic())
                {
                    self.english_locked = false;
                }
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

        let formatted_old = self.format_commit(&old_word);
        let formatted_new = self.format_commit(&self.current_word);
        let (backspaces, commit_text) = compare_and_split_strings(&formatted_old, &formatted_new);
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

        let formatted_old = self.format_commit(&old_word);
        let formatted_new = self.format_commit(&new_word);
        let (backspaces, commit_text) = compare_and_split_strings(&formatted_old, &formatted_new);
        Some(EngineOutput {
            backspaces,
            commit_text,
            consumed: true,
            auto_restored: false,
            macro_expanded: false,
            adaptive_delay_ms: 0,
        })
    }

    /// Cycles the casing of the currently composing word through:
    /// lowercase -> UPPERCASE -> Title Case -> lowercase.
    /// Returns the minimal suffix replacement (backspaces and commit_text) to update the document.
    pub fn cycle_current_word_case(&mut self) -> Option<EngineOutput> {
        if self.current_word.is_empty() {
            return None;
        }

        let old_word = self.current_word.clone();
        let new_word = crate::casing::cycle_text_casing(&old_word);
        if new_word == old_word {
            return None;
        }

        self.current_word = new_word.clone();
        // Also sync casing to raw_strokes where applicable
        let raw_cased = crate::casing::cycle_text_casing(&self.raw_strokes);
        self.raw_strokes = raw_cased;

        let formatted_old = self.format_commit(&old_word);
        let formatted_new = self.format_commit(&new_word);
        let (backspaces, commit_text) = compare_and_split_strings(&formatted_old, &formatted_new);
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
                let exp_len = self.format_commit(&macro_state.expansion).chars().count();
                let delete_count = if macro_state.stage == MacroUndoStage::TrailingSpaceRemoved {
                    exp_len
                } else {
                    1 + exp_len
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
            Some(self.format_commit(&output))
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
            Some(self.format_commit(&format!("{}{}{}", preceding, new_word, trailing_spaces)))
        } else {
            None
        }
    }

    fn process_vni_digit(&mut self, ch: char, auto_cap_consumed: bool) -> EngineOutput {
        if self.current_word.is_empty() {
            self.reset();
            self.raw_strokes.push(ch);
            self.current_word.push(ch);
            self.english_locked = true;
            return EngineOutput {
                backspaces: 0,
                commit_text: ch.to_string(),
                consumed: false,
                auto_restored: false,
                macro_expanded: false,
                adaptive_delay_ms: 0,
            };
        }
        let (new_word, was_transformed, was_undo) = self.vni.apply_key(&self.current_word, ch);
        if was_transformed
            && !was_undo
            && !matches!(ch, '6' | '7' | '8' | '9')
            && (has_phonology_violation(&new_word)
                || new_word.to_lowercase().starts_with("sh"))
        {
            self.english_locked = true;
            self.raw_strokes.push(ch);
            self.current_word.push(ch);
            return EngineOutput {
                backspaces: 0,
                commit_text: self.format_commit(&ch.to_string()),
                consumed: false,
                auto_restored: false,
                macro_expanded: false,
                adaptive_delay_ms: 0,
            };
        }
        if !was_transformed {
            let split_idx = extract_last_syllable_index(&self.current_word);
            let char_count = self.current_word.chars().count();
            if split_idx > 0 && split_idx < char_count {
                let tail: String = self.current_word.chars().skip(split_idx).collect();
                let (new_tail, tail_transformed, tail_undo) = self.vni.apply_key(&tail, ch);
                if tail_transformed {
                    if !self.word_used_vni
                        && ch != '9'
                        && (self.auto_restore.is_english_keyword(&self.current_word)
                            || self.current_word.to_lowercase().starts_with("sh"))
                    {
                        self.english_locked = true;
                        self.raw_strokes.push(ch);
                        self.current_word.push(ch);
                        return EngineOutput {
                            backspaces: 0,
                            commit_text: self.format_commit(&ch.to_string()),
                            consumed: false,
                            auto_restored: false,
                            macro_expanded: false,
                            adaptive_delay_ms: 0,
                        };
                    }
                    self.word_used_vni = true;
                    self.english_locked = false;
                    self.manual_undo |= tail_undo;
                    self.raw_strokes = format!("{}{}", tail, ch);
                    let formatted_old = self.format_commit(&tail);
                    let formatted_new = self.format_commit(&new_tail);
                    let (backspaces, commit_text) =
                        compare_and_split_strings(&formatted_old, &formatted_new);
                    self.current_word = new_tail;
                    let consumed =
                        backspaces > 0 || commit_text != ch.to_string() || auto_cap_consumed;
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
            self.english_locked = true;
            self.raw_strokes.push(ch);
            self.current_word.push(ch);
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
        let formatted_old = self.format_commit(&self.current_word);
        let formatted_new = self.format_commit(&new_word);
        let (backspaces, commit_text) = compare_and_split_strings(&formatted_old, &formatted_new);
        self.current_word = new_word;
        let consumed = backspaces > 0 || commit_text != ch.to_string() || auto_cap_consumed;
        EngineOutput {
            backspaces,
            commit_text,
            consumed,
            auto_restored: false,
            macro_expanded: false,
            adaptive_delay_ms: 0,
        }
    }

    /// Process a key press event.
    pub fn process_key(&mut self, mut ch: char, timestamp_ms: f64) -> EngineOutput {
        self.last_completed_word = None;

        // Keystroke Idle Timeout:
        // If the pause between keystrokes exceeds configured timeout, the user was thinking,
        // reading code, or navigating in Normal mode (e.g. Neovim 'u' -> pause -> 'i' -> '1').
        // Finalize/flush any existing buffer so stale navigation keys never leak across paused contexts.
        if self.config.debounce_ms > 0
            && timestamp_ms > 0.0
            && self.last_keystroke_time_ms > 0.0
            && self.last_char == Some(ch)
            && (timestamp_ms - self.last_keystroke_time_ms) < (self.config.debounce_ms as f64)
        {
            return EngineOutput {
                backspaces: 0,
                commit_text: String::new(),
                consumed: true,
                auto_restored: false,
                macro_expanded: false,
                adaptive_delay_ms: 0,
            };
        }
        let prev_char = self.last_char;
        self.last_char = Some(ch);

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

        // Developer-Aware Markdown Inline Code (`...`) and Fenced Code Block (```...```) FSM:
        if self.config.developer_mode && !self.config.allow_vietnamese_in_backtick {
            if ch == '`' {
                if !self.current_word.is_empty() {
                    self.reset_word_buffers();
                }
                self.last_delimiter = Some('`');
                self.last_delimiter_spaced = false;
                self.sentence_start_pending = false;
                self.dev_code_state.on_backtick();
                return EngineOutput {
                    backspaces: 0,
                    commit_text: "`".to_string(),
                    consumed: false,
                    auto_restored: false,
                    macro_expanded: false,
                    adaptive_delay_ms: 0,
                };
            } else if ch == '\n' || ch == '\r' {
                self.dev_code_state.on_newline();
            } else {
                self.dev_code_state.on_non_backtick();
            }

            if self.dev_code_state.is_bypassing() {
                if !self.current_word.is_empty() {
                    self.reset_word_buffers();
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
        }

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
                let formatted_old = self.format_commit(&old_word);
            let formatted_new = self.format_commit(&self.current_word);
            let (backspaces, commit_text) = compare_and_split_strings(&formatted_old, &formatted_new);
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

        // Developer-Aware: Hash prefix technical token handling (e.g. #hex colors #f97316, #ef4444, CSS IDs #app, hashtags)
        if ch == '#' {
            self.last_delimiter = Some('#');
            self.last_delimiter_spaced = false;

            if self.current_word.is_empty() {
                self.raw_strokes.push('#');
                self.current_word.push('#');
                self.english_locked = true;
                return EngineOutput {
                    backspaces: 0,
                    commit_text: self.format_commit("#"),
                    consumed: false,
                    auto_restored: false,
                    macro_expanded: false,
                    adaptive_delay_ms: 0,
                };
            } else {
                let old_word = self.current_word.clone();
                self.raw_strokes.push('#');
                self.current_word = self.raw_strokes.clone();
                self.english_locked = true;
                let formatted_old = self.format_commit(&old_word);
            let formatted_new = self.format_commit(&self.current_word);
            let (backspaces, commit_text) = compare_and_split_strings(&formatted_old, &formatted_new);
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

        // Word delimiter check (Space, Enter, Punctuation)
        if !ch.is_alphanumeric() && !is_bracket {
            if matches!(ch, '.' | '?' | '!' | '\n') {
                let prev_lower = self.raw_strokes.to_lowercase();
                let is_abbrev_or_number = prev_lower.chars().any(|c| c.is_ascii_digit())
                    || matches!(
                        prev_lower.as_str(),
                        "vd" | "tp" | "mr" | "mrs" | "ms" | "dr" | "vs" | "etc" | "st" | "ts" | "ths"
                    );
                self.sentence_start_pending =
                    ch == '\n' || (!is_abbrev_or_number && !self.raw_strokes.is_empty());
            } else if ch != ' ' {
                self.sentence_start_pending = false;
            }
            let was_preceded_by_tech_delim = self.preceded_by_tech_delimiter;
            let was_preceded_by_slash = self.preceded_by_leading_slash;
            let next_tech_delim = is_macro_technical_delimiter(ch);
            let next_leading_slash = ch == '/'
                && (self.current_word.is_empty() || was_preceded_by_slash);
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

                // 1. Macro expansion check (suppressed if this word was just reverted via Backspace
                // or if surrounded by technical delimiters such as URLs, emails, paths, or function calls)
                let is_suppressed = self.suppressed_macro.as_deref() == Some(&self.current_word)
                    || self.suppressed_macro.as_deref() == Some(&self.raw_strokes)
                    || should_suppress_macro_expansion(was_preceded_by_tech_delim, ch);
                if self.config.enable_macro && !is_suppressed {
                    let expansion_opt = self.macro_table.expand(&self.current_word).map(|e| (e, self.current_word.clone())).or_else(|| {
                        if !self.manual_undo {
                            self.macro_table.expand(&self.raw_strokes).map(|e| (e, self.raw_strokes.clone()))
                        } else {
                            None
                        }
                    });
                    if let Some((expansion, trigger)) = expansion_opt {
                        let target = format!("{}{}", expansion, ch);
                        let formatted_old = self.format_commit(&self.current_word);
                        let formatted_target = self.format_commit(&target);
                        let (backspaces, commit_text) =
                            compare_and_split_strings(&formatted_old, &formatted_target);
                        let saved = expansion.chars().count().saturating_sub(trigger.chars().count()) as u64;
                        self.stats.macros_expanded += 1;
                        self.stats.keystrokes_saved += saved;
                        self.reset();
                        self.preceded_by_tech_delimiter = next_tech_delim;
                        self.preceded_by_leading_slash = next_leading_slash;
                        self.last_macro = Some(LastMacroExpansion {
                            trigger,
                            expansion: expansion.clone(),
                            delimiter: ch,
                            stage: MacroUndoStage::Expanded,
                        });
                        self.last_completed_word = Some(target);
                        // If macro preview is enabled, store the preview state for the addon to display a hint
                        if self.config.macro_preview {
                            self.pending_macro_preview = Some(PendingMacroPreview {
                                trigger: self.current_word.clone(),
                                expansion: expansion.clone(),
                            });
                        }
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
                let is_elongated_vn = is_valid_elongated_vietnamese_word(&self.current_word);
                let is_user_vn_word = self.is_user_vietnamese_word(&self.current_word);
                let is_invalid_syllable = !is_elongated_vn && !is_user_vn_word && !is_valid_vietnamese_syllable(&self.current_word);
                // Always try dict-guarded fuzzy vowel resolution for invalid syllables.
                // This safely fixes common unambiguous cases like nuoc1->nước, nguoi2->người
                // even when smart_fuzzy_vowel is OFF.
                if ch == ' ' && is_invalid_syllable && !self.manual_undo {
                    if let Some(resolved) = try_resolve_fuzzy_intermediate_vowel(&self.current_word, self.config.tone_style) {
                        let target = format!("{}{}", resolved, ch);
                        let formatted_old = self.format_commit(&self.current_word);
                        let formatted_target = self.format_commit(&target);
                        let (backspaces, commit_text) =
                            compare_and_split_strings(&formatted_old, &formatted_target);
                        let raw_lower_fuzzy = self.raw_strokes.to_lowercase();
                        self.update_sentence_context_after_word(&raw_lower_fuzzy, &resolved, false, ch);
                        self.reset_word_buffers();
                        self.preceded_by_tech_delimiter = next_tech_delim;
                        self.preceded_by_leading_slash = next_leading_slash;
                        self.stats.words_typed += 1;
                        self.last_completed_word = Some(target);
                        return EngineOutput {
                            backspaces,
                            commit_text,
                            consumed: true,
                            auto_restored: false,
                            macro_expanded: false,
                            adaptive_delay_ms: 0,
                        };
                    }
                }
                let raw_lower = self.raw_strokes.to_lowercase();
                let is_slash_command_or_path = was_preceded_by_slash
                    && self.current_word != self.raw_strokes
                    && is_leading_slash_command_or_path_token(&raw_lower);
                let is_dot_module_prefix = ch == '.'
                    && !self.manual_undo
                    && self.current_word != self.raw_strokes
                    && matches!(raw_lower.as_str(), "os" | "sys" | "io" | "fs" | "db" | "re");
                let math_context_collision = self.config.math_context
                    && self.math_context_words_remaining > 0
                    && !self.manual_undo
                    && self.current_word != self.raw_strokes
                    && raw_lower == "cos";
                let sentence_momentum_collision = self.config.sentence_momentum
                    && self.in_english_context
                    && !self.manual_undo
                    && self.current_word != self.raw_strokes
                    && is_context_english_collision_word(&raw_lower);
                let prev_word_collision = self.prev_word_was_english
                    && !self.manual_undo
                    && (2..=4).contains(&self.raw_strokes.len())
                    && self.current_word != self.raw_strokes
                    && matches!(raw_lower.as_str(), "is" | "this" | "has" | "see");
                let context_english_collision =
                    math_context_collision || sentence_momentum_collision || prev_word_collision;

                let is_known_english = is_slash_command_or_path
                    || is_dot_module_prefix
                    || context_english_collision
                    || (self.config.auto_restore_english
                        && !has_digits
                        && self.current_word != self.raw_strokes
                        && self.auto_restore.is_english_keyword(&self.raw_strokes));

                let alphanumeric_trap = self.has_letter_after_digit
                    && (!is_valid_vietnamese_syllable(&self.current_word)
                        || self.current_word.chars().any(|c| c.is_ascii_digit()));

                if is_known_english
                    || (!self.manual_undo
                        && !is_elongated_vn
                        && (!has_digits || is_invalid_syllable || alphanumeric_trap)
                        && has_vietnamese_diacritic(&self.current_word))
                {
                    let learned_restore = !has_digits && self.habit_learner.is_learned_word(&self.raw_strokes);
                    let dict_restore = !has_digits
                        && self.config.enable_dictionary
                        && self.auto_restore.should_restore_with_options(
                            &self.raw_strokes,
                            &self.current_word,
                            true,
                            self.config.acronym_bypass,
                        );
                    // Invalid intermediate syllables (duòng, tién, nuóc) MUST auto-restore on Space even if auto_restore_english is false.
                    let english_or_spell_restore = is_known_english
                        || is_invalid_syllable
                        || alphanumeric_trap
                        || (self.config.auto_restore_english
                            && !has_digits
                            && !is_elongated_vn
                            && !is_user_vn_word
                            && (has_phonology_violation(&self.current_word)
                                || (self.config.spell_check
                                    && !self.is_in_any_vietnamese_dictionary(&self.current_word))));

                    if learned_restore || dict_restore || english_or_spell_restore {
                        let restored_word = self.raw_strokes.clone();
                        let target = format!("{}{}", restored_word, ch);
                        let formatted_old = self.format_commit(&self.current_word);
                        let formatted_target = self.format_commit(&target);
                        let (backspaces, commit_text) =
                            compare_and_split_strings(&formatted_old, &formatted_target);
                        self.update_sentence_context_after_word(&raw_lower, &restored_word, true, ch);
                        self.reset_word_buffers();
                        self.preceded_by_tech_delimiter = next_tech_delim;
                        self.preceded_by_leading_slash = next_leading_slash;
                        self.stats.english_restored += 1;
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

            let completed_word_done = self.current_word.clone();

            // Auto-capitalize proper nouns (opt-in, default OFF)
            // Sử dụng danh sách từ được user định nghĩa qua config (proper_nouns_list).
            // List rỗng nghĩa là tính năng tắt (không tự viết hoa bất kỳ từ nào).
            if self.config.auto_capitalize_proper_nouns
                && !completed_word_done.is_empty()
            {
                let lc = completed_word_done.to_lowercase();
                if self
                    .config
                    .proper_nouns_list
                    .iter()
                    .any(|s| s.to_lowercase() == lc)
                {
                    let mut chars: Vec<char> = completed_word_done.chars().collect();
                    if let Some(c) = chars.first_mut() {
                        *c = c.to_ascii_uppercase();
                    }
                    self.current_word = chars.into_iter().collect();
                }
            }

            // No macro or auto-restore needed: reset word buffer and let delimiter pass through natively
            let has_word = !self.current_word.is_empty();
            let raw_lower_done = self.raw_strokes.to_lowercase();
            let completed_word_done = self.current_word.clone(); // re-clone after possible capitalization
            let was_english_locked_done = self.english_locked;
            let raw_target = format!("{}{}", self.current_word, ch);
            let formatted_target = self.format_commit(&raw_target);
            if ch == ' ' && has_word && self.raw_strokes.len() <= 32 {
                if self.reopen_history.len() >= 16 {
                    self.reopen_history.pop_front();
                }
                self.reopen_history.push_back(ReopenWordState {
                    current_word: self.current_word.clone(),
                    raw_strokes: self.raw_strokes.clone(),
                    word_used_telex: self.word_used_telex,
                    word_used_vni: self.word_used_vni,
                    manual_undo: self.manual_undo,
                    english_locked: self.english_locked,
                });
            }
            if has_word {
                self.update_sentence_context_after_word(
                    &raw_lower_done,
                    &completed_word_done,
                    was_english_locked_done,
                    ch,
                );
            } else if matches!(ch, '.' | '!' | '?' | '\n' | '\r') {
                self.in_english_context = false;
                self.in_vietnamese_context = false;
                self.math_context_words_remaining = 0;
                self.prev_word_was_english = false;
            }
            self.reset_word_buffers();
            self.preceded_by_tech_delimiter = next_tech_delim;
            self.preceded_by_leading_slash = next_leading_slash;
            if has_word {
                self.stats.words_typed += 1;
                self.last_completed_word = Some(formatted_target);
            }
            let auto_space = self.config.auto_space_after_punctuation
                && matches!(ch, '.' | '?' | '!')
                && !self.dev_code_state.is_bypassing()
                && prev_char.map(|c| c.is_alphabetic()).unwrap_or(false);
            let commit_text = if auto_space {
                format!("{} ", ch)
            } else {
                ch.to_string()
            };
            return EngineOutput {
                backspaces: 0,
                commit_text,
                consumed: auto_space,
                auto_restored: false,
                macro_expanded: false,
                adaptive_delay_ms: 0,
            };
        }

        // Any non-delimiter key cancels pending reopen state

        // Developer-Aware: Chained dot accessor detection (e.g. "db.collection.findOne")
        // If an alphanumeric key begins immediately after a dot without whitespace, lock English mode
        if self.current_word.is_empty() && self.last_delimiter == Some('.') && !self.last_delimiter_spaced {
            self.english_locked = true;
        }

        // Developer-Aware: Technical Token Protection (e.g. CSS units like "10px", "2rem", "50vh", variables with digits "v1s", "sha256", "md5")
        let mut auto_cap_consumed = false;
        if self.config.auto_capitalize
            && self.sentence_start_pending
            && self.current_word.is_empty()
            && ch.is_ascii_lowercase()
        {
            let is_chained_dot = self.last_delimiter == Some('.') && !self.last_delimiter_spaced;
            if !is_chained_dot {
                ch = ch.to_ascii_uppercase();
                auto_cap_consumed = true;
            }
            self.sentence_start_pending = false;
        } else if !self.current_word.is_empty() {
            self.sentence_start_pending = false;
        }

        // If the token starts with '#' or contains digits in Telex mode and a letter arrives, lock English mode.
        // In VNI/Auto mode, a digit could be a tone modifier (e.g. "biet61"), so we only lock if method is Telex/SimpleTelex,
        // or if the token starts with '#', or if digits are already followed by non-transforming letters.
        let is_telex_method = matches!(self.config.method, InputMethod::Telex | InputMethod::SimpleTelex);
        let has_prior_digits = self.raw_strokes.chars().any(|c| c.is_ascii_digit());
        let starts_with_hash = self.raw_strokes.starts_with('#');
        if ((is_telex_method && has_prior_digits) || starts_with_hash) && ch.is_ascii_alphabetic() && !self.english_locked {
            let old_word = self.current_word.clone();
            self.raw_strokes.push(ch);
            self.current_word = self.raw_strokes.clone();
            self.english_locked = true;
            let formatted_old = self.format_commit(&old_word);
            let formatted_new = self.format_commit(&self.current_word);
            let (backspaces, commit_text) = compare_and_split_strings(&formatted_old, &formatted_new);
            let consumed = backspaces > 0 || commit_text != ch.to_string() || auto_cap_consumed;
            return EngineOutput {
                backspaces,
                commit_text,
                consumed,
                auto_restored: backspaces > 0,
                macro_expanded: false,
                adaptive_delay_ms: 0,
            };
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

        let mut test_chars: Vec<char> = Vec::with_capacity(self.current_word.len() + 1);
        test_chars.extend(self.current_word.chars());
        test_chars.push(ch.to_ascii_lowercase());
        let is_valid_viet_continuation = is_valid_in_progress_syllable(&test_chars);

        if prev_has_lower && ch.is_ascii_uppercase() && !can_transform_vietnamese && !is_valid_viet_continuation {
            let old_word = self.current_word.clone();
            self.raw_strokes.push(ch);
            self.current_word = self.raw_strokes.clone();
            self.english_locked = true;
            let formatted_old = self.format_commit(&old_word);
            let formatted_new = self.format_commit(&self.current_word);
            let (backspaces, commit_text) = compare_and_split_strings(&formatted_old, &formatted_new);
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
            // If we are in an English-locked word or technical token (e.g. #f97316, sha256), keep digit in token
            if self.english_locked || self.raw_strokes.starts_with('#') {
                self.raw_strokes.push(ch);
                self.current_word.push(ch);
                self.casing.track(ch);
                self.english_locked = true;
                return EngineOutput {
                    backspaces: 0,
                    commit_text: ch.to_string(),
                    consumed: false,
                    auto_restored: false,
                    macro_expanded: false,
                    adaptive_delay_ms: 0,
                };
            }

            match self.config.method {
                InputMethod::Telex | InputMethod::SimpleTelex => {
                    // For standalone numbers or numbers following letters (e.g. 10, v1, test123, user2026),
                    // if the prefix is not a valid Vietnamese syllable when auto_restore_english is on,
                    // restore the prefix to raw_strokes immediately while keeping numeric macros (e.g. k8s, mp3)
                    // and completed Vietnamese words followed by digits (e.g. được7duo) intact.
                    let old_word = self.current_word.clone();
                    let next_raw = format!("{}{}", self.raw_strokes, ch);
                    let is_macro_prefix =
                        self.config.enable_macro && self.macro_table.has_prefix(&next_raw);
                    let should_restore_before_digit = !is_macro_prefix
                        && !self.manual_undo
                        && self.current_word != self.raw_strokes
                        && (!is_valid_vietnamese_syllable(&self.current_word)
                            || (self.config.auto_restore_english
                                && (self.auto_restore.is_english_keyword(&self.raw_strokes)
                                    || has_phonology_violation(&self.current_word)
                                    || (self.config.spell_check
                                        && !is_in_vietnamese_dictionary(&self.current_word)))));
                    self.raw_strokes.push(ch);
                    if should_restore_before_digit {
                        self.current_word = self.raw_strokes.clone();
                    } else {
                        self.current_word.push(ch);
                    }
                    self.english_locked = true;
                    let formatted_old = self.format_commit(&old_word);
                    let formatted_new = self.format_commit(&self.current_word);
                    let (backspaces, commit_text) =
                        compare_and_split_strings(&formatted_old, &formatted_new);
                    let consumed =
                        backspaces > 0 || commit_text != ch.to_string() || auto_cap_consumed;
                    return EngineOutput {
                        backspaces,
                        commit_text,
                        consumed,
                        auto_restored: backspaces > 0,
                        macro_expanded: false,
                        adaptive_delay_ms: 0,
                    };
                }
                InputMethod::Auto => {
                    if self.word_used_telex {
                        let old_word = self.current_word.clone();
                        let next_raw = format!("{}{}", self.raw_strokes, ch);
                        let is_macro_prefix =
                            self.config.enable_macro && self.macro_table.has_prefix(&next_raw);
                        let should_restore_before_digit = !is_macro_prefix
                            && !self.manual_undo
                            && self.current_word != self.raw_strokes
                            && (!is_valid_vietnamese_syllable(&self.current_word)
                                || (self.config.auto_restore_english
                                    && (self.auto_restore.is_english_keyword(&self.raw_strokes)
                                        || has_phonology_violation(&self.current_word)
                                        || (self.config.spell_check
                                            && !is_in_vietnamese_dictionary(&self.current_word)))));
                        if should_restore_before_digit {
                            self.raw_strokes.push(ch);
                            self.current_word = self.raw_strokes.clone();
                            self.english_locked = true;
                            let formatted_old = self.format_commit(&old_word);
                            let formatted_new = self.format_commit(&self.current_word);
                            let (backspaces, commit_text) =
                                compare_and_split_strings(&formatted_old, &formatted_new);
                            return EngineOutput {
                                backspaces,
                                commit_text,
                                consumed: true,
                                auto_restored: true,
                                macro_expanded: false,
                                adaptive_delay_ms: 0,
                            };
                        }
                        self.reset();
                        self.preceded_by_tech_delimiter = true;
                        return EngineOutput {
                            backspaces: 0,
                            commit_text: ch.to_string(),
                            consumed: false,
                            auto_restored: false,
                            macro_expanded: false,
                            adaptive_delay_ms: 0,
                        };
                    }
                    return self.process_vni_digit(ch, auto_cap_consumed);
                }
                InputMethod::Vni => {
                    return self.process_vni_digit(ch, auto_cap_consumed);
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
            if let Some(last_ch) = self.current_word.chars().next_back() {
                if last_ch.eq_ignore_ascii_case(&ch) && ch.is_ascii_alphabetic() {
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
                        let formatted_old = self.format_commit(&old_word);
            let formatted_new = self.format_commit(&self.current_word);
            let (backspaces, commit_text) = compare_and_split_strings(&formatted_old, &formatted_new);
                        return EngineOutput {
                            backspaces,
                            commit_text,
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
            let has_vowel = self.current_word.chars().any(crate::telex::is_vietnamese_vowel);
            if has_vowel {
                if let Some(last_ch) = self.current_word.chars().next_back() {
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
                            let formatted_old = self.format_commit(&old_word);
            let formatted_new = self.format_commit(&self.current_word);
            let (backspaces, commit_text) = compare_and_split_strings(&formatted_old, &formatted_new);
                            return EngineOutput {
                                backspaces,
                                commit_text,
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

        let mut was_transformed = was_transformed;
        // Early Alphanumeric Guard: if a VNI digit transforms a word into an invalid Vietnamese
        // syllable (e.g. "icon" + '2' -> "icòn"), lock English immediately so "icon256" passes through.
        if ch.is_ascii_digit() && !matches!(ch, '6' | '7' | '8' | '9') && was_transformed && !was_undo
            && (crate::syllable_parser::parse_syllable(&new_word).is_none()
                || self.auto_restore.is_english_keyword(&self.current_word))
            {
                new_word = format!("{}{}", self.current_word, ch);
                was_transformed = false;
                self.english_locked = true;
            }

        if has_prior_digits && ch.is_ascii_alphabetic() && !was_transformed {
            self.has_letter_after_digit = true;
        }
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
        let is_elongated_vn = has_diacritic && is_valid_elongated_vietnamese_word(&new_word);
        let invalid_structure = has_diacritic
            && !is_elongated_vn
            && (!is_valid_in_progress_syllable(&new_chars) || has_phonology_violation(&new_word));
        let slash_token_match = self.preceded_by_leading_slash
            && has_diacritic
            && is_leading_slash_command_or_path_token(&self.raw_strokes.to_lowercase());
        let learned_match = !is_vni_word
            && has_diacritic
            && self.habit_learner.is_learned_word(&self.raw_strokes);
        let dict_match = !is_vni_word
            && has_diacritic
            && self.config.enable_dictionary
            && self
                .auto_restore
                .should_restore_with_options(
                    &self.raw_strokes,
                    &new_word,
                    true,
                    false,
                );

        if !self.manual_undo
            && !is_vni_word
            && ((self.config.auto_restore_english && invalid_structure)
                || slash_token_match
                || dict_match
                || learned_match)
        {
            new_word = self.raw_strokes.clone();
            self.english_locked = true;
            auto_restored = true;
        } else if self.config.spell_check
            && !self.manual_undo
            && was_transformed
            && invalid_structure
            && !self.is_user_vietnamese_word(&new_word)
        {
            // If Chánh Tả is enabled (even when Restore English is disabled), prevent invalid Vietnamese syllable marks
            new_word = format!("{}{}", self.current_word, ch);
        }

        // Optional Initial Case Anchor (user configurable, default: false)
        // When enabled by user: ĐƯợc/ĐƯỢc -> Được, đƯỢc -> được.
        if self.config.initial_case_anchor && !new_word.is_empty() && !self.english_locked && !auto_restored {
            new_word = crate::casing::harmonize_syllable_case(&new_word);
        }

        let formatted_old = self.format_commit(&self.current_word);
        let formatted_new = self.format_commit(&new_word);
        let (backspaces, commit_text) = compare_and_split_strings(&formatted_old, &formatted_new);
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
#[cfg(test)]
mod tests;
