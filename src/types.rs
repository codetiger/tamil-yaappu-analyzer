use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::tamil::grapheme::GraphemeType;
use crate::tamil::prosody::{AsaiType, SeerCategory, SeerType};
use crate::tamil::unicode::VowelLength;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaaData {
    pub raw_input: String,
    pub original_sol_count: usize,
    pub eetru_sol: EetruSolData,
    pub ani: AniData,
    pub adikal: Vec<AdiData>,
    pub sorkal: Vec<SolData>,
    pub thalaikal: Vec<ThalaiData>,
    pub diagnostics: Vec<Value>,
}

/// Computed ornamentation (அணி) data — etukai, monai, iyaipu.
/// Computed from original (pre-compound-expansion) word positions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniData {
    pub etukai_present: bool,
    pub monai_present: bool,
    pub iyaipu_present: bool,
}

/// Computed eetru (final) word data for easy access in workflow rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EetruSolData {
    pub asai_count: usize,
    pub seer_eerru: AsaiType,
    pub kadai_ezhuthu_mei: Option<String>,
    pub kadai_ezhuthu_alavu: Option<VowelLength>,
    pub seer_category: SeerCategory,
    pub is_kutrilugaram: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdiData {
    pub text: String,
    pub sol_varisaikal: Vec<usize>,
    pub seer_vagaikal: Vec<SeerType>,
    pub logical_sol_count: usize,
    pub syllable_count_total: usize,
    pub matrai_total: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolData {
    pub adi_index: usize,
    pub adi_idanam: usize,
    pub raw_text: String,
    pub normalized_text: String,
    pub phonological_text: Option<String>,
    pub is_valid_script: bool,
    pub invalid_chars: Vec<String>,
    pub is_empty: bool,
    pub danda_stripped: bool,
    pub ezhuthukkal: Vec<EzhuthuData>,
    pub muthal_ezhuthu_monai_kurippu: Option<String>,
    pub kadai_ezhuthu: Option<String>,
    pub kadai_ezhuthu_mei: Option<String>,
    pub kadai_ezhuthu_alavu: Option<VowelLength>,
    pub kadai_ezhuthu_vagai: Option<GraphemeType>,
    pub syllables: Vec<SyllableData>,
    pub asaikal: Vec<AsaiData>,
    pub asai_amaivu: String,
    pub seer_vagai: SeerType,
    pub seer_category: SeerCategory,
    pub asai_count: usize,
    pub seer_muthal: AsaiType,
    pub seer_eerru: AsaiType,
    pub syllabification_failed: bool,
    pub ambiguous_asai: bool,
    pub has_compound_boundary: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compound_source_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compound_part: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compound_source_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EzhuthuData {
    pub text: String,
    pub vagai: GraphemeType,
    pub mei: Option<String>,
    pub alavu: Option<VowelLength>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyllableData {
    pub text: String,
    pub alavu: VowelLength,
    pub is_closed: bool,
    pub matrai: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsaiData {
    pub vagai: AsaiType,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThalaiData {
    pub from_sol_index: usize,
    pub to_sol_index: usize,
    pub from_seer_category: SeerCategory,
    pub to_seer_category: SeerCategory,
    pub eerru_asai: AsaiType,
    pub muthal_asai: AsaiType,
    pub is_cross_adi: bool,
    pub is_intra_compound: bool,
    pub is_to_eetru: bool,
}
