# Tamil Yaappu Analyzer

A prosodic analysis engine for Tamil verse. Given any Tamil text, it produces a rich structural breakdown — graphemes, syllables, metrical units (asai), feet (seer), inter-word junctions (thalai), and ornamentation patterns (etukai, monai, iyaipu).

The engine separates **linguistic analysis** from **rule evaluation**. The preprocessor performs meter-agnostic Tamil prosodic analysis, producing a structured `PaaData` output. Meter-specific validation is defined entirely in JSON workflow files using [JSONLogic](https://github.com/GoPlasmatic/datalogic-rs) rules — supporting any Tamil prosody form (Venba, Aasiriyappa, Kalippa, Vanchippa, etc.) without code changes.

Ships with **Venba/Kural** rules as a reference implementation, validated against all 1330 Thirukurals.

## What It Produces

For a given Tamil verse, the preprocessor generates `PaaData` — a complete prosodic decomposition. Here's what it produces for the word **"அகர"**:

```json
{
  "raw_text": "அகர",
  "ezhuthukkal": [
    { "text": "அ",  "vagai": "uyir",    "mei": null, "alavu": "kuril" },
    { "text": "கர", "vagai": "uyirmei", "mei": "க",  "alavu": "kuril" }
  ],
  "syllables": [
    { "text": "அக", "alavu": "kuril", "is_closed": true,  "matrai": 2 },
    { "text": "ர",  "alavu": "kuril", "is_closed": false, "matrai": 1 }
  ],
  "asaikal": [
    { "vagai": "nirai", "text": "அக" },
    { "vagai": "neer",  "text": "ர"  }
  ],
  "asai_amaivu": "nirai_neer",
  "seer_vagai": "pulima",
  "seer_category": "iyarseer"
}
```

Each word goes through: **raw text → graphemes (ezhuthukkal) → syllables → asai (mora) → seer (foot)**.

### Full PaaData Structure

For a complete verse, PaaData contains:

**Per-word analysis (`sorkal`)** — graphemes, syllables, asai, seer classification, sandhi resolution, compound decomposition markers

**Per-line aggregation (`adikal`)** — seer sequence, syllable/matrai totals, word indices
```json
{
  "text": "அகர முதல எழுத்தெல்லாம் ஆதி",
  "seer_vagaikal": ["pulima", "pulima", "pulimangai", "thema"],
  "logical_sol_count": 4,
  "matrai_total": 18
}
```

**Inter-word junctions (`thalaikal`)** — metrical relationship between each adjacent word pair
```json
{
  "from_sol_index": 0,
  "to_sol_index": 1,
  "from_seer_category": "iyarseer",
  "to_seer_category": "iyarseer",
  "eerru_asai": "neer",
  "muthal_asai": "nirai",
  "is_cross_adi": false,
  "is_intra_compound": false
}
```

**Ornamentation (`ani`)** — rhyme and alliteration patterns detected across the verse
```json
{
  "etukai_present": false,
  "monai_present": false,
  "iyaipu_present": false
}
```

**Final word metrics (`eetru_sol`)** — asai count, ending pattern, vowel length, seer category

## Architecture

The system is built on the [dataflow-rs](https://github.com/GoPlasmatic/dataflow-rs) workflow engine with two distinct layers:

```
                        Tamil Verse Input
                              |
          ┌───────────────────┴───────────────────┐
          |        PREPROCESSOR (Rust)            |
          |        Meter-agnostic analysis        |
          |                                       |
          |  NFC Normalize                        |
          |    → Script Validate                  |
          |      → Sandhi Resolve                 |
          |        → Grapheme Extract             |
          |          → Syllabify                  |
          |            → Asai Classify (mora)     |
          |              → Seer Classify (foot)   |
          |                → Ani Compute          |
          |                  → Compound Decompose |
          |                    → Thalai Data      |
          └───────────────────┬───────────────────┘
                              |
                           PaaData
                     (structured JSON)
                              |
          ┌───────────────────┴───────────────────┐
          |      VALIDATION RULES (JSON)          |
          |      Meter-specific, swappable        |
          |                                       |
          |  Rules access PaaData fields via      |
          |  JSONLogic expressions like:          |
          |    data.paa.sorkal[*].seer_category   |
          |    data.paa.thalaikal[*].eerru_asai   |
          |    data.paa.ani.etukai_present        |
          └───────────────────┬───────────────────┘
                              |
                        Diagnostics
                    (errors / warnings / info)
```

### Preprocessor Pipeline

The preprocessor is the core of the engine. It takes raw Tamil text and produces `PaaData` through these stages:

| Stage | What it does |
|-------|-------------|
| **NFC Normalize** | Canonical Unicode normalization for consistent character handling |
| **Script Validate** | Identifies non-Tamil characters |
| **Sandhi Resolve** | Collapses pluti (extended) vowels; detects compound word boundaries |
| **Grapheme Extract** | Classifies each grapheme as uyir (vowel), mei (consonant), uyirmei (combined), or aytham |
| **Syllabify** | Groups graphemes into syllables with matrai (weight) computation |
| **Asai Classify** | Determines mora type — neer (long) or nirai (double) — respecting morpheme boundaries |
| **Seer Classify** | Groups 2-3 asais into named feet (12 vaaipadu patterns) |
| **Ani Compute** | Detects etukai (rhyme), monai (alliteration), iyaipu (end-rhyme) from original word positions |
| **Compound Decompose** | Splits overflow words (4+ asais) into valid prosodic sub-units |
| **Thalai Data** | Computes junction relationships between adjacent seer, with compound-awareness |

### Validation Rules

Rules are defined in JSON workflow files and evaluated by the [dataflow-rs](https://github.com/GoPlasmatic/dataflow-rs) engine using [datalogic-rs](https://github.com/GoPlasmatic/datalogic-rs) (JSONLogic). Each rule accesses `PaaData` fields and emits diagnostics at three severity levels:

- **`E_`** (Error) — structural violations that break the meter
- **`W_`** (Warning) — prosodic deviations worth noting
- **`I_`** (Info) — optional ornamentation observations

The included Venba/Kural ruleset validates across four layers:

| Layer | What it checks |
|-------|----------------|
| **L1 Structural** | Word count, line count, line split, Tamil script, empty words |
| **L2 Seer** | Final word pattern, seer overflow, kutrilugara ending, sandhi |
| **L3 Vendalai** | Junction rules between adjacent words (compound-aware filtering) |
| **L4 Ornamentation** | Etukai, monai, iyaipu detection |

## Extending to Other Prosody Forms

The preprocessor produces the same rich `PaaData` for any Tamil text. To add support for a new meter:

1. **Create a new workflow JSON file** under `workflows/` (e.g., `workflows/aasiriyappa/l1_structural.json`)
2. **Write JSONLogic rules** that reference `PaaData` fields — word counts, seer patterns, junction types, ornamentation
3. **Register the workflow** in the engine with a priority level

No Rust code changes required. The existing Venba/Kural rules under `workflows/venba/` serve as a template.

For example, Aasiriyappa rules would check for different line counts, different permitted seer combinations, and Aasiriya-specific junction (thalai) patterns — all expressible as JSONLogic over the same `PaaData` structure.

## Tamil Yaappu Concepts

| Term | Tamil | Meaning |
|------|-------|---------|
| **Asai** | அசை | Mora — smallest metrical unit (neer / nirai) |
| **Seer** | சீர் | Foot — group of 2-3 asais with a named pattern (12 vaaipadu) |
| **Thalai** | தளை | Junction — metrical relationship between adjacent seer |
| **Vendalai** | வெண்டளை | White-meter junction rules specific to Venba |
| **Etukai** | எதுகை | Second-letter rhyme between first words of each line |
| **Monai** | மோனை | First-letter alliteration between words |
| **Iyaipu** | இயைபு | End-rhyme between last words of each line |
| **Vaaipadu** | வாய்பாடு | Named metrical pattern (12 types: thema, pulima, koovilam, etc.) |
| **Matrai** | மாத்திரை | Rhythmic weight unit (kuril=1, nedil=2, closed adds +1) |

## Build & Run

Requires Rust toolchain and the [`dataflow-rs`](https://github.com/GoPlasmatic/dataflow-rs) crate.

```bash
cargo build           # Build
cargo run             # Run with sample verse
cargo test            # Run all tests
```

## References

See [REFERENCE.md](REFERENCE.md) for academic papers and data sources used in this project.

## License

Apache 2.0
