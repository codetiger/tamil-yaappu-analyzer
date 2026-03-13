use dataflow_rs::engine::message::Message;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tamil_yaappu_analyzer::create_engine;
use tamil_yaappu_analyzer::preprocessor::preprocess;
use tamil_yaappu_analyzer::tamil::prosody::{AsaiType, SeerCategory, SeerType};

const KURAL_JSON: &str = include_str!("../kural.json");

// === Engine helpers ===

async fn run_engine(input: &str) -> Message {
    let engine = create_engine();
    let mut message = Message::new(Arc::new(json!({})));
    message.context["data"]["input"] = json!(input);
    engine.process_message(&mut message).await.unwrap();
    message
}

#[test]
fn test_valid_kural_1_preprocessing() {
    let input = "அகர முதல எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு";
    let paa = preprocess(input);

    // 7 words
    assert_eq!(paa.sorkal.len(), 7);
    // 2 lines
    assert_eq!(paa.adikal.len(), 2);
    // Line 1: 4 words, Line 2: 3 words
    assert_eq!(paa.adikal[0].sol_varisaikal.len(), 4);
    assert_eq!(paa.adikal[1].sol_varisaikal.len(), 3);
    // 6 junctions
    assert_eq!(paa.thalaikal.len(), 6);
    // No diagnostics from preprocessor
    assert!(paa.diagnostics.is_empty());
}

#[test]
fn test_kural_1_word_details() {
    let input = "அகர முதல எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு";
    let paa = preprocess(input);

    // Word 0: அகர -> nirai_seer
    assert_eq!(paa.sorkal[0].raw_text, "அகர");
    assert_eq!(paa.sorkal[0].asai_amaivu, "nirai_neer");
    assert_eq!(paa.sorkal[0].asaikal.len(), 2);

    // Word 3: ஆதி -> neer_seer
    assert_eq!(paa.sorkal[3].raw_text, "ஆதி");
    assert_eq!(paa.sorkal[3].asai_amaivu, "neer_neer");

    // Word 2: எழுத்தெல்லாம் -> extended (3 asai)
    assert_eq!(paa.sorkal[2].raw_text, "எழுத்தெல்லாம்");
    assert_eq!(paa.sorkal[2].asaikal.len(), 3);
    assert_eq!(paa.sorkal[2].asai_amaivu, "nirai_neer_neer");

    // Word 6: உலகு -> nirai_seer
    assert_eq!(paa.sorkal[6].raw_text, "உலகு");
    assert_eq!(paa.sorkal[6].asai_amaivu, "nirai_neer");
}

#[test]
fn test_kural_1_line_assignments() {
    let input = "அகர முதல எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு";
    let paa = preprocess(input);

    // Words 0-3 on line 0
    for i in 0..4 {
        assert_eq!(paa.sorkal[i].adi_index, 0, "Word {} should be on line 0", i);
    }
    // Words 4-6 on line 1
    for i in 4..7 {
        assert_eq!(paa.sorkal[i].adi_index, 1, "Word {} should be on line 1", i);
    }
}

#[test]
fn test_kural_1_junction_cross_adi() {
    let input = "அகர முதல எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு";
    let paa = preprocess(input);

    // Junction 3 (between word 3 and 4) crosses adi boundary
    assert_eq!(paa.thalaikal[3].from_sol_index, 3);
    assert_eq!(paa.thalaikal[3].to_sol_index, 4);
    assert!(paa.thalaikal[3].is_cross_adi);

    // Other junctions don't cross
    assert!(!paa.thalaikal[0].is_cross_adi);
    assert!(!paa.thalaikal[1].is_cross_adi);
    assert!(!paa.thalaikal[2].is_cross_adi);
    assert!(!paa.thalaikal[4].is_cross_adi);
    assert!(!paa.thalaikal[5].is_cross_adi);
}

#[test]
fn test_script_validation_flags() {
    let input = "அகர முதல எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு";
    let paa = preprocess(input);

    for sol in &paa.sorkal {
        assert!(
            sol.is_valid_script,
            "Word '{}' should be valid Tamil",
            sol.raw_text
        );
        assert!(!sol.is_empty);
        assert!(sol.invalid_chars.is_empty());
    }
}

#[test]
fn test_non_tamil_input() {
    let input = "hello முதல எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு";
    let paa = preprocess(input);

    assert!(!paa.sorkal[0].is_valid_script);
    assert!(!paa.sorkal[0].invalid_chars.is_empty());
    // Other words are still valid
    assert!(paa.sorkal[1].is_valid_script);
}

#[test]
fn test_single_line_input() {
    let input = "அகர முதல எழுத்தெல்லாம் ஆதி பகவன் முதற்றே உலகு";
    let paa = preprocess(input);

    assert_eq!(paa.adikal.len(), 1);
    assert_eq!(paa.sorkal.len(), 7);
    // All words on line 0
    for sol in &paa.sorkal {
        assert_eq!(sol.adi_index, 0);
    }
    // No cross-adi junctions
    for thalai in &paa.thalaikal {
        assert!(!thalai.is_cross_adi);
    }
}

#[test]
fn test_few_words() {
    let input = "அகர முதல\nபகவன்";
    let paa = preprocess(input);

    assert_eq!(paa.sorkal.len(), 3);
    assert_eq!(paa.adikal.len(), 2);
    assert_eq!(paa.thalaikal.len(), 2);
}

#[test]
fn test_line_level_aggregation() {
    let input = "அகர முதல எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு";
    let paa = preprocess(input);

    // Line 1 matrai total: from proposal = 17
    assert_eq!(paa.adikal[0].matrai_total, 17);
    // Line 2 matrai total: from proposal = 12
    assert_eq!(paa.adikal[1].matrai_total, 12);
}

#[test]
fn test_grapheme_details_word0() {
    let input = "அகர முதல எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு";
    let paa = preprocess(input);

    // Word 0: அகர -> 3 ezhuthukkal
    let w = &paa.sorkal[0];
    assert_eq!(w.ezhuthukkal.len(), 3);
    assert_eq!(w.ezhuthukkal[0].text, "அ");
    assert_eq!(
        w.ezhuthukkal[0].vagai,
        tamil_yaappu_analyzer::tamil::grapheme::GraphemeType::Uyir
    );
    assert_eq!(w.ezhuthukkal[1].text, "க");
    assert_eq!(w.ezhuthukkal[2].text, "ர");

    // kadai_ezhuthu
    assert_eq!(w.kadai_ezhuthu, Some("ர".to_string()));
}

#[test]
fn test_danda_stripping() {
    let input = "அகர முதல எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு।";
    let paa = preprocess(input);

    let last = &paa.sorkal[6];
    assert!(last.non_tamil_stripped);
    assert_eq!(last.normalized_text, "உலகு।");
    assert_eq!(last.raw_text, "உலகு।");
}

