//! Mathematical Glyphs & Diacritic Algebra for Vietnamese Orthography.
//!
//! Models a Vietnamese vowel as:
//!
//! ```text
//! Letter = Base ASCII ('a', 'e', 'i', 'o', 'u', 'y', 'd')
//!        x Quality Modifier (None, Circumflex 'â'/'ê'/'ô', Breve 'ă', Horn 'ơ'/'ư', Stroke 'đ')
//!        x Tone Mark (Level / Ngang, Acute / Sắc, Grave / Huyền, Hook / Hỏi, Tilde / Ngã, Dot / Nặng)
//! ```
//!
//! Provides zero-dependency, constant-time `O(1)` algebraic operations to compose,
//! decompose, strip, and swap modifiers and tones without external runtime crates.

/// Quality mark modifier (mũ, trăng, móc, gạch ngang).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QualityModifier {
    /// No quality modifier (plain vowel).
    #[default]
    None,
    /// Circumflex / Mũ (`â`, `ê`, `ô`)
    Circumflex,
    /// Breve / Trăng (`ă`)
    Breve,
    /// Horn / Móc (`ơ`, `ư`)
    Horn,
    /// Stroke (`đ` / `Đ`)
    Stroke,
}

/// Tone mark (0 = Ngang, 1 = Sắc, 2 = Huyền, 3 = Hỏi, 4 = Ngã, 5 = Nặng).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ToneMark {
    /// Level / Ngang (no tone)
    #[default]
    Level = 0,
    /// Acute / Sắc
    Acute = 1,
    /// Grave / Huyền
    Grave = 2,
    /// Hook / Hỏi
    Hook = 3,
    /// Tilde / Ngã
    Tilde = 4,
    /// Dot / Nặng
    Dot = 5,
}

impl ToneMark {
    /// Creates a `ToneMark` from numeric tone index (`0..=5`).
    #[inline]
    pub fn from_index(idx: usize) -> Self {
        match idx {
            1 => Self::Acute,
            2 => Self::Grave,
            3 => Self::Hook,
            4 => Self::Tilde,
            5 => Self::Dot,
            _ => Self::Level,
        }
    }

    /// Converts `ToneMark` into numeric tone index (`0..=5`).
    #[inline]
    pub fn to_index(self) -> usize {
        self as usize
    }

    /// Unicode combining diacritical mark for this tone.
    #[inline]
    pub fn combining_char(self) -> Option<char> {
        match self {
            Self::Level => None,
            Self::Acute => Some('\u{0301}'),
            Self::Grave => Some('\u{0300}'),
            Self::Hook => Some('\u{0309}'),
            Self::Tilde => Some('\u{0303}'),
            Self::Dot => Some('\u{0323}'),
        }
    }
}

/// Fully decomposed Vietnamese glyph structure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VietnameseGlyph {
    /// ASCII base character ('a', 'e', 'i', 'o', 'u', 'y', 'd')
    pub base: char,
    /// Quality modifier (Circumflex, Breve, Horn, Stroke, or None)
    pub quality: QualityModifier,
    /// Tone mark (0..=5)
    pub tone: ToneMark,
    /// Whether the original glyph was uppercase
    pub is_upper: bool,
}

