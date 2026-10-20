//! What each mode hides, before the host's `disabledElements` (pdfcraft UI profile ids).

use crate::elements;
use crate::protocol::Mode;

/// Tool groups annotate mode keeps (everything else in "All tools" is hidden).
pub const ANNOTATE_GROUPS: &[&str] = &["comment", "fill_sign", "measure", "stamp"];

/// Hidden in every mode: what makes no sense in a viewer embedded in a page (opening, closing
/// and creating files, update checks, document compare, which the SDK does not offer).
pub const ALWAYS_HIDDEN: &[&str] =
    &["help.check_updates", "file.revert", "group:compare", "help.discord", "help.app_page", "help.github", "help.website"];

/// Hidden in view mode besides what the read-only lock refuses: saving (nothing can change).
pub const VIEW_HIDDEN: &[&str] = &["file.save", "file.save_as"];

/// The pdfcraft ids hidden for `mode` and the documented `disabled` elements (unknown ids are
/// ignored: the protocol validated them).
pub fn hidden_ids(mode: Mode, disabled: &[String]) -> Vec<&'static str> {
    let mut out: Vec<&'static str> = ALWAYS_HIDDEN.to_vec();
    if let Some(open) = elements::find("open") {
        out.extend_from_slice(open.hides);
    }
    if mode == Mode::View {
        out.extend_from_slice(VIEW_HIDDEN);
    }
    for id in disabled {
        if let Some(e) = elements::find(id) {
            out.extend_from_slice(e.hides);
        }
    }
    out.sort_unstable();
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_hides_saving_and_every_mode_hides_opening() {
        let view = hidden_ids(Mode::View, &[]);
        assert!(view.contains(&"file.save") && view.contains(&"file.open") && view.contains(&"group:compare"));
        let annotate = hidden_ids(Mode::Annotate, &[]);
        assert!(!annotate.contains(&"file.save"), "annotate keeps Save (it hands the PDF to the host)");
        assert!(annotate.contains(&"file.open"));
    }

    #[test]
    fn disabled_elements_add_their_ids() {
        let ids = hidden_ids(Mode::Annotate, &["print".into(), "tool:stamp".into(), "nope".into()]);
        assert!(ids.contains(&"print.dialog") && ids.contains(&"comment.stamp") && ids.contains(&"group:stamp"));
        let mut sorted = ids.clone();
        sorted.dedup();
        assert_eq!(sorted.len(), ids.len());
    }
}
