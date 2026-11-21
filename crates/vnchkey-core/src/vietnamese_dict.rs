//! Built-in Vietnamese Syllables Dictionary (~6,750 standard words) & Phonology Rules.
//!
//! Ported from OpenViet IME (`vietnamese-words.js` & `dictionary.js`, originally sourced from
//! Hunspell Vietnamese / Hồ Ngọc Đức / OpenOffice / xKey), supporting both Modern (`hoà`, `thuỷ`)
//! and Classic (`hòa`, `thủy`) tone placement as well as Southern Vietnamese vocabulary (`chánh`,
//! `bển`, `trển`, `nghen`, `hổng`, `thiệt`, `nhứt`, `ảnh`, `bả`, `ổng`, `cổ`).

use std::collections::HashSet;
use std::sync::OnceLock;

include!("vietnamese_dict_data.rs");

#[inline]
fn bloom_might_contain(word: &str) -> bool {
    let bytes = word.as_bytes();
    let mut h1: u64 = 0xcbf29ce484222325;
    let mut h2: u64 = 0;
    const K: u64 = 0x517cc1b727220a95;
    for &b in bytes {
        h1 ^= b as u64;
        h1 = h1.wrapping_mul(0x100000001b3);
        h2 = (h2.rotate_left(5) ^ (b as u64)).wrapping_mul(K);
    }
    let idx1 = (h1 as usize) & 16383;
    let idx2 = (h2 as usize) & 16383;
    ((VIETNAMESE_BLOOM_FILTER[idx1 >> 6] >> (idx1 & 63)) & 1) != 0
        && ((VIETNAMESE_BLOOM_FILTER[idx2 >> 6] >> (idx2 & 63)) & 1) != 0
}

#[inline]
fn dict_contains(word: &str) -> bool {
    bloom_might_contain(word) && VIETNAMESE_WORDS_SORTED.binary_search(&word).is_ok()
}

/// Space-separated built-in Vietnamese syllables (~6,717 standard words + regional Southern words).

/// Single Vietnamese vowels and standalone characters.
pub const ALL_VIETNAMESE_SINGLE_VOWELS: &[&str] = &[
    "a", "á", "à", "ả", "ã", "ạ",
    "ă", "ắ", "ằ", "ẳ", "ẵ", "ặ",
    "â", "ấ", "ầ", "ẩ", "ẫ", "ậ",
    "e", "é", "è", "ẻ", "ẽ", "ẹ",
    "ê", "ế", "ề", "ể", "ễ", "ệ",
    "i", "í", "ì", "ỉ", "ĩ", "ị",
    "o", "ó", "ò", "ỏ", "õ", "ọ",
    "ô", "ố", "ồ", "ổ", "ỗ", "ộ",
    "ơ", "ớ", "ờ", "ở", "ỡ", "ợ",
    "u", "ú", "ù", "ủ", "ũ", "ụ",
    "ư", "ứ", "ừ", "ử", "ữ", "ự",
    "y", "ý", "ỳ", "ỷ", "ỹ", "ỵ",
    "đ",
];

/// Normalizes typographical quotes and apostrophes (`’`, `‘`, `ʼ`, ```, `´`) to standard ASCII single quote `'`.
pub fn normalize_apostrophe(s: &str) -> std::borrow::Cow<'_, str> {
    if !s.chars().any(|c| matches!(c, '\u{2018}' | '\u{2019}' | '\u{02BC}' | '`' | '´')) {
        return std::borrow::Cow::Borrowed(s);
    }
    std::borrow::Cow::Owned(
        s.chars()
            .map(|c| match c {
                '\u{2018}' | '\u{2019}' | '\u{02BC}' | '`' | '´' => '\'',
                _ => c,
            })
            .collect(),
    )
}

