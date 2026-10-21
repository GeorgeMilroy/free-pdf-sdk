//! The RPC protocol between the wrapper (host page) and the viewer (iframe), version 1.
//!
//! Every message is a plain object `{ fps: 1, ch, kind, ... }`; `kind` is `hello` (viewer →
//! host, once it listens), `req` (host → viewer: `{ id, method, params }`), `res` (viewer → host:
//! `{ id, ok: true, result }` or `{ id, ok: false, error: { code, message } }`) or `evt` (viewer
//! → host: `{ name, data }`). Binary data travels as a transferred `ArrayBuffer` in
//! `params.data` (requests) or `result` (responses). docs/architecture.md has the full spec.
//!
//! Everything here treats its input as untrusted: [`parse_request`] checks every parameter's
//! type, size and range and never panics.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::error::RpcError;
pub use crate::features::Mode;
use crate::features::{FeatureConfig, UiOptions};

/// The protocol version in every message's `fps` field.
pub const PROTOCOL_VERSION: u32 = 1;

/// Size limits (bytes or characters).
pub mod limits {
    /// A PDF.
    pub const MAX_PDF_BYTES: usize = 512 * 1024 * 1024;
    /// An XFDF document.
    pub const MAX_XFDF_BYTES: usize = 32 * 1024 * 1024;
    /// A document name.
    pub const MAX_NAME_CHARS: usize = 512;
    /// An author name (pdfcraft's own limit).
    pub const MAX_AUTHOR_CHARS: usize = 200;
    /// A password.
    pub const MAX_PASSWORD_CHARS: usize = 1024;
    /// A locale tag.
    pub const MAX_LOCALE_CHARS: usize = 35;
    /// `disabledElements` entries.
    pub const MAX_ELEMENTS: usize = 256;
    /// A method name.
    pub const MAX_METHOD_CHARS: usize = 64;
    /// Zoom, in percent.
    pub const MIN_ZOOM: f64 = 10.0;
    pub const MAX_ZOOM: f64 = 6400.0;
    /// Pages a document may have (a sanity bound for `goToPage`).
    pub const MAX_PAGE: u64 = 1_000_000;
}

/// Colour theme.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
    System,
}

/// A zoom request.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Zoom {
    /// Percent (100 = actual size).
    Percent(f32),
    FitWidth,
    FitPage,
}

/// Settings sent by `configure` (the `init` options other than the document).
#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    /// Mode, feature overrides, interface options and `disabledElements`.
    pub features: FeatureConfig,
    pub theme: Theme,
    pub author: Option<String>,
    pub locale: Option<String>,
}

/// Interface options a request changes; options it doesn't name keep their value.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiPatch {
    pub flags: BTreeMap<String, bool>,
    pub panels: BTreeMap<String, bool>,
}

impl UiPatch {
    /// `ui` with this patch applied.
    pub fn apply(&self, mut ui: UiOptions) -> UiOptions {
        for (key, value) in &self.flags {
            let v = *value;
            match key.as_str() {
                "toolsPanel" => ui.tools_panel = v,
                "toolsRail" => ui.tools_rail = v,
                "quickToolbar" => ui.quick_toolbar = v,
                "rightRail" => ui.right_rail = v,
                "zoomControls" => ui.zoom_controls = v,
                "pageNavigation" => ui.page_navigation = v,
                "menu" => ui.menu = v,
                "searchBox" => ui.search_box = v,
                "lockLayout" => ui.lock_layout = v,
                "showPlannedTools" => ui.show_planned_tools = v,
                "externalLinks" => ui.external_links = v,
                _ => {}
            }
        }
        for (name, on) in &self.panels {
            ui.panels.insert(name.clone(), *on);
        }
        ui
    }
}

