//! Records the pdfcraft commit from Cargo.lock as FPS_PDFCRAFT_REV.

fn main() {
    let lock = std::path::Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default()).join("../../Cargo.lock");
    println!("cargo:rerun-if-changed={}", lock.display());
    let text = std::fs::read_to_string(&lock).unwrap_or_default();
    let mut rev = String::from("unknown");
    let mut in_engine = false;
    for line in text.lines() {
        if line.starts_with("name = ") {
            in_engine = line == "name = \"pdfcraft-engine\"";
        } else if in_engine && let Some(source) = line.strip_prefix("source = \"git+") {
            if let Some((_, hash)) = source.trim_end_matches('"').rsplit_once('#') {
                rev = hash.to_owned();
            }
            break;
        }
    }
    println!("cargo:rustc-env=FPS_PDFCRAFT_REV={rev}");
}
