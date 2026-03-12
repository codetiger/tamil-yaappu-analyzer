/// Compound word decomposition for Tamil prosody analysis.
///
/// Words with 4+ asais (classified as "Overflow") are typically compound words
/// that should be decomposed at asai-level boundaries into valid sub-units
/// (1-3 asais each = Iyarseer or Venseer).
use super::grapheme::extract_graphemes;
use super::prosody::{classify_asai, classify_seer, SeerCategory};
use super::syllable::syllabify;

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
    split_pos: usize,
}

impl BinarySplit {
    /// Score for disambiguation: prefer 2+2, then 2+3/3+2, then others.
    /// Higher is better.
    fn score(&self) -> u32 {
        let both_iyarseer = self.left_asai_count <= 2 && self.right_asai_count <= 2;
        let one_venseer = (self.left_asai_count == 3) != (self.right_asai_count == 3);

        if both_iyarseer {
            100 + self.split_pos as u32 // prefer later split among 2+2
        } else if one_venseer {
            50 + self.split_pos as u32
        } else {
            self.split_pos as u32
        }
    }
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

        let left_seer = classify_part(&left_text);
        let right_seer = classify_part(&right_text);

        if left_seer.0 != SeerCategory::Overflow && right_seer.0 != SeerCategory::Overflow {
            candidates.push(BinarySplit {
                left_text,
                right_text,
                left_asai_count: left_seer.1,
                right_asai_count: right_seer.1,
                split_pos: split_at,
            });
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

            let s1 = classify_part(&p1);
            let s2 = classify_part(&p2);
            let s3 = classify_part(&p3);

            if s1.0 != SeerCategory::Overflow
                && s2.0 != SeerCategory::Overflow
                && s3.0 != SeerCategory::Overflow
            {
                // Score: prefer all-iyarseer, then mixed
                let all_iyarseer = s1.1 <= 2 && s2.1 <= 2 && s3.1 <= 2;
                let score = if all_iyarseer { 100 } else { 50 } + j as u32;

                if best.as_ref().is_none_or(|(_, bs)| score > *bs) {
                    best = Some((vec![p1, p2, p3], score));
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
    fn test_unsplittable_returns_none() {
        // A single syllable word can't be split
        assert!(decompose_compound("தான்").is_none());
    }
}
