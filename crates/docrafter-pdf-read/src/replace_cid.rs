//! Replace text in docrafter-generated PDFs (Identity-H / DejaVu CID strings).

use docrafter_core::{Error, Result};
use docrafter_font::{dejavu_sans_bold_bytes, dejavu_sans_regular_bytes, ParsedFace};
use lopdf::{content::Content, Document, Object};

use crate::pages::validate_page_numbers;

struct CidCodec {
    regular: ParsedFace,
    bold: ParsedFace,
}

impl CidCodec {
    fn new() -> Result<Self> {
        Ok(Self {
            regular: ParsedFace::parse(dejavu_sans_regular_bytes())?,
            bold: ParsedFace::parse(dejavu_sans_bold_bytes())?,
        })
    }

    fn decode(&self, bytes: &[u8]) -> String {
        decode_cid_bytes(&self.regular, bytes)
    }

    fn encode(&self, text: &str, bold: bool) -> Vec<u8> {
        if bold {
            self.bold.encode_cid(text)
        } else {
            self.regular.encode_cid(text)
        }
    }
}

fn decode_cid_bytes(face: &ParsedFace, bytes: &[u8]) -> String {
    let mut out = String::new();
    for chunk in bytes.chunks_exact(2) {
        let gid = u16::from_be_bytes([chunk[0], chunk[1]]);
        if let Some(ch) = face.char_from_glyph_id(gid) {
            out.push(ch);
        }
    }
    out
}

/// Replace `from` with `to` in CID-encoded `Tj`/`TJ` strings (docrafter PDFs).
pub fn replace_text_on_page_cid(doc: &mut Document, page: u32, from: &str, to: &str) -> Result<()> {
    if from.is_empty() {
        return Err(Error::InvalidInput(
            "replace_text: empty search string".into(),
        ));
    }
    validate_page_numbers(doc, &[page])?;
    let codec = CidCodec::new()?;
    let page_id = doc
        .page_iter()
        .nth(page.saturating_sub(1) as usize)
        .ok_or_else(|| Error::Pdf(format!("page {page} not found")))?;
    let content_data = doc
        .get_page_content(page_id)
        .map_err(|e| Error::Pdf(format!("read page content: {e}")))?;
    let mut content = Content::decode(&content_data)
        .map_err(|e| Error::Pdf(format!("decode content page {page}: {e}")))?;
    let mut current_bold = false;
    let mut changed = false;
    for operation in &mut content.operations {
        match operation.operator.as_str() {
            "Tf" => {
                if let Some(Object::Name(name)) = operation.operands.first() {
                    current_bold = name == b"F2";
                }
            }
            "Tj" => {
                if replace_in_operand(&codec, &mut operation.operands, from, to, current_bold)? {
                    changed = true;
                }
            }
            "TJ" => {
                for operand in &mut operation.operands {
                    if let Object::Array(items) = operand {
                        for item in items.iter_mut() {
                            if let Object::String(bytes, _) = item {
                                if replace_bytes(&codec, bytes, from, to, current_bold)? {
                                    changed = true;
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    if !changed {
        return Err(Error::Pdf(format!(
            "replace_text: '{from}' not found on page {page}"
        )));
    }
    let modified = content
        .encode()
        .map_err(|e| Error::Pdf(format!("encode content page {page}: {e}")))?;
    doc.change_page_content(page_id, modified)
        .map_err(|e| Error::Pdf(format!("write content page {page}: {e}")))?;
    Ok(())
}

fn replace_in_operand(
    codec: &CidCodec,
    operands: &mut [Object],
    from: &str,
    to: &str,
    bold: bool,
) -> Result<bool> {
    let Some(Object::String(bytes, _)) = operands.first_mut() else {
        return Ok(false);
    };
    replace_bytes(codec, bytes, from, to, bold)
}

fn replace_bytes(
    codec: &CidCodec,
    bytes: &mut Vec<u8>,
    from: &str,
    to: &str,
    bold: bool,
) -> Result<bool> {
    let decoded = codec.decode(bytes);
    if !decoded.contains(from) {
        return Ok(false);
    }
    let new_text = decoded.replace(from, to);
    *bytes = codec.encode(&new_text, bold);
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use docrafter_pdf_write::{Paragraph, PdfDocument};

    #[test]
    fn replaces_cid_text_on_generated_pdf() {
        let bytes = PdfDocument::new()
            .push(Paragraph::new("Hello world"))
            .to_bytes()
            .unwrap();
        let mut doc = Document::load_mem(&bytes).unwrap();
        replace_text_on_page_cid(&mut doc, 1, "world", "docrafter").unwrap();
        let text = doc.extract_text(&[1]).unwrap();
        assert!(text.contains("docrafter"), "got: {text}");
    }
}
