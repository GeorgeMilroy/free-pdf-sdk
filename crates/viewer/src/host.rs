//! The viewer app: pdfcraft's `PdfCraftApp`, wrapped so the RPC requests queued by the bridge
//! run between frames with full access to it, and so state changes become events.

use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use fps_core::doc;
use fps_core::protocol::{Config, Mode, Request, Theme, Zoom, parse_request};
use fps_core::{ErrorCode, RpcError};
use pdfcraft_engine::DocId;
use pdfcraft_ui_egui::PdfCraftApp;
use pdfcraft_ui_egui::theme::ThemePreference;
use serde_json::{Value, json};

use crate::bridge::{Bridge, Inbox, Payload};

/// How long a load may wait for pdfcraft to finish opening (it opens synchronously on the web
/// today; this guards against that changing).
const LOAD_TIMEOUT_SECS: f64 = 60.0;

/// Saves the UI's Save button handed over (`host_save`), waiting to become events.
type SaveQueue = Rc<RefCell<Vec<(String, Vec<u8>, bool)>>>;

struct PendingLoad {
    id: u32,
    owner_password: Option<String>,
    started: f64,
}

pub struct Viewer {
    app: PdfCraftApp,
    bridge: Bridge,
    inbox: Inbox,
    saves: SaveQueue,
    mode: Mode,
    author: Option<String>,
    disabled: Vec<String>,
    pending_load: Option<PendingLoad>,
    last_dirty: bool,
    /// The open document, its edit generation and a fingerprint of its comments.
    last_comments: Option<(DocId, u64, u64)>,
    now: f64,
}

impl Viewer {
    pub fn new(app: PdfCraftApp, bridge: Bridge, inbox: Inbox) -> Self {
        let mut viewer = Self {
            app,
            bridge,
            inbox,
            saves: Rc::new(RefCell::new(Vec::new())),
            mode: Mode::View,
            author: None,
            disabled: Vec::new(),
            pending_load: None,
            last_dirty: false,
            last_comments: None,
            now: 0.0,
        };
        // The UI's Save hands the saved PDF to the host (event `saveRequested`); the viewer never
        // downloads files on its own.
        let saves = Rc::clone(&viewer.saves);
        viewer.app.host_save = Some(Box::new(move |name: &str, bytes: &[u8], save_as: bool| {
            saves.try_borrow_mut().map_err(|_| "busy".to_owned())?.push((name.to_owned(), bytes.to_vec(), save_as));
            Ok(())
        }));
        viewer
    }

    fn active(&self) -> Option<DocId> {
        self.app.active_ids().map(|(_, id)| id)
    }

    /// Runs the queued requests, finishes a pending load and posts events.
    fn pump(&mut self, ctx: &egui::Context) {
        self.now = ctx.input(|i| i.time);
        loop {
            let next = self.inbox.try_borrow_mut().ok().and_then(|mut q| q.pop_front());
            let Some(req) = next else { break };
            let id = req.id;
            match parse_request(&req.method, &req.params, req.data).and_then(|r| self.dispatch(id, r)) {
                Ok(Some(payload)) => self.bridge.reply_ok(id, payload),
                Ok(None) => {} // answered later (load)
                Err(e) => self.bridge.reply_err(id, &e),
            }
        }
        self.poll_load();
        self.emit_changes();
    }

    fn doc_id(&self) -> Result<DocId, RpcError> {
        if self.pending_load.is_some() {
            return Err(RpcError::new(ErrorCode::Busy, "a document is loading"));
        }
        self.active().ok_or_else(RpcError::no_document)
    }

