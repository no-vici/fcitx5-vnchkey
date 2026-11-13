//! # vnchKey Core
//!
//! Memory-safe Vietnamese syllable transformation engine designed for Fcitx5 and Linux desktop.
//! Incorporates English auto-restore (Restore English), Chánh Tả validation, and smart casing.
//!
//! # Safety
//! All modules in this crate forbid unsafe code:
#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]

pub mod auto_restore;
pub mod casing;
pub mod engine;
pub mod habit_learner;
pub mod macros;
pub mod symbols;
pub mod syllable_parser;
pub mod telex;
pub mod vietnamese_dict;
pub mod vni;

pub use auto_restore::{EnglishAutoRestore, DEFAULT_ENGLISH_WORDS};
pub use casing::{harmonize_syllable_case, CaseTracker, CasingStyle};
pub use engine::{
    to_composite_unicode, Charset, Engine, EngineConfig, EngineOutput, InputMethod, ToneStyle,
    W2UType,
};
pub use habit_learner::{
    CorrectionKind, HabitLearner, DEFAULT_MAX_LOG_BYTES, DEFAULT_PROMOTE_THRESHOLD,
};
pub use macros::MacroTable;
pub use symbols::{lookup_symbol, search_symbols, SymbolEntry, SYMBOL_TABLE};
pub use syllable_parser::{parse_syllable, ParsedSyllable, ToneMark};
pub use telex::TelexEngine;
pub use vietnamese_dict::{
    has_phonology_violation, is_in_vietnamese_dictionary, vietnamese_word_set,
};
pub use vni::VniEngine;
