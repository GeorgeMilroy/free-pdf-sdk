//! Synthetic fixtures for the Free PDF SDK tests. No customer files: every PDF here is built
//! from code, deterministically, and the XFDF files in fixtures/ are hand-written in the shape
//! web PDF viewers commonly export (author in `title`, 0-based `page`, `name` UUIDs, replies by
//! `inreplyto`).
//!
//! `cargo run -p fps-tests --bin make-fixtures` writes the PDFs to tests/fixtures/ for the e2e
//! tests; `fixtures_on_disk_match_the_generators` checks they are current.

// Test support: the arithmetic below sizes and numbers a handful of objects in a fixture.
#![allow(clippy::arithmetic_side_effects)]

use std::fmt::Write as _;
use std::sync::Arc;

/// The owner password of [`protected_owner_only`].
pub const OWNER_PASSWORD: &str = "fps-owner-secret";

/// The permissions a typical owner-only protection grants without the owner password:
/// low-quality printing (bit 3), copying (bit 5), accessibility (bit 10) and page assembly
/// (bit 11); no commenting (bit 6), no changes (bit 4), no form filling (bit 9), no
/// high-quality printing (bit 12). Bits 7-8 and 13-32 are 1, as §7.6.4.2 requires.
pub const OWNER_ONLY_PERMISSIONS: i32 = -4096 | 0b1100_0000 | (1 << 2) | (1 << 4) | (1 << 9) | (1 << 10);

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

/// [`plain_pdf`] encrypted with owner-only protection: AES-256 (R6), no user password,
/// owner password [`OWNER_PASSWORD`], permissions [`OWNER_ONLY_PERMISSIONS`]. Deterministic.
pub fn protected_owner_only(pages: usize) -> Result<Vec<u8>, String> {
    protected(pages, "", OWNER_PASSWORD, OWNER_ONLY_PERMISSIONS)
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
    Ok(vec![("plain-3p.pdf", plain_pdf(3)), ("protected-aes256-owner.pdf", protected_owner_only(3)?), ("demo.pdf", demo_pdf())])
}

