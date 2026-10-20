//! Reading XFDF: the annotations an XFDF document holds, as the SDK lists them.
//!
//! The listing (`getAnnotations()`) is read from pdfcraft's own XFDF export, so it shows exactly
//! what `exportXfdf()` stores. ComPDF-written XFDF is read the same way (fixtures in tests/).

use serde::Serialize;

use crate::error::{ErrorCode, RpcError};
use crate::protocol::limits::MAX_XFDF_BYTES;

/// XFDF elements that are annotations pdfcraft imports and exports (pdfcraft-xfdf `SUBTYPES`).
pub const ANNOTATION_ELEMENTS: &[&str] = &[
    "text",
    "freetext",
    "line",
    "square",
    "circle",
    "polygon",
    "polyline",
    "highlight",
    "underline",
    "squiggly",
    "strikeout",
    "stamp",
    "caret",
    "ink",
    "fileattachment",
    "sound",
    "redact",
];

/// The deepest element nesting accepted (real XFDF is about 6 deep).
const MAX_DEPTH: usize = 256;

/// One annotation element of an XFDF document.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct XfdfAnnot {
    /// Element name: the lowercase annotation subtype (`square`, `freetext`, …).
    pub element: String,
    pub name: Option<String>,
    pub in_reply_to: Option<String>,
    /// 0-based page index.
    pub page: Option<u32>,
    pub title: Option<String>,
    pub subject: Option<String>,
    pub contents: Option<String>,
    /// `date` as written (a PDF date, `D:YYYYMMDDHHmmSS…`).
    pub date: Option<String>,
    pub creation_date: Option<String>,
    /// `#RRGGBB`.
    pub color: Option<String>,
    /// A review status reply (`state`): not a comment of its own.
    pub state: Option<String>,
}

/// Reads the annotations of an XFDF document, in document order.
pub fn read(xfdf: &str) -> Result<Vec<XfdfAnnot>, RpcError> {
    if xfdf.len() > MAX_XFDF_BYTES {
        return Err(RpcError::new(ErrorCode::TooLarge, "the XFDF is larger than 32 MiB"));
    }
    if nesting_depth(xfdf) > MAX_DEPTH {
        return Err(RpcError::new(ErrorCode::XfdfInvalid, "the XFDF is nested too deeply"));
    }
    let opts = roxmltree::ParsingOptions { allow_dtd: false, ..roxmltree::ParsingOptions::default() };
    let doc = roxmltree::Document::parse_with_options(xfdf, opts).map_err(|e| RpcError::new(ErrorCode::XfdfInvalid, format!("not XFDF: {e}")))?;
    let root = doc.root_element();
    if root.tag_name().name() != "xfdf" {
        return Err(RpcError::new(ErrorCode::XfdfInvalid, "not XFDF: the root element is not <xfdf>"));
    }
    let mut out = Vec::new();
    // Only the first <annots>, as pdfcraft imports it.
    if let Some(annots) = root.children().find(|n| n.is_element() && n.tag_name().name() == "annots") {
        for node in annots.children().filter(roxmltree::Node::is_element) {
            let element = node.tag_name().name().to_ascii_lowercase();
            if !ANNOTATION_ELEMENTS.contains(&element.as_str()) {
                continue;
            }
            out.push(XfdfAnnot {
                name: attr(node, "name"),
                in_reply_to: attr(node, "inreplyto"),
                page: node.attribute("page").and_then(|p| p.trim().parse::<u32>().ok()),
                title: attr(node, "title"),
                subject: attr(node, "subject"),
                contents: contents(node),
                date: attr(node, "date"),
                creation_date: attr(node, "creationdate"),
                color: node.attribute("color").and_then(normalize_color),
                state: attr(node, "state"),
                element,
            });
        }
    }
    Ok(out)
}

/// Whether the XFDF carries form field values (`<fields>` with fields in it). The SDK imports
/// comments only: importing field values would change the form, not the comments.
pub fn has_field_values(xfdf: &str) -> Result<bool, RpcError> {
    let opts = roxmltree::ParsingOptions { allow_dtd: false, ..roxmltree::ParsingOptions::default() };
    let doc = roxmltree::Document::parse_with_options(xfdf, opts).map_err(|e| RpcError::new(ErrorCode::XfdfInvalid, format!("not XFDF: {e}")))?;
    Ok(doc.root_element().children().filter(|c| c.is_element() && c.tag_name().name() == "fields").any(|f| f.children().any(|c| c.is_element())))
}

