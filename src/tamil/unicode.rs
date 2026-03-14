use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VowelLength {
    Kuril,
    Nedil,
}

impl VowelLength {
    pub fn as_str(self) -> &'static str {
        match self {
            VowelLength::Kuril => "kuril",
            VowelLength::Nedil => "nedil",
        }
    }
}

// Tamil vowels (உயிர்) - 12 letters
pub const VOWELS: [char; 12] = [
    'அ', // U+0B85
    'ஆ', // U+0B86
    'இ', // U+0B87
    'ஈ', // U+0B88
    'உ', // U+0B89
    'ஊ', // U+0B8A
    'எ', // U+0B8E
    'ஏ', // U+0B8F
    'ஐ', // U+0B90
    'ஒ', // U+0B92
    'ஓ', // U+0B93
    'ஔ', // U+0B94
];

// Tamil consonants (மெய்) - 18 letters
pub const CONSONANTS: [char; 18] = [
    'க', // U+0B95
    'ங', // U+0B99
    'ச', // U+0B9A
    'ஞ', // U+0B9E
    'ட', // U+0B9F
    'ண', // U+0BA3
    'த', // U+0BA4
    'ந', // U+0BA8
    'ன', // U+0BA9
    'ப', // U+0BAA
    'ம', // U+0BAE
    'ய', // U+0BAF
    'ர', // U+0BB0
    'ற', // U+0BB1
    'ல', // U+0BB2
    'ள', // U+0BB3
    'ழ', // U+0BB4
    'வ', // U+0BB5
];

// Grantha consonants (used in loanwords)
pub const GRANTHA: [char; 5] = [
    'ஜ', // U+0B9C
    'ஶ', // U+0BB6
    'ஷ', // U+0BB7
    'ஸ', // U+0BB8
    'ஹ', // U+0BB9
];

pub const PULLI: char = '\u{0BCD}';
pub const AYTHAM: char = '\u{0B83}';
pub const DANDA: char = '\u{0964}';
pub const DOUBLE_DANDA: char = '\u{0965}';

// Vallinam (hard) consonants — the six consonants that can form kutrilugaram
pub const VALLINAM: [char; 6] = ['க', 'ச', 'ட', 'த', 'ப', 'ற'];
const U_MATRA: char = '\u{0BC1}'; // ு (short 'u' matra)

pub fn is_vallinam(c: char) -> bool {
    VALLINAM.contains(&c)
}

/// Check if a grapheme text represents kutrilugaram (vallinam consonant + short 'u' matra).
/// Kutrilugaram is the shortened 'u' sound (half matra) at the end of a word.
pub fn is_kutrilugaram_ending(text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    chars.len() == 2 && is_vallinam(chars[0]) && chars[1] == U_MATRA
}

// Kuril (short) vowels
const KURIL_VOWELS: [char; 5] = ['அ', 'இ', 'உ', 'எ', 'ஒ'];
// Nedil (long) vowels
const NEDIL_VOWELS: [char; 7] = ['ஆ', 'ஈ', 'ஊ', 'ஏ', 'ஐ', 'ஓ', 'ஔ'];

// Kuril matras
const KURIL_MATRAS: [char; 4] = [
    '\u{0BBF}', // ி
    '\u{0BC1}', // ு
    '\u{0BC6}', // ெ
    '\u{0BCA}', // ொ
];

// Nedil matras
const NEDIL_MATRAS: [char; 7] = [
    '\u{0BBE}', // ா
    '\u{0BC0}', // ீ
    '\u{0BC2}', // ூ
    '\u{0BC7}', // ே
    '\u{0BC8}', // ை
    '\u{0BCB}', // ோ
    '\u{0BCC}', // ௌ
];

pub fn normalize_nfc(text: &str) -> String {
    text.nfc().collect()
}

pub fn is_tamil_char(c: char) -> bool {
    ('\u{0B80}'..='\u{0BFF}').contains(&c) || c == DANDA || c == DOUBLE_DANDA
}

pub fn is_consonant(c: char) -> bool {
    CONSONANTS.contains(&c) || GRANTHA.contains(&c)
}

pub fn is_vowel(c: char) -> bool {
    VOWELS.contains(&c)
}

pub fn is_vowel_matra(c: char) -> bool {
    ('\u{0BBE}'..='\u{0BCC}').contains(&c)
}

pub fn is_pulli(c: char) -> bool {
    c == PULLI
}

pub fn is_aytham(c: char) -> bool {
    c == AYTHAM
}

pub fn vowel_length(c: char) -> Option<VowelLength> {
    if KURIL_VOWELS.contains(&c) {
        Some(VowelLength::Kuril)
    } else if NEDIL_VOWELS.contains(&c) {
        Some(VowelLength::Nedil)
    } else {
        None
    }
}