/// Proper nouns, Tay Nguyen & Vietnamese geographical names and compounds (phrase level).
pub const PLACE_NAMES: &[&str] = &[
    // Tay Nguyen provinces, cities, districts, communes & peaks
    "đắk lắk", "đắk nông", "kon tum", "gia lai", "lâm đồng",
    "buôn ma thuột", "pleiku", "krông nô", "krông pắk", "krông ana",
    "krông a na", "krông búk", "krông bông", "krông năng", "krông a",
    "cư m'gar", "cư kuin", "ea h'leo", "ea kar", "ea súp", "m'đrắk",
    "đắk mil", "đắk song", "đắk glong", "đắk tô", "đắk hà", "đắk đoa",
    "chư sê", "chư prông", "chư păh", "đức cơ", "ia grai", "ia h'drai",
    "sa thầy", "ngọc hồi", "tu mơ rông", "đơn dương", "đam rông",
    "lạc dương", "di linh", "bảo lâm", "cát tiên", "kon plông",
    "a yun pa", "đắk r'lấp", "đăk tô", "tà nung", "ô loan",
    // Notable geographic sites & cities
    "hà nội", "hải phòng", "quảng ninh", "hạ long", "sa pa", "mộc châu",
    "điện biên phủ", "lai châu", "sơn la", "cao bằng", "lạng sơn",
    "huế", "đà nẵng", "hội an", "quy nhơn", "nha trang", "đà lạt",
    "phan thiết", "quảng ngãi", "tam kỳ", "đông hà", "vinh",
    "sài gòn", "cần thơ", "vũng tàu", "biên hòa", "mỹ tho",
    "bến tre", "long xuyên", "rạch giá", "cà mau", "bạc liêu", "sóc trăng",
    "biển hồ", "thác bà",
];

/// Rare Han-Viet words, classical syllables, and southern dialect / internet slang.
pub const RARE_WORDS: &[&str] = &[
    // Rare Han-Viet words
    "khuếch", "khuynh", "khoảnh", "ngoắt", "ngoèo", "nghễu", "nguầy", "huỳnh", "quỳnh",
    "khuỷu", "khuấy", "khuây", "ngoáy", "nguậy", "quánh", "quạnh", "quắt", "quặp",
    "nguệch", "ngoạc",
    // Interjections, spoken and southern dialect words
    "dzô", "dzậy", "hông", "hổng", "zị", "zi", "zia", "hén", "nghen", "tui", "mầy",
    "chảnh", "xạo",
    // Southern & Saigon classical vocabulary ("Gom góp từ ngữ của miền Nam và Saigon xưa")
    "chớn", "dzách", "dách", "quỉ", "quởn", "quỡn", "quơn", "quớt", "tỏn", "xề",
    "nhổng", "cháng", "chác", "cháo", "lữa", "nư", "hum", "mụi", "lét", "đéc",
];

static VIETNAMESE_WORD_SET: OnceLock<HashSet<String>> = OnceLock::new();

/// Returns the number of words in the built-in sorted Vietnamese dictionary without allocating a HashSet.
pub fn vietnamese_word_count() -> usize {
    VIETNAMESE_WORDS_SORTED.len()
}

/// Returns a reference to the globally initialized Vietnamese word dictionary.
pub fn vietnamese_word_set() -> &'static HashSet<String> {
    VIETNAMESE_WORD_SET.get_or_init(|| {
        let mut set = HashSet::with_capacity(7500);
        for &word in VIETNAMESE_WORDS_SORTED {
            set.insert(word.to_string());
        }
        for &v in ALL_VIETNAMESE_SINGLE_VOWELS {
            set.insert(v.to_string());
        }
        for &w in RARE_WORDS {
            set.insert(w.to_string());
        }
        // Insert both the full phrase and individual syllables of each place name
        for &place in PLACE_NAMES {
            set.insert(place.to_string());
            for token in place.split_whitespace() {
                set.insert(token.to_string());
                if token.contains('\'') {
                    for part in token.split('\'') {
                        if !part.is_empty() {
                            set.insert(part.to_string());
                        }
                    }
                }
            }
        }
        set
    })
}