// === L2 Seer Validation Tests ===

#[test]
fn test_l2_all_words_classified_kural1() {
    let input = "அகர முதல எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு";
    let paa = preprocess(input);

    // All words should have a valid seer classification (any type is OK)
    // Kani/Malar are valid in venba — only junction constraints (L3) limit them
    for sol in &paa.sorkal {
        assert!(
            !sol.asaikal.is_empty(),
            "Word '{}' should have asai classification",
            sol.raw_text,
        );
    }
}

#[test]
fn test_l2_extended_seer_detection() {
    let input = "அகர முதல எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு";
    let paa = preprocess(input);

    // Word 2 (எழுத்தெல்லாம்) should be Venseer (3-asai = Pulimangai)
    assert_eq!(paa.sorkal[2].seer_vagai, SeerType::Pulimangai);
    assert_eq!(paa.sorkal[2].seer_category, SeerCategory::Venseer);
    assert_eq!(paa.sorkal[2].asaikal.len(), 3);

    // Other words should be Iyarseer (2-asai)
    for (i, sol) in paa.sorkal.iter().enumerate() {
        if i != 2 {
            assert_eq!(
                sol.seer_category,
                SeerCategory::Iyarseer,
                "Word {} '{}' should be iyarseer",
                i,
                sol.raw_text
            );
        }
    }
}

#[test]
fn test_l2_final_word_ends_neer() {
    let input = "அகர முதல எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு";
    let paa = preprocess(input);

    // Word 6 must end with neer asai
    assert_eq!(paa.sorkal[6].seer_eerru, AsaiType::Neer);
}

#[test]
fn test_l2_seer_types_kural1() {
    let input = "அகர முதல எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு";
    let paa = preprocess(input);

    let expected: [(&str, SeerType); 7] = [
        ("அகர", SeerType::Pulima),
        ("முதல", SeerType::Pulima),
        ("எழுத்தெல்லாம்", SeerType::Pulimangai),
        ("ஆதி", SeerType::Thema),
        ("பகவன்", SeerType::Pulima),
        ("முதற்றே", SeerType::Pulima),
        ("உலகு", SeerType::Pulima),
    ];

    for (i, (word, seer)) in expected.iter().enumerate() {
        assert_eq!(paa.sorkal[i].raw_text, *word);
        assert_eq!(
            paa.sorkal[i].seer_vagai, *seer,
            "Word {} '{}' seer mismatch",
            i, word
        );
    }
}

// === L3 Thalai/Junction Validation Tests ===

#[test]
fn test_l3_no_thalai_breaks_kural1() {
    let input = "அகர முதல எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு";
    let paa = preprocess(input);

    // All junctions in Kural #1 should pass vendalai
    // (no nirai-nirai collision since all valid venba seers end with neer)
    for thalai in &paa.thalaikal {
        let is_nirai_nirai =
            thalai.eerru_asai == AsaiType::Nirai && thalai.muthal_asai == AsaiType::Nirai;
        assert!(
            !is_nirai_nirai,
            "Junction {}->{}: nirai-nirai thalai break",
            thalai.from_sol_index, thalai.to_sol_index
        );
    }
}

#[test]
fn test_l3_all_junctions_neer_ending() {
    let input = "அகர முதல எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு";
    let paa = preprocess(input);

    // All valid venba seers end with neer asai
    for thalai in &paa.thalaikal {
        assert_eq!(
            thalai.eerru_asai,
            AsaiType::Neer,
            "Junction {}->{}: eerru_asai should be neer (valid venba seer ending)",
            thalai.from_sol_index,
            thalai.to_sol_index
        );
    }
}

// === L4 Ornamentation Tests ===

#[test]
fn test_l4_etukai_present_kural1() {
    let input = "அகர முதல எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு";
    let paa = preprocess(input);

    // Etukai: 2nd grapheme consonant base of first word of each line
    // Word[0]="அகர" -> ezhuthukkal[1] = "க" (mei="க")
    // First word line 2 = "பகவன்" -> ezhuthukkal[1] = "க" (mei="க")
    assert!(paa.ani.etukai_present, "Etukai should be present: க == க");
}

#[test]
fn test_l4_monai_absent_kural1() {
    let input = "அகர முதல எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு";
    let paa = preprocess(input);

    // Word[0]="அகர" starts with அ (vowel group அ)
    // Word[2]="எழுத்தெல்லாம்" starts with எ (vowel group எ)
    // Word[1]="முதல" starts with ம (consonant)
    // None match → monai absent
    assert!(
        !paa.ani.monai_present,
        "Monai should be absent for Kural #1"
    );
}

#[test]
fn test_l4_iyaipu_present_kural1() {
    let input = "அகர முதல எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு";
    let paa = preprocess(input);

    // Last syllable line 1 "ஆதி" → "தி" (Kuril)
    // Last syllable line 2 "உலகு" → "கு" (Kuril)
    // Kuril == Kuril → iyaipu present
    assert!(
        paa.ani.iyaipu_present,
        "Iyaipu should be present for Kural #1 (both end with kuril syllable)"
    );
}

#[test]
fn test_l4_etukai_with_consonant_match() {
    // Kural #2: கற்றதனால் ஆய பயனென்கொல் வாலறிவன் / நற்றாள் தொழாஅர் எனின்
    // Word[0]="கற்றதனால்" -> 2nd grapheme = "ற்" (mei="ற")
    // First word line 2 = "நற்றாள்" -> 2nd grapheme = "ற்" (mei="ற")
    let input = "கற்றதனால் ஆய பயனென்கொல் வாலறிவன்\nநற்றாள் தொழாஅர் எனின்";
    let paa = preprocess(input);
    assert!(paa.ani.etukai_present, "Etukai should match: ற == ற");
}

// === L2 Invalid Seer Type Tests ===

#[test]
fn test_l2_koovilam_karuvilam_seer_detection_in_corpus() {
    // Scan corpus for kurals containing Koovilam or Karuvilam seer words
    let kurals: Vec<String> = serde_json::from_str(KURAL_JSON).unwrap();
    let mut koovilam_count = 0;
    let mut karuvilam_count = 0;

    for kural in &kurals {
        let paa = preprocess(kural);
        for sol in &paa.sorkal {
            if sol.seer_vagai == SeerType::Koovilam {
                koovilam_count += 1;
            }
            if sol.seer_vagai == SeerType::Karuvilam {
                karuvilam_count += 1;
            }
        }
    }

    eprintln!("\n=== L2 Koovilam/Karuvilam Detection ===");
    eprintln!("  Koovilam seer words in corpus: {}", koovilam_count);
    eprintln!("  Karuvilam seer words in corpus: {}", karuvilam_count);
    eprintln!("========================================\n");
    // These are valid seers in venba; junction constraints are checked at L3
}

