//! End-to-end checks of the SDK's document operations on pdfcraft's engine: XFDF in and
//! out, the owner-password unlock flow on an AES-256 file, and delete/replace semantics.

// Test helpers outside #[test] functions may fail loudly too.
#![allow(clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use fps_core::ErrorCode;
use fps_core::doc;
use fps_core::xfdf::{self, AnnotationInfo};
use fps_tests::{OWNER_PASSWORD, fixture_text, plain_pdf, protected_owner_only};
use pdfcraft_engine::{DocId, Session};

fn open(bytes: Vec<u8>) -> (Session, DocId) {
    let mut s = Session::new();
    let id = s.open("t.pdf", None, Arc::new(bytes), None).expect("opens");
    (s, id)
}

/// What the viewer does for importXfdf: check, delete for `replace`, import, count.
fn import(s: &mut Session, id: DocId, xfdf: &str, replace: bool) -> Result<usize, fps_core::RpcError> {
    let plan = doc::plan_import(s.get(id).expect("doc"), xfdf)?;
    let before = if replace {
        delete_all(s, id)?;
        Vec::new()
    } else {
        doc::annotations(s, id)?
    };
    if let Some(edit) = plan.edit {
        doc::apply(s, id, edit)?;
    }
    let after = doc::annotations(s, id)?;
    Ok(doc::count_imported(&after, &before, &plan.expected))
}

fn getter(id: DocId) -> impl for<'a> Fn(&'a Session) -> Option<&'a pdfcraft_engine::Document> {
    move |s| s.get(id)
}

/// What the viewer does for deleteAllAnnotations.
fn delete_all(s: &mut Session, id: DocId) -> Result<usize, fps_core::RpcError> {
    doc::delete_all(s, getter(id), |s, e| doc::apply(s, id, e))
}

fn by_id<'a>(list: &'a [AnnotationInfo], id: &str) -> &'a AnnotationInfo {
    list.iter().find(|a| a.id == id).unwrap_or_else(|| panic!("{id} listed"))
}

#[test]
fn fixtures_on_disk_match_the_generators() {
    for (name, bytes) in fps_tests::pdf_fixtures().expect("fixtures build") {
        let on_disk = std::fs::read(fps_tests::fixtures_dir().join(name)).unwrap_or_default();
        assert!(on_disk == bytes, "tests/fixtures/{name} is stale: run `cargo run -p fps-tests --bin make-fixtures`");
    }
}

#[test]
fn review_xfdf_round_trips_with_the_same_count_authors_and_pages() {
    let source = fixture_text("review.xfdf");
    let expected = xfdf::threads(&xfdf::read(&source).expect("fixture parses"));
    assert_eq!(expected.len(), 6, "5 shapes and 1 note; its 2 replies are not counted");

    let (mut s, id) = open(plain_pdf(3));
    assert_eq!(import(&mut s, id, &source, false), Ok(6));
    let listed = doc::annotations(&s, id).expect("lists");
    assert_eq!(listed.len(), 6, "exact count: {listed:#?}");
    for e in &expected {
        let got = by_id(&listed, &e.id);
        assert_eq!(got.author, e.author, "{}: author (XFDF title)", e.id);
        assert_eq!(got.page_index, e.page_index, "{}: page", e.id);
        assert_eq!(got.kind, e.kind, "{}: type", e.id);
        assert_eq!(got.contents.as_deref().unwrap_or(""), e.contents.as_deref().unwrap_or(""), "{}: contents", e.id);
        assert_eq!(got.replies.len(), e.replies.len(), "{}: replies", e.id);
    }
    let note = by_id(&listed, "0f8b4c2e-1a2b-4c3d-9e8f-000000000006");
    let reply_authors: Vec<_> = note.replies.iter().filter_map(|r| r.author.as_deref()).collect();
    assert_eq!(reply_authors, ["Daniel Ochieng", "Priya Raman"]);

    // Export: the author is in `title`, and a second viewer reads back the same comments.
    let exported = doc::export_xfdf(&s, id).expect("exports");
    assert!(exported.contains("title=\"Priya Raman\"") && exported.contains("title=\"Daniel Ochieng\""), "{exported}");
    let (mut s2, id2) = open(plain_pdf(3));
    assert_eq!(import(&mut s2, id2, &exported, false), Ok(6));
    let again = doc::annotations(&s2, id2).expect("lists");
    assert_eq!(again.len(), 6);
    for e in &expected {
        assert_eq!(by_id(&again, &e.id).author, e.author);
        assert_eq!(by_id(&again, &e.id).page_index, e.page_index);
    }
}