impl VietnameseGlyph {
    /// Decomposes any Unicode Vietnamese character into its algebraic components.
    /// Returns `None` if `c` is not a Vietnamese vowel or `đ`/`Đ`.
    pub fn decompose(c: char) -> Option<Self> {
        let is_upper = c.is_uppercase();
        let lower = c.to_lowercase().next().unwrap_or(c);

        if lower == 'đ' {
            return Some(Self {
                base: 'd',
                quality: QualityModifier::Stroke,
                tone: ToneMark::Level,
                is_upper,
            });
        }

        let (base, quality, tone) = match lower {
            'a' => ('a', QualityModifier::None, ToneMark::Level),
            'á' => ('a', QualityModifier::None, ToneMark::Acute),
            'à' => ('a', QualityModifier::None, ToneMark::Grave),
            'ả' => ('a', QualityModifier::None, ToneMark::Hook),
            'ã' => ('a', QualityModifier::None, ToneMark::Tilde),
            'ạ' => ('a', QualityModifier::None, ToneMark::Dot),

            'ă' => ('a', QualityModifier::Breve, ToneMark::Level),
            'ắ' => ('a', QualityModifier::Breve, ToneMark::Acute),
            'ằ' => ('a', QualityModifier::Breve, ToneMark::Grave),
            'ẳ' => ('a', QualityModifier::Breve, ToneMark::Hook),
            'ẵ' => ('a', QualityModifier::Breve, ToneMark::Tilde),
            'ặ' => ('a', QualityModifier::Breve, ToneMark::Dot),

            'â' => ('a', QualityModifier::Circumflex, ToneMark::Level),
            'ấ' => ('a', QualityModifier::Circumflex, ToneMark::Acute),
            'ầ' => ('a', QualityModifier::Circumflex, ToneMark::Grave),
            'ẩ' => ('a', QualityModifier::Circumflex, ToneMark::Hook),
            'ẫ' => ('a', QualityModifier::Circumflex, ToneMark::Tilde),
            'ậ' => ('a', QualityModifier::Circumflex, ToneMark::Dot),

            'e' => ('e', QualityModifier::None, ToneMark::Level),
            'é' => ('e', QualityModifier::None, ToneMark::Acute),
            'è' => ('e', QualityModifier::None, ToneMark::Grave),
            'ẻ' => ('e', QualityModifier::None, ToneMark::Hook),
            'ẽ' => ('e', QualityModifier::None, ToneMark::Tilde),
            'ẹ' => ('e', QualityModifier::None, ToneMark::Dot),

            'ê' => ('e', QualityModifier::Circumflex, ToneMark::Level),
            'ế' => ('e', QualityModifier::Circumflex, ToneMark::Acute),
            'ề' => ('e', QualityModifier::Circumflex, ToneMark::Grave),
            'ể' => ('e', QualityModifier::Circumflex, ToneMark::Hook),
            'ễ' => ('e', QualityModifier::Circumflex, ToneMark::Tilde),
            'ệ' => ('e', QualityModifier::Circumflex, ToneMark::Dot),

            'i' => ('i', QualityModifier::None, ToneMark::Level),
            'í' => ('i', QualityModifier::None, ToneMark::Acute),
            'ì' => ('i', QualityModifier::None, ToneMark::Grave),
            'ỉ' => ('i', QualityModifier::None, ToneMark::Hook),
            'ĩ' => ('i', QualityModifier::None, ToneMark::Tilde),
            'ị' => ('i', QualityModifier::None, ToneMark::Dot),

            'o' => ('o', QualityModifier::None, ToneMark::Level),
            'ó' => ('o', QualityModifier::None, ToneMark::Acute),
            'ò' => ('o', QualityModifier::None, ToneMark::Grave),
            'ỏ' => ('o', QualityModifier::None, ToneMark::Hook),
            'õ' => ('o', QualityModifier::None, ToneMark::Tilde),
            'ọ' => ('o', QualityModifier::None, ToneMark::Dot),

            'ô' => ('o', QualityModifier::Circumflex, ToneMark::Level),
            'ố' => ('o', QualityModifier::Circumflex, ToneMark::Acute),
            'ồ' => ('o', QualityModifier::Circumflex, ToneMark::Grave),
            'ổ' => ('o', QualityModifier::Circumflex, ToneMark::Hook),
            'ỗ' => ('o', QualityModifier::Circumflex, ToneMark::Tilde),
            'ộ' => ('o', QualityModifier::Circumflex, ToneMark::Dot),

            'ơ' => ('o', QualityModifier::Horn, ToneMark::Level),
            'ớ' => ('o', QualityModifier::Horn, ToneMark::Acute),
            'ờ' => ('o', QualityModifier::Horn, ToneMark::Grave),
            'ở' => ('o', QualityModifier::Horn, ToneMark::Hook),
            'ỡ' => ('o', QualityModifier::Horn, ToneMark::Tilde),
            'ợ' => ('o', QualityModifier::Horn, ToneMark::Dot),

            'u' => ('u', QualityModifier::None, ToneMark::Level),
            'ú' => ('u', QualityModifier::None, ToneMark::Acute),
            'ù' => ('u', QualityModifier::None, ToneMark::Grave),
            'ủ' => ('u', QualityModifier::None, ToneMark::Hook),
            'ũ' => ('u', QualityModifier::None, ToneMark::Tilde),
            'ụ' => ('u', QualityModifier::None, ToneMark::Dot),

            'ư' => ('u', QualityModifier::Horn, ToneMark::Level),
            'ứ' => ('u', QualityModifier::Horn, ToneMark::Acute),
            'ừ' => ('u', QualityModifier::Horn, ToneMark::Grave),
            'ử' => ('u', QualityModifier::Horn, ToneMark::Hook),
            'ữ' => ('u', QualityModifier::Horn, ToneMark::Tilde),
            'ự' => ('u', QualityModifier::Horn, ToneMark::Dot),

            'y' => ('y', QualityModifier::None, ToneMark::Level),
            'ý' => ('y', QualityModifier::None, ToneMark::Acute),
            'ỳ' => ('y', QualityModifier::None, ToneMark::Grave),
            'ỷ' => ('y', QualityModifier::None, ToneMark::Hook),
            'ỹ' => ('y', QualityModifier::None, ToneMark::Tilde),
            'ỵ' => ('y', QualityModifier::None, ToneMark::Dot),

            _ => return None,
        };

        Some(Self {
            base,
            quality,
            tone,
            is_upper,
        })
    }

