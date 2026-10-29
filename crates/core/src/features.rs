//! The SDK's configuration model: modes (presets), features (what the viewer offers) and
//! interface options. [`resolve`] turns a host's configuration into the profile the viewer hands
//! to pdfcraft. The tables here are also the source of the documentation (`reference()`).
//!
//! Feature keys are pdfcraft's own feature names (crates/ui-egui/src/features.rs in the fork);
//! `viewer` (always on) and `app` (desktop-only: opening other files, tabs, update checks) are
//! not offered.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Value, json};

/// A feature a host can turn on or off.
pub struct FeatureDoc {
    pub key: &'static str,
    pub title: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    /// Whether it changes documents.
    pub edits: bool,
}

const fn f(key: &'static str, title: &'static str, category: &'static str, edits: bool, description: &'static str) -> FeatureDoc {
    FeatureDoc { key, title, category, description, edits }
}

/// Every feature, grouped by category, in documentation order.
pub const FEATURES: &[FeatureDoc] = &[
    f(
        "comments",
        "Comments",
        "Annotation",
        true,
        "Sticky notes, text boxes, callouts, file attachments, replies, review status, the comment list and comment import/export.",
    ),
    f("markup", "Text markup", "Annotation", true, "Highlight, underline, strikethrough, squiggly underline, insert (caret) and replace text."),
    f("shapes", "Shapes", "Annotation", true, "Rectangles, ovals, lines, arrows, polygons, polylines and clouds."),
    f("ink", "Freehand ink", "Annotation", true, "Freehand drawing and the eraser."),
    f("stamps", "Stamps", "Annotation", true, "Standard and custom stamps."),
    f("measurement", "Measurement", "Annotation", true, "Distance, perimeter and area measurements with a drawing scale."),
    f(
        "fillSign",
        "Fill & Sign",
        "Forms & signing",
        true,
        "Typed text, checkmarks, crosses, dots, lines, dates, and drawn or typed signatures and initials placed on the page.",
    ),
    f("formFill", "Form filling", "Forms & signing", true, "Filling in form fields, clearing a form, importing and exporting form data."),
    f(
        "formDesign",
        "Form design",
        "Forms & signing",
        true,
        "Adding, editing and deleting form fields, text field tables, tab and calculation order, and field detection.",
    ),
    f("digitalSignatures", "Digital signatures", "Forms & signing", true, "Signing and certifying with a digital ID (certificate)."),
    f("signaturePanel", "Signature panel", "Forms & signing", false, "The signatures panel: viewing and validating existing digital signatures."),
    f("pages", "Page organization", "Pages & content", true, "Insert, delete, rotate, move, duplicate, crop, replace and number pages."),
    f("bookmarks", "Bookmark editing", "Pages & content", true, "Adding, renaming, moving and deleting bookmarks (viewing them is always on)."),
    f(
        "editContent",
        "Content editing",
        "Pages & content",
        true,
        "Editing text and images, links, headers and footers, watermarks, backgrounds, Bates numbers and document properties.",
    ),
    f("flatten", "Flatten", "Pages & content", true, "Flattening comments and form fields into the page content."),
    f("redaction", "Redaction", "Pages & content", true, "Marking and applying redactions, sanitizing and removing hidden information."),
    f("accessibility", "Accessibility", "Pages & content", true, "Accessibility checking, tags, reading order and alternate text."),
    f("standards", "PDF standards", "Pages & content", true, "Converting to PDF/A, and removing a document's PDF/A or PDF/X identification."),
    f(
        "javascript",
        "JavaScript",
        "Pages & content",
        true,
        "Running the document's JavaScript, the console and editing scripts. Off unless you turn it on.",
    ),
    f("security", "Security", "Security", true, "Adding or removing passwords and permissions."),
    f("history", "Undo and redo", "Editing", false, "Undo and redo of the changes the other features allow."),
    f("save", "Save", "Output", false, "The viewer's own Save button; the PDF goes to your page in the saveRequested event."),
    f("download", "Download a copy", "Output", false, "Save As, saving optimized copies, attachments and reports from the viewer."),
    f("export", "Export", "Output", false, "Exporting to Word, PowerPoint, images, HTML and text; extracting and splitting pages."),
    f("print", "Print", "Output", false, "Printing."),
    f("search", "Search", "Viewing", false, "Finding text in the document."),
    f("copy", "Copy", "Viewing", false, "Copying text and images (when the document allows it)."),
    f(
        "documentInfo",
        "Document properties",
        "Viewing",
        false,
        "The document properties dialog (read-only unless content editing is on) and word count.",
    ),
    f("help", "Help", "Interface", false, "Keyboard shortcuts and About."),
    f("palette", "Command palette", "Interface", false, "The \"Find tools and commands\" box and palette (Ctrl/Cmd+K)."),
    f("preferences", "Preferences", "Interface", false, "The preferences dialog, theme switching and rearranging panels."),
];

