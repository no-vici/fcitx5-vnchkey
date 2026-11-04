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
pub mod telex;
pub mod vni;

pub use auto_restore::EnglishAutoRestore;
pub use casing::{CaseTracker, CasingStyle};
pub use engine::{Engine, EngineConfig, EngineOutput, InputMethod, ToneStyle, W2UType};
pub use macros::MacroTable;
pub use telex::TelexEngine;
pub use vni::VniEngine;