// === Monai Kurippu Tests ===

#[test]
fn test_monai_kurippu_consonant_words() {
    let input = "அகர முதல எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு";
    let paa = preprocess(input);

    // Word 0 (அகர): starts with uyir அ -> monai group "அ"
    assert_eq!(
        paa.sorkal[0].muthal_ezhuthu_monai_kurippu,
        Some("அ".to_string()),
        "Pure vowel அ should have monai group அ"
    );
    // Word 1 (முதல): starts with uyirmei மு -> monai group "ம"
    assert_eq!(
        paa.sorkal[1].muthal_ezhuthu_monai_kurippu,
        Some("ம".to_string()),
        "Consonant-starting word should use mei as monai key"
    );
    // Word 4 (பகவன்): starts with uyirmei ப -> monai group "ப"
    assert_eq!(
        paa.sorkal[4].muthal_ezhuthu_monai_kurippu,
        Some("ப".to_string()),
    );
}

#[test]
fn test_monai_kurippu_vowel_groups() {
    // அ and ஆ should be in the same monai group
    let input = "அகர ஆதி எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு";
    let paa = preprocess(input);

    // Word 0 (அகர): அ -> group "அ"
    // Word 1 (ஆதி): ஆ -> group "அ" (same kuril/nedil pair)
    assert_eq!(
        paa.sorkal[0].muthal_ezhuthu_monai_kurippu, paa.sorkal[1].muthal_ezhuthu_monai_kurippu,
        "அ and ஆ should share the same monai group"
    );
}

#[test]
fn test_monai_kurippu_different_vowel_groups() {
    let input = "அகர முதல எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு";
    let paa = preprocess(input);

    // Word 0 (அகர): அ -> group "அ"
    // Word 2 (எழுத்தெல்லாம்): எ -> group "எ"
    assert_ne!(
        paa.sorkal[0].muthal_ezhuthu_monai_kurippu, paa.sorkal[2].muthal_ezhuthu_monai_kurippu,
        "அ and எ should be in different monai groups"
    );
}

#[test]
fn test_monai_vowel_detection_in_corpus() {
    // Count kurals where monai is detected via vowel matching
    // (cases missed by the old mei-only comparison)
    let kurals: Vec<String> = serde_json::from_str(KURAL_JSON).unwrap();
    let mut vowel_monai_count = 0;

    for kural in &kurals {
        let paa = preprocess(kural);
        if paa.sorkal.len() != 7 {
            continue;
        }

        let mk0 = paa.sorkal[0].muthal_ezhuthu_monai_kurippu.as_deref();
        let mk2 = paa.sorkal[2].muthal_ezhuthu_monai_kurippu.as_deref();
        let mk1 = paa.sorkal[1].muthal_ezhuthu_monai_kurippu.as_deref();

        // Check if monai is via vowel group (both words start with uyir)
        let w0_is_vowel = paa.sorkal[0]
            .ezhuthukkal
            .first()
            .map(|e| e.vagai == tamil_yaappu_analyzer::tamil::grapheme::GraphemeType::Uyir)
            .unwrap_or(false);

        if w0_is_vowel {
            let w2_is_vowel = paa.sorkal[2]
                .ezhuthukkal
                .first()
                .map(|e| e.vagai == tamil_yaappu_analyzer::tamil::grapheme::GraphemeType::Uyir)
                .unwrap_or(false);
            let w1_is_vowel = paa.sorkal[1]
                .ezhuthukkal
                .first()
                .map(|e| e.vagai == tamil_yaappu_analyzer::tamil::grapheme::GraphemeType::Uyir)
                .unwrap_or(false);

            let primary = w2_is_vowel && mk0.is_some() && mk0 == mk2;
            let secondary = w1_is_vowel && mk0.is_some() && mk0 == mk1;

            if primary || secondary {
                vowel_monai_count += 1;
            }
        }
    }

    eprintln!("\n=== Vowel Monai Detection ===");
    eprintln!("  Kurals with vowel-vowel monai: {}", vowel_monai_count);
    eprintln!("  (These were missed by the old mei-only comparison)");
    eprintln!("=============================\n");
    assert!(
        vowel_monai_count > 0,
        "Expected some kurals with vowel-vowel monai"
    );
}

// === L3 Thalai Dependency Tests ===

#[test]
fn test_l3_dependency_non_tamil_word() {
    // Non-Tamil characters produce empty asaikal/asai_amaivu
    let input = "hello முதல எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு";
    let paa = preprocess(input);

    // Word 0 "hello" should have empty asai_amaivu
    assert_eq!(
        paa.sorkal[0].asai_amaivu, "",
        "Non-Tamil word should have empty asai_amaivu"
    );
    assert!(
        paa.sorkal[0].asaikal.is_empty(),
        "Non-Tamil word should have no asaikal"
    );
}

fn load_kurals() -> Vec<String> {
    serde_json::from_str(KURAL_JSON).expect("Failed to parse kural.json")
}

