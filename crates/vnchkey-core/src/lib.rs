//! # vnchKey Core
//! 
//! Memory-safe Vietnamese syllable transformation engine designed for Fcitx5 and Linux desktop.
//! Incorporates GoNhanh's English auto-restore and XKey's smart casing algorithms.
//!
//! # Safety
//! All modules in this crate forbid unsafe code:
#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]

pub mod engine;
pub mod telex;
pub mod vni;
pub mod auto_restore;
pub mod casing;
pub mod macros;
pub mod iki;
pub mod sentinel;

pub use engine::{Engine, InputMethod, ToneStyle, EngineConfig, EngineOutput};
pub use telex::TelexEngine;
pub use vni::VniEngine;
pub use auto_restore::EnglishAutoRestore;
pub use casing::{CaseTracker, CasingStyle};
pub use macros::MacroTable;
pub use iki::IkiSensor;
pub use sentinel::{SentinelBarrier, BarrierStatus};
