//! `[DOC-1]` — the error pages ship with their errors: each `docs/errors/<code>.md` is built into
//! the compiler, so `ember explain <code>` prints it wherever the compiler runs.

use std::path::Path;

fn main() {
    let pages = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/errors");
    println!("cargo:rerun-if-changed={}", pages.display());
    let mut entries: Vec<(String, String)> = Vec::new();
    if let Ok(read) = std::fs::read_dir(&pages) {
        for entry in read.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "md") {
                let code = path.file_stem().map(|stem| stem.to_string_lossy().into_owned()).unwrap_or_default();
                let full = std::fs::canonicalize(&path).unwrap_or(path);
                println!("cargo:rerun-if-changed={}", full.display());
                entries.push((code, full.display().to_string()));
            }
        }
    }
    entries.sort();
    let mut out = String::from("/// `[DOC-1]` — each error page, by its code.\npub static ERROR_PAGES: &[(&str, &str)] = &[\n");
    for (code, path) in &entries {
        out.push_str(&format!("    ({code:?}, include_str!({path:?})),\n"));
    }
    out.push_str("];\n");
    let target = Path::new(&std::env::var("OUT_DIR").expect("cargo sets OUT_DIR")).join("error_pages.rs");
    std::fs::write(target, out).expect("the error-page table is writable");
}