/// A validated request.
#[derive(Clone, Debug, PartialEq)]
pub enum Request {
    Configure(Config),
    Load {
        name: String,
        data: Vec<u8>,
        password: Option<String>,
    },
    Unlock {
        password: String,
    },
    GetPermissions,
    ImportXfdf {
        xfdf: String,
        replace: bool,
    },
    ExportXfdf,
    GetAnnotations,
    DeleteAllAnnotations,
    SetAuthor {
        author: String,
    },
    SetTheme {
        theme: Theme,
    },
    SetMode {
        mode: Mode,
    },
    /// Replaces the feature overrides (relative to the mode).
    SetFeatures {
        features: BTreeMap<String, bool>,
    },
    /// Changes the given interface options.
    SetUi {
        ui: UiPatch,
    },
    GetConfig,
    SetDisabledElements {
        ids: Vec<String>,
    },
    SetLocale {
        locale: String,
    },
    GetPageCount,
    GoToPage {
        page: u32,
    },
    SetZoom {
        zoom: Zoom,
    },
    IsDirty,
    Save,
    Version,
}

impl Request {
    /// Whether the request changes the document (refused in view mode).
    pub fn mutates_document(&self) -> bool {
        matches!(self, Self::ImportXfdf { .. } | Self::DeleteAllAnnotations)
    }

    /// Refuses requests the configuration doesn't allow: importing XFDF needs the `comments`
    /// feature (the viewer treats imported XFDF as comment data), and deleting every comment
    /// needs some annotation feature (the viewer then checks each annotation's own kind).
    pub fn check_config(&self, config: &FeatureConfig) -> Result<(), RpcError> {
        match self {
            Self::ImportXfdf { .. } if !config.enabled().contains("comments") => {
                Err(RpcError::new(crate::ErrorCode::ReadOnly, "importing XFDF needs the comments feature"))
            }
            Self::DeleteAllAnnotations if !config.allows_annotation_changes() => Err(RpcError::read_only()),
            _ => Ok(()),
        }
    }

    /// Whether the request needs an open document.
    pub fn needs_document(&self) -> bool {
        matches!(
            self,
            Self::Unlock { .. }
                | Self::GetPermissions
                | Self::ImportXfdf { .. }
                | Self::ExportXfdf
                | Self::GetAnnotations
                | Self::DeleteAllAnnotations
                | Self::GetPageCount
                | Self::GoToPage { .. }
                | Self::SetZoom { .. }
                | Self::Save
        )
    }
}

/// The documented method names.
pub const METHODS: &[&str] = &[
    "configure",
    "load",
    "unlock",
    "getPermissions",
    "importXfdf",
    "exportXfdf",
    "getAnnotations",
    "deleteAllAnnotations",
    "setAuthor",
    "setTheme",
    "setMode",
    "setFeatures",
    "setUi",
    "getConfig",
    "setDisabledElements",
    "setLocale",
    "getPageCount",
    "goToPage",
    "setZoom",
    "isDirty",
    "save",
    "version",
];

