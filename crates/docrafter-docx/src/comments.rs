//! Word comment parts (`word/comments.xml`).

use docrafter_core::{Error, Result};
use quick_xml::escape::escape;
use quick_xml::events::Event;
use quick_xml::Reader;

/// A document comment (review pane).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocxComment {
    /// Comment id (matches range markers in `document.xml`).
    pub id: u32,
    /// Author display name.
    pub author: String,
    /// Comment body text.
    pub text: String,
}

/// Build `word/comments.xml`.
pub fn build_comments_xml(comments: &[DocxComment]) -> String {
    let mut out = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
"#,
    );
    for c in comments {
        out.push_str(&format!(
            r#"<w:comment w:id="{id}" w:author="{author}" w:initials="DC"><w:p><w:r><w:t xml:space="preserve">{text}</w:t></w:r></w:p></w:comment>"#,
            id = c.id,
            author = escape_text(&c.author),
            text = escape_text(&c.text),
        ));
    }
    out.push_str("</w:comments>");
    out
}

/// XML markers for the first commented paragraph.
pub fn comment_range_markers(id: u32) -> String {
    format!(
        r#"<w:commentRangeStart w:id="{id}"/><w:commentRangeEnd w:id="{id}"/><w:r><w:commentReference w:id="{id}"/></w:r>"#
    )
}

/// Parse `word/comments.xml` into review comments.
pub fn parse_comments_xml(xml: &str) -> Result<Vec<DocxComment>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut comments = Vec::new();
    let mut current: Option<DocxComment> = None;
    let mut in_text = false;
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) if e.local_name().as_ref() == b"comment" => {
                let id = comment_attr(&e, b"id")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(comments.len() as u32);
                let author = comment_attr(&e, b"author").unwrap_or_default();
                current = Some(DocxComment {
                    id,
                    author,
                    text: String::new(),
                });
            }
            Ok(Event::Start(e)) if e.local_name().as_ref() == b"t" && current.is_some() => {
                in_text = true;
            }
            Ok(Event::Text(e)) if in_text => {
                if let Some(c) = current.as_mut() {
                    c.text.push_str(&e.unescape().map_err(xml_err)?);
                }
            }
            Ok(Event::End(e)) if e.local_name().as_ref() == b"t" => {
                in_text = false;
            }
            Ok(Event::End(e)) if e.local_name().as_ref() == b"comment" => {
                if let Some(c) = current.take() {
                    comments.push(c);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(xml_err(e)),
            _ => {}
        }
    }
    Ok(comments)
}

fn comment_attr(e: &quick_xml::events::BytesStart, name: &[u8]) -> Option<String> {
    e.attributes()
        .filter_map(|a| a.ok())
        .find(|a| a.key.local_name().as_ref() == name)
        .map(|a| String::from_utf8_lossy(&a.value).into_owned())
}

fn xml_err(e: impl std::fmt::Display) -> Error {
    Error::Docx(format!("comments XML parse error: {e}"))
}

fn escape_text(s: &str) -> String {
    escape(s).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_build_roundtrip() {
        let comments = vec![
            DocxComment {
                id: 0,
                author: "Reviewer".into(),
                text: "Please check".into(),
            },
            DocxComment {
                id: 1,
                author: "Editor".into(),
                text: "LGTM".into(),
            },
        ];
        let xml = build_comments_xml(&comments);
        let parsed = parse_comments_xml(&xml).unwrap();
        assert_eq!(parsed, comments);
    }
}