#[test]
fn markup_and_guest_fixtures_import_exactly() {
    let (mut s, id) = open(plain_pdf(3));
    assert_eq!(import(&mut s, id, &fixture_text("markup.xfdf"), false), Ok(4));
    assert_eq!(import(&mut s, id, &fixture_text("guest.xfdf"), false), Ok(2));
    let listed = doc::annotations(&s, id).expect("lists");
    assert_eq!(listed.len(), 6);
    assert_eq!(listed.iter().filter(|a| a.author.as_deref() == Some("Guest")).count(), 2, "authors are kept as written");
    let kinds: Vec<&str> = listed.iter().map(|a| a.kind.as_str()).collect();
    for k in ["highlight", "underline", "strikeout", "squiggly", "square", "circle"] {
        assert!(kinds.contains(&k), "{k} in {kinds:?}");
    }
}

#[test]
fn the_old_annots_check_is_not_a_count() {
    // pdfcraft always writes <annots>, so `xfdf.includes("<annots")` says nothing.
    let (s, id) = open(plain_pdf(1));
    let exported = doc::export_xfdf(&s, id).expect("exports");
    assert!(exported.contains("<annots"));
    assert_eq!(doc::annotations(&s, id).map(|l| l.len()), Ok(0));
}

#[test]
fn empty_xfdf_imports_nothing_without_an_error() {
    let (mut s, id) = open(plain_pdf(1));
    assert_eq!(import(&mut s, id, &fixture_text("empty.xfdf"), false), Ok(0));
    assert!(!s.get(id).expect("doc").dirty, "nothing changed");
}

#[test]
fn malformed_xfdf_is_a_typed_error() {
    let (mut s, id) = open(plain_pdf(1));
    assert_eq!(import(&mut s, id, "<xfdf><annots><square", false).err().map(|e| e.code), Some(ErrorCode::XfdfInvalid));
    assert_eq!(import(&mut s, id, "%PDF-1.7", false).err().map(|e| e.code), Some(ErrorCode::XfdfInvalid));
}

#[test]
fn replace_and_delete_all() {
    let (mut s, id) = open(plain_pdf(3));
    assert_eq!(import(&mut s, id, &fixture_text("review.xfdf"), false), Ok(6));
    assert_eq!(import(&mut s, id, &fixture_text("guest.xfdf"), true), Ok(2));
    assert_eq!(doc::annotations(&s, id).map(|l| l.len()), Ok(2), "replace removed the review comments");
    assert_eq!(delete_all(&mut s, id), Ok(2));
    assert_eq!(doc::annotations(&s, id).map(|l| l.len()), Ok(0));
    assert!(doc::next_deletion(s.get(id).expect("doc")).is_none());
    assert_eq!(delete_all(&mut s, id), Ok(0));
    // Replies go with their comment.
    assert_eq!(import(&mut s, id, &fixture_text("review.xfdf"), false), Ok(6));
    assert_eq!(delete_all(&mut s, id), Ok(6));
    assert!(s.get(id).expect("doc").info.annotations.is_empty(), "no reply is left behind");
    // Each deletion is one undo step.
    s.undo(id).expect("undo");
    assert_eq!(doc::annotations(&s, id).map(|l| l.len()), Ok(1));
}

/// pdfcraft deletes a comment with its replies wherever they sit in the page's /Annots: a reply
/// stored before an earlier comment shifts that comment's position. Deleting one at a time with
/// fresh positions copes (a batch of positions computed up front deleted the wrong one, or failed).
#[test]
fn delete_all_copes_with_replies_stored_before_their_comment() {
    let xfdf = r#"<?xml version="1.0" encoding="UTF-8"?>
<xfdf xmlns="http://ns.adobe.com/xfdf/"><annots>
<text page="0" rect="10,10,30,30" name="r2" inreplyto="b2" title="R"><contents>reply to b2</contents></text>
<square page="0" rect="40,40,80,80" name="a1" title="A"><contents>first</contents></square>
<circle page="0" rect="100,100,140,140" name="b2" title="B"><contents>second</contents></circle>
</annots></xfdf>"#;
    let (mut s, id) = open(plain_pdf(1));
    assert_eq!(import(&mut s, id, xfdf, false), Ok(2));
    // The old way, positions computed up front and deleted highest first in one batch, fails here.
    let d = s.get(id).expect("doc");
    let mut targets: Vec<(usize, usize)> = d.info.annotations.iter().filter(|a| a.in_reply_to.is_none()).map(|a| (a.page, a.index)).collect();
    targets.sort_unstable_by(|a, b| b.cmp(a));
    let batch = pdfcraft_engine::Edit::Batch {
        label: "old".into(),
        edits: targets.into_iter().map(|(page, index)| pdfcraft_engine::Edit::DeleteAnnotation { page, index }).collect(),
    };
    let old = doc::apply(&mut s, id, batch);
    let left = s.get(id).expect("doc").info.annotations.len();
    assert!(old.is_err() || left > 0, "the up-front batch should not have worked");
    if old.is_ok() {
        s.undo(id).expect("undo");
    }
    assert_eq!(delete_all(&mut s, id), Ok(2));
    assert!(s.get(id).expect("doc").info.annotations.is_empty());
}