/// Parses a request. `params` must be an object (or absent, as `null`); `data` holds the bytes
/// the wrapper transferred in `params.data`, already taken out of `params`.
pub fn parse_request(method: &str, params: &Value, data: Option<Vec<u8>>) -> Result<Request, RpcError> {
    if method.chars().count() > limits::MAX_METHOD_CHARS {
        return Err(RpcError::new(crate::ErrorCode::UnknownMethod, "method name too long"));
    }
    let p = Params::new(params)?;
    let req = match method {
        "configure" => Request::Configure(Config {
            features: FeatureConfig {
                mode: p.opt_str("mode", 16)?.map(|s| mode(&s)).transpose()?.unwrap_or(Mode::View),
                features: p.opt_features("features")?.unwrap_or_default(),
                ui: p.opt_ui("ui")?.unwrap_or_default().apply(UiOptions::default()),
                disabled: p.opt_elements("disabledElements")?.unwrap_or_default(),
            },
            theme: p.opt_str("theme", 16)?.map(|s| theme(&s)).transpose()?.unwrap_or(Theme::System),
            author: p.opt_str("author", limits::MAX_AUTHOR_CHARS)?.map(|a| author(&a)).transpose()?,
            locale: p.opt_str("locale", limits::MAX_LOCALE_CHARS)?.map(|l| locale(&l)).transpose()?,
        }),
        "load" => {
            let Some(data) = data else { return Err(RpcError::bad_request("load needs params.data: the PDF as an ArrayBuffer")) };
            if data.len() > limits::MAX_PDF_BYTES {
                return Err(RpcError::new(crate::ErrorCode::TooLarge, format!("the PDF is larger than {} MiB", limits::MAX_PDF_BYTES / 1024 / 1024)));
            }
            if data.is_empty() {
                return Err(RpcError::bad_request("the PDF is empty"));
            }
            let name = p.opt_str("name", limits::MAX_NAME_CHARS)?.map(|n| sanitize_name(&n)).filter(|n| !n.is_empty());
            let password = p.opt_str("password", limits::MAX_PASSWORD_CHARS)?;
            Request::Load { name: name.unwrap_or_else(|| "document.pdf".to_owned()), data, password }
        }
        "unlock" => Request::Unlock { password: p.req_str("password", limits::MAX_PASSWORD_CHARS)? },
        "getPermissions" => Request::GetPermissions,
        "importXfdf" => {
            let xfdf = p.req_str("xfdf", limits::MAX_XFDF_BYTES)?;
            if xfdf.len() > limits::MAX_XFDF_BYTES {
                return Err(RpcError::new(crate::ErrorCode::TooLarge, "the XFDF is larger than 32 MiB"));
            }
            Request::ImportXfdf { xfdf, replace: p.opt_bool("replace")?.unwrap_or(false) }
        }
        "exportXfdf" => Request::ExportXfdf,
        "getAnnotations" => Request::GetAnnotations,
        "deleteAllAnnotations" => Request::DeleteAllAnnotations,
        "setAuthor" => Request::SetAuthor { author: author(&p.req_str("author", limits::MAX_AUTHOR_CHARS)?)? },
        "setTheme" => Request::SetTheme { theme: theme(&p.req_str("theme", 16)?)? },
        "setMode" => Request::SetMode { mode: mode(&p.req_str("mode", 16)?)? },
        "setFeatures" => Request::SetFeatures { features: p.opt_features("features")?.unwrap_or_default() },
        "setUi" => Request::SetUi { ui: p.opt_ui("ui")?.unwrap_or_default() },
        "getConfig" => Request::GetConfig,
        "setDisabledElements" => Request::SetDisabledElements { ids: p.opt_elements("ids")?.unwrap_or_default() },
        "setLocale" => Request::SetLocale { locale: locale(&p.req_str("locale", limits::MAX_LOCALE_CHARS)?)? },
        "getPageCount" => Request::GetPageCount,
        "goToPage" => {
            let page = p.req_u64("page")?;
            if page == 0 || page > limits::MAX_PAGE {
                return Err(RpcError::bad_request("page must be a page number from 1"));
            }
            Request::GoToPage { page: u32::try_from(page).map_err(|_| RpcError::bad_request("page out of range"))? }
        }
        "setZoom" => Request::SetZoom { zoom: zoom(p.get("zoom"))? },
        "isDirty" => Request::IsDirty,
        "save" => Request::Save,
        "version" => Request::Version,
        other => return Err(RpcError::new(crate::ErrorCode::UnknownMethod, format!("unknown method {:?}", truncate(other, 64)))),
    };
    Ok(req)
}

/// A typed view of a params object.
struct Params<'a>(Option<&'a serde_json::Map<String, Value>>);

impl<'a> Params<'a> {
    fn new(v: &'a Value) -> Result<Self, RpcError> {
        match v {
            Value::Null => Ok(Self(None)),
            Value::Object(m) => Ok(Self(Some(m))),
            _ => Err(RpcError::bad_request("params must be an object")),
        }
    }

