//! Telex Input Method Transformer.

use crate::engine::ToneStyle;

/// Behavior of typing key 'w'.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum W2UType {
    /// Only produce ư when not at the start of a word (default)
    NonStart,
    /// Always produce ư
    Always,
    /// Never produce standalone ư
    Disabled,
}

/// Telex engine processing character combinations.
#[derive(Debug, Clone)]
pub struct TelexEngine {
    tone_style: ToneStyle,
    w2u: W2UType,
    bracket_transform: bool,
}

impl TelexEngine {
    /// Creates a new TelexEngine with modern defaults.
    pub fn new(tone_style: ToneStyle) -> Self {
        Self {
            tone_style,
            w2u: W2UType::NonStart,
            bracket_transform: false,
        }
    }

    /// Creates a new TelexEngine with custom options.
    pub fn with_options(tone_style: ToneStyle, w2u: W2UType, bracket_transform: bool) -> Self {
        Self {
            tone_style,
            w2u,
            bracket_transform,
        }
    }

    /// Transforms raw strokes using Telex rules.
    /// Returns (transformed_string, was_transformed).
    pub fn feed(&self, input: &str) -> (String, bool) {
        if input.is_empty() {
            return (String::new(), false);
        }

        let mut letters: Vec<char> = Vec::with_capacity(input.len());
        let mut current_tone: usize = 0;
        let mut modified = false;

        for ch in input.chars() {
            let lower = ch.to_ascii_lowercase();

            // Bracket transformation: [ -> ơ, ] -> ư, { -> Ơ, } -> Ư
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

            // Tone keys: s (1), f (2), r (3), x (4), j (5), z (0)
            // A tone key applies only if at least one vowel has been typed
            let has_vowel = letters.iter().any(|&c| is_vietnamese_vowel(c));

            if has_vowel && matches!(lower, 's' | 'f' | 'r' | 'x' | 'j' | 'z') {
                let tone = match lower {
                    's' => 1,
                    'f' => 2,
                    'r' => 3,
                    'x' => 4,
                    'j' => 5,
                    'z' => 0,
                    _ => 0,
                };
                if current_tone == tone && tone != 0 {
                    // Double tone key pressed: cancel tone and output literal character
                    current_tone = 0;
                    letters.push(ch);
                } else {
                    current_tone = tone;
                    modified = true;
                }
                continue;
            }

            // Horn modifier 'w' / 'W'
            if lower == 'w' {
                let is_upper = ch.is_uppercase();

                // 1. Check for adjacent 'uo', 'ưo', 'uơ', 'ươ' pair
                let mut u_idx = None;
                let mut o_idx = None;
                for (idx, &c) in letters.iter().enumerate() {
                    let l = c.to_lowercase().next().unwrap();
                    if l == 'u' || l == 'ư' { u_idx = Some(idx); }
                    if l == 'o' || l == 'ơ' { o_idx = Some(idx); }
                }

                if let (Some(u), Some(o)) = (u_idx, o_idx) {
                    if (u as isize - o as isize).abs() == 1 {
                        let u_is_horn = letters[u].to_lowercase().next().unwrap() == 'ư';
                        let o_is_horn = letters[o].to_lowercase().next().unwrap() == 'ơ';
                        if u_is_horn && o_is_horn {
                            letters[u] = if letters[u].is_uppercase() { 'U' } else { 'u' };
                            letters[o] = if letters[o].is_uppercase() { 'O' } else { 'o' };
                            letters.push(ch);
                        } else {
                            letters[u] = if letters[u].is_uppercase() { 'Ư' } else { 'ư' };
                            letters[o] = if letters[o].is_uppercase() { 'Ơ' } else { 'ơ' };
                            modified = true;
                        }
                        continue;
                    }
                }

                // 2. Check if there is 'a' / 'ă'
                let mut a_found = false;
                for c in letters.iter_mut().rev() {
                    let l = c.to_lowercase().next().unwrap();
                    if l == 'a' {
                        *c = if c.is_uppercase() { 'Ă' } else { 'ă' };
                        modified = true;
                        a_found = true;
                        break;
                    } else if l == 'ă' {
                        *c = if c.is_uppercase() { 'A' } else { 'a' };
                        letters.push(ch);
                        a_found = true;
                        break;
                    }
                }
                if a_found { continue; }

                // 3. Check if there is 'o' / 'ơ'
                let mut o_found = false;
                for c in letters.iter_mut().rev() {
                    let l = c.to_lowercase().next().unwrap();
                    if l == 'o' {
                        *c = if c.is_uppercase() { 'Ơ' } else { 'ơ' };
                        modified = true;
                        o_found = true;
                        break;
                    } else if l == 'ơ' {
                        *c = if c.is_uppercase() { 'O' } else { 'o' };
                        letters.push(ch);
                        o_found = true;
                        break;
                    }
                }
                if o_found { continue; }

                // 4. Check if there is 'u' / 'ư'
                let mut u_found = false;
                for c in letters.iter_mut().rev() {
                    let l = c.to_lowercase().next().unwrap();
                    if l == 'u' {
                        *c = if c.is_uppercase() { 'Ư' } else { 'ư' };
                        modified = true;
                        u_found = true;
                        break;
                    } else if l == 'ư' {
                        *c = if c.is_uppercase() { 'U' } else { 'u' };
                        letters.push(ch);
                        u_found = true;
                        break;
                    }
                }
                if u_found { continue; }

                // 5. Standalone 'w' -> 'ư' based on w2u mode
                match self.w2u {
                    W2UType::Disabled => {
                        letters.push(ch);
                    }
                    W2UType::NonStart => {
                        if letters.is_empty() {
                            letters.push(ch);
                        } else {
                            if let Some(last) = letters.last_mut() {
                                let l = last.to_lowercase().next().unwrap();
                                if l == 'ư' {
                                    *last = if last.is_uppercase() { 'W' } else { 'w' };
                                    modified = true;
                                    continue;
                                }
                            }
                            letters.push(if is_upper { 'Ư' } else { 'ư' });
                            modified = true;
                        }
                    }
                    W2UType::Always => {
                        if let Some(last) = letters.last_mut() {
                            let l = last.to_lowercase().next().unwrap();
                            if l == 'ư' {
                                *last = if last.is_uppercase() { 'W' } else { 'w' };
                                modified = true;
                                continue;
                            }
                        }
                        letters.push(if is_upper { 'Ư' } else { 'ư' });
                        modified = true;
                    }
                }
                continue;
            }

            // Circumflex and stroke modifiers:
            // 'dd' -> 'đ', 'aa' -> 'â', 'ee' -> 'ê', 'oo' -> 'ô'
            if let Some(last) = letters.last_mut() {
                let last_lower = last.to_ascii_lowercase();
                if lower == last_lower {
                    match lower {
                        'd' => {
                            *last = if last.is_uppercase() || ch.is_uppercase() { 'Đ' } else { 'đ' };
                            modified = true;
                            continue;
                        }
                        'a' => {
                            *last = if last.is_uppercase() || ch.is_uppercase() { 'Â' } else { 'â' };
                            modified = true;
                            continue;
                        }
                        'e' => {
                            *last = if last.is_uppercase() || ch.is_uppercase() { 'Ê' } else { 'ê' };
                            modified = true;
                            continue;
                        }
                        'o' => {
                            *last = if last.is_uppercase() || ch.is_uppercase() { 'Ô' } else { 'ô' };
                            modified = true;
                            continue;
                        }
                        _ => {}
                    }
                }
                // Toggle back if already transformed
                match (last_lower, lower) {
                    ('đ', 'd') => {
                        *last = if last.is_uppercase() { 'D' } else { 'd' };
                        letters.push(ch);
                        continue;
                    }
                    ('â', 'a') => {
                        *last = if last.is_uppercase() { 'A' } else { 'a' };
                        letters.push(ch);
                        continue;
                    }
                    ('ê', 'e') => {
                        *last = if last.is_uppercase() { 'E' } else { 'e' };
                        letters.push(ch);
                        continue;
                    }
                    ('ô', 'o') => {
                        *last = if last.is_uppercase() { 'O' } else { 'o' };
                        letters.push(ch);
                        continue;
                    }
                    _ => {}
                }
            }

            letters.push(ch);
        }

        if !modified && current_tone == 0 {
            return (input.to_string(), false);
        }

        if current_tone > 0 {
            place_tone(&mut letters, current_tone, self.tone_style);
            modified = true;
        }

        (letters.into_iter().collect(), modified)
    }

