use serde::{Deserialize, Serialize};

use super::grapheme::{GraphemeType, TamilGrapheme};
use super::unicode::VowelLength;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TamilSyllable {
    pub text: String,
    pub alavu: VowelLength,
    pub is_closed: bool,
    pub matrai: u8,
}

fn compute_matrai(alavu: VowelLength, is_closed: bool) -> u8 {
    match (alavu, is_closed) {
        (VowelLength::Kuril, false) => 1,
        (VowelLength::Kuril, true) => 2,
        (VowelLength::Nedil, false) => 2,
        (VowelLength::Nedil, true) => 3,
    }
}

pub fn syllabify(graphemes: &[TamilGrapheme]) -> Vec<TamilSyllable> {
    let mut syllables = Vec::new();
    let mut i = 0;

    while i < graphemes.len() {
        let g = &graphemes[i];

        match g.vagai {
            GraphemeType::Uyir | GraphemeType::Uyirmei => {
                let nucleus_alavu = g.alavu.unwrap_or(VowelLength::Kuril);
                let mut text = g.text.clone();
                let mut is_closed = false;

                // Look ahead for Mei coda(s)
                let mut j = i + 1;
                while j < graphemes.len() && graphemes[j].vagai == GraphemeType::Mei {
                    // Check what follows this Mei
                    if j + 1 >= graphemes.len() {
                        // End of word: Mei is coda
                        text.push_str(&graphemes[j].text);
                        is_closed = true;
                        j += 1;
                    } else {
                        match graphemes[j + 1].vagai {
                            GraphemeType::Uyir | GraphemeType::Uyirmei => {
                                // Mei followed by vowel-bearing: Mei is onset of next syllable
                                break;
                            }
                            GraphemeType::Mei | GraphemeType::Aytham => {
                                // Mei followed by another Mei: this Mei is coda
                                text.push_str(&graphemes[j].text);
                                is_closed = true;
                                j += 1;
                            }
                        }
                    }
                }

                let matrai = compute_matrai(nucleus_alavu, is_closed);
                syllables.push(TamilSyllable {
                    text,
                    alavu: nucleus_alavu,
                    is_closed,
                    matrai,
                });
                i = j;
            }

            GraphemeType::Mei => {
                // Orphan Mei at start or unexpected position
                // Attach to previous syllable as coda if possible
                if let Some(last) = syllables.last_mut() {
                    last.text.push_str(&g.text);
                    last.is_closed = true;
                    last.matrai = compute_matrai(last.alavu, true);
                } else {
                    // Standalone Mei with no preceding syllable (e.g., word "ம்")
                    syllables.push(TamilSyllable {
                        text: g.text.clone(),
                        alavu: VowelLength::Kuril,
                        is_closed: true,
                        matrai: 1,
                    });
                }
                i += 1;
            }

            GraphemeType::Aytham => {
                syllables.push(TamilSyllable {
                    text: g.text.clone(),
                    alavu: VowelLength::Kuril,
                    is_closed: false,
                    matrai: 1,
                });
                i += 1;
            }
        }
    }

    syllables
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tamil::grapheme::extract_graphemes;

    #[test]
    fn test_single_open_kuril() {
        let gs = extract_graphemes("அ");
        let syls = syllabify(&gs);
        assert_eq!(syls.len(), 1);
        assert_eq!(syls[0].text, "அ");
        assert_eq!(syls[0].alavu, VowelLength::Kuril);
        assert!(!syls[0].is_closed);
        assert_eq!(syls[0].matrai, 1);
    }

    #[test]
    fn test_single_open_nedil() {
        let gs = extract_graphemes("கா");
        let syls = syllabify(&gs);
        assert_eq!(syls.len(), 1);
        assert_eq!(syls[0].text, "கா");
        assert_eq!(syls[0].alavu, VowelLength::Nedil);
        assert!(!syls[0].is_closed);
        assert_eq!(syls[0].matrai, 2);
    }

    #[test]
    fn test_closed_kuril() {
        // கன் -> graphemes [க, ன்] -> syllable கன் (kuril, closed)
        let gs = extract_graphemes("கன்");
        let syls = syllabify(&gs);
        assert_eq!(syls.len(), 1);
        assert_eq!(syls[0].text, "கன்");
        assert_eq!(syls[0].alavu, VowelLength::Kuril);
        assert!(syls[0].is_closed);
        assert_eq!(syls[0].matrai, 2);
    }

    #[test]
    fn test_closed_nedil() {
        // லாம் -> graphemes [லா, ம்] -> syllable லாம் (nedil, closed)
        let gs = extract_graphemes("லாம்");
        let syls = syllabify(&gs);
        assert_eq!(syls.len(), 1);
        assert_eq!(syls[0].text, "லாம்");
        assert_eq!(syls[0].alavu, VowelLength::Nedil);
        assert!(syls[0].is_closed);
        assert_eq!(syls[0].matrai, 3);
    }

    #[test]
    fn test_pakavan() {
        // பகவன் -> syllables [ப(1), க(1), வன்(2)]
        let gs = extract_graphemes("பகவன்");
        let syls = syllabify(&gs);
        assert_eq!(syls.len(), 3);
        assert_eq!(syls[0].text, "ப");
        assert_eq!(syls[0].matrai, 1);
        assert!(!syls[0].is_closed);
        assert_eq!(syls[1].text, "க");
        assert_eq!(syls[1].matrai, 1);
        assert!(!syls[1].is_closed);
        assert_eq!(syls[2].text, "வன்");
        assert_eq!(syls[2].matrai, 2);
        assert!(syls[2].is_closed);
    }

    #[test]
    fn test_ezhuthellaam() {
        // எழுத்தெல்லாம் -> syllables [எ(1), ழுத்(2), தெல்(2), லாம்(3)]
        let gs = extract_graphemes("எழுத்தெல்லாம்");
        let syls = syllabify(&gs);
        assert_eq!(syls.len(), 4);
        assert_eq!(syls[0].text, "எ");
        assert_eq!(syls[0].matrai, 1);
        assert!(!syls[0].is_closed);
        assert_eq!(syls[1].text, "ழுத்");
        assert_eq!(syls[1].matrai, 2);
        assert!(syls[1].is_closed);
        assert_eq!(syls[2].text, "தெல்");
        assert_eq!(syls[2].matrai, 2);
        assert!(syls[2].is_closed);
        assert_eq!(syls[3].text, "லாம்");
        assert_eq!(syls[3].matrai, 3);
        assert!(syls[3].is_closed);
    }

    #[test]
    fn test_muthatree() {
        // முதற்றே -> syllables [மு(1), தற்(2), றே(2)]
        let gs = extract_graphemes("முதற்றே");
        let syls = syllabify(&gs);
        assert_eq!(syls.len(), 3);
        assert_eq!(syls[0].text, "மு");
        assert_eq!(syls[0].matrai, 1);
        assert!(!syls[0].is_closed);
        assert_eq!(syls[1].text, "தற்");
        assert_eq!(syls[1].matrai, 2);
        assert!(syls[1].is_closed);
        assert_eq!(syls[2].text, "றே");
        assert_eq!(syls[2].alavu, VowelLength::Nedil);
        assert_eq!(syls[2].matrai, 2);
        assert!(!syls[2].is_closed);
    }

    #[test]
    fn test_aathi() {
        // ஆதி -> syllables [ஆ(2), தி(1)]
        let gs = extract_graphemes("ஆதி");
        let syls = syllabify(&gs);
        assert_eq!(syls.len(), 2);
        assert_eq!(syls[0].text, "ஆ");
        assert_eq!(syls[0].alavu, VowelLength::Nedil);
        assert_eq!(syls[0].matrai, 2);
        assert_eq!(syls[1].text, "தி");
        assert_eq!(syls[1].alavu, VowelLength::Kuril);
        assert_eq!(syls[1].matrai, 1);
    }

    #[test]
    fn test_ulaku() {
        // உலகு -> syllables [உ(1), ல(1), கு(1)]
        let gs = extract_graphemes("உலகு");
        let syls = syllabify(&gs);
        assert_eq!(syls.len(), 3);
        assert_eq!(syls[0].text, "உ");
        assert_eq!(syls[0].matrai, 1);
        assert_eq!(syls[1].text, "ல");
        assert_eq!(syls[1].matrai, 1);
        assert_eq!(syls[2].text, "கு");
        assert_eq!(syls[2].matrai, 1);
    }
}