    fn get(&self, key: &str) -> Option<&'a Value> {
        self.0.and_then(|m| m.get(key)).filter(|v| !v.is_null())
    }

    fn opt_str(&self, key: &str, max_chars: usize) -> Result<Option<String>, RpcError> {
        match self.get(key) {
            None => Ok(None),
            Some(Value::String(s)) => {
                if s.chars().count() > max_chars {
                    return Err(RpcError::new(crate::ErrorCode::TooLarge, format!("{key} is longer than {max_chars} characters")));
                }
                Ok(Some(s.clone()))
            }
            Some(_) => Err(RpcError::bad_request(format!("{key} must be a string"))),
        }
    }

    fn req_str(&self, key: &str, max_chars: usize) -> Result<String, RpcError> {
        self.opt_str(key, max_chars)?.ok_or_else(|| RpcError::bad_request(format!("{key} is required")))
    }

    fn opt_bool(&self, key: &str) -> Result<Option<bool>, RpcError> {
        match self.get(key) {
            None => Ok(None),
            Some(Value::Bool(b)) => Ok(Some(*b)),
            Some(_) => Err(RpcError::bad_request(format!("{key} must be a boolean"))),
        }
    }

    fn req_u64(&self, key: &str) -> Result<u64, RpcError> {
        match self.get(key) {
            Some(Value::Number(n)) => n.as_u64().ok_or_else(|| RpcError::bad_request(format!("{key} must be a whole number"))),
            Some(_) => Err(RpcError::bad_request(format!("{key} must be a number"))),
            None => Err(RpcError::bad_request(format!("{key} is required"))),
        }
    }

    /// `{ featureKey: boolean }`, every key a documented feature.
    fn opt_features(&self, key: &str) -> Result<Option<BTreeMap<String, bool>>, RpcError> {
        let Some(v) = self.get(key) else { return Ok(None) };
        let Value::Object(m) = v else { return Err(RpcError::bad_request(format!("{key} must be an object of booleans"))) };
        let mut out = BTreeMap::new();
        for (k, v) in m {
            if !crate::features::is_feature(k) {
                return Err(RpcError::bad_request(format!("unknown feature {:?} in {key}", truncate(k, 64))));
            }
            let Value::Bool(b) = v else { return Err(RpcError::bad_request(format!("{key}.{} must be a boolean", truncate(k, 64)))) };
            out.insert(k.clone(), *b);
        }
        Ok(Some(out))
    }

    /// The `ui` options object: documented boolean options, and `panels` (an object of booleans).
    fn opt_ui(&self, key: &str) -> Result<Option<UiPatch>, RpcError> {
        let Some(v) = self.get(key) else { return Ok(None) };
        let Value::Object(m) = v else { return Err(RpcError::bad_request(format!("{key} must be an object"))) };
        let mut patch = UiPatch::default();
        for (k, v) in m {
            if k == "panels" {
                let Value::Object(panels) = v else { return Err(RpcError::bad_request(format!("{key}.panels must be an object of booleans"))) };
                for (name, on) in panels {
                    if !crate::features::PANELS.iter().any(|(p, _)| p == name) {
                        return Err(RpcError::bad_request(format!("unknown panel {:?} in {key}.panels", truncate(name, 64))));
                    }
                    let Value::Bool(b) = on else {
                        return Err(RpcError::bad_request(format!("{key}.panels.{} must be a boolean", truncate(name, 64))));
                    };
                    patch.panels.insert(name.clone(), *b);
                }
                continue;
            }
            if !crate::features::UI_FLAGS.iter().any(|(f, ..)| f == k) {
                return Err(RpcError::bad_request(format!("unknown option {:?} in {key}", truncate(k, 64))));
            }
            let Value::Bool(b) = v else { return Err(RpcError::bad_request(format!("{key}.{} must be a boolean", truncate(k, 64)))) };
            patch.flags.insert(k.clone(), *b);
        }
        Ok(Some(patch))
    }

    fn opt_elements(&self, key: &str) -> Result<Option<Vec<String>>, RpcError> {
        let Some(v) = self.get(key) else { return Ok(None) };
        let Value::Array(items) = v else { return Err(RpcError::bad_request(format!("{key} must be an array of strings"))) };
        if items.len() > limits::MAX_ELEMENTS {
            return Err(RpcError::new(crate::ErrorCode::TooLarge, format!("{key} has more than {} entries", limits::MAX_ELEMENTS)));
        }
        let mut out = Vec::with_capacity(items.len());
        for item in items {
            let Value::String(s) = item else { return Err(RpcError::bad_request(format!("{key} must be an array of strings"))) };
            if crate::elements::find(s).is_none() {
                return Err(RpcError::bad_request(format!("unknown element id {:?} in {key}", truncate(s, 64))));
            }
            if !out.contains(s) {
                out.push(s.clone());
            }
        }
        Ok(Some(out))
    }
}

fn mode(s: &str) -> Result<Mode, RpcError> {
    Mode::from_key(s).ok_or_else(|| RpcError::bad_request("mode must be one of view, review, annotate, fill, sign, organize, edit or full"))
}

