//! Telex Input Method Transformer & Vietnamese Orthography Core.

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

    /// Sets the tone style convention.
    pub fn set_tone_style(&mut self, tone_style: ToneStyle) {
        self.tone_style = tone_style;
    }

    /// Sets the W to Ư rule.
    pub fn set_type_w_to_u(&mut self, w2u: W2UType) {
        self.w2u = w2u;
    }

    /// Enables or disables bracket transformation.
    pub fn set_bracket_transform(&mut self, enabled: bool) {
        self.bracket_transform = enabled;
    }

    /// Incrementally applies a single typed character `ch` to the current displayed `word`.
    /// Returns `(new_word, was_transformed, was_manual_undo)`.
    pub fn apply_key(&self, word: &str, ch: char, raw_strokes: &str) -> (String, bool, bool) {
        self.apply_key_internal(word, ch, raw_strokes, self.w2u)
    }

    fn apply_key_internal(
        &self,
        word: &str,
        ch: char,
        raw_strokes: &str,
        w2u_mode: W2UType,
    ) -> (String, bool, bool) {
        let mut letters: Vec<char> = word.chars().collect();
        let current_tone = extract_tone(&letters);
        let lower = ch.to_ascii_lowercase();

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

        // 2. Pre-vowel modifiers (dd -> đ, standalone w -> ư)
        if !has_vowel {
            if lower == 'd' && letters.len() == 1 {
                let first = letters[0];
                if first == 'd' || first == 'D' {
                    letters[0] = if first.is_uppercase() || ch.is_uppercase() {
                        'Đ'
                    } else {
                        'đ'
                    };
                    return (letters.into_iter().collect(), true, false);
                } else if first == 'đ' || first == 'Đ' {
                    letters[0] = if first.is_uppercase() { 'D' } else { 'd' };
                    letters.push(ch);
                    return (letters.into_iter().collect(), true, true);
                }
            }

            if lower == 'w' {
                let is_upper = ch.is_uppercase();
                match w2u_mode {
                    W2UType::Disabled => {
                        letters.push(ch);
                        return (letters.into_iter().collect(), false, false);
                    }
                    W2UType::NonStart => {
                        if !letters.is_empty() && is_valid_vietnamese_onset(&letters) {
                            letters.push(if is_upper { 'Ư' } else { 'ư' });
                            return (letters.into_iter().collect(), true, false);
                        }
                        letters.push(ch);
                        return (letters.into_iter().collect(), false, false);
                    }
                    W2UType::Always => {
                        if letters.is_empty() || is_valid_vietnamese_onset(&letters) {
                            letters.push(if is_upper { 'Ư' } else { 'ư' });
                            return (letters.into_iter().collect(), true, false);
                        }
                        letters.push(ch);
                        return (letters.into_iter().collect(), false, false);
                    }
                }
            }

            letters.push(ch);
            return (letters.into_iter().collect(), false, false);
        }

        // 3. When at least one vowel is present, check if the syllable structure allows modification
        if can_modify_syllable(&letters) {
            // 3a. Delayed or adjacent 'd' stroke (e.g. duocwd -> đươc, dad -> đa, đad -> dad)
            if lower == 'd' {
                if let Some(&first) = letters.first() {
                    if first == 'd' || first == 'D' {
                        letters[0] = if first.is_uppercase() || ch.is_uppercase() {
                            'Đ'
                        } else {
                            'đ'
                        };
                        if current_tone > 0 {
                            apply_smart_uo_horn(&mut letters);
                            rebalance_tone(&mut letters, current_tone, self.tone_style);
                        }
                        return (letters.into_iter().collect(), true, false);
                    } else if first == 'đ' || first == 'Đ' {
                        letters[0] = if first.is_uppercase() { 'D' } else { 'd' };
                        letters.push(ch);
                        return (letters.into_iter().collect(), true, true);
                    }
                }
            }

            // 3b. Tone keys: s (1), f (2), r (3), x (4), j (5), z (0)
            if matches!(lower, 's' | 'f' | 'r' | 'x' | 'j' | 'z') {
                let tone = match lower {
                    's' => 1,
                    'f' => 2,
                    'r' => 3,
                    'x' => 4,
                    'j' => 5,
                    _ => 0,
                };

                // Only apply tone if there is at least one effective vowel
                // (e.g., 'qu' alone has 'u' as glide, not a standalone vowel nucleus)
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
                        // Pressing the same tone key again removes the tone and outputs literal key
                        rebalance_tone(&mut letters, 0, self.tone_style);
                        letters.push(ch);
                        return (letters.into_iter().collect(), true, true);
                    } else {
                        // Adding or changing tone
                        let mut test_letters = letters.clone();
                        apply_smart_uo_horn(&mut test_letters);
                        let word_str: String = test_letters.iter().collect();
                        if let Some(parsed) = crate::syllable_parser::parse_syllable(&word_str) {
                            let new_tone = crate::syllable_parser::ToneMark::from_index(tone);
                            if !parsed.with_tone(new_tone).is_valid() {
                                // Phonetic violation (e.g. stop coda -c, -ch, -p, -t taking huyền, hỏi, ngã)
                                letters.push(ch);
                                return (letters.into_iter().collect(), false, false);
                            }
                        }

                        apply_smart_uo_horn(&mut letters);
                        rebalance_tone(&mut letters, tone, self.tone_style);
                        return (letters.into_iter().collect(), true, false);
                    }
                }
            }

            // 3c. Circumflex modifiers: 'a' -> 'â', 'e' -> 'ê', 'o' -> 'ô'
            // Supports both inline (aa, ee, oo) and end-of-word (khongo -> không, toana -> toân, tienge -> tiêng)
            if matches!(lower, 'a' | 'e' | 'o') {
                // Special case for 'o': if syllable has 'ươ' (e.g. dươc + o -> duôc), convert 'ươ' -> 'uô'
                if lower == 'o' {
                    if let Some((u_idx, o_idx)) = find_adjacent_uo(&letters) {
                        let (u_base, _, u_up) = decompose_vowel(letters[u_idx]).unwrap();
                        let (o_base, _, o_up) = decompose_vowel(letters[o_idx]).unwrap();
                        if u_base == 'ư' && o_base == 'ơ' {
                            letters[u_idx] = compose_vowel('u', 0, u_up);
                            letters[o_idx] = compose_vowel('ô', 0, o_up || ch.is_uppercase());
                            rebalance_tone(&mut letters, current_tone, self.tone_style);
                            return (letters.into_iter().collect(), true, false);
                        }
                    }
                }

                let matching: Vec<usize> = letters
                    .iter()
                    .enumerate()
                    .filter_map(|(i, &c)| {
                        decompose_vowel(c).and_then(|(base, _, _)| {
                            if vowel_family(base) == lower {
                                Some(i)
                            } else {
                                None
                            }
                        })
                    })
                    .collect();

                if matching.len() == 1 {
                    let idx = matching[0];
                    let (base, _, is_up) = decompose_vowel(letters[idx]).unwrap();
                    let hat_base = match lower {
                        'a' => 'â',
                        'e' => 'ê',
                        'o' => 'ô',
                        _ => unreachable!(),
                    };

                    if base == hat_base {
                        // Already circumflexed -> revert to plain vowel and append literal key (e.g. không + o -> khongo)
                        letters[idx] = compose_vowel(lower, 0, is_up);
                        letters.push(ch);
                        rebalance_tone(&mut letters, current_tone, self.tone_style);
                        return (letters.into_iter().collect(), true, true);
                    } else {
                        // Promote plain or horn/breve vowel to circumflex
                        letters[idx] = compose_vowel(hat_base, 0, is_up || ch.is_uppercase());
                        rebalance_tone(&mut letters, current_tone, self.tone_style);
                        return (letters.into_iter().collect(), true, false);
                    }
                }
            }

            // 3d. Horn & Breve modifier: 'w' / 'W'
            if lower == 'w' {
                // Case 1: Adjacent 'uo' pair (duocw -> dươc, thuow -> thuơ, thuongw -> thương)
                if let Some((u_idx, o_idx)) = find_adjacent_uo(&letters) {
                    let (_, _, u_up) = decompose_vowel(letters[u_idx]).unwrap();
                    let (o_base, _, o_up) = decompose_vowel(letters[o_idx]).unwrap();
                    let (u_base, _, _) = decompose_vowel(letters[u_idx]).unwrap();

                    // Exceptions:
                    // 1. 'qu' + 'o' (e.g. 'quon' for 'quởn' / 'quỡn', 'quo' for 'quơ'): in 'qu', 'u' is glide/semi-vowel and never takes a horn ('ư').
                    // 2. open syllable 'thuo' -> 'thuơ' (for 'thuở')
                    // 3. open syllable 'huo' -> 'huơ' (for 'huơ tay')
                    let is_qu_o = u_idx > 0 && letters[u_idx - 1].eq_ignore_ascii_case(&'q');
                    let is_open_thuo = u_idx == 2
                        && o_idx + 1 == letters.len()
                        && letters[0].eq_ignore_ascii_case(&'t')
                        && letters[1].eq_ignore_ascii_case(&'h');
                    let is_open_huo = u_idx == 1
                        && o_idx + 1 == letters.len()
                        && letters[0].eq_ignore_ascii_case(&'h');

                    if is_open_thuo || is_open_huo || is_qu_o {
                        if o_base == 'ơ' {
                            letters[o_idx] = compose_vowel('o', 0, o_up);
                            letters.push(ch);
                            rebalance_tone(&mut letters, current_tone, self.tone_style);
                            return (letters.into_iter().collect(), true, true);
                        } else {
                            letters[o_idx] = compose_vowel('ơ', 0, o_up);
                            rebalance_tone(&mut letters, current_tone, self.tone_style);
                            return (letters.into_iter().collect(), true, false);
                        }
                    }

                    if u_base == 'ư' && o_base == 'ơ' {
                        // Toggle off: 'ươ' -> 'uo' + 'w'
                        letters[u_idx] = compose_vowel('u', 0, u_up);
                        letters[o_idx] = compose_vowel('o', 0, o_up);
                        letters.push(ch);
                        rebalance_tone(&mut letters, current_tone, self.tone_style);
                        return (letters.into_iter().collect(), true, true);
                    } else {
                        // Promote to 'ươ' (harmonize casing: 'Ư' only if 'o' is also uppercase)
                        let target_u_up = u_up && (o_up || ch.is_uppercase());
                        letters[u_idx] = compose_vowel('ư', 0, target_u_up);
                        letters[o_idx] = compose_vowel('ơ', 0, o_up || ch.is_uppercase());
                        rebalance_tone(&mut letters, current_tone, self.tone_style);
                        return (letters.into_iter().collect(), true, false);
                    }
                }

                // Case 2: Adjacent 'uu' pair (cuuw -> cưu, luuw -> lưu, huuw -> hưu)
                if let Some((u1_idx, _u2_idx)) = find_adjacent_uu(&letters) {
                    let (u1_base, _, u1_up) = decompose_vowel(letters[u1_idx]).unwrap();
                    if u1_base == 'ư' {
                        letters[u1_idx] = compose_vowel('u', 0, u1_up);
                        letters.push(ch);
                        rebalance_tone(&mut letters, current_tone, self.tone_style);
                        return (letters.into_iter().collect(), true, true);
                    } else {
                        letters[u1_idx] = compose_vowel('ư', 0, u1_up);
                        rebalance_tone(&mut letters, current_tone, self.tone_style);
                        return (letters.into_iter().collect(), true, false);
                    }
                }

                // Case 3: Open syllable 'ua' not preceded by 'q' (muaw -> mưa, chuaw -> chưa, thuaw -> thưa)
                if let Some((u_idx, a_idx)) = find_open_ua_non_qu(&letters) {
                    let (u_base, _, u_up) = decompose_vowel(letters[u_idx]).unwrap();
                    let (a_base, _, _) = decompose_vowel(letters[a_idx]).unwrap();
                    if a_base == 'a' {
                        if u_base == 'ư' {
                            letters[u_idx] = compose_vowel('u', 0, u_up);
                            letters.push(ch);
                            rebalance_tone(&mut letters, current_tone, self.tone_style);
                            return (letters.into_iter().collect(), true, true);
                        } else {
                            letters[u_idx] = compose_vowel('ư', 0, u_up);
                            rebalance_tone(&mut letters, current_tone, self.tone_style);
                            return (letters.into_iter().collect(), true, false);
                        }
                    }
                }

                // Case 4: Single vowel in 'a', 'o', 'u' family (searching backwards, skipping 'u' in 'qu')
                for idx in (0..letters.len()).rev() {
                    if is_qu_glide(&letters, idx) {
                        continue;
                    }
                    if let Some((base, _, is_up)) = decompose_vowel(letters[idx]) {
                        let fam = vowel_family(base);
                        if matches!(fam, 'a' | 'o' | 'u') {
                            let target_base = match fam {
                                'a' => 'ă',
                                'o' => 'ơ',
                                'u' => 'ư',
                                _ => unreachable!(),
                            };
                            if base == target_base {
                                // Check if this 'ư' at the end of the word was created by standalone 'w'
                                let raw_has_u = raw_strokes
                                    .chars()
                                    .any(|c| c.eq_ignore_ascii_case(&'u'));
                                if fam == 'u' && idx + 1 == letters.len() && !raw_has_u {
                                    letters[idx] = if is_up { 'W' } else { 'w' };
                                    rebalance_tone(&mut letters, current_tone, self.tone_style);
                                    return (letters.into_iter().collect(), true, true);
                                }
                                // Otherwise revert to plain base and append 'w'
                                letters[idx] = compose_vowel(fam, 0, is_up);
                                letters.push(ch);
                                rebalance_tone(&mut letters, current_tone, self.tone_style);
                                return (letters.into_iter().collect(), true, true);
                            } else {
                                letters[idx] = compose_vowel(target_base, 0, is_up);
                                rebalance_tone(&mut letters, current_tone, self.tone_style);
                                return (letters.into_iter().collect(), true, false);
                            }
                        }
                    }
                }

                // Case 5: Syllable only has 'i', 'e', 'y' (e.g. 'gi' + 'w' -> 'giư')
                if w2u_mode != W2UType::Disabled
                    && letters.len() == 2
                    && letters[0].eq_ignore_ascii_case(&'g')
                    && letters[1].eq_ignore_ascii_case(&'i')
                {
                    letters.push(if ch.is_uppercase() { 'Ư' } else { 'ư' });
                    rebalance_tone(&mut letters, current_tone, self.tone_style);
                    return (letters.into_iter().collect(), true, false);
                }
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

    /// Transforms raw strokes using Telex rules.
    /// Returns `(transformed_string, was_transformed)`.
    pub fn feed(&self, input: &str) -> (String, bool) {
        self.feed_with_mode(input, self.w2u)
    }

    /// Feeds Simple Telex (`w` does not produce standalone `ư`).
    pub fn feed_simple(&self, input: &str) -> (String, bool) {
        self.feed_with_mode(input, W2UType::Disabled)
    }

    fn feed_with_mode(&self, input: &str, w2u_mode: W2UType) -> (String, bool) {
        if input.is_empty() {
            return (String::new(), false);
        }
        let mut word = String::with_capacity(input.len());
        let mut raw = String::with_capacity(input.len());
        let mut any_transformed = false;
        for ch in input.chars() {
            let (next_word, step_transformed, _) =
                self.apply_key_internal(&word, ch, &raw, w2u_mode);
            raw.push(ch);
            word = next_word;
            any_transformed |= step_transformed;
        }
        let really_changed = word != input || any_transformed;
        (word, really_changed)
    }
}