    /// Feeds simple telex (w without solitary u conversion).
    pub fn feed_simple(&self, input: &str) -> (String, bool) {
        self.feed(input)
    }
}

/// Places tone on the correct Vietnamese vowel index according to standard orthography rules.
pub fn place_tone(letters: &mut [char], tone: usize, tone_style: ToneStyle) {
    if tone == 0 || letters.is_empty() {
        return;
    }

    let mut vowel_indices = Vec::new();
    for (idx, &c) in letters.iter().enumerate() {
        if is_vietnamese_vowel(c) {
            vowel_indices.push(idx);
        }
    }

    if vowel_indices.is_empty() {
        return;
    }

    let target_idx = if vowel_indices.len() == 1 {
        vowel_indices[0]
    } else if vowel_indices.len() == 2 {
        let first = letters[vowel_indices[0]].to_lowercase().next().unwrap();
        let second = letters[vowel_indices[1]].to_lowercase().next().unwrap();

        let has_coda = vowel_indices[1] + 1 < letters.len()
            && !is_vietnamese_vowel(letters[vowel_indices[1] + 1]);

        if has_coda {
            // Syllables with final coda consonant (được, tiếng, muốn, hoàng, chuẩn):
            // Tone is ALWAYS on the second vowel!
            vowel_indices[1]
        } else if (first == 'o' && second == 'a')
            || (first == 'u' && second == 'y')
            || (first == 'o' && second == 'e')
        {
            // Open diphthongs 'oa', 'uy', 'oe'
            if tone_style == ToneStyle::Modern {
                vowel_indices[1] // hoà, uỷ, khoẻ
            } else {
                vowel_indices[0] // hòa, ủy, khỏe
            }
        } else {
            // Open diphthongs 'ia', 'ya', 'ua', 'ưa', 'ai', 'ao', 'au', 'ay', 'âu', 'oi', 'ui', 'ơi'...
            // Tone is on the first vowel (nucleus): mía, múa, chứa, dấu, bài, báo, túi...
            vowel_indices[0]
        }
    } else {
        // Triphthongs: ươi, uôi, iêu, yêu, oai, ươu -> middle vowel
        vowel_indices[1]
    };

    letters[target_idx] = add_tone_to_vowel(letters[target_idx], tone);
}

