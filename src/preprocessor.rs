use async_trait::async_trait;
use dataflow_rs::engine::{
    error::{DataflowError, Result},
    functions::{config::FunctionConfig, AsyncFunctionHandler},
    message::{Change, Message},
};
use datalogic_rs::DataLogic;
use serde_json::Value;
use std::sync::Arc;

use crate::tamil::{compound, grapheme, prosody, sandhi, syllable, unicode};
use crate::types::*;

pub struct Preprocessor;

#[async_trait]
impl AsyncFunctionHandler for Preprocessor {
    async fn execute(
        &self,
        message: &mut Message,
        config: &FunctionConfig,
        _datalogic: Arc<DataLogic>,
    ) -> Result<(usize, Vec<Change>)> {
        // Verify custom function config
        let _input = match config {
            FunctionConfig::Custom { input, .. } => input,
            _ => {
                return Err(DataflowError::Validation(
                    "Expected custom function config".to_string(),
                ))
            }
        };

        // Read raw input
        let raw_input = message.data()["input"]
            .as_str()
            .ok_or_else(|| DataflowError::Validation("Missing data.input string".to_string()))?
            .to_string();

        // Run preprocessing pipeline
        let paa = preprocess(&raw_input);

        // Serialize to JSON
        let paa_value = serde_json::to_value(&paa)
            .map_err(|e| DataflowError::Validation(format!("Serialization error: {}", e)))?;

        // Write to message context
        let old_value = message.data().get("paa").cloned().unwrap_or(Value::Null);

        message.data_mut()["paa"] = paa_value.clone();
        message.invalidate_context_cache();

        Ok((
            200,
            vec![Change {
                path: Arc::from("data.paa"),
                old_value: Arc::new(old_value),
                new_value: Arc::new(paa_value),
            }],
        ))
    }
}

/// Contextual metadata for a word being processed through the prosodic pipeline.
struct WordContext {
    raw_text: String,
    adi_index: usize,
    adi_idanam: usize,
    normalized_text: String,
    phonological_text: Option<String>,
    is_valid_script: bool,
    invalid_chars: Vec<String>,
    is_empty: bool,
    non_tamil_stripped: bool,
    has_compound_boundary: bool,
    compound_source_index: Option<usize>,
    compound_part: Option<usize>,
    compound_source_text: Option<String>,
}

/// Process a text fragment through the full prosodic pipeline, returning a SolData.
fn process_word_text(analysis_text: &str, ctx: WordContext) -> SolData {
    let graphemes = grapheme::extract_graphemes(analysis_text);
    let gdata = grapheme::word_grapheme_data(&graphemes);

    let muthal_ezhuthu_monai_kurippu = graphemes.first().and_then(|g| match g.vagai {
        grapheme::GraphemeType::Uyir => g
            .text
            .chars()
            .next()
            .and_then(unicode::vowel_monai_group)
            .map(|c| c.to_string()),
        grapheme::GraphemeType::Uyirmei | grapheme::GraphemeType::Mei => {
            g.mei.map(|c| c.to_string())
        }
        grapheme::GraphemeType::Aytham => Some("ஃ".to_string()),
    });

    let syllables = syllable::syllabify(&graphemes);
    let asaikal = prosody::classify_asai(&syllables);
    let seer_data = prosody::classify_seer(&asaikal);

    let syllabification_failed = seer_data.asai_count == 0;
    let ambiguous_asai = ctx.has_compound_boundary
        && ctx.compound_source_index.is_none()
        && seer_data.asai_count > 0
        && seer_data.asai_count <= 3;

    let ezhuthukkal: Vec<EzhuthuData> = graphemes.iter().map(EzhuthuData::from).collect();
    let syllable_data: Vec<SyllableData> = syllables.iter().map(SyllableData::from).collect();
    let asai_data: Vec<AsaiData> = asaikal.iter().map(AsaiData::from).collect();

    SolData {
        adi_index: ctx.adi_index,
        adi_idanam: ctx.adi_idanam,
        raw_text: ctx.raw_text,
        normalized_text: ctx.normalized_text,
        phonological_text: ctx.phonological_text,
        is_valid_script: ctx.is_valid_script,
        invalid_chars: ctx.invalid_chars,
        is_empty: ctx.is_empty,
        non_tamil_stripped: ctx.non_tamil_stripped,
        ezhuthukkal,
        muthal_ezhuthu_monai_kurippu,
        kadai_ezhuthu: gdata.kadai_ezhuthu,
        kadai_ezhuthu_mei: gdata.kadai_ezhuthu_mei,
        kadai_ezhuthu_alavu: gdata.kadai_ezhuthu_alavu,
        kadai_ezhuthu_vagai: gdata.kadai_ezhuthu_vagai,
        syllables: syllable_data,
        asaikal: asai_data,
        asai_amaivu: seer_data.asai_amaivu,
        seer_vagai: seer_data.seer_vagai,
        seer_category: seer_data.seer_category,
        asai_count: seer_data.asai_count,
        seer_muthal: seer_data.seer_muthal,
        seer_eerru: seer_data.seer_eerru,
        syllabification_failed,
        ambiguous_asai,
        has_compound_boundary: ctx.has_compound_boundary,
        compound_source_index: ctx.compound_source_index,
        compound_part: ctx.compound_part,
        compound_source_text: ctx.compound_source_text,
    }
}