/// Finds adjacent `u`/`ư` (not preceded by `q`) and `o`/`ô`/`ơ` in `letters`.
pub fn find_adjacent_uo(letters: &[char]) -> Option<(usize, usize)> {
    for i in 0..letters.len().saturating_sub(1) {
        if is_qu_glide(letters, i) {
            continue;
        }
        if let (Some((b1, _, _)), Some((b2, _, _))) =
            (decompose_vowel(letters[i]), decompose_vowel(letters[i + 1]))
        {
            if vowel_family(b1) == 'u' && vowel_family(b2) == 'o' {
                return Some((i, i + 1));
            }
        }
    }
    None
}

/// Finds adjacent `u`/`ư` (not preceded by `q`) and `u` in `letters`.
pub fn find_adjacent_uu(letters: &[char]) -> Option<(usize, usize)> {
    for i in 0..letters.len().saturating_sub(1) {
        if is_qu_glide(letters, i) {
            continue;
        }
        if let (Some((b1, _, _)), Some((b2, _, _))) =
            (decompose_vowel(letters[i]), decompose_vowel(letters[i + 1]))
        {
            if vowel_family(b1) == 'u' && b2 == 'u' {
                return Some((i, i + 1));
            }
        }
    }
    None
}

/// Finds open syllable `ua`/`ưa` (at the end of `letters`) not preceded by `q`.
fn find_open_ua_non_qu(letters: &[char]) -> Option<(usize, usize)> {
    if letters.len() < 2 {
        return None;
    }
    let u_idx = letters.len() - 2;
    let a_idx = letters.len() - 1;
    if is_qu_glide(letters, u_idx) {
        return None;
    }
    if let (Some((b1, _, _)), Some((b2, _, _))) = (
        decompose_vowel(letters[u_idx]),
        decompose_vowel(letters[a_idx]),
    ) {
        if vowel_family(b1) == 'u' && vowel_family(b2) == 'a' {
            return Some((u_idx, a_idx));
        }
    }
    None
}