/// Verify all 1330 kurals from the corpus.
/// Checks structural invariants and prints all failures before asserting.
#[test]
fn test_all_kurals() {
    let kurals = load_kurals();
    assert_eq!(kurals.len(), 1330);

    let mut failures: Vec<String> = Vec::new();

    let mut seven_words = 0;
    let mut four_three_split = 0;
    let mut all_valid_script = 0;

    for (idx, kural) in kurals.iter().enumerate() {
        let kural_num = idx + 1;
        let paa = preprocess(kural);

        // --- Structural checks (collect failures, don't stop) ---

        if paa.adikal.len() != 2 {
            failures.push(format!(
                "Kural #{}: expected 2 lines, got {} | {}",
                kural_num,
                paa.adikal.len(),
                kural.replace('\n', " / ")
            ));
        }

        if paa.sorkal.is_empty() {
            failures.push(format!("Kural #{}: no words produced", kural_num));
            continue;
        }

        if paa.adikal.len() == 2 {
            if paa.adikal[0].sol_varisaikal.is_empty() {
                failures.push(format!("Kural #{}: line 1 has no words", kural_num));
            }
            if paa.adikal[1].sol_varisaikal.is_empty() {
                failures.push(format!("Kural #{}: line 2 has no words", kural_num));
            }
        }

        // Word count check (use original_sol_count to ignore compound expansion)
        if paa.original_sol_count == 7 {
            seven_words += 1;
        } else {
            failures.push(format!(
                "Kural #{}: expected 7 original words, got {} | {}",
                kural_num,
                paa.original_sol_count,
                kural.replace('\n', " / ")
            ));
        }

        // Line split check (use logical_sol_count to ignore compound expansion)
        if paa.adikal.len() == 2
            && paa.adikal[0].logical_sol_count == 4
            && paa.adikal[1].logical_sol_count == 3
        {
            four_three_split += 1;
        } else if paa.adikal.len() == 2 {
            failures.push(format!(
                "Kural #{}: expected 4+3 split, got {}+{} | {}",
                kural_num,
                paa.adikal[0].logical_sol_count,
                paa.adikal[1].logical_sol_count,
                kural.replace('\n', " / ")
            ));
        }

        // Empty word check
        for (wi, sol) in paa.sorkal.iter().enumerate() {
            if sol.is_empty {
                failures.push(format!("Kural #{} word {}: empty word", kural_num, wi));
            }
        }

        // Script validation check
        let invalid_words: Vec<String> = paa
            .sorkal
            .iter()
            .filter(|s| !s.is_valid_script)
            .map(|s| format!("'{}' {:?}", s.raw_text, s.invalid_chars))
            .collect();
        if invalid_words.is_empty() {
            all_valid_script += 1;
        } else {
            failures.push(format!(
                "Kural #{}: non-Tamil chars in {}",
                kural_num,
                invalid_words.join(", ")
            ));
        }

        // Junction count check
        if paa.thalaikal.len() != paa.sorkal.len() - 1 {
            failures.push(format!(
                "Kural #{}: expected {} junctions, got {}",
                kural_num,
                paa.sorkal.len() - 1,
                paa.thalaikal.len()
            ));
        }

        // Cross-adi junction check
        if !paa.thalaikal.iter().any(|t| t.is_cross_adi) {
            failures.push(format!("Kural #{}: no cross-adi junction found", kural_num));
        }

        // Every word must have graphemes, syllables, and asaikal
        for (wi, sol) in paa.sorkal.iter().enumerate() {
            if sol.ezhuthukkal.is_empty() {
                failures.push(format!(
                    "Kural #{} word {} '{}': no graphemes",
                    kural_num, wi, sol.raw_text
                ));
            }
            if sol.syllables.is_empty() {
                failures.push(format!(
                    "Kural #{} word {} '{}': no syllables",
                    kural_num, wi, sol.raw_text
                ));
            }
            if sol.asaikal.is_empty() {
                failures.push(format!(
                    "Kural #{} word {} '{}': no asaikal",
                    kural_num, wi, sol.raw_text
                ));
            }
        }
    }

    // Print summary
    eprintln!("\n=== Kural Corpus Results ===");
    eprintln!("Total kurals: {}", kurals.len());
    eprintln!(
        "7-word kurals: {} ({:.1}%)",
        seven_words,
        seven_words as f64 / kurals.len() as f64 * 100.0
    );
    eprintln!(
        "4+3 split kurals: {} ({:.1}%)",
        four_three_split,
        four_three_split as f64 / kurals.len() as f64 * 100.0
    );
    eprintln!(
        "All-valid-script kurals: {} ({:.1}%)",
        all_valid_script,
        all_valid_script as f64 / kurals.len() as f64 * 100.0
    );

    if !failures.is_empty() {
        eprintln!("\nFailures ({}):", failures.len());
        for f in &failures {
            eprintln!("  {}", f);
        }
    }
    eprintln!("===========================\n");

    assert!(
        failures.is_empty(),
        "{} failures found across all kurals (see output above)",
        failures.len()
    );
}

/// Corpus statistics for L2/L3/L4 validation layers.
/// Reports frequencies and validates against expected ranges from docs.
#[test]
fn test_corpus_l2_l3_l4_statistics() {
    let kurals = load_kurals();
    let total = kurals.len();

    let mut etukai_present = 0;
    let mut monai_present = 0;
    let mut iyaipu_present = 0;
    let mut extended_seer_kurals = 0;
    let mut kani_malar_kurals = 0;
    let mut thalai_break_kurals = 0;
    let mut final_word_neer = 0;
    let mut final_word_eerrasai = 0;
    let mut final_word_valid = 0;

    for kural in &kurals {
        let paa = preprocess(kural);

        // L2: Count kurals with koovilam/karuvilam seers (valid in venba, just tracking)
        let has_koovilam_karuvilam = paa
            .sorkal
            .iter()
            .any(|s| s.seer_vagai == SeerType::Koovilam || s.seer_vagai == SeerType::Karuvilam);
        if has_koovilam_karuvilam {
            kani_malar_kurals += 1;
        }

        // L2: Final word seer_eerru check (use eetru_sol — compound-expansion-safe)
        let ends_neer = paa.eetru_sol.seer_eerru == AsaiType::Neer;
        let is_eerrasai = paa.eetru_sol.asai_count == 1;
        if ends_neer {
            final_word_neer += 1;
        }
        if is_eerrasai {
            final_word_eerrasai += 1;
        }
        if ends_neer || is_eerrasai {
            final_word_valid += 1;
        }

        // L2: Check for venseer (3-asai) words
        let has_venseer = paa
            .sorkal
            .iter()
            .any(|s| s.seer_category == SeerCategory::Venseer);
        if has_venseer {
            extended_seer_kurals += 1;
        }

        // L3: Check for thalai breaks (exclude intra-compound junctions)
        let has_thalai_break = paa.thalaikal.iter().any(|t| {
            !t.is_intra_compound
                && !t.is_cross_adi
                && ((t.from_seer_category == SeerCategory::Iyarseer
                    && t.eerru_asai == t.muthal_asai)
                    || (t.from_seer_category == SeerCategory::Venseer
                        && t.muthal_asai == AsaiType::Nirai))
        });
        if has_thalai_break {
            thalai_break_kurals += 1;
        }

        // L4: Use precomputed ani (ornamentation) data — handles compound expansion correctly
        if paa.ani.etukai_present {
            etukai_present += 1;
        }
        if paa.ani.monai_present {
            monai_present += 1;
        }
        if paa.ani.iyaipu_present {
            iyaipu_present += 1;
        }
    }

    let pct = |n: usize| -> f64 { n as f64 / total as f64 * 100.0 };

    eprintln!("\n=== L2/L3/L4 Corpus Statistics ===");
    eprintln!("Total kurals: {}", total);
    eprintln!();
    eprintln!("L2 Seer:");
    eprintln!(
        "  Koovilam/Karuvilam seer present: {} ({:.1}%)",
        kani_malar_kurals,
        pct(kani_malar_kurals)
    );
    eprintln!(
        "  Venseer (3-asai) present: {} ({:.1}%)",
        extended_seer_kurals,
        pct(extended_seer_kurals)
    );
    eprintln!(
        "  Final word ends neer: {} ({:.1}%)",
        final_word_neer,
        pct(final_word_neer)
    );
    eprintln!(
        "  Final word eerrasai (1 asai): {} ({:.1}%)",
        final_word_eerrasai,
        pct(final_word_eerrasai)
    );
    eprintln!(
        "  Final word valid (neer OR eerrasai): {} ({:.1}%)",
        final_word_valid,
        pct(final_word_valid)
    );
    eprintln!();
    eprintln!("L3 Thalai:");
    eprintln!(
        "  Intra-line thalai breaks: {} ({:.1}%)",
        thalai_break_kurals,
        pct(thalai_break_kurals)
    );
    eprintln!();
    eprintln!("L4 Ornamentation:");
    eprintln!(
        "  Etukai present: {} ({:.1}%)",
        etukai_present,
        pct(etukai_present)
    );
    eprintln!(
        "  Monai present: {} ({:.1}%)",
        monai_present,
        pct(monai_present)
    );
    eprintln!(
        "  Iyaipu present: {} ({:.1}%)",
        iyaipu_present,
        pct(iyaipu_present)
    );
    eprintln!("=================================\n");

    // Debug: print first 5 kurals where final word doesn't end with neer
    let mut debug_count = 0;
    for (idx, kural) in kurals.iter().enumerate() {
        let paa = preprocess(kural);
        if paa.sorkal.len() != 7 {
            continue;
        }
        if paa.sorkal[6].seer_eerru != AsaiType::Neer && debug_count < 5 {
            debug_count += 1;
            let w = &paa.sorkal[6];
            eprintln!("Kural #{}: {}", idx + 1, kural.replace('\n', " / "));
            eprintln!(
                "  Final word '{}': seer={:?} eerru={:?} asai={} asaikal={:?}",
                w.raw_text,
                w.seer_vagai,
                w.seer_eerru,
                w.asai_amaivu,
                w.asaikal
                    .iter()
                    .map(|a| format!("{}({:?})", a.text, a.vagai))
                    .collect::<Vec<_>>()
            );
            eprintln!(
                "  Syllables: {:?}",
                w.syllables
                    .iter()
                    .map(|s| format!("{}(alavu={:?},closed={})", s.text, s.alavu, s.is_closed))
                    .collect::<Vec<_>>()
            );
        }
    }
}

