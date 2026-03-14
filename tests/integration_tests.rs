use tamil_yaappu_analyzer::preprocessor::preprocess;

const KURAL_JSON: &str = include_str!("../kural.json");

// === Structure Tests ===

#[test]
fn test_kural_1_structure() {
    let input = "அகர முதல எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு";
    let paa = preprocess(input);

    assert_eq!(paa.raw, input);
    assert_eq!(paa.adikal.len(), 2);
    assert_eq!(paa.adikal[0].sorkal.len(), 4);
    assert_eq!(paa.adikal[1].sorkal.len(), 3);
    assert_eq!(paa.adikal[0].raw, "அகர முதல எழுத்தெல்லாம் ஆதி");
    assert_eq!(paa.adikal[1].raw, "பகவன் முதற்றே உலகு");
}

#[test]
fn test_single_line_input() {
    let input = "அகர முதல எழுத்தெல்லாம் ஆதி பகவன் முதற்றே உலகு";
    let paa = preprocess(input);

    assert_eq!(paa.adikal.len(), 1);
    assert_eq!(paa.adikal[0].sorkal.len(), 7);
}

#[test]
fn test_few_words() {
    let input = "அகர முதல\nபகவன்";
    let paa = preprocess(input);

    assert_eq!(paa.adikal.len(), 2);
    assert_eq!(paa.adikal[0].sorkal.len(), 2);
    assert_eq!(paa.adikal[1].sorkal.len(), 1);
}

#[test]
fn test_punctuation_tokens_skipped() {
    let input = "அகர - முதல\nபகவன்";
    let paa = preprocess(input);

    // Standalone "-" should be skipped
    assert_eq!(paa.adikal[0].sorkal.len(), 2);
    assert_eq!(paa.adikal[0].sorkal[0].raw, "அகர");
    assert_eq!(paa.adikal[0].sorkal[1].raw, "முதல");
}

// === Raw Text Tests ===

#[test]
fn test_word_raw_text() {
    let input = "அகர முதல எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு";
    let paa = preprocess(input);

    assert_eq!(paa.adikal[0].sorkal[0].raw, "அகர");
    assert_eq!(paa.adikal[0].sorkal[1].raw, "முதல");
    assert_eq!(paa.adikal[0].sorkal[2].raw, "எழுத்தெல்லாம்");
    assert_eq!(paa.adikal[0].sorkal[3].raw, "ஆதி");
    assert_eq!(paa.adikal[1].sorkal[0].raw, "பகவன்");
    assert_eq!(paa.adikal[1].sorkal[1].raw, "முதற்றே");
    assert_eq!(paa.adikal[1].sorkal[2].raw, "உலகு");
}

// === Asai Sequence Tests ===

#[test]
fn test_asai_seq_kural_1() {
    let input = "அகர முதல எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு";
    let paa = preprocess(input);

    // அகர: nirai neer (pulima)
    assert_eq!(paa.adikal[0].sorkal[0].asai_seq, vec!["nirai", "neer"]);
    // முதல: nirai neer (pulima)
    assert_eq!(paa.adikal[0].sorkal[1].asai_seq, vec!["nirai", "neer"]);
    // எழுத்தெல்லாம்: nirai neer neer (pulimangai - 3 asai)
    assert_eq!(
        paa.adikal[0].sorkal[2].asai_seq,
        vec!["nirai", "neer", "neer"]
    );
    // ஆதி: neer neer (thema)
    assert_eq!(paa.adikal[0].sorkal[3].asai_seq, vec!["neer", "neer"]);
    // பகவன்: nirai neer (pulima)
    assert_eq!(paa.adikal[1].sorkal[0].asai_seq, vec!["nirai", "neer"]);
    // முதற்றே: nirai neer (pulima)
    assert_eq!(paa.adikal[1].sorkal[1].asai_seq, vec!["nirai", "neer"]);
    // உலகு: nirai neer (pulima)
    assert_eq!(paa.adikal[1].sorkal[2].asai_seq, vec!["nirai", "neer"]);
}

#[test]
fn test_asai_seq_extended_seer() {
    // Word with 3+ asai (overflow/venseer)
    let input = "எழுத்தெல்லாம்\nஉலகு";
    let paa = preprocess(input);
    assert_eq!(paa.adikal[0].sorkal[0].asai_seq.len(), 3);
}

// === Comparison Key Tests ===