    /// Recomposes algebraic components into a canonical NFC precomposed character.
    pub fn compose(self) -> char {
        if self.base == 'd' && self.quality == QualityModifier::Stroke {
            return if self.is_upper { 'Đ' } else { 'đ' };
        }

        let ch = match (self.base, self.quality, self.tone) {
            ('a', QualityModifier::None, ToneMark::Level) => 'a',
            ('a', QualityModifier::None, ToneMark::Acute) => 'á',
            ('a', QualityModifier::None, ToneMark::Grave) => 'à',
            ('a', QualityModifier::None, ToneMark::Hook) => 'ả',
            ('a', QualityModifier::None, ToneMark::Tilde) => 'ã',
            ('a', QualityModifier::None, ToneMark::Dot) => 'ạ',

            ('a', QualityModifier::Breve, ToneMark::Level) => 'ă',
            ('a', QualityModifier::Breve, ToneMark::Acute) => 'ắ',
            ('a', QualityModifier::Breve, ToneMark::Grave) => 'ằ',
            ('a', QualityModifier::Breve, ToneMark::Hook) => 'ẳ',
            ('a', QualityModifier::Breve, ToneMark::Tilde) => 'ẵ',
            ('a', QualityModifier::Breve, ToneMark::Dot) => 'ặ',

            ('a', QualityModifier::Circumflex, ToneMark::Level) => 'â',
            ('a', QualityModifier::Circumflex, ToneMark::Acute) => 'ấ',
            ('a', QualityModifier::Circumflex, ToneMark::Grave) => 'ầ',
            ('a', QualityModifier::Circumflex, ToneMark::Hook) => 'ẩ',
            ('a', QualityModifier::Circumflex, ToneMark::Tilde) => 'ẫ',
            ('a', QualityModifier::Circumflex, ToneMark::Dot) => 'ậ',

            ('e', QualityModifier::None, ToneMark::Level) => 'e',
            ('e', QualityModifier::None, ToneMark::Acute) => 'é',
            ('e', QualityModifier::None, ToneMark::Grave) => 'è',
            ('e', QualityModifier::None, ToneMark::Hook) => 'ẻ',
            ('e', QualityModifier::None, ToneMark::Tilde) => 'ẽ',
            ('e', QualityModifier::None, ToneMark::Dot) => 'ẹ',

            ('e', QualityModifier::Circumflex, ToneMark::Level) => 'ê',
            ('e', QualityModifier::Circumflex, ToneMark::Acute) => 'ế',
            ('e', QualityModifier::Circumflex, ToneMark::Grave) => 'ề',
            ('e', QualityModifier::Circumflex, ToneMark::Hook) => 'ể',
            ('e', QualityModifier::Circumflex, ToneMark::Tilde) => 'ễ',
            ('e', QualityModifier::Circumflex, ToneMark::Dot) => 'ệ',

            ('i', QualityModifier::None, ToneMark::Level) => 'i',
            ('i', QualityModifier::None, ToneMark::Acute) => 'í',
            ('i', QualityModifier::None, ToneMark::Grave) => 'ì',
            ('i', QualityModifier::None, ToneMark::Hook) => 'ỉ',
            ('i', QualityModifier::None, ToneMark::Tilde) => 'ĩ',
            ('i', QualityModifier::None, ToneMark::Dot) => 'ị',

            ('o', QualityModifier::None, ToneMark::Level) => 'o',
            ('o', QualityModifier::None, ToneMark::Acute) => 'ó',
            ('o', QualityModifier::None, ToneMark::Grave) => 'ò',
            ('o', QualityModifier::None, ToneMark::Hook) => 'ỏ',
            ('o', QualityModifier::None, ToneMark::Tilde) => 'õ',
            ('o', QualityModifier::None, ToneMark::Dot) => 'ọ',

            ('o', QualityModifier::Circumflex, ToneMark::Level) => 'ô',
            ('o', QualityModifier::Circumflex, ToneMark::Acute) => 'ố',
            ('o', QualityModifier::Circumflex, ToneMark::Grave) => 'ồ',
            ('o', QualityModifier::Circumflex, ToneMark::Hook) => 'ổ',
            ('o', QualityModifier::Circumflex, ToneMark::Tilde) => 'ỗ',
            ('o', QualityModifier::Circumflex, ToneMark::Dot) => 'ộ',

            ('o', QualityModifier::Horn, ToneMark::Level) => 'ơ',
            ('o', QualityModifier::Horn, ToneMark::Acute) => 'ớ',
            ('o', QualityModifier::Horn, ToneMark::Grave) => 'ờ',
            ('o', QualityModifier::Horn, ToneMark::Hook) => 'ở',
            ('o', QualityModifier::Horn, ToneMark::Tilde) => 'ỡ',
            ('o', QualityModifier::Horn, ToneMark::Dot) => 'ợ',

            ('u', QualityModifier::None, ToneMark::Level) => 'u',
            ('u', QualityModifier::None, ToneMark::Acute) => 'ú',
            ('u', QualityModifier::None, ToneMark::Grave) => 'ù',
            ('u', QualityModifier::None, ToneMark::Hook) => 'ủ',
            ('u', QualityModifier::None, ToneMark::Tilde) => 'ũ',
            ('u', QualityModifier::None, ToneMark::Dot) => 'ụ',

            ('u', QualityModifier::Horn, ToneMark::Level) => 'ư',
            ('u', QualityModifier::Horn, ToneMark::Acute) => 'ứ',
            ('u', QualityModifier::Horn, ToneMark::Grave) => 'ừ',
            ('u', QualityModifier::Horn, ToneMark::Hook) => 'ử',
            ('u', QualityModifier::Horn, ToneMark::Tilde) => 'ữ',
            ('u', QualityModifier::Horn, ToneMark::Dot) => 'ự',

            ('y', QualityModifier::None, ToneMark::Level) => 'y',
            ('y', QualityModifier::None, ToneMark::Acute) => 'ý',
            ('y', QualityModifier::None, ToneMark::Grave) => 'ỳ',
            ('y', QualityModifier::None, ToneMark::Hook) => 'ỷ',
            ('y', QualityModifier::None, ToneMark::Tilde) => 'ỹ',
            ('y', QualityModifier::None, ToneMark::Dot) => 'ỵ',

            _ => self.base,
        };

        if self.is_upper {
            ch.to_uppercase().next().unwrap_or(ch)
        } else {
            ch
        }
    }

