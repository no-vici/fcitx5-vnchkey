//! Syllable State Machine and Input Orchestrator.

use crate::telex::TelexEngine;
use crate::vni::VniEngine;
use crate::auto_restore::EnglishAutoRestore;
use crate::casing::{CaseTracker, CasingStyle};
use crate::macros::MacroTable;
use crate::iki::IkiSensor;
use crate::sentinel::SentinelBarrier;

/// Supported Vietnamese typing methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMethod {
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
    /// Selected input method (Telex or VNI)
    pub method: InputMethod,
    /// Preferred tone placement convention
    pub tone_style: ToneStyle,
    /// GoNhanh innovation: automatically restore English keywords
    pub auto_restore_english: bool,
    /// XKey innovation: preserve exact capitalization pattern
    pub smart_casing: bool,
    /// Quick undo when pressing Backspace or Escape
    pub quick_undo: bool,
    /// Enable Sentinel Barrier N+1 protocol
    pub sentinel_enabled: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            method: InputMethod::Telex,
            tone_style: ToneStyle::Modern,
            auto_restore_english: true,
            smart_casing: true,
            quick_undo: true,
            sentinel_enabled: true,
        }
    }
}

/// Output returned after processing a single keystroke.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineOutput {
    /// Number of backspaces to emit to replace dirty pre-edit characters
    pub backspaces: usize,
    /// Replacement UTF-8 text to commit
    pub commit_text: String,
    /// Whether this stroke was consumed by the Vietnamese engine
    pub consumed: bool,
    /// Whether GoNhanh auto-restore restored raw English on this word
    pub auto_restored: bool,
    /// Whether a macro expansion was applied
    pub macro_expanded: bool,
    /// Calculated delay in milliseconds from IKI sensor
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
    iki: IkiSensor,
    sentinel: SentinelBarrier,
    current_word: String,
    raw_strokes: String,
}

impl Engine {
    /// Creates a new Engine with the specified configuration.
    pub fn new(config: EngineConfig) -> Self {
        Self {
            telex: TelexEngine::new(config.tone_style),
            vni: VniEngine::new(config.tone_style),
            auto_restore: EnglishAutoRestore::new(),
            casing: CaseTracker::new(),
            macro_table: MacroTable::new(),
            iki: IkiSensor::new(),
            sentinel: SentinelBarrier::new(),
            config,
            current_word: String::with_capacity(32),
            raw_strokes: String::with_capacity(32),
        }
    }

    /// Resets all internal buffers (e.g. on mouse click or window unfocus).
    pub fn reset(&mut self) {
        self.current_word.clear();
        self.raw_strokes.clear();
        self.casing.reset();
        self.sentinel.reset();
    }

    /// Process a key press event.
    pub fn process_key(&mut self, ch: char, timestamp_ms: f64) -> EngineOutput {
        let adaptive_delay = self.iki.record_stroke(timestamp_ms);

        // Word delimiter check (Space, Enter, Punctuation)
        if !ch.is_alphanumeric() {
            let mut auto_restored = false;
            let mut backspaces = 0;
            let mut commit_text = String::new();

            // GoNhanh feature: Verify if the finished word was a valid English word
            if self.config.auto_restore_english && !self.current_word.is_empty() {
                if self.auto_restore.is_english_keyword(&self.raw_strokes) {
                    // Restore original English characters
                    let word_char_count = self.current_word.chars().count();
                    backspaces = word_char_count;
                    commit_text = self.raw_strokes.clone();
                    auto_restored = true;
                }
            }

            // XKey feature: Macro lookup
            if !auto_restored && !self.current_word.is_empty() {
                if let Some(expansion) = self.macro_table.expand(&self.current_word) {
                    backspaces = self.current_word.chars().count();
                    commit_text = expansion;
                }
            }

            commit_text.push(ch);
            self.reset();

            return EngineOutput {
                backspaces,
                commit_text,
                consumed: true,
                auto_restored,
                macro_expanded: false,
                adaptive_delay_ms: adaptive_delay,
            };
        }

        self.raw_strokes.push(ch);
        self.casing.track(ch);

        // Transform via selected method
        let (transformed, is_transformed) = match self.config.method {
            InputMethod::Telex => self.telex.feed(&self.raw_strokes),
            InputMethod::Vni => self.vni.feed(&self.raw_strokes),
            InputMethod::SimpleTelex => self.telex.feed_simple(&self.raw_strokes),
        };

        if is_transformed {
            let backspaces = self.current_word.chars().count();
            
            // Apply XKey smart casing
            let final_word = if self.config.smart_casing {
                self.casing.apply(&transformed)
            } else {
                transformed
            };

            self.current_word = final_word.clone();

            EngineOutput {
                backspaces,
                commit_text: final_word,
                consumed: true,
                auto_restored: false,
                macro_expanded: false,
                adaptive_delay_ms: adaptive_delay,
            }
        } else {
            self.current_word.push(ch);
            EngineOutput {
                backspaces: 0,
                commit_text: ch.to_string(),
                consumed: false,
                auto_restored: false,
                macro_expanded: false,
                adaptive_delay_ms: adaptive_delay,
            }
        }
    }
}