// ================================================================
// Engine Analysis Workflow Integration Tests
// These test the analysis workflows through the dataflow engine.
// ================================================================

/// Helper to get an analysis tag value from the message.
fn get_analysis_tag(message: &Message, tag: &str) -> serde_json::Value {
    message
        .data()
        .get("analysis")
        .and_then(|a| a.get("tags"))
        .and_then(|t| t.get(tag))
        .cloned()
        .unwrap_or(serde_json::Value::Null)
}

/// Helper to get a classification field from the message.
fn get_classification(message: &Message, field: &str) -> serde_json::Value {
    message
        .data()
        .get("analysis")
        .and_then(|a| a.get("classification"))
        .and_then(|c| c.get(field))
        .cloned()
        .unwrap_or(serde_json::Value::Null)
}

#[tokio::test]
async fn test_engine_valid_kural1() {
    let msg = run_engine("அகர முதல எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு").await;

    // Classification
    assert_eq!(get_classification(&msg, "paa_family"), json!("venba"));
    assert_eq!(get_classification(&msg, "venba_type"), json!("kural_venba"));
    assert_eq!(get_classification(&msg, "adi_count"), json!(2));
    assert_eq!(get_classification(&msg, "sol_count"), json!(7));

    // Structural tags
    assert_eq!(get_analysis_tag(&msg, "valid_tamil"), json!(true));
    assert_eq!(get_analysis_tag(&msg, "no_empty_words"), json!(true));
    assert_eq!(get_analysis_tag(&msg, "syllabification_ok"), json!(true));

    // Ornamentation tags
    assert_eq!(
        get_analysis_tag(&msg, "etukai"),
        json!(true),
        "Etukai present (க == க)"
    );
    assert_eq!(
        get_analysis_tag(&msg, "monai"),
        json!(false),
        "Monai absent (அ vs ம)"
    );
    assert_eq!(
        get_analysis_tag(&msg, "iyaipu"),
        json!(true),
        "Iyaipu present (kuril == kuril)"
    );

    // Seer tags
    assert_eq!(get_analysis_tag(&msg, "has_overflow"), json!(false));
    assert_eq!(get_analysis_tag(&msg, "kutrilugaram"), json!(true));
    assert_eq!(get_analysis_tag(&msg, "eetru_type"), json!("kaasu"));
}

#[tokio::test]
async fn test_engine_classification_word_count() {
    // 3 words — not kural_venba
    let msg = run_engine("அகர முதல\nபகவன்").await;
    assert_eq!(get_classification(&msg, "venba_type"), json!("unknown"));
    assert_eq!(get_classification(&msg, "sol_count"), json!(3));
}

#[tokio::test]
async fn test_engine_classification_line_count() {
    // 7 words on a single line — not kural_venba
    let msg = run_engine("அகர முதல எழுத்தெல்லாம் ஆதி பகவன் முதற்றே உலகு").await;
    assert_eq!(get_classification(&msg, "adi_count"), json!(1));
    assert_eq!(get_classification(&msg, "venba_type"), json!("unknown"));
}

#[tokio::test]
async fn test_engine_invalid_script_tag() {
    let msg = run_engine("hello முதல எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு").await;
    assert_eq!(
        get_analysis_tag(&msg, "valid_tamil"),
        json!(false),
        "Non-Tamil word should set valid_tamil=false"
    );
}

#[tokio::test]
async fn test_engine_venseer_no_overflow() {
    // Kural #1 has எழுத்தெல்லாம் (3 asai = Pulimangai, valid Venseer)
    let msg = run_engine("அகர முதல எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு").await;
    assert_eq!(get_analysis_tag(&msg, "has_overflow"), json!(false));
}

#[tokio::test]
async fn test_engine_syllabification_fail_tag() {
    // Non-Tamil word "hello" → syllabification_ok should be false
    let msg = run_engine("hello முதல எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு").await;
    assert_eq!(
        get_analysis_tag(&msg, "syllabification_ok"),
        json!(false),
        "Non-Tamil word should set syllabification_ok=false"
    );
}

#[tokio::test]
async fn test_engine_etukai_present() {
    // Kural #1: Word[0]="அகர" 2nd grapheme mei="க", Word[4]="பகவன்" 2nd mei="க" → match
    let msg = run_engine("அகர முதல எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு").await;
    assert_eq!(get_analysis_tag(&msg, "etukai"), json!(true));
}

