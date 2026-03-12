use dataflow_rs::{Engine, Message};
use serde_json::json;
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

use tamil_prosody_validator::create_engine;

#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// A WebAssembly-compatible Tamil prosody analysis engine.
///
/// Wraps the analysis pipeline (preprocessor + A1-A5 classification/tagging)
/// and returns the complete dataflow Message as JSON.
#[wasm_bindgen]
pub struct TamilProsodyEngine {
    inner: Arc<Engine>,
}

#[wasm_bindgen]
impl TamilProsodyEngine {
    /// Create a new engine with all workflows and the Tamil preprocessor
    /// pre-loaded. Reuse this instance for multiple calls.
    #[wasm_bindgen(constructor)]
    pub fn new() -> TamilProsodyEngine {
        TamilProsodyEngine {
            inner: Arc::new(create_engine()),
        }
    }

    /// Run the full analysis pipeline (preprocessor + A1-A5 classification/tagging)
    /// on the given Tamil text and return the complete dataflow Message as JSON.
    ///
    /// The returned JSON contains:
    /// - context.data.paa: full prosodic breakdown (PaaData)
    /// - context.data.analysis: classification + tags with reasoning
    /// - audit_trail: workflow execution trace
    #[wasm_bindgen]
    pub fn process(&self, input: &str) -> js_sys::Promise {
        let engine = Arc::clone(&self.inner);
        let input = input.to_string();

        future_to_promise(async move {
            let mut message = Message::new(Arc::new(json!({})));
            message.context["data"]["input"] = json!(input);

            match engine.process_message(&mut message).await {
                Ok(()) => serde_json::to_string(&message)
                    .map(|s| JsValue::from_str(&s))
                    .map_err(|e| JsValue::from_str(&e.to_string())),
                Err(e) => Err(JsValue::from_str(&e.to_string())),
            }
        })
    }
}
