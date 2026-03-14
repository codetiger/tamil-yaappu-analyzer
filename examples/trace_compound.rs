use tamil_yaappu_analyzer::tamil::{compound, grapheme, prosody, syllable};

fn trace(word: &str) {
    let gs = grapheme::extract_graphemes(word);
    let syls = syllable::syllabify(&gs);
    let asai = prosody::classify_asai(&syls);
    let seer = prosody::classify_seer(&asai);
    println!(
        "{}: {} asai {:?} ({:?})",
        word,
        asai.len(),
        asai.iter()
            .map(|a| format!("{:?}({})", a.vagai, a.text))
            .collect::<Vec<_>>(),
        seer.seer_category
    );
    if let Some(parts) = compound::decompose_compound(word) {
        println!("  Compound split: {:?}", parts);
    } else {
        println!("  No decomposition needed");
    }
}

fn main() {
    trace("வீடுநிறைந்த");
    trace("நியதிக்குட்பட்டு");
    trace("உடைமையுள்");
}
