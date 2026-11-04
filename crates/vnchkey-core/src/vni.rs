//! VNI Input Method Transformer.

use crate::engine::ToneStyle;

/// VNI engine transforming numbers 1-9 to Vietnamese diacritics and tones.
#[derive(Debug, Clone)]
pub struct VniEngine {
    tone_style: ToneStyle,
}

impl VniEngine {
    /// Creates a new VniEngine.
    pub fn new(tone_style: ToneStyle) -> Self {
        Self { tone_style }
    }

    /// Transforms raw strokes using VNI rules.
    /// Handles arbitrary key ordering (e.g. mung72 or mung27 -> mừng).
    pub fn feed(&self, input: &str) -> (String, bool) {
        if input.is_empty() || !input.chars().any(|c| c.is_ascii_digit()) {
            return (input.to_string(), false);
        }

        let mut letters: Vec<char> = Vec::new();
        let mut current_tone: usize = 0;
        let mut modified = false;

        for ch in input.chars() {
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

        if current_tone > 0 {
            // Apply tone to the target vowel index
            let mut vowel_indices = Vec::new();
            for (idx, &c) in letters.iter().enumerate() {
                if is_vietnamese_vowel(c) {
                    vowel_indices.push(idx);
                }
            }

            if !vowel_indices.is_empty() {
                let target_idx = if vowel_indices.len() == 1 {
                    vowel_indices[0]
                } else if vowel_indices.len() == 2 && self.tone_style == ToneStyle::Modern {
                    vowel_indices[1]
                } else if vowel_indices.len() >= 3 {
                    vowel_indices[1]
                } else {
                    vowel_indices[0]
                };

                letters[target_idx] = add_tone_to_vowel(letters[target_idx], current_tone);
            }
        }

        (letters.into_iter().collect(), true)
    }
}

fn is_vietnamese_vowel(c: char) -> bool {
    matches!(c, 'a' | 'ă' | 'â' | 'e' | 'ê' | 'i' | 'o' | 'ô' | 'ơ' | 'u' | 'ư' | 'y'
              | 'A' | 'Ă' | 'Â' | 'E' | 'Ê' | 'I' | 'O' | 'Ô' | 'Ơ' | 'U' | 'Ư' | 'Y'
              | 'á' | 'à' | 'ả' | 'ã' | 'ạ' | 'ắ' | 'ằ' | 'ẳ' | 'ẵ' | 'ặ' | 'ấ' | 'ầ' | 'ẩ' | 'ẫ' | 'ậ'
              | 'é' | 'è' | 'ẻ' | 'ẽ' | 'ẹ' | 'ế' | 'ề' | 'ể' | 'ễ' | 'ệ' | 'í' | 'ì' | 'ỉ' | 'ĩ' | 'ị'
              | 'ó' | 'ò' | 'ỏ' | 'õ' | 'ọ' | 'ố' | 'ồ' | 'ổ' | 'ỗ' | 'ộ' | 'ớ' | 'ờ' | 'ở' | 'ỡ' | 'ợ'
              | 'ú' | 'ù' | 'ủ' | 'ũ' | 'ụ' | 'ứ' | 'ừ' | 'ử' | 'ữ' | 'ự' | 'ý' | 'ỳ' | 'ỷ' | 'ỹ' | 'ỵ')
}

fn add_tone_to_vowel(v: char, tone: usize) -> char {
    let lower = v.to_lowercase().next().unwrap();
    let is_upper = v.is_uppercase();
    let toned = match (lower, tone) {
        ('a', 1) => 'á', ('a', 2) => 'à', ('a', 3) => 'ả', ('a', 4) => 'ã', ('a', 5) => 'ạ',
        ('ă', 1) => 'ắ', ('ă', 2) => 'ằ', ('ă', 3) => 'ẳ', ('ă', 4) => 'ẵ', ('ă', 5) => 'ặ',
        ('â', 1) => 'ấ', ('â', 2) => 'ầ', ('â', 3) => 'ẩ', ('â', 4) => 'ẫ', ('â', 5) => 'ậ',
        ('e', 1) => 'é', ('e', 2) => 'è', ('e', 3) => 'ẻ', ('e', 4) => 'ẽ', ('e', 5) => 'ẹ',
        ('ê', 1) => 'ế', ('ê', 2) => 'ề', ('ê', 3) => 'ể', ('ê', 4) => 'ễ', ('ê', 5) => 'ệ',
        ('i', 1) => 'í', ('i', 2) => 'ì', ('i', 3) => 'ỉ', ('i', 4) => 'ĩ', ('i', 5) => 'ị',
        ('o', 1) => 'ó', ('o', 2) => 'ò', ('o', 3) => 'ỏ', ('o', 4) => 'õ', ('o', 5) => 'ọ',
        ('ô', 1) => 'ố', ('ô', 2) => 'ồ', ('ô', 3) => 'ổ', ('ô', 4) => 'ỗ', ('ô', 5) => 'ộ',
        ('ơ', 1) => 'ớ', ('ơ', 2) => 'ờ', ('ơ', 3) => 'ở', ('ơ', 4) => 'ỡ', ('ơ', 5) => 'ợ',
        ('u', 1) => 'ú', ('u', 2) => 'ù', ('u', 3) => 'ủ', ('u', 4) => 'ũ', ('u', 5) => 'ụ',
        ('ư', 1) => 'ứ', ('ư', 2) => 'ừ', ('ư', 3) => 'ử', ('ư', 4) => 'ữ', ('ư', 5) => 'ự',
        ('y', 1) => 'ý', ('y', 2) => 'ỳ', ('y', 3) => 'ỷ', ('y', 4) => 'ỹ', ('y', 5) => 'ỵ',
        _ => lower,
    };
    if is_upper {
        toned.to_uppercase().next().unwrap()
    } else {
        toned
    }
}
