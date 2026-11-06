//! Vietnamese Syllable Parser & Phonological Decomposition Engine.
//!
//! Provides a linguistically accurate Vietnamese syllable decomposition and reconstruction engine,
//! inspired by Bamboo-Core / fcitx5-bamboo phonological models.
//!
//! # Linguistic Model
//! In Vietnamese phonology, every complete syllable is structured as:
//!
//! ```text
//! Syllable = Onset (Âm đầu) + Glide (Âm đệm) + Nucleus (Âm chính) + Coda (Âm cuối) + Tone (Dấu thanh)
//! ```
//!
//! - **Onset (Âm đầu)**: Initial consonant (`b`, `c`, `ch`, `d`, `đ`, `g`, `gh`, `gi`, `h`, `k`, `kh`,
//!   `l`, `m`, `n`, `ng`, `ngh`, `nh`, `p`, `ph`, `q`, `r`, `s`, `t`, `th`, `tr`, `v`, `x`, or empty).
//! - **Glide (Âm đệm)**: Pre-vocalic semi-vowel (`u` or `o`), or `None`. In `qu-`, `q` is the onset and
//!   `u` functions as the glide.
//! - **Nucleus (Âm chính)**: Monophthong (`a`, `ă`, `â`, `e`, `ê`, `i`, `o`, `ô`, `ơ`, `u`, `ư`, `y`)
//!   or diphthong (`ia`/`iê`, `ya`/`yê`, `ua`/`uô`, `ưa`/`ươ`).
//! - **Coda (Âm cuối)**: Final consonant (`c`, `ch`, `m`, `n`, `ng`, `nh`, `p`, `t`),
//!   off-glide semi-vowel (`i`, `y`, `o`, `u`), or empty.
//! - **Tone (Dấu thanh)**: One of 6 tones (Ngang, Sắc, Huyền, Hỏi, Ngã, Nặng).

use crate::engine::ToneStyle;
use crate::telex::{compose_vowel, decompose_vowel, is_vietnamese_vowel};

/// Tone mark representation in Vietnamese orthography.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ToneMark {
    /// Thanh ngang / không dấu (No tone)
    #[default]
    None = 0,
    /// Dấu sắc (Acute / Rising)
    Acute = 1,
    /// Dấu huyền (Grave / Falling)
    Grave = 2,
    /// Dấu hỏi (Hook above / Asking)
    HookAbove = 3,
    /// Dấu ngã (Tilde / Tumbling)
    Tilde = 4,
    /// Dấu nặng (Dot below / Heavy)
    DotBelow = 5,
}

impl ToneMark {
    /// Returns the tone mark name in Vietnamese.
    pub fn name_vi(&self) -> &'static str {
        match self {
            Self::None => "Ngang",
            Self::Acute => "Sắc",
            Self::Grave => "Huyền",
            Self::HookAbove => "Hỏi",
            Self::Tilde => "Ngã",
            Self::DotBelow => "Nặng",
        }
    }

    /// Returns the numeric tone index (`0` = Ngang, `1` = Sắc, `2` = Huyền, `3` = Hỏi, `4` = Ngã, `5` = Nặng).
    pub fn to_index(&self) -> usize {
        *self as usize
    }

    /// Converts from numeric tone index (`0..=5`).
    pub fn from_index(idx: usize) -> Self {
        match idx {
            1 => Self::Acute,
            2 => Self::Grave,
            3 => Self::HookAbove,
            4 => Self::Tilde,
            5 => Self::DotBelow,
            _ => Self::None,
        }
    }
}

impl From<usize> for ToneMark {
    fn from(idx: usize) -> Self {
        Self::from_index(idx)
    }
}

impl From<ToneMark> for usize {
    fn from(t: ToneMark) -> Self {
        t.to_index()
    }
}

/// Represents a decomposed Vietnamese syllable in standard phonological components.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParsedSyllable {
    /// Âm đầu (Onset consonant, e.g. "ngh", "ch", "q", "đ", or "" for zero-onset).
    pub onset: String,
    /// Âm đệm (Glide semi-vowel: 'u', 'o', or None).
    pub glide: Option<char>,
    /// Âm chính (Vowel nucleus, e.g. "a", "ă", "â", "iê", "ươ", "uô", "yê", "i", "ư"...).
    pub nucleus: String,
    /// Âm cuối (Coda consonant or semi-vowel, e.g. "ng", "c", "i", "y", "u", "o", or "").
    pub coda: String,
    /// Dấu thanh (Tone mark).
    pub tone: ToneMark,
}