/// Checks whether a character is any Vietnamese vowel (plain or accented).
pub fn is_vietnamese_vowel(c: char) -> bool {
    matches!(
        c,
        'a' | 'ă' | 'â' | 'e' | 'ê' | 'i' | 'o' | 'ô' | 'ơ' | 'u' | 'ư' | 'y'
        | 'A' | 'Ă' | 'Â' | 'E' | 'Ê' | 'I' | 'O' | 'Ô' | 'Ơ' | 'U' | 'Ư' | 'Y'
        | 'á' | 'à' | 'ả' | 'ã' | 'ạ'
        | 'ắ' | 'ằ' | 'ẳ' | 'ẵ' | 'ặ'
        | 'ấ' | 'ầ' | 'ẩ' | 'ẫ' | 'ậ'
        | 'é' | 'è' | 'ẻ' | 'ẽ' | 'ẹ'
        | 'ế' | 'ề' | 'ể' | 'ễ' | 'ệ'
        | 'í' | 'ì' | 'ỉ' | 'ĩ' | 'ị'
        | 'ó' | 'ò' | 'ỏ' | 'õ' | 'ọ'
        | 'ố' | 'ồ' | 'ổ' | 'ỗ' | 'ộ'
        | 'ớ' | 'ờ' | 'ở' | 'ỡ' | 'ợ'
        | 'ú' | 'ù' | 'ủ' | 'ũ' | 'ụ'
        | 'ứ' | 'ừ' | 'ử' | 'ữ' | 'ự'
        | 'ý' | 'ỳ' | 'ỷ' | 'ỹ' | 'ỵ'
        | 'Á' | 'À' | 'Ả' | 'Ã' | 'Ạ'
        | 'Ắ' | 'Ằ' | 'Ẳ' | 'Ẵ' | 'Ặ'
        | 'Ấ' | 'Ầ' | 'Ẩ' | 'Ẫ' | 'Ậ'
        | 'É' | 'È' | 'Ẻ' | 'Ẽ' | 'Ẹ'
        | 'Ế' | 'Ề' | 'Ể' | 'Ễ' | 'Ệ'
        | 'Í' | 'Ì' | 'Ỉ' | 'Ĩ' | 'Ị'
        | 'Ó' | 'Ò' | 'Ỏ' | 'Õ' | 'Ọ'
        | 'Ố' | 'Ồ' | 'Ổ' | 'Ỗ' | 'Ộ'
        | 'Ớ' | 'Ờ' | 'Ở' | 'Ỡ' | 'Ợ'
        | 'Ú' | 'Ù' | 'Ủ' | 'Ũ' | 'Ụ'
        | 'Ứ' | 'Ừ' | 'Ử' | 'Ữ' | 'Ự'
        | 'Ý' | 'Ỳ' | 'Ỷ' | 'Ỹ' | 'Ỵ'
    )
}