/// A mode: a preset set of features.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    View,
    Review,
    Annotate,
    Fill,
    Sign,
    Organize,
    Edit,
    Full,
}

/// Features on in every mode.
const BASE: &[&str] = &["search", "copy", "print", "documentInfo", "help", "palette", "preferences", "signaturePanel"];

impl Mode {
    pub const ALL: [Mode; 8] = [Mode::View, Mode::Review, Mode::Annotate, Mode::Fill, Mode::Sign, Mode::Organize, Mode::Edit, Mode::Full];

    pub const fn key(self) -> &'static str {
        match self {
            Mode::View => "view",
            Mode::Review => "review",
            Mode::Annotate => "annotate",
            Mode::Fill => "fill",
            Mode::Sign => "sign",
            Mode::Organize => "organize",
            Mode::Edit => "edit",
            Mode::Full => "full",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|m| m.key() == key)
    }

    pub const fn description(self) -> &'static str {
        match self {
            Mode::View => "Read-only viewing: navigation, search, copy, print. Nothing can change the document.",
            Mode::Review => "Notes, text boxes and text markup with replies and review status. Nothing else changes.",
            Mode::Annotate => "Every kind of annotation: comments, markup, shapes, ink, stamps, measurement and Fill & Sign. No other edits.",
            Mode::Fill => "Filling in forms and Fill & Sign. No comments, page or content changes.",
            Mode::Sign => "Form filling, Fill & Sign and digital signatures.",
            Mode::Organize => {
                "Page organization and bookmarks, extracting and splitting pages, and export to other formats (Word, PowerPoint, images, HTML, text). No content or comment changes."
            }
            Mode::Edit => {
                "Full authoring: annotations, forms, pages, content and accessibility. No security, redaction, digital signatures or JavaScript."
            }
            Mode::Full => "Everything the viewer offers, except JavaScript (turn it on with features.javascript).",
        }
    }

    /// The features this mode turns on.
    pub fn features(self) -> BTreeSet<&'static str> {
        let extra: &[&str] = match self {
            Mode::View => &[],
            Mode::Review => &["comments", "markup", "history", "save"],
            Mode::Annotate => &["comments", "markup", "shapes", "ink", "stamps", "measurement", "fillSign", "history", "save"],
            Mode::Fill => &["formFill", "fillSign", "history", "save"],
            Mode::Sign => &["formFill", "fillSign", "digitalSignatures", "history", "save"],
            Mode::Organize => &["pages", "bookmarks", "export", "history", "save"],
            Mode::Edit => &[
                "comments",
                "markup",
                "shapes",
                "ink",
                "stamps",
                "measurement",
                "fillSign",
                "formFill",
                "formDesign",
                "flatten",
                "pages",
                "bookmarks",
                "editContent",
                "accessibility",
                "standards",
                "export",
                "download",
                "history",
                "save",
            ],
            Mode::Full => return FEATURES.iter().map(|f| f.key).filter(|k| *k != "javascript").collect(),
        };
        BASE.iter().chain(extra).copied().collect()
    }
}

/// Whether `key` names a feature.
pub fn is_feature(key: &str) -> bool {
    FEATURES.iter().any(|f| f.key == key)
}

/// A feature's documentation entry.
pub fn feature(key: &str) -> Option<&'static FeatureDoc> {
    FEATURES.iter().find(|f| f.key == key)
}

/// Features that make annotations (what `importXfdf` and `deleteAllAnnotations` need).
pub const ANNOTATION_FEATURES: &[&str] = &["comments", "markup", "shapes", "ink", "stamps", "measurement", "fillSign"];

