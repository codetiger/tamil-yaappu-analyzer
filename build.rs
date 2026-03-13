fn main() {
    // Force recompilation when workflow JSON files change.
    // These are embedded via include_str!() in src/lib.rs.
    println!("cargo:rerun-if-changed=workflows/preprocessor.json");
    println!("cargo:rerun-if-changed=workflows/analysis/a1_counts.json");
    println!("cargo:rerun-if-changed=workflows/analysis/a2_structural.json");
    println!("cargo:rerun-if-changed=workflows/analysis/a3_seer.json");
    println!("cargo:rerun-if-changed=workflows/analysis/a4_thalai.json");
    println!("cargo:rerun-if-changed=workflows/analysis/a5_ornamentation.json");
    println!("cargo:rerun-if-changed=workflows/analysis/a6_classify.json");
}
