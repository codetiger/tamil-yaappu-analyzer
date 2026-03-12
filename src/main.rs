use dataflow_rs::engine::{functions::AsyncFunctionHandler, message::Message, Engine, Workflow};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

use tamil_prosody_validator::Preprocessor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // 1. Load workflows
    let preprocess_wf = Workflow::from_json(include_str!("../workflows/preprocessor.json"))?;
    let kural_l1_wf =
        Workflow::from_json(include_str!("../workflows/venba/kural/l1_structural.json"))?;
    let l2_seer_wf = Workflow::from_json(include_str!("../workflows/venba/l2_seer.json"))?;
    let l3_vendalai_wf = Workflow::from_json(include_str!("../workflows/venba/l3_vendalai.json"))?;
    let l4_ornamentation_wf =
        Workflow::from_json(include_str!("../workflows/venba/l4_ornamentation.json"))?;

    // 2. Register custom functions
    let mut custom_fns: HashMap<String, Box<dyn AsyncFunctionHandler + Send + Sync>> =
        HashMap::new();
    custom_fns.insert("preprocessor".to_string(), Box::new(Preprocessor));

    // 3. Create engine (preprocessor priority 0, L1=1, L2=2, L3=3, L4=4)
    let engine = Engine::new(
        vec![
            preprocess_wf,
            kural_l1_wf,
            l2_seer_wf,
            l3_vendalai_wf,
            l4_ornamentation_wf,
        ],
        Some(custom_fns),
    );

    // 4. Sample Kural #1
    let kural = "அகர முதல எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு";
    println!("Input: {}\n", kural);

    let mut message = Message::new(Arc::new(json!({})));
    message.context["data"]["input"] = json!(kural);

    // 5. Process through all workflows
    engine.process_message(&mut message).await?;

    // 6. Print Alagiduthal Vaaipadu
    if let Some(paa) = message.data().get("paa") {
        if let Some(sorkal) = paa.get("sorkal").and_then(|s| s.as_array()) {
            println!("Alagiduthal Vaaipadu:");
            for (i, sol) in sorkal.iter().enumerate() {
                let text = sol.get("raw_text").and_then(|v| v.as_str()).unwrap_or("?");
                let asai_amaivu = sol
                    .get("asai_amaivu")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let seer_vagai = sol
                    .get("seer_vagai")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?");
                let seer_category = sol
                    .get("seer_category")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?");

                let asai_tamil = asai_amaivu
                    .split('_')
                    .map(|a| match a {
                        "neer" => "நேர்",
                        "nirai" => "நிரை",
                        _ => a,
                    })
                    .collect::<Vec<_>>()
                    .join("/");

                let seer_tamil = match seer_vagai {
                    "thema" => "தேமா",
                    "pulima" => "புளிமா",
                    "koovilam" => "கூவிளம்",
                    "karuvilam" => "கருவிளம்",
                    "themangai" => "தேமாங்காய்",
                    "themangani" => "தேமாங்கனி",
                    "koovilankai" => "கூவிளங்காய்",
                    "koovilankani" => "கூவிளங்கனி",
                    "pulimangai" => "புளிமாங்காய்",
                    "pulimangani" => "புளிமாங்கனி",
                    "karuvilangai" => "கருவிளங்காய்",
                    "karuvilankani" => "கருவிளங்கனி",
                    _ => seer_vagai,
                };

                let category_label = match seer_category {
                    "iyarseer" => "இயற்சீர்",
                    "venseer" => "வெண்சீர்",
                    _ => seer_category,
                };

                println!(
                    "  {:>2}. {:<16} {:<16} {} ({})",
                    i + 1,
                    text,
                    asai_tamil,
                    seer_tamil,
                    category_label
                );
            }
            println!();
        }
    }

    // 7. Print diagnostics
    if message.has_errors() {
        println!("Validation Errors:");
        for error in &message.errors {
            println!("  [{}] {}", error.code, error.message);
        }
    } else {
        println!("Thirukural is valid!");
    }

    Ok(())
}