fn attr(node: roxmltree::Node<'_, '_>, name: &str) -> Option<String> {
    node.attribute(name).map(str::to_owned).filter(|s| !s.is_empty())
}

/// `<contents>` text, or the text of `<contents-richtext>` without its markup.
fn contents(node: roxmltree::Node<'_, '_>) -> Option<String> {
    let child = |name: &str| node.children().find(|c| c.is_element() && c.tag_name().name() == name);
    if let Some(c) = child("contents") {
        let text: String = c.descendants().filter(roxmltree::Node::is_text).filter_map(|t| t.text()).collect();
        return Some(text.replace("\r\n", "\n").replace('\r', "\n"));
    }
    let rich = child("contents-richtext")?;
    let mut text = String::new();
    for d in rich.descendants() {
        if d.is_element() && matches!(d.tag_name().name(), "p" | "br") && !text.is_empty() {
            text.push('\n');
        }
        if let Some(t) = d.text().filter(|_| d.is_text()) {
            text.push_str(t);
        }
    }
    Some(text.trim_end().to_owned())
}

/// `#RRGGBB` in upper case, or `None`.
fn normalize_color(s: &str) -> Option<String> {
    let hex = s.trim().strip_prefix('#')?;
    (hex.len() == 6 && hex.bytes().all(|b| b.is_ascii_hexdigit())).then(|| format!("#{}", hex.to_ascii_uppercase()))
}

/// How deeply elements nest, from a quick scan of the tags (an upper bound is enough).
fn nesting_depth(xml: &str) -> usize {
    let bytes = xml.as_bytes();
    let (mut depth, mut max) = (0usize, 0usize);
    let mut i = 0usize;
    while let Some(&b) = bytes.get(i) {
        if b == b'<' {
            match bytes.get(i.saturating_add(1)) {
                Some(b'/') => depth = depth.saturating_sub(1),
                Some(b'?' | b'!') => {}
                _ => {
                    // Self-closing tags (`<x/>`) don't nest.
                    let end = bytes.get(i..).and_then(|rest| rest.iter().position(|&c| c == b'>')).map(|p| i.saturating_add(p));
                    let self_closing = end.and_then(|e| e.checked_sub(1)).and_then(|e| bytes.get(e)) == Some(&b'/');
                    if !self_closing {
                        depth = depth.saturating_add(1);
                        max = max.max(depth);
                    }
                }
            }
        }
        i = i.saturating_add(1);
    }
    max
}

/// A PDF date (`D:YYYYMMDDHHmmSSOHH'mm'`, every part after the year optional) as ISO 8601
/// (`YYYY-MM-DDTHH:mm:SS+HH:mm`, `Z` for UTC, no offset when the date has none). `None` when
/// it is not a date.
pub fn pdf_date_to_iso(s: &str) -> Option<String> {
    let s = s.trim();
    let s = s.strip_prefix("D:").unwrap_or(s);
    let digits: String = s.chars().take_while(char::is_ascii_digit).collect();
    if digits.len() < 4 {
        return None;
    }
    let part = |from: usize, len: usize, default: u32| -> Option<u32> {
        match digits.get(from..from.checked_add(len)?) {
            Some(p) => p.parse().ok(),
            None => Some(default),
        }
    };
    let (year, month, day) = (part(0, 4, 0)?, part(4, 2, 1)?, part(6, 2, 1)?);
    let (hour, minute, second) = (part(8, 2, 0)?, part(10, 2, 0)?, part(12, 2, 0)?);
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) || hour > 23 || minute > 59 || second > 60 {
        return None;
    }
    let rest = s.get(digits.len()..).unwrap_or("");
    let offset = match rest.chars().next() {
        Some('Z') => "Z".to_owned(),
        Some(sign @ ('+' | '-')) => {
            let o: String = rest.chars().skip(1).filter(char::is_ascii_digit).collect();
            let oh: u32 = o.get(0..2).and_then(|h| h.parse().ok()).unwrap_or(0);
            let om: u32 = o.get(2..4).and_then(|m| m.parse().ok()).unwrap_or(0);
            if oh > 23 || om > 59 {
                return None;
            }
            format!("{sign}{oh:02}:{om:02}")
        }
        _ => String::new(),
    };
    Some(format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}{offset}"))
}

