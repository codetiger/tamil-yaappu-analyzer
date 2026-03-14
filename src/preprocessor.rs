use async_trait::async_trait;
use dataflow_rs::engine::{
    error::{DataflowError, Result},
    functions::{AsyncFunctionHandler, config::FunctionConfig},
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
        let _input = match config {
            FunctionConfig::Custom { input, .. } => input,
            _ => {
                return Err(DataflowError::Validation(
                    "Expected custom function config".to_string(),
                ));
            }
        };

        let raw_input = message.data()["input"]
            .as_str()
            .ok_or_else(|| DataflowError::Validation("Missing data.input string".to_string()))?
            .to_string();

        let paa = preprocess(&raw_input);

        let paa_value = serde_json::to_value(&paa)
            .map_err(|e| DataflowError::Validation(format!("Serialization error: {}", e)))?;

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

/// Process a single word through the prosodic pipeline.
fn process_word(raw_word: &str) -> SolData {
    let normalized = unicode::normalize_nfc(raw_word);
    let (text, _) = unicode::strip_non_tamil(&normalized);

    if text.is_empty() {
        return SolData {
            raw: raw_word.to_string(),
            muthal_ezhuthu: None,
            irandaam_ezhuthu: None,
            kadai_ezhuthu: None,
            kadai_alavu: None,
            asai_seq: vec![],
        };
    }

    let sandhi_result = sandhi::resolve(&text);
    let analysis_text = if sandhi_result.pluti_resolved || sandhi_result.kutriyalukaram_merged {
        &sandhi_result.phonological_text
    } else {
        &text
    };

    let graphemes = grapheme::extract_graphemes(analysis_text);
    let syllables = syllable::syllabify(&graphemes);
    let asaikal = prosody::classify_asai(&syllables);

    // muthal_ezhuthu: monai comparison key from first grapheme
    let muthal_ezhuthu = graphemes.first().and_then(|g| match g.vagai {
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

    // irandaam_ezhuthu: etukai comparison key from second grapheme's consonant
    let irandaam_ezhuthu = graphemes.get(1).map(|g| {
        g.mei
            .map(|c| c.to_string())
            .unwrap_or_else(|| g.text.clone())
    });

    // kadai_ezhuthu: last grapheme text
    let kadai_ezhuthu = graphemes.last().map(|g| g.text.clone());

    // kadai_alavu: vowel length of last syllable
    let kadai_alavu = syllables.last().map(|s| s.alavu.as_str().to_string());

    // asai_seq: sequence of asai types
    let asai_seq: Vec<String> = asaikal
        .iter()
        .map(|a| a.vagai.as_str().to_string())
        .collect();

    SolData {
        raw: raw_word.to_string(),
        muthal_ezhuthu,
        irandaam_ezhuthu,
        kadai_ezhuthu,
        kadai_alavu,
        asai_seq,
    }
}

/// Process a whitespace token, decomposing overflow compounds into sub-words.
/// Returns one SolData for normal words, multiple for decomposed compounds.
fn process_and_decompose(raw_word: &str) -> Vec<SolData> {
    let sol = process_word(raw_word);

    // Not overflow — return as-is
    if sol.asai_seq.len() <= 3 {
        return vec![sol];
    }

    // Compute the analysis text (same logic as process_word)
    let normalized = unicode::normalize_nfc(raw_word);
    let (text, _) = unicode::strip_non_tamil(&normalized);
    if text.is_empty() {
        return vec![sol];
    }
    let sandhi_result = sandhi::resolve(&text);
    let analysis_text = if sandhi_result.pluti_resolved || sandhi_result.kutriyalukaram_merged {
        &sandhi_result.phonological_text
    } else {
        &text
    };

    // Try compound decomposition first
    if let Some(parts) = compound::decompose_compound(analysis_text) {
        let results: Vec<SolData> = parts.iter().map(|p| process_word(p)).collect();
        if results
            .iter()
            .all(|s| !s.asai_seq.is_empty() && s.asai_seq.len() <= 3)
        {
            return results;
        }
    }

    // Try seer-based splitting as last resort
    if let Some(parts) = split_by_seer(analysis_text) {
        let results: Vec<SolData> = parts.iter().map(|p| process_word(p)).collect();
        if results
            .iter()
            .all(|s| !s.asai_seq.is_empty() && s.asai_seq.len() <= 3)
        {
            return results;
        }
    }

    // Couldn't decompose — return original
    vec![sol]
}

/// Split text into words using seer constraints (sliding window).
/// Groups asais into valid seers (2-asai iyarseer preferred, 3-asai venseer when needed).
fn split_by_seer(text: &str) -> Option<Vec<String>> {
    let graphemes = grapheme::extract_graphemes(text);
    let syllables = syllable::syllabify(&graphemes);
    let asaikal = prosody::classify_asai(&syllables);

    if asaikal.len() <= 3 {
        return None;
    }

    let total = asaikal.len();
    let mut parts = Vec::new();
    let mut i = 0;

    // If odd number of asais, take a 3-asai group first to make the rest even
    if total % 2 == 1 {
        parts.push(format!(
            "{}{}{}",
            asaikal[0].text, asaikal[1].text, asaikal[2].text
        ));
        i = 3;
    }

    // Take 2-asai groups for the rest
    while i + 1 < asaikal.len() {
        parts.push(format!("{}{}", asaikal[i].text, asaikal[i + 1].text));
        i += 2;
    }

    // Handle leftover single asai
    if i < asaikal.len() {
        parts.push(asaikal[i].text.clone());
    }

    if parts.len() > 1 { Some(parts) } else { None }
}

const KUTRIYALUKARAM_ENDINGS: [&str; 6] = ["கு", "சு", "டு", "து", "பு", "று"];

/// Cross-word kutriyalukaram elision.
/// When word_n ends with an open-kuril kutriyalukaram syllable (கு/சு/டு/து/பு/று)
/// and word_{n+1} starts with a Tamil vowel, the kutriyalukaram is metrically
/// silent. We strip it from the word text, reprocess to get the correct asai_seq,
/// and replace only asai_seq (keeping original metadata for ornamentation).
fn apply_cross_word_elision(sorkal: &mut [SolData]) {
    if sorkal.len() < 2 {
        return;
    }

    // Collect (index, new_asai_seq) pairs, then apply
    let mut updates: Vec<(usize, Vec<String>)> = Vec::new();

    for i in 0..sorkal.len() - 1 {
        // Word must end with a kutriyalukaram grapheme
        let ku_ending = match &sorkal[i].kadai_ezhuthu {
            Some(ke) if KUTRIYALUKARAM_ENDINGS.contains(&ke.as_str()) => ke.clone(),
            _ => continue,
        };

        // kadai_alavu must be kuril (open kuril syllable)
        if sorkal[i].kadai_alavu.as_deref() != Some("kuril") {
            continue;
        }

        // Next word must start with a Tamil vowel
        let next_starts_vowel = sorkal[i + 1]
            .raw
            .chars()
            .find(|c| c.is_alphabetic())
            .is_some_and(unicode::is_vowel);
        if !next_starts_vowel {
            continue;
        }

        // Strip kutriyalukaram ending from raw text and reprocess
        if let Some(stripped) = sorkal[i].raw.strip_suffix(&ku_ending) {
            if stripped.is_empty() {
                continue;
            }
            let new_sol = process_word(stripped);
            if !new_sol.asai_seq.is_empty() {
                updates.push((i, new_sol.asai_seq));
            }
        }
    }

    // Apply updates — only replace asai_seq, keep original metadata
    for (i, new_asai_seq) in updates {
        sorkal[i].asai_seq = new_asai_seq;
    }
}

pub fn preprocess(raw_input: &str) -> PaaData {
    let adikal: Vec<AdiData> = raw_input
        .split('\n')
        .map(|line| {
            let mut sorkal: Vec<SolData> = line
                .split_whitespace()
                .filter(|word| word.chars().any(|c| c.is_alphabetic()))
                .flat_map(process_and_decompose)
                .collect();

            // Cross-word kutriyalukaram elision pass
            apply_cross_word_elision(&mut sorkal);

            AdiData {
                raw: line.to_string(),
                sorkal,
            }
        })
        .collect();

    PaaData {
        raw: raw_input.to_string(),
        adikal,
    }
}
