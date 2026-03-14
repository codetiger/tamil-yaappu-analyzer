use dataflow_rs::engine::message::Message;
use serde_json::json;
use std::sync::Arc;
use tamil_yaappu_analyzer::create_engine;

const KURAL_JSON: &str = include_str!("../kural.json");

#[tokio::test]
async fn test_all_kurals_classify_as_kural_venba() {
    let kurals: Vec<String> = serde_json::from_str(KURAL_JSON).unwrap();
    let engine = create_engine();

    let mut failures = Vec::new();

    for (i, kural) in kurals.iter().enumerate() {
        let mut message = Message::new(Arc::new(json!({})));
        message.context["data"]["input"] = json!(kural);

        match engine.process_message(&mut message).await {
            Ok(_) => {
                let classification = &message.data()["analysis"]["classification"];
                let primary_pa = classification["primary_pa"].as_str().unwrap_or("MISSING");
                let granularity = classification["granularity_type"]
                    .as_str()
                    .unwrap_or("MISSING");
                let is_valid = classification["is_valid"].as_bool().unwrap_or(false);

                if primary_pa != "venba" || granularity != "kural_venba" || !is_valid {
                    let data = message.data();
                    let mut detail = format!(
                        "Kural #{}: primary_pa={}, granularity={}, is_valid={}\n  Input: {}",
                        i + 1,
                        primary_pa,
                        granularity,
                        is_valid,
                        kural
                    );

                    // Word-level preprocessor info per line
                    if let Some(adikal) = data["paa"]["adikal"].as_array() {
                        for (li, adi) in adikal.iter().enumerate() {
                            let raw = adi["raw"].as_str().unwrap_or("");
                            let wc = adi["word_count"].as_u64().unwrap_or(0);
                            detail.push_str(&format!(
                                "\n  Line {}: \"{}\" (words: {})",
                                li + 1,
                                raw,
                                wc
                            ));

                            if let Some(sorkal) = adi["sorkal"].as_array() {
                                for (wi, sol) in sorkal.iter().enumerate() {
                                    let asai_seq = sol["asai_seq"]
                                        .as_array()
                                        .map(|a| {
                                            a.iter()
                                                .filter_map(|v| v.as_str())
                                                .collect::<Vec<_>>()
                                                .join(",")
                                        })
                                        .unwrap_or_default();
                                    detail.push_str(&format!(
                                        "\n    W{}: \"{}\" asai=[{}] count={} vaaippaadu={} seer={} kutri={} thalai={} ventalai={}",
                                        wi + 1,
                                        sol["raw"].as_str().unwrap_or(""),
                                        asai_seq,
                                        sol["asai_count"].as_u64().unwrap_or(0),
                                        sol["vaaippaadu"].as_str().unwrap_or("null"),
                                        sol["seer_group"].as_str().unwrap_or("null"),
                                        sol["is_kutriyalukaram"].as_bool().unwrap_or(false),
                                        sol["thalai_from_prev"].as_str().unwrap_or("null"),
                                        sol["is_ventalai"].as_bool().map(|b| b.to_string()).unwrap_or("null".into()),
                                    ));
                                }
                            }
                        }
                    }

                    // Summary tags
                    let tags = &data["analysis"]["tags"];
                    detail.push_str(&format!(
                        "\n  Tags: eetru={} overflow={} kani_seer={} thalai_valid={} sol_per_adi={} valid_tamil={}",
                        tags["eetru_type"].as_str().unwrap_or("null"),
                        tags["has_overflow"].as_bool().unwrap_or(false),
                        tags["has_kani_seer"].as_bool().unwrap_or(false),
                        tags["thalai_all_valid"].as_bool().unwrap_or(false),
                        tags["sol_per_adi"],
                        tags["valid_tamil"].as_bool().unwrap_or(false),
                    ));

                    failures.push(detail);
                }
            }
            Err(e) => {
                failures.push(format!("Kural #{}: ERROR: {}", i + 1, e));
            }
        }
    }

    if !failures.is_empty() {
        panic!(
            "{} out of 1330 kurals failed classification:\n{}",
            failures.len(),
            failures.join("\n")
        );
    }
}