    /// Changes or sets the tone mark while preserving the base and quality modifier.
    #[inline]
    pub fn with_tone(mut self, tone: ToneMark) -> Self {
        self.tone = tone;
        self
    }

    /// Changes or sets the quality modifier (e.g. `o` -> `ô` or `ơ`) while preserving the tone.
    #[inline]
    pub fn with_quality(mut self, quality: QualityModifier) -> Self {
        self.quality = quality;
        self
    }
}

/// Helper function to strip all marks from a character, returning ASCII base letter.
#[inline]
pub fn base_ascii(c: char) -> char {
    if let Some(glyph) = VietnameseGlyph::decompose(c) {
        if glyph.is_upper {
            glyph.base.to_ascii_uppercase()
        } else {
            glyph.base
        }
    } else {
        c
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glyph_roundtrip_all_tones_and_qualities() {
        let samples = [
            ('á', 'a', QualityModifier::None, ToneMark::Acute, false),
            ('Ă', 'a', QualityModifier::Breve, ToneMark::Level, true),
            ('ậ', 'a', QualityModifier::Circumflex, ToneMark::Dot, false),
            ('Ề', 'e', QualityModifier::Circumflex, ToneMark::Grave, true),
            ('ợ', 'o', QualityModifier::Horn, ToneMark::Dot, false),
            ('Ứ', 'u', QualityModifier::Horn, ToneMark::Acute, true),
            ('đ', 'd', QualityModifier::Stroke, ToneMark::Level, false),
            ('Đ', 'd', QualityModifier::Stroke, ToneMark::Level, true),
            ('ỹ', 'y', QualityModifier::None, ToneMark::Tilde, false),
        ];

        for (ch, base, quality, tone, is_upper) in samples {
            let glyph = VietnameseGlyph::decompose(ch).expect("Must decompose");
            assert_eq!(glyph.base, base);
            assert_eq!(glyph.quality, quality);
            assert_eq!(glyph.tone, tone);
            assert_eq!(glyph.is_upper, is_upper);
            assert_eq!(glyph.compose(), ch);
        }
    }

    #[test]
    fn test_glyph_tone_swapping_preserves_quality() {
        let glyph = VietnameseGlyph::decompose('ở').unwrap();
        assert_eq!(glyph.quality, QualityModifier::Horn);
        assert_eq!(glyph.tone, ToneMark::Hook);

        let new_ch = glyph.with_tone(ToneMark::Acute).compose();
        assert_eq!(new_ch, 'ớ');

        let level_ch = glyph.with_tone(ToneMark::Level).compose();
        assert_eq!(level_ch, 'ơ');
    }
}
