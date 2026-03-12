/// Tamil sandhi (புணர்ச்சி) resolution for prosodic analysis.
///
/// Two main transformations:
/// 1. **Pluti vowel collapsing** — Classical Tamil poetry extends vowels
///    by appending the corresponding kuril vowel letter after a nedil matra.
///    E.g., படாஅர் (pa-ṭā-a-r) is metrically படார் (pa-ṭār).
/// 2. **Compound boundary detection** — When a vowel matra is followed by
///    a vowel letter that is NOT its pluti pair, this indicates a morpheme
///    boundary within a compound word.
///
/// Pluti vowel pairs: (nedil_matra, kuril_vowel)
/// When a nedil matra is immediately followed by its corresponding kuril vowel letter,
/// the vowel is a pluti (extended) form and should be collapsed for metrical analysis.
const PLUTI_PAIRS: [(char, char); 5] = [
    ('\u{0BBE}', '\u{0B85}'), // ா + அ
    ('\u{0BC2}', '\u{0B89}'), // ூ + உ
    ('\u{0BC0}', '\u{0B87}'), // ீ + இ
    ('\u{0BC7}', '\u{0B8E}'), // ே + எ
    ('\u{0BCB}', '\u{0B92}'), // ோ + ஒ
];

/// Result of sandhi resolution for a single word.
#[derive(Debug, Clone)]
pub struct SandhiResult {
    /// The phonological (metrically normalized) text.
    pub phonological_text: String,
    /// Whether any pluti vowels were collapsed.
    pub pluti_resolved: bool,
    /// Number of pluti vowels found and collapsed.
    pub pluti_count: usize,
    /// Whether a compound morpheme boundary was detected.
    pub has_compound_boundary: bool,
}

/// Resolve sandhi in a single word for prosodic analysis.
///
/// Currently handles:
/// - Pluti vowel collapsing (ாஅ→ா, ூஉ→ூ, ீஇ→ீ, ேஎ→ே, ோஒ→ோ)
/// - Compound boundary detection (matra + non-pluti vowel)
pub fn resolve(text: &str) -> SandhiResult {
    let chars: Vec<char> = text.chars().collect();
    let mut result = String::with_capacity(text.len());
    let mut pluti_count = 0;
    let mut has_compound_boundary = false;
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        if i + 1 < chars.len() && is_nedil_matra(c) {
            let next = chars[i + 1];

            // Check if this is a pluti pair
            if is_pluti_pair(c, next) {
                // Collapse: keep the matra, skip the vowel
                result.push(c);
                pluti_count += 1;
                i += 2;
                continue;
            }

            // Check if this is a compound boundary (matra + standalone vowel)
            if super::unicode::is_vowel(next) {
                has_compound_boundary = true;
            }
        }

        result.push(c);
        i += 1;
    }

    SandhiResult {
        phonological_text: result,
        pluti_resolved: pluti_count > 0,
        pluti_count,
        has_compound_boundary,
    }
}

fn is_nedil_matra(c: char) -> bool {
    matches!(
        c,
        '\u{0BBE}' // ா
        | '\u{0BC0}' // ீ
        | '\u{0BC2}' // ூ
        | '\u{0BC7}' // ே
        | '\u{0BC8}' // ை
        | '\u{0BCB}' // ோ
        | '\u{0BCC}' // ௌ
    )
}