/// Checks if `letters[idx]` is the `'u'` glide in a `'qu'` onset cluster followed by another vowel.
pub fn is_qu_glide(letters: &[char], idx: usize) -> bool {
    if idx == 1 && letters[0].eq_ignore_ascii_case(&'q') {
        if let Some((base, _, _)) = decompose_vowel(letters[1]) {
            if vowel_family(base) == 'u' {
                // It is a glide if another vowel follows it, or by itself in 'qu'
                return true;
            }
        }
    }
    false
}

/// Checks if `letters` has at least one vowel that can carry a tone (not just bare `qu`).
pub fn has_effective_vowel(letters: &[char]) -> bool {
    if letters.len() == 2
        && letters[0].eq_ignore_ascii_case(&'q')
        && letters[1].eq_ignore_ascii_case(&'u')
    {
        return false;
    }
    letters.iter().any(|&c| is_vietnamese_vowel(c))
}

/// Automatically promotes plain `uo` -> `ươ` when followed by a consonant coda (`duoc` -> `dươc`).
pub fn apply_smart_uo_horn(letters: &mut [char]) -> bool {
    if let Some((u_idx, o_idx)) = find_adjacent_uo(letters) {
        // Exception 1: 'qu' + 'o' (e.g. 'quon' for 'quởn' / 'quỡn')
        // In 'qu', 'u' is glide/semi-vowel and never takes a horn ('ư').
        if u_idx > 0 && letters[u_idx - 1].eq_ignore_ascii_case(&'q') {
            let (o_base, o_tone, o_up) = decompose_vowel(letters[o_idx]).unwrap();
            let has_consonant_coda =
                o_idx + 1 < letters.len() && !is_vietnamese_vowel(letters[o_idx + 1]);
            if has_consonant_coda && o_base == 'o' {
                letters[o_idx] = compose_vowel('ơ', o_tone, o_up);
                return true;
            }
            return false;
        }

        // Exception 2: open syllable 'thuo' (for 'thuở') or 'huo' (for 'huơ')
        let is_open_thuo = u_idx == 2
            && o_idx + 1 == letters.len()
            && letters[0].eq_ignore_ascii_case(&'t')
            && letters[1].eq_ignore_ascii_case(&'h');
        let is_open_huo = u_idx == 1
            && o_idx + 1 == letters.len()
            && letters[0].eq_ignore_ascii_case(&'h');
        if is_open_thuo || is_open_huo {
            return false;
        }

        let has_consonant_coda =
            o_idx + 1 < letters.len() && !is_vietnamese_vowel(letters[o_idx + 1]);
        if has_consonant_coda {
            // Natural Vietnamese Phonology: Plain 'uo' before a coda (e.g. 'duong', 'tuong', 'buong', 'nuoc')
            // cannot take a tone mark directly unless the user explicitly horned 'ươ' (via 'w' or '7')
            // or circumflexed 'uô' (via 'aa'/'oo'/'6').
            // Exception: The highly frequent shorthand for 'được' (where onset is already 'đ'/'Đ' and coda is 'c')
            // allows auto-promoting 'đ...uoc' -> 'đ...ược' when adding tone (e.g. dduocj -> được, duoc95 -> được).
            let is_dd_uoc = u_idx == 1
                && (letters[0] == 'đ' || letters[0] == 'Đ')
                && letters[o_idx + 1..].iter().collect::<String>().eq_ignore_ascii_case("c");
            if !is_dd_uoc {
                return false;
            }

            let (u_base, u_tone, u_up) = decompose_vowel(letters[u_idx]).unwrap();
            let (o_base, o_tone, o_up) = decompose_vowel(letters[o_idx]).unwrap();
            // Only auto-promote if 'o' is not explicitly circumflexed ('ô' as in 'cuộc', 'muốn')
            if o_base != 'ô' && (u_base == 'u' || o_base == 'o') {
                // TitleCase / Shift-bleed guard: if 'o' (and rest of syllable) is lowercase,
                // 'u' cannot be uppercase 'Ư' in a TitleCase word (e.g. 'DUong' / 'Duong' -> 'Đường', never 'ĐƯờng').
                let target_u_up = u_up && o_up;
                letters[u_idx] = compose_vowel('ư', u_tone, target_u_up);
                letters[o_idx] = compose_vowel('ơ', o_tone, o_up);
                return true;
            }
        }
    }
    false
}

