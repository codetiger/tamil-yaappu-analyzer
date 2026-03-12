use dataflow_rs::engine::message::Message;
use serde_json::json;
use std::sync::Arc;

use tamil_prosody_validator::{create_engine, PaaData};

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

    // Print Alagiduthal Vaaipadu
    if let Some(paa_value) = message.data().get("paa") {
        let paa: PaaData = serde_json::from_value(paa_value.clone())?;
        println!("Alagiduthal Vaaipadu:");
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
        println!();
    }

    // Print diagnostics
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
