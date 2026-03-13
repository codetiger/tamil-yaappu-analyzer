pub mod preprocessor;
pub mod tamil;
pub mod types;

pub use preprocessor::Preprocessor;
pub use types::PaaData;

use dataflow_rs::engine::{Engine, Workflow, functions::AsyncFunctionHandler};
use std::collections::HashMap;

pub fn create_engine() -> Engine {
    let preprocess_wf =
        Workflow::from_json(include_str!("../workflows/preprocessor.json")).unwrap();
    let a1_counts_wf =
        Workflow::from_json(include_str!("../workflows/analysis/a1_counts.json")).unwrap();
    let a2_structural_wf =
        Workflow::from_json(include_str!("../workflows/analysis/a2_structural.json")).unwrap();
    let a3_seer_wf =
        Workflow::from_json(include_str!("../workflows/analysis/a3_seer.json")).unwrap();
    let a4_thalai_wf =
        Workflow::from_json(include_str!("../workflows/analysis/a4_thalai.json")).unwrap();
    let a5_ornamentation_wf =
        Workflow::from_json(include_str!("../workflows/analysis/a5_ornamentation.json")).unwrap();
    let a6_classify_wf =
        Workflow::from_json(include_str!("../workflows/analysis/a6_classify.json")).unwrap();

    let mut custom_fns: HashMap<String, Box<dyn AsyncFunctionHandler + Send + Sync>> =
        HashMap::new();
    custom_fns.insert("preprocessor".to_string(), Box::new(Preprocessor));

    Engine::new(
        vec![
            preprocess_wf,
            a1_counts_wf,
            a2_structural_wf,
            a3_seer_wf,
            a4_thalai_wf,
            a5_ornamentation_wf,
            a6_classify_wf,
        ],
        Some(custom_fns),
    )
}