/// Extracts the active tone (`0..=5`) from any accented vowel in `letters`.
pub fn extract_tone(letters: &[char]) -> usize {
    for &c in letters {
        if let Some((_, tone, _)) = decompose_vowel(c) {
            if tone > 0 {
                return tone;
            }
        }
    }
    0
}

/// Strips existing tones from all vowels in `letters` and places `tone` on the orthographically correct vowel.
/// Powered by the next-gen Vietnamese syllable parser (`syllable_parser.rs`).
pub fn rebalance_tone(letters: &mut [char], tone: usize, tone_style: ToneStyle) {
    let word: String = letters.iter().collect();
    if let Some(parsed) = crate::syllable_parser::parse_syllable(&word) {
        if !parsed.nucleus.is_empty() {
            let tone_mark = crate::syllable_parser::ToneMark::from_index(tone);
            let reconstructed = parsed.with_tone(tone_mark).reconstruct(tone_style);
            let recon_chars: Vec<char> = reconstructed.chars().collect();
            if recon_chars.len() == letters.len() {
                letters.copy_from_slice(&recon_chars);
                return;
            }
        }
    }

    for c in letters.iter_mut() {
        if let Some((base, _, is_up)) = decompose_vowel(*c) {
            *c = compose_vowel(base, 0, is_up);
        }
    }
    if tone > 0 {
        place_tone(letters, tone, tone_style);
    }
}

/// Places tone on the correct Vietnamese vowel index according to standard orthography rules.
pub fn place_tone(letters: &mut [char], tone: usize, tone_style: ToneStyle) {
    if tone == 0 || letters.is_empty() {
        return;
    }

    let mut all_vowels = Vec::new();
    for (idx, &c) in letters.iter().enumerate() {
        if is_vietnamese_vowel(c) {
            all_vowels.push(idx);
        }
    }

    if all_vowels.is_empty() {
        return;
    }

    // Filter out onset glides ('i' in 'gi' + vowel, and 'u' in 'qu' + vowel)
    let mut effective_vowels = all_vowels.clone();
    if effective_vowels.len() > 1 && effective_vowels[0] == 1 {
        let first_char = letters[0].to_ascii_lowercase();
        let (second_base, _, _) = decompose_vowel(letters[1]).unwrap();
        if (first_char == 'g' && vowel_family(second_base) == 'i')
            || (first_char == 'q' && vowel_family(second_base) == 'u')
        {
            effective_vowels.remove(0);
        }
    }

    let target_idx = if effective_vowels.len() == 1 {
        effective_vowels[0]
    } else if let Some(&marked_idx) = effective_vowels.iter().find(|&&idx| {
        matches!(
            decompose_vowel(letters[idx]).map(|(b, _, _)| b),
            Some('ă' | 'â' | 'ê' | 'ô' | 'ơ')
        )
    }) {
        // Primary diacritic vowels (ă, â, ê, ô, ơ) ALWAYS take the tone (e.g. giờ, được, người, viện)
        marked_idx
    } else if let Some(&horn_u_idx) = effective_vowels
        .iter()
        .find(|&&idx| matches!(decompose_vowel(letters[idx]).map(|(b, _, _)| b), Some('ư')))
    {
        // Horn 'ư' when no 'ơ' is present (e.g. mưa, cứu, gửi, giữ)
        horn_u_idx
    } else if effective_vowels.len() == 2 {
        let idx0 = effective_vowels[0];
        let idx1 = effective_vowels[1];
        let b0 = decompose_vowel(letters[idx0]).unwrap().0;
        let b1 = decompose_vowel(letters[idx1]).unwrap().0;

        let has_coda = idx1 + 1 < letters.len() && !is_vietnamese_vowel(letters[idx1 + 1]);
        if has_coda {
            // Closed syllable with 2 vowels (toan, hoang, quynh): tone goes on second vowel
            idx1
        } else if (b0 == 'o' && b1 == 'a') || (b0 == 'o' && b1 == 'e') || (b0 == 'u' && b1 == 'y') {
            if tone_style == ToneStyle::Modern {
                idx1 // hoà, khoẻ, thuỷ
            } else {
                idx0 // hòa, khỏe, thủy
            }
        } else {
            // Open diphthongs (ia, ua, ai, ao, au, ay, eo, iu, oi, ui...): tone on first vowel
            idx0
        }
    } else {
        // Triphthongs (oai, uay, oeo, uoi, yeu...): middle vowel
        effective_vowels[1]
    };

    if let Some((base, _, is_up)) = decompose_vowel(letters[target_idx]) {
        letters[target_idx] = compose_vowel(base, tone, is_up);
    }
}

/// Checks if a slice of characters is a valid Vietnamese onset consonant cluster.
pub fn is_valid_vietnamese_onset(chars: &[char]) -> bool {
    let raw: String = chars.iter().flat_map(|c| c.to_lowercase()).collect();
    let s = crate::vietnamese_dict::normalize_apostrophe(&raw);
    matches!(
        s.as_str(),
        "" | "b"
            | "c"
            | "ch"
            | "d"
            | "đ"
            | "dz"
            | "g"
            | "gh"
            | "gi"
            | "h"
            | "k"
            | "kh"
            | "l"
            | "m"
            | "n"
            | "ng"
            | "ngh"
            | "nh"
            | "p"
            | "ph"
            | "q"
            | "qu"
            | "r"
            | "s"
            | "t"
            | "th"
            | "tr"
            | "v"
            | "x"
            // Tay Nguyen & ethnic minority loan / dialect onsets:
            | "kr"
            | "pl"
            | "pr"
            | "gr"
            | "dr"
            | "đr"
            | "gl"
            | "br"
            | "kl"
            | "bl"
            | "kn"
            | "ml"
            | "mr"
            | "sr"
            | "sl"
            | "fl"
            | "z"
            // Onsets with apostrophe:
            | "m'"
            | "h'"
            | "r'"
            | "d'"
            | "m'đ"
            | "m'đr"
            | "m'dr"
            | "h'd"
            | "h'dr"
    )
}

