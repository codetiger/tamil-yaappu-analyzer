use dataflow_rs::engine::message::Message;
use serde_json::json;
use std::sync::Arc;

use tamil_yaappu_analyzer::create_engine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args: Vec<String> = std::env::args().collect();
    let index: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);

    let kurals: Vec<String> = serde_json::from_str(include_str!("../tests/data/kural.json"))?;

    if index < 1 || index > kurals.len() {
        eprintln!("Kural index must be between 1 and {}", kurals.len());
        std::process::exit(1);
    }

    let kural = &kurals[index - 1];

    let engine = create_engine();
    let mut message = Message::new(Arc::new(json!({})));
    message.context["data"]["input"] = json!(kural);

    engine.process_message(&mut message).await?;

    println!("{}", serde_json::to_string_pretty(message.data())?);

    Ok(())
}
