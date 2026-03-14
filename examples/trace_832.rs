use tamil_yaappu_analyzer::tamil::{grapheme, prosody, syllable};

fn trace(word: &str) {
    println!("=== {} ===", word);
    let gs = grapheme::extract_graphemes(word);
    println!(
        "Graphemes: {}",
        gs.iter()
            .map(|g| format!("{}({:?},{:?})", g.text, g.vagai, g.alavu))
            .collect::<Vec<_>>()
            .join(" ")
    );
    let syls = syllable::syllabify(&gs);
    println!(
        "Syllables: {}",
        syls.iter()
            .map(|s| format!(
                "{}({:?},{})",
                s.text,
                s.alavu,
                if s.is_closed { "c" } else { "o" }
            ))
            .collect::<Vec<_>>()
            .join(" ")
    );
    let asai = prosody::classify_asai(&syls);
    println!(
        "Asai: {}",
        asai.iter()
            .map(|a| format!("{:?}({})", a.vagai, a.text))
            .collect::<Vec<_>>()
            .join(" + ")
    );
    let seer = prosody::classify_seer(&asai);
    println!(
        "Seer: {:?} ({:?}), count={}\n",
        seer.seer_vagai, seer.seer_category, seer.asai_count
    );
}

fn main() {
    // The overflow word
    trace("பேதைமையுள்");
    // Break down the sub-parts
    trace("மையுள்");
    trace("மை");
    trace("யுள்");
    // Other words from this kural
    trace("எல்லாம்");
    trace("பேதைமை");
    trace("கையல்ல");
}
