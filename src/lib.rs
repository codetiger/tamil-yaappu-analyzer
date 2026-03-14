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
    let a1_seer_wf =
        Workflow::from_json(include_str!("../workflows/analysis/a1_seer.json")).unwrap();
    let a2_thalai_wf =
        Workflow::from_json(include_str!("../workflows/analysis/a2_thalai.json")).unwrap();
    let a3_adi_wf = Workflow::from_json(include_str!("../workflows/analysis/a3_adi.json")).unwrap();
    let a4_thodai_wf =
        Workflow::from_json(include_str!("../workflows/analysis/a4_thodai.json")).unwrap();
    let a5_classify_wf =
        Workflow::from_json(include_str!("../workflows/analysis/a5_classify.json")).unwrap();

    let mut custom_fns: HashMap<String, Box<dyn AsyncFunctionHandler + Send + Sync>> =
        HashMap::new();
    custom_fns.insert("preprocessor".to_string(), Box::new(Preprocessor));

    Engine::new(
        vec![
            preprocess_wf,
            a1_seer_wf,
            a2_thalai_wf,
            a3_adi_wf,
            a4_thodai_wf,
            a5_classify_wf,
        ],
        Some(custom_fns),
    )
}
