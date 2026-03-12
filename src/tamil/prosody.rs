use std::fmt;

use serde::{Deserialize, Serialize};

use super::syllable::TamilSyllable;
use super::unicode::VowelLength;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AsaiType {
    Neer,
    Nirai,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asai {
    pub vagai: AsaiType,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeerType {
    // Iyarseer (2-asai) — eerasai vaaipadu
    Thema,     // Neer + Neer    (was NeerSeer)
    Pulima,    // Nirai + Neer   (was NiraiSeer)
    Koovilam,  // Neer + Nirai   (was KaniSeer)
    Karuvilam, // Nirai + Nirai  (was MalarSeer)
    // Venseer (3-asai) — moovasai vaaipadu
    Themangai,     // Neer + Neer + Neer
    Themangani,    // Neer + Neer + Nirai
    Koovilankai,   // Neer + Nirai + Neer
    Koovilankani,  // Neer + Nirai + Nirai
    Pulimangai,    // Nirai + Neer + Neer
    Pulimangani,   // Nirai + Neer + Nirai
    Karuvilangai,  // Nirai + Nirai + Neer
    Karuvilankani, // Nirai + Nirai + Nirai
    // Invalid (4+ asais)
    Overflow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeerCategory {
    Iyarseer, // 1 or 2 asais
    Venseer,  // 3 asais
    Overflow, // 4+ asais
}

impl AsaiType {
    pub fn as_str(self) -> &'static str {
        match self {
            AsaiType::Neer => "neer",
            AsaiType::Nirai => "nirai",
        }
    }
}

impl fmt::Display for AsaiType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AsaiType::Neer => write!(f, "நேர்"),
            AsaiType::Nirai => write!(f, "நிரை"),
        }
    }
}

impl fmt::Display for SeerType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SeerType::Thema => write!(f, "தேமா"),
            SeerType::Pulima => write!(f, "புளிமா"),
            SeerType::Koovilam => write!(f, "கூவிளம்"),
            SeerType::Karuvilam => write!(f, "கருவிளம்"),
            SeerType::Themangai => write!(f, "தேமாங்காய்"),
            SeerType::Themangani => write!(f, "தேமாங்கனி"),
            SeerType::Koovilankai => write!(f, "கூவிளங்காய்"),
            SeerType::Koovilankani => write!(f, "கூவிளங்கனி"),
            SeerType::Pulimangai => write!(f, "புளிமாங்காய்"),
            SeerType::Pulimangani => write!(f, "புளிமாங்கனி"),
            SeerType::Karuvilangai => write!(f, "கருவிளங்காய்"),
            SeerType::Karuvilankani => write!(f, "கருவிளங்கனி"),
            SeerType::Overflow => write!(f, "overflow"),
        }
    }
}

impl fmt::Display for SeerCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SeerCategory::Iyarseer => write!(f, "இயற்சீர்"),
            SeerCategory::Venseer => write!(f, "வெண்சீர்"),
            SeerCategory::Overflow => write!(f, "overflow"),
        }
    }
}

pub struct SeerData {
    pub seer_vagai: SeerType,
    pub seer_category: SeerCategory,
    pub asai_count: usize,
    pub asai_amaivu: String,
    pub seer_muthal: AsaiType,
    pub seer_eerru: AsaiType,
}

pub fn classify_asai(syllables: &[TamilSyllable]) -> Vec<Asai> {
    classify_asai_with_boundaries(syllables, &[])
}

