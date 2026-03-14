use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaaData {
    pub raw: String,
    pub adikal: Vec<AdiData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdiData {
    pub raw: String,
    pub sorkal: Vec<SolData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolData {
    pub raw: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub muthal_ezhuthu: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub irandaam_ezhuthu: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kadai_ezhuthu: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kadai_alavu: Option<String>,
    pub asai_seq: Vec<String>,
}
