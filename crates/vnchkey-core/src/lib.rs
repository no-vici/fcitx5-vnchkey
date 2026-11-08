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
pub mod macros;
pub mod syllable_parser;
pub mod telex;
pub mod vietnamese_dict;
pub mod vni;

pub use auto_restore::{EnglishAutoRestore, DEFAULT_ENGLISH_WORDS};
pub use casing::{CaseTracker, CasingStyle};
pub use engine::{
    to_composite_unicode, Charset, Engine, EngineConfig, EngineOutput, InputMethod, ToneStyle,
    W2UType,
};
pub use macros::MacroTable;
pub use syllable_parser::{parse_syllable, ParsedSyllable, ToneMark};
pub use telex::TelexEngine;
pub use vietnamese_dict::{
    has_phonology_violation, is_in_vietnamese_dictionary, vietnamese_word_set,
};
pub use vni::VniEngine;