pub fn matra_vowel_length(c: char) -> Option<VowelLength> {
    if KURIL_MATRAS.contains(&c) {
        Some(VowelLength::Kuril)
    } else if NEDIL_MATRAS.contains(&c) {
        Some(VowelLength::Nedil)
    } else {
        None
    }
}

/// Returns the monai group for a standalone vowel.
/// Short/long pairs share the same group (e.g., அ/ஆ → 'அ').
pub fn vowel_monai_group(c: char) -> Option<char> {
    match c {
        'அ' | 'ஆ' => Some('அ'),
        'இ' | 'ஈ' => Some('இ'),
        'உ' | 'ஊ' => Some('உ'),
        'எ' | 'ஏ' => Some('எ'),
        'ஐ' => Some('ஐ'),
        'ஒ' | 'ஓ' => Some('ஒ'),
        'ஔ' => Some('ஔ'),
        _ => None,
    }
}

/// Map a vowel sign (matra) to its standalone vowel letter.
/// Used for sandhi decomposition: splitting uyirmei back into mei + vowel.
pub fn matra_to_vowel(matra: char) -> Option<char> {
    match matra {
        '\u{0BBE}' => Some('ஆ'), // ா → ஆ
        '\u{0BBF}' => Some('இ'), // ி → இ
        '\u{0BC0}' => Some('ஈ'), // ீ → ஈ
        '\u{0BC1}' => Some('உ'), // ு → உ
        '\u{0BC2}' => Some('ஊ'), // ூ → ஊ
        '\u{0BC6}' => Some('எ'), // ெ → எ
        '\u{0BC7}' => Some('ஏ'), // ே → ஏ
        '\u{0BC8}' => Some('ஐ'), // ை → ஐ
        '\u{0BCA}' => Some('ஒ'), // ொ → ஒ
        '\u{0BCB}' => Some('ஓ'), // ோ → ஓ
        '\u{0BCC}' => Some('ஔ'), // ௌ → ஔ
        _ => None,
    }
}

/// Strip all non-Tamil-script characters from a word.
/// Keeps only U+0B80–U+0BFF (Tamil block), removing dandas, hyphens, dots, etc.
pub fn strip_non_tamil(word: &str) -> (String, bool) {
    let stripped: String = word
        .chars()
        .filter(|&c| is_tamil_char(c) && c != DANDA && c != DOUBLE_DANDA)
        .collect();
    let was_stripped = stripped.len() != word.len();
    (stripped, was_stripped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nfc_normalization() {
        let text = "அகர";
        let normalized = normalize_nfc(text);
        assert_eq!(text, normalized);
    }

    #[test]
    fn test_vowel_classification() {
        assert_eq!(vowel_length('அ'), Some(VowelLength::Kuril));
        assert_eq!(vowel_length('ஆ'), Some(VowelLength::Nedil));
        assert_eq!(vowel_length('இ'), Some(VowelLength::Kuril));
        assert_eq!(vowel_length('ஈ'), Some(VowelLength::Nedil));
        assert_eq!(vowel_length('உ'), Some(VowelLength::Kuril));
        assert_eq!(vowel_length('ஊ'), Some(VowelLength::Nedil));
        assert_eq!(vowel_length('எ'), Some(VowelLength::Kuril));
        assert_eq!(vowel_length('ஏ'), Some(VowelLength::Nedil));
        assert_eq!(vowel_length('ஐ'), Some(VowelLength::Nedil));
        assert_eq!(vowel_length('ஒ'), Some(VowelLength::Kuril));
        assert_eq!(vowel_length('ஓ'), Some(VowelLength::Nedil));
        assert_eq!(vowel_length('ஔ'), Some(VowelLength::Nedil));
    }

    #[test]
    fn test_matra_classification() {
        assert_eq!(matra_vowel_length('\u{0BBE}'), Some(VowelLength::Nedil)); // ா
        assert_eq!(matra_vowel_length('\u{0BBF}'), Some(VowelLength::Kuril)); // ி
        assert_eq!(matra_vowel_length('\u{0BC0}'), Some(VowelLength::Nedil)); // ீ
        assert_eq!(matra_vowel_length('\u{0BC1}'), Some(VowelLength::Kuril)); // ு
        assert_eq!(matra_vowel_length('\u{0BC2}'), Some(VowelLength::Nedil)); // ூ
        assert_eq!(matra_vowel_length('\u{0BC6}'), Some(VowelLength::Kuril)); // ெ
        assert_eq!(matra_vowel_length('\u{0BC7}'), Some(VowelLength::Nedil)); // ே
        assert_eq!(matra_vowel_length('\u{0BC8}'), Some(VowelLength::Nedil)); // ை
    }

    #[test]
    fn test_consonant_check() {
        assert!(is_consonant('க'));
        assert!(is_consonant('ன'));
        assert!(is_consonant('ஜ')); // grantha
        assert!(!is_consonant('அ')); // vowel
    }
}