/// Classify syllables into asai with morpheme boundary awareness.
/// When a morpheme boundary falls between two kuril-open syllables,
/// the first kuril is emitted as standalone Neer instead of combining into Nirai.
pub fn classify_asai_with_boundaries(
    syllables: &[TamilSyllable],
    morpheme_boundaries: &[usize],
) -> Vec<Asai> {
    let mut asaikal = Vec::new();
    let mut i = 0;

    while i < syllables.len() {
        let syl = &syllables[i];

        if syl.alavu == VowelLength::Nedil {
            // Rule 1: Nedil (open or closed) -> Neer
            asaikal.push(Asai {
                vagai: AsaiType::Neer,
                text: syl.text.clone(),
            });
            i += 1;
        } else if syl.is_closed {
            // Rule 2: Kuril closed -> Neer
            asaikal.push(Asai {
                vagai: AsaiType::Neer,
                text: syl.text.clone(),
            });
            i += 1;
        } else {
            // Kuril open — may combine with next syllable to form Nirai
            // Per Yapparungalak Karigai: kuril + kuril = Nirai, kuril + nedil = Nirai
            // Covers R-ASAI-08..R-ASAI-11 (Table 2 rows 5-8)
            if i + 1 < syllables.len() {
                // Check if morpheme boundary prevents nirai grouping
                if morpheme_boundaries.contains(&(i + 1)) {
                    // Morpheme boundary: emit first kuril as standalone Neer
                    asaikal.push(Asai {
                        vagai: AsaiType::Neer,
                        text: syl.text.clone(),
                    });
                    i += 1;
                } else {
                    // Rule 3: Kuril open + next syllable (kuril or nedil) -> Nirai
                    let combined = format!("{}{}", syl.text, syllables[i + 1].text);
                    asaikal.push(Asai {
                        vagai: AsaiType::Nirai,
                        text: combined,
                    });
                    i += 2;
                }
            } else {
                // Word-final kuril open -> standalone Neer
                asaikal.push(Asai {
                    vagai: AsaiType::Neer,
                    text: syl.text.clone(),
                });
                i += 1;
            }
        }
    }

    asaikal
}

