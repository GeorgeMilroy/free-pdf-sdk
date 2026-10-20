//! `disabledElements`: the documented ids a host may hide, and the pdfcraft command and tool-group
//! ids each one stands for. The viewer hands the expanded set to pdfcraft's UI profile.

/// A documented element id and what it hides in pdfcraft: command ids (`file.save`) and tool
/// group ids prefixed `group:` (`group:export`).
pub struct Element {
    pub id: &'static str,
    pub hides: &'static [&'static str],
}

/// Every id `disabledElements` accepts (docs/api.md lists the same table).
pub const ELEMENTS: &[Element] = &[
    Element {
        id: "download",
        hides: &[
            "file.save_as",
            "comment.export",
            "form.export_data",
            "measure.export",
            "export.docx",
            "export.html",
            "export.image",
            "export.pptx",
            "export.rtf",
            "export.text",
            "export.all_images",
            "group:export",
        ],
    },
    Element { id: "save", hides: &["file.save", "file.save_as"] },
    Element { id: "print", hides: &["print.dialog"] },
    Element {
        id: "open",
        hides: &["file.open", "file.open_recent", "file.pin_folder", "file.close", "file.close_all", "create.blank", "group:create", "group:combine"],
    },
    Element { id: "flatten", hides: &["comment.flatten", "form.flatten"] },
    Element { id: "settings", hides: &["app.preferences"] },
    Element { id: "import", hides: &["comment.import", "form.import_data"] },
    Element { id: "tool:note", hides: &["comment.note"] },
    Element { id: "tool:freetext", hides: &["comment.freetext"] },
    Element { id: "tool:callout", hides: &["comment.callout"] },
    Element { id: "tool:highlight", hides: &["comment.highlight"] },
    Element { id: "tool:underline", hides: &["comment.underline"] },
    Element { id: "tool:strikeout", hides: &["comment.strikeout"] },
    Element { id: "tool:squiggly", hides: &["comment.squiggly"] },
    Element { id: "tool:caret", hides: &["comment.caret"] },
    Element { id: "tool:replace", hides: &["comment.replace"] },
    Element { id: "tool:ink", hides: &["comment.ink", "comment.eraser"] },
    Element { id: "tool:line", hides: &["comment.line"] },
    Element { id: "tool:arrow", hides: &["comment.arrow"] },
    Element { id: "tool:square", hides: &["comment.square"] },
    Element { id: "tool:circle", hides: &["comment.circle"] },
    Element { id: "tool:polygon", hides: &["comment.polygon"] },
    Element { id: "tool:polyline", hides: &["comment.polyline"] },
    Element { id: "tool:cloud", hides: &["comment.cloud"] },
    Element { id: "tool:attach", hides: &["comment.attach"] },
    Element { id: "tool:stamp", hides: &["comment.stamp", "group:stamp"] },
    Element { id: "tool:image", hides: &["edit.image"] },
    Element { id: "tool:link", hides: &["edit.link", "edit.links_from_urls"] },
    Element {
        id: "tool:measure",
        hides: &[
            "measure.distance",
            "measure.perimeter",
            "measure.area",
            "measure.scale",
            "measure.info",
            "measure.snap",
            "measure.export",
            "group:measure",
        ],
    },
    Element {
        id: "tool:fillsign",
        hides: &[
            "sign.fill.text",
            "sign.fill.check",
            "sign.fill.cross",
            "sign.fill.dot",
            "sign.fill.line",
            "sign.fill.date",
            "sign.fill.signature",
            "sign.fill.initials",
            "group:fill_sign",
        ],
    },
    Element { id: "tool:signature", hides: &["sign.fill.signature", "sign.fill.initials"] },
];

/// The element with this id.
pub fn find(id: &str) -> Option<&'static Element> {
    ELEMENTS.iter().find(|e| e.id == id)
}

/// The pdfcraft ids hidden by `ids` (each must be a documented id), sorted and without repeats.
pub fn expand<'a>(ids: impl IntoIterator<Item = &'a str>) -> Result<Vec<&'static str>, String> {
    let mut out: Vec<&'static str> = Vec::new();
    for id in ids {
        let Some(e) = find(id) else { return Err(format!("unknown element id {id:?}")) };
        out.extend_from_slice(e.hides);
    }
    out.sort_unstable();
    out.dedup();
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique_and_documented_ones_exist() {
        let mut ids: Vec<&str> = ELEMENTS.iter().map(|e| e.id).collect();
        let n = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), n, "element ids are unique");
        for id in [
            "download",
            "print",
            "save",
            "open",
            "flatten",
            "settings",
            "tool:highlight",
            "tool:underline",
            "tool:strikeout",
            "tool:squiggly",
            "tool:stamp",
            "tool:image",
            "tool:link",
        ] {
            assert!(find(id).is_some(), "{id} is documented");
        }
    }

    #[test]
    fn expand_merges_and_rejects_unknown_ids() {
        let hidden = expand(["save", "download"]).unwrap_or_default();
        assert!(hidden.contains(&"file.save") && hidden.contains(&"file.save_as") && hidden.contains(&"group:export"));
        assert_eq!(hidden.iter().filter(|h| **h == "file.save_as").count(), 1);
        assert!(expand(["nope"]).is_err());
        assert_eq!(expand([]).unwrap_or_default(), Vec::<&str>::new());
    }
}