/// A reply in a comment thread.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ReplyInfo {
    pub id: String,
    pub author: Option<String>,
    pub contents: Option<String>,
    pub date: Option<String>,
}

/// An annotation as `getAnnotations()` reports it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnotationInfo {
    pub id: String,
    /// Lowercase subtype, as ComPDF names it: `text` (note), `freetext`, `square`, `circle`,
    /// `line`, `ink`, `polygon`, `polyline`, `highlight`, `underline`, `squiggly`, `strikeout`,
    /// `stamp`, `caret`, `fileattachment`, `sound`, `redact`.
    #[serde(rename = "type")]
    pub kind: String,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub contents: Option<String>,
    /// ISO 8601, from `date` (modified), else `creationdate`.
    pub date: Option<String>,
    /// 0-based.
    pub page_index: u32,
    /// `#RRGGBB`.
    pub color: Option<String>,
    pub replies: Vec<ReplyInfo>,
}

/// Groups XFDF annotations into threads: top-level annotations with their replies.
///
/// Top level: no `inreplyto`, or one naming an annotation that isn't there (an orphan is kept,
/// never dropped). Replies to replies join the thread of their root. Review-status replies
/// (`state`) are not listed as replies.
pub fn threads(items: &[XfdfAnnot]) -> Vec<AnnotationInfo> {
    use std::collections::HashMap;
    let by_name: HashMap<&str, usize> = items.iter().enumerate().filter_map(|(i, a)| a.name.as_deref().map(|n| (n, i))).collect();
    // The top-level ancestor of item i (bounded walk: a reply cycle ends at the item itself).
    let root_of = |mut i: usize| -> usize {
        for _ in 0..items.len() {
            let Some(parent) = items.get(i).and_then(|a| a.in_reply_to.as_deref()).and_then(|p| by_name.get(p)).copied() else { return i };
            if parent == i {
                return i;
            }
            i = parent;
        }
        i
    };
    let is_top = |i: usize| items.get(i).is_some_and(|a| a.in_reply_to.as_deref().is_none_or(|p| !by_name.contains_key(p)));
    let mut out: Vec<AnnotationInfo> = Vec::new();
    let mut slot: HashMap<usize, usize> = HashMap::new();
    for (i, a) in items.iter().enumerate() {
        if is_top(i) {
            slot.insert(i, out.len());
            out.push(AnnotationInfo {
                id: a.name.clone().unwrap_or_else(|| format!("p{}-n{}", a.page.unwrap_or(0), i)),
                kind: a.element.clone(),
                author: a.title.clone(),
                subject: a.subject.clone(),
                contents: a.contents.clone(),
                date: a.date.as_deref().or(a.creation_date.as_deref()).and_then(pdf_date_to_iso),
                page_index: a.page.unwrap_or(0),
                color: a.color.clone(),
                replies: Vec::new(),
            });
        }
    }
    for (i, a) in items.iter().enumerate() {
        if is_top(i) || a.state.is_some() {
            continue;
        }
        let root = root_of(i);
        if let Some(thread) = slot.get(&root).and_then(|&s| out.get_mut(s)) {
            thread.replies.push(ReplyInfo {
                id: a.name.clone().unwrap_or_else(|| format!("r{i}")),
                author: a.title.clone(),
                contents: a.contents.clone(),
                date: a.date.as_deref().or(a.creation_date.as_deref()).and_then(pdf_date_to_iso),
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<xfdf xmlns="http://ns.adobe.com/xfdf/" xml:space="preserve">
  <annots>
    <square page="0" rect="10,10,50,50" color="#ff0000" title="Ann" subject="Rectangle" name="a1" date="D:20240102030405+05'30'"><contents>Box</contents></square>
    <text page="1" rect="1,1,2,2" title="Bob" name="r1" inreplyto="a1" date="D:20240103"><contents>Reply</contents></text>
    <text page="1" rect="1,1,2,2" title="Cy" name="r2" inreplyto="r1"><contents>Reply to reply</contents></text>
    <text page="1" rect="1,1,2,2" title="Dee" name="s1" inreplyto="a1" state="Accepted" statemodel="Review"/>
    <text page="2" rect="1,1,2,2" title="Eve" name="o1" inreplyto="missing"><contents>Orphan</contents></text>
    <freetext page="1" rect="1,1,2,2" title="Fay" name="f1"><contents-richtext><body xmlns="http://www.w3.org/1999/xhtml"><p>Line <b>one</b></p><p>two</p></body></contents-richtext></freetext>
    <popup page="0" rect="1,1,2,2"/>
    <link page="0" rect="1,1,2,2"/>
  </annots>
</xfdf>"##;

    #[test]
    fn reads_threads_with_exact_counts() {
        let items = read(SAMPLE).unwrap_or_default();
        assert_eq!(items.len(), 6, "popup and link are not comments");
        let list = threads(&items);
        assert_eq!(list.len(), 3, "square, orphan reply and free text are top level");
        let first = &list[0];
        assert_eq!((first.id.as_str(), first.kind.as_str(), first.author.as_deref()), ("a1", "square", Some("Ann")));
        assert_eq!(first.color.as_deref(), Some("#FF0000"));
        assert_eq!(first.date.as_deref(), Some("2024-01-02T03:04:05+05:30"));
        assert_eq!(first.replies.len(), 2, "the status reply is not listed");
        assert_eq!(first.replies[1].contents.as_deref(), Some("Reply to reply"));
        assert_eq!(list[1].page_index, 2);
        assert_eq!(list[2].contents.as_deref(), Some("Line one\ntwo"));
    }

    #[test]
    fn rejects_non_xfdf_and_hostile_input() {
        assert_eq!(read("not xml").err().map(|e| e.code), Some(ErrorCode::XfdfInvalid));
        assert_eq!(read("<html/>").err().map(|e| e.code), Some(ErrorCode::XfdfInvalid));
        let dtd = "<?xml version=\"1.0\"?><!DOCTYPE x [<!ENTITY a \"aaaa\">]><xfdf><annots/></xfdf>";
        assert_eq!(read(dtd).err().map(|e| e.code), Some(ErrorCode::XfdfInvalid));
        let deep = format!("<xfdf>{}{}</xfdf>", "<a>".repeat(10_000), "</a>".repeat(10_000));
        assert_eq!(read(&deep).err().map(|e| e.code), Some(ErrorCode::XfdfInvalid));
        assert_eq!(read("<xfdf xmlns=\"http://ns.adobe.com/xfdf/\"><annots/></xfdf>").map(|v| v.len()), Ok(0));
    }

    #[test]
    fn reply_cycles_terminate() {
        let a = XfdfAnnot { element: "text".into(), name: Some("x".into()), in_reply_to: Some("y".into()), ..Default::default() };
        let b = XfdfAnnot { element: "text".into(), name: Some("y".into()), in_reply_to: Some("x".into()), ..Default::default() };
        // Both reply to each other: neither is top level, and nothing hangs.
        assert!(threads(&[a, b]).is_empty());
    }

    #[test]
    fn converts_pdf_dates() {
        assert_eq!(pdf_date_to_iso("D:20240102030405Z").as_deref(), Some("2024-01-02T03:04:05Z"));
        assert_eq!(pdf_date_to_iso("D:20240102030405-08'00'").as_deref(), Some("2024-01-02T03:04:05-08:00"));
        assert_eq!(pdf_date_to_iso("D:2024").as_deref(), Some("2024-01-01T00:00:00"));
        assert_eq!(pdf_date_to_iso("20240102").as_deref(), Some("2024-01-02T00:00:00"));
        assert_eq!(pdf_date_to_iso("D:20241302"), None);
        assert_eq!(pdf_date_to_iso("yesterday"), None);
        assert_eq!(pdf_date_to_iso("D:20240102030405+99'00'"), None);
    }
}
