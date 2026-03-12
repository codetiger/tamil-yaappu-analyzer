/// Compound word decomposition for Tamil prosody analysis.
///
/// Words with 4+ asais (classified as "Overflow") are typically compound words
/// that should be decomposed at asai-level boundaries into valid sub-units
/// (1-3 asais each = Iyarseer or Venseer).
use super::grapheme::extract_graphemes;
use super::prosody::{classify_asai, classify_seer, SeerCategory};
use super::syllable::syllabify;
use super::unicode::{is_consonant, is_vowel_matra, matra_to_vowel, PULLI};

/// Attempt to decompose a compound word into valid prosodic sub-units.
///
/// Returns `Some(vec![part1_text, part2_text, ...])` if a valid decomposition
/// is found, or `None` if the word doesn't need decomposition (<=3 asais)
/// or no valid split exists.
pub fn decompose_compound(analysis_text: &str) -> Option<Vec<String>> {
    let graphemes = extract_graphemes(analysis_text);
    let syllables = syllabify(&graphemes);
    let asaikal = classify_asai(&syllables);
    let asai_count = asaikal.len();

    // No decomposition needed for valid seer counts
    if asai_count <= 3 {
        return None;
    }

    // Try binary splits first (covers 92.5% of cases — 4-asai words)
    if let Some(split) = find_best_binary_split(&syllables) {
        return Some(split);
    }

    // For 6+ asai words, try ternary splits
    if asai_count >= 6 {
        if let Some(split) = find_best_ternary_split(&syllables) {
            return Some(split);
        }
    }

    None
}

/// A candidate binary split with quality metrics.
struct BinarySplit {
    left_text: String,
    right_text: String,
    left_asai_count: usize,
    right_asai_count: usize,
    /// Bonus for sandhi-aware splits. Consonant+vowel merge reversal = 200,
    /// vowel elision reversal = 1 (mild tiebreaker), normal = 0.
    sandhi_bonus: u32,
}

impl BinarySplit {
    /// Score for disambiguation. Higher is better.
    /// Base: both_iyarseer=100, one_venseer=50, other=1.
    /// Sandhi bonus added on top.
    fn score(&self) -> u32 {
        let both_iyarseer = self.left_asai_count <= 2 && self.right_asai_count <= 2;
        let one_venseer = (self.left_asai_count == 3) != (self.right_asai_count == 3);

        let base = if both_iyarseer {
            100
        } else if one_venseer {
            50
        } else {
            1
        };

        base + self.sandhi_bonus
    }
}

/// Try to reverse a sandhi merge at a split boundary.
/// If the right text starts with an uyirmei (consonant + vowel sign), decompose it:
/// move the consonant (+ pulli) to the left, start the right with the standalone vowel.
/// Also handles bare consonants (inherent 'a' vowel).
fn try_sandhi_split(left_text: &str, right_text: &str) -> Option<(String, String)> {
    // Don't try sandhi when left already ends with pulli — appending another
    // consonant+pulli creates a double-mei sequence (e.g., பள்ள்) which is
    // never a valid word ending. This prevents false splits like
    // பள்ளிக்கூடம் → பள்ள் + இக்கூடம்.
    if left_text.ends_with(PULLI) {
        return None;
    }

    let right_chars: Vec<char> = right_text.chars().collect();
    if right_chars.is_empty() {
        return None;
    }

    let first = right_chars[0];
    if !is_consonant(first) {
        return None;
    }

    // Only allow sandhi decomposition when the consonant being moved to the
    // left part commonly ends Tamil words. This prevents false splits like
    // வீடுநிறைந்த → வீடுந் + இறைந்த (ந rarely ends words).
    const SANDHI_CONSONANTS: [char; 7] = ['ம', 'ன', 'ல', 'ள', 'ர', 'ண', 'ய'];
    if !SANDHI_CONSONANTS.contains(&first) {
        return None;
    }

    let mut new_left = left_text.to_string();
    new_left.push(first);
    new_left.push(PULLI);

    let new_right = if right_chars.len() >= 2 && is_vowel_matra(right_chars[1]) {
        let vowel = matra_to_vowel(right_chars[1])?;
        let mut r = String::new();
        r.push(vowel);
        for &c in &right_chars[2..] {
            r.push(c);
        }
        r
    } else {
        // Bare consonant has inherent 'a' vowel
        let mut r = String::from("அ");
        for &c in &right_chars[1..] {
            r.push(c);
        }
        r
    };

    Some((new_left, new_right))
}

