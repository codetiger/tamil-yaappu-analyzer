pub mod preprocessor;
pub mod tamil;
pub mod types;

pub use preprocessor::Preprocessor;
pub use types::PaaData;

use dataflow_rs::engine::{functions::AsyncFunctionHandler, Engine, Workflow};
use std::collections::HashMap;

pub fn create_engine() -> Engine {
    let preprocess_wf =
        Workflow::from_json(include_str!("../workflows/preprocessor.json")).unwrap();
    let kural_l1_wf =
        Workflow::from_json(include_str!("../workflows/venba/kural/l1_structural.json")).unwrap();
    let l2_seer_wf = Workflow::from_json(include_str!("../workflows/venba/l2_seer.json")).unwrap();
    let l3_vendalai_wf =
        Workflow::from_json(include_str!("../workflows/venba/l3_vendalai.json")).unwrap();
    let l4_ornamentation_wf =
        Workflow::from_json(include_str!("../workflows/venba/l4_ornamentation.json")).unwrap();

    let mut custom_fns: HashMap<String, Box<dyn AsyncFunctionHandler + Send + Sync>> =
        HashMap::new();
    custom_fns.insert("preprocessor".to_string(), Box::new(Preprocessor));

    Engine::new(
        vec![
            preprocess_wf,
            kural_l1_wf,
            l2_seer_wf,
            l3_vendalai_wf,
            l4_ornamentation_wf,
        ],
        Some(custom_fns),
    )
}