/// Interface options: parts of the viewer a host can show or hide.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiOptions {
    /// The Tools panel on the left (the tool groups of the enabled features).
    pub tools_panel: bool,
    /// The slim rail that brings a closed Tools panel back.
    pub tools_rail: bool,
    /// The floating tool bar beside the page.
    pub quick_toolbar: bool,
    /// The right-hand rail (panel buttons, page navigation, zoom).
    pub right_rail: bool,
    /// Zoom and page display controls on the rail.
    pub zoom_controls: bool,
    /// Page navigation on the rail.
    pub page_navigation: bool,
    /// The menu bar.
    pub menu: bool,
    /// The "Find tools and commands" box in the top bar.
    pub search_box: bool,
    /// Right-hand panels shown or hidden (`comments`, `bookmarks`, ...). Unlisted ones show
    /// when their feature is on.
    pub panels: BTreeMap<String, bool>,
    /// Panels can't be moved, floated or closed from their tabs.
    pub lock_layout: bool,
    /// List tools that are planned but not available yet.
    pub show_planned_tools: bool,
    /// Links to web pages may open (after a prompt).
    pub external_links: bool,
}

impl Default for UiOptions {
    fn default() -> Self {
        Self {
            tools_panel: true,
            tools_rail: true,
            quick_toolbar: true,
            right_rail: true,
            zoom_controls: true,
            page_navigation: true,
            menu: true,
            search_box: true,
            panels: BTreeMap::new(),
            lock_layout: false,
            show_planned_tools: false,
            external_links: true,
        }
    }
}

/// Right-hand panels `ui.panels` accepts.
pub const PANELS: &[(&str, &str)] = &[
    ("comments", "The comment list"),
    ("bookmarks", "Bookmarks"),
    ("pages", "Page thumbnails"),
    ("fields", "Form fields"),
    ("layers", "Layers"),
    ("attachments", "File attachments"),
    ("signatures", "Digital signatures (needs signaturePanel)"),
    ("accessibility", "Accessibility checker (needs accessibility)"),
    ("tags", "Structure tags (needs accessibility)"),
    ("search", "Advanced search results (needs search)"),
];

/// The boolean `ui` options: name, default, description.
pub const UI_FLAGS: &[(&str, bool, &str)] = &[
    ("toolsPanel", true, "The Tools panel on the left, listing the tool groups of the enabled features."),
    ("toolsRail", true, "A slim rail that brings the Tools panel back once it is closed."),
    ("quickToolbar", true, "The floating tool bar beside the page (Select, Hand and the enabled tools)."),
    ("rightRail", true, "The right-hand rail: panel buttons, page navigation and zoom."),
    ("zoomControls", true, "Zoom and page display controls on the rail."),
    ("pageNavigation", true, "Page navigation on the rail."),
    ("menu", true, "The menu bar (File, Edit, View... with the enabled commands)."),
    ("searchBox", true, "The \"Find tools and commands\" box in the top bar (needs the palette feature)."),
    ("lockLayout", false, "Hides the Reset side panels command (panels are docked: they can't be moved or floated anyway)."),
    ("showPlannedTools", false, "Also list tools that are planned but not available yet."),
    ("externalLinks", true, "Links to web pages in documents may open (after a prompt)."),
];

/// A host's configuration, before resolution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeatureConfig {
    pub mode: Mode,
    /// Overrides of the mode's features: key → on/off.
    pub features: BTreeMap<String, bool>,
    pub ui: UiOptions,
    /// `disabledElements` ids (crate::elements).
    pub disabled: Vec<String>,
}

impl Default for FeatureConfig {
    fn default() -> Self {
        Self { mode: Mode::View, features: BTreeMap::new(), ui: UiOptions::default(), disabled: Vec::new() }
    }
}

/// What the viewer hands to pdfcraft's UI profile.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedProfile {
    /// The enabled feature keys (pdfcraft's names).
    pub features: BTreeSet<&'static str>,
    /// No editing feature is on: the viewer stays in Read mode.
    pub read_only: bool,
    /// pdfcraft ids to hide: commands, `group:<id>`, `ui:<part>`, `panel:<name>`.
    pub hidden: BTreeSet<String>,
    pub tools_rail: bool,
    pub hide_planned: bool,
    pub lock_layout: bool,
    pub external_links: bool,
}