pub fn classify_seer(asaikal: &[Asai]) -> SeerData {
    let asai_amaivu = asaikal
        .iter()
        .map(|a| a.vagai.as_str())
        .collect::<Vec<_>>()
        .join("_");

    let seer_muthal = asaikal.first().map(|a| a.vagai).unwrap_or(AsaiType::Neer);
    let seer_eerru = asaikal.last().map(|a| a.vagai).unwrap_or(AsaiType::Neer);

    let asai_count = asaikal.len();

    let (seer_vagai, seer_category) = match asai_count {
        // Single asai: treated as iyarseer (subset of 2-asai pattern)
        1 => match asaikal[0].vagai {
            AsaiType::Neer => (SeerType::Thema, SeerCategory::Iyarseer),
            AsaiType::Nirai => (SeerType::Pulima, SeerCategory::Iyarseer),
        },
        // Iyarseer (eerasai) — 4 vaaipadu
        2 => {
            let seer = match (asaikal[0].vagai, asaikal[1].vagai) {
                (AsaiType::Neer, AsaiType::Neer) => SeerType::Thema,
                (AsaiType::Nirai, AsaiType::Neer) => SeerType::Pulima,
                (AsaiType::Neer, AsaiType::Nirai) => SeerType::Koovilam,
                (AsaiType::Nirai, AsaiType::Nirai) => SeerType::Karuvilam,
            };
            (seer, SeerCategory::Iyarseer)
        }
        // Venseer (moovasai) — 8 vaaipadu
        3 => {
            let seer = match (asaikal[0].vagai, asaikal[1].vagai, asaikal[2].vagai) {
                (AsaiType::Neer, AsaiType::Neer, AsaiType::Neer) => SeerType::Themangai,
                (AsaiType::Neer, AsaiType::Neer, AsaiType::Nirai) => SeerType::Themangani,
                (AsaiType::Neer, AsaiType::Nirai, AsaiType::Neer) => SeerType::Koovilankai,
                (AsaiType::Neer, AsaiType::Nirai, AsaiType::Nirai) => SeerType::Koovilankani,
                (AsaiType::Nirai, AsaiType::Neer, AsaiType::Neer) => SeerType::Pulimangai,
                (AsaiType::Nirai, AsaiType::Neer, AsaiType::Nirai) => SeerType::Pulimangani,
                (AsaiType::Nirai, AsaiType::Nirai, AsaiType::Neer) => SeerType::Karuvilangai,
                (AsaiType::Nirai, AsaiType::Nirai, AsaiType::Nirai) => SeerType::Karuvilankani,
            };
            (seer, SeerCategory::Venseer)
        }
        // 4+ asais — overflow (invalid in Kural Venba)
        _ => (SeerType::Overflow, SeerCategory::Overflow),
    };

    SeerData {
        seer_vagai,
        seer_category,
        asai_count,
        asai_amaivu,
        seer_muthal,
        seer_eerru,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tamil::grapheme::extract_graphemes;
    use crate::tamil::syllable::syllabify;

    fn asai_for(word: &str) -> Vec<Asai> {
        let gs = extract_graphemes(word);
        let syls = syllabify(&gs);
        classify_asai(&syls)
    }

    fn seer_for(word: &str) -> SeerData {
        let asaikal = asai_for(word);
        classify_seer(&asaikal)
    }

    #[test]
    fn test_akar_asai() {
        // அகர -> [Nirai("அக"), Neer("ர")]
        let asaikal = asai_for("அகர");
        assert_eq!(asaikal.len(), 2);
        assert_eq!(asaikal[0].vagai, AsaiType::Nirai);
        assert_eq!(asaikal[0].text, "அக");
        assert_eq!(asaikal[1].vagai, AsaiType::Neer);
        assert_eq!(asaikal[1].text, "ர");
    }

    #[test]
    fn test_aathi_asai() {
        // ஆதி -> [Neer("ஆ"), Neer("தி")]
        let asaikal = asai_for("ஆதி");
        assert_eq!(asaikal.len(), 2);
        assert_eq!(asaikal[0].vagai, AsaiType::Neer);
        assert_eq!(asaikal[1].vagai, AsaiType::Neer);
    }

    #[test]
    fn test_pakavan_asai() {
        // பகவன் -> [Nirai("பக"), Neer("வன்")]
        let asaikal = asai_for("பகவன்");
        assert_eq!(asaikal.len(), 2);
        assert_eq!(asaikal[0].vagai, AsaiType::Nirai);
        assert_eq!(asaikal[0].text, "பக");
        assert_eq!(asaikal[1].vagai, AsaiType::Neer);
        assert_eq!(asaikal[1].text, "வன்");
    }

    #[test]
    fn test_ezhuthellaam_asai() {
        // எழுத்தெல்லாம் -> [Nirai("எழுத்"), Neer("தெல்"), Neer("லாம்")]
        let asaikal = asai_for("எழுத்தெல்லாம்");
        assert_eq!(asaikal.len(), 3);
        assert_eq!(asaikal[0].vagai, AsaiType::Nirai);
        assert_eq!(asaikal[0].text, "எழுத்");
        assert_eq!(asaikal[1].vagai, AsaiType::Neer);
        assert_eq!(asaikal[1].text, "தெல்");
        assert_eq!(asaikal[2].vagai, AsaiType::Neer);
        assert_eq!(asaikal[2].text, "லாம்");
    }

    #[test]
    fn test_ulaku_asai() {
        // உலகு -> [Nirai("உல"), Neer("கு")]
        let asaikal = asai_for("உலகு");
        assert_eq!(asaikal.len(), 2);
        assert_eq!(asaikal[0].vagai, AsaiType::Nirai);
        assert_eq!(asaikal[0].text, "உல");
        assert_eq!(asaikal[1].vagai, AsaiType::Neer);
        assert_eq!(asaikal[1].text, "கு");
    }

    #[test]
    fn test_muthatree_asai() {
        // முதற்றே -> [Nirai("முதற்"), Neer("றே")]
        let asaikal = asai_for("முதற்றே");
        assert_eq!(asaikal.len(), 2);
        assert_eq!(asaikal[0].vagai, AsaiType::Nirai);
        assert_eq!(asaikal[0].text, "முதற்");
        assert_eq!(asaikal[1].vagai, AsaiType::Neer);
        assert_eq!(asaikal[1].text, "றே");
    }

    #[test]
    fn test_seer_thema() {
        // ஆதி -> neer + neer -> Thema (iyarseer)
        let sd = seer_for("ஆதி");
        assert_eq!(sd.seer_vagai, SeerType::Thema);
        assert_eq!(sd.seer_category, SeerCategory::Iyarseer);
        assert_eq!(sd.asai_count, 2);
        assert_eq!(sd.asai_amaivu, "neer_neer");
        assert_eq!(sd.seer_muthal, AsaiType::Neer);
        assert_eq!(sd.seer_eerru, AsaiType::Neer);
    }

    #[test]
    fn test_seer_pulima() {
        // அகர -> nirai + neer -> Pulima (iyarseer)
        let sd = seer_for("அகர");
        assert_eq!(sd.seer_vagai, SeerType::Pulima);
        assert_eq!(sd.seer_category, SeerCategory::Iyarseer);
        assert_eq!(sd.asai_count, 2);
        assert_eq!(sd.asai_amaivu, "nirai_neer");
        assert_eq!(sd.seer_muthal, AsaiType::Nirai);
        assert_eq!(sd.seer_eerru, AsaiType::Neer);
    }

    #[test]
    fn test_seer_pulimangai() {
        // எழுத்தெல்லாம் -> nirai + neer + neer -> Pulimangai (venseer)
        let sd = seer_for("எழுத்தெல்லாம்");
        assert_eq!(sd.seer_vagai, SeerType::Pulimangai);
        assert_eq!(sd.seer_category, SeerCategory::Venseer);
        assert_eq!(sd.asai_count, 3);
        assert_eq!(sd.asai_amaivu, "nirai_neer_neer");
        assert_eq!(sd.seer_muthal, AsaiType::Nirai);
        assert_eq!(sd.seer_eerru, AsaiType::Neer);
    }

    #[test]
    fn test_kuril_open_before_nedil_nirai() {
        // தலை -> syllables [த(kuril,open), லை(nedil,open)]
        // Rule 3: kuril open + nedil -> Nirai (per Table 2 row 6: Kuril+Nedil = Nirai)
        let asaikal = asai_for("தலை");
        assert_eq!(asaikal.len(), 1);
        assert_eq!(asaikal[0].vagai, AsaiType::Nirai);
        assert_eq!(asaikal[0].text, "தலை");
    }

    #[test]
    fn test_thalai_seer_pulima() {
        // தலை -> [Nirai] -> single-asai Pulima (iyarseer)
        let sd = seer_for("தலை");
        assert_eq!(sd.seer_vagai, SeerType::Pulima);
        assert_eq!(sd.seer_category, SeerCategory::Iyarseer);
        assert_eq!(sd.asai_amaivu, "nirai");
        assert_eq!(sd.seer_eerru, AsaiType::Nirai);
    }

    #[test]
    fn test_boundary_aware_asai_prevents_nirai() {
        // அகர without boundary: [Nirai("அக"), Neer("ர")] = Pulima
        // அகர with boundary at syllable 1: [Neer("அ"), Nirai("கர")]
        // Boundary prevents 0+1 grouping, but 1+2 still group into Nirai
        let gs = extract_graphemes("அகர");
        let syls = syllabify(&gs);

        // Without boundary: normal nirai grouping
        let asai_normal = classify_asai(&syls);
        assert_eq!(asai_normal.len(), 2);
        assert_eq!(asai_normal[0].vagai, AsaiType::Nirai);

        // With boundary at syllable index 1: prevents first nirai grouping
        let asai_boundary = classify_asai_with_boundaries(&syls, &[1]);
        assert_eq!(asai_boundary.len(), 2);
        assert_eq!(asai_boundary[0].vagai, AsaiType::Neer);
        assert_eq!(asai_boundary[0].text, "அ");
        assert_eq!(asai_boundary[1].vagai, AsaiType::Nirai);
        assert_eq!(asai_boundary[1].text, "கர");
    }

    #[test]
    fn test_boundary_aware_without_boundary_unchanged() {
        // Empty boundaries should produce identical results to classify_asai
        let gs = extract_graphemes("பகவன்");
        let syls = syllabify(&gs);
        let normal = classify_asai(&syls);
        let with_empty = classify_asai_with_boundaries(&syls, &[]);
        assert_eq!(normal.len(), with_empty.len());
        for (a, b) in normal.iter().zip(with_empty.iter()) {
            assert_eq!(a.vagai, b.vagai);
            assert_eq!(a.text, b.text);
        }
    }
}