/// Regression: every historically valid kural must classify as kural_venba
/// when run through the full engine analysis pipeline.
#[tokio::test]
async fn test_all_kurals_classify_as_kural_venba() {
    let kurals: Vec<String> = serde_json::from_str(KURAL_JSON).unwrap();
    let engine = create_engine();
    let mut failures: Vec<String> = Vec::new();

    for (idx, kural) in kurals.iter().enumerate() {
        let mut message = Message::new(Arc::new(json!({})));
        message.context["data"]["input"] = json!(kural);
        engine.process_message(&mut message).await.unwrap();

        let venba_type = get_classification(&message, "venba_type");
        let venba_type = venba_type.as_str().unwrap_or("missing");

        if venba_type != "kural_venba" {
            failures.push(format!(
                "Kural #{}: classified as '{}' | {}",
                idx + 1,
                venba_type,
                kural.replace('\n', " / ")
            ));
        }
    }

    if !failures.is_empty() {
        eprintln!("\n=== Classification Failures ({}) ===", failures.len());
        for f in &failures {
            eprintln!("  {}", f);
        }
        eprintln!("====================================\n");
    }

    assert!(
        failures.is_empty(),
        "{} kurals failed kural_venba classification (see output above)",
        failures.len()
    );
}

// === Sandhi Resolution Tests ===

#[test]
fn test_sandhi_pluti_resolution_in_preprocessing() {
    // Kural #2: "கற்றதனால் ஆய பயனென்கொல் வாலறிவன்\nநற்றாள் தொழாஅர் எனின்"
    // Word 5 "தொழாஅர்" has pluti ாஅ → should resolve to "தொழார்"
    let input = "கற்றதனால் ஆய பயனென்கொல் வாலறிவன்\nநற்றாள் தொழாஅர் எனின்";
    let paa = preprocess(input);

    let w5 = &paa.sorkal[5];
    assert_eq!(w5.raw_text, "தொழாஅர்");
    assert_eq!(w5.phonological_text, Some("தொழார்".to_string()));
    // After pluti: தொழார் = [தொ(kuril,open) + ழார்(nedil,closed)] = 1 Nirai asai
    assert_eq!(w5.asai_count, 1);
    assert_eq!(w5.seer_category, SeerCategory::Iyarseer);
}

#[test]
fn test_sandhi_pluti_uu_resolution() {
    // Test ூ+உ pluti: word "தூஉம்" → "தூம்"
    // Use a synthetic 7-word input with a pluti word
    let input = "அகர முதல எழுத்தெல்லாம் ஆதி\nபகவன் தூஉம் உலகு";
    let paa = preprocess(input);

    let w5 = &paa.sorkal[5]; // "தூஉம்"
    assert_eq!(w5.phonological_text, Some("தூம்".to_string()));
    assert_eq!(w5.asai_count, 1); // Single nedil closed syllable = Neer
}

#[test]
fn test_sandhi_no_pluti_words_unchanged() {
    let input = "அகர முதல எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு";
    let paa = preprocess(input);

    // No pluti in Kural #1 — all phonological_text should be None
    for sol in &paa.sorkal {
        assert_eq!(
            sol.phonological_text, None,
            "Word '{}' has no pluti",
            sol.raw_text
        );
    }
}

#[test]
fn test_sandhi_pluti_impact_on_corpus() {
    let kurals = load_kurals();
    let mut pluti_resolved_count = 0;

    for kural in &kurals {
        let paa = preprocess(kural);
        for sol in &paa.sorkal {
            if sol.phonological_text.is_some() {
                pluti_resolved_count += 1;
            }
        }
    }

    eprintln!("\n=== Sandhi Resolution Corpus Impact ===");
    eprintln!("  Words with pluti resolved: {}", pluti_resolved_count);
    eprintln!("========================================\n");
    // Expect ~100+ words with pluti in the corpus
    assert!(
        pluti_resolved_count > 80,
        "Expected significant pluti resolution in corpus"
    );
}

#[test]
fn test_sandhi_compound_boundary_detection() {
    let kurals = load_kurals();
    let mut compound_count = 0;
    let mut compound_overflow_count = 0;

    for kural in &kurals {
        let paa = preprocess(kural);
        for sol in &paa.sorkal {
            if sol.has_compound_boundary {
                compound_count += 1;
                if sol.seer_category == SeerCategory::Overflow {
                    compound_overflow_count += 1;
                }
            }
        }
    }

    eprintln!("\n=== Compound Boundary Detection ===");
    eprintln!("  Words with compound boundary: {}", compound_count);
    eprintln!(
        "  Of those, overflow (4+ asais): {}",
        compound_overflow_count
    );
    eprintln!("====================================\n");
    assert!(compound_count > 0, "Should detect some compound boundaries");
}

// === TEMPORARY: Overflow Suffix Analysis ===
// Research test to analyze overflow (4+ asai) words and find common suffix patterns
// for compound word decomposition.

/// Compound decomposition corpus test — measures how many overflow words
/// are resolved by the compound word decomposition algorithm.
#[test]
fn test_compound_decomposition_corpus() {
    let kurals = load_kurals();
    let mut total_overflow = 0;
    let mut total_compound_splits = 0;
    let mut total_expanded_words = 0;
    let mut remaining_overflow: Vec<(usize, String)> = Vec::new();

    for (idx, kural) in kurals.iter().enumerate() {
        let paa = preprocess(kural);

        // Count compound sub-units
        for sol in &paa.sorkal {
            if sol.compound_source_index.is_some() {
                total_expanded_words += 1;
            }
            if sol.seer_category == SeerCategory::Overflow {
                total_overflow += 1;
                remaining_overflow.push((idx + 1, sol.raw_text.clone()));
            }
        }

        // Count compound splits (distinct source indices)
        let mut seen_sources = std::collections::HashSet::new();
        for sol in &paa.sorkal {
            if let Some(src) = sol.compound_source_index {
                seen_sources.insert(src);
            }
        }
        total_compound_splits += seen_sources.len();
    }

    eprintln!("\n=== Compound Decomposition Corpus Results ===");
    eprintln!("  Compound splits performed: {}", total_compound_splits);
    eprintln!("  Expanded sub-words created: {}", total_expanded_words);
    eprintln!("  Remaining overflow words: {}", total_overflow);

    // Show unique remaining overflows
    let mut unique_remaining: HashMap<String, Vec<usize>> = HashMap::new();
    for (kural_num, word) in &remaining_overflow {
        unique_remaining
            .entry(word.clone())
            .or_default()
            .push(*kural_num);
    }
    eprintln!("  Unique remaining overflow: {}", unique_remaining.len());

    if !unique_remaining.is_empty() {
        eprintln!("\n  Remaining overflow words:");
        let mut sorted: Vec<_> = unique_remaining.iter().collect();
        sorted.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
        for (word, kural_nums) in sorted.iter().take(20) {
            eprintln!(
                "    {} (x{}) kurals={:?}",
                word,
                kural_nums.len(),
                kural_nums
            );
        }
    }
    eprintln!("=============================================\n");

    // Verify structural integrity after compound expansion
    for kural in &kurals {
        let paa = preprocess(kural);
        assert_eq!(paa.original_sol_count, 7);
        assert_eq!(paa.adikal.len(), 2);
        assert_eq!(paa.adikal[0].logical_sol_count, 4);
        assert_eq!(paa.adikal[1].logical_sol_count, 3);

        // Verify compound sub-units have valid seer types
        for sol in &paa.sorkal {
            if sol.compound_source_index.is_some() {
                assert_ne!(
                    sol.seer_category,
                    SeerCategory::Overflow,
                    "Compound sub-unit '{}' from '{}' should not be overflow",
                    sol.normalized_text,
                    sol.compound_source_text.as_deref().unwrap_or("?")
                );
            }
        }
    }
}

