//! Synthetic fixtures for the Free PDF SDK tests. No customer files: every PDF here is built
//! from code, deterministically, and the XFDF files in fixtures/ are hand-written in the shape
//! ComPDF WebViewer 2.9 exports (author in `title`, 0-based `page`, `name` UUIDs, replies by
//! `inreplyto`).
//!
//! `cargo run -p fps-tests --bin make-fixtures` writes the PDFs to tests/fixtures/ for the e2e
//! tests; `fixtures_on_disk_match_the_generators` checks they are current.

// Test support: the arithmetic below sizes and numbers a handful of objects in a fixture.
#![allow(clippy::arithmetic_side_effects)]

use std::fmt::Write as _;
use std::sync::Arc;

/// The owner password of [`protected_like_adobe`].
pub const OWNER_PASSWORD: &str = "fps-owner-secret";

/// The permissions a typical owner-only protection grants without the owner password:
/// low-quality printing (bit 3), copying (bit 5), accessibility (bit 10) and page assembly
/// (bit 11); no commenting (bit 6), no changes (bit 4), no form filling (bit 9), no
/// high-quality printing (bit 12). Bits 7-8 and 13-32 are 1, as §7.6.4.2 requires.
pub const ADOBE_PERMISSIONS: i32 = -4096 | 0b1100_0000 | (1 << 2) | (1 << 4) | (1 << 9) | (1 << 10);

/// A PDF of `pages` US Letter pages, each saying "Page N". Plain PDF 1.7 with a classic xref.
pub fn plain_pdf(pages: usize) -> Vec<u8> {
    let pages = pages.max(1);
    let mut objects: Vec<String> = Vec::new();
    // 1: catalog, 2: pages, 3: font, then a page and its content stream per page.
    objects.push("<< /Type /Catalog /Pages 2 0 R >>".to_owned());
    let kids: Vec<String> = (0..pages).map(|i| format!("{} 0 R", 4 + i * 2)).collect();
    objects.push(format!("<< /Type /Pages /Kids [{}] /Count {} >>", kids.join(" "), pages));
    objects.push("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>".to_owned());
    for i in 0..pages {
        let content = format!(
            "BT /F1 36 Tf 72 680 Td (Page {}) Tj ET\nBT /F1 12 Tf 72 640 Td (Free PDF SDK synthetic fixture. No customer data.) Tj ET\n0.2 0.4 0.7 RG 2 w 72 600 m 540 600 l S\n",
            i + 1
        );
        objects.push(format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 3 0 R >> >> /Contents {} 0 R >>",
            5 + i * 2
        ));
        objects.push(format!("<< /Length {} >>\nstream\n{}endstream", content.len(), content));
    }
    // A binary comment line, so tools treat the file as binary.
    let mut bytes: Vec<u8> = b"%PDF-1.7\n%\xe2\xe3\xcf\xd3\n".to_vec();
    let mut out = String::new();
    let mut offsets = Vec::with_capacity(objects.len());
    for (i, body) in objects.iter().enumerate() {
        offsets.push(bytes.len());
        bytes.extend_from_slice(format!("{} 0 obj\n{}\nendobj\n", i + 1, body).as_bytes());
    }
    let xref_at = bytes.len();
    let _ = write!(out, "xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1);
    for o in &offsets {
        let _ = writeln!(out, "{o:010} 00000 n ");
    }
    let _ = writeln!(
        out,
        "trailer\n<< /Size {} /Root 1 0 R /ID [<46505346495854555245303030303031> <46505346495854555245303030303031>] >>\nstartxref\n{}\n%%EOF",
        objects.len() + 1,
        xref_at
    );
    bytes.extend_from_slice(out.as_bytes());
    bytes
}

/// [`plain_pdf`] encrypted the way many production PDFs are: AES-256 (R6), no user password,
/// owner password [`OWNER_PASSWORD`], permissions [`ADOBE_PERMISSIONS`]. Deterministic.
pub fn protected_like_adobe(pages: usize) -> Result<Vec<u8>, String> {
    protected(pages, "", OWNER_PASSWORD, ADOBE_PERMISSIONS)
}

/// [`plain_pdf`] encrypted with AES-256 and the given passwords and permissions.
pub fn protected(pages: usize, user: &str, owner: &str, permissions: i32) -> Result<Vec<u8>, String> {
    let mut doc = pdfcraft_cos::Document::open(Arc::new(plain_pdf(pages))).map_err(|e| e.to_string())?;
    doc.set_encryption(&pdfcraft_cos::NewEncryption {
        algorithm: pdfcraft_cos::Algorithm::Aes256,
        user_password: user,
        owner_password: owner,
        permissions,
        encrypt_metadata: true,
        seed: [0x46; 32],
    })
    .map_err(|e| e.to_string())?;
    pdfcraft_cos::write_full(&doc, &Default::default()).map_err(|e| e.to_string())
}

/// The fixtures directory.
pub fn fixtures_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

/// A fixture file's text.
pub fn fixture_text(name: &str) -> String {
    std::fs::read_to_string(fixtures_dir().join(name)).unwrap_or_default()
}

/// The PDF fixtures the e2e tests load, by file name.
pub fn pdf_fixtures() -> Result<Vec<(&'static str, Vec<u8>)>, String> {
    Ok(vec![("plain-3p.pdf", plain_pdf(3)), ("protected-aes256-owner.pdf", protected_like_adobe(3)?)])
}