/// Strips any diacritical tone mark from a vowel.
pub fn strip_tone(c: char) -> char {
    match c {
        'á' | 'à' | 'ả' | 'ã' | 'ạ' => 'a',
        'ắ' | 'ằ' | 'ẳ' | 'ẵ' | 'ặ' => 'ă',
        'ấ' | 'ầ' | 'ẩ' | 'ẫ' | 'ậ' => 'â',
        'é' | 'è' | 'ẻ' | 'ẽ' | 'ẹ' => 'e',
        'ế' | 'ề' | 'ể' | 'ễ' | 'ệ' => 'ê',
        'í' | 'ì' | 'ỉ' | 'ĩ' | 'ị' => 'i',
        'ó' | 'ò' | 'ỏ' | 'õ' | 'ọ' => 'o',
        'ố' | 'ồ' | 'ổ' | 'ỗ' | 'ộ' => 'ô',
        'ớ' | 'ờ' | 'ở' | 'ỡ' | 'ợ' => 'ơ',
        'ú' | 'ù' | 'ủ' | 'ũ' | 'ụ' => 'u',
        'ứ' | 'ừ' | 'ử' | 'ữ' | 'ự' => 'ư',
        'ý' | 'ỳ' | 'ỷ' | 'ỹ' | 'ỵ' => 'y',
        'Á' | 'À' | 'Ả' | 'Ã' | 'Ạ' => 'A',
        'Ắ' | 'Ằ' | 'Ẳ' | 'Ẵ' | 'Ặ' => 'Ă',
        'Ấ' | 'Ầ' | 'Ẩ' | 'Ẫ' | 'Ậ' => 'Â',
        'É' | 'È' | 'Ẻ' | 'Ẽ' | 'Ẹ' => 'E',
        'Ế' | 'Ề' | 'Ể' | 'Ễ' | 'Ệ' => 'Ê',
        'Í' | 'Ì' | 'Ỉ' | 'Ĩ' | 'Ị' => 'I',
        'Ó' | 'Ò' | 'Ỏ' | 'Õ' | 'Ọ' => 'O',
        'Ố' | 'Ồ' | 'Ổ' | 'Ỗ' | 'Ộ' => 'Ô',
        'Ớ' | 'Ờ' | 'Ở' | 'Ỡ' | 'Ợ' => 'Ơ',
        'Ú' | 'Ù' | 'Ủ' | 'Ũ' | 'Ụ' => 'U',
        'Ứ' | 'Ừ' | 'Ử' | 'Ữ' | 'Ự' => 'Ư',
        'Ý' | 'Ỳ' | 'Ỷ' | 'Ỹ' | 'Ỵ' => 'Y',
        _ => c,
    }
}