impl ParsedSyllable {
    /// Creates a new `ParsedSyllable` with given components.
    pub fn new(
        onset: impl Into<String>,
        glide: Option<char>,
        nucleus: impl Into<String>,
        coda: impl Into<String>,
        tone: ToneMark,
    ) -> Self {
        Self {
            onset: onset.into(),
            glide,
            nucleus: nucleus.into(),
            coda: coda.into(),
            tone,
        }
    }

    /// Returns `true` if the syllable has no coda (open syllable / âm tiết mở).
    pub fn is_open(&self) -> bool {
        self.coda.is_empty()
    }

    /// Returns `true` if the coda ends with a stop consonant (`c`, `ch`, `p`, `t`).
    pub fn has_stop_coda(&self) -> bool {
        matches!(self.coda.to_ascii_lowercase().as_str(), "c" | "ch" | "p" | "t")
    }

    /// Returns a new `ParsedSyllable` with the tone replaced.
    pub fn with_tone(&self, tone: ToneMark) -> Self {
        let mut clone = self.clone();
        clone.tone = tone;
        clone
    }

    /// Validates whether the syllable conforms to Vietnamese orthography and phonotactics.
    pub fn is_valid(&self) -> bool {
        let lower_nucleus = self.nucleus.to_lowercase();
        let lower_coda = self.coda.to_lowercase();
        let lower_onset = self.onset.to_lowercase();

        // 1. Must contain at least one vowel in nucleus
        if lower_nucleus.is_empty() {
            return false;
        }

        // 2. Stop codas (c, ch, p, t, k) can ONLY take Ngang (0 / untoned), Sắc (1), or Nặng (5)
        if matches!(lower_coda.as_str(), "c" | "ch" | "p" | "t" | "k")
            && matches!(self.tone, ToneMark::Grave | ToneMark::HookAbove | ToneMark::Tilde)
        {
            return false;
        }

        // 3. Open diphthongs (ia, ua, ưa) cannot take a consonant coda
        if matches!(lower_nucleus.as_str(), "ia" | "ua" | "ưa") && !lower_coda.is_empty() {
            return false;
        }

        // 4. Spelling rules: 'gh', 'ngh' must precede 'i', 'e', 'ê' (or glide 'u' + 'y'/'ê')
        if matches!(lower_onset.as_str(), "gh" | "ngh") {
            if let Some(first_v) = lower_nucleus.chars().next() {
                if !matches!(first_v, 'i' | 'e' | 'ê') {
                    return false;
                }
            }
        }

        true
    }

