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

/// Harmonizes capitalization across a Vietnamese syllable based on its initial character.
/// - If all alphabetic characters are uppercase (ALL CAPS), preserves uppercase (`ĐƯỢC` -> `ĐƯỢC`).
/// - If the first character is uppercase, enforces TitleCase (`ĐƯợc`, `ĐƯỢc` -> `Được`).
/// - If the first character is lowercase, enforces lowercase across the entire word (`đƯỢc` -> `được`).
pub fn harmonize_syllable_case(word: &str) -> String {
    let chars: Vec<char> = word.chars().collect();
    if chars.len() <= 1 {
        return word.to_string();
    }

    let alpha_chars: Vec<char> = chars.iter().copied().filter(|c| c.is_alphabetic()).collect();
    if alpha_chars.is_empty() {
        return word.to_string();
    }

    // If entire word is uppercase (ALL CAPS), keep it intact (e.g. "ĐƯỢC", "VIỆT NAM")
    if alpha_chars.iter().all(|c| c.is_uppercase()) {
        return word.to_string();
    }

    // Anchor to the very first alphabetic character
    let first_is_upper = alpha_chars[0].is_uppercase();

    if first_is_upper {
        // TitleCase: First letter uppercase, every subsequent letter MUST be lowercase
        // (unless immediately preceded by an apostrophe like M'Đrắk, H'Leo)
        let mut seen_first = false;
        let mut after_apostrophe = false;
        let mut out = String::with_capacity(word.len());
        for c in chars {
            if c == '\'' || c == '’' {
                out.push(c);
                after_apostrophe = true;
            } else if c.is_alphabetic() {
                if !seen_first {
                    out.extend(c.to_uppercase());
                    seen_first = true;
                } else if after_apostrophe && c.is_uppercase() {
                    out.extend(c.to_uppercase());
                } else {
                    out.extend(c.to_lowercase());
                }
                after_apostrophe = false;
            } else {
                out.push(c);
                after_apostrophe = false;
            }
        }
        out
    } else {
        // LowerCase: First letter lowercase, entire word MUST be lowercase
        word.to_lowercase()
    }
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
