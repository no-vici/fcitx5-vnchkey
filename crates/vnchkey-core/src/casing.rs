//! XKey Innovation: Smart Casing Engine.
//!
//! Preserves character-by-character capitalization vectors during diacritic
//! replacement and expansion (e.g. `VIET` -> `VIỆT`, `Viet` -> `Việt`).

/// Casing category of an individual letter.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CasingStyle {
    /// All lowercase (e.g. "việt")
    Lower,
    /// Capitalized first letter (e.g. "Việt")
    Title,
    /// All uppercase (e.g. "VIỆT")
    Upper,
    /// Mixed or camelCase
    Mixed,
}

/// Tracks character-level capitalization vectors.
#[derive(Debug, Clone, Default)]
pub struct CaseTracker {
    cases: Vec<bool>, // true if uppercase
}

impl CaseTracker {
    /// Creates a new CaseTracker.
    pub fn new() -> Self {
        Self { cases: Vec::with_capacity(16) }
    }

    /// Resets the tracker.
    pub fn reset(&mut self) {
        self.cases.clear();
    }

    /// Records the case of a typed character.
    pub fn track(&mut self, ch: char) {
        if ch.is_alphabetic() {
            if self.cases.len() < 64 {
                self.cases.push(ch.is_uppercase());
            }
        }
    }

    /// Applies the tracked casing pattern to transformed text.
    pub fn apply(&self, transformed: &str) -> String {
        if self.cases.is_empty() {
            return transformed.to_string();
        }

        let all_upper = self.cases.iter().all(|&c| c);
        if all_upper {
            return transformed.to_uppercase();
        }

        let first_upper = self.cases[0] && self.cases.iter().skip(1).all(|&c| !c);
        if first_upper {
            let mut chars = transformed.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        } else {
            transformed.to_string()
        }
    }
}