/// Checks if a slice of characters is a valid Vietnamese coda consonant cluster.
pub fn is_valid_vietnamese_coda(chars: &[char]) -> bool {
    let s: String = chars.iter().flat_map(|c| c.to_lowercase()).collect();
    matches!(
        s.as_str(),
        "" | "c"
            | "ch"
            | "m"
            | "n"
            | "ng"
            | "nh"
            | "p"
            | "t"
            // Tay Nguyen & ethnic minority loan codas (Đắk, Lắk, Păh, Kar, Mil):
            | "k"
            | "h"
            | "r"
            | "l"
    )
}

/// Checks if the current `letters` form a modifiable in-progress Vietnamese syllable.
pub fn can_modify_syllable(letters: &[char]) -> bool {
    let first_v = match letters.iter().position(|&c| is_vietnamese_vowel(c)) {
        Some(idx) => idx,
        None => return false,
    };
    let last_v = letters.iter().rposition(|&c| is_vietnamese_vowel(c)).unwrap();

    // All characters between first_v and last_v must be vowels (contiguous vowel nucleus)
    if !letters[first_v..=last_v]
        .iter()
        .all(|&c| is_vietnamese_vowel(c))
    {
        return false;
    }

    // Check onset before first_v and coda after last_v
    is_valid_vietnamese_onset(&letters[..first_v])
        && is_valid_vietnamese_coda(&letters[last_v + 1..])
}

/// Checks if `word` obeys Vietnamese orthographic & phonotactic rules (Chánh Tả).
pub fn is_valid_vietnamese_syllable(word: &str) -> bool {
    let letters: Vec<char> = word.chars().collect();
    if letters.is_empty() {
        return true;
    }
    // Allow standalone 'đ'/'Đ' (from 'dd' abbreviation)
    if letters.len() == 1 && (letters[0] == 'đ' || letters[0] == 'Đ') {
        return true;
    }

    if !can_modify_syllable(&letters) {
        return false;
    }

    let first_v = letters.iter().position(|&c| is_vietnamese_vowel(c)).unwrap();
    let last_v = letters.iter().rposition(|&c| is_vietnamese_vowel(c)).unwrap();
    let vowel_len = last_v - first_v + 1;
    if vowel_len > 3 {
        return false;
    }

    // Reject impossible vowel combinations in Vietnamese (e.g. 'ae' in 'case', 'ey', 'ou', 'yo', 'yu')
    // Note: 'ea' (as in 'Ea H\'leo', 'Ea Kar') and 'ei' (as in 'Pleiku') are preserved for proper names.
    let vowel_bases: String = letters[first_v..=last_v]
        .iter()
        .filter_map(|&c| decompose_vowel(c).map(|(b, _, _)| b))
        .collect();
    if matches!(
        vowel_bases.as_str(),
        "ae" | "aê" | "êa" | "ey" | "ou" | "ôu" | "ơu" | "yo" | "yu" | "aa" | "ee" | "oo"
    ) {
        return false;
    }

    // Phonotactic check: stop codas (c, ch, p, t, k) cannot take Huyền (2), Hỏi (3), or Ngã (4)
    let coda: String = letters[last_v + 1..]
        .iter()
        .map(|c| c.to_ascii_lowercase())
        .collect();
    if matches!(coda.as_str(), "c" | "ch" | "p" | "t" | "k") {
        let tone = extract_tone(&letters);
        if matches!(tone, 2 | 3 | 4) {
            return false;
        }
    }

    true
}

/// Checks if `letters` can be a valid in-progress Vietnamese syllable (either an onset prefix
/// or an onset + valid vowel nucleus + optional coda).
pub fn is_valid_in_progress_syllable(letters: &[char]) -> bool {
    if letters.is_empty() {
        return true;
    }
    if letters.len() == 1 && (letters[0] == 'đ' || letters[0] == 'Đ') {
        return true;
    }

    let first_v = match letters.iter().position(|&c| is_vietnamese_vowel(c)) {
        Some(idx) => idx,
        None => return is_valid_vietnamese_onset(letters),
    };
    let last_v = letters.iter().rposition(|&c| is_vietnamese_vowel(c)).unwrap();

    if !can_modify_syllable(letters) {
        return false;
    }

    let word_str: String = letters.iter().collect();
    if crate::vietnamese_dict::has_phonology_violation(&word_str) {
        return false;
    }

    let has_coda = last_v + 1 < letters.len();

    // 'qu' cannot take a coda without a following main vowel
    if first_v == 1
        && last_v == 1
        && letters[0].eq_ignore_ascii_case(&'q')
        && has_coda
    {
        return false;
    }

    let mut effective_start = first_v;
    if last_v > first_v && first_v == 1 {
        let first_char = letters[0].to_ascii_lowercase();
        if let Some((second_base, _, _)) = decompose_vowel(letters[1]) {
            let fam = vowel_family(second_base);
            if (first_char == 'g' && fam == 'i') || (first_char == 'q' && fam == 'u') {
                effective_start = 2;
            }
        }
    }

    let eff_families: Vec<char> = letters[effective_start..=last_v]
        .iter()
        .filter_map(|&c| decompose_vowel(c).map(|(b, _, _)| vowel_family(b)))
        .collect();

    match eff_families.len() {
        0 | 1 => {}
        2 => {
            let pair = (eff_families[0], eff_families[1]);
            let valid_diphthong = matches!(
                pair,
                ('a', 'i' | 'o' | 'u' | 'y')
                    | ('e', 'o' | 'u')
                    | ('i', 'a' | 'e' | 'u')
                    | ('o', 'a' | 'e' | 'i' | 'o')
                    | ('u', 'a' | 'e' | 'i' | 'o' | 'u' | 'y')
                    | ('y', 'e')
            );
            if !valid_diphthong {
                return false;
            }
            if has_coda {
                let valid_closed = matches!(
                    pair,
                    ('i', 'e')
                        | ('y', 'e')
                        | ('u', 'o')
                        | ('o', 'a')
                        | ('o', 'e')
                        | ('u', 'a')
                        | ('u', 'e')
                        | ('u', 'y')
                        | ('o', 'o')
                );
                if !valid_closed {
                    return false;
                }
            }
        }
        3 => {
            let triple = (eff_families[0], eff_families[1], eff_families[2]);
            if has_coda {
                if triple != ('u', 'y', 'e') {
                    return false;
                }
            } else {
                let valid_triphthong = matches!(
                    triple,
                    ('i', 'e', 'u')
                        | ('y', 'e', 'u')
                        | ('o', 'a', 'i' | 'o' | 'y')
                        | ('o', 'e', 'o')
                        | ('u', 'a', 'y')
                        | ('u', 'o', 'i' | 'u')
                        | ('u', 'y', 'a' | 'e' | 'u')
                );
                if !valid_triphthong {
                    return false;
                }
            }
        }
        _ => return false,
    }

    if has_coda {
        let coda: String = letters[last_v + 1..]
            .iter()
            .map(|c| c.to_ascii_lowercase())
            .collect();
        if matches!(coda.as_str(), "c" | "ch" | "p" | "t") {
            let tone = extract_tone(letters);
            if matches!(tone, 2 | 3 | 4) {
                return false;
            }
        }
    }

    true
}