pub fn preprocess(raw_input: &str) -> PaaData {
    // Step 1: Parse lines and words
    let lines: Vec<&str> = raw_input.split('\n').collect();
    let mut all_words: Vec<(usize, &str)> = Vec::new();
    for (line_idx, line) in lines.iter().enumerate() {
        for word in line.split_whitespace() {
            // Skip pure punctuation tokens (e.g. standalone "-", ".")
            if !word.chars().any(|c| c.is_alphabetic()) {
                continue;
            }
            all_words.push((line_idx, word));
        }
    }

    // Step 2-6: Process each word (initial pass — no compound expansion yet)
    let mut sorkal: Vec<SolData> = Vec::new();
    let mut word_in_line_count: Vec<usize> = vec![0; lines.len()];

    for (global_idx, (line_idx, raw_word)) in all_words.iter().enumerate() {
        let adi_idanam = word_in_line_count[*line_idx];
        word_in_line_count[*line_idx] += 1;

        let normalized = unicode::normalize_nfc(raw_word);
        let (text, non_tamil_stripped) = unicode::strip_non_tamil(&normalized);
        let is_empty = text.is_empty();
        // Validate stripped text; if stripping removed everything (e.g. "hello"), flag as invalid
        let (is_valid_script, invalid_chars) = if is_empty {
            let invalid: Vec<char> = normalized.chars().collect();
            (false, invalid)
        } else {
            unicode::validate_script(&text)
        };

        let sandhi_result = sandhi::resolve(&text);
        let phonological_text = if sandhi_result.pluti_resolved {
            Some(sandhi_result.phonological_text.clone())
        } else {
            None
        };

        let analysis_text = phonological_text.as_deref().unwrap_or(&text).to_string();

        sorkal.push(process_word_text(
            &analysis_text,
            WordContext {
                raw_text: raw_word.to_string(),
                adi_index: *line_idx,
                adi_idanam,
                normalized_text: normalized,
                phonological_text,
                is_valid_script,
                invalid_chars: invalid_chars.iter().map(|c| c.to_string()).collect(),
                is_empty,
                non_tamil_stripped,
                has_compound_boundary: sandhi_result.has_compound_boundary,
                compound_source_index: None,
                compound_part: None,
                compound_source_text: None,
            },
        ));
    }

    let original_sol_count = sorkal.len();

    // Step 6a: Compute ornamentation (ani) data using original word positions
    let ani = compute_ani(&sorkal, &lines);

    // Step 6b: Compound word expansion — decompose overflow words
    let mut expanded_sorkal: Vec<SolData> = Vec::new();
    for (orig_idx, sol) in sorkal.into_iter().enumerate() {
        if sol.seer_category == prosody::SeerCategory::Overflow {
            let analysis_text = sol
                .phonological_text
                .as_deref()
                .unwrap_or(&sol.normalized_text);
            if let Some(parts) = compound::decompose_compound(analysis_text) {
                for (part_idx, part_text) in parts.iter().enumerate() {
                    expanded_sorkal.push(process_word_text(
                        part_text,
                        WordContext {
                            raw_text: sol.raw_text.clone(),
                            adi_index: sol.adi_index,
                            adi_idanam: 0, // recalculated below
                            normalized_text: part_text.to_string(),
                            phonological_text: Some(part_text.to_string()),
                            is_valid_script: sol.is_valid_script,
                            invalid_chars: sol.invalid_chars.clone(),
                            is_empty: false,
                            non_tamil_stripped: sol.non_tamil_stripped
                                && part_idx == parts.len() - 1,
                            has_compound_boundary: false,
                            compound_source_index: Some(orig_idx),
                            compound_part: Some(part_idx),
                            compound_source_text: Some(sol.raw_text.clone()),
                        },
                    ));
                }
                continue;
            }
        }
        expanded_sorkal.push(sol);
    }

    // Recalculate adi_idanam after expansion
    let mut line_word_counts: Vec<usize> = vec![0; lines.len()];
    let mut sorkal = expanded_sorkal;
    for sol in sorkal.iter_mut() {
        sol.adi_idanam = line_word_counts[sol.adi_index];
        line_word_counts[sol.adi_index] += 1;
    }

    let adikal = build_adikal(&sorkal, &lines);
    let thalaikal = build_thalaikal(&sorkal);
    let eetru_sol = build_eetru_sol(&sorkal);

    PaaData {
        raw_input: raw_input.to_string(),
        original_sol_count,
        eetru_sol,
        ani,
        adikal,
        sorkal,
        thalaikal,
        diagnostics: vec![],
    }
}