impl FeatureConfig {
    /// The enabled features: the mode's, then the overrides.
    pub fn enabled(&self) -> BTreeSet<&'static str> {
        let mut on = self.mode.features();
        for (key, value) in &self.features {
            if let Some(doc) = feature(key) {
                if *value {
                    on.insert(doc.key);
                } else {
                    on.remove(doc.key);
                }
            }
        }
        on
    }

    /// Whether the API may change annotations (`importXfdf`, `deleteAllAnnotations`).
    pub fn allows_annotation_changes(&self) -> bool {
        let on = self.enabled();
        ANNOTATION_FEATURES.iter().any(|k| on.contains(k))
    }

    /// The profile for pdfcraft.
    pub fn resolve(&self) -> ResolvedProfile {
        let features = self.enabled();
        let read_only = !features.iter().any(|k| feature(k).is_some_and(|f| f.edits));
        let mut hidden: BTreeSet<String> = crate::profile::ALWAYS_HIDDEN.iter().map(|s| (*s).to_owned()).collect();
        if let Ok(ids) = crate::elements::expand(self.disabled.iter().map(String::as_str)) {
            hidden.extend(ids.into_iter().map(str::to_owned));
        }
        // One document per viewer: no tab strip, Home or workspace tabs.
        hidden.extend(["ui:tabs", "ui:home", "ui:mode_tabs"].map(str::to_owned));
        let ui = &self.ui;
        for (on, part) in [
            (ui.tools_panel, "ui:tools_panel"),
            (ui.quick_toolbar, "ui:quick_bar"),
            (ui.right_rail, "ui:rail"),
            (ui.zoom_controls, "ui:rail_zoom"),
            (ui.page_navigation, "ui:rail_page_nav"),
            (ui.menu, "ui:menu"),
            (ui.search_box && features.contains("palette"), "ui:search"),
        ] {
            if !on {
                hidden.insert(part.to_owned());
            }
        }
        for (name, on) in &ui.panels {
            if !on {
                hidden.insert(format!("panel:{name}"));
            }
        }
        ResolvedProfile {
            features,
            read_only,
            hidden,
            tools_rail: ui.tools_rail && ui.tools_panel,
            hide_planned: !ui.show_planned_tools,
            lock_layout: ui.lock_layout,
            external_links: ui.external_links,
        }
    }

    /// The configuration as `getConfig` reports it.
    pub fn to_json(&self) -> Value {
        let mut on = self.enabled();
        // Without a document-changing feature there is nothing to save or undo.
        if !on.iter().any(|k| feature(k).is_some_and(|f| f.edits)) {
            on.remove("save");
            on.remove("history");
        }
        let features: serde_json::Map<String, Value> = FEATURES.iter().map(|f| (f.key.to_owned(), Value::Bool(on.contains(f.key)))).collect();
        let ui = &self.ui;
        json!({
            "mode": self.mode.key(),
            "features": features,
            "ui": {
                "toolsPanel": ui.tools_panel,
                "toolsRail": ui.tools_rail,
                "quickToolbar": ui.quick_toolbar,
                "rightRail": ui.right_rail,
                "zoomControls": ui.zoom_controls,
                "pageNavigation": ui.page_navigation,
                "menu": ui.menu,
                "searchBox": ui.search_box,
                "panels": ui.panels,
                "lockLayout": ui.lock_layout,
                "showPlannedTools": ui.show_planned_tools,
                "externalLinks": ui.external_links,
            },
            "disabledElements": self.disabled,
            "readOnly": !on.iter().any(|k| feature(k).is_some_and(|f| f.edits)),
        })
    }
}

