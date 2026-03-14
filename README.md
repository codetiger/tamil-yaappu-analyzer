# Tamil Yaappu Analyzer

A prosodic analysis and classification engine for Tamil verse. Given any Tamil text, it produces a rich structural breakdown — graphemes, syllables, metrical units (asai), feet (seer), inter-word junctions (thalai), ornamentation patterns (etukai, monai, iyaipu) — and classifies the verse into its prosody form.

The engine separates **linguistic analysis** from **classification logic**. The Rust preprocessor performs meter-agnostic Tamil prosodic analysis, producing structured `PaaData`. Classification and tagging are defined entirely in JSON workflow files using [JSONLogic](https://github.com/GoPlasmatic/datalogic-rs) rules, processed by the [dataflow-rs](https://github.com/GoPlasmatic/dataflow-rs) engine.

Classifies all four major Tamil verse forms:
- **Venba** — kural, sindhiyal, nerisai, innisai, pahrodai
- **Asiriyappa** — nerisai, nilaimandila
- **Kalippa**
- **Vanjippa** — kuraladi, chinthadi

Validated against **1,574+ verses** from classical Tamil literature including all 1,330 Thirukurals.

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

### Full Output Structure

For a complete verse, the engine produces:

**Prosodic data (`data.paa`)** — per-word and per-line breakdowns including graphemes, syllables, asai, seer classification, sandhi resolution, compound decomposition, junction (thalai) data with type/validity, and ornamentation (ani) with detail strings.

**Classification (`data.analysis.classification`)** — the determined verse form:
```json
{
  "primary_pa": "venba",
  "granularity_type": "kural_venba",
  "osai_type": "venba_osai",
  "is_valid": true
}
```

**Analysis tags (`data.analysis.tags`)** — boolean/string tags from all analysis layers covering seer patterns, thalai validity, line structure, and rhyme patterns.

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
          |      → Danda Strip                    |
          |        → Sandhi Resolve               |
          |          → Grapheme Extract            |
          |            → Syllabify                 |
          |              → Asai Classify (mora)    |
          |                → Seer Classify (foot)  |
          |                  → Ani Compute         |
          |                    → Compound Decompose|
          |                      → Thalai Data     |
          |                        → Eetru Classify|
          └───────────────────┬───────────────────┘
                              |
                           PaaData
                     (structured JSON)
                              |
          ┌───────────────────┴───────────────────┐
          |      ANALYSIS WORKFLOWS (JSON)        |
          |      5 layers, declarative rules      |
          |                                       |
          |  A1 Seer     — foot patterns & tags   |
          |  A2 Thalai   — junction validity      |
          |  A3 Adi      — line structure         |
          |  A4 Thodai   — rhyme & ornamentation  |
          |  A5 Classify — verse form & sub-type  |
          └───────────────────┬───────────────────┘
                              |
                     Classification + Tags
```

### Preprocessor Pipeline

The preprocessor is the core of the engine. It takes raw Tamil text and produces `PaaData` through these stages:

| Stage | What it does |
|-------|-------------|
| **NFC Normalize** | Canonical Unicode normalization for consistent character handling |
| **Script Validate** | Identifies non-Tamil characters |
| **Danda Strip** | Removes danda punctuation marks |
| **Sandhi Resolve** | Collapses pluti (extended) vowels; detects compound word boundaries |
| **Grapheme Extract** | Classifies each grapheme as uyir (vowel), mei (consonant), uyirmei (combined), or aytham |
| **Syllabify** | Groups graphemes into syllables with matrai (weight) computation |
| **Asai Classify** | Determines mora type — neer (long) or nirai (double) — respecting morpheme boundaries |
| **Seer Classify** | Groups 2-3 asais into named feet (12 vaaipadu patterns) |
| **Ani Compute** | Detects etukai (rhyme), monai (alliteration), iyaipu (end-rhyme) from original word positions |
| **Compound Decompose** | Splits overflow words (4+ asais) into valid prosodic sub-units |
| **Thalai Data** | Computes junction relationships between adjacent seer, with compound-awareness |
| **Eetru Classify** | Classifies the final word's ending pattern |

### Analysis Workflows

Classification and tagging are defined in JSON workflow files using JSONLogic, organized into 5 layers matching classical Tamil prosody theory:

| Layer | File | What it does |
|-------|------|-------------|
| **A1 Seer** | `a1_seer.json` | Enriches words with asai count, vaaippaadu, seer group, kutriyalukaram detection; computes eetru pattern and summary tags |
| **A2 Thalai** | `a2_thalai.json` | Computes thalai type and vendalai validity between consecutive words; summary tags for link harmony |
| **A3 Adi** | `a3_adi.json` | Enriches lines with word count, adi type, line position, thanichol detection; summary tags for line structure |
| **A4 Thodai** | `a4_thodai.json` | Computes rhyme identifiers, etukai, monai, iyaipu, vikarpam patterns |
| **A5 Classify** | `a5_classify.json` | Final classification: primary pa, osai type, granularity type, validity |

## Web UI

The engine compiles to WebAssembly for browser use. The WASM module exposes the full analysis pipeline, and a static frontend renders the results with line-based layout and evidence highlighting.

### Building the Web UI

```bash
cd wasm && wasm-pack build --target web --out-dir web/pkg  # Build WASM package
# Then serve the web/ directory with any static file server
```

## Extending to Other Prosody Forms

The preprocessor produces the same rich `PaaData` for any Tamil text. To add support for a new meter:

1. **Create new workflow JSON files** under `workflows/analysis/`
2. **Write JSONLogic rules** that reference `PaaData` fields — word counts, seer patterns, junction types, ornamentation
3. **Extend `a5_classify.json`** with classification logic for the new form

No Rust code changes required. The existing analysis workflows serve as a template.

## Test Corpus

| File | Count | Expected Classification | Source |
|------|-------|------------------------|--------|
| `kural.json` | 1,330 | `kural_venba` | Thirukural |
| `nerisai_venba.json` | 50 | `nerisai_venba` | Naladiyar, Nalavenba |
| `innisai_venba.json` | 50 | `innisai_venba` | Naladiyar, Nalavenba |
| `sindhiyal_venba.json` | 2 | `sindhiyal_venba` | Yappurungalakkarikkai |
| `nerisai_asiriyappa.json` | 50 | `nerisai_asiriyappa` | Kurunthokai |
| `nilaimandila_asiriyappa.json` | 12 | `nilaimandila_asiriyappa` | Kurunthokai |
| `kalippa.json` | 50 | `kalippa` | Kurunthokai |
| `vanjippa.json` | 30 | `vanjippa` | Naladiyar, Nalavenba |

Source texts obtained from [Project Madurai](https://www.projectmadurai.org/).

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
cargo build                            # Build
cargo run                              # Run with Kural #1
cargo run 42                           # Run with Kural #42
cargo test                             # Run all tests
cargo test --test classify_all_kurals  # Full 1330-kural validation
cargo test --test classify_all_verses  # All verse type classification tests
```

## References

See [REFERENCE.md](REFERENCE.md) for academic papers and data sources used in this project.

## License

Apache 2.0
