use dataflow_rs::engine::message::Message;
use serde_json::json;
use std::sync::Arc;
use tamil_yaappu_analyzer::create_engine;

async fn assert_all_classify_as(
    verses: &[String],
    expected_primary: &str,
    expected_granularity: &str,
    label: &str,
) {
    let engine = create_engine();
    let mut failures = Vec::new();

    for (i, verse) in verses.iter().enumerate() {
        let mut message = Message::new(Arc::new(json!({})));
        message.context["data"]["input"] = json!(verse);

        match engine.process_message(&mut message).await {
            Ok(_) => {
                let c = &message.data()["analysis"]["classification"];
                let primary = c["primary_pa"]["value"].as_str().unwrap_or("MISSING");
                let granularity = c["granularity_type"]["value"].as_str().unwrap_or("MISSING");
                let is_valid = c["is_valid"]["value"].as_bool().unwrap_or(false);

                if primary != expected_primary || granularity != expected_granularity || !is_valid {
                    let preview: String = verse.chars().take(30).collect();
                    failures.push(format!(
                        "  #{}: got pa={}, gran={}, valid={}\n    {}",
                        i + 1,
                        primary,
                        granularity,
                        is_valid,
                        preview
                    ));
                }
            }
            Err(e) => {
                failures.push(format!("  #{}: ERROR: {}", i + 1, e));
            }
        }
    }

    if !failures.is_empty() {
        panic!(
            "{}: {} of {} verses failed (expected {}/{}):\n{}",
            label,
            failures.len(),
            verses.len(),
            expected_primary,
            expected_granularity,
            failures.join("\n")
        );
    }
}

#[tokio::test]
async fn test_sindhiyal_venba() {
    let verses: Vec<String> =
        serde_json::from_str(include_str!("data/sindhiyal_venba.json")).unwrap();
    assert_all_classify_as(&verses, "venba", "sindhiyal_venba", "sindhiyal_venba").await;
}

#[tokio::test]
async fn test_nerisai_venba() {
    let verses: Vec<String> =
        serde_json::from_str(include_str!("data/nerisai_venba.json")).unwrap();
    assert_all_classify_as(&verses, "venba", "nerisai_venba", "nerisai_venba").await;
}

#[tokio::test]
async fn test_innisai_venba() {
    let verses: Vec<String> =
        serde_json::from_str(include_str!("data/innisai_venba.json")).unwrap();
    assert_all_classify_as(&verses, "venba", "innisai_venba", "innisai_venba").await;
}

#[tokio::test]
async fn test_nerisai_asiriyappa() {
    let verses: Vec<String> =
        serde_json::from_str(include_str!("data/nerisai_asiriyappa.json")).unwrap();
    assert_all_classify_as(
        &verses,
        "asiriyappa",
        "nerisai_asiriyappa",
        "nerisai_asiriyappa",
    )
    .await;
}

#[tokio::test]
async fn test_nilaimandila_asiriyappa() {
    let verses: Vec<String> =
        serde_json::from_str(include_str!("data/nilaimandila_asiriyappa.json")).unwrap();
    assert_all_classify_as(
        &verses,
        "asiriyappa",
        "nilaimandila_asiriyappa",
        "nilaimandila_asiriyappa",
    )
    .await;
}

#[tokio::test]
async fn test_kalippa() {
    let verses: Vec<String> = serde_json::from_str(include_str!("data/kalippa.json")).unwrap();
    assert_all_classify_as(&verses, "kalippa", "kalippa", "kalippa").await;
}

#[tokio::test]
async fn test_vanjippa() {
    let verses: Vec<String> = serde_json::from_str(include_str!("data/vanjippa.json")).unwrap();
    assert_all_classify_as(&verses, "vanjippa", "vanjippa", "vanjippa").await;
}