fn theme(s: &str) -> Result<Theme, RpcError> {
    match s {
        "light" => Ok(Theme::Light),
        "dark" => Ok(Theme::Dark),
        "system" => Ok(Theme::System),
        _ => Err(RpcError::bad_request("theme must be \"light\", \"dark\" or \"system\"")),
    }
}

fn zoom(v: Option<&Value>) -> Result<Zoom, RpcError> {
    match v {
        Some(Value::String(s)) if s == "fit-width" => Ok(Zoom::FitWidth),
        Some(Value::String(s)) if s == "fit-page" => Ok(Zoom::FitPage),
        Some(Value::Number(n)) => {
            let z = n.as_f64().filter(|z| z.is_finite()).ok_or_else(|| RpcError::bad_request("zoom must be a finite number"))?;
            if !(limits::MIN_ZOOM..=limits::MAX_ZOOM).contains(&z) {
                return Err(RpcError::bad_request(format!("zoom must be from {} to {} percent", limits::MIN_ZOOM, limits::MAX_ZOOM)));
            }
            #[allow(clippy::cast_possible_truncation)] // in [10, 6400]: exact enough in f32
            Ok(Zoom::Percent(z as f32))
        }
        _ => Err(RpcError::bad_request("zoom must be a percentage, \"fit-width\" or \"fit-page\"")),
    }
}

/// An author name: trimmed, control characters removed, not empty.
fn author(s: &str) -> Result<String, RpcError> {
    let clean: String = s.chars().filter(|c| !c.is_control()).collect::<String>().trim().to_owned();
    if clean.is_empty() {
        return Err(RpcError::bad_request("author must not be empty"));
    }
    Ok(clean)
}

/// A BCP 47-ish tag: letters, digits and '-' only.
fn locale(s: &str) -> Result<String, RpcError> {
    if s.is_empty() || !s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return Err(RpcError::bad_request("locale must be a language tag such as \"en\" or \"de-DE\""));
    }
    Ok(s.to_ascii_lowercase().replace('_', "-"))
}

/// A file name with path separators and control characters removed.
fn sanitize_name(s: &str) -> String {
    let base = s.rsplit(['/', '\\']).next().unwrap_or(s);
    base.chars().filter(|c| !c.is_control()).collect::<String>().trim().to_owned()
}