    /// Reconstructs the syllable into a standard Vietnamese UTF-8 string according
    /// to the requested tone style (Modern vs Classic).
    pub fn reconstruct(&self, tone_style: ToneStyle) -> String {
        let mut res = String::with_capacity(
            self.onset.len() + 2 + self.nucleus.len() + self.coda.len(),
        );

        let is_gi_alone = self.onset.eq_ignore_ascii_case("gi")
            && self.nucleus.eq_ignore_ascii_case("i");

        if is_gi_alone {
            if let Some(first_g) = self.onset.chars().next() {
                res.push(first_g);
            }
        } else {
            res.push_str(&self.onset);
        }

        if self.tone == ToneMark::None {
            if let Some(g) = self.glide {
                res.push(g);
            }
            res.push_str(&self.nucleus);
            res.push_str(&self.coda);
            return res;
        }

        let tone_idx = self.tone.to_index();
        let has_coda = !self.coda.is_empty();
        let mut nuc_chars: Vec<char> = self.nucleus.chars().collect();
        let mut glide_char = self.glide;

        if let Some(g) = self.glide {
            let is_onset_q = self.onset.eq_ignore_ascii_case("q");

            match tone_style {
                ToneStyle::Modern => {
                    // Modern style: tone goes on the nucleus vowel (hoà, hoá, thuỷ, khoè)
                    if nuc_chars.len() == 1 {
                        nuc_chars[0] = apply_tone(nuc_chars[0], tone_idx);
                    } else if nuc_chars.len() >= 2 {
                        if has_coda {
                            nuc_chars[1] = apply_tone(nuc_chars[1], tone_idx);
                        } else {
                            nuc_chars[0] = apply_tone(nuc_chars[0], tone_idx);
                        }
                    }
                }
                ToneStyle::Classic => {
                    if is_onset_q {
                        // After 'q', tone is always placed on the nucleus vowel
                        if nuc_chars.len() == 1 {
                            nuc_chars[0] = apply_tone(nuc_chars[0], tone_idx);
                        } else if nuc_chars.len() >= 2 {
                            if has_coda {
                                nuc_chars[1] = apply_tone(nuc_chars[1], tone_idx);
                            } else {
                                nuc_chars[0] = apply_tone(nuc_chars[0], tone_idx);
                            }
                        }
                    } else if !has_coda {
                        // Open syllable with glide (hòa, hóa, thủy, khỏe): tone on glide
                        glide_char = Some(apply_tone(g, tone_idx));
                    } else {
                        // Closed syllable with glide (toán, hoàn, luận, khuấy): tone on nucleus
                        nuc_chars[0] = apply_tone(nuc_chars[0], tone_idx);
                    }
                }
            }
        } else {
            // Syllable without glide
            if nuc_chars.len() == 1 {
                nuc_chars[0] = apply_tone(nuc_chars[0], tone_idx);
            } else if nuc_chars.len() >= 2 {
                if has_coda {
                    // Closed diphthong: tone on second vowel (tiến, muốn, mượn, được, buồn, rượu, suối)
                    nuc_chars[1] = apply_tone(nuc_chars[1], tone_idx);
                } else {
                    // Open diphthong: tone on first vowel (mía, múa, mứa, cửa, chĩa)
                    nuc_chars[0] = apply_tone(nuc_chars[0], tone_idx);
                }
            }
        }

        if let Some(g) = glide_char {
            res.push(g);
        }
        for c in nuc_chars {
            res.push(c);
        }
        res.push_str(&self.coda);

        res
    }
}

impl std::fmt::Display for ParsedSyllable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.reconstruct(ToneStyle::Modern))
    }
}

/// Applies a diacritical tone mark (`0..=5`) to a vowel character while preserving case.
fn apply_tone(c: char, tone: usize) -> char {
    if let Some((base, _, is_upper)) = decompose_vowel(c) {
        compose_vowel(base, tone, is_upper)
    } else {
        c
    }
}

