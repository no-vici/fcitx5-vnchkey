//! Syllable State Machine and Input Orchestrator.

use crate::telex::TelexEngine;
pub use crate::telex::W2UType;
use crate::vni::VniEngine;
use crate::auto_restore::EnglishAutoRestore;
use crate::casing::CaseTracker;
use crate::macros::MacroTable;
use crate::iki::IkiSensor;
use crate::sentinel::SentinelBarrier;

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
            telex: TelexEngine::with_options(config.tone_style, config.type_w_to_u, config.bracket_transform),
            vni: VniEngine::with_bracket_transform(config.tone_style, config.bracket_transform),
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

    /// Enables or disables GoNhanh English auto-restore.
    pub fn set_auto_restore_english(&mut self, enabled: bool) {
        self.config.auto_restore_english = enabled;
    }

    /// Returns whether English auto-restore is enabled.
    pub fn auto_restore_english(&self) -> bool {
        self.config.auto_restore_english
    }

    /// Enables or disables XKey smart casing.
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

    /// Resets all internal buffers (e.g. on mouse click or window unfocus).
    pub fn reset(&mut self) {
        self.current_word.clear();
        self.raw_strokes.clear();
        self.casing.reset();
        self.sentinel.reset();
    }

    /// Handles physical Backspace pressed by the user without getting stuck.
    pub fn handle_backspace(&mut self) {
        self.raw_strokes.pop();
        self.current_word.pop();
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
            if !auto_restored && self.config.enable_macro && !self.current_word.is_empty() {
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
            InputMethod::Auto => {
                if self.raw_strokes.chars().any(|c| c.is_ascii_digit()) {
                    let (vni_res, vni_ok) = self.vni.feed(&self.raw_strokes);
                    if vni_ok {
                        (vni_res, true)
                    } else {
                        self.telex.feed(&self.raw_strokes)
                    }
                } else {
                    self.telex.feed(&self.raw_strokes)
                }
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn type_word(engine: &mut Engine, word: &str) -> String {
        let mut result = String::new();
        for (i, ch) in word.chars().enumerate() {
            let out = engine.process_key(ch, (i * 50) as f64);
            if out.backspaces > 0 {
                let current_len = result.chars().count();
                let keep = current_len.saturating_sub(out.backspaces);
                result = result.chars().take(keep).collect();
            }
            result.push_str(&out.commit_text);
        }
        result
    }

    #[test]
    fn test_engine_auto_telex_and_vni() {
        let mut engine = Engine::new(EngineConfig {
            method: InputMethod::Auto,
            ..EngineConfig::default()
        });

        // Test Telex in Auto mode: dduocwj -> được
        let r1 = type_word(&mut engine, "dduocwj ");
        assert_eq!(r1, "được ");

        // Test VNI in Auto mode: duoc975 -> được
        let r2 = type_word(&mut engine, "duoc975 ");
        assert_eq!(r2, "được ");

        // Test Telex in Auto mode: tinhf -> tình
        let r3 = type_word(&mut engine, "tinhf ");
        assert_eq!(r3, "tình ");

        // Test VNI in Auto mode: tinh2 -> tình
        let r4 = type_word(&mut engine, "tinh2 ");
        assert_eq!(r4, "tình ");
    }
}