/// Normalizes tone style between Modern (`oà`, `oè`, `uý`) and Classic (`òa`, `òe`, `úy`)
/// so both styles are recognized by the built-in dictionary.
pub fn normalize_tone_variants(word: &str) -> (String, String) {
    let lower = word.to_lowercase();
    let modern_to_classic = [
        ("oà", "òa"), ("oá", "óa"), ("oả", "ỏa"), ("oã", "õa"), ("oạ", "ọa"),
        ("oè", "òe"), ("oé", "óe"), ("oẻ", "ỏe"), ("oẽ", "õe"), ("oẹ", "ọe"),
        ("uỳ", "ùy"), ("uý", "úy"), ("uỷ", "ủy"), ("uỹ", "ũy"), ("uỵ", "ụy"),
    ];

    let mut as_classic = lower.clone();
    let mut as_modern = lower;
    for &(mod_pat, cls_pat) in &modern_to_classic {
        if as_classic.contains(mod_pat) {
            as_classic = as_classic.replace(mod_pat, cls_pat);
        }
        if as_modern.contains(cls_pat) {
            as_modern = as_modern.replace(cls_pat, mod_pat);
        }
    }
    (as_modern, as_classic)
}

/// Checks if a completed word is a valid Vietnamese syllable in the built-in ~6,750-word dictionary.
pub fn is_in_vietnamese_dictionary(word: &str) -> bool {
    if word.is_empty() {
        return false;
    }
    // Allow all-caps Vietnamese abbreviations containing Đ (e.g. ĐN, HĐND, ĐHQG, PGĐ)
    if word.chars().count() >= 2
        && word.chars().all(|c| c.is_uppercase())
        && (word.contains('Đ') || word.contains('đ'))
    {
        return true;
    }
    let normalized = normalize_apostrophe(word);
    let (modern, classic) = normalize_tone_variants(&normalized);
    dict_contains(&modern) || dict_contains(&classic)
}