/// Parses an arbitrary Vietnamese word or in-progress syllable into a `ParsedSyllable`.
///
/// Returns `None` if `word` is empty.
pub fn parse_syllable(word: &str) -> Option<ParsedSyllable> {
    if word.is_empty() {
        return None;
    }

    // Step 1: Extract diacritical tone mark and strip tones from characters
    let mut raw_chars: Vec<char> = Vec::with_capacity(word.len());
    let mut detected_tone = ToneMark::None;

    for c in word.chars() {
        if let Some((base, t, is_upper)) = decompose_vowel(c) {
            if t > 0 {
                detected_tone = ToneMark::from_index(t);
            }
            raw_chars.push(compose_vowel(base, 0, is_upper));
        } else {
            raw_chars.push(c);
        }
    }

    let lower_chars: Vec<char> = raw_chars
        .iter()
        .flat_map(|c| c.to_lowercase())
        .collect();

    let n = raw_chars.len();
    let mut onset_len = 0;
    let onset_glide: Option<char> = None;

    // Step 2: Detect Onset (Phụ âm đầu)
    if n >= 3 && lower_chars[0] == 'n' && lower_chars[1] == 'g' && lower_chars[2] == 'h' {
        onset_len = 3;
    } else if n >= 2 && lower_chars[0] == 'q' && lower_chars[1] == 'u' {
        // Special case: 'qu' -> onset 'q', glide 'u'
        let onset_str: String = raw_chars[0..1].iter().collect();
        let glide_ch = raw_chars[1];
        let rem_raw = &raw_chars[2..];
        let rem_lower = &lower_chars[2..];

        let (nucleus, coda) = parse_vowels_and_coda(rem_raw, rem_lower, true);

        return Some(ParsedSyllable {
            onset: onset_str,
            glide: Some(glide_ch),
            nucleus,
            coda,
            tone: detected_tone,
        });
    } else if n >= 2 && lower_chars[0] == 'g' && lower_chars[1] == 'i' {
        // Special case: 'gi'
        if n == 2 {
            // E.g. "gi", "gì", "gí"
            let onset_str: String = raw_chars[0..2].iter().collect();
            return Some(ParsedSyllable {
                onset: onset_str,
                glide: None,
                nucleus: raw_chars[1].to_string(), // 'i'
                coda: String::new(),
                tone: detected_tone,
            });
        }
        let next_ch = lower_chars[2];
        if is_vietnamese_vowel(next_ch) {
            // E.g. "gió", "giúp", "giữa", "gieo", "giết"
            onset_len = 2;
        } else {
            // E.g. "gìn", "gịt", "gíp" -> 'i' is the vowel nucleus, following consonant is coda
            let onset_str: String = raw_chars[0..2].iter().collect();
            let coda_str: String = raw_chars[2..].iter().collect();
            return Some(ParsedSyllable {
                onset: onset_str,
                glide: None,
                nucleus: raw_chars[1].to_string(),
                coda: coda_str,
                tone: detected_tone,
            });
        }
    } else if n >= 2 {
        let pair = (lower_chars[0], lower_chars[1]);
        if matches!(
            pair,
            ('c', 'h')
                | ('g', 'h')
                | ('k', 'h')
                | ('n', 'h')
                | ('n', 'g')
                | ('p', 'h')
                | ('t', 'h')
                | ('t', 'r')
                | ('d', 'z')
                | ('k', 'r')
                | ('p', 'r')
                | ('d', 'r')
                | ('đ', 'r')
                | ('b', 'r')
                | ('k', 'l')
                | ('p', 'l')
                | ('b', 'l')
        ) {
            onset_len = 2;
        }
    }

    if onset_len == 0 && n >= 1 {
        let first = lower_chars[0];
        if matches!(
            first,
            'b' | 'c'
                | 'd'
                | 'đ'
                | 'g'
                | 'h'
                | 'k'
                | 'l'
                | 'm'
                | 'n'
                | 'p'
                | 'q'
                | 'r'
                | 's'
                | 't'
                | 'v'
                | 'x'
                | 'f'
                | 'j'
                | 'w'
                | 'z'
        ) {
            onset_len = 1;
        }
    }

    let onset_str: String = raw_chars[0..onset_len].iter().collect();
    let rem_raw = &raw_chars[onset_len..];
    let rem_lower = &lower_chars[onset_len..];

    if rem_raw.is_empty() {
        return Some(ParsedSyllable {
            onset: onset_str,
            glide: onset_glide,
            nucleus: String::new(),
            coda: String::new(),
            tone: detected_tone,
        });
    }

    // Step 3: Parse Glide (Âm đệm), Nucleus (Âm chính), and Coda (Âm cuối)
    let mut glide: Option<char> = None;
    let mut rem_start = 0;

    if rem_lower.len() >= 2 {
        let first = rem_lower[0];
        let second = rem_lower[1];

        if first == 'o' && matches!(second, 'a' | 'ă' | 'e') {
            // E.g. oa, oă, oe (hoa, toán, hoặc, khoe)
            glide = Some(rem_raw[0]);
            rem_start = 1;
        } else if first == 'u' {
            if matches!(second, 'â' | 'ê' | 'ơ') {
                // E.g. uâ, uê, uơ (xuân, thuê, thuở)
                glide = Some(rem_raw[0]);
                rem_start = 1;
            } else if second == 'y' {
                // E.g. uy, uya, uyê (thủy, khuya, chuyên)
                glide = Some(rem_raw[0]);
                rem_start = 1;
            } else if rem_lower.len() >= 3 && second == 'a' && rem_lower[2] == 'y' {
                // E.g. uay (quay)
                glide = Some(rem_raw[0]);
                rem_start = 1;
            } else if second == 'e' {
                // E.g. loanword / dialect (que, quẹt)
                glide = Some(rem_raw[0]);
                rem_start = 1;
            }
        }
    }

    let (nucleus, coda) = parse_vowels_and_coda(&rem_raw[rem_start..], &rem_lower[rem_start..], false);

    Some(ParsedSyllable {
        onset: onset_str,
        glide,
        nucleus,
        coda,
        tone: detected_tone,
    })
}