#[test]
fn delete_all_deletes_locked_comments_too() {
    let xfdf = r#"<?xml version="1.0" encoding="UTF-8"?>
<xfdf xmlns="http://ns.adobe.com/xfdf/"><annots>
<square page="0" rect="40,40,80,80" name="l1" title="A" flags="print,locked"/>
<circle page="0" rect="100,100,140,140" name="u1" title="B"/>
</annots></xfdf>"#;
    let (mut s, id) = open(plain_pdf(1));
    assert_eq!(import(&mut s, id, xfdf, false), Ok(2));
    assert!(s.get(id).expect("doc").info.annotations.iter().any(|a| a.locked), "the fixture has a locked comment");
    assert_eq!(delete_all(&mut s, id), Ok(2));
    assert!(s.get(id).expect("doc").info.annotations.is_empty());
}

#[test]
fn import_refuses_form_values_and_skips_missing_pages() {
    let (mut s, id) = open(plain_pdf(1));
    let fields = r#"<xfdf xmlns="http://ns.adobe.com/xfdf/"><annots/><fields><field name="Total"><value>0</value></field></fields></xfdf>"#;
    assert_eq!(import(&mut s, id, fields, false).err().map(|e| e.code), Some(ErrorCode::XfdfInvalid));
    let empty_fields = r#"<xfdf xmlns="http://ns.adobe.com/xfdf/"><annots/><fields/></xfdf>"#;
    assert_eq!(import(&mut s, id, empty_fields, false), Ok(0));
    let beyond = r#"<xfdf xmlns="http://ns.adobe.com/xfdf/"><annots>
<square page="0" rect="40,40,80,80" name="p0" title="A"/>
<square page="7" rect="40,40,80,80" name="p7" title="A"/>
</annots></xfdf>"#;
    assert_eq!(import(&mut s, id, beyond, false), Ok(1), "the page-8 comment isn't counted");
    // A bad XFDF with replace deletes nothing.
    assert_eq!(import(&mut s, id, "<xfdf><annots><square", true).err().map(|e| e.code), Some(ErrorCode::XfdfInvalid));
    assert_eq!(doc::annotations(&s, id).map(|l| l.len()), Ok(1));
}

#[test]
fn reimporting_the_same_comments_replaces_them_by_name() {
    let (mut s, id) = open(plain_pdf(3));
    let review = fixture_text("review.xfdf");
    assert_eq!(import(&mut s, id, &review, false), Ok(6));
    assert_eq!(import(&mut s, id, &review, false), Ok(6), "all six are present");
    assert_eq!(doc::annotations(&s, id).map(|l| l.len()), Ok(6), "no duplicates");
}