/// Checks Vietnamese phonology rules (`c/k`, `g/gh`, `ng/ngh`) from OpenViet IME:
/// - Rule 1 (`c/k`): `'c'` (unless `'ch'`) is never followed by `'e'`, `'ê'`, `'i'`, `'y'` (`center`, `city`, `cell`).
/// - Rule 2 (`g/gh`): `'g'` (unless `'gh'` or `'gi'`) is never followed by `'e'`, `'ê'` (`get`, `general`, `gear`).
/// - Rule 3 (`ng/ngh`): `'ng'` (unless `'ngh'`) is never followed by `'e'`, `'ê'`, `'i'` (`danger`, `finger`, `engine`).
pub fn has_phonology_violation(word: &str) -> bool {
    let chars: Vec<char> = word.chars().map(|c| c.to_lowercase().next().unwrap_or(c)).collect();
    if chars.len() < 2 {
        return false;
    }

    let is_e_family = |c: char| {
        matches!(
            c,
            'e' | 'é' | 'è' | 'ẻ' | 'ẽ' | 'ẹ' | 'ê' | 'ế' | 'ề' | 'ể' | 'ễ' | 'ệ'
        )
    };
    let is_i_family = |c: char| matches!(c, 'i' | 'í' | 'ì' | 'ỉ' | 'ĩ' | 'ị');
    let is_y_family = |c: char| matches!(c, 'y' | 'ý' | 'ỳ' | 'ỷ' | 'ỹ' | 'ỵ');

    // Rule 1: 'c' followed by e, ê, i, y
    if chars[0] == 'c' && (is_e_family(chars[1]) || is_i_family(chars[1]) || is_y_family(chars[1])) {
        return true;
    }

    // Rule 2: 'g' followed directly by e, ê
    if chars[0] == 'g' && is_e_family(chars[1]) {
        return true;
    }

    // Rule 3: 'ng' followed directly by e, ê, i
    if chars.len() >= 3
        && chars[0] == 'n'
        && chars[1] == 'g'
        && (is_e_family(chars[2]) || is_i_family(chars[2]))
    {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_vietnamese_dictionary() {
        assert!(is_in_vietnamese_dictionary("được"));
        assert!(is_in_vietnamese_dictionary("không"));
        assert!(is_in_vietnamese_dictionary("giờ"));
        assert!(is_in_vietnamese_dictionary("chánh"));
        assert!(is_in_vietnamese_dictionary("hoà"));
        assert!(is_in_vietnamese_dictionary("hòa"));
        assert!(is_in_vietnamese_dictionary("thuỷ"));
        assert!(is_in_vietnamese_dictionary("thủy"));
        // Southern dialect & modern internet words
        assert!(is_in_vietnamese_dictionary("dzìa"));
        assert!(is_in_vietnamese_dictionary("dzô"));
        assert!(is_in_vietnamese_dictionary("dzí"));
        assert!(is_in_vietnamese_dictionary("dzị"));
        assert!(is_in_vietnamese_dictionary("dzậy"));
        assert!(is_in_vietnamese_dictionary("gởi"));
        assert!(is_in_vietnamese_dictionary("gời"));
        assert!(!is_in_vietnamese_dictionary("têt"));
        assert!(!is_in_vietnamese_dictionary("windoư"));
    }

    #[test]
    fn test_tay_nguyen_and_rare_words_dictionary() {
        // Apostrophe normalization
        assert_eq!(normalize_apostrophe("M’Đrắk"), "M'Đrắk");
        assert_eq!(normalize_apostrophe("Cư M‘gar"), "Cư M'gar");
        assert_eq!(normalize_apostrophe("Ea Hʼleo"), "Ea H'leo");
        assert!(is_in_vietnamese_dictionary("m’đrắk"));
        assert!(is_in_vietnamese_dictionary("m'đrắk"));
        assert!(is_in_vietnamese_dictionary("m’gar"));

        // Full compound place names
        let compounds = [
            "đắk lắk", "buôn ma thuột", "krông a na", "kon plông", "cư m'gar",
            "ea h'leo", "m'đrắk", "chư prông", "ia grai", "sa thầy", "hà nội",
        ];
        for compound in &compounds {
            assert!(
                is_in_vietnamese_dictionary(compound),
                "Expected compound '{}' to be in dictionary",
                compound
            );
        }

        // Tay Nguyen & ethnic place name syllables
        let tay_nguyen_words = [
            "đắk", "đăk", "lắk", "nông", "kon", "tum", "gia", "lai", "buôn", "ma", "thuột",
            "pleiku", "krông", "nô", "búk", "pắk", "ana", "bông", "năng", "cư", "kuin",
            "ea", "kar", "súp", "m'đrắk", "đrắk", "mil", "glong", "đoa", "chư", "sê",
            "prông", "păh", "đức", "cơ", "ia", "grai", "drai", "mơ", "rông", "đam", "di",
            "linh", "r'lấp", "nung",
        ];
        for word in &tay_nguyen_words {
            assert!(
                is_in_vietnamese_dictionary(word),
                "Expected '{}' to be in dictionary",
                word
            );
        }

        // Rare Han-Viet words
        let rare_words = [
            "khuếch", "khuynh", "khoảnh", "ngoắt", "ngoèo", "nghễu", "nguầy", "huỳnh", "quỳnh",
            "khuỷu", "khuấy", "khuây", "ngoáy", "nguậy", "quánh", "quạnh", "quắt", "quặp",
        ];
        for word in &rare_words {
            assert!(
                is_in_vietnamese_dictionary(word),
                "Expected '{}' to be in dictionary",
                word
            );
        }

        // Dialect & internet slang
        let slang_words = [
            "dzô", "dzậy", "hông", "hổng", "zị", "zi", "zia", "hén", "nghen", "tui", "mầy",
            "tao", "chảnh", "xạo", "sẩy", "trời", "ơi",
        ];
        for word in &slang_words {
            assert!(
                is_in_vietnamese_dictionary(word),
                "Expected '{}' to be in dictionary",
                word
            );
        }
    }

    #[test]
    fn test_phonology_violations() {
        assert!(has_phonology_violation("cẻnt"));
        assert!(has_phonology_violation("cỉty"));
        assert!(has_phonology_violation("gểt"));
        assert!(has_phonology_violation("ngẻ"));
        assert!(!has_phonology_violation("kể"));
        assert!(!has_phonology_violation("ghế"));
        assert!(!has_phonology_violation("nghệ"));
        assert!(!has_phonology_violation("giờ"));
    }
}