/// Helper function to parse vowel nucleus and coda from a remaining character slice.
fn parse_vowels_and_coda(
    rem_raw: &[char],
    rem_lower: &[char],
    after_qu: bool,
) -> (String, String) {
    if rem_raw.is_empty() {
        return (String::new(), String::new());
    }

    let mut nuc_len = 0;

    // Check for 2-letter diphthong nucleus
    if rem_lower.len() >= 2 {
        let pair = (rem_lower[0], rem_lower[1]);
        let is_diphthong = match pair {
            ('i', 'a') | ('i', 'ê') | ('u', 'a') | ('u', 'ô') | ('ư', 'a') | ('ư', 'ơ') | ('y', 'a') | ('y', 'ê') | ('o', 'o') => {
                // 'ua' after 'q' is not a diphthong: in 'quang', 'u' is glide, 'a' is nucleus
                if after_qu && pair == ('u', 'a') {
                    false
                } else {
                    true
                }
            }
            _ => false,
        };

        if is_diphthong {
            nuc_len = 2;
        }
    }

    // Single vowel nucleus fallback
    if nuc_len == 0 && !rem_lower.is_empty() && is_vietnamese_vowel(rem_lower[0]) {
        nuc_len = 1;
    }

    let nucleus: String = rem_raw[0..nuc_len].iter().collect();
    let coda: String = rem_raw[nuc_len..].iter().collect();

    (nucleus, coda)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_monophthongs() {
        let s = parse_syllable("cá").unwrap();
        assert_eq!(s.onset, "c");
        assert_eq!(s.glide, None);
        assert_eq!(s.nucleus, "a");
        assert_eq!(s.coda, "");
        assert_eq!(s.tone, ToneMark::Acute);

        let s2 = parse_syllable("nhà").unwrap();
        assert_eq!(s2.onset, "nh");
        assert_eq!(s2.nucleus, "a");
        assert_eq!(s2.coda, "");
        assert_eq!(s2.tone, ToneMark::Grave);

        let s3 = parse_syllable("đỏ").unwrap();
        assert_eq!(s3.onset, "đ");
        assert_eq!(s3.nucleus, "o");
        assert_eq!(s3.tone, ToneMark::HookAbove);

        let s4 = parse_syllable("chư").unwrap();
        assert_eq!(s4.onset, "ch");
        assert_eq!(s4.nucleus, "ư");
        assert_eq!(s4.coda, "");
        assert_eq!(s4.tone, ToneMark::None);
    }

    #[test]
    fn test_parse_diphthongs_and_clusters() {
        let s = parse_syllable("nghiêng").unwrap();
        assert_eq!(s.onset, "ngh");
        assert_eq!(s.glide, None);
        assert_eq!(s.nucleus, "iê");
        assert_eq!(s.coda, "ng");
        assert_eq!(s.tone, ToneMark::None);

        let s2 = parse_syllable("buôn").unwrap();
        assert_eq!(s2.onset, "b");
        assert_eq!(s2.nucleus, "uô");
        assert_eq!(s2.coda, "n");
        assert_eq!(s2.tone, ToneMark::None);

        let s3 = parse_syllable("được").unwrap();
        assert_eq!(s3.onset, "đ");
        assert_eq!(s3.nucleus, "ươ");
        assert_eq!(s3.coda, "c");
        assert_eq!(s3.tone, ToneMark::DotBelow);

        let s4 = parse_syllable("mía").unwrap();
        assert_eq!(s4.onset, "m");
        assert_eq!(s4.nucleus, "ia");
        assert_eq!(s4.coda, "");
        assert_eq!(s4.tone, ToneMark::Acute);
    }

    #[test]
    fn test_parse_glide_and_triphthongs() {
        let s = parse_syllable("hoá").unwrap();
        assert_eq!(s.onset, "h");
        assert_eq!(s.glide, Some('o'));
        assert_eq!(s.nucleus, "a");
        assert_eq!(s.coda, "");
        assert_eq!(s.tone, ToneMark::Acute);

        let s2 = parse_syllable("khuấy").unwrap();
        assert_eq!(s2.onset, "kh");
        assert_eq!(s2.glide, Some('u'));
        assert_eq!(s2.nucleus, "â");
        assert_eq!(s2.coda, "y");
        assert_eq!(s2.tone, ToneMark::Acute);

        let s3 = parse_syllable("khoai").unwrap();
        assert_eq!(s3.onset, "kh");
        assert_eq!(s3.glide, Some('o'));
        assert_eq!(s3.nucleus, "a");
        assert_eq!(s3.coda, "i");
        assert_eq!(s3.tone, ToneMark::None);

        let s4 = parse_syllable("chuyện").unwrap();
        assert_eq!(s4.onset, "ch");
        assert_eq!(s4.glide, Some('u'));
        assert_eq!(s4.nucleus, "yê");
        assert_eq!(s4.coda, "n");
        assert_eq!(s4.tone, ToneMark::DotBelow);

        let s5 = parse_syllable("khuya").unwrap();
        assert_eq!(s5.onset, "kh");
        assert_eq!(s5.glide, Some('u'));
        assert_eq!(s5.nucleus, "ya");
        assert_eq!(s5.coda, "");
        assert_eq!(s5.tone, ToneMark::None);
    }

    #[test]
    fn test_parse_special_qu_and_gi() {
        let s = parse_syllable("quốc").unwrap();
        assert_eq!(s.onset, "q");
        assert_eq!(s.glide, Some('u'));
        assert_eq!(s.nucleus, "ô");
        assert_eq!(s.coda, "c");
        assert_eq!(s.tone, ToneMark::Acute);

        let s2 = parse_syllable("quán").unwrap();
        assert_eq!(s2.onset, "q");
        assert_eq!(s2.glide, Some('u'));
        assert_eq!(s2.nucleus, "a");
        assert_eq!(s2.coda, "n");
        assert_eq!(s2.tone, ToneMark::Acute);

        let s3 = parse_syllable("quý").unwrap();
        assert_eq!(s3.onset, "q");
        assert_eq!(s3.glide, Some('u'));
        assert_eq!(s3.nucleus, "y");
        assert_eq!(s3.coda, "");
        assert_eq!(s3.tone, ToneMark::Acute);

        let s4 = parse_syllable("giặt").unwrap();
        assert_eq!(s4.onset, "gi");
        assert_eq!(s4.glide, None);
        assert_eq!(s4.nucleus, "ă");
        assert_eq!(s4.coda, "t");
        assert_eq!(s4.tone, ToneMark::DotBelow);

        let s5 = parse_syllable("gìn").unwrap();
        assert_eq!(s5.onset, "gi");
        assert_eq!(s5.glide, None);
        assert_eq!(s5.nucleus, "i");
        assert_eq!(s5.coda, "n");
        assert_eq!(s5.tone, ToneMark::Grave);

        let s6 = parse_syllable("gì").unwrap();
        assert_eq!(s6.onset, "gi");
        assert_eq!(s6.nucleus, "i");
        assert_eq!(s6.coda, "");
        assert_eq!(s6.tone, ToneMark::Grave);

        let s7 = parse_syllable("giúp").unwrap();
        assert_eq!(s7.onset, "gi");
        assert_eq!(s7.nucleus, "u");
        assert_eq!(s7.coda, "p");
        assert_eq!(s7.tone, ToneMark::Acute);
    }

    #[test]
    fn test_parse_zero_onset() {
        let s = parse_syllable("yêu").unwrap();
        assert_eq!(s.onset, "");
        assert_eq!(s.nucleus, "yê");
        assert_eq!(s.coda, "u");
        assert_eq!(s.tone, ToneMark::None);

        let s2 = parse_syllable("uống").unwrap();
        assert_eq!(s2.onset, "");
        assert_eq!(s2.nucleus, "uô");
        assert_eq!(s2.coda, "ng");
        assert_eq!(s2.tone, ToneMark::Acute);
    }

    #[test]
    fn test_reconstruct_tone_styles() {
        let hoa = parse_syllable("hoá").unwrap();
        assert_eq!(hoa.reconstruct(ToneStyle::Modern), "hoá");
        assert_eq!(hoa.reconstruct(ToneStyle::Classic), "hóa");

        let thuy = parse_syllable("thủy").unwrap();
        assert_eq!(thuy.reconstruct(ToneStyle::Modern), "thuỷ");
        assert_eq!(thuy.reconstruct(ToneStyle::Classic), "thủy");

        let toan = parse_syllable("toán").unwrap();
        assert_eq!(toan.reconstruct(ToneStyle::Modern), "toán");
        assert_eq!(toan.reconstruct(ToneStyle::Classic), "toán");

        let quan = parse_syllable("quán").unwrap();
        assert_eq!(quan.reconstruct(ToneStyle::Modern), "quán");
        assert_eq!(quan.reconstruct(ToneStyle::Classic), "quán");

        let quy = parse_syllable("quý").unwrap();
        assert_eq!(quy.reconstruct(ToneStyle::Modern), "quý");
        assert_eq!(quy.reconstruct(ToneStyle::Classic), "quý");

        let tien = parse_syllable("tiến").unwrap();
        assert_eq!(tien.reconstruct(ToneStyle::Modern), "tiến");
        assert_eq!(tien.reconstruct(ToneStyle::Classic), "tiến");

        let mia = parse_syllable("mía").unwrap();
        assert_eq!(mia.reconstruct(ToneStyle::Modern), "mía");
        assert_eq!(mia.reconstruct(ToneStyle::Classic), "mía");

        let duoc = parse_syllable("được").unwrap();
        assert_eq!(duoc.reconstruct(ToneStyle::Modern), "được");
        assert_eq!(duoc.reconstruct(ToneStyle::Classic), "được");

        let gin = parse_syllable("gìn").unwrap();
        assert_eq!(gin.reconstruct(ToneStyle::Modern), "gìn");
        assert_eq!(gin.reconstruct(ToneStyle::Classic), "gìn");

        let gi = parse_syllable("gì").unwrap();
        assert_eq!(gi.reconstruct(ToneStyle::Modern), "gì");
        assert_eq!(gi.reconstruct(ToneStyle::Classic), "gì");
    }

    #[test]
    fn test_tone_modification() {
        let toan = parse_syllable("toán").unwrap();
        let toan_huyen = toan.with_tone(ToneMark::Grave);
        assert_eq!(toan_huyen.reconstruct(ToneStyle::Modern), "toàn");

        let toan_hoi = toan.with_tone(ToneMark::HookAbove);
        assert_eq!(toan_hoi.reconstruct(ToneStyle::Modern), "toản");

        let toan_nga = toan.with_tone(ToneMark::Tilde);
        assert_eq!(toan_nga.reconstruct(ToneStyle::Modern), "toãn");

        let toan_nang = toan.with_tone(ToneMark::DotBelow);
        assert_eq!(toan_nang.reconstruct(ToneStyle::Modern), "toạn");

        let toan_ngang = toan.with_tone(ToneMark::None);
        assert_eq!(toan_ngang.reconstruct(ToneStyle::Modern), "toan");
    }

    #[test]
    fn test_casing_preservation() {
        let s1 = parse_syllable("Nghiêng").unwrap();
        assert_eq!(s1.onset, "Ngh");
        assert_eq!(s1.reconstruct(ToneStyle::Modern), "Nghiêng");

        let s2 = parse_syllable("HOÁ").unwrap();
        assert_eq!(s2.onset, "H");
        assert_eq!(s2.reconstruct(ToneStyle::Modern), "HOÁ");
        assert_eq!(s2.reconstruct(ToneStyle::Classic), "HÓA");

        let s3 = parse_syllable("ĐƯỢC").unwrap();
        assert_eq!(s3.reconstruct(ToneStyle::Modern), "ĐƯỢC");

        let s4 = parse_syllable("QUỐC").unwrap();
        assert_eq!(s4.reconstruct(ToneStyle::Modern), "QUỐC");
    }

    #[test]
    fn test_is_valid() {
        assert!(parse_syllable("được").unwrap().is_valid());
        assert!(parse_syllable("nghiêng").unwrap().is_valid());
        assert!(parse_syllable("hoa").unwrap().is_valid());
        assert!(parse_syllable("quốc").unwrap().is_valid());

        // Invalid: stop coda with grave tone
        let mut invalid_stop = parse_syllable("được").unwrap();
        invalid_stop.tone = ToneMark::Grave;
        assert!(!invalid_stop.is_valid());

        // Invalid: open diphthong 'ia' with consonant coda
        let invalid_open_diph = ParsedSyllable::new("m", None, "ia", "n", ToneMark::None);
        assert!(!invalid_open_diph.is_valid());

        // Invalid: 'gh' before 'a'
        let invalid_gh = ParsedSyllable::new("gh", None, "a", "", ToneMark::None);
        assert!(!invalid_gh.is_valid());
    }
}