/// Build adi (line) data from processed words.
fn build_adikal(sorkal: &[SolData], lines: &[&str]) -> Vec<AdiData> {
    let mut adikal: Vec<AdiData> = Vec::new();
    for (line_idx, line_text) in lines.iter().enumerate() {
        let mut sol_varisaikal: Vec<usize> = Vec::new();
        let mut seer_vagaikal: Vec<prosody::SeerType> = Vec::new();
        let mut seen_sources = std::collections::HashSet::new();
        let mut logical_count = 0usize;
        let mut syllable_count_total = 0usize;
        let mut matrai_total = 0u32;

        for (i, w) in sorkal.iter().enumerate() {
            if w.adi_index != line_idx {
                continue;
            }
            sol_varisaikal.push(i);
            seer_vagaikal.push(w.seer_vagai);
            syllable_count_total += w.syllables.len();
            matrai_total += w.syllables.iter().map(|s| s.matrai as u32).sum::<u32>();

            if let Some(src) = w.compound_source_index {
                if seen_sources.insert(src) {
                    logical_count += 1;
                }
            } else {
                logical_count += 1;
            }
        }

        adikal.push(AdiData {
            text: line_text.to_string(),
            sol_varisaikal,
            seer_vagaikal,
            logical_sol_count: logical_count,
            syllable_count_total,
            matrai_total,
        });
    }
    adikal
}

/// Compute junctions (thalaikal) between consecutive words.
fn build_thalaikal(sorkal: &[SolData]) -> Vec<ThalaiData> {
    let last_sol_index = sorkal.len().saturating_sub(1);
    (0..sorkal.len().saturating_sub(1))
        .map(|i| {
            let is_intra_compound = match (
                sorkal[i].compound_source_index,
                sorkal[i + 1].compound_source_index,
            ) {
                (Some(a), Some(b)) => a == b,
                _ => false,
            };

            let eerru_asai = sorkal[i].seer_eerru;
            let muthal_asai = sorkal[i + 1].seer_muthal;
            let from_cat = sorkal[i].seer_category;
            let to_cat = sorkal[i + 1].seer_category;

            let (thalai_type, thalai_valid, thalai_detail) =
                classify_thalai(from_cat, to_cat, eerru_asai, muthal_asai, is_intra_compound);

            ThalaiData {
                from_sol_index: i,
                to_sol_index: i + 1,
                from_seer_category: from_cat,
                to_seer_category: to_cat,
                eerru_asai,
                muthal_asai,
                is_cross_adi: sorkal[i].adi_index != sorkal[i + 1].adi_index,
                is_intra_compound,
                is_to_eetru: i + 1 == last_sol_index,
                thalai_type,
                thalai_valid,
                thalai_detail,
            }
        })
        .collect()
}