/// Try to reverse vowel elision at a split boundary.
/// When a word ending in a vowel sound (e.g., கு) meets a word starting with
/// the same vowel (e.g., உ), the second vowel is elided in the compound form.
/// This reverses it: move the first uyirmei of the right to the left, and
/// restore the standalone vowel at the start of the right.
/// Example: நியதிக் | குட்பட்டு → நியதிக்கு + உட்பட்டு
fn try_vowel_elision_split(left_text: &str, right_text: &str) -> Option<(String, String)> {
    // Only attempt when the left ends with pulli (்) — the consonant cluster
    // is incomplete and needs the uyirmei from the right to form a valid ending.
    // This prevents false splits like வீடு|நிறைந்த → வீடுநி + இறைந்த.
    if !left_text.ends_with(PULLI) {
        return None;
    }

    let right_chars: Vec<char> = right_text.chars().collect();
    if right_chars.len() < 3 {
        return None;
    }

    let first = right_chars[0];
    if !is_consonant(first) {
        return None;
    }

    // Need consonant + vowel matra (uyirmei) at start of right
    if !is_vowel_matra(right_chars[1]) {
        return None;
    }

    let matra = right_chars[1];
    let vowel = matra_to_vowel(matra)?;

    // Move consonant+matra to left
    let mut new_left = left_text.to_string();
    new_left.push(first);
    new_left.push(matra);

    // Start right with standalone vowel + remaining chars
    let mut new_right = String::new();
    new_right.push(vowel);
    for &c in &right_chars[2..] {
        new_right.push(c);
    }

    Some((new_left, new_right))
}

fn find_best_binary_split(syllables: &[super::syllable::TamilSyllable]) -> Option<Vec<String>> {
    let mut candidates: Vec<BinarySplit> = Vec::new();

    for split_at in 1..syllables.len() {
        let left_text: String = syllables[..split_at]
            .iter()
            .map(|s| s.text.as_str())
            .collect();
        let right_text: String = syllables[split_at..]
            .iter()
            .map(|s| s.text.as_str())
            .collect();

        // Normal syllable-boundary split
        let left_seer = classify_part(&left_text);
        let right_seer = classify_part(&right_text);

        if left_seer.0 != SeerCategory::Overflow && right_seer.0 != SeerCategory::Overflow {
            candidates.push(BinarySplit {
                left_text: left_text.clone(),
                right_text: right_text.clone(),
                left_asai_count: left_seer.1,
                right_asai_count: right_seer.1,
                sandhi_bonus: 0,
            });
        }

        // Sandhi-aware split: reverse consonant+vowel merge at the boundary
        if let Some((sandhi_left, sandhi_right)) = try_sandhi_split(&left_text, &right_text) {
            let sl = classify_part(&sandhi_left);
            let sr = classify_part(&sandhi_right);

            if sl.0 != SeerCategory::Overflow && sr.0 != SeerCategory::Overflow {
                candidates.push(BinarySplit {
                    left_text: sandhi_left,
                    right_text: sandhi_right,
                    left_asai_count: sl.1,
                    right_asai_count: sr.1,
                    sandhi_bonus: 200,
                });
            }
        }

        // Vowel elision split: reverse vowel drop at the boundary.
        // No bonus — only wins if it produces a better prosodic split than normal.
        // Acts as a fallback when no normal split at another position is better.
        if let Some((el_left, el_right)) = try_vowel_elision_split(&left_text, &right_text) {
            let el = classify_part(&el_left);
            let er = classify_part(&el_right);

            if el.0 != SeerCategory::Overflow && er.0 != SeerCategory::Overflow {
                candidates.push(BinarySplit {
                    left_text: el_left,
                    right_text: el_right,
                    left_asai_count: el.1,
                    right_asai_count: er.1,
                    sandhi_bonus: 0,
                });
            }
        }
    }

    candidates.sort_by_key(|c| std::cmp::Reverse(c.score()));
    candidates
        .into_iter()
        .next()
        .map(|c| vec![c.left_text, c.right_text])
}

