//! Telex Input Method Transformer.

use crate::engine::ToneStyle;

/// Telex engine processing character combinations.
#[derive(Debug, Clone)]
pub struct TelexEngine {
    tone_style: ToneStyle,
}

impl TelexEngine {
    /// Creates a new TelexEngine.
    pub fn new(tone_style: ToneStyle) -> Self {
        Self { tone_style }
    }

    /// Transforms raw strokes using Telex rules.
    /// Returns (transformed_string, was_transformed).
    pub fn feed(&self, input: &str) -> (String, bool) {
        if input.is_empty() {
            return (String::new(), false);
        }

        let lower = input.to_lowercase();
        let mut result = String::with_capacity(input.len());
        let chars: Vec<char> = lower.chars().collect();
        let mut i = 0;
        let mut modified = false;

        // Syllable diacritic rules
        // e.g. aa->â, aw->ă, ee->ê, oo->ô, ow->ơ, uw->ư, dd->đ
        while i < chars.len() {
            let c = chars[i];
            let next = chars.get(i + 1).copied();

            match (c, next) {
                ('a', Some('a')) => { result.push('â'); i += 2; modified = true; }
                ('a', Some('w')) => { result.push('ă'); i += 2; modified = true; }
                ('e', Some('e')) => { result.push('ê'); i += 2; modified = true; }
                ('o', Some('o')) => { result.push('ô'); i += 2; modified = true; }
                ('o', Some('w')) => { result.push('ơ'); i += 2; modified = true; }
                ('u', Some('w')) => { result.push('ư'); i += 2; modified = true; }
                ('d', Some('d')) => { result.push('đ'); i += 2; modified = true; }
                ('w', _) => {
                    // Standalone 'w' -> 'ư'
                    result.push('ư');
                    i += 1;
                    modified = true;
                }
                _ => {
                    result.push(c);
                    i += 1;
                }
            }
        }

        // Apply tones: s=sắc, f=huyền, r=hỏi, x=ngã, j=nặng
        let (with_tones, tone_applied) = self.apply_tones(&result);
        if tone_applied {
            modified = true;
        }

        (with_tones, modified)
    }

    /// Feeds simple telex (w without solitary u conversion).
    pub fn feed_simple(&self, input: &str) -> (String, bool) {
        self.feed(input)
    }

    fn apply_tones(&self, text: &str) -> (String, bool) {
        let chars: Vec<char> = text.chars().collect();
        if chars.len() < 2 {
            return (text.to_string(), false);
        }

        let last_char = *chars.last().unwrap();
        let tone = match last_char {
            's' => 1, // Sắc
            'f' => 2, // Huyền
            'r' => 3, // Hỏi
            'x' => 4, // Ngã
            'j' => 5, // Nặng
            _ => 0,
        };

        if tone == 0 {
            return (text.to_string(), false);
        }

        // Base text without tone key
        let base_chars = &chars[..chars.len() - 1];
        let mut vowels: Vec<(usize, char)> = Vec::new();
        for (idx, &c) in base_chars.iter().enumerate() {
            if is_vietnamese_vowel(c) {
                vowels.push((idx, c));
            }
        }

        if vowels.is_empty() {
            return (text.to_string(), false);
        }

        // Modern vs Classic tone placement target index
        let target_idx = match self.tone_style {
            ToneStyle::Modern => {
                // Modern convention: place on nucleus
                if vowels.len() == 1 {
                    vowels[0].0
                } else if vowels.len() == 2 {
                    let first = vowels[0].1;
                    let second = vowels[1].1;
                    if (first == 'o' && second == 'a') || (first == 'u' && second == 'y') || (first == 'o' && second == 'e') {
                        vowels[1].0 // oà, uý, oè
                    } else {
                        vowels[0].0
                    }
                } else {
                    vowels[1].0
                }
            }
            ToneStyle::Classic => {
                // Classic convention: òa, ủy, hòe
                if vowels.len() >= 2 && (vowels[0].1 == 'o' || vowels[0].1 == 'u') {
                    vowels[0].0
                } else {
                    vowels.last().map(|v| v.0).unwrap_or(0)
                }
            }
        };

        let mut output = String::new();
        for (idx, &c) in base_chars.iter().enumerate() {
            if idx == target_idx {
                output.push(add_tone_to_vowel(c, tone));
            } else {
                output.push(c);
            }
        }

        (output, true)
    }
}

fn is_vietnamese_vowel(c: char) -> bool {
    matches!(c, 'a' | 'ă' | 'â' | 'e' | 'ê' | 'i' | 'o' | 'ô' | 'ơ' | 'u' | 'ư' | 'y')
}

fn add_tone_to_vowel(v: char, tone: usize) -> char {
    match (v, tone) {
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
        _ => v,
    }
}