/// At most `n` characters of `s` (for error messages that quote input).
pub fn truncate(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ErrorCode;
    use serde_json::json;

    fn parse(method: &str, params: Value) -> Result<Request, RpcError> {
        parse_request(method, &params, None)
    }

    fn code(r: Result<Request, RpcError>) -> Option<ErrorCode> {
        r.err().map(|e| e.code)
    }

    #[test]
    fn parses_every_documented_method() {
        let data = Some(vec![b'%'; 8]);
        for method in METHODS {
            let params = match *method {
                "unlock" => json!({ "password": "x" }),
                "importXfdf" => json!({ "xfdf": "<xfdf/>" }),
                "setAuthor" => json!({ "author": "A" }),
                "setTheme" => json!({ "theme": "dark" }),
                "setMode" => json!({ "mode": "annotate" }),
                "setLocale" => json!({ "locale": "de" }),
                "goToPage" => json!({ "page": 2 }),
                "setZoom" => json!({ "zoom": 150 }),
                _ => json!({}),
            };
            let r = parse_request(method, &params, data.clone());
            assert!(r.is_ok(), "{method}: {r:?}");
        }
    }

    #[test]
    fn rejects_unknown_methods_and_non_object_params() {
        assert_eq!(code(parse("nope", json!({}))), Some(ErrorCode::UnknownMethod));
        assert_eq!(code(parse(&"x".repeat(100), json!({}))), Some(ErrorCode::UnknownMethod));
        assert_eq!(code(parse("save", json!([1, 2]))), Some(ErrorCode::BadRequest));
        assert_eq!(code(parse("save", json!("s"))), Some(ErrorCode::BadRequest));
        assert_eq!(parse("save", Value::Null), Ok(Request::Save));
    }

    #[test]
    fn checks_types_and_ranges() {
        assert_eq!(code(parse("unlock", json!({}))), Some(ErrorCode::BadRequest));
        assert_eq!(code(parse("unlock", json!({ "password": 5 }))), Some(ErrorCode::BadRequest));
        assert_eq!(code(parse("unlock", json!({ "password": "x".repeat(2000) }))), Some(ErrorCode::TooLarge));
        assert_eq!(code(parse("goToPage", json!({ "page": 0 }))), Some(ErrorCode::BadRequest));
        assert_eq!(code(parse("goToPage", json!({ "page": -3 }))), Some(ErrorCode::BadRequest));
        assert_eq!(code(parse("goToPage", json!({ "page": 1.5 }))), Some(ErrorCode::BadRequest));
        assert_eq!(code(parse("goToPage", json!({ "page": "2" }))), Some(ErrorCode::BadRequest));
        assert_eq!(parse("goToPage", json!({ "page": 3 })), Ok(Request::GoToPage { page: 3 }));
        assert_eq!(code(parse("setZoom", json!({ "zoom": 5 }))), Some(ErrorCode::BadRequest));
        assert_eq!(code(parse("setZoom", json!({ "zoom": 1e9 }))), Some(ErrorCode::BadRequest));
        assert_eq!(code(parse("setZoom", json!({ "zoom": "fit" }))), Some(ErrorCode::BadRequest));
        assert_eq!(parse("setZoom", json!({ "zoom": "fit-width" })), Ok(Request::SetZoom { zoom: Zoom::FitWidth }));
        assert_eq!(code(parse("setTheme", json!({ "theme": "blue" }))), Some(ErrorCode::BadRequest));
        assert_eq!(code(parse("setMode", json!({ "mode": "draw" }))), Some(ErrorCode::BadRequest));
        assert_eq!(parse("setMode", json!({ "mode": "organize" })), Ok(Request::SetMode { mode: Mode::Organize }));
        assert_eq!(code(parse("importXfdf", json!({ "xfdf": "<x/>", "replace": "yes" }))), Some(ErrorCode::BadRequest));
        assert_eq!(code(parse("setLocale", json!({ "locale": "de;rm -rf" }))), Some(ErrorCode::BadRequest));
    }

    #[test]
    fn validates_authors_and_names() {
        assert_eq!(parse("setAuthor", json!({ "author": "  Ann\u{0007} Lee \n" })), Ok(Request::SetAuthor { author: "Ann Lee".into() }));
        assert_eq!(code(parse("setAuthor", json!({ "author": "  \n" }))), Some(ErrorCode::BadRequest));
        assert_eq!(code(parse("setAuthor", json!({ "author": "a".repeat(201) }))), Some(ErrorCode::TooLarge));
        let r = parse_request("load", &json!({ "name": "../../etc/x.pdf" }), Some(b"%PDF".to_vec()));
        assert!(matches!(r, Ok(Request::Load { ref name, .. }) if name == "x.pdf"), "{r:?}");
        let r = parse_request("load", &json!({}), Some(b"%PDF".to_vec()));
        assert!(matches!(r, Ok(Request::Load { ref name, password: None, .. }) if name == "document.pdf"));
        assert_eq!(code(parse_request("load", &json!({}), None)), Some(ErrorCode::BadRequest));
        assert_eq!(code(parse_request("load", &json!({}), Some(Vec::new()))), Some(ErrorCode::BadRequest));
    }

    #[test]
    fn checks_disabled_elements() {
        let r = parse("configure", json!({ "disabledElements": ["print", "print", "tool:stamp"], "mode": "annotate", "author": "Q" }));
        assert_eq!(
            r,
            Ok(Request::Configure(Config {
                features: FeatureConfig { mode: Mode::Annotate, disabled: vec!["print".into(), "tool:stamp".into()], ..FeatureConfig::default() },
                theme: Theme::System,
                author: Some("Q".into()),
                locale: None,
            }))
        );
        assert_eq!(code(parse("configure", json!({ "disabledElements": ["nope"] }))), Some(ErrorCode::BadRequest));
        assert_eq!(code(parse("configure", json!({ "disabledElements": "print" }))), Some(ErrorCode::BadRequest));
        assert_eq!(code(parse("configure", json!({ "disabledElements": [1] }))), Some(ErrorCode::BadRequest));
        let many: Vec<&str> = std::iter::repeat_n("print", 300).collect();
        assert_eq!(code(parse("configure", json!({ "disabledElements": many }))), Some(ErrorCode::TooLarge));
        // Defaults: view mode, system theme.
        let Ok(Request::Configure(c)) = parse("configure", json!({})) else { panic!("configure") };
        assert_eq!((c.features.mode, c.theme), (Mode::View, Theme::System));
    }

    #[test]
    fn checks_features_and_ui_options() {
        let r = parse(
            "configure",
            json!({ "mode": "fill", "features": { "comments": true, "print": false }, "ui": { "quickToolbar": false, "panels": { "layers": false } } }),
        );
        let Ok(Request::Configure(c)) = r else { panic!("{r:?}") };
        assert!(c.features.enabled().contains("comments") && !c.features.enabled().contains("print"));
        assert!(!c.features.ui.quick_toolbar && c.features.ui.panels.get("layers") == Some(&false));
        assert_eq!(code(parse("configure", json!({ "features": { "teleport": true } }))), Some(ErrorCode::BadRequest));
        assert_eq!(code(parse("configure", json!({ "features": { "comments": "yes" } }))), Some(ErrorCode::BadRequest));
        assert_eq!(code(parse("configure", json!({ "features": ["comments"] }))), Some(ErrorCode::BadRequest));
        assert_eq!(code(parse("configure", json!({ "ui": { "wings": false } }))), Some(ErrorCode::BadRequest));
        assert_eq!(code(parse("configure", json!({ "ui": { "panels": { "spaceship": false } } }))), Some(ErrorCode::BadRequest));
        assert_eq!(code(parse("setUi", json!({ "ui": { "menu": 0 } }))), Some(ErrorCode::BadRequest));
        let r = parse("setUi", json!({ "ui": { "menu": false } }));
        let Ok(Request::SetUi { ui }) = r else { panic!("{r:?}") };
        assert!(!ui.apply(UiOptions::default()).menu);
    }

    #[test]
    fn only_import_and_delete_mutate() {
        assert!(Request::DeleteAllAnnotations.mutates_document());
        assert!(Request::ImportXfdf { xfdf: String::new(), replace: true }.mutates_document());
        assert!(!Request::Save.mutates_document());
        assert!(!Request::ExportXfdf.mutates_document());
    }

    #[test]
    fn configurations_without_annotations_refuse_every_mutation() {
        let view = FeatureConfig::default();
        let organize = FeatureConfig { mode: Mode::Organize, ..FeatureConfig::default() };
        let annotate = FeatureConfig { mode: Mode::Annotate, ..FeatureConfig::default() };
        let mutating = [Request::DeleteAllAnnotations, Request::ImportXfdf { xfdf: "<xfdf/>".into(), replace: false }];
        for r in &mutating {
            assert_eq!(r.check_config(&view).err().map(|e| e.code), Some(ErrorCode::ReadOnly));
            assert_eq!(r.check_config(&organize).err().map(|e| e.code), Some(ErrorCode::ReadOnly));
            assert_eq!(r.check_config(&annotate), Ok(()));
        }
        // Importing needs comments: fill (Fill & Sign only) and annotate without comments refuse it.
        let import = Request::ImportXfdf { xfdf: "<xfdf/>".into(), replace: false };
        let fill = FeatureConfig { mode: Mode::Fill, ..FeatureConfig::default() };
        let mut no_comments = FeatureConfig { mode: Mode::Annotate, ..FeatureConfig::default() };
        no_comments.features.insert("comments".into(), false);
        assert_eq!(import.check_config(&fill).err().map(|e| e.code), Some(ErrorCode::ReadOnly));
        assert_eq!(import.check_config(&no_comments).err().map(|e| e.code), Some(ErrorCode::ReadOnly));
        assert_eq!(Request::DeleteAllAnnotations.check_config(&fill), Ok(()));
        for r in [Request::Save, Request::ExportXfdf, Request::GetAnnotations, Request::GetPermissions, Request::IsDirty] {
            assert_eq!(r.check_config(&view), Ok(()), "{r:?} reads only");
        }
    }
}
