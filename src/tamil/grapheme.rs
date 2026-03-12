use serde::{Deserialize, Serialize};

use super::unicode::{
    is_aytham, is_consonant, is_pulli, is_vowel, is_vowel_matra, matra_vowel_length, vowel_length,
    VowelLength,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphemeType {
    Uyir,
    Mei,
    Uyirmei,
    Aytham,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TamilGrapheme {
    pub text: String,
    pub vagai: GraphemeType,
    pub mei: Option<char>,
    pub alavu: Option<VowelLength>,
}

pub fn extract_graphemes(word: &str) -> Vec<TamilGrapheme> {
    let chars: Vec<char> = word.chars().collect();
    let mut graphemes = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        if is_aytham(c) {
            graphemes.push(TamilGrapheme {
                text: c.to_string(),
                vagai: GraphemeType::Aytham,
                mei: None,
                alavu: None,
            });
            i += 1;
        } else if is_vowel(c) {
            graphemes.push(TamilGrapheme {
                text: c.to_string(),
                vagai: GraphemeType::Uyir,
                mei: None,
                alavu: vowel_length(c),
            });
            i += 1;
        } else if is_consonant(c) {
            if i + 1 < chars.len() && is_pulli(chars[i + 1]) {
                // Mei: consonant + pulli
                let text: String = [c, chars[i + 1]].iter().collect();
                graphemes.push(TamilGrapheme {
                    text,
                    vagai: GraphemeType::Mei,
                    mei: Some(c),
                    alavu: None,
                });
                i += 2;
            } else if i + 1 < chars.len() && is_vowel_matra(chars[i + 1]) {
                // UyirMei: consonant + vowel matra
                let matra = chars[i + 1];
                let text: String = [c, matra].iter().collect();
                graphemes.push(TamilGrapheme {
                    text,
                    vagai: GraphemeType::Uyirmei,
                    mei: Some(c),
                    alavu: matra_vowel_length(matra),
                });
                i += 2;
            } else {
                // Bare consonant with inherent 'a' vowel (kuril)
                graphemes.push(TamilGrapheme {
                    text: c.to_string(),
                    vagai: GraphemeType::Uyirmei,
                    mei: Some(c),
                    alavu: Some(VowelLength::Kuril),
                });
                i += 1;
            }
        } else {
            // Non-Tamil character, skip
            i += 1;
        }
    }

    graphemes
}

/// Data about the last grapheme of a word, pre-computed for JSONLogic access
pub struct WordGraphemeData {
    pub kadai_ezhuthu: Option<String>,
    pub kadai_ezhuthu_mei: Option<String>,
    pub kadai_ezhuthu_alavu: Option<VowelLength>,
    pub kadai_ezhuthu_vagai: Option<GraphemeType>,
}

pub fn word_grapheme_data(graphemes: &[TamilGrapheme]) -> WordGraphemeData {
    match graphemes.last() {
        Some(g) => WordGraphemeData {
            kadai_ezhuthu: Some(g.text.clone()),
            kadai_ezhuthu_mei: g.mei.map(|c| c.to_string()),
            kadai_ezhuthu_alavu: g.alavu,
            kadai_ezhuthu_vagai: Some(g.vagai.clone()),
        },
        None => WordGraphemeData {
            kadai_ezhuthu: None,
            kadai_ezhuthu_mei: None,
            kadai_ezhuthu_alavu: None,
            kadai_ezhuthu_vagai: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_vowel() {
        let gs = extract_graphemes("அ");
        assert_eq!(gs.len(), 1);
        assert_eq!(gs[0].vagai, GraphemeType::Uyir);
        assert_eq!(gs[0].alavu, Some(VowelLength::Kuril));
        assert_eq!(gs[0].mei, None);
    }

    #[test]
    fn test_uyirmei_with_matra() {
        let gs = extract_graphemes("கா");
        assert_eq!(gs.len(), 1);
        assert_eq!(gs[0].vagai, GraphemeType::Uyirmei);
        assert_eq!(gs[0].mei, Some('க'));
        assert_eq!(gs[0].alavu, Some(VowelLength::Nedil));
    }

    #[test]
    fn test_mei() {
        let gs = extract_graphemes("க்");
        assert_eq!(gs.len(), 1);
        assert_eq!(gs[0].vagai, GraphemeType::Mei);
        assert_eq!(gs[0].mei, Some('க'));
        assert_eq!(gs[0].alavu, None);
    }

    #[test]
    fn test_bare_consonant() {
        let gs = extract_graphemes("க");
        assert_eq!(gs.len(), 1);
        assert_eq!(gs[0].vagai, GraphemeType::Uyirmei);
        assert_eq!(gs[0].mei, Some('க'));
        assert_eq!(gs[0].alavu, Some(VowelLength::Kuril));
    }

    #[test]
    fn test_aytham() {
        let gs = extract_graphemes("ஃ");
        assert_eq!(gs.len(), 1);
        assert_eq!(gs[0].vagai, GraphemeType::Aytham);
    }

    #[test]
    fn test_word_akar() {
        // அகர -> [அ(uyir), க(uyirmei, inherent a), ர(uyirmei, inherent a)]
        let gs = extract_graphemes("அகர");
        assert_eq!(gs.len(), 3);
        assert_eq!(gs[0].vagai, GraphemeType::Uyir);
        assert_eq!(gs[0].text, "அ");
        assert_eq!(gs[1].vagai, GraphemeType::Uyirmei);
        assert_eq!(gs[1].text, "க");
        assert_eq!(gs[1].alavu, Some(VowelLength::Kuril));
        assert_eq!(gs[2].vagai, GraphemeType::Uyirmei);
        assert_eq!(gs[2].text, "ர");
    }

    #[test]
    fn test_word_ezhuthellaam() {
        // எழுத்தெல்லாம் -> [எ, ழு, த், தெ, ல், லா, ம்]
        let gs = extract_graphemes("எழுத்தெல்லாம்");
        assert_eq!(gs.len(), 7);
        assert_eq!(gs[0].vagai, GraphemeType::Uyir); // எ
        assert_eq!(gs[1].vagai, GraphemeType::Uyirmei); // ழு
        assert_eq!(gs[2].vagai, GraphemeType::Mei); // த்
        assert_eq!(gs[3].vagai, GraphemeType::Uyirmei); // தெ
        assert_eq!(gs[4].vagai, GraphemeType::Mei); // ல்
        assert_eq!(gs[5].vagai, GraphemeType::Uyirmei); // லா
        assert_eq!(gs[5].alavu, Some(VowelLength::Nedil));
        assert_eq!(gs[6].vagai, GraphemeType::Mei); // ம்
    }

    #[test]
    fn test_word_pakavan() {
        // பகவன் -> [ப, க, வ, ன்]
        let gs = extract_graphemes("பகவன்");
        assert_eq!(gs.len(), 4);
        assert_eq!(gs[0].vagai, GraphemeType::Uyirmei); // ப
        assert_eq!(gs[1].vagai, GraphemeType::Uyirmei); // க
        assert_eq!(gs[2].vagai, GraphemeType::Uyirmei); // வ
        assert_eq!(gs[3].vagai, GraphemeType::Mei); // ன்
    }
}