#[test]
fn test_muthal_ezhuthu_consonant_words() {
    let input = "அகர முதல எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு";
    let paa = preprocess(input);

    // அகர starts with uyir அ -> monai group "அ"
    assert_eq!(
        paa.adikal[0].sorkal[0].muthal_ezhuthu,
        Some("அ".to_string())
    );
    // முதல starts with uyirmei மு -> mei "ம"
    assert_eq!(
        paa.adikal[0].sorkal[1].muthal_ezhuthu,
        Some("ம".to_string())
    );
    // எழுத்தெல்லாம் starts with uyir எ -> monai group "எ"
    assert_eq!(
        paa.adikal[0].sorkal[2].muthal_ezhuthu,
        Some("எ".to_string())
    );
    // ஆதி starts with uyir ஆ -> monai group "அ" (same group as அ)
    assert_eq!(
        paa.adikal[0].sorkal[3].muthal_ezhuthu,
        Some("அ".to_string())
    );
    // பகவன் starts with uyirmei ப -> mei "ப"
    assert_eq!(
        paa.adikal[1].sorkal[0].muthal_ezhuthu,
        Some("ப".to_string())
    );
}

#[test]
fn test_irandaam_ezhuthu() {
    let input = "அகர முதல எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு";
    let paa = preprocess(input);

    // அகர: 2nd grapheme is க -> mei "க"
    assert_eq!(
        paa.adikal[0].sorkal[0].irandaam_ezhuthu,
        Some("க".to_string())
    );
    // முதல: 2nd grapheme is த -> mei "த"
    assert_eq!(
        paa.adikal[0].sorkal[1].irandaam_ezhuthu,
        Some("த".to_string())
    );
    // பகவன்: 2nd grapheme is க -> mei "க"
    assert_eq!(
        paa.adikal[1].sorkal[0].irandaam_ezhuthu,
        Some("க".to_string())
    );
}

#[test]
fn test_kadai_ezhuthu() {
    let input = "அகர முதல எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு";
    let paa = preprocess(input);

    assert_eq!(paa.adikal[0].sorkal[0].kadai_ezhuthu, Some("ர".to_string()));
    assert_eq!(paa.adikal[0].sorkal[1].kadai_ezhuthu, Some("ல".to_string()));
    assert_eq!(paa.adikal[0].sorkal[2].kadai_ezhuthu, Some("ம்".to_string()));
    assert_eq!(
        paa.adikal[0].sorkal[3].kadai_ezhuthu,
        Some("தி".to_string())
    );
    assert_eq!(
        paa.adikal[1].sorkal[2].kadai_ezhuthu,
        Some("கு".to_string())
    );
}

#[test]
fn test_kadai_alavu() {
    let input = "அகர முதல எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு";
    let paa = preprocess(input);

    // அகர: last syllable ர -> kuril
    assert_eq!(
        paa.adikal[0].sorkal[0].kadai_alavu,
        Some("kuril".to_string())
    );
    // எழுத்தெல்லாம்: last syllable லாம் -> nedil
    assert_eq!(
        paa.adikal[0].sorkal[2].kadai_alavu,
        Some("nedil".to_string())
    );
    // ஆதி: last syllable தி -> kuril
    assert_eq!(
        paa.adikal[0].sorkal[3].kadai_alavu,
        Some("kuril".to_string())
    );
    // முதற்றே: last syllable றே -> nedil
    assert_eq!(
        paa.adikal[1].sorkal[1].kadai_alavu,
        Some("nedil".to_string())
    );
    // உலகு: last syllable கு -> kuril
    assert_eq!(
        paa.adikal[1].sorkal[2].kadai_alavu,
        Some("kuril".to_string())
    );
}

// === Edge Cases ===

#[test]
fn test_non_tamil_input_empty_asai() {
    let input = "hello முதல\nபகவன்";
    let paa = preprocess(input);

    // "hello" stripped to empty -> empty asai_seq
    assert!(paa.adikal[0].sorkal[0].asai_seq.is_empty());
    assert_eq!(paa.adikal[0].sorkal[0].muthal_ezhuthu, None);
    assert_eq!(paa.adikal[0].sorkal[0].kadai_ezhuthu, None);

    // முதல should still work
    assert_eq!(paa.adikal[0].sorkal[1].asai_seq, vec!["nirai", "neer"]);
}

#[test]
fn test_danda_stripping() {
    let input = "அகர முதல\nஉலகு।";
    let paa = preprocess(input);

    // Danda should be stripped, word still processes correctly
    let last = &paa.adikal[1].sorkal[0];
    assert_eq!(last.raw, "உலகு।");
    assert_eq!(last.asai_seq, vec!["nirai", "neer"]);
    assert_eq!(last.kadai_alavu, Some("kuril".to_string()));
}