/// A PDF from numbered object bodies (object N is `objects[N - 1]`), with a classic xref.
fn write_pdf(objects: &[String]) -> Vec<u8> {
    let mut bytes: Vec<u8> = b"%PDF-1.7\n%\xe2\xe3\xcf\xd3\n".to_vec();
    let mut offsets = Vec::with_capacity(objects.len());
    for (i, body) in objects.iter().enumerate() {
        offsets.push(bytes.len());
        bytes.extend_from_slice(format!("{} 0 obj\n{}\nendobj\n", i + 1, body).as_bytes());
    }
    let xref_at = bytes.len();
    let mut out = String::new();
    let _ = write!(out, "xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1);
    for o in &offsets {
        let _ = writeln!(out, "{o:010} 00000 n ");
    }
    let _ = writeln!(
        out,
        "trailer\n<< /Size {} /Root 1 0 R /ID [<46505344454d4f303030303030303031> <46505344454d4f303030303030303031>] >>\nstartxref\n{}\n%%EOF",
        objects.len() + 1,
        xref_at
    );
    bytes.extend_from_slice(out.as_bytes());
    bytes
}

fn stream(dict: &str, content: &str) -> String {
    format!("<< {dict} /Length {} >>\nstream\n{content}endstream", content.len())
}

/// A line of text: font (F1 regular, F2 bold), size, position, grey level, text.
fn text(font: &str, size: u32, x: u32, y: u32, grey: &str, s: &str) -> String {
    format!("BT {grey} rg /{font} {size} Tf {x} {y} Td ({s}) Tj ET\n")
}

/// The live demo's sample: a branded cover with a layered floor plan to annotate, an approval
/// form (text fields, a checkbox, notes) for the fill and sign modes, and a dimensioned
/// specification page to measure. Synthetic: no customer data. Deterministic.
pub fn demo_pdf() -> Vec<u8> {
    let ink = "0.03 0.04 0.05";
    let muted = "0.38 0.40 0.43";
    // Page 1: cover and floor plan.
    let mut p1 = String::from("1 0.03 0 rg 0 704 612 88 re f\n0.67 0 0.05 rg 0 704 612 6 re f\n");
    p1 += &text("F2", 30, 48, 742, "1 1 1", "Free PDF SDK");
    p1 += &text("F1", 13, 48, 720, "1 1 1", "Sample document for the live demo");
    p1 += &text("F2", 20, 48, 650, ink, "Welcome");
    for (i, line) in [
        "This PDF is generated by code for the Free PDF SDK demo. It holds no real data.",
        "Try the tools: add a sticky note, highlight this sentence, draw a shape, or stamp it.",
        "Open the Layers panel on the right to show and hide the Structure, Furniture, Electrical",
        "and Dimensions layers of the floor plan below. Change the mode in the demo panel and",
        "watch the interface follow: a feature you switch off is gone from every menu and toolbar.",
    ]
    .iter()
    .enumerate()
    {
        p1 += &text("F1", 11, 48, 624 - 17 * i as u32, ink, line);
    }
    p1 += &text("F2", 14, 48, 512, ink, "Floor plan");
    p1 += &text("F1", 10, 48, 496, muted, "Apartment 3B, scale 1:100. Approve it, or mark what needs to change.");
    p1 += "0.85 0.86 0.88 RG 1 w 48 150 516 330 re S\n";
    // The plan is four optional content groups (objects 19-22), so the Layers panel has
    // something to show and hide: the structure, the furniture, the electrical layout and the
    // dimensions. A quarter-circle door swing is one Bezier; circles are four.
    let circle = |cx: f32, cy: f32, r: f32| {
        let k = r * 0.5523;
        format!(
            "{} {} m {} {} {} {} {} {} c {} {} {} {} {} {} c {} {} {} {} {} {} c {} {} {} {} {} {} c\n",
            cx + r,
            cy,
            cx + r,
            cy + k,
            cx + k,
            cy + r,
            cx,
            cy + r,
            cx - k,
            cy + r,
            cx - r,
            cy + k,
            cx - r,
            cy,
            cx - r,
            cy - k,
            cx - k,
            cy - r,
            cx,
            cy - r,
            cx + k,
            cy - r,
            cx + r,
            cy - k,
            cx + r,
            cy
        )
    };
    p1 += "/OC /Structure BDC\n0.03 0.04 0.05 RG 4 w 100 200 400 230 re S\n";
    p1 += "2 w 300 200 m 300 290 l S 300 330 m 300 430 l S 300 320 m 400 320 l S 435 320 m 500 320 l S\n";
    p1 += "1 1 1 RG 6 w 180 200 m 220 200 l S\n";
    p1 += "0.03 0.04 0.05 RG 1 w 300 330 m 340 330 l S 340 330 m 340 308 322 290 300 290 c S\n";
    p1 += "400 320 m 400 285 l S 400 285 m 419 285 435 301 435 320 c S\n";
    p1 += "180 200 m 180 240 l S 180 240 m 202 240 220 222 220 200 c S\n";
    p1 += "0.35 0.6 0.85 RG 1 w 130 428 m 190 428 l S 130 432 m 190 432 l S 360 428 m 440 428 l S 360 432 m 440 432 l S 98 300 m 98 360 l S 102 300 m 102 360 l S\n";
    p1 += &text("F2", 8, 116, 410, muted, "LIVING");
    p1 += &text("F2", 8, 316, 410, muted, "BEDROOM");
    p1 += &text("F2", 8, 316, 256, muted, "BATH");
    p1 += "EMC\n";
    p1 += "/OC /Furniture BDC\n0.78 0.8 0.83 rg 120 220 90 34 re f 0.6 0.62 0.65 rg 120 246 90 8 re f\n";
    p1 += &format!("0.78 0.8 0.83 rg {}f\n", circle(165.0, 290.0, 16.0));
    p1 += "0.78 0.8 0.83 rg 230 350 60 40 re f 0.6 0.62 0.65 rg 222 356 6 10 re f 222 374 6 10 re f 292 356 6 10 re f 292 374 6 10 re f\n";
    p1 += "0.78 0.8 0.83 rg 330 340 90 70 re f 0.95 0.95 0.96 rg 334 394 36 12 re f 376 394 36 12 re f\n";
    p1 += "0.85 0.87 0.9 rg 320 212 70 34 re f 0.95 0.96 0.97 rg 326 218 58 22 re f 0.85 0.87 0.9 rg 440 212 30 20 re f 450 270 22 30 re f\n";
    p1 += "EMC\n";
    p1 += "/OC /Electrical BDC\n0.9 0.45 0 RG 1 w\n";
    for (cx, cy) in [(200.0, 315.0), (400.0, 375.0), (400.0, 260.0)] {
        p1 += &format!(
            "{}S {} {} m {} {} l S {} {} m {} {} l S\n",
            circle(cx, cy, 7.0),
            cx - 5.0,
            cy - 5.0,
            cx + 5.0,
            cy + 5.0,
            cx - 5.0,
            cy + 5.0,
            cx + 5.0,
            cy - 5.0
        );
    }
    for (cx, cy) in [(104.0, 230.0), (104.0, 400.0), (296.0, 250.0), (496.0, 380.0), (496.0, 230.0), (250.0, 204.0)] {
        p1 += &format!("{}S\n", circle(cx, cy, 3.0));
    }
    p1 += "150 204 4 8 re S 304 340 4 8 re S 440 324 8 4 re S\n";
    p1 += "EMC\n";
    p1 += "/OC /Dimensions BDC\n1 0.03 0 RG 0.75 w 100 184 m 500 184 l S 100 180 m 100 188 l S 500 180 m 500 188 l S\n";
    p1 += "516 200 m 516 430 l S 512 200 m 520 200 l S 512 430 m 520 430 l S\n";
    p1 += "100 444 m 300 444 l S 300 444 m 500 444 l S 100 440 m 100 448 l S 300 440 m 300 448 l S 500 440 m 500 448 l S\n";
    p1 += &text("F1", 8, 288, 172, "1 0.03 0", "8 000");
    p1 += &text("F1", 8, 505, 436, "1 0.03 0", "4 600");
    p1 += &text("F1", 8, 188, 451, "1 0.03 0", "4 000");
    p1 += &text("F1", 8, 388, 451, "1 0.03 0", "4 000");
    p1 += &text("F1", 7, 440, 172, "1 0.03 0", "SCALE 1:100  |  mm");
    p1 += "EMC\n";
    p1 += &text("F1", 9, 48, 40, muted, "Free PDF SDK sample  |  Page 1 of 3");

    // Page 2: approval form; the fields are widgets (objects 12-16).
    let mut p2 = String::new();
    p2 += &text("F2", 20, 48, 720, ink, "Approval form");
    p2 += &text("F1", 11, 48, 698, muted, "Fill in the fields in fill, sign, edit or full mode. Sign with Fill & Sign.");
    for (label, y) in [("Full name", 648), ("Email", 608), ("Company", 568), ("Approved for construction", 528), ("Notes", 488)] {
        p2 += &text("F2", 11, 48, y, ink, label);
    }
    p2 += "0.85 0.86 0.88 RG 1 w 48 300 m 300 300 l S\n";
    p2 += &text("F1", 10, 48, 284, muted, "Signature");
    p2 += "48 240 m 300 240 l S\n";
    p2 += &text("F1", 10, 48, 224, muted, "Date");
    p2 += &text("F1", 9, 48, 40, muted, "Free PDF SDK sample  |  Page 2 of 3");

    // Page 3: a dimensioned drawing to measure.
    let mut p3 = String::new();
    p3 += &text("F2", 20, 48, 720, ink, "Specification");
    p3 += &text("F1", 11, 48, 698, muted, "Measure the room with the distance, perimeter and area tools.");
    p3 += "0.03 0.04 0.05 RG 1.5 w 120 330 360 240 re S\n";
    p3 += "1 0.03 0 RG 0.75 w [4 3] 0 d 120 310 m 480 310 l S 500 330 m 500 570 l S [] 0 d\n";
    p3 += &text("F1", 10, 262, 296, ink, "6 000 mm");
    p3 += &text("F1", 10, 508, 446, ink, "4 000 mm");
    p3 += "0.85 0.86 0.88 RG 0.5 w 120 450 m 480 450 l S 300 330 m 300 570 l S\n";
    for (i, (k, v)) in
        [("Scale", "1:50"), ("Walls", "200 mm blockwork, plastered both sides"), ("Ceiling", "2 700 mm clear height")].iter().enumerate()
    {
        let y = 220 - 22 * i as u32;
        p3 += &text("F2", 11, 120, y, ink, k);
        p3 += &text("F1", 11, 240, y, ink, v);
    }
    p3 += &text("F1", 9, 48, 40, muted, "Free PDF SDK sample  |  Page 3 of 3");

    let page = |contents: u32, resources: &str, extra: &str| {
        format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 3 0 R /F2 4 0 R >>{resources} >> /Contents {contents} 0 R{extra} >>"
        )
    };
    let layers = " /Properties << /Structure 19 0 R /Furniture 20 0 R /Electrical 21 0 R /Dimensions 22 0 R >>";
    let field = |name: &str, tip: &str, rect: &str, flags: &str| {
        format!(
            "<< /Type /Annot /Subtype /Widget /FT /Tx /T ({name}) /TU ({tip}) /Rect [{rect}] /P 8 0 R /F 4 /DA (/Helv 11 Tf 0 g){flags} /MK << /BC [0.75 0.76 0.78] /BG [1 1 1] >> >>"
        )
    };
    let objects = vec![
        "<< /Type /Catalog /Pages 2 0 R /AcroForm 5 0 R /OCProperties << /OCGs [19 0 R 20 0 R 21 0 R 22 0 R] /D << /Order [19 0 R 20 0 R 21 0 R 22 0 R] >> >> >>".to_owned(),
        "<< /Type /Pages /Kids [6 0 R 8 0 R 10 0 R] /Count 3 >>".to_owned(),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>".to_owned(),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold /Encoding /WinAnsiEncoding >>".to_owned(),
        "<< /Fields [12 0 R 13 0 R 14 0 R 15 0 R 16 0 R] /NeedAppearances true /DR << /Font << /Helv 3 0 R >> >> /DA (/Helv 11 Tf 0 g) >>".to_owned(),
        page(7, layers, ""),
        stream("", &p1),
        page(9, "", " /Annots [12 0 R 13 0 R 14 0 R 15 0 R 16 0 R]"),
        stream("", &p2),
        page(11, "", ""),
        stream("", &p3),
        field("name", "Full name", "200 640 540 662", ""),
        field("email", "Email", "200 600 540 622", ""),
        field("company", "Company", "200 560 540 582", ""),
        "<< /Type /Annot /Subtype /Widget /FT /Btn /T (approved) /TU (Approved for construction) /Rect [200 522 218 540] /P 8 0 R /F 4 /V /Off /AS /Off /MK << /BC [0.75 0.76 0.78] /BG [1 1 1] >> /AP << /N << /Yes 17 0 R /Off 18 0 R >> >> >>".to_owned(),
        field("notes", "Notes", "200 400 540 500", " /Ff 4096"),
        stream("/Type /XObject /Subtype /Form /BBox [0 0 18 18]", "1 0.03 0 rg 4 4 10 10 re f\n"),
        stream("/Type /XObject /Subtype /Form /BBox [0 0 18 18]", ""),
        "<< /Type /OCG /Name (Structure) >>".to_owned(),
        "<< /Type /OCG /Name (Furniture) >>".to_owned(),
        "<< /Type /OCG /Name (Electrical) >>".to_owned(),
        "<< /Type /OCG /Name (Dimensions) >>".to_owned(),
    ];
    write_pdf(&objects)
}