/// Classify a junction between two words for venba analysis.
/// The thalai rule is determined by the FROM seer's category:
/// - Iyarseer vendalai: eerru ≠ muthal (Maamun Nirai / Vilamun Ner)
/// - Venseer vendalai: eerru = neer AND muthal = neer (Kaaimun Ner)
fn classify_thalai(
    from_cat: prosody::SeerCategory,
    _to_cat: prosody::SeerCategory,
    eerru: prosody::AsaiType,
    muthal: prosody::AsaiType,
    is_intra_compound: bool,
) -> (ThalaiType, bool, Option<String>) {
    if is_intra_compound {
        return (
            ThalaiType::IntraCompound,
            true,
            Some("Junction within compound word — skipped".to_string()),
        );
    }

    let eerru_s = eerru.as_str();
    let muthal_s = muthal.as_str();

    match from_cat {
        prosody::SeerCategory::Iyarseer => {
            // Iyarseer vendalai: eerru and muthal must differ
            let valid = eerru != muthal;
            let detail = format!(
                "{}→{}: {} iyarseer vendalai",
                eerru_s,
                muthal_s,
                if valid { "valid" } else { "invalid" }
            );
            (ThalaiType::IyarseerVendalai, valid, Some(detail))
        }
        prosody::SeerCategory::Venseer => {
            // Venseer vendalai (KaaimunNer): both must be neer
            let valid = eerru == prosody::AsaiType::Neer && muthal == prosody::AsaiType::Neer;
            let detail = format!(
                "{}→{}: {} venseer vendalai (KaaimunNer)",
                eerru_s,
                muthal_s,
                if valid { "valid" } else { "invalid" }
            );
            (ThalaiType::VenseerVendalai, valid, Some(detail))
        }
        prosody::SeerCategory::Overflow => {
            let detail = format!("{}→{}: overflow seer junction", eerru_s, muthal_s);
            (ThalaiType::Unknown, false, Some(detail))
        }
    }
}

/// Compute eetru (final) word data for workflow rules.
fn build_eetru_sol(sorkal: &[SolData]) -> EetruSolData {
    match sorkal.last() {
        Some(last) => {
            let is_kutrilugaram = last
                .kadai_ezhuthu
                .as_deref()
                .map(unicode::is_kutrilugaram_ending)
                .unwrap_or(false);
            let eetru_type = classify_eetru_type(last.asai_count, last.seer_eerru);
            EetruSolData {
                asai_count: last.asai_count,
                seer_eerru: last.seer_eerru,
                kadai_ezhuthu_mei: last.kadai_ezhuthu_mei.clone(),
                kadai_ezhuthu_alavu: last.kadai_ezhuthu_alavu,
                seer_category: last.seer_category,
                is_kutrilugaram,
                eetru_type,
            }
        }
        None => EetruSolData {
            asai_count: 0,
            seer_eerru: prosody::AsaiType::Neer,
            kadai_ezhuthu_mei: None,
            kadai_ezhuthu_alavu: None,
            seer_category: prosody::SeerCategory::Overflow,
            is_kutrilugaram: false,
            eetru_type: EetruType::Overflow,
        },
    }
}

/// Classify the eetru (final word) type based on asai count and last asai type.
fn classify_eetru_type(asai_count: usize, seer_eerru: prosody::AsaiType) -> EetruType {
    match (asai_count, seer_eerru) {
        (1, prosody::AsaiType::Neer) => EetruType::Naal,
        (1, prosody::AsaiType::Nirai) => EetruType::Malar,
        (2, prosody::AsaiType::Neer) => EetruType::Kaasu,
        (2, prosody::AsaiType::Nirai) => EetruType::Pirappu,
        _ => EetruType::Overflow,
    }
}