/// The configuration reference (modes, features, interface options, element ids) as JSON: the
/// source of the docs' tables and of `docs/reference/config.json`.
pub fn reference() -> Value {
    let modes: Vec<Value> = Mode::ALL
        .into_iter()
        .map(|m| json!({ "key": m.key(), "description": m.description(), "features": m.features().into_iter().collect::<Vec<_>>() }))
        .collect();
    let features: Vec<Value> = FEATURES
        .iter()
        .map(|f| {
            let modes: Vec<&str> = Mode::ALL.into_iter().filter(|m| m.features().contains(f.key)).map(Mode::key).collect();
            json!({ "key": f.key, "title": f.title, "category": f.category, "edits": f.edits, "description": f.description, "modes": modes })
        })
        .collect();
    let ui: Vec<Value> = UI_FLAGS.iter().map(|(k, d, desc)| json!({ "key": k, "default": d, "description": desc })).collect();
    let panels: Vec<Value> = PANELS.iter().map(|(k, d)| json!({ "key": k, "description": d })).collect();
    let elements: Vec<Value> = crate::elements::ELEMENTS.iter().map(|e| json!({ "id": e.id, "hides": e.hides })).collect();
    json!({ "modes": modes, "features": features, "ui": ui, "panels": panels, "disabledElements": elements })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_are_unique_and_modes_name_real_features() {
        let mut seen = BTreeSet::new();
        for f in FEATURES {
            assert!(seen.insert(f.key), "duplicate feature {}", f.key);
        }
        for m in Mode::ALL {
            assert_eq!(Mode::from_key(m.key()), Some(m));
            for k in m.features() {
                assert!(is_feature(k), "{} names unknown feature {k}", m.key());
            }
        }
    }

    #[test]
    fn view_is_read_only_and_annotate_is_annotations_only() {
        let view = FeatureConfig::default().resolve();
        assert!(view.read_only);
        assert!(!view.features.contains("comments"));
        let annotate = FeatureConfig { mode: Mode::Annotate, ..FeatureConfig::default() };
        let r = annotate.resolve();
        assert!(!r.read_only);
        for k in ANNOTATION_FEATURES {
            assert!(r.features.contains(k), "annotate lacks {k}");
        }
        for k in ["pages", "editContent", "formFill", "formDesign", "security", "redaction"] {
            assert!(!r.features.contains(k), "annotate has {k}");
        }
        assert!(annotate.allows_annotation_changes() && !FeatureConfig::default().allows_annotation_changes());
    }

    #[test]
    fn overrides_and_ui_options_apply() {
        let mut c = FeatureConfig { mode: Mode::Annotate, ..FeatureConfig::default() };
        c.features.insert("ink".into(), false);
        c.features.insert("print".into(), false);
        c.features.insert("pages".into(), true);
        c.ui.quick_toolbar = false;
        c.ui.panels.insert("layers".into(), false);
        c.disabled.push("save".into());
        let r = c.resolve();
        assert!(!r.features.contains("ink") && !r.features.contains("print") && r.features.contains("pages"));
        assert!(r.hidden.contains("ui:quick_bar") && r.hidden.contains("panel:layers") && r.hidden.contains("file.save"));
        assert!(r.hidden.contains("ui:mode_tabs") && r.hide_planned && r.tools_rail);
        // Turning every editing feature off makes it read-only.
        let mut ro = FeatureConfig { mode: Mode::Review, ..FeatureConfig::default() };
        for k in ["comments", "markup"] {
            ro.features.insert(k.into(), false);
        }
        assert!(ro.resolve().read_only);
    }

    #[test]
    fn full_mode_leaves_javascript_off() {
        let full = Mode::Full.features();
        assert!(!full.contains("javascript"));
        assert_eq!(full.len(), FEATURES.len() - 1);
    }

    #[test]
    fn the_checked_in_reference_is_current() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/reference/config.json");
        let expected = serde_json::to_string_pretty(&reference()).unwrap_or_default() + "\n";
        let actual = std::fs::read_to_string(path).unwrap_or_default().replace("\r\n", "\n");
        if actual != expected {
            // Regenerate with FPS_WRITE_REFERENCE=1 cargo test -p fps-core.
            if std::env::var_os("FPS_WRITE_REFERENCE").is_some() {
                let _ = std::fs::create_dir_all(concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/reference"));
                let _ = std::fs::write(path, &expected);
                return;
            }
            panic!("docs/reference/config.json is out of date: run FPS_WRITE_REFERENCE=1 cargo test -p fps-core");
        }
    }
}

#[cfg(test)]
mod js_sync {
    use super::*;

    /// The quoted strings between `start` and the next `]` in the wrapper's source.
    fn list(src: &str, start: &str) -> Vec<String> {
        let rest = src.split_once(start).map(|(_, r)| r).unwrap_or_default();
        let body = rest.split_once(']').map(|(b, _)| b).unwrap_or_default();
        body.split('"').skip(1).step_by(2).map(str::to_owned).collect()
    }

    #[test]
    fn the_js_wrapper_lists_the_same_modes_and_features() {
        let src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../js/src/free-pdf-sdk.ts")).unwrap_or_default();
        let modes: Vec<String> = Mode::ALL.iter().map(|m| m.key().to_owned()).collect();
        assert_eq!(list(&src, "export const MODES = ["), modes);
        let features: Vec<String> = FEATURES.iter().map(|f| f.key.to_owned()).collect();
        assert_eq!(list(&src, "export const FEATURES = ["), features);
        let panels = src.split_once("export type PanelName =").and_then(|(_, r)| r.split_once(';')).map(|(b, _)| b).unwrap_or_default();
        let js_panels: Vec<&str> = panels.split('"').skip(1).step_by(2).collect();
        let rust_panels: Vec<&str> = PANELS.iter().map(|(p, _)| *p).collect();
        assert_eq!(js_panels, rust_panels);
    }
}
