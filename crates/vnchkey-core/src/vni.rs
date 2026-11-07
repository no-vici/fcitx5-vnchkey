//! VNI Input Method Transformer.

use crate::engine::ToneStyle;
use crate::telex::{
    apply_smart_uo_horn, can_modify_syllable, compose_vowel, decompose_vowel, extract_tone,
    find_adjacent_uo, find_adjacent_uu, has_effective_vowel, is_qu_glide, is_vietnamese_vowel,
    rebalance_tone, strip_tone, vowel_family,
};

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

    /// Incrementally applies a single typed character `ch` to the current displayed `word`.
    /// Returns `(new_word, was_transformed, was_manual_undo)`.
    pub fn apply_key(&self, word: &str, ch: char) -> (String, bool, bool) {
        let mut letters: Vec<char> = word.chars().collect();
        let current_tone = extract_tone(&letters);

        // 1. Bracket transformation: [ -> ơ, ] -> ư, { -> Ơ, } -> Ư
        if self.bracket_transform {
            let bracket_target = match ch {
                '[' => Some('ơ'),
                ']' => Some('ư'),
                '{' => Some('Ơ'),
                '}' => Some('Ư'),
                _ => None,
            };
            if let Some(target_char) = bracket_target {
                if let Some(&last) = letters.last() {
                    if strip_tone(last) == target_char {
                        letters.pop();
                        letters.push(ch);
                        return (letters.into_iter().collect(), true, true);
                    }
                }
                letters.push(target_char);
                if current_tone > 0 && can_modify_syllable(&letters) {
                    rebalance_tone(&mut letters, current_tone, self.tone_style);
                }
                return (letters.into_iter().collect(), true, false);
            }
        }

        let has_vowel = letters.iter().any(|&c| is_vietnamese_vowel(c));

        // 2. Pre-vowel stroke: d9 -> đ, đ9 -> d9
        if !has_vowel {
            if ch == '9' && letters.len() == 1 {
                let first = letters[0];
                if first == 'd' || first == 'D' {
                    letters[0] = if first.is_uppercase() { 'Đ' } else { 'đ' };
                    return (letters.into_iter().collect(), true, false);
                } else if first == 'đ' || first == 'Đ' {
                    letters[0] = if first.is_uppercase() { 'D' } else { 'd' };
                    letters.push('9');
                    return (letters.into_iter().collect(), true, true);
                }
            }
            letters.push(ch);
            return (letters.into_iter().collect(), false, false);
        }

        // 3. Digit modifiers when at least one vowel is present
        if ch.is_ascii_digit() && can_modify_syllable(&letters) {
            match ch {
                '0'..='5' => {
                    let tone = ch.to_digit(10).unwrap() as usize;
                    if has_effective_vowel(&letters) {
                        if tone == 0 {
                            if current_tone > 0 {
                                rebalance_tone(&mut letters, 0, self.tone_style);
                                return (letters.into_iter().collect(), true, false);
                            } else {
                                letters.push(ch);
                                return (letters.into_iter().collect(), false, false);
                            }
                        } else if current_tone == tone {
                            // Pressing the same tone number again removes the tone and outputs the digit
                            rebalance_tone(&mut letters, 0, self.tone_style);
                            letters.push(ch);
                            return (letters.into_iter().collect(), true, true);
                        } else {
                            // Apply or change tone
                            apply_smart_uo_horn(&mut letters);
                            rebalance_tone(&mut letters, tone, self.tone_style);
                            return (letters.into_iter().collect(), true, false);
                        }
                    }
                }
                '6' => {
                    // Circumflex (^): a/ă -> â, e -> ê, o/ơ -> ô, ươ -> uô
                    if let Some((u_idx, o_idx)) = find_adjacent_uo(&letters) {
                        let (u_base, _, u_up) = decompose_vowel(letters[u_idx]).unwrap();
                        let (o_base, _, o_up) = decompose_vowel(letters[o_idx]).unwrap();
                        if u_base == 'ư' && o_base == 'ơ' {
                            letters[u_idx] = compose_vowel('u', 0, u_up);
                            letters[o_idx] = compose_vowel('ô', 0, o_up);
                            rebalance_tone(&mut letters, current_tone, self.tone_style);
                            return (letters.into_iter().collect(), true, false);
                        }
                    }

                    for idx in (0..letters.len()).rev() {
                        if let Some((base, _, is_up)) = decompose_vowel(letters[idx]) {
                            let fam = vowel_family(base);
                            if matches!(fam, 'a' | 'e' | 'o') {
                                let hat_base = match fam {
                                    'a' => 'â',
                                    'e' => 'ê',
                                    'o' => 'ô',
                                    _ => unreachable!(),
                                };
                                if base == hat_base {
                                    // Toggle off: revert to plain vowel and append '6'
                                    letters[idx] = compose_vowel(fam, 0, is_up);
                                    letters.push('6');
                                    rebalance_tone(&mut letters, current_tone, self.tone_style);
                                    return (letters.into_iter().collect(), true, true);
                                } else {
                                    // Promote to circumflex (preserving tone via rebalance_tone)
                                    letters[idx] = compose_vowel(hat_base, 0, is_up);
                                    rebalance_tone(&mut letters, current_tone, self.tone_style);
                                    return (letters.into_iter().collect(), true, false);
                                }
                            }
                        }
                    }
                }
                '7' => {
                    // Horn (ơ, ư): uo -> ươ, uu -> ưu, o/ô -> ơ, u -> ư
                    // Case 1: Adjacent 'uo' pair
                    if let Some((u_idx, o_idx)) = find_adjacent_uo(&letters) {
                        let (u_base, _, u_up) = decompose_vowel(letters[u_idx]).unwrap();
                        let (o_base, _, o_up) = decompose_vowel(letters[o_idx]).unwrap();

                        // Exceptions:
                        // 1. 'qu' + 'o' (e.g. 'quon' for 'quởn' / 'quỡn'): in 'qu', 'u' is glide/semi-vowel and never takes a horn ('ư').
                        // 2. open syllable 'thuo' -> 'thuơ' (for 'thuở')
                        let is_qu_o = u_idx > 0 && letters[u_idx - 1].eq_ignore_ascii_case(&'q');
                        let is_open_thuo = u_idx == 2
                            && o_idx + 1 == letters.len()
                            && letters[0].eq_ignore_ascii_case(&'t')
                            && letters[1].eq_ignore_ascii_case(&'h');

                        if is_open_thuo || is_qu_o {
                            if o_base == 'ơ' {
                                letters[o_idx] = compose_vowel('o', 0, o_up);
                                letters.push('7');
                                rebalance_tone(&mut letters, current_tone, self.tone_style);
                                return (letters.into_iter().collect(), true, true);
                            } else {
                                letters[o_idx] = compose_vowel('ơ', 0, o_up);
                                rebalance_tone(&mut letters, current_tone, self.tone_style);
                                return (letters.into_iter().collect(), true, false);
                            }
                        }

                        if u_base == 'ư' && o_base == 'ơ' {
                            letters[u_idx] = compose_vowel('u', 0, u_up);
                            letters[o_idx] = compose_vowel('o', 0, o_up);
                            letters.push('7');
                            rebalance_tone(&mut letters, current_tone, self.tone_style);
                            return (letters.into_iter().collect(), true, true);
                        } else {
                            letters[u_idx] = compose_vowel('ư', 0, u_up);
                            letters[o_idx] = compose_vowel('ơ', 0, o_up);
                            rebalance_tone(&mut letters, current_tone, self.tone_style);
                            return (letters.into_iter().collect(), true, false);
                        }
                    }

                    // Case 2: Adjacent 'uu' pair (cuu7 -> cưu, luu7 -> lưu, huu7 -> hưu)
                    if let Some((u1_idx, _u2_idx)) = find_adjacent_uu(&letters) {
                        let (u1_base, _, u1_up) = decompose_vowel(letters[u1_idx]).unwrap();
                        if u1_base == 'ư' {
                            letters[u1_idx] = compose_vowel('u', 0, u1_up);
                            letters.push('7');
                            rebalance_tone(&mut letters, current_tone, self.tone_style);
                            return (letters.into_iter().collect(), true, true);
                        } else {
                            letters[u1_idx] = compose_vowel('ư', 0, u1_up);
                            rebalance_tone(&mut letters, current_tone, self.tone_style);
                            return (letters.into_iter().collect(), true, false);
                        }
                    }

                    // Case 3: Single 'o'/'ô'/'ơ' or 'u'/'ư' (skipping 'u' in 'qu')
                    for idx in (0..letters.len()).rev() {
                        if is_qu_glide(&letters, idx) {
                            continue;
                        }
                        if let Some((base, _, is_up)) = decompose_vowel(letters[idx]) {
                            let fam = vowel_family(base);
                            if matches!(fam, 'o' | 'u') {
                                let horn_base = if fam == 'o' { 'ơ' } else { 'ư' };
                                if base == horn_base {
                                    letters[idx] = compose_vowel(fam, 0, is_up);
                                    letters.push('7');
                                    rebalance_tone(&mut letters, current_tone, self.tone_style);
                                    return (letters.into_iter().collect(), true, true);
                                } else {
                                    letters[idx] = compose_vowel(horn_base, 0, is_up);
                                    rebalance_tone(&mut letters, current_tone, self.tone_style);
                                    return (letters.into_iter().collect(), true, false);
                                }
                            }
                        }
                    }
                }
                '8' => {
                    // Breve (ă): a/â -> ă
                    for idx in (0..letters.len()).rev() {
                        if let Some((base, _, is_up)) = decompose_vowel(letters[idx]) {
                            if vowel_family(base) == 'a' {
                                if base == 'ă' {
                                    letters[idx] = compose_vowel('a', 0, is_up);
                                    letters.push('8');
                                    rebalance_tone(&mut letters, current_tone, self.tone_style);
                                    return (letters.into_iter().collect(), true, true);
                                } else {
                                    letters[idx] = compose_vowel('ă', 0, is_up);
                                    rebalance_tone(&mut letters, current_tone, self.tone_style);
                                    return (letters.into_iter().collect(), true, false);
                                }
                            }
                        }
                    }
                }
                '9' => {
                    // Stroke (đ): d -> đ, đ -> d9
                    if let Some(&first) = letters.first() {
                        if first == 'd' || first == 'D' {
                            letters[0] = if first.is_uppercase() { 'Đ' } else { 'đ' };
                            if current_tone > 0 {
                                apply_smart_uo_horn(&mut letters);
                                rebalance_tone(&mut letters, current_tone, self.tone_style);
                            }
                            return (letters.into_iter().collect(), true, false);
                        } else if first == 'đ' || first == 'Đ' {
                            letters[0] = if first.is_uppercase() { 'D' } else { 'd' };
                            letters.push('9');
                            return (letters.into_iter().collect(), true, true);
                        }
                    }
                }
                _ => {}
            }
        }

        // 4. Normal character append (vowel or consonant)
        let mut fallback = letters.clone();
        fallback.push(ch);

        letters.push(ch);
        if can_modify_syllable(&letters) && current_tone > 0 {
            let mut changed = apply_smart_uo_horn(&mut letters);
            rebalance_tone(&mut letters, current_tone, self.tone_style);
            changed |= letters != fallback;
            if changed {
                return (letters.into_iter().collect(), true, false);
            }
        }

        (letters.into_iter().collect(), false, false)
    }

    /// Transforms raw strokes using VNI rules.
    /// Returns `(transformed_string, was_transformed)`.
    pub fn feed(&self, input: &str) -> (String, bool) {
        if input.is_empty() {
            return (String::new(), false);
        }
        let mut word = String::with_capacity(input.len());
        let mut any_transformed = false;
        for ch in input.chars() {
            let (next_word, step_transformed, _) = self.apply_key(&word, ch);
            word = next_word;
            any_transformed |= step_transformed;
        }
        let really_changed = word != input || any_transformed;
        (word, really_changed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vni_common_and_gio72() {
        let vni = VniEngine::new(ToneStyle::Modern);
        assert_eq!(vni.feed("gio72").0, "giờ");
        assert_eq!(vni.feed("gio27").0, "giờ");
        assert_eq!(vni.feed("gi2o7").0, "giờ");
        assert_eq!(vni.feed("gia2").0, "già");
        assert_eq!(vni.feed("qua1").0, "quá");
        assert_eq!(vni.feed("quoc61").0, "quốc");
        assert_eq!(vni.feed("tinh2").0, "tình");
        assert_eq!(vni.feed("dau61").0, "dấu");
        assert_eq!(vni.feed("duoc975").0, "được");
        assert_eq!(vni.feed("duoc95").0, "được");
        assert_eq!(vni.feed("nuoc1").0, "nước");
        assert_eq!(vni.feed("viet65").0, "việt");
        assert_eq!(vni.feed("khong6").0, "không");
        assert_eq!(vni.feed("dzia2").0, "dzìa");
        assert_eq!(vni.feed("dzi1").0, "dzí");
        assert_eq!(vni.feed("dzo6").0, "dzô");
        assert_eq!(vni.feed("dzo61").0, "dzố");
        assert_eq!(vni.feed("dzi5").0, "dzị");
    }

    #[test]
    fn test_vni_modify_and_undo_marks() {
        let vni = VniEngine::new(ToneStyle::Modern);
        // Modify tone after typing gio72 -> giờ
        assert_eq!(vni.feed("gio721").0, "giớ");
        assert_eq!(vni.feed("gio723").0, "giở");
        assert_eq!(vni.feed("gio720").0, "giơ");
        // Double tone key toggles tone off and outputs digit
        assert_eq!(vni.feed("gio722").0, "giơ2");
        assert_eq!(vni.feed("toan11").0, "toan1");
        // Switch vowel shapes (7 <-> 6, 8 <-> 6) while preserving tone
        assert_eq!(vni.feed("gio726").0, "giồ");
        assert_eq!(vni.feed("ta816").0, "tấ");
        assert_eq!(vni.feed("ta618").0, "tắ");
        // Toggle off shape modifiers
        assert_eq!(vni.feed("gio77").0, "gio7");
        assert_eq!(vni.feed("khong66").0, "khong6");
        assert_eq!(vni.feed("d99").0, "d9");

        // ToneStyle test
        let vni_classic = VniEngine::new(ToneStyle::Classic);
        assert_eq!(vni_classic.feed("toa1").0, "tóa");
        assert_eq!(vni_classic.feed("toan1").0, "toán");
        assert_eq!(vni_classic.feed("hoa2").0, "hòa");
        assert_eq!(vni_classic.feed("thuy3").0, "thủy");

        let vni_modern = VniEngine::new(ToneStyle::Modern);
        assert_eq!(vni_modern.feed("toa1").0, "toá");
        assert_eq!(vni_modern.feed("toan1").0, "toán");
        assert_eq!(vni_modern.feed("hoa2").0, "hoà");
        assert_eq!(vni_modern.feed("thuy3").0, "thuỷ");

        // Southern dialect 'quởn' / 'quỡn' & 'thuở' in VNI
        assert_eq!(vni.feed("quon73").0, "quởn");
        assert_eq!(vni.feed("quon74").0, "quỡn");
        assert_eq!(vni.feed("thuo73").0, "thuở");
    }
}