    fn dispatch(&mut self, id: u32, req: Request) -> Result<Option<Payload>, RpcError> {
        req.check_mode(self.mode)?;
        if req.needs_document() {
            self.doc_id()?;
        }
        let ok = |v: Value| Ok(Some(Payload::Json(v)));
        match req {
            Request::Configure(cfg) => {
                self.configure(cfg);
                ok(json!({}))
            }
            Request::Load { name, data, password } => self.load(id, &name, data, password),
            Request::Unlock { password } => {
                let doc_id = self.doc_id()?;
                let d = self.app.session.get(doc_id).ok_or_else(RpcError::no_document)?;
                if d.can_unlock() {
                    self.app.session.unlock(doc_id, &password).map_err(|e| fps_core::error::from_edit_error(&e))?;
                }
                ok(json!({ "permissions": self.permissions()? }))
            }
            Request::GetPermissions => ok(serde_json::to_value(self.permissions()?).unwrap_or(Value::Null)),
            Request::ImportXfdf { xfdf, replace } => {
                let doc_id = self.doc_id()?;
                let before = doc::annotations(&self.app.session, doc_id)?;
                let d = self.app.session.get(doc_id).ok_or_else(RpcError::no_document)?;
                let plan = doc::plan_import(d, &xfdf, replace)?;
                if let Some(edit) = plan.edit {
                    self.apply(edit)?;
                }
                let after = doc::annotations(&self.app.session, doc_id)?;
                let imported = doc::count_imported(&after, if replace { &[] } else { &before }, &plan.expected);
                ok(json!(imported))
            }
            Request::ExportXfdf => ok(json!(doc::export_xfdf(&self.app.session, self.doc_id()?)?)),
            Request::GetAnnotations => {
                let list = doc::annotations(&self.app.session, self.doc_id()?)?;
                ok(serde_json::to_value(list).unwrap_or(Value::Null))
            }
            Request::DeleteAllAnnotations => {
                let doc_id = self.doc_id()?;
                let d = self.app.session.get(doc_id).ok_or_else(RpcError::no_document)?;
                doc::check_can_annotate(d)?;
                let count = doc::annotations(&self.app.session, doc_id)?.len();
                if let Some(edit) = doc::delete_all_edit(d) {
                    self.apply(edit)?;
                }
                ok(json!(count))
            }
            Request::SetAuthor { author } => {
                self.app.comment_prefs.author = author.clone();
                self.author = Some(author);
                ok(json!({}))
            }
            Request::SetTheme { theme } => {
                self.set_theme(theme);
                ok(json!({}))
            }
            Request::SetMode { mode } => {
                self.set_mode(mode);
                ok(json!({}))
            }
            Request::SetDisabledElements { ids } => {
                self.disabled = ids;
                self.apply_profile();
                ok(json!({}))
            }
            Request::SetLocale { locale } => ok(json!({ "locale": self.set_locale(&locale) })),
            Request::GetPageCount => {
                let d = self.app.session.get(self.doc_id()?).ok_or_else(RpcError::no_document)?;
                ok(json!(doc::page_count(d)))
            }
            Request::GoToPage { page } => {
                let d = self.app.session.get(self.doc_id()?).ok_or_else(RpcError::no_document)?;
                let count = doc::page_count(d);
                if !usize::try_from(page).is_ok_and(|p| p <= count) {
                    return Err(RpcError::bad_request(format!("page {page} is past the last page ({count})")));
                }
                self.app.set_option("page", &page.to_string()).map_err(RpcError::internal)?;
                ok(json!({}))
            }
            Request::SetZoom { zoom } => {
                let fit = match zoom {
                    Zoom::Percent(p) => {
                        self.app.set_option("zoom", &format!("{p}")).map_err(RpcError::internal)?;
                        return ok(json!({}));
                    }
                    Zoom::FitWidth => pdfcraft_ui_egui::canvas::Fit::Width,
                    Zoom::FitPage => pdfcraft_ui_egui::canvas::Fit::Page,
                };
                let view = self.app.active.and_then(|i| self.app.views.get_mut(i)).ok_or_else(RpcError::no_document)?;
                view.set_fit(fit);
                ok(json!({}))
            }
            Request::IsDirty => ok(json!(self.app.first_dirty().is_some())),
            Request::Save => {
                let doc_id = self.doc_id()?;
                let bytes = doc::save_bytes(&self.app.session, doc_id)?;
                self.app.session.mark_saved(doc_id, std::sync::Arc::clone(&bytes), None).map_err(|e| fps_core::error::from_edit_error(&e))?;
                Ok(Some(Payload::Bytes(bytes.as_ref().clone())))
            }
            Request::Version => ok(json!({ "sdk": fps_core::SDK_VERSION, "pdfcraftRev": fps_core::PDFCRAFT_REV })),
        }
    }