/// Compute ornamentation data from the original (pre-expansion) word list.
fn compute_ani(sorkal: &[SolData], lines: &[&str]) -> AniData {
    if lines.len() < 2 {
        return AniData {
            etukai_present: false,
            etukai_detail: None,
            monai_present: false,
            monai_detail: None,
            iyaipu_present: false,
            iyaipu_detail: None,
        };
    }

    let mut line0: Vec<usize> = Vec::new();
    let mut line1: Vec<usize> = Vec::new();
    for (i, s) in sorkal.iter().enumerate() {
        match s.adi_index {
            0 => line0.push(i),
            1 => line1.push(i),
            _ => {}
        }
    }

    // Etukai: 2nd grapheme consonant base of first word of each line
    let (etukai_present, etukai_detail) = match (line0.first(), line1.first()) {
        (Some(&w0), Some(&w4)) => etukai_match(&sorkal[w0], &sorkal[w4]),
        _ => (
            false,
            Some("Insufficient words for etukai comparison".to_string()),
        ),
    };

    // Monai: first-letter alliteration (Word[0] vs Word[2], fallback Word[0] vs Word[1])
    let (monai_present, monai_detail) = if line0.len() >= 3 {
        let (m1, d1) = monai_match(&sorkal[line0[0]], &sorkal[line0[2]], 1, 3);
        if m1 {
            (m1, d1)
        } else {
            monai_match(&sorkal[line0[0]], &sorkal[line0[1]], 1, 2)
        }
    } else if line0.len() >= 2 {
        monai_match(&sorkal[line0[0]], &sorkal[line0[1]], 1, 2)
    } else {
        (
            false,
            Some("Insufficient words for monai comparison".to_string()),
        )
    };

    // Iyaipu: end sound of last word of each line
    let (iyaipu_present, iyaipu_detail) = match (line0.last(), line1.last()) {
        (Some(&w3), Some(&w6)) => iyaipu_match(&sorkal[w3], &sorkal[w6]),
        _ => (
            false,
            Some("Insufficient words for iyaipu comparison".to_string()),
        ),
    };

    AniData {
        etukai_present,
        etukai_detail,
        monai_present,
        monai_detail,
        iyaipu_present,
        iyaipu_detail,
    }
}

/// Etukai: compare the consonant of the 2nd grapheme (ezhuthu) of each word.
fn etukai_match(w0: &SolData, w4: &SolData) -> (bool, Option<String>) {
    if w0.ezhuthukkal.len() < 2 || w4.ezhuthukkal.len() < 2 {
        return (
            false,
            Some("Word too short for etukai (needs 2+ graphemes)".to_string()),
        );
    }
    let g0 = &w0.ezhuthukkal[1];
    let g4 = &w4.ezhuthukkal[1];
    let matched = match (&g0.mei, &g4.mei) {
        (Some(m0), Some(m4)) => m0 == m4,
        (None, None) => g0.text == g4.text,
        _ => false,
    };
    let detail = format!(
        "2nd grapheme: {} vs {} — {}",
        g0.text,
        g4.text,
        if matched { "match" } else { "no match" }
    );
    (matched, Some(detail))
}

/// Monai: compare first-letter monai kurippu (consonant or vowel group).
fn monai_match(w1: &SolData, w2: &SolData, pos1: usize, pos2: usize) -> (bool, Option<String>) {
    match (
        &w1.muthal_ezhuthu_monai_kurippu,
        &w2.muthal_ezhuthu_monai_kurippu,
    ) {
        (Some(a), Some(b)) => {
            let matched = a == b;
            let detail = format!(
                "Word {} & {} first letter group: {} vs {} — {}",
                pos1,
                pos2,
                a,
                b,
                if matched { "match" } else { "no match" }
            );
            (matched, Some(detail))
        }
        _ => (
            false,
            Some(format!("Word {} or {} missing monai kurippu", pos1, pos2)),
        ),
    }
}

/// Iyaipu: compare the vowel length (alavu) of the last syllable of the last
/// word in each line. Matching kuril-kuril or nedil-nedil indicates iyaipu.
fn iyaipu_match(w3: &SolData, w6: &SolData) -> (bool, Option<String>) {
    match (w3.syllables.last(), w6.syllables.last()) {
        (Some(s3), Some(s6)) => {
            let matched = s3.alavu == s6.alavu;
            let a3 = s3.alavu.as_str();
            let a6 = s6.alavu.as_str();
            let detail = format!(
                "Line-end vowel length: {} vs {} — {}",
                a3,
                a6,
                if matched { "match" } else { "no match" }
            );
            (matched, Some(detail))
        }
        _ => (
            false,
            Some("Missing final syllable for iyaipu comparison".to_string()),
        ),
    }
}
