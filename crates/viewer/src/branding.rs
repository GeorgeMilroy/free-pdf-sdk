//! Installs the SDK's brand (brand/, through fps-brand) as pdfcraft's `Brand`.

use pdfcraft_ui_egui::brand::{self, Brand, BrandImage};

/// Uses the SDK's brand from now on; returns the settings key.
pub fn install() -> String {
    let b = fps_brand::brand();
    let rgb = |c: fps_brand::Rgb| egui::Color32::from_rgb(c.0, c.1, c.2);
    let accent = match (b.accent_light, b.accent_dark) {
        (Some(light), Some(dark)) => Some((rgb(light), rgb(dark))),
        (Some(one), None) | (None, Some(one)) => Some((rgb(one), rgb(one))),
        (None, None) => None,
    };
    let storage_key = b.storage_key.to_owned();
    brand::set(Brand {
        app_name: b.app_name.to_owned(),
        version: Some(fps_core::SDK_VERSION.to_owned()),
        logo_light: Some(BrandImage::new_static("bytes://fps-logo-light.svg", b.logo_light_svg, b.app_name)),
        logo_dark: Some(BrandImage::new_static("bytes://fps-logo-dark.svg", b.logo_dark_svg, b.app_name)),
        mark: Some(BrandImage::new_static("bytes://fps-mark.svg", b.mark_svg, b.app_name)),
        accent,
        about_text: b.about_text.map(str::to_owned),
        about_url: b.about_url.map(str::to_owned),
        community_links: false,
        // pdfcraft's project credits (its contributors and the AI models that wrote its commits)
        // say nothing about this SDK; NOTICE and third-party/ carry the attribution.
        credits: false,
        storage_key: storage_key.clone(),
    });
    storage_key
}