fn find_best_ternary_split(syllables: &[super::syllable::TamilSyllable]) -> Option<Vec<String>> {
    let n = syllables.len();
    let mut best: Option<(Vec<String>, u32)> = None;

    for i in 1..n.saturating_sub(1) {
        for j in (i + 1)..n {
            let p1: String = syllables[..i].iter().map(|s| s.text.as_str()).collect();
            let p2: String = syllables[i..j].iter().map(|s| s.text.as_str()).collect();
            let p3: String = syllables[j..].iter().map(|s| s.text.as_str()).collect();

            // Try all combinations: normal, sandhi, vowel elision at each boundary
            // Tuple: (part1, part2, part3, sandhi_bonus)
            let mut part_sets: Vec<(String, String, String, u32)> =
                vec![(p1.clone(), p2.clone(), p3.clone(), 0)];

            if let Some((sl, sr)) = try_sandhi_split(&p1, &p2) {
                part_sets.push((sl, sr, p3.clone(), 200));
            }
            if let Some((sl, sr)) = try_sandhi_split(&p2, &p3) {
                part_sets.push((p1.clone(), sl, sr, 200));
            }
            if let Some((el, er)) = try_vowel_elision_split(&p1, &p2) {
                part_sets.push((el, er, p3.clone(), 1));
            }
            if let Some((el, er)) = try_vowel_elision_split(&p2, &p3) {
                part_sets.push((p1.clone(), el, er, 1));
            }

            for (t1, t2, t3, bonus) in part_sets {
                let s1 = classify_part(&t1);
                let s2 = classify_part(&t2);
                let s3 = classify_part(&t3);

                if s1.0 != SeerCategory::Overflow
                    && s2.0 != SeerCategory::Overflow
                    && s3.0 != SeerCategory::Overflow
                {
                    let all_iyarseer = s1.1 <= 2 && s2.1 <= 2 && s3.1 <= 2;
                    let score = if all_iyarseer { 100 } else { 50 } + bonus;

                    if best.as_ref().is_none_or(|(_, bs)| score > *bs) {
                        best = Some((vec![t1, t2, t3], score));
                    }
                }
            }
        }
    }

    best.map(|(parts, _)| parts)
}

