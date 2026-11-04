//! VNI Input Method Transformer.

use crate::engine::ToneStyle;
use crate::telex::{is_vietnamese_vowel, place_tone};

/// VNI engine processing number-based Vietnamese accents.
#[derive(Debug, Clone)]
pub struct VniEngine {
    tone_style: ToneStyle,
    bracket_transform: bool,
}

impl VniEngine {
    /// Creates a new VniEngine with default settings.
    pub fn new(tone_style: ToneStyle) -> Self {
        Self {
            tone_style,
            bracket_transform: false,
        }
    }

    /// Creates a new VniEngine with bracket transform option.
    pub fn with_bracket_transform(tone_style: ToneStyle, bracket_transform: bool) -> Self {
        Self {
            tone_style,
            bracket_transform,
        }
    }

    /// Sets the tone style convention.
    pub fn set_tone_style(&mut self, tone_style: ToneStyle) {
        self.tone_style = tone_style;
    }

    /// Enables or disables bracket transformation.
    pub fn set_bracket_transform(&mut self, enabled: bool) {
        self.bracket_transform = enabled;
    }

    /// Transforms raw strokes using VNI rules.
    /// Returns (transformed_string, was_transformed).
    pub fn feed(&self, input: &str) -> (String, bool) {
        if input.is_empty() {
            return (String::new(), false);
        }
        if !self.bracket_transform && !input.chars().any(|c| c.is_ascii_digit()) {
            return (input.to_string(), false);
        }

        let mut letters: Vec<char> = Vec::new();
        let mut current_tone: usize = 0;
        let mut modified = false;

        for ch in input.chars() {
            if self.bracket_transform {
                match ch {
                    '[' => {
                        if letters.last() == Some(&'ơ') {
                            letters.pop();
                            letters.push('[');
                        } else {
                            letters.push('ơ');
                            modified = true;
                        }
                        continue;
                    }
                    ']' => {
                        if letters.last() == Some(&'ư') {
                            letters.pop();
                            letters.push(']');
                        } else {
                            letters.push('ư');
                            modified = true;
                        }
                        continue;
                    }
                    '{' => {
                        if letters.last() == Some(&'Ơ') {
                            letters.pop();
                            letters.push('{');
                        } else {
                            letters.push('Ơ');
                            modified = true;
                        }
                        continue;
                    }
                    '}' => {
                        if letters.last() == Some(&'Ư') {
                            letters.pop();
                            letters.push('}');
                        } else {
                            letters.push('Ư');
                            modified = true;
                        }
                        continue;
                    }
                    _ => {}
                }
            }
            if ch.is_ascii_digit() {
                match ch {
                    '1'..='5' => {
                        current_tone = ch.to_digit(10).unwrap() as usize;
                        modified = true;
                    }
                    '0' => {
                        current_tone = 0;
                        modified = true;
                    }
                    '6' => {
                        // Nón: a->â, e->ê, o->ô
                        for c in letters.iter_mut().rev() {
                            match *c {
                                'a' => { *c = 'â'; modified = true; break; }
                                'A' => { *c = 'Â'; modified = true; break; }
                                'e' => { *c = 'ê'; modified = true; break; }
                                'E' => { *c = 'Ê'; modified = true; break; }
                                'o' => { *c = 'ô'; modified = true; break; }
                                'O' => { *c = 'Ô'; modified = true; break; }
                                _ => {}
                            }
                        }
                    }
                    '7' => {
                        // Râu: u->ư, o->ơ, uo->ươ
                        let mut u_idx = None;
                        let mut o_idx = None;
                        for (idx, &c) in letters.iter().enumerate() {
                            let lower = c.to_lowercase().next().unwrap();
                            if lower == 'u' || lower == 'ư' { u_idx = Some(idx); }
                            if lower == 'o' || lower == 'ơ' { o_idx = Some(idx); }
                        }

                        if let (Some(u), Some(o)) = (u_idx, o_idx) {
                            if (u as isize - o as isize).abs() == 1 {
                                letters[u] = if letters[u].is_uppercase() { 'Ư' } else { 'ư' };
                                letters[o] = if letters[o].is_uppercase() { 'Ơ' } else { 'ơ' };
                                modified = true;
                                continue;
                            }
                        }

                        for c in letters.iter_mut().rev() {
                            match *c {
                                'u' => { *c = 'ư'; modified = true; break; }
                                'U' => { *c = 'Ư'; modified = true; break; }
                                'o' => { *c = 'ơ'; modified = true; break; }
                                'O' => { *c = 'Ơ'; modified = true; break; }
                                _ => {}
                            }
                        }
                    }
                    '8' => {
                        // Trăng: a->ă
                        for c in letters.iter_mut().rev() {
                            if *c == 'a' { *c = 'ă'; modified = true; break; }
                            if *c == 'A' { *c = 'Ă'; modified = true; break; }
                        }
                    }
                    '9' => {
                        // Gạch: d->đ
                        for c in letters.iter_mut().rev() {
                            if *c == 'd' { *c = 'đ'; modified = true; break; }
                            if *c == 'D' { *c = 'Đ'; modified = true; break; }
                        }
                    }
                    _ => letters.push(ch),
                }
            } else {
                letters.push(ch);
            }
        }

        if !modified {
            return (input.to_string(), false);
        }

        // Smart VNI horn: if letters contain 'uo' followed by a coda (e.g. duoc, nuoc, buoc),
        // automatically promote 'uo' -> 'ươ'
        let mut u_idx = None;
        let mut o_idx = None;
        for (idx, &c) in letters.iter().enumerate() {
            let lower = c.to_lowercase().next().unwrap();
            if lower == 'u' { u_idx = Some(idx); }
            if lower == 'o' { o_idx = Some(idx); }
        }
        if let (Some(u), Some(o)) = (u_idx, o_idx) {
            if (u as isize - o as isize).abs() == 1 {
                let has_coda = o + 1 < letters.len() && !is_vietnamese_vowel(letters[o + 1]);
                if has_coda {
                    letters[u] = if letters[u].is_uppercase() { 'Ư' } else { 'ư' };
                    letters[o] = if letters[o].is_uppercase() { 'Ơ' } else { 'ơ' };
                }
            }
        }

        if current_tone > 0 {
            place_tone(&mut letters, current_tone, self.tone_style);
        }

        (letters.into_iter().collect(), true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vni_common() {
        let vni = VniEngine::new(ToneStyle::Modern);
        assert_eq!(vni.feed("tinh2").0, "tình");
        assert_eq!(vni.feed("dau61").0, "dấu");
        assert_eq!(vni.feed("duoc975").0, "được");
        assert_eq!(vni.feed("duoc95").0, "được");
        assert_eq!(vni.feed("nuoc1").0, "nước");
        assert_eq!(vni.feed("viet65").0, "việt");
    }
}
