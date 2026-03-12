# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Tamil prosody **analyzer and classifier** for verse compositions. Analyzes Tamil text to produce word breakdowns, prosodic classifications, and analysis tags with reasoning. Built on the **dataflow-rs** workflow engine with **datalogic-rs** (JSONLogic) for declarative analysis rules.

## Build & Test Commands

```bash
cargo build                        # Build
cargo test                         # Run all tests (56 unit + 52 integration)
cargo test --lib                   # Unit tests only
cargo test --test integration_tests # Integration tests only
cargo test test_name               # Run a single test by name
cargo run                          # Run with sample Kural #1
RUST_LOG=debug cargo run           # Run with debug logging
```

**Note:** Depends on a local `dataflow-rs` crate at `../Plasmatic/dataflow-rs`.

## Architecture

### Pipeline (Preprocessor → Analysis Workflows)

The Rust `Preprocessor` is **meter-agnostic** — it enriches raw Tamil text into structured prosodic data (`PaaData`). Classification and tagging live in JSON `map` workflow files processed by `dataflow-rs`.

**Preprocessor pipeline** (`src/preprocessor.rs`):
NFC normalize → script validate → danda strip → **sandhi resolve** → grapheme extract → syllabify → asai classify → seer classify → **ani compute** (with detail strings) → **compound decompose** → junction (thalai) data (with type/validity) → eetru classification

**Analysis workflow layers** (processed in priority order, all use `map` function):
1. `workflows/preprocessor.json` — calls the Rust preprocessor custom function
2. `workflows/analysis/a1_classify.json` — paa family + venba sub-type classification with reasoning
3. `workflows/analysis/a2_structural.json` — structural tags (valid_tamil, no_empty_words, sol_per_adi)
4. `workflows/analysis/a3_seer.json` — seer/meter tags (eetru_type, kutrilugaram, overflow, seer_pattern)
5. `workflows/analysis/a4_thalai.json` — junction tags (thalai_all_valid, thalai_types)
6. `workflows/analysis/a5_ornamentation.json` — etukai/monai/iyaipu tags with detail strings

**Output structure:**
- `data.paa` — Full prosodic breakdown (PaaData)
- `data.analysis.classification` — paa_family, venba_type, reasoning
- `data.analysis.tags` — Boolean/string tags with detail strings

### Key Design Patterns

- **Separation of concerns**: Rust handles Tamil linguistic analysis; JSON `map` workflows handle classification and tagging. To add/modify analysis rules, edit workflow JSON files. To fix linguistic analysis, edit Rust code.
- **Compound word handling**: Sandhi resolution and compound decomposition expand words for prosodic analysis, but ornamentation (ani) is computed from **pre-expansion** word positions.
- **Thalai analysis**: Each junction has `thalai_type`, `thalai_valid`, and `thalai_detail` computed in Rust. Workflow a4 summarizes these into tags.
- **Classification framework**: Currently classifies Kural Venba (2 lines, 7 words, 4+3). Other types return "unknown". Extend a1_classify.json for new types.

### Tamil Modules (`src/tamil/`)

- `unicode.rs` — Tamil Unicode character classification (vowels, consonants, vowel signs, lengths)
- `grapheme.rs` — Tamil grapheme cluster extraction and classification
- `syllable.rs` — Syllabification (splitting graphemes into syllables)
- `sandhi.rs` — Pluti vowel resolution + compound boundary detection
- `prosody.rs` — Asai (mora) classification, seer (foot) types, 12 vaaipadu names
- `compound.rs` — Compound word decomposition via asai-boundary exhaustive search

### Data Model (`src/types.rs`)

`PaaData` is the central enriched structure containing: raw input, word/line data, seer classifications, junction (thalai) data with type/validity, ornamentation (ani) data with detail strings, and eetru classification.

## Corpus

`kural.json` contains all 1330 Thirukurals used for integration testing. All must classify as `kural_venba`.
