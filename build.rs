fn main() {
    // Force recompilation when workflow JSON files change.
    // These are embedded via include_str!() in src/lib.rs.
    println!("cargo:rerun-if-changed=workflows/preprocessor.json");
    println!("cargo:rerun-if-changed=workflows/analysis/a1_seer.json");
    println!("cargo:rerun-if-changed=workflows/analysis/a2_thalai.json");
    println!("cargo:rerun-if-changed=workflows/analysis/a3_adi.json");
    println!("cargo:rerun-if-changed=workflows/analysis/a4_thodai.json");
    println!("cargo:rerun-if-changed=workflows/analysis/a5_classify.json");
}