/// Applies tone mark (1=Sắc, 2=Huyền, 3=Hỏi, 4=Ngã, 5=Nặng) to a vowel.
pub fn add_tone_to_vowel(v: char, tone: usize) -> char {
    let base = strip_tone(v);
    match (base, tone) {
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

        ('A', 1) => 'Á', ('A', 2) => 'À', ('A', 3) => 'Ả', ('A', 4) => 'Ã', ('A', 5) => 'Ạ',
        ('Ă', 1) => 'Ắ', ('Ă', 2) => 'Ằ', ('Ă', 3) => 'Ẳ', ('Ă', 4) => 'Ẵ', ('Ă', 5) => 'Ặ',
        ('Â', 1) => 'Ấ', ('Â', 2) => 'Ầ', ('Â', 3) => 'Ẩ', ('Â', 4) => 'Ẫ', ('Â', 5) => 'Ậ',
        ('E', 1) => 'É', ('E', 2) => 'È', ('E', 3) => 'Ẻ', ('E', 4) => 'Ẽ', ('E', 5) => 'Ẹ',
        ('Ê', 1) => 'Ế', ('Ê', 2) => 'Ề', ('Ê', 3) => 'Ể', ('Ê', 4) => 'Ễ', ('Ê', 5) => 'Ệ',
        ('I', 1) => 'Í', ('I', 2) => 'Ì', ('I', 3) => 'Ỉ', ('I', 4) => 'Ĩ', ('I', 5) => 'Ị',
        ('O', 1) => 'Ó', ('O', 2) => 'Ò', ('O', 3) => 'Ỏ', ('O', 4) => 'Õ', ('O', 5) => 'Ọ',
        ('Ô', 1) => 'Ố', ('Ô', 2) => 'Ồ', ('Ô', 3) => 'Ổ', ('Ô', 4) => 'Ỗ', ('Ô', 5) => 'Ộ',
        ('Ơ', 1) => 'Ớ', ('Ơ', 2) => 'Ờ', ('Ơ', 3) => 'Ở', ('Ơ', 4) => 'Ỡ', ('Ơ', 5) => 'Ợ',
        ('U', 1) => 'Ú', ('U', 2) => 'Ù', ('U', 3) => 'Ủ', ('U', 4) => 'Ũ', ('U', 5) => 'Ụ',
        ('Ư', 1) => 'Ứ', ('Ư', 2) => 'Ừ', ('Ư', 3) => 'Ử', ('Ư', 4) => 'Ữ', ('Ư', 5) => 'Ự',
        ('Y', 1) => 'Ý', ('Y', 2) => 'Ỳ', ('Y', 3) => 'Ỷ', ('Y', 4) => 'Ỹ', ('Y', 5) => 'Ỵ',
        _ => base,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telex_duoc() {
        let telex = TelexEngine::new(ToneStyle::Modern);
        // Delayed horn and tone: dduocwj -> được
        let (res, modif) = telex.feed("dduocwj");
        assert_eq!(res, "được");
        assert!(modif);

        // Inline horn, delayed tone: dduowcj -> được
        let (res, modif) = telex.feed("dduowcj");
        assert_eq!(res, "được");
        assert!(modif);

        // Delayed horn without tone: duocw -> dươc
        let (res, modif) = telex.feed("duocw");
        assert_eq!(res, "dươc");
        assert!(modif);
    }

    #[test]
    fn test_telex_common_words() {
        let telex = TelexEngine::new(ToneStyle::Modern);
        assert_eq!(telex.feed("tinhf").0, "tình");
        assert_eq!(telex.feed("yeeu").0, "yêu");
        assert_eq!(telex.feed("daaus").0, "dấu");
        assert_eq!(telex.feed("nguowif").0, "người");
        assert_eq!(telex.feed("nguoiwf").0, "người");
        assert_eq!(telex.feed("vieetj").0, "việt");
        assert_eq!(telex.feed("muoons").0, "muốn");
        assert_eq!(telex.feed("cuoocj").0, "cuộc");
        assert_eq!(telex.feed("thuwsc").0, "thức");
    }

    #[test]
    fn test_telex_bracket_transform() {
        let telex_bracket = TelexEngine::with_options(ToneStyle::Modern, W2UType::NonStart, true);
        assert_eq!(telex_bracket.feed("[").0, "ơ");
        assert_eq!(telex_bracket.feed("]").0, "ư");
        assert_eq!(telex_bracket.feed("{").0, "Ơ");
        assert_eq!(telex_bracket.feed("}").0, "Ư");
        // Pressing twice returns raw bracket
        assert_eq!(telex_bracket.feed("[[").0, "[");
        assert_eq!(telex_bracket.feed("]]").0, "]");
    }

    #[test]
    fn test_telex_standalone_w_and_initial_consonants() {
        let telex_non_start = TelexEngine::with_options(ToneStyle::Modern, W2UType::NonStart, false);
        // In NonStart mode, initial 'w' remains 'w' (e.g. windows, web)
        assert_eq!(telex_non_start.feed("w").0, "w");
        assert_eq!(telex_non_start.feed("sa").0, "sa");
        assert_eq!(telex_non_start.feed("sang").0, "sang");
        assert_eq!(telex_non_start.feed("trang").0, "trang");
        assert_eq!(telex_non_start.feed("xa").0, "xa");

        // In Always mode, initial 'w' produces 'ư'
        let telex_always = TelexEngine::with_options(ToneStyle::Modern, W2UType::Always, false);
        assert_eq!(telex_always.feed("w").0, "ư");
    }
}
