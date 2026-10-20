//! Document operations over a pdfcraft [`Session`]: what the RPC methods read and the edits they
//! make. The viewer applies the edits through `PdfCraftApp::apply_edit` (so the canvas updates);
//! tests apply them to the session directly.

use std::sync::Arc;

use pdfcraft_engine::{DataFormat, DocId, Document, Edit, Session};
use serde::Serialize;

use crate::error::{ErrorCode, RpcError, from_edit_error};
use crate::xfdf::{self, AnnotationInfo};

/// What the document's security allows, as `getPermissions()` reports it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Permissions {
    /// Comments may be added, changed and imported.
    pub can_annotate: bool,
    pub can_print: bool,
    pub can_copy: bool,
    pub is_encrypted: bool,
    /// The owner password would lift restrictions (`unlock`).
    pub can_unlock: bool,
}

/// The permissions of `doc`. Importing XFDF additionally needs pdfcraft's "modify" permission,
/// so `canAnnotate` is true only when both are granted: what the host can rely on.
pub fn permissions(doc: &Document) -> Permissions {
    Permissions {
        can_annotate: doc.allows_annotation() && doc.allows_modification(),
        can_print: doc.allows_printing(),
        can_copy: doc.allows_copying(),
        is_encrypted: doc.info.encrypted,
        can_unlock: doc.can_unlock(),
    }
}

/// The document's pages.
pub fn page_count(doc: &Document) -> usize {
    doc.info.pages.len()
}

/// The document's comments as XFDF (author in `title`, 0-based `page`).
pub fn export_xfdf(session: &Session, id: DocId) -> Result<String, RpcError> {
    let bytes = session.export_data(id, DataFormat::Xfdf, true, false).map_err(|e| from_edit_error(&e))?;
    String::from_utf8(bytes).map_err(|_| RpcError::internal("the XFDF export is not UTF-8"))
}

/// The document's comments, threaded (`getAnnotations()`).
pub fn annotations(session: &Session, id: DocId) -> Result<Vec<AnnotationInfo>, RpcError> {
    let exported = export_xfdf(session, id)?;
    Ok(xfdf::threads(&xfdf::read(&exported)?))
}

/// Refuses a comment change the document's security doesn't allow.
pub fn check_can_annotate(doc: &Document) -> Result<(), RpcError> {
    if !doc.editable() {
        return Err(RpcError::new(ErrorCode::ReadOnly, doc.read_only_reason.clone().unwrap_or_else(|| "this document can't be edited".to_owned())));
    }
    if !doc.allows_annotation() {
        return Err(RpcError::new(
            ErrorCode::NotPermitted,
            "the document's security settings don't allow comments; unlock it with the owner password",
        ));
    }
    Ok(())
}

/// The edit that deletes the next comment (a top-level one: its replies and pop-ups go with it),
/// or `None` when none is left. Apply it, then ask again: pdfcraft removes replies and pop-ups
/// wherever they sit in the page's list, so positions are only valid for one deletion. A locked
/// comment is unlocked first, in the same undo step: the host asked for all of them.
pub fn next_deletion(doc: &Document) -> Option<Edit> {
    let names: std::collections::HashSet<&str> = doc.info.annotations.iter().filter_map(|a| a.name.as_deref()).collect();
    let target =
        doc.info.annotations.iter().filter(|a| a.in_reply_to.as_deref().is_none_or(|p| !names.contains(p))).max_by_key(|a| (a.page, a.index))?;
    let (page, index) = (target.page, target.index);
    let delete = Edit::DeleteAnnotation { page, index };
    Some(if target.locked {
        Edit::Batch { label: "Delete comment".to_owned(), edits: vec![Edit::LockAnnotation { page, index, locked: false }, delete] }
    } else {
        delete
    })
}

/// Deletes every comment of the document `doc` finds in `state`, one [`next_deletion`] at a time
/// through `apply`. Returns how many comments were deleted. Ends with an error when a deletion
/// doesn't shrink the list, so it always ends.
pub fn delete_all<S>(
    state: &mut S,
    doc: impl Fn(&S) -> Option<&Document>,
    mut apply: impl FnMut(&mut S, Edit) -> Result<(), RpcError>,
) -> Result<usize, RpcError> {
    let mut deleted = 0usize;
    loop {
        let (before, edit) = {
            let d = doc(state).ok_or_else(RpcError::no_document)?;
            (d.info.annotations.len(), next_deletion(d))
        };
        let Some(edit) = edit else { return Ok(deleted) };
        apply(state, edit)?;
        let after = doc(state).map_or(0, |d| d.info.annotations.len());
        if after >= before {
            return Err(RpcError::internal("a comment could not be deleted"));
        }
        deleted = deleted.saturating_add(1);
    }
}

