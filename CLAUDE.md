# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Tamil prosody **analyzer and classifier** for verse compositions. Pure library crate published on crates.io. Analyzes Tamil text to produce word breakdowns, prosodic classifications, and analysis tags with reasoning. Built on the **dataflow-rs** workflow engine with **datalogic-rs** (JSONLogic) for declarative analysis rules.

## Build & Test Commands

```bash
cargo build                            # Build library
cargo test                             # Run all tests (58 unit + 28 integration)
cargo test --lib                       # Unit tests only
cargo test --test integration_tests    # Integration tests only
cargo test --test classify_all_kurals  # Full 1330-kural validation (~15s)
cargo test --test classify_all_verses  # All verse type classification tests
cargo test test_name                   # Run a single test by name
cargo fmt --check                      # Check formatting
cargo clippy                           # Lint
cargo run --example analyze_kural          # Run with Kural #1
cargo run --example analyze_kural -- 42    # Run with Kural #42
RUST_LOG=debug cargo run --example analyze_kural  # Run with debug logging
```

### WASM Build (for web UI)

```bash
cd wasm && wasm-pack build --target web --out-dir web/pkg  # Build WASM package
# Then serve web/ directory with any static file server
```

## Architecture

### Crate Structure

Pure library crate (`src/lib.rs`) — no binary target. The old `main.rs` lives in `examples/analyze_kural.rs`. The `wasm/` subcrate wraps the library for browser use via `wasm-bindgen` (marked `publish = false`).

Public API exports: `Preprocessor`, `PaaData`, and `create_engine()`.

### Pipeline (Preprocessor → Analysis Workflows)

The Rust `Preprocessor` is **meter-agnostic** — it enriches raw Tamil text into structured prosodic data (`PaaData`). Classification and tagging live in JSON `map` workflow files processed by `dataflow-rs`.

**Preprocessor pipeline** (`src/preprocessor.rs`):
NFC normalize → script validate → danda strip → **sandhi resolve** → grapheme extract → syllabify → asai classify → seer classify → **ani compute** (with detail strings) → **compound decompose** → junction (thalai) data (with type/validity) → eetru classification

**Analysis workflow layers** (6 chained workflows, all use `map` function):
1. `workflows/preprocessor.json` — calls the Rust preprocessor custom function
2. `workflows/analysis/a1_seer.json` — Seer (meter): foot patterns, eetru, summary tags
3. `workflows/analysis/a2_thalai.json` — Thalai (linkage): junction validity between words
4. `workflows/analysis/a3_adi.json` — Adi (line structure): word counts, line types, thanichol
5. `workflows/analysis/a4_thodai.json` — Thodai (rhyme): etukai, monai, iyaipu, vikarpam
6. `workflows/analysis/a5_classify.json` — Final Pa: primary_pa, osai_type, granularity_type, is_valid

**Output structure:**
- `data.paa` — Full prosodic breakdown with enriched word/line data
- `data.analysis.classification` — primary_pa, osai_type, granularity_type, is_valid
- `data.analysis.tags` — Boolean/string tags for all analysis layers

### Key Design Patterns

- **Separation of concerns**: Rust handles Tamil linguistic analysis; JSON `map` workflows handle classification and tagging. To add/modify analysis rules, edit workflow JSON files. To fix linguistic analysis, edit Rust code.
- **Compound word handling**: Sandhi resolution and compound decomposition expand words for prosodic analysis, but ornamentation (ani) is computed from **pre-expansion** word positions.
- **Thalai analysis**: Junction type and validity computed in workflow a2 via JSONLogic reduce, matching the 8 thalai mappings from classical prosody.
- **Classification framework**: Classifies Venba (kural, sindhiyal, 4-line, pahrodai), Asiriyappa (nerisai, nilaimandila), Vanjippa (kuraladi, chinthadi), and Kalippa. Venba check runs first (strictest). Extend a5_classify.json for new rules.
- **Workflow embedding**: All JSON workflows are embedded at compile time via `include_str!()`. The `build.rs` script tracks these files for recompilation.

### Data Model (`src/types.rs`)

`PaaData` is the central enriched structure containing: raw input, word/line data, seer classifications, junction (thalai) data with type/validity, ornamentation (ani) data with detail strings, and eetru classification.

### WASM / Web UI (`wasm/`, `web/`)

The `wasm/` crate wraps the analysis engine for browser use via `wasm-bindgen`. `TamilProsodyEngine` exposes a `process(input)` method returning the full dataflow Message as JSON. The `web/` directory contains the static frontend (HTML/CSS/JS). GitHub Pages deployment is handled by `.github/workflows/deploy-pages.yml`.

## Test Data (`tests/data/`)

Test corpus of **1,574+ verses** organized by expected classification type (kural_venba, nerisai_venba, innisai_venba, sindhiyal_venba, nerisai_asiriyappa, nilaimandila_asiriyappa, kalippa, vanjippa). Source texts from Project Madurai (projectmadurai.org).
