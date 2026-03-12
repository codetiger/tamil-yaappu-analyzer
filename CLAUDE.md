# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Compiler-style validator for Thirukural (Tamil verse) compositions. Checks 7-word Tamil verse against structural, prosodic, thalai (junction), ornamentation, and semantic rules. Built on the **dataflow-rs** workflow engine with **datalogic-rs** (JSONLogic) for declarative validation rules.

## Build & Test Commands

```bash
cargo build                        # Build
cargo test                         # Run all tests (52 unit + 52 integration)
cargo test --lib                   # Unit tests only
cargo test --test integration_tests # Integration tests only
cargo test test_name               # Run a single test by name
cargo run                          # Run with sample Kural #1
RUST_LOG=debug cargo run           # Run with debug logging
```

**Note:** Depends on a local `dataflow-rs` crate at `../Plasmatic/dataflow-rs`.

## Architecture

### Pipeline (Preprocessor → Workflow Rules)

The Rust `Preprocessor` is **meter-agnostic** — it enriches raw Tamil text into structured prosodic data. Meter-specific validation lives in JSON workflow files processed by `dataflow-rs`.

**Preprocessor pipeline** (`src/preprocessor.rs`):
NFC normalize → script validate → danda strip → **sandhi resolve** → grapheme extract → syllabify → asai classify → seer classify → **ani compute** → **compound decompose** → junction (thalai) data

**Workflow layers** (processed in priority order):
1. `workflows/preprocessor.json` — calls the Rust preprocessor custom function
2. `workflows/venba/kural/l1_structural.json` — word count, line count rules
3. `workflows/venba/l2_seer.json` — syllabic meter (seer) validation
4. `workflows/venba/l3_vendalai.json` — junction (thalai/vendalai) rules with `is_intra_compound` filtering
5. `workflows/venba/l4_ornamentation.json` — etukai/monai/iyaipu (alliteration/rhyme)

### Key Design Patterns

- **Separation of concerns**: Rust handles Tamil linguistic analysis; JSON handles rule evaluation. To add/modify validation rules, edit workflow JSON files. To fix linguistic analysis, edit Rust code.
- **Compound word handling**: Sandhi resolution and compound decomposition expand words for prosodic analysis, but ornamentation (ani) is computed from **pre-expansion** word positions.
- **L3/L4 rules** use `W_`/`I_` severity (warnings/info), not `E_` (errors).
- **L3 vendalai rules** filter on `is_intra_compound == false` to skip junctions within compound sub-units.

### Tamil Modules (`src/tamil/`)

- `unicode.rs` — Tamil Unicode character classification (vowels, consonants, vowel signs, lengths)
- `grapheme.rs` — Tamil grapheme cluster extraction and classification
- `syllable.rs` — Syllabification (splitting graphemes into syllables)
- `sandhi.rs` — Pluti vowel resolution + compound boundary detection
- `prosody.rs` — Asai (mora) classification, seer (foot) types, 12 vaaipadu names
- `compound.rs` — Compound word decomposition via asai-boundary exhaustive search

### Data Model (`src/types.rs`)

`PaaData` is the central enriched structure containing: raw input, word/line data, seer classifications, junction (thalai) data, ornamentation (ani) data, and diagnostics.

## Corpus

`kural.json` contains all 1330 Thirukurals used for integration testing. All must pass with zero `E_` errors.
