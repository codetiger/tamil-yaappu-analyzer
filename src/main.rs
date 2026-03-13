use dataflow_rs::engine::message::Message;
use serde_json::json;
use std::sync::Arc;

use tamil_yaappu_analyzer::{PaaData, create_engine};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let engine = create_engine();

    // Sample Kural #1
    let kural = "அகர முதல எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு";
    println!("Input: {}\n", kural);

    let mut message = Message::new(Arc::new(json!({})));
    message.context["data"]["input"] = json!(kural);

    engine.process_message(&mut message).await?;

    // Print Classification
    if let Some(analysis) = message.data().get("analysis") {
        if let Some(classification) = analysis.get("classification") {
            println!("=== Classification ===");
            if let Some(family) = classification.get("paa_family").and_then(|v| v.as_str()) {
                println!("  Paa Family:  {}", family);
            }
            if let Some(vtype) = classification.get("venba_type").and_then(|v| v.as_str()) {
                println!("  Venba Type:  {}", vtype);
            }
            println!();
        }

        if let Some(tags) = analysis.get("tags").and_then(|t| t.as_object()) {
            println!("=== Analysis Tags ===");
            for (key, value) in tags {
                match value {
                    serde_json::Value::Bool(b) => {
                        println!("  {:<24} {}", key, if *b { "yes" } else { "no" });
                    }
                    serde_json::Value::String(s) => {
                        println!("  {:<24} {}", key, s);
                    }
                    serde_json::Value::Array(arr) => {
                        let items: Vec<String> = arr
                            .iter()
                            .map(|v| {
                                v.as_str()
                                    .map(|s| s.to_string())
                                    .or_else(|| v.as_u64().map(|n| n.to_string()))
                                    .unwrap_or_else(|| v.to_string())
                            })
                            .collect();
                        println!("  {:<24} [{}]", key, items.join(", "));
                    }
                    _ => {
                        println!("  {:<24} {}", key, value);
                    }
                }
            }
            println!();
        }
    }

    // Print Word Breakdown
    if let Some(paa_value) = message.data().get("paa") {
        let paa: PaaData = serde_json::from_value(paa_value.clone())?;
        println!("=== Word Breakdown ===");
        for (i, sol) in paa.sorkal.iter().enumerate() {
            let asai_tamil = sol
                .asaikal
                .iter()
                .map(|a| a.vagai.to_string())
                .collect::<Vec<_>>()
                .join("/");

            println!(
                "  {:>2}. {:<16} {:<16} {} ({})",
                i + 1,
                sol.raw_text,
                asai_tamil,
                sol.seer_vagai,
                sol.seer_category
            );
        }
    }

    Ok(())
}
