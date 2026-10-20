//! Free PDF SDK branding: the product name, logos, accent colours, About text and the settings
//! key, read from `brand/` at compile time. Replace the files there to rebrand (see brand/README.md).
//!
//! This crate has no egui dependency: the viewer turns [`BrandInfo`] into pdfcraft's `Brand`.

mod generated {
    include!(concat!(env!("OUT_DIR"), "/brand.rs"));
}

/// An sRGB colour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
    /// Parses `#RRGGBB`; anything else is `None`.
    pub fn parse(s: &str) -> Option<Self> {
        let hex = s.strip_prefix('#')?;
        if hex.len() != 6 || !hex.is_ascii() {
            return None;
        }
        let byte = |i: usize| hex.get(i..i.checked_add(2)?).and_then(|h| u8::from_str_radix(h, 16).ok());
        Some(Self(byte(0)?, byte(2)?, byte(4)?))
    }
}

/// The SDK's identity.
#[derive(Clone, Debug)]
pub struct BrandInfo {
    pub app_name: &'static str,
    pub accent_light: Option<Rgb>,
    pub accent_dark: Option<Rgb>,
    pub about_text: Option<&'static str>,
    pub about_url: Option<&'static str>,
    pub storage_key: &'static str,
    pub logo_light_svg: &'static [u8],
    pub logo_dark_svg: &'static [u8],
    pub mark_svg: &'static [u8],
}

fn non_empty(s: &'static str) -> Option<&'static str> {
    (!s.trim().is_empty()).then_some(s)
}

/// The brand in brand/ (falls back to "Free PDF SDK" and "free-pdf-sdk" for empty values).
pub fn brand() -> BrandInfo {
    BrandInfo {
        app_name: non_empty(generated::APP_NAME).unwrap_or(APP_NAME),
        accent_light: Rgb::parse(generated::ACCENT_LIGHT),
        accent_dark: Rgb::parse(generated::ACCENT_DARK),
        about_text: non_empty(generated::ABOUT_TEXT),
        about_url: non_empty(generated::ABOUT_URL).filter(|u| u.starts_with("https://")),
        storage_key: non_empty(generated::STORAGE_KEY).unwrap_or("free-pdf-sdk"),
        logo_light_svg: include_bytes!("../../../brand/logo-light.svg"),
        logo_dark_svg: include_bytes!("../../../brand/logo-dark.svg"),
        mark_svg: include_bytes!("../../../brand/mark.svg"),
    }
}

/// The default product name.
pub const APP_NAME: &str = "Free PDF SDK";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hex_colours_and_rejects_the_rest() {
        assert_eq!(Rgb::parse("#2B6CB0"), Some(Rgb(0x2B, 0x6C, 0xB0)));
        assert_eq!(Rgb::parse("2B6CB0"), None);
        assert_eq!(Rgb::parse("#2B6CB"), None);
        assert_eq!(Rgb::parse("#2B6CBG"), None);
        assert_eq!(Rgb::parse("#ééé"), None);
    }

    #[test]
    fn the_shipped_brand_names_no_trademark() {
        let b = brand();
        assert_eq!(b.app_name, "Free PDF SDK");
        assert_eq!(b.storage_key, "free-pdf-sdk");
        for svg in [b.logo_light_svg, b.logo_dark_svg, b.mark_svg] {
            let text = String::from_utf8_lossy(svg);
            assert!(text.starts_with("<svg"), "an SVG");
            for mark in ["ArtCraft", "PdfCraft", "artcraft", "pdfcraft"] {
                assert!(!text.contains(mark), "no {mark} in the brand");
            }
        }
        assert!(b.accent_light.is_some() && b.accent_dark.is_some());
    }
}