fn is_pluti_pair(matra: char, vowel: char) -> bool {
    PLUTI_PAIRS.iter().any(|(m, v)| *m == matra && *v == vowel)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pluti_aa() {
        // படாஅர் → படார்
        let r = resolve("படாஅர்");
        assert_eq!(r.phonological_text, "படார்");
        assert!(r.pluti_resolved);
        assert_eq!(r.pluti_count, 1);
    }

    #[test]
    fn test_pluti_uu() {
        // தூஉம் → தூம்
        let r = resolve("தூஉம்");
        assert_eq!(r.phonological_text, "தூம்");
        assert!(r.pluti_resolved);
        assert_eq!(r.pluti_count, 1);
    }

    #[test]
    fn test_pluti_ii() {
        // ஒரீஇ → ஒரீ
        let r = resolve("ஒரீஇ");
        assert_eq!(r.phonological_text, "ஒரீ");
        assert!(r.pluti_resolved);
    }

    #[test]
    fn test_pluti_ee() {
        // resolve ே + எ
        let r = resolve("சேஎய்");
        assert_eq!(r.phonological_text, "சேய்");
        assert!(r.pluti_resolved);
    }

    #[test]
    fn test_pluti_oo() {
        // போஒம் → போம்
        let r = resolve("போஒம்");
        assert_eq!(r.phonological_text, "போம்");
        assert!(r.pluti_resolved);
    }

    #[test]
    fn test_no_pluti() {
        let r = resolve("அகர");
        assert_eq!(r.phonological_text, "அகர");
        assert!(!r.pluti_resolved);
        assert_eq!(r.pluti_count, 0);
    }

    #[test]
    fn test_compound_boundary_detected() {
        // சுவைஒளி: ை + ஒ is NOT pluti → compound boundary
        let r = resolve("சுவைஒளி");
        assert_eq!(r.phonological_text, "சுவைஒளி"); // not modified
        assert!(!r.pluti_resolved);
        assert!(r.has_compound_boundary);
    }

    #[test]
    fn test_multiple_pluti() {
        // உள்ளதூஉம் → உள்ளதூம்
        let r = resolve("உள்ளதூஉம்");
        assert_eq!(r.phonological_text, "உள்ளதூம்");
        assert_eq!(r.pluti_count, 1);
    }

    #[test]
    fn test_thozhaar_pluti() {
        // தொழாஅர் → தொழார்
        let r = resolve("தொழாஅர்");
        assert_eq!(r.phonological_text, "தொழார்");
        assert!(r.pluti_resolved);
    }

    #[test]
    fn test_geminated_consonants_not_compound_boundary() {
        // Doubled consonants are normal Tamil gemination, NOT compound boundaries
        // முதற்றே has ற்ற — just gemination
        let r = resolve("முதற்றே");
        assert!(!r.has_compound_boundary);

        // எல்லாம் has ல்ல — just gemination
        let r2 = resolve("எல்லாம்");
        assert!(!r2.has_compound_boundary);

        // அகர — no boundary
        let r3 = resolve("அகர");
        assert!(!r3.has_compound_boundary);
    }

    #[test]
    fn test_padaar_pluti() {
        // படாஅர் → படார் (verifies correct syllable count change)
        let r = resolve("படாஅர்");
        assert_eq!(r.phonological_text, "படார்");

        // Verify downstream: the phonological form should have 2 syllables not 3
        use crate::tamil::grapheme::extract_graphemes;
        use crate::tamil::prosody::classify_asai;
        use crate::tamil::syllable::syllabify;

        let gs_before = extract_graphemes("படாஅர்");
        let syls_before = syllabify(&gs_before);
        assert_eq!(syls_before.len(), 3); // ப, டா, அர்

        let gs_after = extract_graphemes("படார்");
        let syls_after = syllabify(&gs_after);
        assert_eq!(syls_after.len(), 2); // ப, டார்

        let asai_before = classify_asai(&syls_before);
        let asai_after = classify_asai(&syls_after);
        // படாஅர் syllables: [ப(kuril,open), டா(nedil,open), அர்(kuril,closed)]
        // kuril+nedil = Nirai: [Nirai("படா"), Neer("அர்")] = 2 asais
        assert_eq!(asai_before.len(), 2); // 2 asais without pluti resolution
                                          // படார் syllables: [ப(kuril,open), டார்(nedil,closed)]
                                          // kuril+nedil = Nirai: [Nirai("படார்")] = 1 asai
        assert_eq!(asai_after.len(), 1); // 1 asai with pluti resolution
    }
}