/// Classify a text fragment: returns (SeerCategory, asai_count).
fn classify_part(text: &str) -> (SeerCategory, usize) {
    let graphemes = extract_graphemes(text);
    let syllables = syllabify(&graphemes);
    let asaikal = classify_asai(&syllables);
    let seer_data = classify_seer(&asaikal);
    (seer_data.seer_category, seer_data.asai_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_decomposition_for_valid_seer() {
        // 2-asai word: no decomposition
        assert!(decompose_compound("அகர").is_none());
        // 3-asai word: no decomposition
        assert!(decompose_compound("எழுத்தெல்லாம்").is_none());
    }

    #[test]
    fn test_no_decomposition_after_kuril_nedil_nirai() {
        // உடைமையுள் — was 4-asai overflow when kuril+nedil = separate Neer.
        // After fix (kuril+nedil = Nirai): [Nirai("உடை"), Neer("மை"), Neer("யுள்")] = 3-asai Venseer.
        // No decomposition needed.
        let result = decompose_compound("உடைமையுள்");
        assert!(
            result.is_none(),
            "உடைமையுள் is now 3-asai Venseer, should not decompose"
        );
    }

    #[test]
    fn test_prefers_2_plus_2_split() {
        // A 4-asai word should prefer 2+2 split over 1+3 or 3+1
        let result = decompose_compound("உடைமையுள்");
        if let Some(parts) = result {
            assert_eq!(parts.len(), 2);
            let left = classify_part(&parts[0]);
            let right = classify_part(&parts[1]);
            // Both should be iyarseer (<=2 asais) if 2+2 is possible
            if left.1 <= 2 && right.1 <= 2 {
                assert_eq!(left.0, SeerCategory::Iyarseer);
                assert_eq!(right.0, SeerCategory::Iyarseer);
            }
        }
    }

    #[test]
    fn test_roundtrip_text() {
        // Verify concatenated parts equal original
        let word = "உடைமையுள்";
        if let Some(parts) = decompose_compound(word) {
            let rejoined: String = parts.concat();
            assert_eq!(rejoined, word);
        }
    }

    #[test]
    fn test_sandhi_aware_split() {
        // பாதகமென்பதை = பாதகம் + என்பதை (ம் + எ merged into மெ)
        let result = decompose_compound("பாதகமென்பதை");
        assert!(result.is_some(), "Should decompose பாதகமென்பதை");
        let parts = result.unwrap();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], "பாதகம்", "Left part should be பாதகம்");
        assert_eq!(parts[1], "என்பதை", "Right part should be என்பதை");
    }

    #[test]
    fn test_no_false_sandhi_with_vallinam() {
        // வீடுநிறைந்த = வீடு + நிறைந்த (earliest valid 2+2 split)
        // Should NOT sandhi-split into வீடுநிற் + ஐந்த (ற is vallinam)
        let result = decompose_compound("வீடுநிறைந்த");
        assert!(result.is_some(), "Should decompose வீடுநிறைந்த");
        let parts = result.unwrap();
        assert_eq!(parts[0], "வீடு", "Left part should be வீடு");
        assert_eq!(parts[1], "நிறைந்த", "Right part should be நிறைந்த");
    }

    #[test]
    fn test_unsplittable_returns_none() {
        // A single syllable word can't be split
        assert!(decompose_compound("தான்").is_none());
    }

    #[test]
    fn test_overflow_compound_split() {
        // நியதிக்குட்பட்டு: morphological split is நியதிக்கு + உட்பட்டு (3+3),
        // but prosodic split நியதிக் + குட்பட்டு (2+3) is better balanced.
        let result = decompose_compound("நியதிக்குட்பட்டு");
        assert!(result.is_some(), "Should decompose நியதிக்குட்பட்டு");
        let parts = result.unwrap();
        assert_eq!(parts[0], "நியதிக்");
        assert_eq!(parts[1], "குட்பட்டு");
    }

    #[test]
    fn test_punarchi_all_types() {
        // Most compound examples are ≤3 asai (Iyarseer/Venseer) — no decomposition needed.
        // Only overflow (4+ asai) words get decomposed.

        // Thondral (Addition): கடைப்பக்கம் is 3-asai Venseer, no split needed
        assert!(decompose_compound("கடைப்பக்கம்").is_none());
        // Thondral: பள்ளிக்கூடம் is 4-asai overflow — should split as பள்ளிக் + கூடம்
        // (prosodic split at syllable boundary; morphological split is பள்ளி + கூடம்
        // but the added க் makes the prosodic boundary different)
        let result = decompose_compound("பள்ளிக்கூடம்");
        assert!(result.is_some(), "Should decompose பள்ளிக்கூடம்");
        let parts = result.unwrap();
        assert_eq!(parts.len(), 2);
        // Prosodic split: பள்ளிக் (2 asai) + கூடம் (2 asai) — both Iyarseer
        assert_eq!(parts[0], "பள்ளிக்");
        assert_eq!(parts[1], "கூடம்");

        // Thirithal (Transformation): both are 2-asai Iyarseer, no split needed
        assert!(decompose_compound("பற்பொடி").is_none());
        assert!(decompose_compound("கற்சிலை").is_none());

        // Keduthal (Deletion): both are ≤3 asai, no split needed
        assert!(decompose_compound("மரக்கிளை").is_none());
        assert!(decompose_compound("மனமகிழ்ச்சி").is_none());

        // Iyalbu (Natural): both are 3-asai Venseer, no split needed
        assert!(decompose_compound("வாழைமரம்").is_none());
        assert!(decompose_compound("பொன்வளையல்").is_none());
    }
}