// ================================================================
// Steps 1-7: New L2 Rule Tests (kutrilugaram, syllabify_fail, ambiguous_asai)
// ================================================================

#[test]
fn test_kutrilugaram_detection() {
    // Kural #1: final word "உலகு" ends with கு → is_kutrilugaram = true
    let paa = preprocess("அகர முதல எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு");
    assert!(
        paa.eetru_sol.is_kutrilugaram,
        "உலகு ends with கு → kutrilugaram"
    );

    // Kural #2: final word "எனின்" ends with ன் → is_kutrilugaram = false
    let paa2 = preprocess("கற்றதனால் ஆய பயனென்கொல் வாலறிவன்\nநற்றாள் தொழாஅர் எனின்");
    assert!(
        !paa2.eetru_sol.is_kutrilugaram,
        "எனின் ends with ன் → not kutrilugaram"
    );
}

#[test]
fn test_kutrilugaram_1asai_exempt() {
    // Kural #2: final word "எனின்" is 1-asai (Malar) → kutrilugaram should NOT be required
    let paa = preprocess("கற்றதனால் ஆய பயனென்கொல் வாலறிவன்\nநற்றாள் தொழாஅர் எனின்");
    let last = &paa.sorkal[paa.sorkal.len() - 1];
    assert_eq!(
        last.asai_count, 1,
        "Final word should be 1-asai (Malar/Naal)"
    );
    // 1-asai eetru seer does NOT need kutrilugaram per grammar rules
    assert!(!paa.eetru_sol.is_kutrilugaram);
}

#[test]
fn test_kutrilugaram_2asai_valid() {
    // Kural #1: final word "உலகு" is 2-asai ending Neer + kutrilugaram (கு) → Pirappu ✓
    let paa = preprocess("அகர முதல எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு");
    assert_eq!(paa.eetru_sol.asai_count, 2);
    assert!(
        paa.eetru_sol.is_kutrilugaram,
        "2-asai ending with கு → valid Pirappu"
    );
}

#[test]
fn test_syllabification_failed_field() {
    // Non-Tamil input: "hello" → syllabification_failed = true
    let paa = preprocess("hello முதல எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு");
    assert!(
        paa.sorkal[0].syllabification_failed,
        "hello → syllabification_failed"
    );

    // Valid Tamil: "அகர" → syllabification_failed = false
    assert!(!paa.sorkal[1].syllabification_failed, "முதல → not failed");
}

#[test]
fn test_ambiguous_asai_field() {
    // Doubled consonants (ற்ற) are normal Tamil gemination, NOT compound boundaries
    // Only matra+vowel pattern indicates real compound boundaries
    let paa = preprocess("அகர முதல எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு");
    let muthatree = &paa.sorkal[5]; // முதற்றே
    assert!(
        !muthatree.has_compound_boundary,
        "முதற்றே gemination is not a compound boundary"
    );
    assert!(
        !muthatree.ambiguous_asai,
        "முதற்றே should not have ambiguous_asai"
    );

    // Matra + standalone vowel IS a compound boundary → ambiguous_asai
    // சுவைஒளி: ை + ஒ (matra + vowel, not pluti)
    let paa2 = preprocess("சுவைஒளி");
    let sol = &paa2.sorkal[0];
    assert!(
        sol.has_compound_boundary,
        "சுவைஒளி should have compound boundary"
    );
    assert!(
        sol.ambiguous_asai,
        "சுவைஒளி should have ambiguous_asai (3 asai, not decomposed)"
    );
}

#[test]
fn test_ambiguous_asai_not_for_decomposed() {
    // Compound sub-units (from decomposition) should NOT have ambiguous_asai
    let kurals = load_kurals();
    for kural in &kurals {
        let paa = preprocess(kural);
        for sol in &paa.sorkal {
            if sol.compound_source_index.is_some() {
                assert!(
                    !sol.ambiguous_asai,
                    "Compound sub-unit '{}' should not be ambiguous",
                    sol.normalized_text
                );
            }
        }
    }
}

#[tokio::test]
async fn test_engine_kural1_kutrilugaram_tag() {
    // Kural #1 has 2-asai final word "உலகு" with kutrilugaram
    let msg = run_engine("அகர முதல எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு").await;
    assert_eq!(
        get_analysis_tag(&msg, "kutrilugaram"),
        json!(true),
        "Kural #1 final word has kutrilugaram ending"
    );
}

/// Corpus statistics for new L2 rules: kutrilugaram improvement and ambiguous_asai count.
#[test]
fn test_corpus_new_l2_statistics() {
    let kurals = load_kurals();
    let total = kurals.len();

    let mut kutrilugaram_true = 0;
    let mut asai1_final = 0;
    let mut asai2_final = 0;
    let mut asai2_kutrilugaram = 0;
    let mut ambiguous_count = 0;
    let mut syllabify_fail_count = 0;

    for kural in &kurals {
        let paa = preprocess(kural);

        // Eetru sol stats
        if paa.eetru_sol.is_kutrilugaram {
            kutrilugaram_true += 1;
        }
        if paa.eetru_sol.asai_count == 1 {
            asai1_final += 1;
        }
        if paa.eetru_sol.asai_count == 2 {
            asai2_final += 1;
        }
        if paa.eetru_sol.asai_count == 2 && paa.eetru_sol.is_kutrilugaram {
            asai2_kutrilugaram += 1;
        }

        // Per-word stats
        for sol in &paa.sorkal {
            if sol.ambiguous_asai {
                ambiguous_count += 1;
            }
            if sol.syllabification_failed {
                syllabify_fail_count += 1;
            }
        }
    }

    let pct = |n: usize| -> f64 { n as f64 / total as f64 * 100.0 };

    eprintln!("\n=== New L2 Corpus Statistics ===");
    eprintln!("Total kurals: {}", total);
    eprintln!();
    eprintln!("Eetru Seer:");
    eprintln!(
        "  1-asai final words (Naal/Malar): {} ({:.1}%)",
        asai1_final,
        pct(asai1_final)
    );
    eprintln!(
        "  2-asai final words: {} ({:.1}%)",
        asai2_final,
        pct(asai2_final)
    );
    eprintln!(
        "  2-asai with kutrilugaram: {} ({:.1}%)",
        asai2_kutrilugaram,
        pct(asai2_kutrilugaram)
    );
    eprintln!(
        "  is_kutrilugaram overall: {} ({:.1}%)",
        kutrilugaram_true,
        pct(kutrilugaram_true)
    );
    eprintln!();
    eprintln!("  -> Old W_KUTRILUGARA rule would fire on: ~60% (all words)");
    eprintln!(
        "  -> New rule fires only on 2-asai without kutrilugaram: {} ({:.1}%)",
        asai2_final - asai2_kutrilugaram,
        pct(asai2_final - asai2_kutrilugaram)
    );
    eprintln!();
    eprintln!("New L2 fields:");
    eprintln!("  Words with ambiguous_asai: {}", ambiguous_count);
    eprintln!(
        "  Words with syllabification_failed: {}",
        syllabify_fail_count
    );
    eprintln!("================================\n");

    // Verify no syllabification failures in corpus (all valid Tamil)
    assert_eq!(
        syllabify_fail_count, 0,
        "No Tamil words should fail syllabification"
    );
}