/// The edit `importXfdf` makes, with what it expects to import. `None` when the XFDF holds no
/// annotation (pdfcraft refuses an import that imports nothing; the SDK reports 0 instead).
pub struct ImportPlan {
    pub edit: Option<Edit>,
    /// Top-level annotations in the XFDF (what `importXfdf` should report).
    pub expected: Vec<xfdf::XfdfAnnot>,
}

/// Plans an XFDF import into `doc` (for `replace`, the caller deletes the existing comments
/// first). Checks the XFDF parses, holds comments only, and the document allows comments.
/// Annotations on pages the document doesn't have are not counted: pdfcraft skips them.
pub fn plan_import(doc: &Document, xfdf_text: &str) -> Result<ImportPlan, RpcError> {
    check_can_annotate(doc)?;
    if xfdf::has_field_values(xfdf_text)? {
        return Err(RpcError::new(ErrorCode::XfdfInvalid, "the XFDF holds form field values; importXfdf imports comments only"));
    }
    let pages = page_count(doc);
    let items: Vec<xfdf::XfdfAnnot> =
        xfdf::read(xfdf_text)?.into_iter().filter(|a| a.page.and_then(|p| usize::try_from(p).ok()).is_some_and(|p| p < pages)).collect();
    let top: Vec<xfdf::XfdfAnnot> = {
        let names: std::collections::HashSet<&str> = items.iter().filter_map(|a| a.name.as_deref()).collect();
        items.iter().filter(|a| a.in_reply_to.as_deref().is_none_or(|p| !names.contains(p))).cloned().collect()
    };
    // pdfcraft imports data under its "modify" permission (bit 4), not only "annotate" (bit 6).
    if !items.is_empty() && !doc.allows_modification() {
        return Err(RpcError::new(
            ErrorCode::NotPermitted,
            "the document's security settings don't allow importing comments; unlock it with the owner password",
        ));
    }
    let edit = (!items.is_empty()).then(|| Edit::ImportData { name: "annotations.xfdf".to_owned(), bytes: Arc::new(xfdf_text.as_bytes().to_vec()) });
    Ok(ImportPlan { edit, expected: top })
}

/// How many of the planned top-level annotations are in the document now.
pub fn count_imported(after: &[AnnotationInfo], before: &[AnnotationInfo], expected: &[xfdf::XfdfAnnot]) -> usize {
    let present: std::collections::HashSet<&str> = after.iter().map(|a| a.id.as_str()).collect();
    let named = expected.iter().filter(|a| a.name.as_deref().is_some_and(|n| present.contains(n))).count();
    let unnamed_expected = expected.iter().filter(|a| a.name.is_none()).count();
    let unnamed_new = after.len().saturating_sub(before.len()).saturating_sub(named);
    named.saturating_add(unnamed_new.min(unnamed_expected))
}

/// Applies `edit` to the session (tests and tools; the viewer uses the app).
pub fn apply(session: &mut Session, id: DocId, edit: Edit) -> Result<(), RpcError> {
    session.apply(id, edit).map_err(|e| from_edit_error(&e))
}

/// The saved PDF with every change (incremental where pdfcraft can).
pub fn save_bytes(session: &Session, id: DocId) -> Result<Arc<Vec<u8>>, RpcError> {
    session.save_bytes(id).map_err(|e| from_edit_error(&e))
}

/// Maps an open error.
pub fn from_open_error(e: &pdfcraft_render::OpenError) -> RpcError {
    use pdfcraft_render::OpenError as O;
    match e {
        O::NeedsPassword => RpcError::new(ErrorCode::NeedsPassword, "the document needs a password to open"),
        O::WrongPassword => RpcError::new(ErrorCode::WrongPassword, "the password is incorrect"),
        other => RpcError::new(ErrorCode::LoadFailed, other.to_string()),
    }
}