/// Extracts the starting character index of the last valid Vietnamese syllable in `word`
/// (equivalent to `extractLastSyllable` in `fcitx5-lilypad` / `bamboo-core`).
/// When preceding characters (e.g. Vim normal-mode commands `i`, `a`, `cw`, `ciw`, `jk`, or
/// an adjacent word prefix) are attached to the buffer, this returns the index where the
/// trailing Vietnamese syllable begins.
pub fn extract_last_syllable_index(word: &str) -> usize {
    let chars: Vec<char> = word.chars().collect();
    let mut from_idx = 0;

    for i in 0..chars.len() {
        if !is_valid_in_progress_syllable(&chars[from_idx..=i]) {
            let prev_slice = &chars[from_idx..i];
            let min_start = if let Some(last_v_rel) =
                prev_slice.iter().rposition(|&c| is_vietnamese_vowel(c))
            {
                let last_v = from_idx + last_v_rel;
                if !is_vietnamese_vowel(chars[i]) && last_v + 1 < i {
                    // Consonant typed after an existing consonant coda (e.g. "ath" -> "th", "bietk" -> "k")
                    last_v + 1
                } else {
                    from_idx + 1
                }
            } else {
                from_idx + 1
            };

            let mut next_from = i + 1;
            for k in min_start..=i {
                if is_valid_in_progress_syllable(&chars[k..=i]) {
                    next_from = k;
                    break;
                }
            }
            from_idx = next_from;
        }
    }

    from_idx
}

/// Checks if `s` contains any transformed Vietnamese character (accented vowel or `đ`/`Đ`).
pub fn has_vietnamese_diacritic(s: &str) -> bool {
    s.chars().any(|c| {
        if c == 'đ' || c == 'Đ' {
            return true;
        }
        if let Some((base, tone, _)) = decompose_vowel(c) {
            tone > 0 || matches!(base, 'ă' | 'â' | 'ê' | 'ô' | 'ơ' | 'ư')
        } else {
            false
        }
    })
}

/// Returns the vowel family (`'a'`, `'e'`, `'i'`, `'o'`, `'u'`, `'y'`) of a lowercase base vowel.
pub fn vowel_family(base: char) -> char {
    match base {
        'a' | 'ă' | 'â' => 'a',
        'e' | 'ê' => 'e',
        'i' => 'i',
        'o' | 'ô' | 'ơ' => 'o',
        'u' | 'ư' => 'u',
        'y' => 'y',
        _ => base,
    }
}

/// Decomposes a Vietnamese vowel into `(lowercase_base, tone, is_uppercase)`.
/// `lowercase_base` is one of `'a'`, `'ă'`, `'â'`, `'e'`, `'ê'`, `'i'`, `'o'`, `'ô'`, `'ơ'`, `'u'`, `'ư'`, `'y'`.
/// `tone` is `0` (ngang), `1` (sắc), `2` (huyền), `3` (hỏi), `4` (ngã), `5` (nặng).
pub fn decompose_vowel(c: char) -> Option<(char, usize, bool)> {
    let is_up = c.is_uppercase();
    let lower = c.to_lowercase().next()?;
    let (base, tone) = match lower {
        'a' => ('a', 0),
        'á' => ('a', 1),
        'à' => ('a', 2),
        'ả' => ('a', 3),
        'ã' => ('a', 4),
        'ạ' => ('a', 5),

        'ă' => ('ă', 0),
        'ắ' => ('ă', 1),
        'ằ' => ('ă', 2),
        'ẳ' => ('ă', 3),
        'ẵ' => ('ă', 4),
        'ặ' => ('ă', 5),

        'â' => ('â', 0),
        'ấ' => ('â', 1),
        'ầ' => ('â', 2),
        'ẩ' => ('â', 3),
        'ẫ' => ('â', 4),
        'ậ' => ('â', 5),

        'e' => ('e', 0),
        'é' => ('e', 1),
        'è' => ('e', 2),
        'ẻ' => ('e', 3),
        'ẽ' => ('e', 4),
        'ẹ' => ('e', 5),

        'ê' => ('ê', 0),
        'ế' => ('ê', 1),
        'ề' => ('ê', 2),
        'ể' => ('ê', 3),
        'ễ' => ('ê', 4),
        'ệ' => ('ê', 5),

        'i' => ('i', 0),
        'í' => ('i', 1),
        'ì' => ('i', 2),
        'ỉ' => ('i', 3),
        'ĩ' => ('i', 4),
        'ị' => ('i', 5),

        'o' => ('o', 0),
        'ó' => ('o', 1),
        'ò' => ('o', 2),
        'ỏ' => ('o', 3),
        'õ' => ('o', 4),
        'ọ' => ('o', 5),

        'ô' => ('ô', 0),
        'ố' => ('ô', 1),
        'ồ' => ('ô', 2),
        'ổ' => ('ô', 3),
        'ỗ' => ('ô', 4),
        'ộ' => ('ô', 5),

        'ơ' => ('ơ', 0),
        'ớ' => ('ơ', 1),
        'ờ' => ('ơ', 2),
        'ở' => ('ơ', 3),
        'ỡ' => ('ơ', 4),
        'ợ' => ('ơ', 5),

        'u' => ('u', 0),
        'ú' => ('u', 1),
        'ù' => ('u', 2),
        'ủ' => ('u', 3),
        'ũ' => ('u', 4),
        'ụ' => ('u', 5),

        'ư' => ('ư', 0),
        'ứ' => ('ư', 1),
        'ừ' => ('ư', 2),
        'ử' => ('ư', 3),
        'ữ' => ('ư', 4),
        'ự' => ('ư', 5),

        'y' => ('y', 0),
        'ý' => ('y', 1),
        'ỳ' => ('y', 2),
        'ỷ' => ('y', 3),
        'ỹ' => ('y', 4),
        'ỵ' => ('y', 5),

        _ => return None,
    };
    Some((base, tone, is_up))
}

