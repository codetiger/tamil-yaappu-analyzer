# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Tamil prosody **analyzer and classifier** for verse compositions. Analyzes Tamil text to produce word breakdowns, prosodic classifications, and analysis tags with reasoning. Built on the **dataflow-rs** workflow engine with **datalogic-rs** (JSONLogic) for declarative analysis rules.

## Build & Test Commands

```bash
cargo build                            # Build
cargo test                             # Run all tests (58 unit + 28 integration)
cargo test --lib                       # Unit tests only
cargo test --test integration_tests    # Integration tests only
cargo test --test classify_all_kurals  # Full 1330-kural validation (~15s)
cargo test --test classify_all_verses  # All verse type classification tests
cargo test test_name                   # Run a single test by name
cargo run                              # Run with Kural #1
cargo run 42                            # Run with Kural #42
RUST_LOG=debug cargo run               # Run with debug logging
```

### WASM Build (for web UI)

```bash
cd wasm && wasm-pack build --target web --out-dir web/pkg  # Build WASM package
# Then serve web/ directory with any static file server
```

## Architecture

### Pipeline (Preprocessor → Analysis Workflows)

The Rust `Preprocessor` is **meter-agnostic** — it enriches raw Tamil text into structured prosodic data (`PaaData`). Classification and tagging live in JSON `map` workflow files processed by `dataflow-rs`.

**Preprocessor pipeline** (`src/preprocessor.rs`):
NFC normalize → script validate → danda strip → **sandhi resolve** → grapheme extract → syllabify → asai classify → seer classify → **ani compute** (with detail strings) → **compound decompose** → junction (thalai) data (with type/validity) → eetru classification

**Analysis workflow layers** (5 layers matching classical prosody theory, all use `map` function):
1. `workflows/preprocessor.json` — calls the Rust preprocessor custom function
2. `workflows/analysis/a1_seer.json` — Seer (meter): enriches words with asai_count, vaaippaadu, seer_group, is_kutriyalukaram; computes eetru pattern and summary tags (has_overflow, has_kani_seer)
3. `workflows/analysis/a2_thalai.json` — Thalai (linkage): computes thalai_from_prev and is_ventalai between consecutive words; summary tags (thalai_all_valid, thalai_types, link_harmony)
4. `workflows/analysis/a3_adi.json` — Adi (line structure): enriches lines with word_count, adi_type, line_position, is_standard_venba_line, has_thanichol; summary tags (sol_per_adi, valid_tamil)
5. `workflows/analysis/a4_thodai.json` — Thodai (rhyme & pattern): computes rhyme_id_list, etukai, monai, is_iyaipu, vikarpam_count/type
6. `workflows/analysis/a5_classify.json` — Final Pa (classification): primary_pa (venba/asiriyappa/kalippa/vanjippa), osai_type, granularity_type, is_valid

**Output structure:**
- `data.paa` — Full prosodic breakdown with enriched word/line data
- `data.analysis.classification` — primary_pa, osai_type, granularity_type, is_valid
- `data.analysis.tags` — Boolean/string tags for all analysis layers

### Key Design Patterns

- **Separation of concerns**: Rust handles Tamil linguistic analysis; JSON `map` workflows handle classification and tagging. To add/modify analysis rules, edit workflow JSON files. To fix linguistic analysis, edit Rust code.
- **Compound word handling**: Sandhi resolution and compound decomposition expand words for prosodic analysis, but ornamentation (ani) is computed from **pre-expansion** word positions.
- **Thalai analysis**: Junction type and validity computed in workflow a2 via JSONLogic reduce, matching the 8 thalai mappings from classical prosody.
- **Classification framework**: Classifies Venba (with sub-types: kural, sindhiyal, 4-line, pahrodai), Asiriyappa (nerisai, nilaimandila), Vanjippa (kuraladi, chinthadi), and Kalippa. Venba check runs first (strictest). Extend a5_classify.json for new rules.

### Tamil Modules (`src/tamil/`)

- `unicode.rs` — Tamil Unicode character classification (vowels, consonants, vowel signs, lengths)
- `grapheme.rs` — Tamil grapheme cluster extraction and classification
- `syllable.rs` — Syllabification (splitting graphemes into syllables)
- `sandhi.rs` — Pluti vowel resolution + compound boundary detection
- `prosody.rs` — Asai (mora) classification, seer (foot) types, 12 vaaipadu names
- `compound.rs` — Compound word decomposition via asai-boundary exhaustive search

### Data Model (`src/types.rs`)

`PaaData` is the central enriched structure containing: raw input, word/line data, seer classifications, junction (thalai) data with type/validity, ornamentation (ani) data with detail strings, and eetru classification.

### WASM / Web UI (`wasm/`, `web/`)

The `wasm/` crate wraps the analysis engine for browser use via `wasm-bindgen`. `TamilProsodyEngine` exposes a `process(input)` method returning the full dataflow Message as JSON. The `web/` directory contains the static frontend (HTML/CSS/JS) that calls the WASM module. Key JS files: `app.js` (main app), `data-mapper.js` (maps engine output to UI), `evidence.js` (highlighting), `tamil-terms.js` (Tamil terminology labels).

### Build Script (`build.rs`)

Forces recompilation when workflow JSON files change (they're embedded via `include_str!`). **Note:** The filenames listed in `build.rs` are out of date — they reference old workflow names (a1_counts, a2_structural, etc.) that no longer match the actual files in `workflows/analysis/`.

## Test Data (`tests/data/`)

Test corpus organized by expected classification type:

| File | Count | Expected Classification | Source |
|------|-------|------------------------|--------|
| `kural.json` | 1330 | `kural_venba` | Thirukural |
| `nerisai_venba.json` | 50 | `nerisai_venba` | Naladiyar, Nalavenba |
| `innisai_venba.json` | 50 | `innisai_venba` | Naladiyar, Nalavenba |
| `sindhiyal_venba.json` | 2 | `sindhiyal_venba` | Yappurungalakkarikkai |
| `nerisai_asiriyappa.json` | 50 | `nerisai_asiriyappa` | Kurunthokai |
| `nilaimandila_asiriyappa.json` | 12 | `nilaimandila_asiriyappa` | Kurunthokai |
| `kalippa.json` | 50 | `kalippa` | Kurunthokai |
| `vanjippa.json` | 30 | `vanjippa` | Naladiyar, Nalavenba |

Source texts obtained from Project Madurai (projectmadurai.org).