    fn permissions(&self) -> Result<doc::Permissions, RpcError> {
        let d = self.app.session.get(self.doc_id()?).ok_or_else(RpcError::no_document)?;
        Ok(doc::permissions(d))
    }

    /// Applies an edit through the app, so the canvas, undo and panels follow.
    fn apply(&mut self, edit: pdfcraft_engine::Edit) -> Result<(), RpcError> {
        self.app.toast = None;
        if self.app.apply_edit(edit) {
            return Ok(());
        }
        let message = self.app.toast.take().map_or_else(|| "the change could not be made".to_owned(), |(m, _)| m);
        Err(RpcError::internal(message))
    }

    fn configure(&mut self, cfg: Config) {
        self.set_mode(cfg.mode);
        self.set_theme(cfg.theme);
        if let Some(author) = cfg.author {
            self.app.comment_prefs.author = author.clone();
            self.author = Some(author);
        }
        self.disabled = cfg.disabled_elements;
        self.apply_profile();
        if let Some(locale) = cfg.locale {
            self.set_locale(&locale);
        }
    }

    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
        self.apply_profile();
    }

    /// The UI profile for the mode and the disabled elements (pdfcraft UI profile hook; phase 5).
    fn apply_profile(&mut self) {}

    fn set_theme(&mut self, theme: Theme) {
        let (pref, attr) = match theme {
            Theme::Light => (ThemePreference::Light, "light"),
            Theme::Dark => (ThemePreference::Dark, "dark"),
            Theme::System => (ThemePreference::System, "system"),
        };
        self.app.set_theme_preference(pref);
        // The page behind the canvas, for the first paint and resizes.
        if let Some(root) = web_sys::window().and_then(|w| w.document()).and_then(|d| d.document_element()) {
            let _ = root.set_attribute("data-theme", attr);
        }
    }

    /// Applies the closest language pdfcraft has; returns the code in use.
    fn set_locale(&mut self, locale: &str) -> String {
        let primary = locale.split('-').next().unwrap_or(locale);
        for candidate in [locale, primary, "en"] {
            if self.app.set_option("language", candidate).is_ok() {
                return candidate.to_owned();
            }
        }
        "en".to_owned()
    }

    fn close_all(&mut self) {
        // A password prompt for the previous file must not catch the next one.
        self.app.submit_password(None);
        for i in (0..self.app.views.len()).rev() {
            self.app.close_tab(i);
        }
        self.last_comments = None;
    }

    fn load(&mut self, id: u32, name: &str, data: Vec<u8>, password: Option<String>) -> Result<Option<Payload>, RpcError> {
        if self.pending_load.is_some() {
            return Err(RpcError::new(ErrorCode::Busy, "a document is already loading"));
        }
        self.close_all();
        if let Err(e) = self.app.open_bytes(name, None, data) {
            let err = RpcError::new(ErrorCode::LoadFailed, e);
            self.document_error(&err);
            return Err(err);
        }
        // A user password: try the one given (it may be the owner password, which also opens).
        if self.app.password_prompt.is_some() {
            let Some(pw) = password.clone() else {
                self.app.submit_password(None);
                let err = RpcError::new(ErrorCode::NeedsPassword, "the document needs a password to open");
                self.document_error(&err);
                return Err(err);
            };
            self.app.submit_password(Some(pw));
            if self.app.password_prompt.is_some() {
                self.app.submit_password(None);
                let err = RpcError::new(ErrorCode::WrongPassword, "the password is incorrect");
                self.document_error(&err);
                return Err(err);
            }
        }
        self.pending_load = Some(PendingLoad { id, owner_password: password, started: self.now });
        self.poll_load();
        Ok(None)
    }

    /// Answers the pending load once pdfcraft has opened the document.
    fn poll_load(&mut self) {
        let Some(pending) = self.pending_load.as_ref() else { return };
        let Some(doc_id) = self.active() else {
            if self.now - pending.started > LOAD_TIMEOUT_SECS {
                let id = pending.id;
                self.pending_load = None;
                let err = RpcError::new(ErrorCode::LoadFailed, "the document did not open");
                self.document_error(&err);
                self.bridge.reply_err(id, &err);
            }
            return;
        };
        let Some(pending) = self.pending_load.take() else { return };
        // The owner password lifts restrictions now, so the host need not call unlock().
        let mut unlocked = false;
        if let Some(pw) = pending.owner_password.as_deref()
            && self.app.session.get(doc_id).is_some_and(|d| d.can_unlock())
        {
            unlocked = self.app.session.unlock(doc_id, pw).is_ok();
            if !unlocked {
                self.bridge.event("error", &json!({ "code": "WRONG_PASSWORD", "message": "the owner password is incorrect; the document stays restricted" }));
            }
        }
        if let Some(author) = self.author.clone() {
            self.app.comment_prefs.author = author;
        }
        let Some(d) = self.app.session.get(doc_id) else { return };
        let info = json!({ "pageCount": doc::page_count(d), "permissions": doc::permissions(d) });
        self.last_dirty = self.app.first_dirty().is_some();
        self.last_comments = Some((doc_id, d.edit_generation(), comments_fingerprint(d)));
        let mut result = info.clone();
        if let Some(obj) = result.as_object_mut() {
            obj.insert("unlocked".into(), json!(unlocked));
        }
        self.bridge.reply_ok(pending.id, Payload::Json(result));
        self.bridge.event("documentLoaded", &info);
    }

    fn document_error(&self, e: &RpcError) {
        self.bridge.event("documentError", &json!({ "code": e.code.as_str(), "message": e.message }));
    }

    /// `dirtyChanged`, `annotationsChanged` and `saveRequested`.
    fn emit_changes(&mut self) {
        if self.pending_load.is_some() {
            return;
        }
        let dirty = self.app.first_dirty().is_some();
        if dirty != self.last_dirty {
            self.last_dirty = dirty;
            self.bridge.event("dirtyChanged", &json!({ "dirty": dirty }));
        }
        if let Some(doc_id) = self.active()
            && let Some(d) = self.app.session.get(doc_id)
        {
            let generation = d.edit_generation();
            let changed = match self.last_comments {
                Some((id, g, _)) if id == doc_id && g == generation => None,
                Some((id, _, fp)) if id == doc_id => Some((fp, comments_fingerprint(d))),
                _ => Some((u64::MAX, comments_fingerprint(d))),
            };
            if let Some((old, new)) = changed {
                self.last_comments = Some((doc_id, generation, new));
                if old != new && old != u64::MAX {
                    self.bridge.event("annotationsChanged", &json!({ "count": top_level_comments(d) }));
                }
            }
        }
        let saves: Vec<(String, Vec<u8>, bool)> = self.saves.try_borrow_mut().map(|mut s| std::mem::take(&mut *s)).unwrap_or_default();
        for (name, bytes, save_as) in saves {
            self.bridge.event_with_bytes("saveRequested", &json!({ "name": name, "saveAs": save_as }), &bytes);
        }
    }
}

/// Top-level comments (replies excluded), as `getAnnotations()` counts them.
fn top_level_comments(d: &pdfcraft_engine::Document) -> usize {
    let names: std::collections::HashSet<&str> = d.info.annotations.iter().filter_map(|a| a.name.as_deref()).collect();
    d.info.annotations.iter().filter(|a| a.in_reply_to.as_deref().is_none_or(|p| !names.contains(p))).count()
}

/// Changes when a comment is added, removed, moved or edited.
fn comments_fingerprint(d: &pdfcraft_engine::Document) -> u64 {
    let mut h = DefaultHasher::new();
    for a in &d.info.annotations {
        (a.page, a.index, &a.subtype, &a.name, &a.author, &a.contents, &a.modified, &a.in_reply_to, &a.state).hash(&mut h);
        for v in a.rect.iter().chain(a.color.iter().flatten()) {
            v.to_bits().hash(&mut h);
        }
    }
    h.finish()
}

impl eframe::App for Viewer {
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        self.app.raw_input_hook(ctx, raw);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.app.save(storage);
    }

    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.app.logic(ctx, frame);
        self.pump(ctx);
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.app.ui(ui, frame);
    }
}