/// Composes a Vietnamese vowel from `lowercase_base`, `tone` (`0..=5`), and `is_upper`.
pub fn compose_vowel(lowercase_base: char, tone: usize, is_upper: bool) -> char {
    let lower_res = match (lowercase_base, tone) {
        ('a', 1) => 'á',
        ('a', 2) => 'à',
        ('a', 3) => 'ả',
        ('a', 4) => 'ã',
        ('a', 5) => 'ạ',
        ('a', _) => 'a',

        ('ă', 1) => 'ắ',
        ('ă', 2) => 'ằ',
        ('ă', 3) => 'ẳ',
        ('ă', 4) => 'ẵ',
        ('ă', 5) => 'ặ',
        ('ă', _) => 'ă',

        ('â', 1) => 'ấ',
        ('â', 2) => 'ầ',
        ('â', 3) => 'ẩ',
        ('â', 4) => 'ẫ',
        ('â', 5) => 'ậ',
        ('â', _) => 'â',

        ('e', 1) => 'é',
        ('e', 2) => 'è',
        ('e', 3) => 'ẻ',
        ('e', 4) => 'ẽ',
        ('e', 5) => 'ẹ',
        ('e', _) => 'e',

        ('ê', 1) => 'ế',
        ('ê', 2) => 'ề',
        ('ê', 3) => 'ể',
        ('ê', 4) => 'ễ',
        ('ê', 5) => 'ệ',
        ('ê', _) => 'ê',

        ('i', 1) => 'í',
        ('i', 2) => 'ì',
        ('i', 3) => 'ỉ',
        ('i', 4) => 'ĩ',
        ('i', 5) => 'ị',
        ('i', _) => 'i',

        ('o', 1) => 'ó',
        ('o', 2) => 'ò',
        ('o', 3) => 'ỏ',
        ('o', 4) => 'õ',
        ('o', 5) => 'ọ',
        ('o', _) => 'o',

        ('ô', 1) => 'ố',
        ('ô', 2) => 'ồ',
        ('ô', 3) => 'ổ',
        ('ô', 4) => 'ỗ',
        ('ô', 5) => 'ộ',
        ('ô', _) => 'ô',

        ('ơ', 1) => 'ớ',
        ('ơ', 2) => 'ờ',
        ('ơ', 3) => 'ở',
        ('ơ', 4) => 'ỡ',
        ('ơ', 5) => 'ợ',
        ('ơ', _) => 'ơ',

        ('u', 1) => 'ú',
        ('u', 2) => 'ù',
        ('u', 3) => 'ủ',
        ('u', 4) => 'ũ',
        ('u', 5) => 'ụ',
        ('u', _) => 'u',

        ('ư', 1) => 'ứ',
        ('ư', 2) => 'ừ',
        ('ư', 3) => 'ử',
        ('ư', 4) => 'ữ',
        ('ư', 5) => 'ự',
        ('ư', _) => 'ư',

        ('y', 1) => 'ý',
        ('y', 2) => 'ỳ',
        ('y', 3) => 'ỷ',
        ('y', 4) => 'ỹ',
        ('y', 5) => 'ỵ',
        ('y', _) => 'y',

        _ => lowercase_base,
    };

    if is_upper {
        lower_res.to_uppercase().next().unwrap_or(lower_res)
    } else {
        lower_res
    }
}

/// Checks whether a character is any Vietnamese vowel (plain or accented).
pub fn is_vietnamese_vowel(c: char) -> bool {
    decompose_vowel(c).is_some()
}

/// Strips any diacritical tone mark from a vowel.
pub fn strip_tone(c: char) -> char {
    if let Some((base, _, is_up)) = decompose_vowel(c) {
        compose_vowel(base, 0, is_up)
    } else {
        c
    }
}

