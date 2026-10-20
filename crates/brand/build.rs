//! Reads brand/brand.toml (flat `key = "value"` lines) into constants for src/lib.rs.

use std::fmt::Write as _;
use std::path::PathBuf;

const KEYS: [&str; 6] = ["app_name", "accent_light", "accent_dark", "about_text", "about_url", "storage_key"];

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default());
    let brand_dir = manifest.join("../../brand");
    let toml = brand_dir.join("brand.toml");
    println!("cargo:rerun-if-changed={}", toml.display());
    for svg in ["logo-light.svg", "logo-dark.svg", "mark.svg"] {
        println!("cargo:rerun-if-changed={}", brand_dir.join(svg).display());
    }
    let text = match std::fs::read_to_string(&toml) {
        Ok(t) => t,
        Err(e) => {
            println!("cargo:warning=cannot read {}: {e}", toml.display());
            std::process::exit(1);
        }
    };
    let mut values: Vec<(String, String)> = Vec::new();
    for (n, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            println!("cargo:warning=brand.toml line {}: expected key = \"value\"", n.saturating_add(1));
            std::process::exit(1);
        };
        let (key, value) = (key.trim(), value.trim());
        let Some(value) = value.strip_prefix('"').and_then(|v| v.strip_suffix('"')) else {
            println!("cargo:warning=brand.toml line {}: the value must be a \"quoted string\"", n.saturating_add(1));
            std::process::exit(1);
        };
        if !KEYS.contains(&key) {
            println!("cargo:warning=brand.toml line {}: unknown key {key}", n.saturating_add(1));
            std::process::exit(1);
        }
        values.push((key.to_owned(), value.replace("\\\"", "\"")));
    }
    let mut out = String::new();
    for key in KEYS {
        let value = values.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str()).unwrap_or("");
        let _ = writeln!(out, "pub(crate) const {}: &str = {:?};", key.to_uppercase(), value);
    }
    let dest = PathBuf::from(std::env::var("OUT_DIR").unwrap_or_default()).join("brand.rs");
    if let Err(e) = std::fs::write(&dest, out) {
        println!("cargo:warning=cannot write {}: {e}", dest.display());
        std::process::exit(1);
    }
}