#[test]
fn test_single_grapheme_word() {
    // ஆ is a single grapheme — no irandaam_ezhuthu
    let input = "ஆ\nஉலகு";
    let paa = preprocess(input);

    let w = &paa.adikal[0].sorkal[0];
    assert_eq!(w.muthal_ezhuthu, Some("அ".to_string())); // vowel group
    assert_eq!(w.irandaam_ezhuthu, None);
    assert_eq!(w.kadai_ezhuthu, Some("ஆ".to_string()));
    assert_eq!(w.asai_seq, vec!["neer"]);
}

// === Sandhi / Pluti Tests ===

#[test]
fn test_sandhi_pluti_resolution() {
    // நிலாஅ has pluti vowel ஆஅ -> resolved to நிலா for analysis
    let input = "நிலாஅ\nஉலகு";
    let paa = preprocess(input);

    let w = &paa.adikal[0].sorkal[0];
    assert_eq!(w.raw, "நிலாஅ");
    // After pluti resolution, analyzed as நிலா (2 syllables: நி + லா = nirai)
    assert_eq!(w.asai_seq, vec!["nirai"]);
}

#[test]
fn test_sandhi_no_pluti_unchanged() {
    let input = "அகர\nஉலகு";
    let paa = preprocess(input);

    let w = &paa.adikal[0].sorkal[0];
    assert_eq!(w.asai_seq, vec!["nirai", "neer"]);
}

// === Etukai Key Tests (irandaam_ezhuthu comparison) ===

#[test]
fn test_etukai_keys_match_kural_1() {
    let input = "அகர முதல எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு";
    let paa = preprocess(input);

    // For etukai: compare irandaam_ezhuthu of first word of line 0 and line 1
    let line0_first = &paa.adikal[0].sorkal[0];
    let line1_first = &paa.adikal[1].sorkal[0];

    // அகர -> "க", பகவன் -> "க" — match (etukai present)
    assert_eq!(line0_first.irandaam_ezhuthu, line1_first.irandaam_ezhuthu);
}

// === Corpus Test ===

#[test]
fn test_all_kurals_preprocess() {
    let kurals: Vec<String> = serde_json::from_str(KURAL_JSON).unwrap();

    for (i, kural) in kurals.iter().enumerate() {
        let paa = preprocess(kural);

        // Every kural should have 2 lines
        assert_eq!(paa.adikal.len(), 2, "Kural #{} should have 2 lines", i + 1);

        // Every word should have non-empty asai_seq
        for (line_idx, adi) in paa.adikal.iter().enumerate() {
            assert!(
                !adi.sorkal.is_empty(),
                "Kural #{} line {} should have words",
                i + 1,
                line_idx
            );
            for sol in &adi.sorkal {
                assert!(
                    !sol.asai_seq.is_empty(),
                    "Kural #{} word '{}' should have non-empty asai_seq",
                    i + 1,
                    sol.raw
                );
            }
        }
    }
}

#[test]
fn test_corpus_word_counts() {
    let kurals: Vec<String> = serde_json::from_str(KURAL_JSON).unwrap();
    let mut word_count_distribution: std::collections::HashMap<(usize, usize), usize> =
        std::collections::HashMap::new();

    for kural in &kurals {
        let paa = preprocess(kural);
        let line1_words = paa.adikal[0].sorkal.len();
        let line2_words = paa.adikal[1].sorkal.len();
        *word_count_distribution
            .entry((line1_words, line2_words))
            .or_insert(0) += 1;
    }

    // Kural venba: most should be (4, 3)
    let four_three = word_count_distribution.get(&(4, 3)).unwrap_or(&0);
    assert!(
        *four_three > 1200,
        "Expected >1200 kurals with (4,3) word pattern, got {}",
        four_three
    );
}

#[test]
fn test_corpus_asai_seq_last_word() {
    let kurals: Vec<String> = serde_json::from_str(KURAL_JSON).unwrap();
    let mut last_word_asai_counts: std::collections::HashMap<usize, usize> =
        std::collections::HashMap::new();

    for kural in &kurals {
        let paa = preprocess(kural);
        let last_line = paa.adikal.last().unwrap();
        let last_word = last_line.sorkal.last().unwrap();
        *last_word_asai_counts
            .entry(last_word.asai_seq.len())
            .or_insert(0) += 1;
    }

    // Kural venba eetru: last word should have 1 or 2 asai
    let one_asai = last_word_asai_counts.get(&1).unwrap_or(&0);
    let two_asai = last_word_asai_counts.get(&2).unwrap_or(&0);
    assert!(
        one_asai + two_asai > 1300,
        "Expected >1300 kurals with 1-2 asai final word, got {}",
        one_asai + two_asai
    );
}