/// Applies tone mark (1=Sắc, 2=Huyền, 3=Hỏi, 4=Ngã, 5=Nặng) to a vowel.
pub fn add_tone_to_vowel(v: char, tone: usize) -> char {
    if let Some((base, _, is_up)) = decompose_vowel(v) {
        compose_vowel(base, tone, is_up)
    } else {
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telex_duoc() {
        let telex = TelexEngine::new(ToneStyle::Modern);
        assert_eq!(telex.feed("dduocwj").0, "được");
        assert_eq!(telex.feed("dduowcj").0, "được");
        assert_eq!(telex.feed("duocw").0, "dươc");
        assert_eq!(telex.feed("duocwd").0, "đươc");
        assert_eq!(telex.feed("duocwdj").0, "được");
        assert_eq!(telex.feed("duocwjd").0, "được");
        assert_eq!(telex.feed("duocd").0, "đuoc");
        assert_eq!(telex.feed("duocdj").0, "được");
        assert_eq!(telex.feed("dad").0, "đa");
        assert_eq!(telex.feed("dend").0, "đen");

        // Natural Vietnamese Phonology: Plain 'uo' and 'ie' without diacritics before coda cannot take tone
        assert_eq!(telex.feed("duongf").0, "duongf");
        assert_eq!(telex.feed("tuongf").0, "tuongf");
        assert_eq!(telex.feed("buongf").0, "buongf");
        assert_eq!(telex.feed("nuocs").0, "nuocs");
        assert_eq!(telex.feed("cuocs").0, "cuocs");
        assert_eq!(telex.feed("muons").0, "muons");
        assert_eq!(telex.feed("luots").0, "luots");
        assert_eq!(telex.feed("tiens").0, "tiens");
        assert_eq!(telex.feed("tiengs").0, "tiengs");
        assert_eq!(telex.feed("biets").0, "biets");

        // Valid horned/circumflexed inputs
        assert_eq!(telex.feed("duowngf").0, "dường");
        assert_eq!(telex.feed("dduowngf").0, "đường");
        assert_eq!(telex.feed("tuowngf").0, "tường");
        assert_eq!(telex.feed("nuowcs").0, "nước");
        assert_eq!(telex.feed("buoongf").0, "buồng");
    }

    #[test]
    fn test_telex_free_marking_and_undo_at_end_of_word() {
        let telex = TelexEngine::new(ToneStyle::Modern);
        // Late circumflex at end of word (like OpenViet IME)
        assert_eq!(telex.feed("khongo").0, "không");
        assert_eq!(telex.feed("khongos").0, "khống");
        assert_eq!(telex.feed("toana").0, "toân");
        assert_eq!(telex.feed("tienge").0, "tiêng");
        assert_eq!(telex.feed("tienges").0, "tiếng");

        // Undo circumflex at end of word by pressing key again
        assert_eq!(telex.feed("khongoo").0, "khongo");
        assert_eq!(telex.feed("toanaa").0, "toana");
        assert_eq!(telex.feed("tiengee").0, "tienge");

        // Undo tone at end of word by pressing same tone key or 'z'
        assert_eq!(telex.feed("khongosz").0, "không");
        assert_eq!(telex.feed("khongoss").0, "khôngs");
        assert_eq!(telex.feed("toans").0, "toán");
        assert_eq!(telex.feed("toanss").0, "toans");
        assert_eq!(telex.feed("toansz").0, "toan");
        // Modify tone at end of word
        assert_eq!(telex.feed("toansf").0, "toàn");
        assert_eq!(telex.feed("gowf").0, "gờ");
        assert_eq!(telex.feed("giowf").0, "giờ");
        assert_eq!(telex.feed("giofw").0, "giờ");
        assert_eq!(telex.feed("dziaf").0, "dzìa");
        assert_eq!(telex.feed("dzis").0, "dzí");
        assert_eq!(telex.feed("dzoo").0, "dzô");
        assert_eq!(telex.feed("dzij").0, "dzị");
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
        assert_eq!(telex.feed("muaw").0, "mưa");
        assert_eq!(telex.feed("cuuws").0, "cứu");
        assert_eq!(telex.feed("thuowr").0, "thuở");
        assert_eq!(telex.feed("huow").0, "huơ");
        assert_eq!(telex.feed("quawngj").0, "quặng");
        // Southern dialect 'quởn' / 'quỡn' (horn only on 'o', 'u' after 'q' is never 'ư')
        assert_eq!(telex.feed("quonwr").0, "quởn");
        assert_eq!(telex.feed("quowrn").0, "quởn");
        assert_eq!(telex.feed("quonwx").0, "quỡn");
    }

    #[test]
    fn test_telex_bracket_transform() {
        let telex_bracket = TelexEngine::with_options(ToneStyle::Modern, W2UType::NonStart, true);
        assert_eq!(telex_bracket.feed("[").0, "ơ");
        assert_eq!(telex_bracket.feed("]").0, "ư");
        assert_eq!(telex_bracket.feed("{").0, "Ơ");
        assert_eq!(telex_bracket.feed("}").0, "Ư");
        assert_eq!(telex_bracket.feed("[[").0, "[");
        assert_eq!(telex_bracket.feed("]]").0, "]");
    }

    #[test]
    fn test_telex_standalone_w_and_initial_consonants() {
        let telex_non_start =
            TelexEngine::with_options(ToneStyle::Modern, W2UType::NonStart, false);
        assert_eq!(telex_non_start.feed("w").0, "w");
        assert_eq!(telex_non_start.feed("tw").0, "tư");
        assert_eq!(telex_non_start.feed("tww").0, "tw");
        assert_eq!(telex_non_start.feed("sa").0, "sa");
        assert_eq!(telex_non_start.feed("sang").0, "sang");
        assert_eq!(telex_non_start.feed("trang").0, "trang");
        assert_eq!(telex_non_start.feed("xa").0, "xa");

        let telex_always = TelexEngine::with_options(ToneStyle::Modern, W2UType::Always, false);
        assert_eq!(telex_always.feed("w").0, "ư");
        assert_eq!(telex_always.feed("ww").0, "w");
    }

    #[test]
    fn test_telex_tay_nguyen_and_rare_words() {
        let telex = TelexEngine::new(ToneStyle::Modern);
        // Tay Nguyen loan onsets & codas
        assert_eq!(telex.feed("kroong").0, "krông");
        assert_eq!(telex.feed("kroongf").0, "krồng");
        assert_eq!(telex.feed("ddawsk").0, "đắk");
        assert_eq!(telex.feed("ddawk").0, "đăk");
        assert_eq!(telex.feed("lawsk").0, "lắk");
        assert_eq!(telex.feed("pawsk").0, "pắk");
        assert_eq!(telex.feed("busk").0, "búk");
        assert_eq!(telex.feed("proong").0, "prông");
        assert_eq!(telex.feed("pawh").0, "păh");
        assert_eq!(telex.feed("ddrawsk").0, "đrắk");

        // Rare words & dialect
        assert_eq!(telex.feed("khueechs").0, "khuếch");
        assert_eq!(telex.feed("khuynh").0, "khuynh");
        assert_eq!(telex.feed("nguaayf").0, "nguầy");
        assert_eq!(telex.feed("nguaayj").0, "nguậy");
        assert_eq!(telex.feed("maayf").0, "mầy");
        assert_eq!(telex.feed("quawst").0, "quắt");
        assert_eq!(telex.feed("quawjp").0, "quặp");
        assert_eq!(telex.feed("quasnh").0, "quánh");
        assert_eq!(telex.feed("zij").0, "zị");
    }

    #[test]
    fn test_dynamic_tone_shifting() {
        let telex_modern = TelexEngine::new(ToneStyle::Modern);
        // 'hosa' -> 'ho' + 's' -> 'hó', type 'a' -> dynamically shifts to 'hoá'
        assert_eq!(telex_modern.feed("hosa").0, "hoá");
        assert_eq!(telex_modern.feed("hosan").0, "hoán");
        assert_eq!(telex_modern.feed("tosa").0, "toá");
        assert_eq!(telex_modern.feed("tosan").0, "toán");

        let telex_classic = TelexEngine::new(ToneStyle::Classic);
        // 'hosa' -> 'hóa', type 'n' -> 'hoán' (tone shifts to 'a' because of coda 'n')
        assert_eq!(telex_classic.feed("hosa").0, "hóa");
        assert_eq!(telex_classic.feed("hosan").0, "hoán");
        assert_eq!(telex_classic.feed("tosa").0, "tóa");
        assert_eq!(telex_classic.feed("tosan").0, "toán");

        // Classic style must correctly place tone on 'ê' (thuế, huế) never on 'u' (thúê, húê)
        assert_eq!(telex_classic.feed("thuees").0, "thuế");
        assert_eq!(telex_classic.feed("huees").0, "huế");
        assert_eq!(telex_classic.feed("thusee").0, "thuế");
        assert_eq!(telex_classic.feed("husee").0, "huế");
        assert_eq!(telex_classic.feed("thuese").0, "thuế");
        assert_eq!(telex_classic.feed("huese").0, "huế");
    }

    #[test]
    fn test_shape_modifier_swapping_and_tone_preservation() {
        let telex = TelexEngine::new(ToneStyle::Modern);
        // aaw -> ă, oow -> ơ
        assert_eq!(telex.feed("aaw").0, "ă");
        assert_eq!(telex.feed("oow").0, "ơ");
        assert_eq!(telex.feed("aa").0, "â");
        assert_eq!(telex.feed("oo").0, "ô");

        // Swap â <-> ă preserving tone
        assert_eq!(telex.feed("amsw").0, "ắm");
        assert_eq!(telex.feed("amswa").0, "ấm");
        assert_eq!(telex.feed("canw").0, "căn");
        assert_eq!(telex.feed("canwa").0, "cân");
        assert_eq!(telex.feed("tapjw").0, "tặp");
        assert_eq!(telex.feed("tapjwa").0, "tập");

        // Swap ô <-> ơ preserving tone
        assert_eq!(telex.feed("bonsw").0, "bớn");
        assert_eq!(telex.feed("bonswo").0, "bốn");
        assert_eq!(telex.feed("hopjw").0, "hợp");
        assert_eq!(telex.feed("hopjwo").0, "hộp");
        assert_eq!(telex.feed("cotjw").0, "cợt");
        assert_eq!(telex.feed("cotjwo").0, "cột");
    }

    #[test]
    fn test_phonetic_tone_protection() {
        let telex = TelexEngine::new(ToneStyle::Modern);
        // 'được' has stop coda 'c', which cannot take huyền (f), hỏi (r), ngã (x)
        assert_eq!(telex.feed("dduowcjf").0, "đượcf");
        assert_eq!(telex.feed("dduowcjr").0, "đượcr");
        assert_eq!(telex.feed("dduowcjx").0, "đượcx");
        // But can change to sắc (s) -> 'đước'
        assert_eq!(telex.feed("dduowcjs").0, "đước");
        // And pressing 'j' again toggles tone off -> 'đươcj'
        assert_eq!(telex.feed("dduowcjj").0, "đươcj");
    }
}