// === Venba Sub-Type Classification Tests ===

#[tokio::test]
async fn test_engine_sindhiyal_venba_classification() {
    // 3 lines, 4+4+3 = 11 words
    let msg = run_engine(
        "சுரையாழ அம்மி மிதப்ப வரையனைய\n\
         யானைக்கு நீத்து முயற்கு நிலைஎன்ப\n\
         கானக நாடன் சுனை",
    )
    .await;
    assert_eq!(get_classification(&msg, "paa_family"), json!("venba"));
    assert_eq!(
        get_classification(&msg, "venba_type"),
        json!("sindhiyal_venba")
    );
    assert_eq!(get_classification(&msg, "adi_count"), json!(3));
    assert_eq!(get_classification(&msg, "sol_count"), json!(11));
}

#[tokio::test]
async fn test_engine_alaviyal_venba_classification() {
    // 4 lines, 4+4+4+3 = 15 words (Nerisai example — classified as alaviyal)
    let msg = run_engine(
        "நெல்லுக் கிறைத்தநீர் வாய்க்கால் வழியோடிப்\n\
         புல்லுக்கு மாங்கே பொசியுமாம் தொல்லுலகில்\n\
         நல்லா ரொருவர் உளரேல் அவர்பொருட்டு\n\
         எல்லோர்க்கும் பெய்யும் மழை",
    )
    .await;
    assert_eq!(get_classification(&msg, "paa_family"), json!("venba"));
    assert_eq!(
        get_classification(&msg, "venba_type"),
        json!("alaviyal_venba")
    );
    assert_eq!(get_classification(&msg, "adi_count"), json!(4));
    assert_eq!(get_classification(&msg, "sol_count"), json!(15));
}

#[tokio::test]
async fn test_engine_alaviyal_venba_innisai_classification() {
    // 4 lines, Innisai example — also classified as alaviyal
    let msg = run_engine(
        "கடைகலக்காற் காயார் கழிகமழ்ஞ் செய்யார்\n\
         கொடையளிக்கண் போச்சாவார் கோலநேர் செய்யார்\n\
         இடையறுத்துப் போகப் பிறனொருவன் சேரார்\n\
         கடையபாக வாழ்தமென் பார்",
    )
    .await;
    assert_eq!(
        get_classification(&msg, "venba_type"),
        json!("alaviyal_venba")
    );
    assert_eq!(get_classification(&msg, "adi_count"), json!(4));
}

#[tokio::test]
async fn test_engine_pahrodai_venba_5_lines() {
    // 5 lines, 4+4+4+4+3 = 19 words
    let msg = run_engine(
        "தென்னவன் கன்னிச் செழுஞ்சாரல் மாமலைவாய்ப்\n\
         பொன்னிறப் பூவேர் புதுமலராம் நன்னெறியார்\n\
         ஆரம் புனைந்த அம்மணி மேகலை\n\
         பாரம் சுமந்து பயிலுமே வீரர்க்கும்\n\
         ஓசை ஒலியும் உடைத்து",
    )
    .await;
    assert_eq!(get_classification(&msg, "paa_family"), json!("venba"));
    assert_eq!(
        get_classification(&msg, "venba_type"),
        json!("pahrodai_venba")
    );
    assert_eq!(get_classification(&msg, "adi_count"), json!(5));
}

#[tokio::test]
async fn test_engine_pahrodai_venba_6_lines() {
    // 6 lines, 4+4+4+4+4+3 = 23 words
    let msg = run_engine(
        "வான்மழை பெய்து வழிந்தோடும் வாரிபோல்\n\
         யான்பெற்ற செல்வமும் ஈகையே வான்பொருளும்\n\
         இல்லார்க்கு ஈவதே இன்பமென எண்ணுவார்\n\
         நல்லார் ஒருவரே நற்பயனாம் புல்லார்க்கும்\n\
         எல்லாம் வழங்கி மகிழ்வதே மெல்லியல்\n\
         நல்லார் செயல் தரும்",
    )
    .await;
    assert_eq!(
        get_classification(&msg, "venba_type"),
        json!("pahrodai_venba")
    );
    assert_eq!(get_classification(&msg, "adi_count"), json!(6));
}

#[tokio::test]
async fn test_engine_kali_venba_classification() {
    // 13 lines (Kandar Kali Venba opening, last line 3 words)
    let msg = run_engine(
        "பூமேவு செங்கமலப் புத்தேளும் தேறரிய\n\
         பாமேவு தெய்வப் பழமறையும் தேமேவு\n\
         நாதமும் நாதாந்த முடிவும் நவைதீர்ந்த\n\
         போதமும் காணாத போதமாய் ஆதிநடு\n\
         அந்தம் கடந்தநித்தி யானந்த போதமாய்ப்\n\
         பந்தம் தணந்த பரஞ்சுடராய் வந்த\n\
         அடியார் இதயத் தாமரை மேலமர்ந்த\n\
         நெடியான் மருகன் நிமலன் வடியார்\n\
         வேலோன் மயில்வீரன் வெற்றிப் புயத்தவன்\n\
         காலோன் வணங்கும் கதிரவன் மேலோர்\n\
         புகழும் புகழவன் பொன்னடி போற்றி\n\
         இகழும் வினைதீர்க்கும் ஈசன் மகனாய்\n\
         கந்தன் மலரடி போற்றி",
    )
    .await;
    assert_eq!(get_classification(&msg, "venba_type"), json!("kali_venba"));
    assert_eq!(get_classification(&msg, "adi_count"), json!(13));
}

#[tokio::test]
async fn test_engine_non_venba_graceful_unknown() {
    // 2 lines with 5+3 words (not venba's required 4+3)
    // Should classify as "unknown" without error
    let msg = run_engine("அகர முதல எழுத்தெல்லாம் ஆதி பகவன்\nமுதற்றே உலகு தமிழ்").await;
    assert_eq!(get_classification(&msg, "paa_family"), json!("unknown"));
    assert_eq!(get_classification(&msg, "venba_type"), json!("unknown"));
}