#[test]
fn owner_protected_aes256_reports_no_commenting_until_unlocked() {
    let bytes = protected_owner_only(3).expect("fixture");
    let (mut s, id) = open(bytes);
    let p = doc::permissions(s.get(id).expect("doc"));
    assert!(p.is_encrypted && !p.can_annotate && p.can_unlock, "{p:?}");
    assert!(p.can_copy, "owner-only protection grants copying");

    // Import needs the permission.
    let err = import(&mut s, id, &fixture_text("review.xfdf"), false).err();
    assert_eq!(err.map(|e| e.code), Some(ErrorCode::NotPermitted));
    // So does drawing.
    assert_eq!(doc::check_can_annotate(s.get(id).expect("doc")).err().map(|e| e.code), Some(ErrorCode::NotPermitted));

    // A wrong owner password is a typed error and changes nothing.
    let wrong = s.unlock(id, "nope").err().map(|e| fps_core::error::from_edit_error(&e).code);
    assert_eq!(wrong, Some(ErrorCode::WrongPassword));
    assert!(!doc::permissions(s.get(id).expect("doc")).can_annotate);

    s.unlock(id, OWNER_PASSWORD).expect("owner password unlocks");
    let p = doc::permissions(s.get(id).expect("doc"));
    assert!(p.can_annotate && p.is_encrypted && !p.can_unlock, "{p:?}");
    assert_eq!(import(&mut s, id, &fixture_text("review.xfdf"), false), Ok(6));

    // The saved file is still protected, and keeps the comments.
    let saved = doc::save_bytes(&s, id).expect("saves");
    assert!(saved.starts_with(b"%PDF"));
    let (s2, id2) = open(saved.to_vec());
    assert!(s2.get(id2).expect("doc").info.encrypted);
    assert!(!doc::permissions(s2.get(id2).expect("doc")).can_annotate, "still owner-protected");
    assert_eq!(doc::annotations(&s2, id2).map(|l| l.len()), Ok(6));
}

#[test]
fn known_losses_are_what_the_docs_say() {
    let (mut s, id) = open(plain_pdf(2));
    assert_eq!(import(&mut s, id, &fixture_text("known-losses.xfdf"), false), Ok(3), "every annotation itself imports");
    let exported = doc::export_xfdf(&s, id).expect("exports");
    // site/src/docs/annotations-and-xfdf.md, "Known losses": pdfcraft's XFDF carries no /Measure
    // dictionary and no appearance streams.
    assert!(!exported.contains("<measure"), "measurement scale is not kept");
    assert!(!exported.contains("<appearance"), "stamp artwork is not kept");
    let listed = doc::annotations(&s, id).expect("lists");
    assert_eq!(listed.iter().find(|a| a.kind == "line").and_then(|a| a.contents.as_deref()), Some("105.83 mm"), "the text survives");
}

/// Real XFDF exported from production (tests/fixtures/real/*.xfdf, git-ignored: never commit
/// customer data). Each must import with the same count, authors and pages it lists, and
/// survive an export and re-import. Skipped when the folder is empty.
#[test]
fn real_xfdf_samples_round_trip() {
    let dir = fps_tests::fixtures_dir().join("real");
    let Ok(entries) = std::fs::read_dir(&dir) else { return };
    let mut checked = 0usize;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("xfdf") {
            continue;
        }
        let source = std::fs::read_to_string(&path).unwrap_or_default();
        let expected = xfdf::threads(&xfdf::read(&source).unwrap_or_else(|e| panic!("{}: {e}", path.display())));
        // Generous page count: real artwork can be long; the fixture's pages are blank.
        let pages = expected.iter().map(|a| (a.page_index as usize).saturating_add(1)).max().unwrap_or(1);
        let (mut s, id) = open(plain_pdf(pages));
        let n = import(&mut s, id, &source, false).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        assert_eq!(n, expected.len(), "{}: imported count", path.display());
        let exported = doc::export_xfdf(&s, id).expect("exports");
        let (mut s2, id2) = open(plain_pdf(pages));
        assert_eq!(import(&mut s2, id2, &exported, false), Ok(expected.len()), "{}: re-import", path.display());
        let again = doc::annotations(&s2, id2).expect("lists");
        for e in &expected {
            let got = by_id(&again, &e.id);
            assert_eq!((&got.author, got.page_index, &got.kind), (&e.author, e.page_index, &e.kind), "{}: {}", path.display(), e.id);
            assert_eq!(got.replies.len(), e.replies.len(), "{}: {} replies", path.display(), e.id);
        }
        checked = checked.saturating_add(1);
    }
    eprintln!("real XFDF samples checked: {checked}");
}

#[test]
fn the_demo_document_has_three_pages_and_its_form() {
    let (s, id) = open(fps_tests::demo_pdf());
    let d = s.get(id).expect("demo opens");
    assert_eq!(d.info.pages.len(), 3);
    let mut names: Vec<&str> = d.info.fields.iter().map(|f| f.name.as_str()).collect();
    names.sort_unstable();
    assert_eq!(names, ["approved", "company", "email", "name", "notes"]);
    assert!(d.allows_annotation() && d.allows_form_filling());
    let layers: Vec<(&str, bool)> = d.info.layers.iter().map(|l| (l.name.as_str(), l.visible)).collect();
    assert_eq!(layers, [("Structure", true), ("Furniture", true), ("Electrical", true), ("Dimensions", true)]);
}
